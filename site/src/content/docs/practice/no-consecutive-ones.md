---
title: "Counting strings with no two consecutive 1s"
description: "Solved GATE-style problem: how many binary strings of length 5 have no two consecutive 1s? A recurrence, the DFA behind it, and a Delta check."
sidebar:
  label: "5. No two consecutive 1s"
  order: 5
---

**Numerical answer type.** How many binary strings of length 5 contain no two consecutive `1`'s?

## Solution

Let `f(n)` be the number of such strings of length `n`. A valid string either ends in `0`
(anything valid of length `n - 1`, then `0`) or ends in `01` (anything valid of length `n - 2`,
then `01`). So `f(n) = f(n - 1) + f(n - 2)`, with `f(1) = 2` and `f(2) = 3`.

`f(3) = 5`, `f(4) = 8`, `f(5) = 13`.

The recurrence is the DFA for the language in disguise: one state for "last symbol was 0 (or
nothing yet)", one for "last symbol was 1", and a dead state after `11`.

**Answer: 13.**

## Check it in Delta

`count(L, length: n)` counts the words of length `n` in a regular language. Here `L` is the
complement of "contains `11`".

```delta
let Has11 = re"(0 ∪ 1)* 1 1 (0 ∪ 1)*"
let L = complement(to_nfa(Has11))
assert count(L, length: 5) == 13
assert count(L, length: 1) == 2
assert count(L, length: 2) == 3
```
