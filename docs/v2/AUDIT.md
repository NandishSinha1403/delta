# Delta 1 → Delta 2: Audit

Read-only audit of the v1 code at tag `v1.0-final` (`50bf153`), done in Phase 0. It checks the
migration table in `04-ROADMAP.md` §1 against the real code. Nothing was changed while writing it.

**Verdicts:** **Keep** (moves over almost as is) · **Change** (the idea and most of the code stay,
reshaped for v2) · **Drop** (replaced; at most the approach is reused).

## 0. Summary

- v1 is **Sipser Chapter 1.1–1.3 only**: DFA, NFA, GNFA and regex, plus 7 conversions and 2
  simulators. About 6,900 lines of Rust and 5,500 of TypeScript. There are 79 Rust tests (43 real,
  plus 36 ts-rs export stubs), 25 Vitest tests and 14 Playwright tests at 3 viewports. All pass at
  the tag.
- **Wrong in the plan:** v1 has **no minimization of any kind** (no Hopcroft, Moore, Brzozowski or
  table filling). It also has no equivalence, emptiness, finiteness or inclusion checks and no
  complement. `02-ENGINE.md` §2.2 ("the v1 subset construction, **Hopcroft minimization** and state
  elimination") and `04-ROADMAP.md` §1 overstate what can be ported. Every decision procedure in
  `01-LANGUAGE.md` §6.3 and §12 "Decisions" is **new work** (questions Q-50, Q-51).
- **The algorithms port well.** They are small, one per file, budgeted, traced and oracle-tested.
  The model layer does not: `Dfa`, `Nfa` and `Gnfa` are three unrelated structs with `u32` state
  ids, and v2 wants one generic `Machine` whose states are arbitrary values.
- **Book coupling is deep but mechanical.** 226 citation strings sit in 50 files (§3). About 60 of
  them reach the user at runtime: trace rules, step titles, state origins, diagnostic `BookRef`s,
  UI copy and hover docs. They all go through two channels (`&'static str` rule names and the
  `BookRef` struct), so moving them to `delta-explain` is a well-defined job.
- **The worker, Stop/respawn, CI, Pages deploy and Playwright setup can be kept almost as they
  are.** The engine API changes shape (from `evaluate`/`run` to the v2 `DeltaEngine`), but the
  transport around it stays.

---

## 1. Module by module

### 1.1 `crates/delta-core` (3,459 lines)

| Module | Lines | What it is | Verdict | Notes for v2 |
|---|---:|---|---|---|
| `ids.rs` | 40 | `StateId(u32)`, `SymId(u32)` newtypes | **Keep** | Still the right internal index. v2 `states: IndexSet<Value>` is indexed by a `StateIx`, which is the same idea |
| `bitset.rs` | 188 | Growable bitset, hashable | **Keep** | Used for NFA state sets and `Machine.accept`. Fast-path material |
| `alphabet.rs` | 117 | Interned symbol names, `<RESET>` display | **Change** | Keep the interning. v2 needs several alphabets per machine (input, stack, tape, output → `Alphabets`) and symbols as `Value`s |
| `budget.rs` | 52 | `Budget{max_states,max_trace,max_regex_nodes}`, `TlError` | **Change** | Keep "budget outcome is a value". v2 adds steps, recursion depth and output size (01 §13). `TlError::Budget` becomes B6xx; `Internal` stays X999 |
| `state.rs` | 50 | `StateInfo{name, origin}`, `Origin::{User,Subset,Trap,Rule(&'static str),Copied}` | **Change** | `Origin::Rule("Theorem 1.45")` is a book string in core. Replace it with a stable rule id (`"nfa/union"`), and keep provenance (02 §5) |
| `dfa.rs` | 94 | `Dfa`: dense \|Q\|×\|Σ\| table | **Change** | Becomes the **dfa fast path**, built from a generic `Machine` whose model is deterministic and total |
| `nfa.rs` | 132 | `Nfa`: adjacency lists, `Label::{Eps,Sym}`, ε-closure, `StateGroup` | **Change** | Becomes the **nfa fast path** (bitsets). `StateGroup` (regex sub-expression boxes) is a render hint and goes to `delta-render` or provenance |
| `gnfa.rs` | 62 | `Gnfa`: dense regex matrix, always in special form | **Change** | Becomes the `gnfa` model's fast representation. The special-form invariant becomes the `special-form` property check (SEMANTICS §5.6) |
| `regex.rs` | 330 | Hash-consed n-ary regex arena, faithful vs simplified constructors, printer | **Keep** | Solid. Its printer becomes notation-profile aware (`+` vs `∪`, `λ`) |
| `object.rs` | 53 | `Object::{Dfa,Nfa,Gnfa,Regex}` closed enum | **Drop** | Replaced by the interpreter's `Value` |
| `oracle.rs` | 100 | Brute-force acceptance per definition, `all_strings` | **Keep** | The heart of differential testing (04 §3). Add a generic-configuration oracle |
| `trace.rs` | 124 | `Step{rule,title,message,add_states,add_edges,focus,…,snapshot}` | **Change** | `rule` and `title` hold book text. v2 wants typed steps (`AddState`, `AddEdge`, `Closure`, `Rip`, …) with neutral messages. `add_edges` names edges by (from, to) pairs, so it depends on edge merging in `view.rs`; v2 should use edge indices |
| `view.rs` | 328 | `MachineView`, `TraceView`, … (serde + ts-rs) | **Change → `delta-render`** | Display code in core, with book text in `state_view()` (`"…(Theorem 1.39)"`, `"…(Definition 1.5)"`). Split it out, as 02 §1 plans |
| `convert/mod.rs` | 42 | helpers: `set_name`, `fresh_name` (priming) | **Keep** | |
| `convert/c01_dfa_to_nfa.rs` | 52 | DFA viewed as an NFA | **Drop** | With one generic `Machine`, a dfa *is* an nfa-compatible machine. Only the model changes |
| `convert/c02_subset.rs` | 343 | Subset construction, reachable or full, budgeted | **Keep** (fast path) | Key for 02 §9's 10⁵-state target. `Full` mode (Example 1.41's 2^k table) → `method: "full"`? (Q-52) |
| `convert/c03_regex_to_nfa.rs` | 474 | Regex → NFA, textbook construction, groups, step titles | **Keep** (`method: "textbook"`) | 40 book strings: rule names `"Lemma 1.55 (1)"` and titles `"Theorem 1.45 · 0∪1"`. Thompson and Glushkov (01 §12) are new |
| `convert/c04_to_gnfa.rs` | 185 | DFA/NFA → GNFA in special form | **Keep** | Becomes `gnfa(M)` |
| `convert/c05_gnfa_to_regex.rs` | 285 | State elimination with rip orders, faithful/simplified | **Keep** (`method: "state-elimination"`) | `order:` already exists, matching 01 §12. Arden and Brzozowski–McCluskey are new |
| `convert/c06_product.rs` | 311 | Product of two DFAs, reachable pairs, alphabet extension with a trap | **Keep** (fast path for `union`/`intersect`/`difference`) | Mirrors the v2 `product` example in 01 §7.1, so it makes a good differential-test reference |
| `convert/c07_nfa_closure.rs` | 440 | NFA union, concatenation and star, with shared wiring helpers | **Keep** | Helpers shared with c03: keep that sharing |
| `simulate/s1_dfa.rs` | 96 | DFA run path and `RunView` | **Change** | Run → generic `trace` value. Keep as the dfa fast path |
| `simulate/s2_nfa.rs` | 205 | NFA set simulation, plain-English step sentences | **Change** | The "fingers" explanation text is good UX. Keep it as neutral wording in `delta-std` (no book words in it) |
| `tests/properties.rs` | 110 | proptest: regex→NFA→DFA agree; full round trip | **Keep** | Re-express once v2 values exist |
| `Cargo.toml` | | deps: rustc-hash, serde, serde_json, **ts-rs** | **Change** | Core depends on serde, serde_json and ts-rs only because of `view.rs`. After the split, core needs at most serde |

