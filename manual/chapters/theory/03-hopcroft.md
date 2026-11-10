---
title: "Hopcroft minimization"
description: "DFA minimization by partition refinement: definition, correctness, complexity, and Delta's minimize/hopcroft."
sidebar:
  order: 3
---

Readers: **owner**. Sipser 3e covers minimal DFAs through the Myhill–Nerode theorem in the
problems of chapter 1 (Problem 1.52); Hopcroft's algorithm is from Hopcroft (1971).

## Definition

Two states p, q are **equivalent** if for every word w, δ(p, w) is accepting iff δ(q, w) is. The
minimal DFA merges equivalent reachable states. Hopcroft computes the coarsest partition that is
stable:

1. Start with {F, Q ∖ F}.
2. Keep a worklist of **splitters** (B, a). Taking one: let pre = states that go into B on a.
   Every block X that has states both in and out of pre splits into X ∩ pre and X ∖ pre.
3. Add the **smaller** half (with every symbol) to the worklist. Repeat until it's empty.

## Why it's correct

Splitting only ever separates states that are distinguishable (one goes into B on a, the other
doesn't, and B is already known to be distinguishable from its complement), so equivalent states
are never split. When the worklist is empty, every block is stable against every block and symbol,
so states in one block can't be distinguished by any word (induction on word length). The
"smaller half" trick is sound because splitting by B and by X ∖ B is equivalent once X has been
used.

## Complexity

Textbook Hopcroft: O(k · n log n), because each state can be in a "smaller half" at most log n
times. Delta's version recomputes the partition for every splitter (it scans all blocks), which is
simpler but closer to O(k · n² log n) on bad inputs — fine at teaching sizes; listed as a known
issue for big DFAs.

## In Delta

`crates/delta-core/src/fa/minimize.rs`, id `minimize/hopcroft`. It first makes the DFA total and
drops unreachable states (`prepare`), builds inverse transitions, refines, then builds the
quotient with states numbered breadth-first. Three other algorithms — `minimize/moore`,
`minimize/brzozowski` (determinize the reverse, twice) and `minimize/table` (mark distinguishable
pairs) — must give the same result, which is a strong test.

<!-- excerpt: crates/delta-core/src/fa/minimize.rs#hopcroft-loop -->

## Questions they'll ask

1. **"Why is the minimal DFA unique?"** — Myhill–Nerode: its states correspond to the
   equivalence classes of the relation "x ≡ y iff no suffix z separates xz and yz"; any minimal
   DFA is isomorphic to that.
2. **"Moore vs Hopcroft?"** — Moore refines all blocks by successor classes each round,
   O(k n²) worst case; Hopcroft only processes splitters and keeps the smaller half, O(k n log n).
3. **"How does Brzozowski's method work?"** — Reverse, determinize, reverse, determinize. The
   result is minimal because determinizing a co-deterministic, reachable automaton yields the
   minimal DFA. Can be exponential in between.
4. **"What do you do with unreachable states?"** — Remove them first; they'd survive refinement
   as separate blocks otherwise.
5. **"Why make the DFA total first?"** — Missing transitions behave like a dead state; making it
   explicit lets refinement treat it uniformly.
6. **"How do you test four algorithms agree?"** — Same input, compare outputs up to the BFS
   numbering (they're canonical), on random DFAs.

## Self-test

<details class="selftest"><summary>1. Initial partition?</summary>{F, Q ∖ F} (dropping an empty block).</details>
<details class="selftest"><summary>2. What is "pre" for a splitter (B, a)?</summary>All states with an a-transition into B.</details>
<details class="selftest"><summary>3. Why add only the smaller half?</summary>Correctness: splitting by B and its complement within X is equivalent; complexity: each state is in a smaller half O(log n) times.</details>
<details class="selftest"><summary>4. Complexity of textbook Hopcroft?</summary>O(k · n log n).</details>
<details class="selftest"><summary>5. Name the other three minimization methods in Delta.</summary>Moore, Brzozowski (double reversal), table filling.</details>
