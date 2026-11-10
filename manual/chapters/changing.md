---
title: "8. Changing it"
description: Dev setup, layout, conventions, tests, recipes and the release process.
---

Readers: **contributor**.

## Setup

Rust (stable, via rustup) with the `wasm32-unknown-unknown` target; Node 22 via `fnm`; pnpm.
Claude Code's shell may need: `source "$HOME/.cargo/env"; eval "$(fnm env --shell zsh)"; fnm use 22`.

## Branches and sources of truth

- Work on `v2`; never merge to `main` without the owner. `v1.0-final` tags the last v1.
- `docs/v2/` is the spec: `01-LANGUAGE` (syntax), `SEMANTICS` (meaning), `02-ENGINE`,
  `03-PRODUCTS`, `04-ROADMAP`, `QUESTIONS` (open decisions), `PROGRESS` (state and "start here").
- **Spec first:** new syntax or semantics changes `docs/v2/` and adds a conformance case in the
  same commit as the code.

## Conventions

- "Simple Rust": index ids, `Vec`s, owned data, one algorithm per file, no `unsafe`, no
  lifetimes in public types.
- No panics in engine crates: `unwrap`, `expect` and `panic!` are denied by Clippy. Errors are
  values.
- Every algorithm has a stable id in its doc comment (`determinize/subset`, `minimize/hopcroft`).
- No textbook words in the code (citations live in explain profiles — and in this manual).
- The owner is learning Rust: the first use of each Rust concept in a file gets a one-line
  comment.
- Manual excerpts: wrap code you want quoted in `// manual:begin NAME` … `// manual:end NAME`
  (in JSX: `{/* manual:begin NAME */}`). The manual build fails if a quoted marker disappears.

## Tests (all must pass before a commit)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                 # includes conformance
./scripts/build-wasm.sh
./scripts/check-wasm-size.sh
cd web && pnpm lint && pnpm typecheck && pnpm test && pnpm build && pnpm test:e2e
```

As of 10 NOV 2026: 145 Rust tests, 54 conformance cases (all pass, `xfail.txt` empty), 26
Vitest, 90 Playwright (phone, tablet, desktop).

## Recipe: add a built-in function

1. Write the algorithm in `crates/delta-core/src/fa/<file>.rs` with its id and a test against
   a brute-force oracle on short words.
2. Expose it in `crates/delta-lang` (`builtins.rs` or `hooks.rs`), with a signature record in
   `docs.rs` so the library page and the AI guide list it.
3. Add an example to `01-LANGUAGE.md` §12 if it's user-visible, and a conformance case.
4. Run `cargo run -p delta-cli -- gen-docs site` and commit the generated pages.

## Recipe: add an error code

1. Pick the range (E0 lexical … E7 decidability), emit it with `Diagnostic::error`.
2. Add an `ErrorDoc` in `docs.rs` with an example that produces it — a test runs every example.
3. `gen-docs` writes `site/src/content/docs/docs/errors/<code>.md`.

## Recipe: add a statement (how `save` was added)

`docs/v2/01-LANGUAGE.md` table and grammar → `SEMANTICS.md` §10.4 → conformance cases 51–54 →
lexer keyword → `StmtKind::Save` → parser → checker and the two AST walkers in `interp.rs` →
interpreter output → schema in `docs.rs` → hosts (CLI writes files, Pad downloads).

## Release process

A phase ends with all checks green, a commit, a push to `v2`, a short summary, and an update of
`PROGRESS.md` ("Next session: start here"). Deploys happen when the owner says so (see
[Operating it](/operating/)).

## Questions they'll ask

1. **"How do you keep docs and code in sync?"** — Generated where possible (library, errors, AI
   guide, schema, social cards via `gen-docs`; CI fails if they're stale), every docs code block
   is executed by `tests/docs.rs`, and the manual's excerpts come from markers.
2. **"What's a conformance case?"** — A `.delta` file plus a partial-match JSON expectation; the
   harness runs `deltac run --json`, kills it after 10 s, and passes iff exactly the cases in
   `xfail.txt` fail, so that list only shrinks.
3. **"How do you test an algorithm like Hopcroft?"** — Against a brute-force oracle (all words up
   to a length), and against the three other minimization algorithms (they must agree).
4. **"Why deny unwrap?"** — The engine runs in a user's browser on arbitrary input; a panic would
   kill the worker. Errors as values keep every failure a diagnostic.
5. **"How do you review AI-written code?"** — Spec first, then conformance and oracle tests that
   the code must pass, Clippy at `-D warnings`, and reading every diff; the manual's excerpts are
   the parts worth knowing line by line.

## Self-test

<details class="selftest"><summary>1. What three things change together when syntax changes?</summary>docs/v2 (spec), a conformance case, and the code — in one commit.</details>
<details class="selftest"><summary>2. What makes the conformance list "only shrink"?</summary>The harness fails if a case listed in xfail.txt starts passing.</details>
<details class="selftest"><summary>3. Where do error docs come from?</summary><code>ErrorDoc</code> entries in docs.rs, written by gen-docs; each example is run by a test.</details>
<details class="selftest"><summary>4. Which lints are hard errors in engine crates?</summary>unwrap_used, expect_used, panic (and unsafe_code is forbidden).</details>
<details class="selftest"><summary>5. What does the manual build check about excerpts?</summary>Every referenced file has its begin/end markers; otherwise the build fails.</details>
