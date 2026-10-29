#!/usr/bin/env bash
set -euo pipefail
cargo build -p delta-wasm --release --target wasm32-unknown-unknown
# wasm-bindgen-cli version MUST equal the wasm-bindgen crate version in Cargo.lock
cargo install wasm-bindgen-cli --version "$(cargo pkgid -p wasm-bindgen | sed 's/.*[@#]//')" --locked
wasm-bindgen target/wasm32-unknown-unknown/release/delta_wasm.wasm \
  --target web --out-dir web/src/engine/pkg
# optional size pass if binaryen is installed:
command -v wasm-opt >/dev/null && wasm-opt -Oz web/src/engine/pkg/delta_wasm_bg.wasm -o web/src/engine/pkg/delta_wasm_bg.wasm || true