### 1.2 `crates/delta-dsl` (4,817 lines) → replaced by `delta-lang`

| Module | Lines | Verdict | Notes |
|---|---:|---|---|
| `lexer.rs` | 277 | **Drop** (reuse approach) | Unicode/ASCII → same token is right (and tested). v2 needs newline tokens, `'a'` symbols, `re"…"`, set-builder and many more operators |
| `parser.rs` | 1,289 | **Drop** (reuse approach) | **Reuse:** recovery to the next statement line or the closing `}` (02 §4 "one bad line never blanks the output"), and the test `syntax_errors_recover_and_the_rest_of_the_file_still_works` |
| `ast.rs` | 193 | **Drop** | Machine-only AST (`Field::{Q,Sigma,Start,Accept,Delta}`) |
| `build.rs` | 750 | **Drop** (reuse rules) | Checker for Definitions 1.5/1.37. Its rules become **model-property checks** (E3xx) driven by `deterministic`/`total`/`epsilon`. Reuse the lenient inference of Q and Σ (01 §5.1 "inferred if omitted") and its quick fixes (add trap state, mark partial, add to Q) |
| `eval.rs` | 1,326 | **Drop** | Hard-coded dispatch over `dfa`/`nfa`/`gnfa`/`regex`/`union`/`intersect`/`concat`/`star`, with step titles that cite the book built inline. v2: built-ins with signature records in `delta-std` |
| `diag.rs` | 193 | **Change** (keep the plumbing) | `Span`, `Severity`, `TextEdit`, `Fix` and collect-all-in-one-pass are right. `code: &'static str` is fine. `BookRef` in `Diagnostic` moves to an `explain` id that `delta-explain` resolves |
| `document.rs` | 157 | **Change** | Its **UTF-16 offset mapping** (`Utf16Map`, test `spans_are_utf16`) must survive: CodeMirror needs it, and the v2 JSON in 02 §7 has only `line` (Q-40). The `Document{objects,diagnostics,results,outline}` shape is replaced by the v2 `outputs` stream |
| `book.rs` | 68 | **Move** | 17 Sipser quotations → `delta-explain/profiles/sipser3e.toml` (`[rule."model/total"]` etc.) |
| `tests.rs` | 391 | **Change** | 21 end-to-end tests. Their *inputs* are v1 syntax; their *expectations* (state counts, regex text, codes) carry over |
| `guide_tests.rs` | 136 | **Change** | "Every code block in the AI guide runs" becomes the v2 docs CI rule (04 §3 Docs). The harness idea carries over |

