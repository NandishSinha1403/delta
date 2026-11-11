---
title: "Union and intersection by the product construction"
description: "Run both machines at once on pairs of states."
sidebar:
  order: 9
---

Run both machines at once on pairs of states.

```delta
dfa EvenA {
  start e
  accept e
  e -a-> o;   e -b-> e
  o -a-> e;   o -b-> o
}
dfa HasB {
  start n
  accept y
  n -a-> n;   n -b-> y
  y -[a, b]-> y
}
let I = intersect(EvenA, HasB)
let U = union(EvenA, HasB)
check I accepts "b", "aab"
check I rejects "", "ab"
check U accepts "", "ab"
check U rejects "a"
show I
```
