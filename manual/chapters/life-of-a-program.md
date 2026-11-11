---
title: "3. Life of one program"
description: "How it works: follow `check EndsAB ≡ L` from a keystroke to the Tests panel, through every real file and function."
---

Readers: **all**. This is the chapter to know cold: most technical interviews end up here.

## The big picture

```text
 Browser main thread                         Web Worker                    Rust (WASM)
 ───────────────────                         ──────────                    ───────────
 CodeEditor ─setSource→ useUiStore
 App.tsx (150 ms debounce)
   → useEngineStore.evaluate ─Comlink→ worker.ts evaluate ─→ delta_wasm::evaluate
                                                              → pad::document
                                                                → execute: parse → check → run
                                                                → JSON (objects, tests, outputs)
   ← JSON.parse ←──────────────────────────── string ←──────────┘
 TestsPanel / DiagramPanel / Console render the Document
```

Text version: the editor writes the source into a Zustand store; 150 ms after typing stops, the
store asks the engine client to evaluate it. The client sends the source to a Web Worker through
Comlink; the worker calls the Rust function `evaluate`, compiled to WebAssembly, which parses,
checks and runs the whole program and returns one JSON document. The store keeps that document,
and every panel renders from it.

## The program

```delta
dfa EndsAB {
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q2
  q2 -a-> q1;   q2 -b-> q0
}
lang L = { w ∈ {a,b}* | w ends with "ab" }
check EndsAB ≡ L up to 10
```

We follow line 9 in three versions, because the interesting thing is how the *same syntax*
takes three different paths:

| Version | Path | Result |
|---|---|---|
| `check EndsAB ≡ L` | stopped by the **checker** before anything runs | E701, with a quick fix `up to 12` |
| `check EndsAB ≡ L up to 10` (the tour) | **bounded** comparison of every word up to length 10 | pass, `bounded: 10` |
| `check EndsAB ≡ re"(a ∪ b)* a b b"` | **exact** decision: subset automata, breadth-first over pairs | fail, witness `"ab"` |

## 1. Keystroke → store → worker

CodeMirror's update listener (`web/src/editor/CodeEditor.tsx`) calls `setSource` on every
change. `App.tsx` waits until typing stops for 150 ms (when Auto is on), then evaluates:

<!-- excerpt: web/src/App.tsx#pad-debounce -->

The store gives each evaluation a ticket, so a slow, stale result can never overwrite a newer
one, and shows the loader only if a run takes longer than `SLOW_MS`:

<!-- excerpt: web/src/state/engine.ts#store-evaluate -->

`engine.evaluate` (`web/src/engine/client.ts`) races the worker call against a 10-second timer
and the worker's crash promise. On timeout it terminates the worker and starts a new one — the
only way to stop WebAssembly. The worker itself is small:

<!-- excerpt: web/src/engine/worker.ts#worker-evaluate -->

`evaluate` is `crates/delta-wasm/src/lib.rs`: it calls `delta_lang::pad::document(src)` and
serializes the result to a string (one copy across the boundary, parsed once in the worker).

## 2. Lexer: two modes

`crates/delta-lang/src/lexer.rs`. The parser asks the lexer for each token in a **mode**:
*normal* for ordinary code, *machine* inside a machine body, where `-a->` and `-[ … ]->` are
single edge tokens and bare names like `q0` are atoms. That's why `q0 -a-> q1` needs no quotes
or colons. Unicode and ASCII spellings lex to the same operator; `≡` and `equiv` are both
`Op::Equiv`:

<!-- excerpt: crates/delta-lang/src/lexer.rs#lexer-symbols -->

## 3. Parser: precedence and recovery

`crates/delta-lang/src/parser.rs`, `parse(src, extra_models)`. Statements are recursive
descent; expressions are Pratt parsing with a binding-power table. `≡` binds like comparison,
looser than `∪`, so `A ∪ B ≡ C` means `(A ∪ B) ≡ C`:

<!-- excerpt: crates/delta-lang/src/parser.rs#parser-equiv -->

