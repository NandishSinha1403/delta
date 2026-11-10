---
title: "1. The idea in one page"
description: The problem Delta solves, for whom, and what it deliberately doesn't do.
---

Readers: **visitor**, **owner**.

**The problem.** The theory of computation is taught with pictures and proofs. Students draw a
DFA, run it by hand on three words, and hope. Constructions like the subset construction or
state elimination are long, mechanical and error-prone on paper, and a mistake is invisible
until it's graded. Existing tools (JFLAP and its successors) let you draw one machine, but you
can't *program* with machines: you can't write a construction once and apply it to any input, and
nothing tells you that the question you just asked has no algorithm.

**Who has it.** Students in a first automata course (finite automata → context-free → Turing
machines), teachers who want live examples and checkable exercises, and anyone who wants to test
a claim about a regular language in seconds.

**What Delta does about it.** It makes machines, languages, regexes and grammars **values in a
language**:

```delta
dfa EndsAB {
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q2
  q2 -a-> q1;   q2 -b-> q0
}
lang L = { w ∈ {a,b}* | w ends with "ab" }
check EndsAB ≡ L up to 10
check EndsAB ≡ re"(a ∪ b)* a b"
```

Delta runs that program, draws the machine, and checks both claims: the first by comparing all
words up to length 10 (because `L` is only known to be *decidable*), the second exactly (both
sides are regular), with the shortest counterexample if they differ.

- **Runs** any machine model step by step, with the verdicts accept, reject, loops or unknown.
- **Draws** state diagrams, formal tuples and transition tables.
- **Checks** claims, and refuses undecidable ones before running (error E701), offering a bounded
  check instead.
- **Builds** with constructions written like their definitions (`explore`, `fixpoint`).

**What it deliberately doesn't do.**

- It doesn't prove theorems. A bounded check (`up to n`) is evidence, not proof, and the output
  says `bounded: n`.
- It doesn't pretend undecidable questions are decidable. No timeouts dressed up as answers.
- It has no server, accounts or storage (yet). Programs live in the browser and in share links.
- It doesn't quote the textbook in the engine. Citations live only in explain profiles (and in
  this private manual).

## Questions they'll ask

1. **"Isn't this JFLAP?"** — JFLAP draws and simulates one machine. Delta is a language: machines
   are values, constructions are functions, languages have classes, and decidability is
   checked. You can write the subset construction yourself and `check` it against the built-in.
2. **"Why would a student write code instead of drawing?"** — They still see the drawing; the
   code is the source of it. Code can be shared in a link, diffed, graded automatically and
   checked against a specification language.
3. **"What does 'checks claims' mean exactly?"** — `check A ≡ B`, `check A ⊆ B`,
   `check M accepts "w"`, `assert expr`. Each becomes a test result with pass/fail and, for ≡ and
   ⊆, a counterexample word.
4. **"What's the scope today?"** — Finite automata and regexes fully (constructions, minimization,
   decisions); every machine model can be simulated; context-free grammars have membership
   (Earley), emptiness and finiteness. CNF, CYK and PDA↔CFG conversions are Phase 6.
5. **"Why not just a Python library?"** — Notation. `q0 -a-> q1`, `{ w ∈ Σ* | … }` and `M ≡ L` are
   how the course writes things, and a dedicated checker can reject an undecidable question
   before running. A library can't give you either without a lot of ceremony.

## Self-test

<details class="selftest"><summary>1. What are the four things Delta does?</summary>Runs, draws, checks, builds.</details>
<details class="selftest"><summary>2. Why is `check EndsAB ≡ L` (no bound) rejected?</summary>L is a set-builder language, so its class is Decidable; equivalence is undecidable once either side is beyond the decidable-question boundary (here Regular vs Decidable), so the checker reports E701 and offers `up to 12`.</details>
<details class="selftest"><summary>3. Name two things Delta deliberately doesn't do.</summary>Prove theorems; pretend undecidable questions are decidable; store programs on a server; quote the textbook in the engine.</details>
<details class="selftest"><summary>4. What are the four run verdicts?</summary>accept, reject, loops (a configuration repeats on every branch), unknown (budget ran out).</details>
<details class="selftest"><summary>5. Who is the primary user?</summary>Students (and teachers) in a first automata-theory course.</details>
