# 06 — Roadmap, Testing Strategy, Risks

Estimates assume **one developer at ~15–20 hours/week**, learning Rust along the way. Double the speed with two people, or full-time. Every phase ends with a **deployed, usable release** on `delta.nandish.dev`, never a half-built branch.

---

## 1. Phases

### Phase 0: Foundations (1–2 weeks)
**Goal: prove the whole pipeline before writing any automata logic.**
- Monorepo (`crates/`, `web/`), Rust workspace, React + Vite + TS strict, Tailwind, Zustand.
- A "hello" Rust function compiled to WASM and called from a Web Worker via Comlink, with generated TS types.
- GitHub Actions: CI + deploy. DNS CNAME, custom domain, HTTPS **live** at `delta.nandish.dev`.
- Responsive shell skeleton: desktop 3-pane / tablet 2-pane / phone tab bar (empty panels).
- CodeMirror wired in with the symbol bar.
- ✅ *Exit:* pushing to `main` updates the live site; a Rust function's result shows on a phone.

### Phase 1: Regular languages MVP → v0.1 (5–7 weeks)
- Core types: `ids`, `BitSet`, `Alphabet`, `Dfa`, `Nfa`, `Gnfa`, regex arena + printer (minimal parentheses).
- DSL: `dfa`, `nfa`, `regex`, `let`, `check accepts/rejects`, `run`; three δ forms; checker E101–E106, W110–W111.
- Conversions **C1, C2, C3, C4, C5** with traces; simulations **S1, S2**.
- Diagram renderer (Graphviz layout → own SVG), Tuple panel (KaTeX), δ-table, Run panel (tape + fingers), basic Steps scrubber.
- Example gallery: M1 (Figure 1.6), Examples 1.38, 1.41, 1.56, 1.58, 1.66.
- Golden tests for every Chapter 1 example used; first property tests.
- ✅ *Exit:* a student can type an NFA on a phone, convert to DFA, scrub the subset construction, then get the regex.

### Phase 2: Regular languages complete + polish → v0.2 (3–4 weeks)
- **C6–C10**, decision procedures **D1–D5, D8, D9** (counterexamples, shortlex enumeration, pumping split).
- Conversion Studio (side-by-side, faithful/optimized toggle, linked highlighting).
- Exports: share link, SVG/PNG, **TikZ**, DOT, JSON. PWA/offline. Light/dark themes.
- Layout block + drag-to-position (writes `layout {}`).
- ✅ *Exit:* everything in Chapter 1 of the book can be done in the tool.

### Phase 3: Context-free languages → v0.3 (5–6 weeks)
- `cfg`, `pda`, `dpda` in the DSL; **C11–C17, C19**; **D6, D7, D10**; **S3** (fast + exact mode).
- CNF step view (Example 2.10 style), CYK table, parse trees, derivation stepper, stack visualization.
- Pumping-lemma game (with CFG/PDA oracles).
- ✅ *Exit:* Examples 2.3, 2.4, 2.10, 2.14 reproduced; CFG ↔ PDA round trips verified.

