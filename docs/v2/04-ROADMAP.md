# 04 — Roadmap, Migration, Testing, Risks

## 1. Migration from v1

**Phase 0 starts with a read-only audit by Claude Code** (`docs/v2/AUDIT.md`), confirming these decisions against the real code.

| v1 part | v2 |
|---|---|
| Rust algorithms (subset, regex → NFA, state elimination, simulations, closure ops) and their tests | **Keep.** Adapt to the generic `Machine` + `Model`; keep them as fast paths |
| Golden and property tests | **Keep all.** Re-express the inputs in v2 syntax; expected results unchanged |
| `delta-dsl` (v1 parser/checker) | **Replace** with `delta-lang`; reuse its error-recovery approach and diagnostics plumbing |
| Theorem/Definition strings | **Move** to `delta-explain/profiles/sipser3e.toml` |
| Error codes | **Renumber** to the v2 scheme |
| WASM worker, Comlink, budgets, Stop/respawn, GitHub Actions, Pages deploy, Playwright | **Keep** |
| React Pad (editor, diagram renderer, panels, themes, symbol bar) | **Keep and restructure** into the v2 layout |
| v1 `.delta` examples | **Convert** with `delta migrate` (v1 → v2 syntax) |

**Repository:** tag `main` as `v1.0-final`, and work on a long-lived `v2` branch. v1 stays live until the v2 beta; afterwards v1 can stay at `/v1`.

---

## 2. Phases and estimates

Hours are **Claude Code working time**, estimated from how fast Delta 1's Phases 0–1 went. Add about a third for your own reviewing and testing. On the Pro plan, usage limits stretch the calendar: expect **8–12 weeks of evenings**, and less on a higher plan.

| # | Phase | Delivers | Hours |
|---|---|---|---|
| 0 | **Audit and spec freeze** | `AUDIT.md`; `SEMANTICS.md` (formal meaning of every model property, acceptance and core construct); spec examples as failing tests; open questions resolved | 2–4 |
| 1 | **Language core** | Lexer, newline-aware parser with recovery, resolver, type inference, interpreter; values, set-builder, functions, `let`/`var`, control flow, `print`/`assert`; budgets; `delta run --json` | 12–20 |
| 2 | **Models and machines** | `Model` descriptors, generic `Machine` and `Label` ops, generic simulator, **prelude** (dfa, nfa, gnfa, twdfa…), all literal forms, model rule checking (E3xx); v1 finite-automata algorithms ported as fast paths; explain profiles; v1 golden tests passing | 10–16 |
| 3 | **Languages, classes, constructions** | `lang` values, class lattice, closure and decision tables, compile-time E7xx, bounded checks, `as … by`; `explore` and `fixpoint` with traces | 6–12 |
| 4 | **The Pad** | DartPad layout, all panels including Lang, Run/Auto/Stop, editor intelligence (from signature records, LaTeX completion), share links, embeds, PWA (installable, offline), phone and tablet layouts | 8–14 |
| 5 | **Front page and docs** | Astro + Starlight site, live hero, feature sections, tour, generated library reference, ≥ 30 cookbook recipes, error pages, generated AI guides, `/llms.txt`, schema; **SEO**: per-page titles, descriptions and canonicals, sitemap, robots.txt, JSON-LD, Open Graph cards with diagram previews, `/tools/` landing pages (one per capability, each with a live Pad), `/learn/` concept pages, `/practice/` solved problems, Cloudflare Web Analytics, Lighthouse SEO and performance ≥ 95 → **v2.0-beta goes live** at `delta.nandish.dev` | 6–10 |
| 5b | **Terminal and packages** | `delta run [--show] [--format svg\|png\|pdf\|tikz] [--out DIR]`, `delta repl`, the `save` statement, inline images in the terminal, native rendering (pure-Rust layout, resvg), cargo-dist releases (macOS/Linux/Windows, Homebrew tap, shell/PowerShell installers, MSI), `cargo install delta-cli` | 6–10 |
| 5c | **Private manual** | Interview-prep manual in the private repo `delta-manual` (pitches, life of one program, theory deep dives, language design, architecture trade-offs, war stories; questions and a self-test per chapter), a separate Pages project behind Cloudflare Access at `manual.delta.nandish.dev`, noindex; from here on every phase's definition of done includes updating its manual chapters | 4–8 |
| 6 | **Beyond finite automata** | pda/dpda, tm/ntm/lba/mtm, grammars (cnf, parse, derive, to_pda, to_cfg), mealy/moore/fst, counters and queues; storage views in the Pad → **v2.0** | 12–20 |
| 7 | **Polish** | notation profiles (hmu, linz), JFLAP import/export, TikZ export, accessibility audit, performance pass, presentation mode | 4–8 |
| 8 | **Mobile apps** | Capacitor projects (Android, iOS), file handling, share sheet, haptics, deep links, icons and splash screens, signed CI builds, TestFlight and Play internal testing, store listings and submission | 6–12 (+ store review waiting) |
| | **Total** | | **≈ 66–116 h** |

