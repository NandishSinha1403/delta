#!/usr/bin/env bash
# Builds the private manual into manual/dist (never into site/dist). Deploy from manual/ so
# the password gate (functions/) goes too:
#   cd manual && npx wrangler@4.42.0 pages deploy dist --project-name=delta-manual --branch=main
set -euo pipefail
cd "$(dirname "$0")/../manual"
pnpm install --frozen-lockfile 2>/dev/null || pnpm install
pnpm build
