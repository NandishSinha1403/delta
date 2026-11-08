---
title: 11. Constructions as code
description: explore builds a machine from successors; fixpoint repeats until nothing changes.
---

`explore` builds a machine from a start state and a successor rule; only reachable states are
created, and the construction is steppable in the Pad.

```delta
fn subsets(N) = explore {
  alphabet N.alphabet
  start  N.eclose({ N.start })
  step   R on a -> N.eclose(⋃ { N.δ(q, a) | q ∈ R })
  accept R if R ∩ N.accept ≠ ∅
}
let N = to_nfa(re"(a ∪ b)* a b")
check subsets(N) ≡ N
check subsets(N) ≡ determinize(N)
```

`fixpoint X from init { … }` repeats until the value stops changing:

```delta
fn reachable(M) = fixpoint X from { M.start } { X ∪ { M.δ(q, a) | q ∈ X, a ∈ M.alphabet } }
dfa M {
  alphabet a
  start [0]
  accept [1]
  [0] -a-> [1];   [1] -a-> [0];   [2] -a-> [0]
}
assert reachable(M) == {0, 1}
```
