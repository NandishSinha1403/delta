---
title: "Equivalence with a witness"
description: "Deciding L(A) = L(B) for finite automata and returning the shortest word where they differ."
sidebar:
  order: 5
---

Readers: **owner**. Sipser 3e: Theorem 4.5 (EQ_DFA is decidable, via emptiness of the symmetric
difference, Theorem 4.4).

## Definition

L(A) = L(B) iff the symmetric difference (L(A) ∖ L(B)) ∪ (L(B) ∖ L(A)) is empty. Sipser builds a
DFA for the symmetric difference and tests emptiness. Delta does the same thing **on the fly**:
breadth-first search over pairs (subset of A, subset of B), stopping at the first pair where
exactly one side accepts. The path to it is a **witness**.

## Why it's correct

The pairs reachable on w are exactly (A's configuration set on w, B's on w) — the subset
construction for each side, run in parallel (the product). A pair with exactly one side accepting
exists iff some word is in exactly one language. Breadth-first with symbols in index order means
the first such pair found has the shortest word, and among those the lexicographically least:
the **shortlex-least** counterexample.

## Complexity

For two DFAs: O(|A| · |B| · |Σ|). For NFAs: up to 2^|A| · 2^|B| pairs — equivalence of NFAs is
PSPACE-complete, so no algorithm avoids that in the worst case. Bounded by the states budget.

## In Delta

`crates/delta-core/src/fa/decide.rs`, id `decide/equivalence`. Inclusion is the same search with
`bad = x ∧ ¬y`; universality compares with Σ*; emptiness is a reachability check.

<!-- excerpt: crates/delta-core/src/fa/decide.rs#equivalence-search -->

When one side isn't known to be regular, `check … up to n` compares membership on all words up
to length n (Σ_{i≤n} kⁱ of them), and the result says `bounded: n` — evidence, not proof. See [Life of one program](/life-of-a-program/).

## Questions they'll ask

1. **"Why not minimize both and compare?"** — That works for DFAs (minimal DFAs are unique up to
   isomorphism) but gives no witness and costs a full minimization; the search stops at the first
   difference and gives the word.
2. **"Why is the witness the shortest?"** — BFS explores by length; parent pointers rebuild the
   word; symbols in order make ties lexicographic.
3. **"Is equivalence decidable for context-free languages?"** — No (EQ_CFG is undecidable). For
   deterministic CF it is (Sénizergues, 1997) but impractical; Delta reports E702
   "not implemented" for DCF and E701 for CF.
4. **"Why ε-close each step?"** — NFAs with ε-edges; each subset must include ε-reachable states
   before testing acceptance.
5. **"How would Hopcroft–Karp's union-find method compare?"** — Near-linear for DFAs, merges
   states assumed equal; it also yields a witness. The BFS was chosen for simplicity and because
   it works on NFAs via subsets without building them first.
6. **"What does `check A ⊆ B` return?"** — The shortest word in A but not B.

## Self-test

<details class="selftest"><summary>1. What does the BFS search over?</summary>Pairs of ε-closed state sets (A's, B's).</details>
<details class="selftest"><summary>2. What makes a pair "bad" for equivalence?</summary>Exactly one side accepts.</details>
<details class="selftest"><summary>3. Complexity for two DFAs?</summary>O(|A|·|B|·|Σ|).</details>
<details class="selftest"><summary>4. Which Sipser theorem?</summary>Theorem 4.5 (via 4.4, emptiness).</details>
<details class="selftest"><summary>5. What does Delta say for equivalence of two CF languages?</summary>E701 (undecidable), with a bounded alternative.</details>