### 1.3 `crates/delta-wasm` (36 lines), `crates/delta-cli` (94 lines)

| Part | Verdict | Notes |
|---|---|---|
| `delta-wasm`: `evaluate`, `run`, `version`, panic hook | **Keep crate, new surface** | Becomes the v2 `DeltaEngine` (02 §8): `run`, `check`, `complete`, `hover`, `format`, `layout`, `export`, `importJflap` |
| `delta-cli`: `delta check FILE`, `delta json FILE` | **Change** | v2: `delta run [--json] \| check \| fmt \| test \| migrate`. The Phase 0 conformance harness already calls `delta run --json` (§5) |

### 1.4 `web/` (React Pad, ~5,500 lines)

| Part | Verdict | Notes |
|---|---|---|
| `engine/api.ts`, `client.ts`, `worker.ts` | **Keep** | Comlink worker, 10 s timeout respawn, `restart()` = Stop. Swap the method set for v2 |
| `engine/generated/*` (20 ts-rs files) | **Change** | Regenerate from the v2 output schema (`/schema/output-v2.json`). ts-rs or the JSON Schema: decide one source of truth (Q-41) |
| `state/engine.ts`, `execute.ts`, `ui.ts` | **Keep** | Ticketing (only the newest answer shows), `SLOW_MS = 300` (matches 03 §3.1), Auto off by default, stale banner |
| `editor/CodeEditor.tsx`, `delta-lang.ts`, `codelens.ts`, `symbols.ts`, `SymbolBar.tsx` | **Change** | CodeMirror setup and Dark+/Light+ colours stay. `delta-lang.ts` holds **hand-written hover docs that cite the book** (59 citation hits) and its own keyword list. v2 generates both from signature records via `complete`/`hover` |
| `diagram/Diagram.tsx`, `layout.ts` | **Keep** | Own SVG, nested group boxes, pan/zoom; layout from `delta-render` (Graphviz dropped in Phase 7, Q-34). Add ε dashed, label pills, storage views |
| `panels/DiagramPanel`, `TuplePanel`, `tuple-format.ts`, `RunPanel`, `StepsPanel`, `TestsPanel`, `overlay.ts`, `current.ts` | **Change** | Tuple + δ-table → **Table** panel. Steps stays. Tests stays. Run → storage views + trace (Q-30). New: **Lang**, **Console**. `tuple-format.ts` hard-codes `'Definition 1.5'`/`'1.37'`/`'1.64'` labels |
| `conversions.ts` | **Drop** | Hand-kept copy of the engine's function list with `book:` citations. v2 reads signature records |
| `examples.ts` | **Change** | Gallery titles parsed from `// Example 1.41 — …` comments. v2 samples (~20) are not book-ordered |
| `shell/*` (TopBar, Rail, Tabs, CommandPalette, StatusBar, useLayout, PanelBoundary, Credit) | **Change** | Restructure to the DartPad layout (03 §3). `TopBar` shows a **"Sipser 3e · Ch 1.1–1.3" badge**. `Rail` has a "Book examples" item |
| `index.css` | **Change** | Palette tokens already match 03 §7 (`#0d0e10`, `#111215`, `#17191c`, `#f1faee`, `#a8dadc`, `#1d3557`). **Fonts differ:** v1 uses IBM Plex Mono + JetBrains Mono + KaTeX's ∪. v2 locks Courier Prime → Courier New → STIX Two Math. Check the ∪ and ⁺ glyph problem v1 solved again with Courier Prime |
| Vitest (8 files, 25 tests) | **Keep** most | `tuple-format`, `regex-tex`, `layout`, `symbols`, `palette-filter`, `delta-lang`, `codelens`, `conversions` (drop with `conversions.ts`) |
| `e2e/smoke.spec.ts` (14 tests), `playwright.config.ts` (3 viewports) | **Keep and extend** | v2: 6 viewports, offline PWA, share round trip, embed |

