# Delta 2 — Progress

_Last updated: 2026-11-08 (Phase 0, second session: ten answers confirmed)_

## Current state

| | |
|---|---|
| Branch | `v2` (long-lived; **never merge into `main`** until the v2 beta) |
| v1 | Tag `v1.0-final` = `50bf153` (= `origin/main` at the time, incl. both CNAME commits). v1 stays live from `main` |
| Phase | **0 — Audit and spec freeze: produced, awaiting the owner's review.** The spec is **not frozen** until the open questions are confirmed |
| Conformance | 50 cases (44–50 added for the confirmed answers), **all 50 listed in `conformance/xfail.txt`** (expected to fail: no v2 language yet) |
| v1 tests | All pass (79 Rust + 3 conformance-harness tests; 25 Vitest; 14 Playwright × 3 viewports) |

## What Phase 0 produced

| File | Contents |
|---|---|
| `docs/v2/` (01–04, README) | The Delta 2 plan, moved here from an untracked `delta-v2-plan/` folder. §6.4 fixed (Q-76) |
| `docs/v2/AUDIT.md` | Keep/change/drop for every v1 module; all book strings; coupling problems; test inventory; v1 → v2 error-code mapping |
| `docs/v2/SEMANTICS.md` | Formal meaning of configurations, label ops, every model property, acceptance modes, languages/classes, `explore`, `fixpoint`, the core and desugaring. Provisional choices tagged `[Q-nn]` |
| `docs/v2/QUESTIONS.md` | ~70 questions/contradictions, each with a recommended default and the phase it blocks. Top section: proposed answers |
| `conformance/` | 43 cases (`cases/NN-name.delta` + `.expect.json`), `xfail.txt`, README with the matching rules |
| `crates/delta-cli/tests/conformance.rs` | Harness: runs `delta run --json` on each case, partial-matches the expectation, enforces the xfail list |
| `CLAUDE.md` | Updated for v2 (sources of truth, citation rule, layout) |

## Decisions

**Made**
- Tag `main` as `v1.0-final`; work on the `v2` branch (04 §1).
- Q-76: the §6.4 example is now the (true) "even number of 0s" claim. The old false claim lives on
  as conformance case 19, which must fail with an error.
- Q-90: conformance layout and the xfail-ledger approach (CI green, list only shrinks).
- Q-91: local `main` was fast-forwarded to `origin/main` before tagging.

**Confirmed 2026-11-08** (QUESTIONS.md, "Confirmed answers"; applied to 01, SEMANTICS and
conformance): Q-01 (newline or `;`, everywhere; also answers Q-02), Q-03 (machine-body lexing mode,
edge tokens), Q-06 (conversions are verbs: `to_nfa`, `to_gnfa`, `determinize`), Q-08
(`contains`), Q-26 (states are atoms, `:q0`; atom-keyed maps), Q-44 (citations only in
`delta-explain`), Q-62 (empty stack, option `initial stack Z0`), Q-69 (verdicts accept / reject /
loops / unknown), Q-75 (Finite and Any columns, `F ∩ L` / `F ∖ L` rules), Q-80 (`explore` needs
`alphabet`).

**New, raised while applying them:** Q-92 (`F ∩ L` for an Any L), Q-93 (option syntax).

**Still open:** every other question in QUESTIONS.md. These block Phase 1: Q-04, Q-05, Q-07,
Q-09–Q-15, Q-17–Q-25, Q-27, Q-28, Q-39 (new dependencies: indexmap, im, schemars), Q-40, Q-41,
Q-43, Q-45–Q-48, Q-93.

## Key findings of the audit

1. **v1 has no minimization at all** (no Hopcroft, Moore, Brzozowski or table method), although
   02 §2.2 and 04 §1 say it is ported from v1 (Q-50).
2. **v1 has no decision procedures**: no equivalence, inclusion, emptiness, finiteness,
   universality, shortest/words/count; no complement, difference, reverse, trim, complete,
   remove_epsilon; no Thompson, Glushkov, Arden, Brzozowski–McCluskey or simplify (Q-51). The
   §1 Taste example already needs equivalence.
3. **Estimate impact: add about 8–15 h to Phases 2–3** for that new algorithm work.
4. **Planned crate cycle** (Q-43): `Machine.states: IndexSet<Value>` puts the interpreter's
   `Value` in `delta-core`, and `LangSource::Predicate(Closure)` puts an interpreter closure in
   `delta-classes`, which `delta-lang` depends on. **Proposed fix:** `Value` lives in
   `delta-core` (functions as opaque ids); `delta-classes` asks for predicate membership through a
   trait that `delta-lang` implements.
5. The algorithms port well (small, budgeted, traced, oracle-tested). The model layer does not:
   three unrelated structs (`Dfa`/`Nfa`/`Gnfa`) become one generic `Machine`.
6. 226 book citations in 50 files, about 60 user-visible at runtime, through two channels (`&'static
   str` rule names and `BookRef`). Stable rule/algorithm ids must be introduced before the text can
   move to `delta-explain`.

## Commands

Prefix if Rust or Node 22 aren't on PATH:
`source "$HOME/.cargo/env"; eval "$(fnm env --shell zsh)"; fnm use 22`

```bash
cargo test --workspace                                   # everything; conformance passes iff exactly xfail.txt fails
cargo test -p delta-cli --test conformance -- --nocapture # prints "conformance: N of 43 cases pass"
cargo test -p delta-cli --test conformance -- --ignored  # strict: every case must pass (shows each failure)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The full check list (wasm build, size check, web lint/typecheck/test/build/e2e) is in `CLAUDE.md`.

## Next session: start here

1. Read `CLAUDE.md`, then this file, then `docs/v2/QUESTIONS.md`.
2. ~~Confirm the ten proposed answers~~ (done 2026-11-08). The owner has the list of remaining open
   questions, each marked "real choice" or "obvious default", and is ruling on them.
3. Resolve the remaining Phase 1 blockers (list above), especially Q-39 (dependencies) and Q-43
   (crate layout). Both need the owner's OK under the working agreement.
4. Apply each newly confirmed answer the same way: `01-LANGUAGE.md`, `SEMANTICS.md` (remove the
   `[Q-nn]` tag), the conformance cases (new ones go into `xfail.txt`), and the "Confirmed answers"
   table plus a ✅ on the row in QUESTIONS.md.
5. Complete the EBNF in 01 §15 (Q-14). Then declare the **spec freeze**: Phase 0 is done.
6. Only after an explicit go-ahead: start Phase 1 (language core). Its exit criterion includes the
   P1 conformance cases leaving `xfail.txt`.
