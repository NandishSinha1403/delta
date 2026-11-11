---
title: "A worklist search in ordinary code"
description: "Find the reachable states with a while loop."
sidebar:
  order: 32
---

Find the reachable states with a while loop.

```delta
dfa M {
  alphabet a
  start [0]
  accept [0]
  [0] -a-> [1];   [1] -a-> [2];   [2] -a-> [0];   [3] -a-> [3]
}
var todo = [M.start]
var seen = { M.start }
while todo ≠ [] {
  let q = todo[0]
  todo = todo[1..]
  for a ∈ M.alphabet {
    let r = M.δ(q, a)
    if r ∉ seen {
      seen = seen ∪ { r }
      todo = todo ++ [r]
    }
  }
}
assert seen == {0, 1, 2}
```
