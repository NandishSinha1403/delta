---
title: "11. Limits and costs"
description: Known issues, budgets, platform limits and what Delta costs to run.
---

Readers: **owner**, **operator**.

## Running cost: ₹0

- Cloudflare Pages (Free): static hosting, no build minutes used (direct upload).
- Pages Functions (Free): the manual's password gate, one small request per page view.
- Domain `nandish.dev`: already owned; the subdomain costs nothing extra.
- GitHub: free repository; Actions is currently off.

How it stays at ₹0: no server, no database, no paid plan enabled. Zero Trust / Access was
considered for the manual and dropped (10 NOV 2026) in favour of a password gate.

## Platform limits

| Limit | Value | Source, verified |
|---|---|---|
| Pages files per site (Free) | 20,000 | [Pages limits](https://developers.cloudflare.com/pages/platform/limits/), verified 10 NOV 2026 |
| Pages file size | 25 MiB | same page, verified 10 NOV 2026 |
| Workers/Functions requests (Free) | 100,000 per day | [Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/), not re-verified |

The site has 143 pages; the largest file is the export WASM (2.2 MB raw).

## Engine limits (defaults, `crates/delta-lang/src/interp.rs` `Budget`)

| Budget | Default | When hit |
|---|---|---|
| steps | 10,000,000 | B6xx outcome; a machine run gets verdict `unknown` |
| states created | 100,000 | B6xx (constructions, decisions) |
| recursion depth | 400 | B6xx |
| outputs | 10,000 | B6xx |
| Pad language preview | 1,000 steps per sample word | sample omitted |
| Browser watchdog | 10 s per call | worker restarted |
| Engine WASM | 298 KB gzipped (budget 700) | size check fails |
| Export WASM | 887 KB gzipped (budget 1,000) | size check fails |
| PNG export | 8,192 px per side | image scaled down to fit |

## Known issues

- The TM loop detector stores every configuration with its tape: memory grows quadratically on
  long runs.
- TikZ output hasn't been compiled with LaTeX yet (no LaTeX on the build Mac).
- DCF equivalence is not built yet (E702).
- `is_ambiguous(G, upto: n)` only searches words up to length n; it can never say a grammar is
  unambiguous, and its record's `note` says so.
- `gnf` can grow a grammar a lot (a three-variable grammar became 853 rules); `to_single_tape` builds
  one state per (old state, collected head symbols) and per edge being applied. Both are capped by
  the states budget.
- `range(…)` is listed as a built-in but not implemented (E509); use `0..n` or `0..=n`.
- `to_cfg` grows with the cube of the normalized state count; large PDAs hit the states budget.
- Hopcroft here rebuilds the partition per splitter, so it's slower than the textbook
  O(k·n log n) on large DFAs.

## Questions they'll ask

1. **"How does it scale?"** — The engine runs on the user's device, so hosting cost doesn't grow
   with users. The budgets bound per-program cost.
2. **"What's the largest machine you can determinize?"** — Up to 100,000 subset states by
   default; the user can raise it with `budget { states: … }`. Diagrams above ~300 states fall
   back to tables.
3. **"Why not a server for heavy work?"** — Cost, privacy and offline use. An HTTP API on Workers
   is planned for tools and AI, not for the Pad.
4. **"What are you worst at?"** — Long Turing-machine runs (configuration storage) and big
   Hopcroft inputs; both are known and listed.
5. **"What does it cost per user?"** — Nothing measurable: static files from a CDN.

## Self-test

<details class="selftest"><summary>1. What does Delta cost to run per month?</summary>₹0 (Cloudflare Free plans, existing domain).</details>
<details class="selftest"><summary>2. Default state budget?</summary>100,000.</details>
<details class="selftest"><summary>3. Pages Free file limit?</summary>20,000 files per site; 25 MiB per file.</details>
<details class="selftest"><summary>4. Name two known issues.</summary>TM loop detector memory; TikZ untested; CNF/CYK/PDA↔CFG missing.</details>
<details class="selftest"><summary>5. What verdict does a run get when the step budget runs out?</summary><code>unknown</code>.</details>
