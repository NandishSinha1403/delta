---
title: "Subset construction"
description: "NFA → DFA: definition, correctness, complexity, and Delta's determinize/subset."
sidebar:
  order: 1
---

Readers: **owner** (theory interviews). Sipser 3e: Theorem 1.39 and Corollary 1.40.

## Definition

Given an NFA N = (Q, Σ, δ, q₀, F), build a DFA D whose states are **sets of N's states**:

- E(R) = the ε-closure of R: every state reachable from R by ε-edges alone.
- Start: E({q₀}).
- δ_D(R, a) = E( ⋃_{q ∈ R} δ(q, a) ).
- R is accepting iff R ∩ F ≠ ∅.

Only sets reachable from the start are built (the "lazy" or "reachable" variant).

## Why it's correct

Invariant: after reading w, D is in exactly the set of states N could be in after reading w
(ε-closed). Induction on |w|: true for ε by the start state's definition; if true for w, then
δ_D's definition is literally "all states reachable by one a-edge from a current state, then ε".
So D accepts w iff some state N could be in is accepting iff N accepts w.

## Complexity

At most 2^|Q| states, each with |Σ| transitions, each computed in O(|Q|²) (step plus closure):
O(2^n · k · n²) worst case. The bound is tight: the NFA for "the n-th symbol from the end is 1"
has n + 1 states and its minimal DFA has 2ⁿ. In practice the reachable part is often small.

## In Delta

`crates/delta-core/src/fa/determinize.rs`, id `determinize/subset`. Sets are sorted `Vec<usize>`
(so they hash and compare cheaply); `index` maps each set to its new state number; breadth-first
order with symbols in order makes the numbering deterministic. Every new state and edge records a
`Step` for the Steps panel. The budget is `max_states`:

<!-- excerpt: crates/delta-core/src/fa/determinize.rs#subset -->

The same construction can be written by a user with `explore` (the tour does this), and
`check subsets(N) ≡ N` checks it against the built-in.

## Questions they'll ask

1. **"Why ε-closure on the start too?"** — The NFA can move along ε before reading anything; the
   start set must include those states or words like ε would be misjudged.
2. **"Is the empty set a state?"** — When it's reachable, yes: it's the dead state, and keeping
   it makes D total. Delta keeps it.
3. **"Give an example of exponential blow-up."** — "The n-th symbol from the end is a 1": the DFA
   must remember the last n symbols, 2ⁿ states; the NFA guesses the position with n + 1 states.
4. **"How do you represent sets efficiently?"** — Sorted vectors as hash keys here; bitsets are
   the alternative (delta-core has a bitset type) and are faster for dense sets.
5. **"Why breadth-first?"** — Deterministic numbering (q0, q1, … in discovery order) for stable
   diagrams and traces, and it's what the Steps panel replays.
6. **"Does the result need minimizing?"** — It can have equivalent states; `minimize` is a
   separate step so students can see both.

## Self-test

<details class="selftest"><summary>1. Define δ_D(R, a).</summary>E(⋃_{q∈R} δ(q, a)).</details>
<details class="selftest"><summary>2. When is a subset state accepting?</summary>When it contains an accepting NFA state.</details>
<details class="selftest"><summary>3. Worst-case number of DFA states?</summary>2^|Q|.</details>
<details class="selftest"><summary>4. State the invariant that proves correctness.</summary>After reading w, D's state is exactly the ε-closed set of NFA states reachable on w.</details>
<details class="selftest"><summary>5. What stops Delta's version on a huge NFA?</summary>The states budget (<code>max_states</code>, default 100,000) → Err, reported as a budget outcome.</details>
