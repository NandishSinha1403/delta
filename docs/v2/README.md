# Delta 2 — Plan (full)

> **Delta** (δ) is a programming language for the theory of computation, with a playground, a website and mobile apps.
> Write machines, languages and constructions as code. Press Run and see diagrams, traces and answers.
> **delta.nandish.dev** · Android · iOS

---

## 1. What Delta 2 is

A **complete language** (Turing-complete, with functions, loops and data) whose values are the objects of computation theory: **machines, languages, words, grammars, configurations and computations.** It's a domain language, like R for statistics or Lean for proofs, not a general-purpose one.

Three ideas make it new:

1. **A machine is finite control + storage + an acceptance condition.** DFA, NFA, PDA, TM, Mealy, counter machines and two-way automata aren't hard-coded. They're `model`s in a standard prelude, and users can define more.
2. **Languages are values, and their decidability class is their type.** Delta knows that equivalence of regular languages is decidable and equivalence of context-free languages isn't. It refuses undecidable questions at compile time and offers a bounded check instead.
3. **Constructions read like definitions.** `explore` (start, successors, acceptance) builds machines; `fixpoint` computes closures. The subset construction is four lines.

Nothing in the language assumes one textbook. Book citations live in optional **explain profiles**.

## 2. What we're shipping

| Product | Where | What |
|---|---|---|
| **Website (front page)** | `delta.nandish.dev/` | Landing page with a live, editable example; feature tour; links to the Pad, Docs and apps |
| **Pad** | `/pad` | DartPad-style playground: write, Run, see diagram / console / table / steps / tests |
| **Docs** | `/docs` | Language tour, generated library reference, cookbook, notation guide, error pages, AI guide |
| **Embeds** | `/pad/embed` | Live snippets for teachers' pages and slides |
| **Android and iOS apps** | Play Store, App Store | The Pad as a native-feeling app: offline, file open/save, share sheet. Wrapped from the same web code with Capacitor |
| **Later** | | HTTP API and MCP server for AI tools; accounts and saved projects |

## 3. Decisions (locked unless you change them)

| Area | Decision |
|---|---|
| Name | **Delta** (δ), files `.delta` |
| Syntax | **Braces, newline-terminated (no semicolons), Python vocabulary.** One canonical formatter; no alternate indentation syntax |
| Math notation | Unicode and ASCII both accepted; `\delta` + Tab inserts δ (LaTeX names); formatter normalizes |
| Comprehensions | **Math set-builder only:** `{ x ∈ S \| p(x) }`, `{ f(x) \| x ∈ S }` |
| Machines | One `machine` construct over **models** (storage + acceptance); dfa/nfa/pda/tm/… are prelude models |
| Languages | `lang` values with a **decidability class** (Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable), checked at compile time |
| Types | Inferred; annotations optional; checker catches malformed machines, alphabet mismatches and undecidable questions |
| Nondeterminism | `choose` keyword **reserved**; feature after v2.0 |
| Engine | Rust → WebAssembly in a Web Worker; tree-walking interpreter; algorithms ported from v1 |
| Web | React + TS + Vite (Pad); Astro + Starlight (front page + docs) |
| Mobile | **Capacitor** wrapping the Pad (Android + iOS); a PWA comes first |
| Hosting | Cloudflare Pages (static, direct upload). A server only for the later API, MCP and accounts |
| Look | Near-black UI; palette #E63946 / #F1FAEE / #A8DADC / #457B9D / #1D3557; Courier Prime code; VS Code Dark+ syntax |
| Codebase | Build on v1: keep engine algorithms and tests, replace the language layer, restructure the UI |

## 4. Documents

| File | Contents |
|---|---|
| [`01-LANGUAGE.md`](01-LANGUAGE.md) | The language: syntax, values, models and machines, languages and classes, explore/fixpoint, library, errors, grammar |
| [`02-ENGINE.md`](02-ENGINE.md) | Crates, model system, class lattice, interpreter, explain profiles, output JSON, performance |
| [`03-PRODUCTS.md`](03-PRODUCTS.md) | Front page, Pad, Docs, embeds, mobile apps, design system |
| [`04-ROADMAP.md`](04-ROADMAP.md) | Migration from v1, phases with hour estimates, testing, risks, API/MCP later |
| [`AUDIT.md`](AUDIT.md) | Phase 0: keep / change / drop for every v1 module, book strings, coupling, test inventory |
| [`SEMANTICS.md`](SEMANTICS.md) | Phase 0: formal meaning of models, acceptance, labels, languages, `explore`, `fixpoint`, the core |
| [`QUESTIONS.md`](QUESTIONS.md) | Phase 0: open questions and contradictions, each with a recommended default |

The v1 plan in `docs/plan/` is history. **v2 wins** wherever they disagree.

## 5. Principles

1. **General first.** Write what you'd write on a whiteboard.
2. **Exact semantics.** Every construct desugars to a small, documented core.
3. **The theory is in the type system.** Closure properties and decidability are checked, not just documented.
4. **Fast built-ins, open extension.** Rust for heavy algorithms; Delta for everything else, including new models.
5. **Show the work.** Every value has a view; every construction has a trace.
6. **Honest limits.** Budgets, not hangs; "undecidable" said out loud; bounded checks labeled as such.
7. **Humans and AIs are both users.** Deterministic output, as text and JSON.
8. **Every example runs.** Docs, spec and cookbook are tested in CI.
