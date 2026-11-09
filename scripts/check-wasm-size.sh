#!/usr/bin/env bash
# Fails if a .wasm is larger than its budget, gzipped (docs/plan/03-ARCHITECTURE.md §6.4).
# The engine loads at startup (700 KB); the export module only on a PNG/PDF download (1000 KB).
set -euo pipefail
check() {
  local wasm=$1 limit_kb=$2
  [ -f "$wasm" ] || { echo "missing $wasm; run ./scripts/build-wasm.sh first" >&2; exit 1; }
  local raw gz
  raw=$(wc -c < "$wasm" | tr -d ' ')
  gz=$(gzip -9 -c "$wasm" | wc -c | tr -d ' ')
  echo "$(basename "$wasm"): $((raw / 1024)) KB raw, $((gz / 1024)) KB gzipped (limit ${limit_kb} KB gzipped)"
  if [ "$gz" -gt $((limit_kb * 1024)) ]; then
    echo "FAIL: $(basename "$wasm") exceeds ${limit_kb} KB gzipped" >&2
    exit 1
  fi
}
check web/src/engine/pkg/delta_wasm_bg.wasm 700
check web/src/engine/export-pkg/delta_export_wasm_bg.wasm 1000
