use std::path::Path;

use serde_json::json;
use wasmtime::{Caller, Engine, Extern, Linker, Memory, Module, Store, TypedFunc};

use crate::ToolSchema;

#[derive(Default)]
struct HostState {
    registrations: Vec<ToolSchema>,
}

fn get_memory<'a>(caller: &mut Caller<'a, HostState>) -> Result<Memory, String> {
    match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => Ok(mem),
        _ => Err("missing export memory".to_string()),
    }
}

fn read_bytes(caller: &mut Caller<'_, HostState>, ptr: i32, len: i32) -> Result<Vec<u8>, String> {
    if ptr < 0 || len < 0 {
        return Err("negative ptr/len".to_string());
    }
    let mem = get_memory(caller)?;
    let mut buf = vec![0u8; len as usize];
    mem.read(caller, ptr as usize, &mut buf)
        .map_err(|e| format!("memory read failed: {e}"))?;
    Ok(buf)
}

fn read_string(caller: &mut Caller<'_, HostState>, ptr: i32, len: i32) -> Result<String, String> {
    let bytes = read_bytes(caller, ptr, len)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn host_register_tool(
    mut caller: Caller<'_, HostState>,
    name_ptr: i32,
    name_len: i32,
    desc_ptr: i32,
    desc_len: i32,
    endpoint_ptr: i32,
    endpoint_len: i32,
) {
    let name = match read_string(&mut caller, name_ptr, name_len) {
        Ok(s) => s,
        Err(_) => return,
    };
    let description = match read_string(&mut caller, desc_ptr, desc_len) {
        Ok(s) => s,
        Err(_) => return,
    };
    let endpoint = match read_string(&mut caller, endpoint_ptr, endpoint_len) {
        Ok(s) => s,
        Err(_) => return,
    };

    caller.data_mut().registrations.push(ToolSchema {
        name,
        description,
        plugin_url: String::new(),
        endpoint,
        parameters: json!({}),
    });
}

/// Loads a legacy Wasm module (wasm32-unknown-unknown style) and returns the tools
/// it registers by calling the host import `pagi.register_tool(...)` from `init()`.
pub(crate) fn register_tools(wasm_path: &Path) -> Result<Vec<ToolSchema>, String> {
    let wasm_path = wasm_path
        .canonicalize()
        .map_err(|e| format!("canonicalize failed: {e}"))?;

    let engine = Engine::default();
    let module = Module::from_file(&engine, &wasm_path).map_err(|e| format!("load wasm failed: {e}"))?;

    let mut linker = Linker::<HostState>::new(&engine);
    linker
        .func_wrap(
            "pagi",
            "register_tool",
            host_register_tool,
        )
        .map_err(|e| format!("link host func failed: {e}"))?;

    let mut store = Store::new(&engine, HostState::default());
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| format!("instantiate wasm failed: {e}"))?;

    // Call init if present.
    if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "init") {
        let _ = init.call(&mut store, ());
    }

    Ok(store.data().registrations.clone())
}

fn pack_u64_to_i32_pair(v: i64) -> (i32, i32) {
    let u = v as u64;
    let ptr = (u >> 32) as u32;
    let len = (u & 0xFFFF_FFFF) as u32;
    (ptr as i32, len as i32)
}

/// Execute an exported tool function from a legacy Wasm module.
///
/// Contract:
/// - module exports `memory`, `alloc(len) -> ptr`, `dealloc(ptr,len)`
/// - tool function is `fn(ptr,len) -> i64` packing (out_ptr,out_len)
pub(crate) fn execute_tool(wasm_path: &Path, symbol_name: &str, params: &serde_json::Value) -> Result<String, String> {
    let wasm_path = wasm_path
        .canonicalize()
        .map_err(|e| format!("canonicalize failed: {e}"))?;

    let engine = Engine::default();
    let module = Module::from_file(&engine, &wasm_path).map_err(|e| format!("load wasm failed: {e}"))?;

    let mut linker = Linker::<HostState>::new(&engine);
    // Provide the same import used for registration; it can be unused by tool execution.
    linker
        .func_wrap("pagi", "register_tool", host_register_tool)
        .map_err(|e| format!("link host func failed: {e}"))?;

    let mut store = Store::new(&engine, HostState::default());
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| format!("instantiate wasm failed: {e}"))?;

    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| "missing export memory".to_string())?;

    let alloc: TypedFunc<i32, i32> = instance
        .get_typed_func(&mut store, "alloc")
        .map_err(|e| format!("missing export alloc: {e}"))?;
    let dealloc: TypedFunc<(i32, i32), ()> = instance
        .get_typed_func(&mut store, "dealloc")
        .map_err(|e| format!("missing export dealloc: {e}"))?;
    let tool_fn: TypedFunc<(i32, i32), i64> = instance
        .get_typed_func(&mut store, symbol_name)
        .map_err(|e| format!("missing export tool function '{symbol_name}': {e}"))?;

    let params_json = serde_json::to_string(params).map_err(|e| e.to_string())?;
    let params_bytes = params_json.as_bytes();

    let in_ptr = alloc
        .call(&mut store, params_bytes.len() as i32)
        .map_err(|e| format!("alloc failed: {e}"))?;
    memory
        .write(&mut store, in_ptr as usize, params_bytes)
        .map_err(|e| format!("memory write failed: {e}"))?;

    let ret = tool_fn
        .call(&mut store, (in_ptr, params_bytes.len() as i32))
        .map_err(|e| format!("tool call failed: {e}"))?;

    let _ = dealloc.call(&mut store, (in_ptr, params_bytes.len() as i32));

    let (out_ptr, out_len) = pack_u64_to_i32_pair(ret);
    if out_ptr <= 0 || out_len <= 0 {
        return Err("tool returned empty output".to_string());
    }

    let mut out_buf = vec![0u8; out_len as usize];
    memory
        .read(&mut store, out_ptr as usize, &mut out_buf)
        .map_err(|e| format!("memory read failed: {e}"))?;

    let _ = dealloc.call(&mut store, (out_ptr, out_len));
    Ok(String::from_utf8_lossy(&out_buf).to_string())
}

