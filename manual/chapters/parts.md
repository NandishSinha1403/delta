---
title: "4. Tour of the parts"
description: Every component of Delta — what it does, why it exists, where it lives, what goes in and out, how it fails.
---

Readers: **contributor**, **owner**.

```text
            ┌──────────── delta-lang ────────────┐
 source ──→ │ lexer → parser → checker → interp  │ ──→ JSON (output schema v2)
            │            ↓ uses                  │
            │  delta-core (fa: subset, minimize, │
            │  decide, product, regex, to_regex) │
            └──────┬──────────────┬──────────────┘
                   │              │
        delta-wasm (Pad)    delta-cli (deltac) ──→ delta-render ──→ delta-export (PNG/PDF)
                   │                                     ↑
               web/ (React)  ── delta-export-wasm ───────┘   site/ (Astro) ← gen-docs
```

Text version: `delta-lang` is the language; it uses `delta-core`'s automata algorithms. Two
front ends wrap it: `delta-wasm` for the browser and `delta-cli` for the terminal. Drawing to
files is `delta-render` (layout, SVG, TikZ, text) and `delta-export` (PNG/PDF), used by the CLI
and, through `delta-export-wasm`, by the Pad. The docs site is generated partly by `deltac
gen-docs`.

| Part | What it does | Where | In → out | How it fails |
|---|---|---|---|---|
| **delta-core** | Index-based automata (`Fa`: states `0..n`, symbols `0..k`, ε = `None`) and the algorithms: subset, four minimizations, product, decisions, regex↔NFA, ops. v1's `Dfa`/`Nfa`/`Gnfa` remain for the v1 checker | `crates/delta-core/src/fa/` | `Fa` → `Fa`/regex/word | `Err(max_states)` when a budget is hit; never panics (Clippy denies `unwrap`) |
| **delta-lang** | Lexer (2 modes), parser (recursive descent + Pratt, recovery), checker (names, types, classes, E7xx), tree-walking interpreter, prelude models, generic simulator, languages and classes, Earley for grammars, the Pad document, generated docs | `crates/delta-lang/src/` | source → `Execution` / JSON | Diagnostics (E0xx–E7xx), budget outcomes (B6xx); a Rust panic would be X999 and is a bug |
| **delta-render** | Layered layout (`render/layered`), SVG, TikZ, text listing. No dependencies | `crates/delta-render/` | `Graph` → `Layout` → text | Never fails; an empty graph is an empty picture |
| **delta-export** | SVG → PNG (resvg) and PDF (svg2pdf) with embedded Commit Mono + Noto Sans Math subsets | `crates/delta-export/` | SVG → bytes | `Err(String)` on bad SVG; PNG capped at 8,192 px a side |
| **delta-wasm** | The browser API: `evaluate`, `run`, `run_json`, `render_file`, `version` | `crates/delta-wasm/` | strings → JSON strings | A thrown JS `Error` |
| **delta-export-wasm** | `png`, `pdf` for the Pad, loaded on the first download | `crates/delta-export-wasm/` | SVG → `Uint8Array` | Thrown error → "Couldn't make …, try again" |
| **delta-cli** (`deltac`) | `run`, `repl`, `gen-docs`; v1 `check`/`json` | `crates/delta-cli/` | files → stdout, files | Exit 1 on errors or failed tests, 2 on bad usage |
| **delta-dsl** | The v1 language (kept until nothing uses it) | `crates/delta-dsl/` | — | — |
| **The Pad** | Editor, diagram, tuple, run, steps, tests, problems, console; share links; PWA | `web/` | source → panels | Error boundary per panel; worker restart on timeout |
| **The site** | Front page, docs, tools, learn, practice; generated reference and error pages | `site/` | Markdown → static HTML | `check-seo.mjs` and `tests/docs.rs` fail the build |
| **Conformance** | Every spec example with expected JSON | `conformance/` | `.delta` → JSON match | Fails unless exactly `xfail.txt` fails |
| **This manual** | Private Starlight site | `manual/` | chapters + excerpts → HTML | Missing excerpt marker or secret-shaped string fails the build |

## The generic simulator

`crates/delta-lang/src/sim.rs` runs **every** model, including user-defined ones, the same way:
breadth-first search over configurations (state, head, storage, output) with a visited set.
Finite automata take a fast path (sets of states), and tests check the two agree. See the
[TM simulation deep dive](/theory/08-tm-simulation/).

## The prelude

`crates/delta-lang/src/prelude.delta` defines the 14 standard models in Delta itself, e.g. a
`dfa` is `input: read-once; storage: none; accept: by-final-state; deterministic; total`. Users
can define new models the same way.

## Questions they'll ask

1. **"Why so many crates?"** — Dependency direction and build targets. `delta-core` has no
   language concepts; `delta-render` has no dependencies so it compiles everywhere; `delta-export`
   is heavy (resvg, svg2pdf), so the main WASM must not depend on it — it's a separate,
   lazily-loaded module.
2. **"What does 'index-based' buy you?"** — Plain `Vec`s and `usize` ids instead of pointers or
   lifetimes: simple Rust, cheap clones, easy hashing of subsets (a sorted `Vec<usize>`), and no
   borrow-checker fights. The language layer maps state *values* to indexes and back.
3. **"How does a user-defined model run?"** — The model is a descriptor (input mode, storage
   kinds, acceptance, determinism); labels are lists of operations (read, push, pop, write, move);
   the generic simulator applies operations to configurations. No per-model code.
4. **"Where would you add a new algorithm?"** — A file in `delta-core/src/fa/` with a stable id
   in its doc comment, a built-in in `delta-lang/src/builtins.rs`/`hooks.rs`, a docs signature
   record, and a conformance case. See [Changing it](/changing/).
5. **"Why is v1 still in the tree?"** — The `check`/`json` commands and some types still use it;
   it goes when nothing does (AUDIT.md lists what's kept).

## Self-test

<details class="selftest"><summary>1. Which crate has no dependencies, and why does that matter?</summary>delta-render; it compiles into the CLI, the main WASM and the docs generator without pulling anything in.</details>
<details class="selftest"><summary>2. Why is PNG/PDF export a separate WASM module?</summary>resvg + svg2pdf + fonts are 887 KB gzipped; loading them only on the first download keeps the engine at 298 KB.</details>
<details class="selftest"><summary>3. What does the generic simulator search over?</summary>Configurations (state, head position, storage contents, output), breadth-first, with a visited set.</details>
<details class="selftest"><summary>4. What exit codes does <code>deltac run</code> use?</summary>0 success; 1 errors or failing tests; 2 bad usage.</details>
<details class="selftest"><summary>5. Where are the standard models defined?</summary>In Delta, in <code>crates/delta-lang/src/prelude.delta</code> (14 models).</details>
