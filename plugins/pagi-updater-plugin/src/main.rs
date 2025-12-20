use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use pagi_common::TwinId;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    external_gateway_url: String,
    plugin_url: String,

    github_owner: String,
    github_repo: String,
    github_token: Option<String>,

    core_binary_path: PathBuf,
    core_bin_name: String,

    /// Optional restart command. If absent, uses `core_binary_path`.
    restart_cmd: Option<String>,
    restart_args: Vec<String>,

    /// If set, enforce sha256 verification.
    require_sha256: bool,

    /// If set, attempt cosign verification (best-effort).
    cosign_pubkey_path: Option<PathBuf>,
}

#[derive(Debug, Error)]
enum UpdaterError {
    #[error("github api error: {0}")]
    Github(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("verification failed: {0}")]
    Verification(String),
    #[error("update not applied: {0}")]
    NotApplied(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayRegisterPayload {
    twin_id: Option<TwinId>,
    tool: GatewayToolSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayToolSchema {
    name: String,
    description: String,
    plugin_url: String,
    endpoint: String,
    parameters: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pagi_http::tracing::init("pagi-updater-plugin");

    let external_gateway_url = std::env::var("EXTERNAL_GATEWAY_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8010".to_string());
    let plugin_url = std::env::var("PLUGIN_URL").unwrap_or_else(|_| "http://127.0.0.1:9060".to_string());

    let github_owner = std::env::var("PAGI_UPDATE_GITHUB_OWNER").unwrap_or_else(|_| "your-org".to_string());
    let github_repo = std::env::var("PAGI_UPDATE_GITHUB_REPO").unwrap_or_else(|_| "pagi-core".to_string());
    let github_token = std::env::var("GITHUB_TOKEN").ok();

    let core_bin_name = std::env::var("PAGI_UPDATE_CORE_BIN_NAME").unwrap_or_else(|_| "pagi-core".to_string());
    let core_binary_path = std::env::var("PAGI_UPDATE_CORE_BINARY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("/usr/local/bin/{core_bin_name}")));

    let restart_cmd = std::env::var("PAGI_UPDATE_RESTART_CMD").ok();
    let restart_args = std::env::var("PAGI_UPDATE_RESTART_ARGS")
        .ok()
        .map(split_args)
        .unwrap_or_default();

    let require_sha256 = std::env::var("PAGI_UPDATE_REQUIRE_SHA256")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true";

    let cosign_pubkey_path = std::env::var("PAGI_UPDATE_COSIGN_PUBKEY").ok().map(PathBuf::from);

    // NOTE:
    // - This plugin is the *mutable* component.
    // - The core binary remains immutable; it never replaces itself.
    // - We depend on `self_update` (GitHub Releases backend) as an upgrade path for
    //   more advanced selection logic, but this plugin performs explicit replacement
    //   of the configured `core_binary_path`.
    let _ = self_update::version::bump_is_greater("0.0.0", "0.0.0");

    let state = AppState {
        http: reqwest::Client::new(),
        external_gateway_url,
        plugin_url,
        github_owner,
        github_repo,
        github_token,
        core_binary_path,
        core_bin_name,
        restart_cmd,
        restart_args,
        require_sha256,
        cosign_pubkey_path,
    };

    // Best-effort: register tools with ExternalGateway on startup.
    let st = state.clone();
    tokio::spawn(async move {
        if let Err(err) = register_tools_with_gateway(&st).await {
            error!(error = %err, "failed to register updater tools with ExternalGateway");
        }
    });

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/check_update", post(check_update_handler))
        .route("/apply_update", post(apply_update_handler))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr: SocketAddr = pagi_http::config::bind_addr(([0, 0, 0, 0], 9060).into());
    info!(%addr, "pagi-updater-plugin listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn split_args(raw: String) -> Vec<String> {
    raw.split_whitespace().map(|s| s.to_string()).collect()
}

async fn register_tools_with_gateway(state: &AppState) -> Result<(), String> {
    let gateway = state.external_gateway_url.trim_end_matches('/');
    let register_url = format!("{gateway}/register_tool");

    let tools = vec![
        GatewayToolSchema {
            name: "check_update".to_string(),
            description: "Check GitHub Releases to see if a newer PAGI-Core binary is available".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/check_update".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "core_binary_path": {"type": "string", "description": "Override the configured core binary path"},
                    "current_version": {"type": "string", "description": "Override detected current version"}
                }
            }),
        },
        GatewayToolSchema {
            name: "apply_update".to_string(),
            description: "Download and atomically replace the PAGI-Core binary from GitHub Releases".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/apply_update".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "core_binary_path": {"type": "string"},
                    "expected_version": {"type": "string", "description": "If set, only apply if latest matches this version"},
                    "restart": {"type": "boolean", "default": true}
                }
            }),
        },
        // Backward-compatible alias.
        GatewayToolSchema {
            name: "check_for_updates".to_string(),
            description: "Alias for check_update".to_string(),
            plugin_url: state.plugin_url.clone(),
            endpoint: "/check_update".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "core_binary_path": {"type": "string"},
                    "current_version": {"type": "string"}
                }
            }),
        },
    ];

    for tool in tools {
        let payload = GatewayRegisterPayload { twin_id: None, tool };
        state
            .http
            .post(&register_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
    }

    info!("registered updater tools with ExternalGateway");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CheckUpdateRequest {
    #[serde(default)]
    core_binary_path: Option<String>,
    #[serde(default)]
    current_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct CheckUpdateResponse {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    asset_name: Option<String>,
}

async fn check_update_handler(
    State(state): State<AppState>,
    Json(req): Json<CheckUpdateRequest>,
) -> impl IntoResponse {
    match check_update(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct ApplyUpdateRequest {
    #[serde(default)]
    core_binary_path: Option<String>,
    #[serde(default)]
    expected_version: Option<String>,
    #[serde(default = "default_true")]
    restart: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
struct ApplyUpdateResponse {
    status: String,
    installed_version: Option<String>,
    backup_path: Option<String>,
}

async fn apply_update_handler(
    State(state): State<AppState>,
    Json(req): Json<ApplyUpdateRequest>,
) -> impl IntoResponse {
    match apply_update(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
    }
}

async fn check_update(state: &AppState, req: CheckUpdateRequest) -> Result<CheckUpdateResponse, UpdaterError> {
    let core_path = req
        .core_binary_path
        .map(PathBuf::from)
        .unwrap_or_else(|| state.core_binary_path.clone());
    let current_version = if let Some(v) = req.current_version {
        v
    } else {
        detect_core_version(&core_path).await.unwrap_or_else(|| "0.0.0".to_string())
    };

    let latest = github_latest_release(state).await?;
    let latest_version = normalize_tag_to_version(&latest.tag_name)?;
    let update_available = Version::parse(&latest_version)
        .map_err(|e| UpdaterError::Parse(format!("invalid latest semver '{latest_version}': {e}")))?
        > Version::parse(&current_version)
            .map_err(|e| UpdaterError::Parse(format!("invalid current semver '{current_version}': {e}")))?;

    // Best-effort pick an asset name.
    let asset_name = select_release_asset(&latest, &state.core_bin_name).map(|a| a.name.clone());

    Ok(CheckUpdateResponse {
        current_version,
        latest_version,
        update_available,
        release_url: latest.html_url,
        asset_name,
    })
}

async fn apply_update(state: &AppState, req: ApplyUpdateRequest) -> Result<ApplyUpdateResponse, UpdaterError> {
    let core_path = req
        .core_binary_path
        .map(PathBuf::from)
        .unwrap_or_else(|| state.core_binary_path.clone());

    let latest = github_latest_release(state).await?;
    let latest_version = normalize_tag_to_version(&latest.tag_name)?;
    if let Some(expected) = req.expected_version.as_deref() {
        if expected != latest_version {
            return Err(UpdaterError::NotApplied(format!(
                "latest version is {latest_version}, expected {expected}"
            )));
        }
    }

    let asset = select_release_asset(&latest, &state.core_bin_name)
        .ok_or_else(|| UpdaterError::Github("no suitable release asset found for this platform".to_string()))?;

    let tmp_dir = tempfile::tempdir()?;
    let download_path = tmp_dir.path().join(&asset.name);
    download_asset(state, &asset, &download_path).await?;

    // If the asset is an archive, we expect it to contain the binary `core_bin_name`.
    let extracted_bin = maybe_extract_binary(&download_path, &state.core_bin_name, tmp_dir.path()).await?;
    let bin_path = extracted_bin.unwrap_or(download_path);

    if state.require_sha256 {
        verify_sha256_from_release_assets(state, &latest, &bin_path, &asset.name).await?;
    }

    if let Some(pubkey) = state.cosign_pubkey_path.as_deref() {
        // Best-effort. If cosign is not installed, do not block update.
        if let Err(err) = verify_cosign_from_release_assets(state, &latest, &bin_path, &asset.name, pubkey).await {
            warn!(error = %err, "cosign verification skipped/failed (best-effort)");
        }
    }

    let backup = atomic_replace_executable(&bin_path, &core_path).await?;

    if req.restart {
        if let Err(err) = restart_core_process(state, &core_path).await {
            warn!(error = %err, "core restart failed (best-effort)");
        }
    }

    Ok(ApplyUpdateResponse {
        status: format!("installed {latest_version}"),
        installed_version: Some(latest_version),
        backup_path: backup.map(|p| p.display().to_string()),
    })
}

async fn detect_core_version(core_path: &Path) -> Option<String> {
    if !core_path.exists() {
        return None;
    }
    let out = Command::new(core_path).arg("--version").output().await.ok()?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    extract_first_semver(&s)
}

fn extract_first_semver(s: &str) -> Option<String> {
    // Very small extractor: finds the first token that semver::Version can parse.
    for tok in s
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '(' || c == ')')
        .map(|t| t.trim_matches(|c: char| c == 'v' || c == 'V'))
    {
        if tok.is_empty() {
            continue;
        }
        if Version::parse(tok).is_ok() {
            return Some(tok.to_string());
        }
    }
    None
}

fn normalize_tag_to_version(tag: &str) -> Result<String, UpdaterError> {
    let t = tag.trim();
    let t = t.strip_prefix('v').unwrap_or(t);
    Version::parse(t)
        .map(|v| v.to_string())
        .map_err(|e| UpdaterError::Parse(format!("invalid tag '{tag}': {e}")))
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize, Clone)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

async fn github_latest_release(state: &AppState) -> Result<GithubRelease, UpdaterError> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        state.github_owner, state.github_repo
    );

    let mut req = state
        .http
        .get(url)
        .header("User-Agent", "pagi-updater-plugin")
        .header("Accept", "application/vnd.github+json");

    if let Some(tok) = state.github_token.as_deref() {
        req = req.header("Authorization", format!("Bearer {tok}"));
    }

    let resp = req.send().await?.error_for_status()?;
    resp.json::<GithubRelease>()
        .await
        .map_err(|e| UpdaterError::Github(e.to_string()))
}

fn select_release_asset(release: &GithubRelease, bin_name: &str) -> Option<GithubAsset> {
    // Heuristic selection that works for:
    // - raw binaries
    // - archives (.tar.gz/.zip) containing the binary
    let os = std::env::consts::OS; // linux|macos|windows
    let arch = std::env::consts::ARCH; // x86_64|aarch64|...

    let mut candidates: Vec<&GithubAsset> = release
        .assets
        .iter()
        .filter(|a| {
            let n = a.name.to_lowercase();
            // ignore checksum/signature files
            !n.ends_with(".sha256")
                && !n.contains("checksums")
                && !n.contains("sha256sums")
                && !n.ends_with(".sig")
                && !n.ends_with(".cosign")
        })
        .collect();

    // Prefer exact-ish match: name contains bin + os + arch.
    candidates.sort_by_key(|a| {
        let n = a.name.to_lowercase();
        let mut score = 0i32;
        if n.contains(&bin_name.to_lowercase()) {
            score -= 10;
        }
        if n.contains(os) {
            score -= 5;
        }
        if n.contains(arch) {
            score -= 5;
        }
        // Prefer archives slightly.
        if n.ends_with(".tar.gz") || n.ends_with(".zip") {
            score -= 1;
        }
        score
    });

    candidates.first().cloned().cloned()
}

async fn download_asset(state: &AppState, asset: &GithubAsset, dest: &Path) -> Result<(), UpdaterError> {
    let mut req = state
        .http
        .get(&asset.browser_download_url)
        .header("User-Agent", "pagi-updater-plugin");
    if let Some(tok) = state.github_token.as_deref() {
        req = req.header("Authorization", format!("Bearer {tok}"));
    }

    let resp = req.send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let mut f = fs::File::create(dest).await?;
    f.write_all(&bytes).await?;
    Ok(())
}

async fn maybe_extract_binary(
    downloaded: &Path,
    bin_name: &str,
    out_dir: &Path,
) -> Result<Option<PathBuf>, UpdaterError> {
    let name = downloaded
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_lowercase();

    if name.ends_with(".tar.gz") {
        return extract_tar_gz(downloaded, bin_name, out_dir).await;
    }
    if name.ends_with(".zip") {
        return extract_zip(downloaded, bin_name, out_dir).await;
    }

    Ok(None)
}

fn join_err_to_io(e: tokio::task::JoinError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}

fn anyhow_to_io(e: anyhow::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}

async fn extract_tar_gz(archive: &Path, bin_name: &str, out_dir: &Path) -> Result<Option<PathBuf>, UpdaterError> {
    // Use blocking extraction because tar/flate2 are sync.
    let archive = archive.to_path_buf();
    let out_dir = out_dir.to_path_buf();
    let bin_name = bin_name.to_string();

    let extracted = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<PathBuf>> {
        let f = std::fs::File::open(&archive)?;
        let gz = flate2::read::GzDecoder::new(f);
        let mut tar = tar::Archive::new(gz);
        tar.unpack(&out_dir)?;

        for entry in walkdir::WalkDir::new(&out_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                if entry.file_name().to_string_lossy() == bin_name {
                    return Ok(Some(entry.into_path()));
                }
                if entry.file_name().to_string_lossy() == format!("{bin_name}.exe") {
                    return Ok(Some(entry.into_path()));
                }
            }
        }
        Ok(None)
    })
    .await
    .map_err(join_err_to_io)?
    .map_err(anyhow_to_io)?;

    Ok(extracted)
}

