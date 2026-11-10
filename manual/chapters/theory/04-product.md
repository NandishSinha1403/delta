---
title: "Product construction"
description: "Intersection, union and difference of regular languages by running two DFAs in parallel."
sidebar:
  order: 4
---

Readers: **owner**. Sipser 3e: Theorem 1.25 (closure under union, proved with the product) and
the remark after it for intersection.

## Definition

For DFAs A = (Q_A, Σ, δ_A, a₀, F_A) and B = (Q_B, Σ, δ_B, b₀, F_B), the product has states
Q_A × Q_B, start (a₀, b₀), and δ((p, q), s) = (δ_A(p, s), δ_B(q, s)). Accepting pairs depend on
the operation: both (∩), either (∪), first and not second (∖), exactly one (symmetric
difference).

## Why it's correct

Induction on w: after reading w the product is in (δ_A(a₀, w), δ_B(b₀, w)). So it accepts w iff
`keep(A accepts w, B accepts w)`.

## Complexity

O(|Q_A| · |Q_B| · |Σ|) time and states, building only reachable pairs.

## In Delta

`crates/delta-core/src/fa/product.rs`, id `product/pairs`. One function with a `keep`
predicate serves every Boolean operation; a missing transition on either side drops the pair
(the inputs are expected to be total where it matters).

<!-- excerpt: crates/delta-core/src/fa/product.rs#product -->

The equivalence decision is the same idea run on subset automata, on the fly, looking for a pair
where `keep` = "exactly one accepts" — see [Equivalence](/theory/05-equivalence/).

## Questions they'll ask

1. **"Why can't you do union with an NFA instead?"** — You can (new start with ε to both); the
   product is needed for intersection and complement-based operations, and gives a DFA directly.
2. **"Is CF closed under intersection?"** — No ({aⁿbⁿcᵐ} ∩ {aᵐbⁿcⁿ} = {aⁿbⁿcⁿ}). But CF ∩ Regular
   is CF: a product of a PDA with a DFA. Delta's closure table encodes both.
3. **"What if the alphabets differ?"** — Delta compiles both sides over the union alphabet; a
   symbol one side doesn't know leads to its dead state.
4. **"Size of the product of two minimal DFAs — is it minimal?"** — Not necessarily; minimize
   afterwards if you need it.
5. **"How do you get difference?"** — keep = (x ∧ ¬y); complement is the product with a
   one-state universal DFA, or swapping accept states of a total DFA.

## Self-test

<details class="selftest"><summary>1. Product transition?</summary>δ((p,q), s) = (δ_A(p,s), δ_B(q,s)).</details>
<details class="selftest"><summary>2. Accepting pairs for intersection?</summary>Both components accepting.</details>
<details class="selftest"><summary>3. Size bound?</summary>|Q_A| · |Q_B| reachable pairs.</details>
<details class="selftest"><summary>4. Is CF ∩ CF context-free?</summary>Not in general (only CS); CF ∩ Regular is CF.</details>
<details class="selftest"><summary>5. What id does Delta give it?</summary><code>product/pairs</code>.</details>
