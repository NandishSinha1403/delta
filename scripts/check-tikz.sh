#!/usr/bin/env bash
# Compiles every machine the samples show as TikZ with tectonic, inside a plain
# \documentclass{standalone} + \usepackage{tikz} document, the only setup the .tex promises.
# Needs `deltac` built (target/release or target/debug) and `tectonic` on PATH. Run in CI.
set -euo pipefail
cd "$(dirname "$0")/.."
DELTAC=${DELTAC:-target/release/deltac}
[ -x "$DELTAC" ] || DELTAC=target/debug/deltac
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT
n=0
for f in samples/*.delta scripts/tikz/extra.delta; do
  name=$(basename "$f" .delta)
  mkdir -p "$OUT/$name"
  # The run itself may report failing checks; only the drawings matter here.
  "$DELTAC" run --format tikz --out "$OUT/$name" "$f" >/dev/null 2>&1 || true
  for tex in "$OUT/$name"/*.tex; do
    [ -e "$tex" ] || continue
    doc="$OUT/$name/doc-$(basename "$tex")"
    printf '\\documentclass{standalone}\n\\usepackage{tikz}\n\\begin{document}\n\\input{%s}\n\\end{document}\n' "$(basename "$tex")" > "$doc"
    if ! (cd "$OUT/$name" && tectonic --chatter minimal "$(basename "$doc")" >/dev/null 2>"$doc.log"); then
      echo "FAIL $f → $(basename "$tex")"; cat "$doc.log"; exit 1
    fi
    n=$((n + 1))
  done
done
[ "$n" -gt 0 ] || { echo "no TikZ drawings were produced"; exit 1; }
echo "tikz: $n drawings compiled with tectonic"
