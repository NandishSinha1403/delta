---
title: "NFA: words that end in 01, and its DFA"
description: "Guess where the suffix starts; the subset construction removes the guess."
sidebar:
  order: 4
---

Guess where the suffix starts; the subset construction removes the guess.

```delta
nfa N {
  alphabet 0, 1
  start q0
  accept q2
  q0 -[0, 1]-> q0;   q0 -0-> q1;   q1 -1-> q2
}
let D = determinize(N)
check D ≡ N
assert D.size == 3
show D
```
