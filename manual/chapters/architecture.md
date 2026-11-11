---
title: "Architecture decisions and trade-offs"
description: "Rust/WASM vs TypeScript, not Flutter, Cloudflare vs GitHub Pages, the crate cycle, budgets vs halting, the deltac name."
---

Readers: **owner** (system design interviews). For each: the choice, the alternatives, the
trade-off, and what would change the decision.

## Rust → WebAssembly engine, not TypeScript

- **Choice:** Rust compiled to WASM, in a Web Worker, behind one interface (`EngineApi`).
- **Alternatives:** a TypeScript engine (simplest); a server.
- **Why:** constructions are exponential in the worst case; the v1 plan estimated a TS engine
  5–20× slower on heavy conversions, and Rust gives typed ids and "no panics" enforced by the
  compiler and Clippy. One engine serves the Pad, the CLI and later an API.
- **Cost:** a WASM build step, a size budget (298 KB gzipped of 700), JSON across the boundary,
  and no way to interrupt WASM (hence the worker watchdog).
- **Would change if:** the engine had to run where WASM can't — then the TS fallback behind
  `EngineApi`.

## Capacitor, not Flutter (or React Native)

- **Choice:** wrap the web Pad with Capacitor for Android/iOS (Phase 8), PWA first.
- **Why:** same React code and WASM engine; native plugins for files and the share sheet.
  Flutter means a second UI in Dart; React Native still needs a WebView for CodeMirror; Tauri
  mobile was younger.
- **Cost:** a web-feeling app; performance bound by the WebView.

## Cloudflare Pages, not GitHub Pages

- **Choice:** static hosting on Cloudflare Pages, direct upload with a pinned wrangler.
- **Why:** GitHub Actions was locked on the account, so the GitHub Pages workflow couldn't run;
  Pages accepts uploads from a laptop, has preview URLs, rollbacks, and Pages Functions (the private
  manual's password gate) on the same account.
- **Cost:** deploys depend on one machine until Actions returns (`deploy.yml` is ready on PR #6).

## The crate-cycle fix (Q-43)

- **Problem:** the plan put the interpreter's `Value` inside core machine types (`IndexSet<Value>`
  states) and an interpreter closure inside a `delta-classes` crate that `delta-lang` depends on —
  a cycle.
- **Proposed fix:** move `Value` into core and make classes ask membership through a trait.
- **What was built:** simpler. `delta-core`'s automata are **index-based** (`Fa`: states `0..n`,
  symbols `0..k`); the language layer maps values to indexes and back. Classes and tables live in
  `delta-lang/src/lang.rs`. No crate needs another's types in a loop, and core stays free of
  language concepts.

## Budgets vs the halting problem

- **Choice:** every computation has budgets (10 M steps, 100 k states, depth 400, 10 k outputs);
  hitting one is an outcome (B6xx) or a verdict (`unknown`); repeated configurations give
  `loops`.
- **Why:** you can't decide halting, so you bound work and say honestly what happened. A timeout
  outside the engine (the 10-second watchdog) is the second line.
- **Cost:** a correct program can hit a budget; users raise it with `budget { … }`.

## `deltac`, not `delta`

- **Choice:** the binary is `deltac`.
- **Why:** `delta` is git-delta, a widely installed diff pager (Homebrew formula `git-delta`,
  binary `delta`); installing both would shadow one on `PATH`.

## Own small code instead of dependencies

Hand-parsed CLI arguments (no clap), own kitty/iTerm2 escape sequences (no viuer), own layered
layout (no layout-rs). **Why:** each is under ~300 lines, keeps the binary and the WASM small,
and every dependency needs the owner's approval anyway.

## Questions they'll ask

1. **"Why a Web Worker?"** — Responsiveness and killability; the UI thread never runs engine code.
2. **"How would you add a server API?"** — The same WASM on Cloudflare Workers (`POST /run`,
   `/check`, `/render`) — no new engine, same JSON schema.
3. **"What breaks first at scale?"** — Nothing server-side (static). Per-user: big
   determinizations hit the state budget; long TM runs use memory.
4. **"Why JSON instead of a binary protocol?"** — Debuggable, versioned, shared with the CLI and
   conformance tests; payloads are small.
5. **"What's one decision you'd revisit?"** — Two renderers (Graphviz on screen, `delta-render`
   for files) until Phase 7, which moved the Pad onto `delta-render`: one layout, and 818 KB of
   lazy Graphviz JS gone.
6. **"How do you keep the WASM small?"** — `opt-level = "z"`, LTO, one codegen unit, `wasm-opt`,
   a CI size check, and heavy features (PNG/PDF) in a separate lazy module.

## Self-test

<details class="selftest"><summary>1. Two reasons for Rust/WASM?</summary>Speed on exponential constructions; typed ids and enforced no-panic; one engine for Pad, CLI, API.</details>
<details class="selftest"><summary>2. Why Cloudflare Pages?</summary>GitHub Actions was locked; direct upload, previews, rollback, Functions (the manual's password gate).</details>
<details class="selftest"><summary>3. How was the crate cycle avoided in practice?</summary>Index-based automata in core; values mapped to indexes in delta-lang; classes in delta-lang.</details>
<details class="selftest"><summary>4. What answers the halting problem in Delta?</summary>Nothing does — budgets bound work; loops only when a configuration repeats; otherwise unknown.</details>
<details class="selftest"><summary>5. Why not Flutter?</summary>A second UI codebase in Dart; Capacitor reuses the React Pad and WASM engine.</details>
