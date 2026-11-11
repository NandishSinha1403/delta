---
title: "Minimize a DFA and see which states merge"
description: "States with the same future merge; the result's states are the merged classes."
sidebar:
  order: 7
---

States with the same future merge; the result's states are the merged classes.

```delta
dfa M {
  start a
  accept c, e
  a -0-> b;   a -1-> c
  b -0-> a;   b -1-> d
  c -0-> e;   c -1-> f
  d -0-> e;   d -1-> f
  e -0-> e;   e -1-> f
  f -0-> f;   f -1-> f
}
let H = minimize(M, method: "hopcroft")
let T = minimize(M, method: "table")
check H ≡ M
assert H.size == T.size
show H
```
