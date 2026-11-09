#!/usr/bin/env bash
set -euo pipefail
# The engine, and the image export module the Pad loads only when a PNG or PDF is downloaded.
cargo build -p delta-wasm -p delta-export-wasm --release --target wasm32-unknown-unknown
# wasm-bindgen-cli version MUST equal the wasm-bindgen crate version in Cargo.lock
cargo install wasm-bindgen-cli --version "$(cargo pkgid -p wasm-bindgen | sed 's/.*[@#]//')" --locked
wasm-bindgen target/wasm32-unknown-unknown/release/delta_wasm.wasm \
  --target web --out-dir web/src/engine/pkg
wasm-bindgen target/wasm32-unknown-unknown/release/delta_export_wasm.wasm \
  --target web --out-dir web/src/engine/export-pkg
# optional size pass if binaryen is installed:
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz web/src/engine/pkg/delta_wasm_bg.wasm -o web/src/engine/pkg/delta_wasm_bg.wasm || true
  wasm-opt -Oz web/src/engine/export-pkg/delta_export_wasm_bg.wasm -o web/src/engine/export-pkg/delta_export_wasm_bg.wasm || true
fi