`check_stmt` reads `check`, the subject expression, and an optional `up to n` into
`StmtKind::Check(subject, CheckKind::Holds(bound))`. On a syntax error the parser records a
diagnostic and **recovers** to the next statement boundary, so one bad line never blanks the
rest of the output (see [Language design](/language-design/)).

## 4. Checker: the class lattice, before running

`crates/delta-lang/src/check.rs`, `check(&program, src, models)`. The checker infers each
name's **class** without running: `dfa EndsAB` is Regular (from the prelude model), and
`lang L = { w ∈ {a,b}* | … }` is Decidable (a predicate over Σ*). For `≡` and `⊆` it asks the
decision table:

<!-- excerpt: crates/delta-lang/src/check.rs#checker-equiv -->

<!-- excerpt: crates/delta-lang/src/check.rs#checker-question -->

`lang::decidable` is a small table (`crates/delta-lang/src/lang.rs`), shown in the
[lattice deep dive](/theory/09-lattice/). Regular ⊔ Decidable = Decidable, and equivalence of
decidable languages is undecidable, so **version 1 stops here** with:

```json
{ "code": "E701", "line": 9,
  "message": "equivalence of decidable languages is undecidable (the operands are Regular and Decidable)",
  "fixes": [{ "title": "use a bounded check: up to 12", "edits": [{ "insert": " up to 12" }] }] }
```

The fix is an edit the Pad offers as a quick fix. With `up to 10` the checker sets its
`bounded` flag and skips the question.

## 5. Interpreter → `check`

`crates/delta-lang/src/lib.rs`, `execute`: if there are no errors, the tree-walking interpreter
runs the program top to bottom.

<!-- excerpt: crates/delta-lang/src/lib.rs#pipeline -->

`StmtKind::Check` goes to `Interp::check` in `crates/delta-lang/src/hooks.rs`. Both sides become
language values (`to_lang`: a machine's language, a regex's language, or the `lang` value
itself), then the bound decides the path:

<!-- excerpt: crates/delta-lang/src/hooks.rs#interp-check-holds -->

## 6a. Bounded path (version 2)

`Interp::bounded` enumerates words in shortlex order, layer by layer, and compares membership
on both sides. For Σ = {a, b} and n = 10 that's 2¹¹ − 1 = 2,047 words. Membership in `EndsAB`
runs the machine (the finite-automaton fast path in `sim.rs`: sets of states); membership in
`L` evaluates the predicate `w ends with "ab"`. Each step ticks the budget.

<!-- excerpt: crates/delta-lang/src/hooks.rs#bounded-check -->

All 2,047 agree: `{ "kind": "test", "name": "EndsAB ≡ L up to 10", "pass": true, "bounded": 10 }`.
The whole tour program took 12,297 engine steps (measured 10 NOV 2026).

## 6b. Exact path (version 3)

`Interp::decide` re-checks decidability at run time (classes can be computed values), compiles
both sides to finite automata over the joint alphabet (`compile_fa`: regexes go through an NFA
construction, other sources through `determinize::subset`), and runs the equivalence search:

<!-- excerpt: crates/delta-lang/src/hooks.rs#hooks-decide -->

The search is a breadth-first walk over the **product of the two subset automata**, built on
the fly, with symbols in order — so the first pair where exactly one side accepts gives the
shortlex-least counterexample:

<!-- excerpt: crates/delta-core/src/fa/decide.rs#equivalence-search -->

For `re"(a ∪ b)* a b b"`, the pair reached by `ab` has `EndsAB` accepting and the regex not, so
the result is `{ "pass": false, "witness": "ab", "message": "they differ on \"ab\"" }`.

## 7. JSON → panels

`pad::document` (`crates/delta-lang/src/pad.rs`) turns the run into the Pad's document: named
objects with their diagram and table views, diagnostics with UTF-16 offsets (CodeMirror counts
UTF-16), the outline for completion, and the outputs, grouped by kind:

<!-- excerpt: crates/delta-lang/src/pad.rs#pad-tests -->

The worker parses the JSON; the store saves it; React re-renders. `TestsPanel`
(`web/src/panels/TestsPanel.tsx`) shows each test with ✓ or ✗, a `bounded ≤ 10` chip, and the
counterexample:

