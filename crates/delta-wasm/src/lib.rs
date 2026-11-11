//! The WebAssembly facade over the engine: a thin JSON-in / JSON-out API.
//!
//! It only runs inside the Web Worker (`web/src/engine/worker.ts`), never on the main thread.

// `wasm_bindgen::prelude::*` imports the attribute and the JS interop types we need.
use wasm_bindgen::prelude::*;

/// Runs once when the WASM module is instantiated.
// `#[wasm_bindgen(start)]` tells wasm-bindgen to call this function automatically on load.
#[wasm_bindgen(start)]
pub fn start() {
    // Turn Rust panics into readable `console.error` messages (docs/plan/08-ERRORS.md §8).
    console_error_panic_hook::set_once();
}

/// The files `import` may read, from `{ "name.jff": "<xml>", … }` (empty if it isn't that shape).
fn files(json: &str) -> std::collections::HashMap<String, String> {
    serde_json::from_str(json).unwrap_or_default()
}

/// Parses, checks and runs a Delta 2 program; returns the Pad's document as JSON. `files` is a
/// JSON object of the open .jff files by name, for `import "x.jff"`.
// `#[wasm_bindgen]` exports this function to JavaScript; an `Err` becomes a thrown JS `Error`.
#[wasm_bindgen]
pub fn evaluate(src: &str, files_json: &str) -> Result<String, JsError> {
    // `.map_err(...)` converts the error type inside a Result and leaves an `Ok` value untouched.
    serde_json::to_string(&delta_lang::pad::document_with(src, &files(files_json)))
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Runs machine `name` of the program on `input`, step by step; returns a `RunResult` as JSON.
#[wasm_bindgen]
pub fn run(src: &str, name: &str, input: &str, files_json: &str) -> Result<String, JsError> {
    serde_json::to_string(&delta_lang::pad::run_with(
        src,
        name,
        input,
        &files(files_json),
    ))
    .map_err(|e| JsError::new(&e.to_string()))
}

/// The output document of 02-ENGINE.md §7 (what `deltac run --json` prints).
#[wasm_bindgen]
pub fn run_json(src: &str) -> Result<String, JsError> {
    serde_json::to_string(&delta_lang::run(src)).map_err(|e| JsError::new(&e.to_string()))
}

/// A `save` output's file as text: `svg`, `tex`, `txt` or `json` (PNG and PDF are made from
/// the SVG by the separate export module, `delta-export-wasm`).
#[wasm_bindgen]
pub fn render_file(value: &str, format: &str, text: &str, dark: bool) -> Result<String, JsError> {
    let v: serde_json::Value =
        serde_json::from_str(value).map_err(|e| JsError::new(&e.to_string()))?;
    let text = if text.is_empty() { None } else { Some(text) };
    delta_lang::render::text_file(format, &v, text, dark).map_err(|e| JsError::new(&e))
}

/// The Pad's diagram geometry for a `MachineView` (JSON), from `delta-render`: the same layout as
/// every download.
#[wasm_bindgen]
pub fn diagram_layout(view: &str) -> Result<String, JsError> {
    let v: serde_json::Value =
        serde_json::from_str(view).map_err(|e| JsError::new(&e.to_string()))?;
    let l =
        delta_lang::render::layout_json(&v).ok_or_else(|| JsError::new("not a machine view"))?;
    serde_json::to_string(&l).map_err(|e| JsError::new(&e.to_string()))
}

/// The engine version, shown in bug reports.
#[wasm_bindgen]
pub fn version() -> String {
    delta_lang::VERSION.to_string()
}