### 1.5 Repo and tooling

| Part | Verdict | Notes |
|---|---|---|
| `.github/workflows/ci.yml` | **Keep** | fmt, clippy `-D warnings`, test, ts-rs drift check, wasm build + size, lint, typecheck, vitest, build, Playwright |
| `.github/workflows/deploy.yml`, `web/public/CNAME` | **Keep** | v1 stays live until the v2 beta (04 §1) |
| `scripts/build-wasm.sh`, `check-wasm-size.sh` | **Keep, change limit** | v1 limit **700 KB** gzipped. v2 target is **≤ 900 KB** (02 §9) |
| Workspace lints (`unsafe_code = forbid`, deny `unwrap`/`expect`/`panic`) | **Keep** | Extend to `delta-lang`, `delta-std`, `delta-classes`, `delta-explain` |
| `examples/*.delta` (13 files) | **Convert** | All named after Sipser items (`ex1-41-N4`, `thm1-25-union`, …). They are golden tests, so convert them with `delta migrate` and keep their expectations |
| `docs/plan/` | **History** | v2 wins (README §4) |
| `docs/DELTA-FOR-AI.md` (2,540 lines), `DELTA-FOR-AI-SHORT.md` | **Drop** (regenerate) | v2 AI guides are generated, never hand-edited (04 §3) |
| `docs/PROGRESS.md` | **Change** | Restart for v2 |

---

## 2. Coupling problems

1. **Display types live in the engine core.** `delta-core/src/view.rs` and the `RunView` types in
   `simulate/mod.rs` derive `Serialize` + `TS`. That pulls serde, serde_json and ts-rs into core
   and ties the core's shape to the UI's JSON. → `delta-render`.
2. **Citations are the identity of steps and states.** `Step.rule: &'static str` holds
   `"Theorem 1.39"`, and `Origin::Rule(&'static str)` holds `"Theorem 1.45"`. Nothing else names
   a rule, so `delta-explain` has **no stable id to key on** (02 §1 rule 2). → Introduce rule and
   algorithm ids (`determinize/subset`, `nfa/union`, `regex/textbook/sym`, …) first, then move the
   text.
