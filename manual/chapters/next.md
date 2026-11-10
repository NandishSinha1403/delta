---
title: "12. What's next"
description: The roadmap in plain words — done, in progress, later.
---

Readers: **owner**, **visitor**.

## Done

| Phase | What | When |
|---|---|---|
| v1 | Regular languages playground (DFA/NFA/regex, subset, product, closure) | 29 OCT 2026 |
| 0 | Audit of v1, formal semantics, ~90 questions, conformance suite | 08 NOV 2026 |
| 1 | Language core: lexer, parser with recovery, checker, interpreter | 08 NOV 2026 |
| 2–3 | Models, generic machines and simulator, languages, classes, `explore`, `fixpoint` | 08 NOV 2026 |
| 4 | The Pad on the v2 engine: share links, embeds, PWA, six viewports | 08 NOV 2026 |
| 5 | Front page, docs, SEO pages; live on Cloudflare at `delta.nandish.dev` | 09 NOV 2026 |
| 5b | `deltac`, `save`, the renderer, PNG/PDF export, REPL, cargo-dist | 09 NOV 2026 |
| 5c | This manual, behind a password gate | 11 NOV 2026 |
| 6 | Grammars (`cnf`, `parse` with CYK and Earley, `derive`, `to_pda`, `to_cfg`), Moore λ syntax and Mealy ↔ Moore, storage views in the Pad, four new tool pages | 11 NOV 2026 |

## Next

- **Left from Phase 6:** `to_single_tape`, `gnf`, bounded `is_ambiguous`, a pumping-lemma tool.
- **Phase 7, polish:** notation profiles (other textbooks), JFLAP import/export, accessibility
  audit, performance; one renderer for screen and export (`delta-render` everywhere, if its
  layout holds up); fonts for exports that match the Pad; TikZ checked with tectonic in CI.
- **Phase 8, mobile:** Capacitor apps for Android and iOS.

## Later

HTTP API on Cloudflare Workers, an MCP server so AI assistants can run and check Delta, accounts
and saved projects with short links, `choose` (nondeterministic programs), reductions, ω-automata.

## Questions they'll ask

1. **"What's the next big technical challenge?"** — Grammars: CNF conversion with readable
   traces, CYK tables in the Pad, and PDA↔CFG constructions that produce grammars a student can
   read.
2. **"Why mobile apps for this?"** — Students study on phones; offline file handling and the
   share sheet make it a study tool, and Capacitor reuses the web code.
3. **"What would an MCP server do?"** — Let an assistant write a Delta program, run it, read the
   JSON (including counterexamples) and fix it — a tight loop for teaching and grading.
4. **"What won't you build?"** — A proof assistant, or anything that hides undecidability.
5. **"When is v2 'done'?"** — When Phase 6 lands and the beta has real users; then `v2` merges
   into `main` (owner's call).

## Self-test

<details class="selftest"><summary>1. What did Phase 6 add?</summary>cnf, parse (CYK and Earley), derive, to_pda, to_cfg, Moore λ and Mealy ↔ Moore, storage views in the Pad, four tool pages.</details>
<details class="selftest"><summary>2. Which phase unifies the renderer?</summary>Phase 7 (polish).</details>
<details class="selftest"><summary>3. How will mobile apps be built?</summary>Capacitor around the web Pad.</details>
<details class="selftest"><summary>4. What's the MCP server for?</summary>AI assistants writing, running and fixing Delta programs.</details>
<details class="selftest"><summary>5. When were Phases 0–4 committed?</summary>08 NOV 2026.</details>
