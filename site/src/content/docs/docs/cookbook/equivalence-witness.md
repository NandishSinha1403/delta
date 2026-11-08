---
title: "Find the shortest word that tells two machines apart"
description: "A failing ≡ check reports the shortlex-least counterexample."
sidebar:
  order: 13
---

A failing ≡ check reports the shortlex-least counterexample.

```delta
dfa A {
  start q0
  accept q1
  q0 -a-> q1;   q1 -a-> q0
}
let R = re"a a*"
assert not equivalent(A, R)
assert shortest(R ∖ A) == "aa"
check A ⊆ R
```
