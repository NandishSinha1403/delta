//! The WebAssembly facade over `delta-core`: a thin JSON-in / JSON-out API.
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

/// Returns M1's formal 5-tuple (Sipser, Figure 1.6) as JSON. Phase 0 pipeline check.
// `#[wasm_bindgen]` exports this function to JavaScript; an `Err` becomes a thrown JS `Error`.
#[wasm_bindgen]
pub fn hello() -> Result<String, JsError> {
    // `.map_err(...)` converts the error type inside a Result and leaves an `Ok` value untouched.
    delta_core::hello::hello_json().map_err(|e| JsError::new(&e.to_string()))
}