async fn extract_zip(archive: &Path, bin_name: &str, out_dir: &Path) -> Result<Option<PathBuf>, UpdaterError> {
    let archive = archive.to_path_buf();
    let out_dir = out_dir.to_path_buf();
    let bin_name = bin_name.to_string();

    let extracted = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<PathBuf>> {
        let f = std::fs::File::open(&archive)?;
        let mut zip = zip::ZipArchive::new(f)?;
        zip.extract(&out_dir)?;

        for entry in walkdir::WalkDir::new(&out_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                if entry.file_name().to_string_lossy() == bin_name {
                    return Ok(Some(entry.into_path()));
                }
                if entry.file_name().to_string_lossy() == format!("{bin_name}.exe") {
                    return Ok(Some(entry.into_path()));
                }
            }
        }
        Ok(None)
    })
    .await
    .map_err(join_err_to_io)?
    .map_err(anyhow_to_io)?;

    Ok(extracted)
}

async fn verify_sha256_from_release_assets(
    state: &AppState,
    release: &GithubRelease,
    downloaded_bin: &Path,
    asset_name: &str,
) -> Result<(), UpdaterError> {
    // Accept either:
    // - <asset>.sha256 (containing "<sha>  <filename>")
    // - checksums.txt / SHA256SUMS (containing lines "<sha>  <filename>")
    let sha_asset = release
        .assets
        .iter()
        .find(|a| a.name == format!("{asset_name}.sha256"))
        .or_else(|| release.assets.iter().find(|a| a.name.to_lowercase().contains("sha256sums")))
        .or_else(|| release.assets.iter().find(|a| a.name.to_lowercase().contains("checksums")));

    let Some(sha_asset) = sha_asset else {
        return Err(UpdaterError::Verification(
            "sha256 required but no checksum asset found in release".to_string(),
        ));
    };

    let tmp_dir = tempfile::tempdir()?;
    let checksums_path = tmp_dir.path().join(&sha_asset.name);
    let tmp_asset = GithubAsset {
        name: sha_asset.name.clone(),
        browser_download_url: sha_asset.browser_download_url.clone(),
    };
    download_asset(state, &tmp_asset, &checksums_path).await?;

    let mut f = fs::File::open(&checksums_path).await?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await?;
    let content = String::from_utf8_lossy(&buf);

    let expected = parse_sha256_for_asset(&content, asset_name)
        .ok_or_else(|| UpdaterError::Verification("checksum file did not contain expected asset".to_string()))?;

    let actual = sha256_hex_file(downloaded_bin).await?;
    if expected.to_lowercase() != actual.to_lowercase() {
        return Err(UpdaterError::Verification(format!(
            "sha256 mismatch: expected {expected} got {actual}"
        )));
    }

    Ok(())
}

