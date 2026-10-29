# Delta — Progress

_Last updated: 2026-10-29_

## Phase 0 — Foundations ✅ (on `main`, deployed via GitHub Actions)
- Rust workspace (`delta-core`, `delta-dsl`, `delta-wasm`, `delta-cli`); Clippy denies unwrap/expect/panic.
- WASM in a Web Worker via Comlink; React 19 + TS strict + Vite + Tailwind v4.
- Responsive shell (desktop 3 panes, tablet 2 panes + sheet, phone tab bar); CodeMirror + symbol bar.
- CI + deploy workflows; Playwright smoke test at 3 viewports.

## Phase 1 — Regular languages (in progress, branch `phase-1-wip`)

### Done: engine (Rust) — all tests and clippy pass
- Core: `ids`, `BitSet`, `Alphabet`, `Dfa`, `Nfa`, `Gnfa`, hash-consed regex arena with a minimal-parentheses printer, traces, budgets.
- Simulations: S1 (DFA run), S2 (NFA "fingers").
- Conversions with traces: C1 DFA→NFA, C2 subset construction (full + reachable), C3 regex→NFA (Sipser's constructions), C4 →GNFA, C5 state elimination (faithful / simplified, rip orders).
- Golden tests: Example 1.41 (8 / 6 states), 1.56 (8 states), 1.58 (12 states), 1.66 (`a*b(a∪b)*`).
- Property tests: regex → NFA → DFA → GNFA → regex round trip, checked against brute-force oracles.
- DSL: lexer, parser with error recovery, checker (E101–E110, W111, W112, E201–E205, E301, W302, I304, E401–E408, W405, E8xx, R901), quick fixes, strict/lenient modes, `let` / `check` / `run`, UTF-16 spans for the editor.
- `examples/*.delta` (Figure 1.6, Examples 1.38, 1.41, 1.56, 1.58, 1.66, tour), used by golden tests and the gallery.
- WASM API: `evaluate(src)`, `run(src, name, input)`. CLI: `delta check FILE`, `delta json FILE`.

### Not done yet: web UI (the web app does not build on this branch yet)
- Engine client switched to `evaluate`/`run` (the Phase 0 `hello` was removed).
- Fonts: IBM Plex Mono **complete** build (`@ibm/plex-mono`; the fontsource build lacks ε Σ δ ∅ ∪ ⊔ →),
  JetBrains Mono as a fallback for ∘ ⁺, IBM Plex Sans for UI, DotGothic16 (Latin subset) for the logo only.
  Packages installed; CSS not written yet.
- Editor: Delta tokenizer and highlighting, completion (keywords, snippets, objects, states, symbols), hover docs,
  lint squiggles with quick fixes, Cmd/Ctrl+/ comments, find/replace.
- Neutral dark theme (VS Code Dark Modern style) as the default, plus a light toggle.
- Diagram (Graphviz layout → own SVG, pan/zoom), Tuple panel (KaTeX), δ-table, Run panel (tape + fingers),
  Steps scrubber, Tests and Problems panels, status bar, examples gallery.
- Update `docs/plan/05-PLAYGROUND-UX.md` with the editor/theme/font decisions; add new codes (E114, E120–E123,
  E408, W117, I115, I116, W201, R910, I811) to `08-ERRORS.md`.
- Phase 1 exit test (Playwright, phone): NFA → `dfa(N)` → scrub the steps → `regex(D)`.

## Next steps
1. Finish the web UI above, get all `CLAUDE.md` checks green, merge `phase-1-wip` into `main` (ask before pushing).
2. Phase 2: C6–C10, D1–D5/D8/D9, Conversion Studio, exports (share link, SVG/PNG, TikZ, DOT, JSON), PWA.
