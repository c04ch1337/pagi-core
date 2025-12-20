use libloading::{Library, Symbol};
use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    os::raw::c_char,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::ToolSchema;

// --- Shared library registration ABI (C-friendly) ---

/// C ABI compatible representation of a tool.
///
/// The pointed-to memory is assumed to be valid for the lifetime of the loaded library.
#[repr(C)]
pub struct RegisteredTool {
    pub name: *const u8,
    pub name_len: usize,
    pub description: *const u8,
    pub desc_len: usize,
    pub endpoint: *const u8,
    pub endpoint_len: usize,
}

type RegisterToolsFn = unsafe extern "C" fn() -> *const RegisteredTool;
type RegisterToolsCountFn = unsafe extern "C" fn() -> usize;

// --- Shared library execution ABI (JSON-in/JSON-out) ---

type FreeFn = unsafe extern "C" fn(*mut c_char);

/// FFI contract: a tool's `endpoint` is treated as a symbol name. That symbol is expected to be an
/// `extern "C" fn(*const c_char) -> *mut c_char` that returns a JSON string.
type ExecuteFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;

static LOADED_LIBS: OnceLock<Mutex<HashMap<PathBuf, Library>>> = OnceLock::new();

fn libs() -> &'static Mutex<HashMap<PathBuf, Library>> {
    LOADED_LIBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn unload_not_in(keep: &HashSet<PathBuf>) {
    let mut guard = libs().lock().expect("loaded lib mutex poisoned");
    guard.retain(|k, _| keep.contains(k));
}

pub(crate) fn register_tools(lib_path: &Path) -> Result<Vec<ToolSchema>, String> {
    let lib_path = lib_path
        .canonicalize()
        .map_err(|e| format!("canonicalize failed: {e}"))?;

    // Call into the shared lib while holding the lock so borrowed `Symbol`s don't outlive `&Library`.
    let tools: Vec<ToolSchema> = {
        let mut guard = libs().lock().map_err(|_| "loaded lib mutex poisoned".to_string())?;
        let lib = guard
            .entry(lib_path.clone())
            .or_insert_with(|| unsafe { Library::new(&lib_path).expect("Library::new failed") });

        unsafe {
            let count_fn: Symbol<RegisterToolsCountFn> = lib
                .get(b"register_tools_count")
                .map_err(|e| format!("missing symbol register_tools_count: {e}"))?;
            let tools_fn: Symbol<RegisterToolsFn> = lib
                .get(b"register_tools")
                .map_err(|e| format!("missing symbol register_tools: {e}"))?;

            let count = count_fn();
            let tools_ptr = tools_fn();
            if tools_ptr.is_null() {
                return Err("register_tools returned NULL".to_string());
            }

            let slice = std::slice::from_raw_parts(tools_ptr, count);
            let mut out = Vec::with_capacity(count);

            for t in slice {
                let name = String::from_utf8_lossy(std::slice::from_raw_parts(t.name, t.name_len)).to_string();
                let description =
                    String::from_utf8_lossy(std::slice::from_raw_parts(t.description, t.desc_len)).to_string();
                let endpoint =
                    String::from_utf8_lossy(std::slice::from_raw_parts(t.endpoint, t.endpoint_len)).to_string();

                out.push(ToolSchema {
                    name,
                    description,
                    plugin_url: String::new(),
                    endpoint,
                    parameters: serde_json::json!({}),
                });
            }

            out
        }
    };

    Ok(tools)
}

pub(crate) fn execute_tool(
    lib_path: &Path,
    symbol_name: &str,
    params: &serde_json::Value,
) -> Result<String, String> {
    let lib_path = lib_path
        .canonicalize()
        .map_err(|e| format!("canonicalize failed: {e}"))?;

    let params_json = serde_json::to_string(params).map_err(|e| e.to_string())?;
    let c_params = CString::new(params_json).map_err(|e| e.to_string())?;

    let result_json = {
        let mut guard = libs().lock().map_err(|_| "loaded lib mutex poisoned".to_string())?;
        let lib = guard
            .entry(lib_path.clone())
            .or_insert_with(|| unsafe { Library::new(&lib_path).expect("Library::new failed") });

        let sym_bytes = symbol_name.as_bytes();

        unsafe {
            let func: Symbol<ExecuteFn> = lib
                .get(sym_bytes)
                .map_err(|e| format!("missing tool symbol '{symbol_name}': {e}"))?;

            let ptr = func(c_params.as_ptr());
            if ptr.is_null() {
                return Err(format!("tool '{symbol_name}' returned NULL"));
            }

            let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();

            // Optional free hook.
            if let Ok(free_fn) = lib.get::<FreeFn>(b"free_cstring") {
                free_fn(ptr);
            }

            s
        }
    };

    Ok(result_json)
}
