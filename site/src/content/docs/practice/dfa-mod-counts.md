---
title: "Minimum DFA states: a's mod 3 and b's mod 2"
description: "Solved GATE-style problem: the number of states in the minimal DFA for strings with a multiple of 3 a's and an even number of b's, checked in Delta."
sidebar:
  label: "1. a's mod 3, b's mod 2"
  order: 1
---

**Numerical answer type.** Let `L` be the set of strings over `{a, b}` in which the number of
`a`'s is divisible by 3 and the number of `b`'s is divisible by 2. How many states does the
minimal DFA for `L` have?

## Solution

A DFA for "number of `a`'s mod 3 is 0" needs 3 states (the count so far, mod 3), and one for
"number of `b`'s mod 2 is 0" needs 2. The **product construction** runs both at once: its states
are pairs (a-count mod 3, b-count mod 2), so it has `3 × 2 = 6` states, and it accepts only in
`(0, 0)`.

None of the 6 states can be merged. Two different pairs differ in the a-count or the b-count; adding
the right number of `a`'s (or `b`'s) leads one of them to `(0, 0)` and the other elsewhere.

**Answer: 6.**

## Check it in Delta

`intersect` builds the product DFA and `minimize` proves that nothing merges.

```delta
dfa A3 {
  start a0
  accept a0
  a0 -a-> a1;   a1 -a-> a2;   a2 -a-> a0
  a0 -b-> a0;   a1 -b-> a1;   a2 -b-> a2
}
dfa B2 {
  start b0
  accept b0
  b0 -b-> b1;   b1 -b-> b0
  b0 -a-> b0;   b1 -a-> b1
}
let M = minimize(intersect(A3, B2))
assert M.size == 6
check M accepts "", "aaa", "bb", "aaabb", "bbaaa"
```
