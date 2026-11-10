---
title: "0b. Pitches"
description: Delta in 30 seconds, 2 minutes and 5 minutes, plus résumé bullets with real numbers.
---

Readers: **owner**. Learn the 30-second version word for word; know the others well enough to
say them in your own words.

## 30 seconds

> Delta is a programming language for the theory of computation. In a course you draw a DFA on
> paper and convince yourself it's right; in Delta you write it as code, and the language runs
> it, draws it, and checks claims like "this machine accepts exactly the words ending in *ab*".
> If the claim is false it shows you the shortest word where it fails. If the claim is one no
> algorithm can decide, it says so before running and offers a bounded check instead. It's a
> Rust engine compiled to WebAssembly, a browser playground, a CLI and a docs site, live at
> delta.nandish.dev.

## 2 minutes

> Automata theory is taught on paper. Students draw machines, but nothing checks them, and the
> constructions in the textbook — subset construction, minimization, converting a machine to a
> regular expression — are done by hand, slowly, with mistakes nobody catches.
>
> Delta makes those objects values in a language. A DFA is a literal you write the way you'd
> draw it: `q0 -a-> q1`. A language can be written as set-builder notation,
> `{ w ∈ {a,b}* | w ends with "ab" }`. Then `check M ≡ L` asks whether they're equal.
>
> The interesting part is that the type system knows the theory. Every language has a class —
> finite, regular, context-free, decidable and so on — and the checker uses the closure and
> decision tables from the course. Equivalence of two regular languages is decidable, so Delta
> decides it exactly and gives the shortest counterexample. Equivalence of a regular and an
> arbitrary decidable language is not, so the checker rejects it with error E701 and offers
> `up to 12`, which compares all words up to length 12.
>
> Technically it's a Rust workspace: a lexer and recursive-descent parser with error recovery, a
> checker, a tree-walking interpreter, and the automata algorithms, compiled to a 298 KB
> (gzipped) WebAssembly module that runs in a Web Worker behind a React playground. The same
> engine powers a CLI with a REPL that draws machines inline in the terminal. It has a
> conformance suite where every example in the language spec is a test.

## 5 minutes

Use the 2-minute version, then pick **two** of these depending on the audience:

- **For a systems or backend interviewer — the engine.** One run: the Pad debounces typing by
  150 ms, sends the source to a Web Worker over Comlink, the worker calls the WASM `evaluate`,
  which parses (two lexing modes: normal and machine-body), checks, interprets, and returns one
  JSON document. Nothing in the engine may panic — `unwrap` and `panic!` are denied by Clippy —
  and every algorithm runs under a budget (10 million steps, 100,000 states, recursion depth
  400) so a runaway Turing machine is an *outcome*, not a crash. If the worker takes more than
  10 seconds, the client kills and respawns it, because WebAssembly can't be interrupted.
- **For a theory or PL interviewer — the class lattice.** Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂
  Decidable ⊂ Recognizable ⊂ Any. Operations compute classes from tables (union of two CF
  languages is CF; complement of CF is only CS), and questions are allowed only where they're
  decidable. Equivalence uses breadth-first search over pairs of subset-construction states, so
  the first difference found is the shortlex-least counterexample.
- **For a frontend interviewer — the Pad.** React 19 + TypeScript strict, CodeMirror 6 with a
  Delta grammar, panels for the diagram, the formal tuple, step-by-step runs and construction
  replays, share links that carry the program compressed in the URL fragment (no server), a PWA
  that works offline, and Playwright tests on phone, tablet and desktop.
- **For a product or general interviewer — the process.** Phase 0 was an audit of v1 and a
  formal semantics before any v2 code; it found the spec's own example was false and that
  algorithms the plan said would be "ported" didn't exist. Each phase ends green: format, lint,
  all tests, WASM size budget, end-to-end tests.

Close with what's next: pushdown automata and grammars (CNF, CYK, PDA↔CFG) in Phase 6, then
mobile apps wrapped with Capacitor.

## Résumé bullets

Numbers measured 10 NOV 2026 at commit `{{commit}}`; re-measure before using them.