3. **Trace edges are addressed by (from, to).** `trace_view` finds each step's arrow by looking up
   the merged `EdgeView` with the same endpoints. This breaks with v2's multiple labelled edges per
   pair (PDA/TM). → Address edges by index into `M.edges`.
4. **The model is implied by which struct you hold.** Determinism and totality are guaranteed by
   the `Dfa` table's shape, not checked against properties. Partial DFAs are completed with a trap
   **in the DSL** (`build.rs`), not in core. In v2, `total`, `missing` and the `(partial)` option
   are model properties checked by one rule set (SEMANTICS §5).
5. **The function list is written out three times.** `eval.rs` (dispatch + option names), web
   `conversions.ts` (labels, `takes`, `book`) and `delta-lang.ts` (hover docs, completions) are
   kept in sync by hand. → signature records (02 §5).
6. **The whole program is re-evaluated for a run.** `run(src, name, input)` re-parses and
   re-evaluates all of `src`. That's fine for v1 sizes, but v2 user code can loop and is budgeted.
   Re-running the whole program on every step request multiplies cost (Q-42).
7. **Diagnostics carry quotations.** `Diagnostic.book: Option<BookRef>` is filled by the checker
   (`build.rs`: 25 call sites). → `explain: Option<RuleId>`.
8. **Planned crate cycle.** 02 §3 puts `LangSource::Predicate(Closure)` and `Machine` in
   `delta-classes`, but `Closure` is an interpreter value (`delta-lang`), and `delta-lang` needs
   `delta-classes` for class propagation. Likewise `Machine.states: IndexSet<Value>` puts the
   interpreter's `Value` inside `delta-core`, which "depends on no other Delta crate" (Q-43).

---

## 3. Book-specific strings (complete inventory)

Patterns searched: `Theorem|Lemma|Definition|Example N|Figure|Exercise|Corollary|Sipser|p. N|book`
in `crates/`, `web/src`, `web/e2e`, `examples/`, `scripts/`, `.github/`. There are 226 numbered
citations. The counts, by citation:

| Citation | Uses | Citation | Uses | Citation | Uses |
|---|---:|---|---:|---|---:|
| Theorem 1.39 | 28 | Lemma 1.55 | 37 | Definition 1.5 | 21 |
| Theorem 1.49 | 25 | Lemma 1.60 | 25 | Definition 1.52 | 12 |
| Theorem 1.25 | 24 | Exercise 1.15 | 6 | Definition 1.37 | 6 |
| Theorem 1.45 | 24 | Example 1.41 | 6 | Definition 1.64 | 3 |
| Theorem 1.47 | 18 | Example 1.7 | 5 | Figure 1.44 | 3 |
| p. 47 | 8 | p. 65 / p. 66 | 5 / 5 | p. 36 | 4 |
| Example 1.38 | 3 | Example 1.9, 1.53, 1.66 | 2 each | Figure 1.6, 1.67 | 2 each |
| Example 1.13, 1.56, 1.58; Figure 1.43 | 1 each | pp. 40, 44 | 2 each | pp. 46, 54, 56, 64, 70, 71, 73 | 1 each |
| `§0.2` (book section) | 7 | `§1.2` | 1 | "Sipser" (word) | 4 |

### 3.1 User-visible at runtime: must move to `delta-explain`

