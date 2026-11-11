---
title: "ε-closure with fixpoint"
description: "Add every state one ε-move away until nothing changes."
sidebar:
  order: 27
---

Add every state one ε-move away until nothing changes.

```delta
fn eclose(N, S) = fixpoint X from S { X ∪ { r | q ∈ X, r ∈ N.δ(q, ε) } }
nfa N {
  alphabet a
  start [0]
  accept [2]
  [0] -ε-> [1];   [1] -ε-> [2];   [2] -a-> [0]
}
assert eclose(N, {0}) == {0, 1, 2}
assert eclose(N, {0}) == N.eclose({0})
```
