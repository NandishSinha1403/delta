# Delta 2 — Progress

_Last updated: 2026-11-11 (main = v2 via PR #6; IBM Plex Mono code font; the five Phase 6 leftovers and the pumping-lemma tool)_

## Current state

| | |
|---|---|
| Branch | `v2`; **merged into `main` on 2026-11-11** (PR #6, owner's call), `main` = `v2` = `ed1fb45` then. Work continues on `v2`; nothing goes to `main` without the owner |
| v1 | Tag `v1.0-final` = `50bf153`. GitHub Pages (v1) has no workflow any more; the domain points at Cloudflare |
| Phase | **1–6 done**. **7** (polish) in progress: themes, renderer switch and STIX state names done |
| Hosting | Cloudflare Pages: `delta` (`delta.nandish.dev`) and `delta-manual`. `deploy.yml` deploys both on push to `main` with wrangler-action, but **skips (grey) until `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` are set**; until then deploy from the Mac (runbook at the end) |
| Conformance | 71 cases, `xfail.txt` empty: all pass |
| Tests | 166 Rust (incl. conformance); 23 Vitest; 99 Playwright (phone, tablet, desktop; incl. the glyph test) + 10 site |

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
  Since Phase 7 the Pad's on-screen diagram uses it too (Graphviz removed); so do exports,
  `deltac` and social cards.
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

**Made 2026-11-11 (Phase 6):**
- Owner: Moore outputs are `λ(q) = w` / `lambda(q) = w` (Q-94).
- (C) Grammar built-ins: `nullable`, `useless`, `trim(G)`, `cnf` (START, DEL, UNIT, TERM, BIN,
  trim; new variables S0, U_a, X1…), `parse(G, w, method: "earley" | "cyk")` → a parse-tree value
  (`root`, `children`, `yield`, `text`), `derive` (leftmost, from the Earley tree), `to_pda`
  (:start, :loop, :accept), `to_cfg` (⟨p,q⟩ variables after normalizing), `mealy_to_moore`,
  `moore_to_mealy`. SEMANTICS §11. CYK uses a grammar that is already in CNF as it is.
- (C) Q-95: PDA membership is decided through `to_cfg` + Earley (cached per machine), because
  breadth-first simulation never ends when ε-moves grow the stack.
- (C) Traced runs keep structured configurations; the Pad's Run view shows the stack (top
  first), tapes with the head, counters, the queue and the output, for up to 3 branches, and a
  run has as many steps as breadth-first layers (not input symbols). Parse trees and constructed
  grammars are Pad objects with their steps.
- (C) Tool pages: cfg-to-cnf, cyk-parser, cfg-to-pda, mealy-moore (12 tools). pumping-lemma waits.
- Not done from 01 §12: `gnf`, `is_ambiguous`, `to_single_tape`, `myhill_nerode`, `pump` (E509).

**Open:** the Web Analytics token (owner is enabling it in Pages → Metrics).

**Made 2026-11-11 (after Phase 6), by the owner unless marked (C):**
- PR #6 merged (`main` = `v2`). CI runs on every push and PR; it installs binaryen 133 (wasm-opt),
  without which the export WASM was 1031 KB on CI (C). `deploy.yml`: a `secrets` job gates the
  deploy job, so missing secrets mean "skipped", not a failure (C).
- Q-32 closed: **IBM Plex Mono** is the one code font everywhere code appears (Pad, site, docs,
  manual, exports, `deltac` images, social cards), including the site's chips, tables and
  captions. Fallbacks for what Plex lacks: Noto Sans Math subset, then a Noto Sans Mono subset for
  ⁺ ⁿ ᵏ ᵐ (owner chose this over dropping the superscripts). `scripts/subset-fonts.py` (C) cuts
  every file from pinned, checksummed sources; `crates/delta-export/fonts/symbols.txt` is checked
  in exports (Rust) and in the Pad (Playwright + `CSS.getPlatformFontsForNode`) (C). Export WASM
  942 KB gzipped. ✗ (U+2717) is in none of the fonts; it's UI/terminal text only (C).
- The five functions, shapes chosen by the owner: `pump` → `{p, x, y, z, state}` with p = the
  given DFA's size (minimal DFA for other regular inputs), E504 if w ∉ L or |w| < p;
  `myhill_nerode` → `{index, classes, separators}`; `is_ambiguous` → labeled record
  `{ambiguous, upto, word, trees, note}`; `to_single_tape` → the multi-track construction.
  `gnf` (C): via cnf, ordered substitution, Z1, Z2, … for left recursion. SEMANTICS §11.8–11.12.
- (C) Tree counting for `is_ambiguous` is done in G itself (normal forms change ambiguity),
  saturating at 2, with rounds so cycles count as ambiguous and trees stay finite.
- (C) E509's text is now "not available for these arguments" (every built-in exists; some still
  report E509 for a wrong argument kind — a Phase 7 item).
- (C) Pumping-lemma tool page (13 tools), manual chapter "theory/10" and limits/next/parts.

