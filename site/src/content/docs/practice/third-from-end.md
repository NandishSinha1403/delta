---
title: "Subset construction: third symbol from the end"
description: "Solved GATE-style problem: how many DFA states the subset construction gives for the NFA 'third symbol from the end is 1', and why none can be removed."
sidebar:
  label: "4. Third from the end"
  order: 4
---

**Numerical answer type.** An NFA with 4 states accepts the binary strings whose third symbol
from the end is `1`. How many states does the minimal DFA for this language have?

## Solution

A DFA has to remember the last three symbols it read, because any of them may turn out to be the
third from the end. There are `2³ = 8` possible windows of three symbols, and any two different
windows are distinguishable. If two windows differ at the `i`-th symbol from the end
(`i ≤ 3`), append `3 - i` more symbols: that position becomes the third from the end, so exactly
one of the two extended words is accepted.

The subset construction on the 4-state NFA reaches exactly 8 sets, one per window, and
minimization merges none of them.

**Answer: 8.** For "the k-th symbol from the end is 1", the NFA has `k + 1` states and the
minimal DFA `2ᵏ`: the standard example of the exponential gap.

## Check it in Delta

```delta
nfa N {
  alphabet 0, 1
  start q0
  accept q3
  q0 -[0, 1]-> q0
  q0 -1-> q1
  q1 -[0, 1]-> q2
  q2 -[0, 1]-> q3
}
let D = determinize(N)
assert D.size == 8
assert minimize(D).size == 8
```
