use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{global_twin_id, shared_lib, upsert_tool, wasm_plugin, GatewayState, ToolSchema};

/// Manifest format for dropped plugins
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub plugin: PluginInfo,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,

    /// For `binary` plugins, path (relative to plugin folder) to the executable.
    #[serde(default = "default_binary_path")]
    pub binary_path: Option<String>,

    /// For `shared_lib` plugins: path (relative to plugin folder) to the library (.so/.dylib/.dll).
    #[serde(default)]
    pub lib_path: Option<String>,

    /// For `wasm` plugins: path (relative to plugin folder) to the module (.wasm).
    #[serde(default)]
    pub wasm_path: Option<String>,

    /// For `component_wasm` plugins: path (relative to plugin folder) to the component (.wasm).
    #[serde(default)]
    pub wasm_component_path: Option<String>,

    /// For `shared_lib`/`config_only` plugins that are callable over HTTP, base URL to reach the plugin.
    /// Example: http://host.docker.internal:9001
    #[serde(default)]
    pub plugin_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    Binary,     // External executable that self-registers via HTTP
    SharedLib,  // .so/.dll loaded via libloading (future)
    ConfigOnly, // Pure data (e.g., static KB)
    Wasm,        // WebAssembly module loaded via Wasmer
    ComponentWasm, // WASI Component Model module loaded via Wasmtime
}

#[cfg(all(target_os = "linux", feature = "seccomp"))]
fn seccomp_enabled() -> bool {
    std::env::var("PAGI_PLUGIN_SECCOMP")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true"
}

