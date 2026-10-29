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

/// Lexes, parses, checks and evaluates a program; returns the `Document` as JSON.
// `#[wasm_bindgen]` exports this function to JavaScript; an `Err` becomes a thrown JS `Error`.
#[wasm_bindgen]
pub fn evaluate(src: &str) -> Result<String, JsError> {
    // `.map_err(...)` converts the error type inside a Result and leaves an `Ok` value untouched.
    serde_json::to_string(&delta_dsl::evaluate(src)).map_err(|e| JsError::new(&e.to_string()))
}

/// Runs machine `name` of the program on `input`; returns a `RunResult` as JSON.
#[wasm_bindgen]
pub fn run(src: &str, name: &str, input: &str) -> Result<String, JsError> {
    serde_json::to_string(&delta_dsl::run_input(src, name, input))
        .map_err(|e| JsError::new(&e.to_string()))
}

/// The engine version, shown in bug reports.
#[wasm_bindgen]
pub fn version() -> String {
    // `env!("X")` reads a value baked in at compile time; Cargo sets CARGO_PKG_VERSION.
    env!("CARGO_PKG_VERSION").to_string()
}
