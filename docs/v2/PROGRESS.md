# Delta 2 — Progress

_Last updated: 2026-11-09 (SEO pages done; 5b and 5c waiting on the owner)_

## Current state

| | |
|---|---|
| Branch | `v2` (long-lived; **never merge into `main`** until the v2 beta) |
| v1 | Tag `v1.0-final` = `50bf153` (= `origin/main` at the time, incl. both CNAME commits). v1 stays live from `main` |
| Phase | **1–5 built** (language, engine, Pad, front page and docs). Next: **5b** (terminal and packages) |
| Hosting | Cloudflare Pages, project `delta` (`delta-17m.pages.dev`), direct upload from a Mac with wrangler 4.42.0 until GitHub Actions is unlocked; then `deploy.yml` deploys on push to `main`. `delta.nandish.dev` moves from GitHub Pages (v1) to it |
| Conformance | 100 cases, `xfail.txt` empty: all pass |
| Tests | 121 Rust (incl. conformance); 26 Vitest; 84 Playwright (phone, tablet, desktop) |

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

1. Read `CLAUDE.md`, then this file.
2. **Domain:** `delta.nandish.dev` is added to the Pages project `delta` (status pending). The owner
   changes the name.com CNAME `delta` → `delta-17m.pages.dev`; then watch the domain via the API
   (`/accounts/<id>/pages/projects/delta/domains/delta.nandish.dev`) until status and certificate
   are `active`, and confirm https://delta.nandish.dev serves v2. Rollback: CNAME back to
   `nandishsinha1403.github.io`.
3. **SEO (Phase 5) — done, committed, not yet deployed.** `/tools/` (8 pages; pumping-lemma,
   cfg-to-cnf, cyk-parser, cfg-to-pda, mealy-moore wait for Phase 6), `/learn/` (9), `/practice/`
   (8), JSON-LD/social cards (`site/src/route-data.ts`), robots.txt, `site/scripts/check-seo.mjs`
   (runs in `build-site.sh`). Lighthouse on the live site: 100 perf / 100 SEO (front page; local
   `astro preview` shows ~73 only because it doesn't compress). OG images are interim
   (`site/public/og/default.png` from the hero); diagram cards come from the 5b renderer.
   Still to do: deploy (manual deploy below); the owner sends the Cloudflare Web Analytics site
   token → set `BEACON_TOKEN` in `site/src/lib/analytics.mjs`; give the owner Google Search
   Console + Bing TXT-record steps for name.com.
4. **Phase 5b (terminal and packages)** — approved deps: rustyline 19, resvg 0.45 + svg2pdf 0.13
   (same usvg 0.45), icy_sixel 0.7, the cargo-dist tool; own layered layout, own iTerm2/kitty
   protocols, hand-parsed args (no clap/viuer/layout-rs); embed Commit Mono (OFL). The binary is
   renamed **`deltac`** (git-delta owns `delta`). Renderer = the planned `crates/delta-render`
   (layout, SVG, TikZ, ASCII; no deps; also in WASM). Spec first: `save <value> to "<file>"` in
   01-LANGUAGE.md §Output, SEMANTICS §10.4, conformance cases; output kind `save`, the engine
   never writes files (CLI writes inside the cwd only; the Pad downloads). **Open question for the
   owner** (they wanted to clarify before answering): how the Pad makes PNG/PDF — lazy second
   "export" WASM with resvg+svg2pdf (recommended) / browser canvas + print / PDF CLI-only.
   Releases on tags need GitHub Actions (locked); build and test locally.
5. **Phase 5c (private manual)** — plan changed: source in this repo under `manual/` (own Starlight
   site, never in `site/dist`), excerpts pulled by path + marker at build time (CI fails on a
   missing marker), separate Pages project protected by Cloudflare Access on `<project>.pages.dev`
   and `*.<project>.pages.dev` (owner's email, one-time PIN, 30-day session), never a custom
   domain, noindex + robots disallow, no links from the public site. No nameserver move needed.
   **Waiting for the owner's MANUAL.md template** (never received). Test the Access login in a
   private window, then give the owner the URL.
6. PR #6 (`v2` → `main`) carries the Cloudflare `deploy.yml`; merge only when the owner says so,
   after Actions is unlocked and `CLOUDFLARE_API_TOKEN`/`CLOUDFLARE_ACCOUNT_ID` are set. The owner
   still needs the step-by-step for a Pages-only API token.
7. Known: the TM loop detector stores every configuration with its tape (quadratic); the Pad's
   language preview is capped at 1,000 steps per word (`pad.rs`, `PREVIEW_STEPS`).

Manual deploy (from the repo root, after all checks): `./scripts/build-site.sh`, then
`npx wrangler@4.42.0 pages deploy site/dist --project-name=delta --branch=main`. Newer wrangler
(4.158) creates `~/.wrangler` and then ignores the OAuth login; move that folder away if it appears.
