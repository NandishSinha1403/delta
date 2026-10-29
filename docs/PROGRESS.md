# Delta — Progress

_Last updated: 2026-10-29_

## Phase 0 — Foundations ✅ (on `main`, deployed via GitHub Actions)
- Rust workspace (`delta-core`, `delta-dsl`, `delta-wasm`, `delta-cli`); Clippy denies unwrap/expect/panic.
- WASM in a Web Worker via Comlink; React 19 + TS strict + Vite + Tailwind v4.
- Responsive shell (desktop 3 panes, tablet 2 panes + sheet, phone tab bar); CodeMirror + symbol bar.
- CI + deploy workflows; Playwright smoke test at 3 viewports.

## Phase 1 — Regular languages ✅ (branch `phase-1-wip`, ready to merge)

### Engine (Rust)
- Core: `ids`, `BitSet`, `Alphabet`, `Dfa`, `Nfa`, `Gnfa`, hash-consed regex arena with a minimal-parentheses printer, traces, budgets.
- Simulations: S1 (DFA run), S2 (NFA "fingers").
- Conversions with traces: C1 DFA→NFA, C2 subset construction (full + reachable), C3 regex→NFA (Sipser's constructions), C4 →GNFA, C5 state elimination (faithful / simplified, rip orders).
- Golden tests: Example 1.41 (8 / 6 states), 1.56 (8 states), 1.58 (12 states), 1.66 (`a*b(a∪b)*`).
- Property tests: regex → NFA → DFA → GNFA → regex round trip, checked against brute-force oracles.
- DSL: lexer, parser with error recovery, checker (codes in `08-ERRORS.md`), quick fixes, strict/lenient modes, `let` / `check` / `run`, UTF-16 spans for the editor.
- `examples/*.delta` (Figure 1.6, Examples 1.38, 1.41, 1.56, 1.58, 1.66, tour): golden tests and the gallery.
- WASM API: `evaluate(src)`, `run(src, name, input)`, `version()`. CLI: `delta check FILE`, `delta json FILE`.

### Web UI
- Engine client: `evaluate` / `run` / `layout` / `version` through the worker; a call over 10 s terminates and respawns it.
  Graphviz is lazy-loaded inside the worker on the first diagram.
- Editor: Delta highlighting, completion (keywords, snippets, objects, states, symbols), hover docs, engine diagnostics
  with book quotes and quick fixes, lint gutter, Cmd/Ctrl+/ comments, find/replace.
- Fonts: IBM Plex Mono (complete build), JetBrains Mono for ∘ ⁺ ⊣ ∪ only, IBM Plex Sans, DotGothic16 for the logo.
- Themes: neutral dark (default) and light, remembered per browser.
- Views: Diagram (Graphviz layout → own SVG, pan / pinch / wheel zoom, fit), Tuple (KaTeX), δ-table, Steps scrubber
  (replays add-steps on the final machine, snapshots for state elimination), Run (tape, fingers, play/step, verdict),
  Tests, Problems (with quick fixes), status bar, examples gallery, per-panel error boundaries.
- Layouts: desktop 3 panes + Run dock; tablet 2 panes + sheet; phone tabs (Steps and Run show the diagram on top).
- Tests: Vitest (tuple formatting, trace replay, DOT/layout parsing); Playwright at phone/tablet/desktop, including the
  Phase 1 exit flow (NFA → `dfa(N)` → scrub → `regex(D)`), runs, diagnostics and the gallery.

## Next steps
1. Merge `phase-1-wip` into `main` (ask before pushing).
2. Phase 2: C6–C10, D1–D5/D8/D9, Conversion Studio, exports (share link, SVG/PNG, TikZ, DOT, JSON), PWA.
