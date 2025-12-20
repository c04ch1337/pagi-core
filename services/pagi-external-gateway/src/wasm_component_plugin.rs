use std::path::Path;

use serde_json::json;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker, TypedFunc};

/// Execute a WASI Component Model plugin.
///
/// Contract (minimal): component exports `execute: func(params: string) -> result<string, string>`.
/// The host passes a JSON string containing `{ "endpoint": <tool_endpoint>, "parameters": <tool_params> }`.
pub(crate) fn execute_tool(
    component_path: &Path,
    tool_endpoint: &str,
    tool_params: &serde_json::Value,
) -> Result<String, String> {
    let component_path = component_path
        .canonicalize()
        .map_err(|e| format!("canonicalize failed: {e}"))?;

    let mut cfg = Config::new();
    cfg.wasm_component_model(true);
    let engine = Engine::new(&cfg).map_err(|e| format!("engine init failed: {e}"))?;

    let component = Component::from_file(&engine, &component_path)
        .map_err(|e| format!("load component failed: {e}"))?;

    // For now, instantiate without WASI. Component plugins should avoid WASI imports unless configured.
    let linker = Linker::new(&engine);
    let mut store = Store::new(&engine, ());

    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(|e| format!("instantiate failed: {e}"))?;

    // `execute: func(params: string) -> result<string, string>`
    let execute: TypedFunc<(String,), (Result<String, String>,)> = instance
        .get_typed_func(&mut store, "execute")
        .map_err(|e| format!("missing export execute: {e}"))?;

    let payload = json!({
        "endpoint": tool_endpoint,
        "parameters": tool_params,
    });
    let payload_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

    let (result,) = execute
        .call(&mut store, (payload_str,))
        .map_err(|e| format!("execute trap: {e}"))?;

    match result {
        Ok(s) => Ok(s),
        Err(err) => Err(err),
    }
}