fn plugin_signature_mode() -> SignatureMode {
    match std::env::var("PAGI_PLUGIN_VERIFY_SIGNATURES")
        .unwrap_or_else(|_| "off".to_string())
        .to_lowercase()
        .as_str()
    {
        "true" | "strict" => SignatureMode::Strict,
        "best_effort" | "best-effort" => SignatureMode::BestEffort,
        _ => SignatureMode::Off,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignatureMode {
    Off,
    BestEffort,
    Strict,
}

fn cosign_pubkey_path() -> Option<PathBuf> {
    std::env::var("PAGI_PLUGIN_COSIGN_PUBKEY").ok().map(PathBuf::from)
}

async fn verify_cosign_blob(pubkey: &Path, blob: &Path, sig: &Path) -> Result<(), String> {
    let st = tokio::process::Command::new("cosign")
        .arg("verify-blob")
        .arg("--key")
        .arg(pubkey)
        .arg("--signature")
        .arg(sig)
        .arg(blob)
        .status()
        .await
        .map_err(|e| e.to_string())?;

    if st.success() {
        Ok(())
    } else {
        Err(format!("cosign verify-blob failed: {st}"))
    }
}

#[cfg(not(all(target_os = "linux", feature = "seccomp")))]
#[allow(dead_code)]
fn seccomp_enabled() -> bool {
    false
}

/// Phase 5: best-effort sandbox for spawned *binary* plugins.
///
/// Design:
/// - Default-allow filter (to avoid breaking normal plugins)
/// - Explicitly deny a small set of highly dangerous syscalls
/// - Applied only in the child process (via `pre_exec`) when `PAGI_PLUGIN_SECCOMP=true`
#[cfg(all(target_os = "linux", feature = "seccomp"))]
fn apply_seccomp_deny_dangerous() -> Result<(), std::io::Error> {
    use libscmp::{resolve_syscall_name, Action, Filter};

    // Ensure the child can't gain privileges via execve of setuid binaries.
    unsafe {
        let rc = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    let mut filter = Filter::new(Action::Allow)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // Deny list: prioritize protection over completeness.
    // If a syscall isn't present on this kernel/arch, skip it.
    let deny = [
        "ptrace",
        "kexec_load",
        "kexec_file_load",
        "reboot",
        "mount",
        "umount2",
        "pivot_root",
        "swapon",
        "swapoff",
        "init_module",
        "finit_module",
        "delete_module",
        "bpf",
        "perf_event_open",
        "keyctl",
        "add_key",
        "request_key",
        "unshare",
        "setns",
    ];

    for name in deny {
        let Some(syscall) = resolve_syscall_name(name) else {
            continue;
        };
        let _ = filter.add_rule(Action::Errno(libc::EPERM), syscall, &[]);
    }

    filter
        .load()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(())
}

fn default_binary_path() -> Option<String> {
    None
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub parameters: serde_json::Value,
}

/// Spawns the auto-discovery task.
///
/// Notes:
/// - `binary` plugins are *spawned* (best-effort) and expected to self-register via HTTP.
/// - `shared_lib`/`config_only` plugins are registered from their manifest into Redis directly.
pub async fn spawn_plugin_watcher(
    plugin_dir: PathBuf,
    state: GatewayState,
    auto_register_global: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !plugin_dir.exists() {
        info!("Plugin directory {:?} does not exist — skipping auto-discovery", plugin_dir);
        return Ok(());
    }

    info!("Starting plugin auto-discovery in {:?}", plugin_dir);

    // Initial scan on startup
    scan_and_register_plugins(&state, &plugin_dir, auto_register_global).await?;

    // Watcher event channel
    let (tx, mut rx) = mpsc::channel::<notify::Result<notify::Event>>(32);

    // Build a watcher that forwards events into tokio (notify calls this handler on a blocking thread).
    let tx_handler = tx.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx_handler.blocking_send(res);
        },
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )?;

    watcher.watch(&plugin_dir, RecursiveMode::Recursive)?;

    let plugin_dir_clone = plugin_dir.clone();
    let state_clone = state.clone();

    tokio::spawn(async move {
        // Keep watcher alive for the lifetime of the task.
        let _watcher = watcher;

        while rx.recv().await.is_some() {
            // Debounce: wait a moment for file ops to settle
            tokio::time::sleep(Duration::from_millis(500)).await;
            while rx.try_recv().is_ok() {}

            if let Err(e) = scan_and_register_plugins(&state_clone, &plugin_dir_clone, auto_register_global).await {
                error!("Error during plugin scan: {e}");
            }
        }
    });

    Ok(())
}

/// Scan the plugin directory and register all valid manifests.
async fn scan_and_register_plugins(
    state: &GatewayState,
    plugin_dir: &Path,
    global_tools: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut registered = 0usize;
    let mut keep_libs: HashSet<PathBuf> = HashSet::new();

    for entry in std::fs::read_dir(plugin_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("manifest.toml");
        if !manifest_path.exists() {
            continue;
        }

        match register_plugin_from_manifest(state, &path, &manifest_path, global_tools, &mut keep_libs).await {
            Ok(count) => registered += count,
            Err(e) => warn!("Failed to register plugin at {:?}: {e}", path),
        }
    }

    // Unload any previously loaded libraries that are no longer present.
    shared_lib::unload_not_in(&keep_libs);

    if registered > 0 {
        info!("Auto-registered {registered} tools from plugins");
    }

    Ok(())
}

/// Register a single plugin from its manifest.
async fn register_plugin_from_manifest(
    state: &GatewayState,
    plugin_path: &Path,
    manifest_path: &Path,
    global_tools: bool,
    keep_libs: &mut HashSet<PathBuf>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // Phase 5: optional signature verification (best-effort/strict) for plugin manifests.
    // This is intentionally *opt-in* and routed through external tooling (cosign) so the
    // core gateway doesn't need to embed heavy crypto stacks.
    let sig_mode = plugin_signature_mode();
    if sig_mode != SignatureMode::Off {
        if let Some(pubkey) = cosign_pubkey_path() {
            let sig_path = manifest_path.with_extension("toml.sig");
            if sig_path.exists() {
                if let Err(err) = verify_cosign_blob(&pubkey, manifest_path, &sig_path).await {
                    if sig_mode == SignatureMode::Strict {
                        return Err(err.into());
                    }
                    warn!(error = %err, "manifest signature verification failed (best-effort)");
                }
            } else if sig_mode == SignatureMode::Strict {
                return Err(format!("missing required manifest signature: {sig_path:?}").into());
            }
        } else if sig_mode == SignatureMode::Strict {
            return Err("PAGI_PLUGIN_VERIFY_SIGNATURES=strict but PAGI_PLUGIN_COSIGN_PUBKEY is not set".into());
        }
    }

    let manifest_str = std::fs::read_to_string(manifest_path)?;
    let manifest: PluginManifest = toml::from_str(&manifest_str)?;

    info!(
        "Discovered plugin: {} v{} ({:?})",
        manifest.plugin.name, manifest.plugin.version, manifest.plugin.plugin_type
    );

    // Handle shared library plugins: load and register tools from exported function
    if manifest.plugin.plugin_type == PluginType::SharedLib {
        if let Some(lib_file) = &manifest.plugin.lib_path {
            let full_lib = plugin_path.join(lib_file);
            if full_lib.exists() {
                let canonical = full_lib.canonicalize().unwrap_or(full_lib.clone());
                keep_libs.insert(canonical.clone());

                let tools = shared_lib::register_tools(&canonical)?;
                let twin_id = if global_tools { global_twin_id() } else { global_twin_id() };

                let mut registered = 0usize;
                for mut tool in tools {
                    // Force sharedlib execution routing.
                    tool.plugin_url = format!("sharedlib://{}", canonical.display());

                    match upsert_tool(state, twin_id, &tool).await {
                        Ok(()) => {
                            info!("Auto-registered sharedlib tool: {}", tool.name);
                            registered += 1;
                        }
                        Err(e) => warn!("Failed to auto-register sharedlib tool {}: {e}", tool.name),
                    }
                }

                return Ok(registered);
            } else {
                warn!("Shared library path {:?} does not exist; skipping", full_lib);
            }
        }

        return Ok(0);
    }

    // Handle Wasm plugins: instantiate module and collect tool registrations via host import.
    if manifest.plugin.plugin_type == PluginType::Wasm {
        if let Some(wasm_file) = &manifest.plugin.wasm_path {
            let full_wasm = plugin_path.join(wasm_file);
            if full_wasm.exists() {
                let canonical = full_wasm.canonicalize().unwrap_or(full_wasm.clone());
                let tools = wasm_plugin::register_tools(&canonical)?;
                let twin_id = if global_tools { global_twin_id() } else { global_twin_id() };

                let mut registered = 0usize;
                for mut tool in tools {
                    tool.plugin_url = format!("wasm://{}", canonical.display());
                    match upsert_tool(state, twin_id, &tool).await {
                        Ok(()) => {
                            info!("Auto-registered wasm tool: {}", tool.name);
                            registered += 1;
                        }
                        Err(e) => warn!("Failed to auto-register wasm tool {}: {e}", tool.name),
                    }
                }

                return Ok(registered);
            } else {
                warn!("Wasm module path {:?} does not exist; skipping", full_wasm);
            }
        }

        return Ok(0);
    }

    // Handle WASI Component Model plugins: register tools from manifest and route execution via wasmtime.
    if manifest.plugin.plugin_type == PluginType::ComponentWasm {
        if let Some(wasm_file) = &manifest.plugin.wasm_component_path {
            let full_wasm = plugin_path.join(wasm_file);
            if full_wasm.exists() {
                let canonical = full_wasm.canonicalize().unwrap_or(full_wasm.clone());
                let plugin_url = format!("wasm-component://{}", canonical.display());

                let twin_id = if global_tools { global_twin_id() } else { global_twin_id() };
                let mut registered = 0usize;

                for tool_def in &manifest.tools {
                    let tool = ToolSchema {
                        name: tool_def.name.clone(),
                        description: tool_def.description.clone(),
                        plugin_url: plugin_url.clone(),
                        endpoint: tool_def.endpoint.clone(),
                        parameters: tool_def.parameters.clone(),
                    };

                    match upsert_tool(state, twin_id, &tool).await {
                        Ok(()) => {
                            info!("Auto-registered component tool: {}", tool.name);
                            registered += 1;
                        }
                        Err(e) => warn!("Failed to auto-register component tool {}: {e}", tool.name),
                    }
                }

                return Ok(registered);
            } else {
                warn!("Component wasm path {:?} does not exist; skipping", full_wasm);
            }
        }

        return Ok(0);
    }

    // Handle binary plugins: spawn if configured
    if manifest.plugin.plugin_type == PluginType::Binary {
        if let Some(binary) = &manifest.plugin.binary_path {
            let full_binary = plugin_path.join(binary);
            if full_binary.exists() {
                let plugin_dir_env = plugin_path.to_path_buf();
                tokio::spawn(async move {
                    let mut cmd = tokio::process::Command::new(full_binary);
                    cmd.env("PLUGIN_DIR", plugin_dir_env);

                    #[cfg(all(target_os = "linux", feature = "seccomp"))]
                    if seccomp_enabled() {
                        // SAFETY: pre_exec runs in the child after fork, before exec.
                        unsafe {
                            cmd.pre_exec(|| apply_seccomp_deny_dangerous());
                        }
                    }

                    let _ = cmd.status().await;
                });

                // Give the binary time to start and self-register.
                tokio::time::sleep(Duration::from_secs(3)).await;
                return Ok(0);
            } else {
                warn!("Binary path {:?} does not exist; skipping spawn", full_binary);
            }
        }

        // If binary plugin has no spawn config, treat it as self-managed.
        return Ok(0);
    }

    let Some(plugin_url) = manifest.plugin.plugin_url.clone() else {
        warn!(
            "Plugin '{}' is {:?} but has no plugin_url; skipping tool registration",
            manifest.plugin.name, manifest.plugin.plugin_type
        );
        return Ok(0);
    };

    let twin_id = if global_tools { global_twin_id() } else { global_twin_id() };

    let mut registered = 0usize;
    for tool_def in manifest.tools {
        let tool = ToolSchema {
            name: tool_def.name.clone(),
            description: tool_def.description,
            plugin_url: plugin_url.clone(),
            endpoint: tool_def.endpoint,
            parameters: tool_def.parameters,
        };

        match upsert_tool(state, twin_id, &tool).await {
            Ok(()) => {
                info!("Auto-registered tool: {}", tool.name);
                registered += 1;
            }
            Err(e) => warn!("Failed to auto-register {}: {e}", tool.name),
        }
    }

    Ok(registered)
}
