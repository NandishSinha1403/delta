# Delta 2 — Progress

_Last updated: 2026-11-10 (5b deployed; 5c manual written and built, waiting for Access before its first deploy)_

## Current state

| | |
|---|---|
| Branch | `v2` (long-lived; **never merge into `main`** until the v2 beta) |
| v1 | Tag `v1.0-final` = `50bf153` (= `origin/main` at the time, incl. both CNAME commits). v1 stays live from `main` |
| Phase | **1–5, 5b and 5c built** (5c: manual written, not deployed until Access is on). Next: **6** (beyond finite automata) |
| Hosting | Cloudflare Pages, project `delta` (`delta-17m.pages.dev`), direct upload from a Mac with wrangler 4.42.0 until GitHub Actions is unlocked; then `deploy.yml` deploys on push to `main`. `delta.nandish.dev` moves from GitHub Pages (v1) to it |
| Conformance | 54 cases, `xfail.txt` empty: all pass |
| Tests | 145 Rust (incl. conformance); 26 Vitest; 90 Playwright (phone, tablet, desktop) |

## What Phase 0 produced

| File | Contents |
|---|---|
| `docs/v2/` (01–04, README) | The Delta 2 plan, moved here from an untracked `delta-v2-plan/` folder. §6.4 fixed (Q-76) |
| `docs/v2/AUDIT.md` | Keep/change/drop for every v1 module; all book strings; coupling problems; test inventory; v1 → v2 error-code mapping |
| `docs/v2/SEMANTICS.md` | Formal meaning of configurations, label ops, every model property, acceptance modes, languages/classes, `explore`, `fixpoint`, the core and desugaring. Provisional choices tagged `[Q-nn]` |
| `docs/v2/QUESTIONS.md` | ~70 questions/contradictions, each with a recommended default and the phase it blocks. Top section: proposed answers |
| `conformance/` | 43 cases (`cases/NN-name.delta` + `.expect.json`), `xfail.txt`, README with the matching rules |
| `crates/delta-cli/tests/conformance.rs` | Harness: runs `deltac run --json` on each case, partial-matches the expectation, enforces the xfail list |
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

**Made 2026-11-08/09 (deploy, SEO, 5b, 5c), by the owner unless marked (C) = Claude's call:**
- Hosting: Cloudflare Pages project `delta`, direct upload (no Git integration), production branch
  `main`; `delta.pages.dev` was taken, so the URL is `delta-17m.pages.dev`. Deploy from the Mac
  until GitHub Actions is unlocked; `deploy.yml` (wrangler-action@v3, pinned wrangler 4.42.0 (C))
  replaces GitHub Pages on PR #6, which is not merged. Never touch `main` without the owner.
- Domain: `delta.nandish.dev` stays on name.com DNS; one CNAME to `delta-17m.pages.dev`.
- Pad preview budget (C): 1,000 steps per sample word, never above the program's budget.
- SEO: tool pages launch now only for what the engine supports (8 of 13); the rest with Phase 6
  (C). Pages are Starlight content (build-time HTML, tested by `tests/docs.rs`); one route
  middleware builds JSON-LD from each page's visible FAQ (C). Header nav: Docs, Tools, Learn,
  Cookbook, GitHub; Tour and Practice in the footer (C). Practice problems are original, in GATE
  style, never copied from past papers (C).
- OG images: rendered by Delta's own renderer (5b); interim card from the hero image until then.
- 5b dependencies approved: rustyline 19, resvg 0.45, svg2pdf 0.13, icy_sixel 0.7, cargo-dist
  (tool). Not added: viuer, layout-rs, clap (own code instead). Font: Commit Mono (OFL).