Each phase ends with a demo you can try on your phone, and its exit criteria (below) pass in CI.

### Exit criteria (abridged)
- **P1:** `divisible_by(n)`, set-builder expressions and a hand-written worklist algorithm run in the CLI with the expected JSON.
- **P2:** All v1 golden tests pass in v2 syntax. The generic simulator agrees with every fast path on 10,000 random machines. The "no book words in core" test passes.
- **P3:** `check C ≡ P` (CF vs Decidable) fails at compile time with E701 and a bounded hint. A `determinize` written with `explore` equals the built-in on 1,000 random NFAs and is steppable.
- **P4:** Playwright on 6 viewports. The PWA installs on Android and iOS and runs offline.
- **P5:** Lighthouse ≥ 95 for the front page and docs. Every docs code block runs in CI.
- **P6:** The cookbook covers every prelude model. CYK and Earley agree. PDA → CFG → PDA round trips pass bounded equivalence.
- **P8:** The apps pass TestFlight and Play internal testing, and are submitted.

---

## 3. Testing strategy

| Layer | Tests |
|---|---|
| Language | Conformance suite: every example in `01-LANGUAGE.md` with expected JSON. Parser fuzzing: no panics. Formatter idempotence (fmt(fmt(x)) = fmt(x)). Unicode/ASCII round trip |
| Semantics | Each model property's definition in `SEMANTICS.md` has executable tests. The generic simulator is differentially tested against the fast paths |
| Algorithms | v1 golden tests. Property tests (regex → NFA → DFA → min → regex round trip; Hopcroft ≅ Moore ≅ Brzozowski). Brute-force oracles on all words up to length 8. Optional cross-checks against FAdo results |
| Classes | Every closure-table and decision-table entry has a test. Every E7xx error has a test with its bounded alternative |
| User constructions | `explore`-based determinize and product constructions equal the built-ins on random inputs |
| Docs | Every code block runs. Library examples come from signature records. AI guides are generated, never hand-edited |
| UI | Vitest; Playwright on 6 viewports (run, stop, share round trip, embed, offline PWA); axe accessibility; visual snapshots of diagrams |
| Apps | Capacitor smoke tests on an Android emulator and iOS simulator in CI; manual checklist before each store release |
| Performance | criterion benchmarks with regression limits; WASM and bundle size limits |

---

## 4. Risks

| Risk | Mitigation |
|---|---|
| The language grows beyond control | Spec freeze in Phase 0; §16 deferred list; new syntax requires a spec change first |
| The generic model system is slower than v1's structs | Fast paths for common models; differential tests keep them honest |
| The class system confuses users | Clear E7xx messages that always offer the bounded alternative; a Lang panel showing the lattice; a docs chapter |
| User code hangs the Pad | Budgets everywhere; Stop restarts the worker; Auto-run off by default |
| App Store rejection ("just a website") | Offline mode, file handling, share sheet and phone UI built before submission; TestFlight first |
| Usage limits slow development | Phase order delivers value early (beta after Phase 5); each phase is shippable |
| Docs drift | Generated or tested in CI |
| Burnout on a niche project | Ship the beta early, get real users (your course, classmates, professors), then decide how far to take Phases 6–8 |

---

## 5. Later: API and MCP

The engine already turns `source → JSON`, so these are thin layers:

| Piece | How |
|---|---|
| HTTP API (`api.delta.nandish.dev`) | The same WASM engine on Cloudflare Workers: `POST /run`, `/check`, `/render` |
| MCP server | Tools `delta_run`, `delta_check`, `delta_render`, `delta_docs`, so AI assistants can write, run and fix Delta in a loop |
| Accounts and saved projects | A small backend (e.g. Cloudflare D1 or Supabase), short links, classroom assignments |
| Editors | VS Code extension through an LSP built on `delta-lang`; a Jupyter kernel or Python package |

Nothing in Phases 0–8 blocks these.
