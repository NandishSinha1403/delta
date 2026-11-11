---
title: "Write the subset construction yourself"
description: "The definition, as code: states are ε-closed sets."
sidebar:
  order: 26
---

The definition, as code: states are ε-closed sets.

```delta
fn subsets(N) = explore {
  alphabet N.alphabet
  start  N.eclose({ N.start })
  step   R on a -> N.eclose(⋃ { N.δ(q, a) | q ∈ R })
  accept R if R ∩ N.accept ≠ ∅
}
let N = to_nfa(re"a (a ∪ b)* b")
let D = subsets(N)
check D ≡ determinize(N)
assert D.size == determinize(N).size
```
