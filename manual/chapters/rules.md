---
title: "9. Rules and decisions"
description: The hard rules and why each exists, where decisions are recorded, and ideas that were dropped.
---

Readers: **owner**, **contributor**.

## Hard rules

- **`v2` never merges into `main` without the owner.** v1 stays live and recoverable until the
  v2 beta is accepted.
- **Spec first.** Behavior is defined in `docs/v2/` and a conformance case before code, so the
  implementation can be checked against something other than itself.
- **The engine never panics.** It runs on arbitrary input in a user's browser; every failure is
  a diagnostic or a budget outcome.
- **Every algorithm runs under a budget.** Many questions are undecidable and many constructions
  are exponential; budgets make "too much" an answer instead of a hang.
- **Undecidable questions are rejected, not attempted.** Honesty is the product: E701 plus a
  bounded alternative.
- **No textbook in the core.** Algorithms have neutral ids; citations live in explain profiles,
  keyed by id, so other textbooks can be supported and copyright stays clean.
- **The textbook PDF is never committed.** It's copyrighted; `/*.pdf` is gitignored and
  `git status` is checked before every commit.
- **Engine calls go through the Web Worker.** The UI must never block.
- **No new dependency without the owner.** Each one is approved by name and version.
- **The manual is private.** Access-protected, noindex, never linked, never a custom domain.

## Where decisions live

- `docs/v2/README.md` §3: the locked decisions (syntax, engine, hosting, look).
- `docs/v2/QUESTIONS.md`: every open question with a recommended default and a status
  (PROPOSED / CONFIRMED / open), e.g. Q-76 (the false example), Q-43 (crate cycle).
- `docs/v2/PROGRESS.md` "Decisions": what was decided in each session, marked (C) when the
  assistant decided and the owner can revisit.

## Dropped ideas

| Idea | Why dropped |
|---|---|
| Indentation-based syntax as an alternative | Two syntaxes double the parser, formatter and docs; braces paste safely into chat and slides |
| Flutter / React Native apps | A second UI codebase; Capacitor wraps the same React Pad |
| A pure TypeScript engine | 5–20× slower on heavy conversions and no typed ids (v1 plan); kept only as a fallback behind `EngineApi` |
| GitHub Pages hosting | Actions was locked on the account; Cloudflare Pages takes direct uploads |
| `delta` as the binary name | Clashes with git-delta |
| clap, viuer, layout-rs | Own small code instead: hand-parsed arguments, own image protocols, own layout |
| `im` persistent collections, a `delta-classes` crate | The plan named them; the implementation uses `Rc` + std collections and keeps classes in `delta-lang/src/lang.rs`, which also avoided the crate cycle |

## Questions they'll ask

1. **"What's a decision you reversed?"** — Hosting: GitHub Pages → Cloudflare Pages when Actions
   was locked. Or the CLI name. Give the trigger, the options, the cost.
2. **"How do you decide what not to build?"** — Each phase has a done-definition; anything else
   goes to QUESTIONS.md or the roadmap's later list (e.g. accounts, API, MCP server).
3. **"Why such strict rules for a side project?"** — The product's value is being *right*
   about theory; the rules (spec first, no panics, budgets, no fake answers) are how it stays
   right as it grows.
4. **"Why keep citations out of the code?"** — Copyright, and neutrality: the same algorithm id
   can point to Sipser, Hopcroft–Ullman or Linz through different explain profiles.
5. **"Who decides?"** — The owner; the assistant may decide obvious things under the plan's
   principles and must list them (the (C) marks).

## Self-test

<details class="selftest"><summary>1. Why are budgets a hard rule?</summary>Undecidable questions and exponential constructions; budgets turn "too much" into an outcome instead of a hang or crash.</details>
<details class="selftest"><summary>2. Where are open questions tracked?</summary><code>docs/v2/QUESTIONS.md</code>.</details>
<details class="selftest"><summary>3. Name two dropped ideas and why.</summary>e.g. indentation syntax (two syntaxes); Flutter (second codebase); GitHub Pages (Actions locked); "delta" name (git-delta).</details>
<details class="selftest"><summary>4. What does (C) mean in PROGRESS.md?</summary>Decided by the assistant (Claude) under the plan's principles; the owner can revisit.</details>
<details class="selftest"><summary>5. Why is the PDF in the repo root never committed?</summary>It's a copyrighted textbook; <code>/*.pdf</code> is gitignored and status is checked before commits.</details>