<!-- excerpt: web/src/panels/TestsPanel.tsx#tests-counterexample -->

The same JSON (the `run_json` variant, schema at `/schema/output-v2.json`) is what
`deltac run --json` prints, so the CLI, the Pad and the conformance suite agree by construction.

## Glossary

- **Pad**: the browser playground (`web/`), served at `/pad/`.
- **Model**: a kind of machine, defined by storage and acceptance (`dfa`, `pda`, `tm`, …); the
  standard ones are in `prelude.delta`.
- **Class**: a language's place in the lattice Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂
  Recognizable ⊂ Any.
- **Bounded check**: `up to n`, a comparison on all words of length ≤ n. Evidence, not proof.
- **Witness / counterexample**: the shortlex-least word on which two languages differ.
- **Budget**: limits on steps, states, recursion depth and output; hitting one is an outcome
  (B6xx), not a crash.
- **Shortlex**: order by length, then alphabetically — the order of breadth-first search.
- **Conformance suite**: `conformance/cases/*.delta` with expected JSON; every spec example.
- **Document**: the JSON the Pad receives for one evaluation.

## Questions they'll ask

1. **"Why does the engine run in a Web Worker?"** — So typing never blocks: a subset
   construction can take seconds. And because WASM can't be interrupted, a worker is the unit you
   can kill: the client terminates it after 10 s and spawns a new one.
2. **"Why JSON across the boundary instead of shared objects?"** — One serialized string per
   evaluation is simple, debuggable and versioned (the same schema as the CLI). The documents are
   small; copying is cheaper than designing a shared-memory protocol.
3. **"How do you avoid stale results when the user types fast?"** — The debounce (150 ms) plus
   a ticket per evaluation: a finished evaluation whose ticket isn't the latest is dropped.
4. **"Why is the counterexample the shortest one?"** — Breadth-first search over pairs, symbols
   in index order, parent pointers to rebuild the word: the first bad pair dequeued is at the
   minimal depth, and among those the first in lexicographic order.
5. **"Why both a static and a run-time decidability check?"** — The checker catches what it can
   infer without running (literal machines, regexes, set-builders) and can offer a quick fix in
   the editor. Classes can also come from computed values (`let M = f(x)`), so `decide` checks
   again at run time.
6. **"What's the complexity of the exact check?"** — Product of the two subset automata: at most
   2^|A| · 2^|B| pairs times |Σ| steps; for two DFAs it's |A|·|B|·|Σ|. Bounded by the state
   budget (100,000 by default), which gives a B6xx outcome, never a hang.
7. **"How are line and column numbers right for Unicode?"** — Spans are byte offsets in Rust;
   `pad.rs` converts them to UTF-16 offsets because CodeMirror and JavaScript strings count
   UTF-16 code units.

## Self-test

<details class="selftest"><summary>1. List the hops from keystroke to Rust, naming files.</summary>CodeEditor.tsx (setSource) → App.tsx (150 ms debounce) → state/engine.ts (evaluate, ticket) → engine/client.ts (Comlink, 10 s timeout) → engine/worker.ts → crates/delta-wasm (evaluate) → delta-lang pad::document → lib.rs execute.</details>
<details class="selftest"><summary>2. Which component rejects `check EndsAB ≡ L` and why?</summary>The checker (check.rs, via lang::decidable): Regular ⊔ Decidable = Decidable, and equivalence is undecidable there → E701 with the fix "up to 12".</details>
<details class="selftest"><summary>3. How many words does `up to 10` compare over {a, b}?</summary>2¹¹ − 1 = 2,047 (lengths 0 through 10).</details>
<details class="selftest"><summary>4. What is searched in the exact path?</summary>Pairs (subset of A's states, subset of B's states), ε-closed, breadth-first with symbols in order, until a pair where exactly one side accepts.</details>
<details class="selftest"><summary>5. What makes the CLI and the Pad agree?</summary>Both use the same engine and output schema; the conformance suite checks `deltac run --json` against every spec example.</details>
