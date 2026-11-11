---
title: "State elimination"
description: "Automaton → regular expression via a GNFA: definition, correctness, complexity, and Delta's to_regex."
sidebar:
  order: 2
---

Readers: **owner**. Sipser 3e: Lemma 1.60 (with Definition 1.64 of a GNFA); the other direction
is Lemma 1.55.

## Definition

A **generalized NFA** (GNFA) has regular expressions on its edges. Convert an automaton:

1. Add a new start state s with an ε-edge to the old start, and a new accept state f with
   ε-edges from every old accepting state. Missing edges are labelled ∅; parallel edges are
   joined with ∪.
2. Repeatedly **rip** a state r (not s or f): for every pair (i, j) of remaining states, replace
   the label R(i, j) with R(i, j) ∪ R(i, r) · R(r, r)* · R(r, j).
3. When only s and f remain, R(s, f) is the answer.

## Why it's correct

Ripping r preserves the language: any path through r enters it from some i, loops on r zero or
more times, and leaves to some j — exactly R(i, r) R(r, r)* R(r, j); paths not through r are
untouched. Induction on the number of states ripped.

## Complexity

O(n³) regex operations (n rips × n² pairs), but the **regex size** can grow exponentially, and
the elimination order matters a lot. Delta offers three methods: state elimination in a given
order, `brzozowski-mccluskey` (always rip the state with the fewest incoming × outgoing edges, a
good heuristic), and Arden's lemma (solve X = AX + B ⇒ X = A*B).

## In Delta

`crates/delta-core/src/fa/to_regex.rs`, id `to_regex/state-elimination`. The GNFA is an
(n + 2) × (n + 2) matrix of `Re`; `Re::alt`, `Re::cat` and `Re::star` simplify as they build
(∅ ∪ R = R, ε·R = R, ∅·R = ∅), which keeps results readable. The rip:

<!-- excerpt: crates/delta-core/src/fa/to_regex.rs#rip -->

## Questions they'll ask

1. **"Why add new start and accept states?"** — So the start has no incoming edges and there's a
   single accept state with no outgoing edges; then ripping never needs special cases and the
   answer is one label.
2. **"Why does order matter?"** — Different orders give equivalent but very different-sized
   regexes; ripping low-degree states first keeps them small.
3. **"How do you test it?"** — All three methods must give regexes whose NFAs are equivalent to
   the input (checked with the equivalence decision), on random automata.
4. **"Is the output minimal?"** — No; regex minimization is PSPACE-hard. Delta simplifies
   locally only.
5. **"What's Arden's lemma?"** — If ε ∉ L(A), the equation X = AX ∪ B has the unique solution
   X = A*B. Writing one equation per state and substituting gives the regex.
6. **"Which direction is easier?"** — Regex → NFA (structural, linear size). Automaton → regex is
   the one with blow-up.

## Self-test

<details class="selftest"><summary>1. The rip rule for (i, j) when removing r?</summary>R(i,j) ∪ R(i,r) R(r,r)* R(r,j).</details>
<details class="selftest"><summary>2. Why can the result be exponentially large?</summary>Each rip can copy labels into many pairs; nested stars and unions multiply.</details>
<details class="selftest"><summary>3. What heuristic does brzozowski-mccluskey use?</summary>Rip the state with the fewest (incoming × outgoing) edges next.</details>
<details class="selftest"><summary>4. Which Sipser result is this?</summary>Lemma 1.60 (via GNFAs).</details>
<details class="selftest"><summary>5. What simplifications does Re::alt/cat apply?</summary>∅ is the identity of ∪ and the zero of ·; ε is the identity of ·.</details>
