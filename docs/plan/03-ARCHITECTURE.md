# 03 — Architecture

## ADR-001: Technology stack (decided)

| Layer | Choice | Why |
|---|---|---|
| Engine (models, conversions, DSL) | **Rust**, compiled to **WebAssembly** | Rust-level speed for exponential constructions (subset construction, PDA search, TM runs); memory-safe; the same code also runs natively (CLI, tests, later desktop/Python) |
| Playground UI | **React 19 + TypeScript (strict) + Vite** | The owner already ships many React/TS projects, so the team's time goes into the product instead of learning a framework |
| Hosting | **GitHub Pages** + GitHub Actions, custom domain `delta.nandish.dev` | Fully static app, zero servers, free, same workflow as previous projects |
| Source of truth | **The code (DSL text)** | Diagram edits become text edits. One model, no sync bugs, works with undo, git and share links |

### Options considered and rejected
| Option | Why not |
|---|---|
| Pure TypeScript engine | Simplest to start, but 5–20× slower on heavy conversions and no typed IDs. Kept as a **fallback**: the engine sits behind one interface (`EngineApi`), so a TS implementation could replace it without UI changes |
| Python (Pyodide in browser) | ~10 MB download, slow start, slow loops |
| Flutter / Dart | Weaker web support for code editors (CodeMirror is far better), canvas-only text hurts accessibility and SEO, and the owner's experience is in React |
| Swift | Weak WebAssembly/web tooling |
| Monaco editor | Excellent on desktop but **poor on mobile/touch** → CodeMirror 6 instead |

---

## 1. Repository layout (one monorepo)

```
delta/
├─ crates/
│  ├─ delta-core/          # models + algorithms (pure Rust, no wasm deps)
│  │  └─ src/{ids.rs, bitset.rs, alphabet.rs, dfa.rs, nfa.rs, gnfa.rs, regex/, cfg/, pda.rs, tm/,
│  │          convert/{c01_dfa_to_nfa.rs, c02_subset.rs, c03_regex_to_nfa.rs, ...},
│  │          decide/, simulate/, trace.rs, budget.rs, export/{dot.rs, tikz.rs, jflap.rs, json.rs}}
│  ├─ delta-dsl/           # lexer, parser, checker (diagnostics), evaluator, formatter, edit→patch
│  ├─ delta-wasm/          # wasm-bindgen facade: thin JSON-in/JSON-out API
│  └─ delta-cli/           # native CLI: `delta check file.delta`, `delta convert`, golden-test runner
├─ web/                 # React + Vite app
│  ├─ public/           # icons, manifest.webmanifest, CNAME
│  └─ src/
│     ├─ engine/        # worker.ts, client.ts (Comlink), pkg/ (generated wasm + .d.ts)
│     ├─ editor/        # CodeMirror language (Lezer grammar), diagnostics, symbol bar
│     ├─ canvas/        # SVG diagram renderer, layout (Graphviz WASM), pan/zoom, animation
│     ├─ panels/        # Tuple, DeltaTable, Steps (trace scrubber), Run, Tests, Grammar, CYK, Tape
│     ├─ shell/         # responsive layout: desktop / tablet / phone shells
│     ├─ state/         # Zustand stores
│     └─ examples/      # gallery: the book's machines as .delta files
├─ examples/            # same .delta files, used by Rust golden tests
├─ docs/                # this plan
├─ scripts/build-wasm.sh
└─ .github/workflows/{ci.yml, deploy.yml}
```

---

## 2. Data flow

```
 ┌──────────── main thread (React) ─────────────┐        ┌──────── Web Worker ────────┐
 │                                               │        │                             │
 │  CodeMirror ──text (debounced 150ms)────────────────────► delta-wasm.evaluate(src)      │
 │                                               │        │   parse → check → evaluate  │
 │  Zustand store ◄──── Document ─────────────────────────── { diagnostics, objects,    │
 │     │                                         │        │     traces, results }       │
 │     ├─► Diagram (SVG)  ◄── layout ───────────────────── Graphviz WASM (lazy)          │
 │     ├─► Tuple / δ-table / Grammar panels      │        │                             │
 │     ├─► Steps scrubber (trace replay)         │        │                             │
 │     └─► Run panel (tape, stack, fingers)      │        │                             │
 │                                               │        │                             │
 │  Diagram edit (drag edge, rename) ──EditOp──────────────► delta-dsl.apply_edit(src, op)  │
 │  CodeMirror ◄──────── TextPatch ──────────────────────── returns minimal text patch   │
 └───────────────────────────────────────────────┘        └─────────────────────────────┘
```

