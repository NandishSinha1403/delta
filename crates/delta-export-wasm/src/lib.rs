//! The Pad's image export, kept out of the main engine module: the Web Worker loads it only the
//! first time someone downloads a PNG or PDF. Same code as `deltac`, so the bytes match.

// `wasm_bindgen::prelude::*` imports the attribute and the JS interop types we need.
use wasm_bindgen::prelude::*;

/// The SVG drawn as a PNG at `scale`, with the "Delta by Nandish" line under it. A `Vec<u8>` arrives in JavaScript as a `Uint8Array`.
#[wasm_bindgen]
pub fn png(svg: &str, scale: f32) -> Result<Vec<u8>, JsError> {
    delta_export::png_file(svg, scale).map_err(|e| JsError::new(&e))
}

/// The SVG as a one-page PDF with its fonts embedded.
#[wasm_bindgen]
pub fn pdf(svg: &str) -> Result<Vec<u8>, JsError> {
    delta_export::pdf(svg).map_err(|e| JsError::new(&e))
}
