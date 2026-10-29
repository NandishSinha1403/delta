#!/usr/bin/env bash
# Fails if the engine's .wasm is larger than the budget, gzipped (docs/plan/03-ARCHITECTURE.md §6.4).
set -euo pipefail
WASM=web/src/engine/pkg/delta_wasm_bg.wasm
LIMIT_KB=700
[ -f "$WASM" ] || { echo "missing $WASM; run ./scripts/build-wasm.sh first" >&2; exit 1; }
raw=$(wc -c < "$WASM" | tr -d ' ')
gz=$(gzip -9 -c "$WASM" | wc -c | tr -d ' ')
echo "delta_wasm_bg.wasm: $((raw / 1024)) KB raw, $((gz / 1024)) KB gzipped (limit ${LIMIT_KB} KB gzipped)"
if [ "$gz" -gt $((LIMIT_KB * 1024)) ]; then
  echo "FAIL: WASM exceeds ${LIMIT_KB} KB gzipped" >&2
  exit 1
fi
