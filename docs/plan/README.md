# Delta — Project Plan

> **Name:** **Delta** (δ), both the language and the playground. Live at **`delta.nandish.dev`**. Files use the `.delta` extension.
> **Reference text:** Michael Sipser, *Introduction to the Theory of Computation*, 3rd ed. (Cengage, 2013). Every definition, tuple and construction in this plan is tied to a numbered Definition / Theorem / Lemma in that book, so the tool behaves exactly like the textbook.

---

## 1. What we are building

A **programmable automata playground**. You write a short piece of code that *is* the formal tuple of a machine (or a regular expression, or a grammar), and Delta:

1. **Parses** it and checks it against the formal definition (e.g. "Definition 1.5 requires δ to be total — δ(q2, 1) is missing").
2. **Draws** it as a state diagram, **shows** the formal tuple (Q, Σ, δ, q₀, F) and the δ-table side by side, kept in sync.
3. **Runs** it on input strings with step-by-step animation (DFA path, NFA "fingers", PDA stack, TM tape).
4. **Converts** it between models using *the book's own constructions*, recording every intermediate step so the student can scrub through the conversion like the figures in the book:

```
            Lemma 1.55              Theorem 1.39            (Hopcroft)
  regex  ───────────────►  NFA  ─────────────────►  DFA  ─────────────►  min-DFA
    ▲                       ▲  (DFA ⊆ NFA, trivial)  │
    │   Lemma 1.60          │                        │  Lemma 1.60 (special form)
    └──── CONVERT(G) ◄──── GNFA ◄────────────────────┘

            Theorem 2.9                Lemma 2.21              Lemma 2.27
  CFG  ───────────────►  CNF      CFG ───────────► PDA ───────────► CFG
                          │
                          └─► CYK membership (Theorem 7.16)

  TM (Def 3.3) · multitape (Thm 3.13) · nondeterministic TM (Thm 3.16) · LBA (Def 5.6)
```

5. **Decides** questions about machines (Chapter 4): A_DFA, E_DFA, EQ_DFA (with a counterexample string), A_CFG, E_CFG.
6. **Exports** to SVG/PNG, LaTeX/TikZ (for homework), Graphviz DOT, JSON, JFLAP `.jff`, and shareable links.

### Decisions locked in