**Made 2026-11-11 (Phase 7, owner-approved plan; (C) = Claude's call):**
- `main` is kept in sync with `v2` after each green step (owner).
- Themes (owner): the user picks the theme of their output. Pad: Settings → Downloads = Light /
  Dark / Match Pad (default Match Pad). Language: `save M to "m.png" (theme: dark)` adds `theme`
  to the save output (SEMANTICS §10.4); without it the host decides. `deltac --theme light|dark`
  (`--dark` kept as a hidden alias). The Pad's light theme already existed; palette differences
  vs the plan were reported, no colors changed.
- Item 8 (owner): `on t:` lasts until the next `on`. Also (C): parameterized models start
  literals (`mtm(2) M { … }`), the k-tape tuple short form for any on-tape model with k ≥ 2,
  `star(regex)` works, wrong argument counts to union/intersect/difference/concat/product/star
  are E507. `range` stays E509 (not implemented; `0..n` exists).
- Renderer (owner): switch the Pad to `delta-render`, keeping finger, highlight, zoom, legend and
  colors; drop Graphviz only after all diagram tests pass. State names in STIX Two Text italic
  everywhere delta-render draws; edge labels and code stay IBM Plex Mono.
- Done 2026-11-11: the Pad lays out with `delta-render` (WASM `diagram_layout`); Graphviz and its
  DOT/JSON code are gone (Q-34 confirmed). Pad `dist` 6.6 MB → 5.8 MB: the lazy 818 KB (635 KB
  gzipped) Graphviz chunk is removed; the startup bundle is unchanged (312 KB gzipped). Export
  module 967 KB gzipped with the STIX subset (budget 1000). Social cards regenerated (11).

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

**Phase 7 in progress (2026-11-11).** Done and deployed: themes, `delta-render` in the Pad
(Graphviz removed), STIX Two Text italic state names (DESIGN.md, manual credits, social cards).
**Next:** the presentation-mode mockup is with the owner; wait for the OK before building it.
Then: notation profiles, JFLAP (roxmltree approved), performance, accessibility
(@axe-core/playwright approved), TikZ in CI, exit check. Light-theme palette differences were
reported to the owner; no colors changed yet.

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
5. **Phase 5c — done and deployed 2026-11-11** at `https://delta-manual.pages.dev/`. `manual/`
   (Starlight; `chapters/` → `scripts/prepare.mjs` fills excerpts from `// manual:begin NAME`
   markers, `deltac --help` and the error list; fails on a missing marker or a secret-shaped
   string; `check-dist.mjs` checks noindex and no outside requests). 29 pages: the 13 template
   chapters, 0b Pitches, Life of one program, 9 theory deep dives, Language design,
   Architecture, War stories, Appendices; each ends with questions and a self-test.
   **Privacy (owner, 2026-11-11): Zero Trust/Access dropped; one password instead.**
   `manual/functions/_middleware.js` gates every request (production and `<hash>.` deployment
   URLs): password page (401) → POST the right password → 303 + `delta_manual` cookie (30 days,
   Secure, HttpOnly, SameSite=Lax) holding HMAC-SHA256(MANUAL_PASS, "delta-manual/v1"); both
   checks constant-time on SHA-256 digests; fails closed (500) without the secret; `/fonts/`,
   `/favicon.svg`, `/robots.txt` public. Secret `MANUAL_PASS` is set on the project (value from
   the owner, never in the repo). `scripts/test-middleware.mjs` runs in the manual build. Deploy
   **from `manual/`** (`cd manual && npx wrangler@4.42.0 pages deploy dist
   --project-name=delta-manual --branch=main`) or the gate isn't uploaded. Known limit: no rate
   limit on password guesses. From now on every phase's done-definition includes updating its
   manual chapters.
6. PR #6 is merged (`main` = `v2` at `ed1fb45`). The owner adds `CLOUDFLARE_API_TOKEN` and
   `CLOUDFLARE_ACCOUNT_ID` later; until then `deploy.yml` skips and every step is deployed from
   the Mac. `main` is not updated after each step unless the owner asks.
7. **Phase 7 (polish) is next**: the plan was sent to the owner; wait for the OK before starting.
   Phase 6 leftovers are done.
8. **Follow-ups for later (owner, 2026-11-10; don't do yet):**
   - Unify on-screen diagrams (Graphviz) and exports (`delta-render`) on `delta-render` in Phase 7
     if its layout holds up, so what you see is what you download.
   - Verify TikZ output with tectonic (single binary) in CI.
9. Known: the TM loop detector stores every configuration with its tape (quadratic); the Pad's
   language preview is capped at 1,000 steps per word (`pad.rs`, `PREVIEW_STEPS`).

Manual deploy (from the repo root, after all checks): `./scripts/build-site.sh`, then
`npx wrangler@4.42.0 pages deploy site/dist --project-name=delta --branch=main`. Newer wrangler
(4.158) creates `~/.wrangler` and then ignores the OAuth login; move that folder away if it appears,
including between two deploys in a row (site, then manual).
