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

/// Parses, checks and runs a Delta 2 program; returns the Pad's document as JSON.
// `#[wasm_bindgen]` exports this function to JavaScript; an `Err` becomes a thrown JS `Error`.
#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsError> {
    // `.map_err(...)` converts the error type inside a Result and leaves an `Ok` value untouched.
    serde_json::to_string(&delta_lang::pad::document(src)).map_err(|e| JsError::new(&e.to_string()))
}

/// Runs machine `name` of the program on `input`, step by step; returns a `RunResult` as JSON.
#[wasm_bindgen]
pub fn run(src: &str, name: &str, input: &str) -> Result<String, JsError> {
    serde_json::to_string(&delta_lang::pad::run(src, name, input))
        .map_err(|e| JsError::new(&e.to_string()))
}

/// The output document of 02-ENGINE.md §7 (what `delta run --json` prints).
#[wasm_bindgen]
pub fn run_json(src: &str) -> Result<String, JsError> {
    serde_json::to_string(&delta_lang::run(src)).map_err(|e| JsError::new(&e.to_string()))
}

/// The engine version, shown in bug reports.
#[wasm_bindgen]
pub fn version() -> String {
    delta_lang::VERSION.to_string()
}