**Code is the single source of truth.** The diagram never mutates a machine directly. It sends an `EditOp` (`AddEdge{from,to,label}`, `MoveState{id,x,y}`, `RenameState`, …). The engine turns it into a **minimal text patch** that preserves the user's formatting and comments, and the editor applies it. Undo/redo is then just the editor's history.

**Positions also live in the code**, in an optional `layout { q1 at (0, 0) … }` block (see `04-DSL.md`), so a shared link reproduces the exact picture. Without the block, Graphviz lays it out automatically.

---

## 3. Rust engine design

### 3.1 "Simple Rust" style guide (keeps it readable like Python/Swift)
- **Index IDs, not references.** `StateId(u32)`, `SymId(u32)`, `ReId(u32)`, `VarId(u32)` newtypes. All collections are `Vec`s indexed by IDs. No lifetimes in public types.
- **Owned data everywhere.** Clone when convenient; machines are small.
- **No `unsafe`** (`#![forbid(unsafe_code)]`), no async, no macros beyond `derive`, no trait-object gymnastics.
- **One file per algorithm**, named after the conversion ID (`c02_subset.rs`) with the book reference in the doc comment.
- **Errors are values:** `Result<T, TlError>`; `TlError` carries a span (for the editor) and a book reference when it's a definition violation.
- `cargo clippy -D warnings`, `rustfmt`, doc comments on every public function.

### 3.2 Core types
(Full field lists in `01-THEORY.md`.)

```rust
pub struct BitSet { words: Vec<u64> }                 // set of StateIds; Hash + Eq for subset keys
pub struct Alphabet { names: Vec<String>, index: FxHashMap<String, SymId> }
pub struct StateInfo { pub name: String, pub origin: Origin }
pub enum Origin { User, Subset(BitSet), Pair(StateId, StateId), NewStart(&'static str),
                  NewAccept(&'static str), Trap, Helper(&'static str), Apq(StateId, StateId) }

pub enum Object {                                     // anything a DSL program can name
    Dfa(Dfa), Nfa(Nfa), Gnfa(Gnfa), Regex(ReArena, ReId), Cfg(Cfg),
    Pda(Pda), Tm(Tm), MultiTm(MultiTm), Ntm(Ntm),
}

pub struct Budget { pub max_states: u32, pub max_steps: u64, pub max_trace: u32, pub max_configs: u32 }
pub enum Outcome<T> { Done(T), BudgetExceeded { partial: Option<T>, which: &'static str } }
```

### 3.3 Performance techniques
| Hot path | Technique |
|---|---|
| DFA simulation, products, minimization | dense `Vec<StateId>` δ-table, row-major |
| Subset construction | `BitSet` subsets, precomputed per-state ε-closures OR-ed together, `FxHashMap<BitSet, StateId>` |
| NFA edges | CSR index per label built once per algorithm |
| Regex / GNFA | hash-consed arena: identical sub-expressions shared, O(1) equality |
| CYK | `BitSet` per cell, rules indexed by (B, C) pair |
| PDA search | BFS with visited set; stacks stored as shared persistent lists (push = O(1), no copying) |
| TM tape | two `Vec` stacks; configurations hashed (FxHash) for repeat detection |

**Targets (≤ 50-state machines):** typing → updated diagram < 100 ms; any Chapter 1 conversion < 20 ms; TM: ≥ 50 M steps/s natively, ≥ 20 M steps/s in WASM.

