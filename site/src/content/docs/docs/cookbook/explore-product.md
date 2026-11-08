---
title: "Write the product construction yourself"
description: "explore builds only the reachable pairs, and the Pad can step through it."
sidebar:
  order: 25
---

explore builds only the reachable pairs, and the Pad can step through it.

```delta
dfa A {
  start e
  accept e
  e -a-> o;   e -b-> e
  o -a-> e;   o -b-> o
}
dfa B {
  start n
  accept y
  n -a-> n;   n -b-> y
  y -a-> n;   y -b-> y
}
fn product(A, B, keep) = explore {
  alphabet A.alphabet ∪ B.alphabet
  start  (A.start, B.start)
  step   (p, q) on a -> (A.δ(p, a), B.δ(q, a))
  accept (p, q) if keep(p ∈ A.accept, q ∈ B.accept)
}
check product(A, B, fn(x, y) = x and y) ≡ intersect(A, B)
check product(A, B, fn(x, y) = x or y) ≡ union(A, B)
```