### Phase 4: Turing machines → v0.4 (4–5 weeks)
- `tm`, `stayput`, multitape, `ntm`, `lba`, enumerator; **S4, S5**.
- Tape view, configuration log in `u q v` notation, run-to-halt with chunked execution + Stop, computation tree for NTMs, single-tape encoding view.
- ✅ *Exit:* Examples 3.7 (M2) and 3.9 (M1, {w#w}) reproduce the book's runs.

### Phase 5: v1.0 (3–4 weeks)
- Diagram editing → code patches (add edge, rename, toggle accept, delete) via `apply_edit`.
- JFLAP `.jff` import/export. Lecture mode. Hover docs with definitions. Onboarding tour.
- Accessibility audit (keyboard diagram navigation, screen-reader descriptions), performance pass, documentation site.
- ✅ *Exit:* v1.0 announcement.

### Stretch (v1.x)
DK-test + LR(0) parse visualization (C18) · multitape → single-tape and NTM → DTM *constructions* · PCP explorer (§5.2) · lexer-spec mode · classroom mode (assignment links with hidden tests, autograder JSON) · Python package (PyO3) for notebooks and autograding · **Tauri** desktop/mobile apps wrapping the same web UI.

**Rough total to v1.0:** ~21–28 weeks part-time.

---

## 2. Definition of done (every feature)
1. Book reference in the Rust doc comment (Definition/Theorem/Lemma number).
2. Faithful mode matches the book; optimized mode (if any) labeled "beyond the book".
3. Emits trace steps with human-readable messages.
4. Golden test (if the book has an example) + property test.
5. Budget-respecting (cannot hang the worker).
6. Works on phone, tablet and desktop layouts; keyboard-accessible.
7. Documented in the in-app help.

---

## 3. Testing strategy

### 3.1 Golden tests from the book (the core of correctness)
Stored as `.delta` files in `examples/` with expected outputs in `examples/*.expected.json`. Run by `cargo test` and by the CLI.

| Test | Expected | Status |
|---|---|---|
| Example 1.41: N₄ → DFA (full) | 8 states; start {1,3}; accept {1},{1,2},{1,3},{1,2,3} | ✔ verified by prototype |
| Example 1.41: N₄ → DFA (reachable) | 6 states {1,3},{2},{2,3},{3},∅,{1,2,3} (Figure 1.44) | ✔ verified by prototype |
| Example 1.56: (ab∪a)* → NFA | 8 states | ✔ verified by prototype |
| Example 1.58: (a∪b)*aba → NFA | 12 states | ✔ verified by prototype |
| Example 1.66: 2-state DFA → regex (rip 2, then 1) | a\*b(a∪b)\* | ✔ verified by prototype |
| Example 1.68: 3-state DFA → regex | as Figure 1.69 | to transcribe from the figure |
| Example 2.10: G₆ → CNF | the grammar listed in step 4 (up to renaming fresh variables) | from the text |
| Example 2.14: PDA for 0ⁿ1ⁿ | accepts ε, 01, 0011; rejects 0, 10, 001 | from the text |
| Example 3.7: TM M2 on 0000 | the book's 22 configurations, accept | ✔ verified by prototype |
| Example 3.9: TM M1 ({w#w}) | accepts 01#01, rejects 01#10 | to transcribe from Figure 3.10 |

### 3.2 Property-based tests (`proptest`), the safety net
Random regexes (depth ≤ 6, alphabet {a, b}), random NFAs (≤ 8 states), random CFGs (≤ 4 variables):
- **Round trip:** R → NFA → DFA → minimize → GNFA → regex R′ → DFA; `equivalent(R, R′)` must hold.
- **Two independent paths agree:** C3 → C2 → C9 vs C10 (derivatives) → C9 give isomorphic minimal DFAs.
- **Minimization:** Hopcroft ≅ table-filling; `minimize(minimize(D)) == minimize(D)`; minimal state count = number of Myhill–Nerode classes found by brute force on short strings.
- **Closure:** union by product ≡ union by NFA (Thm 1.25 vs 1.45); complement(complement(D)) ≡ D; A ∩ B = complement(complement(A) ∪ complement(B)).
- **CFG:** `cnf(G)` generates exactly the same strings as G up to length 8 (oracle: bounded derivation search on G); CYK agrees with that oracle.
- **PDA ↔ CFG:** `pda(G)` and `cfg(pda(G))` accept the same strings up to length 8.
- **TM:** stay-put → standard conversion gives the same accept/reject on random inputs.

### 3.3 Brute-force oracles (differential testing)
A tiny, deliberately naive implementation of each *acceptance definition* (straight from 01-THEORY.md, no optimizations) is used to check the fast implementations on all strings up to length n.

### 3.4 Parser and front end
- `cargo-fuzz` on the DSL parser (it reads untrusted text from share links).
- Snapshot tests for the formatter and diagnostics.
- Vitest for React components and stores.
- **Playwright end-to-end on 6 viewports** (iPhone SE, Pixel 7, iPad portrait/landscape, 1366×768, 1920×1080): type code → diagram appears → run → convert → export.
- Visual regression snapshots of diagrams for the example gallery.
- axe accessibility checks in CI.

### 3.5 Performance
`criterion` benchmarks (subset construction on the classic "n-th symbol from the end is 1" NFA family, which has a 2ⁿ blowup; Hopcroft on 10⁵ states; TM steps/s; CYK n = 200). CI fails on > 20% regression. Bundle-size check in CI.

---

## 4. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Learning Rust slows Phase 1 | high | medium | "Simple Rust" style guide; Chapter 1 only first; the engine sits behind `EngineApi`, so a TS fallback is possible |
| Exponential blowups freeze the UI | high | high | worker isolation + budgets + Stop (terminate/respawn worker) |
| Diagram layout gets messy for dense machines | medium | medium | Graphviz `dot`; manual `layout {}`; table view for big machines; edge-label collision handling |
| Diagram → code patching is fiddly | medium | medium | deferred to Phase 5; until then dragging only writes positions |
| Scope creep (Chapters 4–10 ideas) | high | high | phase exit criteria; stretch list kept separate |
| Graphviz WASM size | medium | low | lazy-load + prefetch; ELK.js as an alternative |
| wasm-bindgen CLI/crate version mismatch breaks builds | medium | low | build script derives the CLI version from `Cargo.lock` |
| Unicode symbols hard to type on phones | high | medium | symbol bar + ASCII spellings for everything |
| Students' notation differs from Sipser (e.g. `+` = union) | high | low | warnings with explanations, not silent reinterpretation |