| Decision | Choice |
|---|---|
| Engine | **Rust → WebAssembly**, written in a deliberately simple style, running in a Web Worker |
| Playground | **React + TypeScript + Vite** (the owner's existing stack) |
| Hosting | **GitHub Pages** via GitHub Actions at **`delta.nandish.dev`**: fully static, no backend |
| Source of truth | **The code.** Diagram edits are turned into code edits |
| Rejected | Flutter/Dart (weaker web editor, heavier load, two new languages at once). Revisit native apps later via Tauri, which reuses the same web UI |

Everything runs client-side, works on phone / tablet / desktop, and installs as a PWA.

---

## 2. Documents in this plan

| # | File | What it covers |
|---|------|----------------|
| 0 | `README.md` | This overview, scope, glossary |
| 1 | [`01-THEORY.md`](01-THEORY.md) | Every automaton model as a formal tuple (from the book), its acceptance condition, and how each tuple maps to a code type |
| 2 | [`02-CONVERSIONS.md`](02-CONVERSIONS.md) | Every conversion and algorithm: book reference, pseudocode, complexity, pitfalls, golden test |
| 3 | [`03-ARCHITECTURE.md`](03-ARCHITECTURE.md) | Language & stack decision (ADR), repo layout, Rust data structures, WASM boundary, performance |
| 4 | [`04-DSL.md`](04-DSL.md) | The "code → automaton" language: syntax, EBNF grammar, examples for every model, pipeline commands, diagnostics |
| 5 | [`05-PLAYGROUND-UX.md`](05-PLAYGROUND-UX.md) | Professional, responsive playground: layouts per screen size, views, animation, export, accessibility |
| 6 | [`06-ROADMAP-AND-TESTING.md`](06-ROADMAP-AND-TESTING.md) | Phases, milestones, team, testing strategy (golden tests from the book), risks |
| 7 | [`07-SUGGESTIONS.md`](07-SUGGESTIONS.md) | My recommendations as principal developer — what to do, what to avoid, ideas that make this stand out |
| 8 | [`08-ERRORS.md`](08-ERRORS.md) | Error and exception handling: every rule per model (DFA, NFA, regex, GNFA, CFG, PDA, TM), strict/lenient modes, runtime outcomes, crash recovery |

Read in order 1 → 4 to understand *what* we model; 3, 5, 6 to understand *how* we build it.

---

## 3. Scope

### In scope (v1.0, book Chapters 1–3 + the Chapter 4/7 algorithms that operate on them)

| Area | Book | Models / features |
|------|------|-------------------|
| Regular languages | Ch. 1 | DFA, NFA (with ε), GNFA, regular expressions; all conversions; closure operations; minimization; pumping lemma explorer |
| Context-free languages | Ch. 2 | CFG, Chomsky normal form, PDA, DPDA check, CFG ↔ PDA, derivations and parse trees, CFL pumping lemma explorer |
| Turing machines | Ch. 3 | Single-tape TM, stay-put variant, multitape TM, nondeterministic TM (computation tree), LBA, enumerator |
| Decidability algorithms | Ch. 4 | A_DFA, A_NFA, A_REX, E_DFA, EQ_DFA, A_CFG, E_CFG as runnable procedures |
| Efficient CFG parsing | Ch. 7 | CYK (Theorem 7.16) with table view |

### Stretch (v1.x)

DK-test / DCFG → DPDA (§2.4), multitape → single-tape *construction* (not just simulation), NTM → DTM construction, PCP explorer (§5.2), lexer-spec mode (many token regexes → one DFA), classroom / autograder mode.

### Out of scope

Chapters 7–10 complexity theory beyond CYK (SAT reductions, space classes, etc.) — they are about *proofs*, not runnable machines. Possibly a later "reductions" visualizer.

---

## 4. Glossary (book notation → code)

| Book | Meaning | In code |
|------|---------|---------|
| Q | finite set of states | `Vec<StateInfo>` indexed by `StateId(u32)` |
| Σ | input alphabet | `Alphabet` (interned symbols → `SymId(u32)`) |
| Σ_ε | Σ ∪ {ε} | `Label::{Eps, Sym(SymId)}` |
| Γ | stack (PDA) or tape (TM) alphabet | `Alphabet` |
| δ | transition function | dense table (DFA/TM), adjacency lists (NFA/PDA), matrix of regex ids (GNFA) |
| P(Q) | power set of Q | `BitSet` over state ids |
| E(R) | ε-closure of a set R of states | `fn eps_closure(&BitSet) -> BitSet` |
| q₀, F | start state, accept states | `StateId`, `BitSet` |
| ⊔ | TM blank | dedicated `TapeSym::BLANK` |
| L(M) | language of M | (not materialized — decided via algorithms) |
| ⟨M⟩ | string encoding of M | canonical JSON / DSL text |

---

## 5. Guiding principles

1. **Book-faithful first, clever second.** Every conversion has a *faithful* mode that reproduces Sipser's construction exactly (same number of states, same labels), and optionally an *optimized* mode. Students must be able to check homework against it.
2. **The code is the tuple.** The DSL mirrors the formal definition so closely that writing it teaches the definition.
3. **Show the work.** Every algorithm returns a *trace* of steps, not just a result.
4. **Never freeze.** All computation runs in a Web Worker with budgets (max states, max steps, time), because subset construction is exponential and Turing machines may not halt.
5. **Every screen size is a first-class citizen.** Phone layout is designed, not shrunk.
6. **Correctness is tested against the book.** Worked examples from the text are golden tests; random round-trips (regex → NFA → DFA → regex) are property tests.
