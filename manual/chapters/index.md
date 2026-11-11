---
title: Delta manual
description: The private manual for Delta, a programming language for the theory of computation.
---

**Delta is a programming language for the theory of computation: you write machines, languages
and constructions as code, and it runs them, draws them and checks claims about them.** It is for
students and teachers of automata theory, and for the person who built it, explaining it.

Status: **in use** (v2 beta live) · Owner: Nandish Sinha · Describes commit `{{commit}}` ·
Updated {{date}}

This manual is private. It lives at `delta-manual.pages.dev` behind a password, is never
indexed and is never linked from the public site. Its source is `manual/chapters/` in the Delta
repository; code excerpts are pulled from the source at build time, so they can't drift.

## How to read it

- **Five minutes before a call:** [Pitches](/pitches/).
- **The one walk-through to know cold:** [Life of one program](/life-of-a-program/).
- **Theory questions:** the [deep dives](/theory/01-subset/), one per algorithm.
- **"Why did you…" questions:** [Language design](/language-design/),
  [Architecture](/architecture/), [War stories](/war-stories/).
- Every chapter ends with **Questions they'll ask** (with model answers) and a **self-test**.
  Answer the self-test out loud before opening the answers.

## Project brief

| Field | Answer |
|---|---|
| Project name | Delta (δ), files `.delta` |
| One sentence | A language and playground where finite automata, pushdown automata, Turing machines, regexes, grammars and languages are values you can build, run, draw and compare |
| Owner | Nandish Sinha |
| Stage | In use: v2 beta at `delta.nandish.dev`; v1 tagged `v1.0-final` |
| Kind | Language + web app (the Pad) + CLI (`deltac`) + static docs site |
| Repo | `github.com/NandishSinha1403/delta`, long-lived branch `v2` (v1 on `main`); this manual describes `{{commit}}` |
| Readers | 1. the owner preparing for interviews, 2. a contributor, 3. an operator, 4. a user, 5. a visitor |
| Public or private | Private (one password, owner only). Operating details and decision history live here, not on the public site |
| Where it lives | `delta-manual.pages.dev` (Cloudflare Pages project `delta-manual`) behind a password gate (`manual/functions/_middleware.js`); never a custom domain |
| Design system | `DESIGN.md` ("Dark Tech": black ground, Schibsted Grotesk + Commit Mono). The manual reuses the site's fonts and theme |
| Sensitive areas | Cloudflare account and tokens; the copyrighted textbook PDF in the repo root (never committed); the manual's password (Pages secret `MANUAL_PASS`) |
| Facts that change often | Cloudflare plan limits, tool versions (wrangler pinned at 4.42.0), test counts and WASM sizes (each quoted with its date) |

**What I couldn't find in the repo:** a written threat model (chapter 10 is the first), a
backup policy for the Cloudflare projects (chapter 7 says what exists), and usage numbers (Web
Analytics isn't on yet).