fn parse_sha256_for_asset(content: &str, asset_name: &str) -> Option<String> {
    for line in content.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        // Typical formats:
        // <sha>  <filename>
        // <sha> *<filename>
        let mut parts = l.split_whitespace();
        let sha = parts.next()?;
        let fname = parts.next().unwrap_or("").trim_start_matches('*');
        if fname == asset_name || fname.ends_with(&format!("/{asset_name}")) {
            return Some(sha.to_string());
        }
    }
    None
}

async fn sha256_hex_file(path: &Path) -> Result<String, UpdaterError> {
    let mut f = fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(hex::encode(digest))
}

async fn verify_cosign_from_release_assets(
    state: &AppState,
    release: &GithubRelease,
    downloaded_bin: &Path,
    asset_name: &str,
    pubkey: &Path,
) -> Result<(), UpdaterError> {
    // Expect signature file named <asset>.sig or <asset>.cosign
    let sig_asset = release
        .assets
        .iter()
        .find(|a| a.name == format!("{asset_name}.sig"))
        .or_else(|| release.assets.iter().find(|a| a.name == format!("{asset_name}.cosign")));
    let Some(sig_asset) = sig_asset else {
        return Err(UpdaterError::Verification("no cosign signature asset found".to_string()));
    };

    let tmp_dir = tempfile::tempdir()?;
    let sig_path = tmp_dir.path().join(&sig_asset.name);
    let tmp_asset = GithubAsset {
        name: sig_asset.name.clone(),
        browser_download_url: sig_asset.browser_download_url.clone(),
    };
    download_asset(state, &tmp_asset, &sig_path).await?;

    // Run: cosign verify-blob --key <pubkey> --signature <sig> <blob>
    let status = Command::new("cosign")
        .arg("verify-blob")
        .arg("--key")
        .arg(pubkey)
        .arg("--signature")
        .arg(&sig_path)
        .arg(downloaded_bin)
        .status()
        .await;

    match status {
        Ok(st) if st.success() => Ok(()),
        Ok(st) => Err(UpdaterError::Verification(format!("cosign exited with {st}"))),
        Err(e) => Err(UpdaterError::Verification(format!("cosign exec failed: {e}"))),
    }
}