| Where | What | Count |
|---|---|---:|
| `delta-dsl/src/book.rs` | 17 `BookRef{item, quote}` constants: §0.2 ×4, Definition 1.5 ×5, p. 36, Definition 1.37, p. 47, Definition 1.52, p. 65, Theorem 1.39, Lemma 1.60 | 17 |
| `delta-dsl/src/build.rs` | `.book(…)` on diagnostics (25 sites); messages `"…Definition 1.5 requires δ : Q × Σ → Q to be total."` (×2), `"…(legal, p. 36)."` | ~28 |
| `delta-dsl/src/eval.rs` | Object `derived` strings and trace titles: `"Theorem 1.39 · subset construction"`, `"p. 47 · every DFA is an NFA"`, `"Lemma 1.55 · regular expression → NFA"`, `"Lemma 1.60 · GNFA in special form"`, `"Lemma 1.60 · CONVERT(G), state elimination"`, `"simplified with the identities of p. 66"`, `"Theorem 1.25 · product construction, union"`, `"Theorem 1.25 (footnote) · …intersection"`, `"Theorem 1.45 · union of NFAs"`, `"Theorem 1.47 · concatenation of NFAs"`, `"Theorem 1.49 · star of an NFA"`; error texts citing Lemma 1.60, Theorem 1.25 footnote, Example 1.53; `.book()` ×4 | ~30 |
| `delta-dsl/src/parser.rs:900` | `"In Sipser, + means one-or-more (R⁺ = RR*), not union. Did you mean ∪?"` + `DEF_1_52_PLUS` | 2 |
| `delta-core/src/convert/*` | `RULE` constants and `Step::new("…")` rules and titles: c02 `"Theorem 1.39"`; c03 `"Lemma 1.55 (1)/(2)/(3)"`, `"Theorem 1.45/1.47/1.49 · <subexpr>"`; c04/c05 `"Lemma 1.60"` (+ `"…as in Figure 1.67"`); c06 `"Theorem 1.25"`, `"…(footnote to Theorem 1.25)…"`; c07 `"Theorem 1.45/1.47/1.49"`; c01 `"p. 47"`; c05 error `"…(Lemma 1.60)."` | ~45 |
| `delta-core/src/view.rs` | `"subset of the NFA's states, ε-closed (Theorem 1.39)"`, `"trap state, added so that δ is total (Definition 1.5)"` | 2 |
| `web/src/editor/delta-lang.ts:188–216` | Hover docs: `'Definition 1.5 — finite automaton'`, `'Definition 1.37 — …'`, `'Definition 1.64 — …'`, `'Definition 1.52 — …'`, and bodies citing Theorem 1.39, Lemma 1.55, Lemma 1.60 | ~16 |
| `web/src/conversions.ts:22–29` | `book: 'Theorem 1.39' / 'Lemma 1.55' / 'Lemma 1.60' ×2 / 'Theorem 1.25 / 1.45' / 'Theorem 1.25' / 'Theorem 1.47' / 'Theorem 1.49'`. Shown in the Rail and the Diagram panel tooltips | 8 |
| `web/src/panels/tuple-format.ts:76,88` | `definition: 'Definition 1.64' / 'Definition 1.37' / 'Definition 1.5'` | 3 |
| `web/src/panels/TuplePanel.tsx:142` | `<p>Definition 1.52</p>` | 1 |
| `web/src/panels/TestsPanel.tsx:129` | `"No problems. The program matches the book's definitions."` | 1 |
| `web/src/panels/DiagramPanel.tsx:179` | `"…to see Lemma 1.55 build one."` | 1 |
| `web/src/shell/TopBar.tsx:124` | Badge `"Sipser 3e · Ch 1.1–1.3"` | 1 |
| `web/src/shell/Rail.tsx:12` | `'Book examples'` | 1 |
| `web/src/editor/SymbolBar.tsx:32` | `aria-label="Insert book symbols"` | 1 |

### 3.2 Code comments and doc comments: rewrite neutrally or keep in profile docs