- Designed and built **Delta**, a programming language for automata theory: 14 machine models
  (DFA, NFA, PDA, DPDA, TM, NTM, LBA, multi-tape TM, Mealy, Moore, FST, counter, two-way DFA,
  GNFA) defined in Delta's own 40-line prelude, with a type system that encodes closure and
  decidability (8 language classes).
- Wrote the engine in **Rust (~18,500 lines across the v2 crates)**: lexer, recursive-descent
  parser with error recovery, static checker, tree-walking interpreter, subset construction,
  four minimization algorithms, three regex→NFA and three NFA→regex constructions, and
  equivalence with shortest counterexamples.
- Shipped it to the browser as a **298 KB gzipped WebAssembly** module (budget 700 KB) in a Web
  Worker behind a **React 19 + TypeScript** playground with share links, embeds and offline PWA
  support.
- Built a test pyramid of **145 Rust tests** (including a **54-case conformance suite** that
  turns every spec example into a test), **26 Vitest** and **90 Playwright** tests across phone,
  tablet and desktop; every code block in the docs is executed in CI.
- Wrote a dependency-free **graph layout and SVG/TikZ renderer** and a shared PNG/PDF exporter
  (resvg, svg2pdf) so the CLI and the browser produce byte-identical images; published a
  **142-page** static docs site (Astro + Starlight) on Cloudflare Pages with Lighthouse 100
  performance and SEO on the front page.

## Questions they'll ask

1. **"Who is it for?"** — Students and teachers of automata theory first; it's built around the
   standard course (finite automata to Turing machines). Secondary: anyone who wants to test a
   claim about a regular language quickly. Say the 30-second pitch, then stop.
2. **"Why a language and not a drawing tool like JFLAP?"** — Drawing tools build one machine at
   a time. A language lets you write *constructions*: a function that builds the subset
   automaton of any NFA is twelve lines of Delta, and it can be checked against the built-in.
   It also lets the type system enforce decidability, which no drawing tool does.
3. **"What was the hardest part?"** — Pick one and be concrete: making equivalence give the
   *shortest* counterexample (breadth-first over pairs, symbols in order), or keeping the
   engine panic-free under budgets, or the two-mode lexer for machine bodies. Then the war story
   behind it.
4. **"How do you know it's correct?"** — The conformance suite (every spec example with its
   expected JSON), oracle tests that compare algorithms against brute force on all short words,
   and property tests (Hopcroft, Moore, Brzozowski and the table method must give the same
   minimal DFA).
5. **"How much did you write yourself, and how did you use AI?"** — Answer truthfully. Describe
   what you did (the spec, the decisions, the reviews, the phase plan) and what AI coding tools
   did, and show that you understand every part. This manual exists so you can.
6. **"What would you do differently?"** — Start with the formal semantics (Phase 0 came only in
   v2), and choose one renderer from the start (the Pad still uses Graphviz on screen while
   exports use Delta's own layout).
7. **"Does anyone use it?"** — Be honest: it's live, but analytics aren't on yet, so you don't
   have numbers. Say what you'd measure (programs run, share links opened, docs searches).

## Self-test

<details class="selftest"><summary>1. Say the 30-second pitch. What are its three verbs?</summary>Runs, draws, checks. Plus: the shortest counterexample when a claim is false, and E701 with a bounded alternative when it's undecidable.</details>
<details class="selftest"><summary>2. What is the gzipped size of the engine WASM, and its budget?</summary>298 KB, budget 700 KB (the PNG/PDF export module is separate: 887 KB, loaded only on download, budget 1,000 KB).</details>
<details class="selftest"><summary>3. Name the eight language classes in order.</summary>Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any.</details>
<details class="selftest"><summary>4. What are the three default budgets?</summary>10,000,000 steps, 100,000 states, recursion depth 400 (and 10,000 outputs).</details>
<details class="selftest"><summary>5. What happens when a program runs too long in the browser?</summary>After 10 seconds the client terminates the Web Worker and starts a new one; WASM can't be interrupted. Inside the engine, budgets usually stop it first with a B6xx outcome.</details>