async fn atomic_replace_executable(new_bin: &Path, target: &Path) -> Result<Option<PathBuf>, UpdaterError> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let _ = fs::create_dir_all(parent).await;

    // Copy into same directory for atomic rename.
    let tmp_target = parent.join(format!(
        ".{}.new",
        target.file_name().and_then(|s| s.to_str()).unwrap_or("pagi-core")
    ));
    fs::copy(new_bin, &tmp_target).await?;

    // Ensure executable bit on unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp_target)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tmp_target, perms)?;
    }

    let backup = if target.exists() {
        let backup_path = parent.join(format!(
            ".{}.bak",
            target.file_name().and_then(|s| s.to_str()).unwrap_or("pagi-core")
        ));
        // Best-effort backup; overwrite.
        let _ = fs::remove_file(&backup_path).await;
        fs::rename(target, &backup_path).await?;
        Some(backup_path)
    } else {
        None
    };

    // Atomic swap.
    match fs::rename(&tmp_target, target).await {
        Ok(()) => Ok(backup),
        Err(e) => {
            // attempt rollback
            if let Some(b) = &backup {
                let _ = fs::rename(b, target).await;
            }
            Err(UpdaterError::Io(e))
        }
    }
}

async fn restart_core_process(state: &AppState, core_path: &Path) -> Result<(), UpdaterError> {
    // Best-effort: spawn and detach.
    let mut cmd = if let Some(c) = state.restart_cmd.as_deref() {
        Command::new(c)
    } else {
        Command::new(core_path)
    };
    cmd.args(&state.restart_args);

    match cmd.spawn() {
        Ok(_child) => {
            info!(core = %core_path.display(), "spawned core restart");
            Ok(())
        }
        Err(e) => Err(UpdaterError::Io(e)),
    }
}