### 3.4 DSL crate (`delta-dsl`)
- Hand-written lexer and **recursive-descent parser** (regexes: Pratt parser with the book's precedence). Hand-written means the best error messages and no heavy dependencies.
- **Error recovery**: one typo must not blank the diagram. The parser recovers at line or block boundaries, and the last good version of each object stays on screen, greyed out.
- **Checker** turns definition violations into diagnostics with quick fixes (see `04-DSL.md` §7).
- **Formatter** (`delta fmt`), so shared code looks consistent.

---

## 4. The WASM boundary

Small, coarse-grained, JSON-shaped API. TypeScript types are **generated from the Rust structs** (`tsify` or `ts-rs`), so the two sides can't drift apart.

```ts
// web/src/engine/api.ts (the only thing React code imports)
export interface EngineApi {
  evaluate(src: string, opts?: EvalOptions): Promise<Document>;          // parse+check+run commands
  run(objectName: string, input: string, opts?: RunOptions): Promise<RunResult>;
  convert(objectName: string, op: ConversionId, opts?: ConvertOptions): Promise<ConvertResult>;
  applyEdit(src: string, edit: EditOp): Promise<TextPatch>;
  format(src: string): Promise<string>;
  export(objectName: string, fmt: 'dot' | 'tikz' | 'json' | 'jff' | 'tl'): Promise<string>;
  importJflap(xml: string): Promise<string>;                              // returns DSL text
}
```

- **Worker + Comlink:** all calls go through a Web Worker; the main thread never blocks.
- **Cancellation:** WASM can't be interrupted mid-call, so (1) every algorithm respects a `Budget`, and (2) a "Stop" button **terminates and respawns the worker** (~50 ms). Long TM runs execute in chunks of N steps so the UI can show progress and stop cleanly.
- **Bundle:** WASM ≈ 300–600 KB gzipped (built with `opt-level="z"`, `lto`, `wasm-opt -Oz`). Graphviz WASM (~1 MB) is **lazy-loaded** on the first diagram render, and pre-fetched while the user reads the landing screen.

---

## 5. Front-end stack

| Concern | Library |
|---|---|
| Build | Vite, pnpm |
| UI primitives | Radix UI (accessible dialogs, tabs, menus) + Tailwind CSS v4 |
| State | Zustand (document store, UI store, run store) |
| Editor | CodeMirror 6 + Lezer grammar for syntax highlighting; engine diagnostics as lint markers |
| Diagram | Own SVG renderer in React; layout from Graphviz WASM (`@hpcc-js/wasm-graphviz`, `dot` engine, `rankdir=LR`, JSON output); pan/zoom with pointer events (touch-friendly); > 300 states → canvas renderer or "table view" |
| Math | KaTeX for the tuple panel (Q, Σ, δ, q₀, F rendered like the book) |
| Worker RPC | Comlink |
| Share links | `lz-string` compressed source in the URL hash (`#code=…`) — no server needed |
| Persistence | autosave draft to `localStorage` (wrapped in try/catch; private windows may block it); open/save `.delta` files |
| PWA | `vite-plugin-pwa` (offline use, install to home screen) |
| Tests | Vitest (unit), Playwright (end-to-end on phone/tablet/desktop viewports), axe (accessibility) |

**No client-side router.** It's one screen. State lives in the store and the URL hash, so GitHub Pages' lack of SPA routing is a non-issue.

---

## 6. Deployment: GitHub Pages at `delta.nandish.dev`

### 6.0 Why GitHub Pages is enough (no Render, no Vercel)
Delta is a **static site**: HTML, JS, CSS and a `.wasm` file. All computation runs in the visitor's browser, so there is no server code to host.

| GitHub Pages limit (official docs) | Delta's expected use |
|---|---|
| Published site ≤ 1 GB | ~3–5 MB (app + WASM + lazy Graphviz) |
| Bandwidth soft limit 100 GB/month | ~1.5 MB per first visit, then cached by the PWA → roughly 60 000+ new visitors/month before the soft limit |
| 10 builds/hour soft limit | **does not apply** to custom GitHub Actions workflows (ours) |
| Deploy must finish in 10 min | build + deploy ≈ 3–5 min |
| No business / commercial SaaS use | free educational tool ✔ |

**When we would need more than GitHub Pages** (none of these are in v1):
| Future need | Option |
|---|---|
| Preview URL for every pull request, custom HTTP headers | Cloudflare Pages or Vercel (static, same build) |
| Accounts, saved projects, classroom submissions, short links | a small backend: Cloudflare Workers + D1, Supabase, or Render |
| Multi-threaded WASM (needs COOP/COEP headers) | Cloudflare Pages / Vercel (Pages can't set headers) |

Moving later is cheap: the build output is just a folder, and only the DNS record changes.

### 6.1 One-time setup
1. **DNS** (wherever nandish.dev is managed): add `CNAME` record `delta` → `<your-github-username>.github.io`.
2. **Repo → Settings → Pages:** Source = *GitHub Actions*; Custom domain = `delta.nandish.dev`; tick **Enforce HTTPS** once the certificate is issued (minutes to ~1 hour).
3. `.dev` is on the HSTS preload list, so browsers **require HTTPS**. GitHub's automatic certificate covers this. Expect the site to be unreachable until the cert exists.
4. Vite `base: '/'` (custom subdomain, so no `/repo-name/` prefix).
5. Optionally verify the domain under your GitHub account's Pages settings, which prevents subdomain takeover.

### 6.2 `.github/workflows/deploy.yml`
```yaml
name: Deploy
on:
  push: { branches: [main] }
  workflow_dispatch:
permissions: { contents: read, pages: write, id-token: write }
concurrency: { group: pages, cancel-in-progress: true }

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: wasm32-unknown-unknown }
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace
      - run: ./scripts/build-wasm.sh          # see below
      - uses: pnpm/action-setup@v4             # reads "packageManager" from web/package.json
        with: { package_json_file: web/package.json }
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: pnpm, cache-dependency-path: web/pnpm-lock.yaml }
      - run: pnpm install --frozen-lockfile
        working-directory: web
      - run: pnpm test && pnpm build
        working-directory: web
      - uses: actions/upload-pages-artifact@v3
        with: { path: web/dist }
  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment: { name: github-pages, url: "${{ steps.d.outputs.page_url }}" }
    steps:
      - id: d
        uses: actions/deploy-pages@v4
```

### 6.3 `scripts/build-wasm.sh`
```bash
#!/usr/bin/env bash
set -euo pipefail
cargo build -p delta-wasm --release --target wasm32-unknown-unknown
# wasm-bindgen-cli version MUST equal the wasm-bindgen crate version in Cargo.lock
cargo install wasm-bindgen-cli --version "$(cargo pkgid -p wasm-bindgen | sed 's/.*[@#]//')" --locked
wasm-bindgen target/wasm32-unknown-unknown/release/delta_wasm.wasm \
  --target web --out-dir web/src/engine/pkg
# optional size pass if binaryen is installed:
command -v wasm-opt >/dev/null && wasm-opt -Oz web/src/engine/pkg/delta_wasm_bg.wasm -o web/src/engine/pkg/delta_wasm_bg.wasm || true
```
GitHub Pages serves `.wasm` with the correct `application/wasm` MIME type, so streaming compilation works. We need no special headers: no SharedArrayBuffer and no WASM threads in v1.

### 6.4 `ci.yml` (pull requests)
`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, golden tests, `pnpm lint`, `pnpm test`, Playwright on 3 viewports, bundle-size check (fail if WASM > 700 KB gzipped).

> **If you later want preview URLs per pull request** or custom headers, Cloudflare Pages is a drop-in upgrade: same static build, same custom domain. Not needed for v1.

---

## 7. Security and robustness
- No backend means no accounts, no user data on servers, and nothing to breach.
- Shared links contain *code*, and the code only describes machines. The engine never `eval`s anything, and every run is budgeted, so a malicious link can at most hit a step limit inside the worker.
- Fuzz the parser (`cargo-fuzz`) — it reads untrusted text from URLs.
- KaTeX/SVG output escapes all user-provided names (state names are user text).
