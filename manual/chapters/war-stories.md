---
title: "War stories"
description: "Things that went wrong, how they were found, and what changed — told in situation, problem, action, result form."
---

Readers: **owner** (behavioral interviews). Tell each in under two minutes: situation, problem,
what you did, result, lesson.

## The spec's own example was false (Q-76)

- **Situation:** Phase 0 wrote the formal semantics and turned every example in the language spec
  into a conformance test.
- **Problem:** §6.4's example claimed that the language of words with as many 0s as 1s is
  regular, "by" the regex `(01)*`. That language isn't regular, and the regex doesn't even
  describe it: `"10"` is in the language but not in `(01)*`.
- **Action:** fixed §6.4 to a true claim (`count(w,'0') % 2 == 0` as Regular by
  `re"(1* 0 1* 0)* 1*"`) and kept the false one as conformance case 19, which must fail with an
  error and a witness.
- **Lesson:** executable specs catch errors in the spec itself. The false example became a test
  of the error path.

## "Ported" algorithms that didn't exist (Q-50, Q-51)

- **Situation:** the v2 plan said Hopcroft minimization and the decision procedures would be
  "ported from v1".
- **Problem:** the Phase 0 audit read every v1 module: v1 had **no minimization at all** and no
  equivalence, inclusion, emptiness, finiteness or universality — yet the very first example in
  the spec needs equivalence.
- **Action:** recorded it as Q-50/Q-51, re-estimated Phases 2–3 (+8–15 h), and built all four
  minimization algorithms and the decision procedures as new work, cross-tested against each
  other and brute force.
- **Lesson:** audit before you plan on reuse; a plan's "port" can hide a rewrite.

## GitHub locked Actions → Cloudflare

- **Situation:** v1 deployed through GitHub Actions to GitHub Pages.
- **Problem:** Actions became unavailable on the account (locked), so nothing could build or
  deploy through CI.
- **Action:** moved hosting to Cloudflare Pages with direct uploads from the Mac (pinned
  wrangler 4.42.0), kept `deploy.yml` ready on PR #6 for when Actions returns, moved the domain
  with one CNAME and a documented rollback.
- **Result:** v2 went live at `delta.nandish.dev` with Lighthouse 100 performance and SEO on the
  front page.
- **Bonus bug:** newer wrangler created `~/.wrangler` and then ignored the OAuth login — fixed by
  pinning and a runbook.

## The 300-state diagram cap

- **Situation:** users can determinize any NFA, and results can have thousands of states.
- **Problem:** Graphviz takes seconds on thousands of nodes, and nobody can read such a picture.
- **Action:** `MAX_DRAWN_STATES = 300` in `DiagramPanel.tsx`: above it, the panel says how many
  states there are and points to the δ-table and tuple, which still show everything.
- **Lesson:** the right answer to "can't draw it fast" was "don't draw it" — with a clear
  message, not a spinner.

## The runaway preview

- **Situation:** the Pad's Lang view previews sample words of each language.
- **Problem:** the preview ran each sample word with a 200,000-step budget, ignoring the
  program's own budget. A Turing machine that moves right forever stores every configuration
  with its tape, so the preview grew quadratically and the worker was killed.
- **Action:** each sample word now gets min(1,000, the program's budget) steps; a regression test
  runs a runaway TM under the default budget.
- **Lesson:** every loop over user input needs the user's budget, including "helpful" UI ones.

## Telemetry in a font package

- **Problem:** CI installs failed because pnpm refused an unlisted install script — from
  `@ibm/plex-mono`, whose postinstall only sends IBM telemetry.
- **Action:** denied that script explicitly in `pnpm-workspace.yaml`; fonts need no build step.
- **Lesson:** a dependency's install script is code you run; deny by default.

## `deltac` and the git-delta clash

- **Problem:** the planned binary name `delta` is already git-delta's, common on developer
  machines.
- **Action:** renamed to `deltac` in Phase 5b, with every doc and the conformance harness updated
  in the same commit.

## Questions they'll ask

1. **"Tell me about a time you found a bug in your own spec."** — Q-76: the false regularity
   claim, found by turning spec examples into tests.
2. **"Tell me about an estimate that was wrong."** — Q-50/51: "ported" algorithms that had to be
   written; the audit caught it before the phase started.
3. **"Tell me about an outage or an infrastructure change."** — Actions locked → Cloudflare, with
   a CNAME rollback ready.
4. **"A performance problem?"** — The runaway preview (budget not propagated) or the 300-state
   cap.
5. **"What did you learn about dependencies?"** — The telemetry install script; and why `deltac`
   uses its own argument parsing and image protocols.

## Self-test

<details class="selftest"><summary>1. What was Q-76 and what became of the false example?</summary>The spec's §6.4 regularity claim was false; it became conformance case 19, which must fail with an error.</details>
<details class="selftest"><summary>2. What did the audit find about minimization?</summary>v1 had none; all four algorithms were new work.</details>
<details class="selftest"><summary>3. Why did hosting move to Cloudflare?</summary>GitHub Actions was locked on the account.</details>
<details class="selftest"><summary>4. What's the diagram cap and where?</summary>300 states, <code>MAX_DRAWN_STATES</code> in DiagramPanel.tsx.</details>
<details class="selftest"><summary>5. What fixed the runaway preview?</summary>Each sample word's budget is min(1,000, program budget), plus a regression test.</details>
