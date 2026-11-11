---
title: "NFA to DFA, step by step"
description: "determinize keeps its trace: open the Steps view to replay it."
sidebar:
  order: 6
---

determinize keeps its trace: open the Steps view to replay it.

```delta
nfa N {
  alphabet a, b
  start q1
  accept q3
  q1 -[a, b]-> q1;   q1 -a-> q2;   q2 -[a, b]-> q3
}
let D = determinize(N)
check D ≡ N
assert D.size == 4
show D
```
