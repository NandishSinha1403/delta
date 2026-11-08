---
title: "Nullable variables with fixpoint"
description: "A variable is nullable if some rule's right side is all nullable."
sidebar:
  order: 20
---

A variable is nullable if some rule's right side is all nullable.

```delta
fn nullable(G) = fixpoint X from ∅ { X ∪ { A | (A, rhs) ∈ G.rules, set(rhs) ⊆ X } }
cfg G {
  S -> A B | a
  A -> ε
  B -> A A | b
}
assert nullable(G) == {'A', 'B', 'S'}
```