- 5b binary name: `deltac` (avoids git-delta's `delta`). The renderer is the planned
  `delta-render` crate (C). `save` is an output the engine emits; the CLI writes files only
  inside the cwd and the Pad downloads (C).
- 5c manual: in this repo under `manual/`, separate Starlight site and Pages project, Cloudflare
  Access on its pages.dev hosts only (email PIN, owner only, 30-day session), never a custom
  domain, noindex; no tokens, keys or account IDs in it. Every phase's done-definition then
  includes its manual chapters.

**Made 2026-11-09 (Phase 5b):**
- Owner: Pad PNG/PDF via a separate, lazily loaded export WASM (resvg + svg2pdf) shared with
  `deltac`, so both produce the same bytes. Built as `crates/delta-export-wasm`; 887 KB gzipped,
  budget 1000 KB in `check-wasm-size.sh` (C); loaded on the first PNG/PDF download only.
- (C) `save` formats: svg, png, pdf, tex (TikZ), txt (the `print` text), json; E514 unknown
  extension, E515 path leaves the folder, E516 image of a non-machine. `save` is a keyword, `to`
  contextual. The output carries `text` (the print form) for `.txt`.
- (C) `deltac run --json` never writes files; plain `run` writes `save` files inside the cwd
  (checked again after resolving symlinks). The Pad never downloads by itself (it re-runs on every
  keystroke): each `save` line in the Console gets a Download button.
- (C) Renderer `render/layered`: DFS cycle breaking, longest-path columns, label boxes as layout
  items in odd columns (labels never overlap), barycentre ordering, isotonic (PAV) placement,
  Catmull-Rom curves; long state names become ellipses; fixed-size self-loops with labels above.
  The Pad's on-screen diagram still uses Graphviz; exports, `deltac` and social cards use this.
- (C) Fonts: Commit Mono 1.143 and a Noto Sans Math 3.000 subset (⊔ ⊢ ▸ … that Commit Mono lacks),
  both OFL, subsets with licences in `crates/delta-export/fonts/` (77 KB together).
- (C) Terminal images: kitty (also Ghostty), iTerm2 (also WezTerm), sixel (foot, mlterm,
  contour), else text; `DELTAC_IMAGES` overrides. `--show`, `--format svg|png|pdf|tikz|txt`,
  `--out DIR`, `--dark`, `run -` reads stdin.
- (C) REPL: the session is a growing program, re-run per entry; only the entry's lines print; an
  entry with an error is dropped; a lone expression is printed; `:show :load :program :reset :quit`.
- (C) Social cards: `deltac gen-docs` renders a 1200×630 card per docs page whose first example
  shows a machine (11 pages now) into `site/public/og/<id>.png`; others keep `default.png`.
- (C) cargo-dist 0.34.0: `dist-workspace.toml`, installers shell/powershell/homebrew/msi, tap
  `NandishSinha1403/homebrew-tap`, 5 targets, generated `release.yml` (runs on `v*` tags once
  Actions is on). `[profile.dist]` uses opt-level 3. Workspace `authors`/`homepage` added for MSI.
- (C) `/sitemap.xml` 301 → `/sitemap-index.xml` (`site/public/_redirects`). v1 was one page at
  `/` with no routes or share links, so no other old URL needs a redirect.

**Open:** the Web Analytics token (owner is enabling it in Pages → Metrics).

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
cargo test -p delta-cli --test conformance -- --nocapture # prints "conformance: N of M cases pass"
cargo test -p delta-cli --test conformance -- --ignored  # strict: every case must pass (shows each failure)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The full check list (wasm build, size check, web lint/typecheck/test/build/e2e) is in `CLAUDE.md`.

## Next session: start here

1. Read `CLAUDE.md`, then this file.
2. **Domain — done 2026-11-09.** The CNAME `delta` → `delta-17m.pages.dev` is live; the Pages
   domain and its Let's Encrypt certificate are `active`, and https://delta.nandish.dev serves v2.
   Rollback: CNAME back to `nandishsinha1403.github.io`.
3. **SEO and 5b site — deployed 2026-11-09/10** and verified live: tools/learn/practice, the
   Terminal page, E514–E516, social cards, `/sitemap.xml` → 301. Still to do: the owner's Web
   Analytics token if Pages → Metrics needs one (`BEACON_TOKEN` in `site/src/lib/analytics.mjs`);
   Google Search Console + Bing TXT-record steps for name.com.
4. **Phase 5b — done.** Needs the owner: crates.io token for `cargo install delta-cli` (publish
   delta-core, delta-render, delta-lang, delta-dsl, delta-export first); a
   `NandishSinha1403/homebrew-tap` repo and `HOMEBREW_TAP_TOKEN`; Actions for `release.yml`.
5. **Phase 5c — written and built, not deployed.** `manual/` (Starlight; `chapters/` → prepare
   script fills excerpts from `// manual:begin NAME` markers in the source, `deltac --help` and the
   error list; fails on a missing marker or a secret-shaped string; `check-dist.mjs` checks
   noindex and no outside requests). Build: `./scripts/build-manual.sh`. 29 pages: the 13 template
   chapters, 0b Pitches, Life of one program (`check EndsAB ≡ L`, three paths), 9 theory deep
   dives, Language design, Architecture, War stories, Appendices; each chapter ends with
   "Questions they'll ask" and a 5-question self-test. Pages project `delta-manual` exists
   (`delta-manual.pages.dev`, **no deployment yet**). **Blocked on the owner:** the wrangler OAuth
   login has no Access scope (`/access/organizations` → authentication error), so Access can't be
   created through it. The owner must (a) enroll in Zero Trust Free (dashboard; the checkout asks
   for overage consent, one user of 50 is ₹0), choosing a team name, and (b) create an API token
   with Account → Access: Apps and Policies → Edit (and Access: Organizations, Identity Providers,
   and Groups → Edit), then paste it for this session only. Then: create the Access app for
   `delta-manual.pages.dev` + `*.delta-manual.pages.dev`, policy include = owner's email,
   one-time PIN, session 720h; verify an unauthenticated request redirects to
   `<team>.cloudflareaccess.com`; only then `npx wrangler@4.42.0 pages deploy manual/dist
   --project-name=delta-manual --branch=main`; test in a private window; give the owner the URL.
   From now on every phase's done-definition includes updating its manual chapters.
6. PR #6 (`v2` → `main`) carries the Cloudflare `deploy.yml`; merge only when the owner says so,
   after Actions is unlocked and `CLOUDFLARE_API_TOKEN`/`CLOUDFLARE_ACCOUNT_ID` are set. The owner
   still needs the step-by-step for a Pages-only API token.
7. **Follow-ups for later (owner, 2026-11-10; don't do yet):**
   - Fonts for exports. Why Commit Mono now: it was the font approved for 5b, and it's DESIGN.md's
     code font for the site. The plan (03-PRODUCTS §7, README) says Courier Prime for code, but
     Q-32 is still open because Courier Prime lacks most maths glyphs (∪ ∩ ∈ ⊆ ≡ δ Σ ⊔ …), and the
     Pad actually uses IBM Plex Mono + KaTeX's serif for state names. Both Courier Prime and Plex
     are OFL, so licence isn't the blocker; coverage is. Plan: try Courier Prime (or whatever the
     Pad settles on) + the Noto Sans Math fallback subset in `delta-export`, so exports match the
     Pad; settle Q-32 first.
   - Unify on-screen diagrams (Graphviz) and exports (`delta-render`) on `delta-render` in Phase 7
     if its layout holds up, so what you see is what you download.
   - Verify TikZ output with tectonic (single binary) in CI.
8. Known: the TM loop detector stores every configuration with its tape (quadratic); the Pad's
   language preview is capped at 1,000 steps per word (`pad.rs`, `PREVIEW_STEPS`).

Manual deploy (from the repo root, after all checks): `./scripts/build-site.sh`, then
`npx wrangler@4.42.0 pages deploy site/dist --project-name=delta --branch=main`. Newer wrangler
(4.158) creates `~/.wrangler` and then ignores the OAuth login; move that folder away if it appears.