Every `//!` header in `delta-core` cites its Definition or Theorem (CLAUDE.md rule "Book-faithful:
every algorithm cites its Definition/Theorem/Lemma number"). **That rule conflicts with 02 §1**:
the CI grep for "Theorem", "Lemma", "Sipser" and "Definition" would fail on comments too (Q-44).
Affected: `lib.rs`, `alphabet.rs`, `dfa.rs`, `nfa.rs`, `gnfa.rs`, `regex.rs`, `state.rs`,
`oracle.rs`, `trace.rs`, all `convert/*.rs`, `simulate/*.rs`, `bitset.rs`, `tests/properties.rs`.

### 3.3 Test names, test data and examples

- Rust test names: `example_1_41_*`, `example_1_56_*`, `example_1_58_*`, `example_1_66_*`,
  `exercise_1_15_naive_star_is_wrong`, `special_form_of_example_1_66`,
  `example_1_41_gives_figure_1_44_and_figure_1_43`,
  `union_of_two_dfas_is_the_product_and_with_an_nfa_is_theorem_1_45`, …. Test *data* may keep its
  origin in a comment if Q-44 allows; otherwise rename the tests by content.
- Vitest/Playwright: `smoke.spec.ts` (12 hits, e.g. expects "Theorem 1.39" text on screen),
  `tuple-format.test.ts` (4), `regex-tex.test.ts` (3), `codelens.test.ts` (3),
  `palette-filter.test.ts` (3), `symbols.test.ts` (1).
- `examples/`: file names (`ex1-38-N1`, `ex1-41-N4`, `ex1-56`, `ex1-58`, `ex1-66`, `fig1-6-M1`,
  `lemma1-55`, `thm1-25-*`, `thm1-45-union`, `thm1-47-concat`, `thm1-49-star`) and their header
  comments.
- ts-rs doc comments copied into `web/src/engine/generated/*.ts` (7 files).

---

## 4. Error codes in use (to renumber, 04 §1)

v1 uses 51 codes. The v2 ranges are in 01 §13.

| v1 | Meaning in v1 | v2 range |
|---|---|---|
| E001–E002 | lexical | E0xx (same) |
| E101–E110, E114, E120–E123 | syntax | E1xx |
| W111, W112, W117, I115, I116 | syntax warnings / info | no v2 range defined (Q-45) |
| E201–E205, W201 | machine field rules (Q, Σ, start, F) | **E3xx** (v2 E2xx is names/types) |
| E301, W302, I304 | δ rules (total, deterministic) | E3xx |
| E401–E404, E408, W405, W406 | regex | E4xx |
| E801–E806, E809, W807, I811 | names, calls, arguments | **E2xx** |
| R901, R902, R910 | runtime: input symbol, budget, run target | E5xx / **B6xx** for R902 |
| X999 | internal | X999 |

---

## 5. Test inventory (at `v1.0-final`)

| Suite | Count | What it covers | v2 fate |
|---|---:|---|---|
| `delta-core` unit | 26 real + 18 ts-rs exports | bitset (3), alphabet (1), regex arena (4), subset (4), regex→NFA (4 incl. golden `(0∪1)*010*` and Examples 1.56/1.58 state counts), GNFA (1), state elimination (3), product (4), closure (6, oracle to length 8, Exercise 1.15), NFA run sentences (1) | **Keep**; re-key off book names |
| `delta-core` proptest | 2 × 200 cases | regex→NFA→DFA agree ≤ 6; full round trip, two modes and two rip orders | **Keep** (P2 exit) |
| `delta-dsl` e2e | 21 | golden examples, Example 1.41/1.56/1.58/1.66 outputs, three δ forms equal, strict vs lenient, DFA rule violations, fixes, regex shorthands and `+` warning, input symbols, suggestions, recovery, UTF-16 spans, no panic on garbage, closures | **Re-express in v2 syntax** (P2 exit: "all v1 golden tests pass") |
| `delta-dsl` lexer | 3 | edges/symbols, unclosed string, Unicode = ASCII | Re-express |
| `delta-dsl` guide | 2 | every block of both AI guides runs | Becomes the docs CI rule |
| ts-rs export stubs | 18 (core) + 10 (dsl) | type generation | Depends on Q-41 |
| Vitest | 25 in 8 files | tuple formatting, regex TeX, DOT/layout parsing, symbols, palette filter, highlighting, codelens, conversions | Keep most |
| Playwright | 14 × 3 viewports | tour render, NFA→DFA→regex flow, run stepping, ε cell, diagnostics on edit, palette, gallery, Convert buttons, symbol bar, credit, Auto/Execute/stale, slow loader + Stop, no loader flash, boxed regex→NFA | Keep the flows; extend to 6 viewports |
| **v2 conformance (new, Phase 0)** | 43 cases | every example in `01-LANGUAGE.md` | `conformance/`. All expected to fail until Phase 1+ (see its README) |

Golden expectations worth preserving word for word: Example 1.41 gives 8 subsets in full mode and
6 reachable ones; Example 1.56 gives 8 states; Example 1.58 gives 12; `(0∪1)*010*` gives its exact
step-title sequence; Example 1.66 rip 2-then-1 gives `a*b(a∪b)*`; Exercise 1.15 (naive star is
wrong).
