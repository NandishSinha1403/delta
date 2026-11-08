#!/usr/bin/env bash
# Builds the whole site (03-PRODUCTS.md §1): the front page and docs (site/, Astro + Starlight)
# with the Pad (web/) served at /pad/. Output: site/dist.
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/build-wasm.sh
cargo run -q -p delta-cli -- gen-docs site
(cd web && DELTA_BASE=/pad/ pnpm build --outDir ../site/public/pad --emptyOutDir)
(cd site && pnpm build)
