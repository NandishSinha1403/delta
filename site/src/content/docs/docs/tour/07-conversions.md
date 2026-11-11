---
title: 7. Conversions
description: Regex ↔ automata, determinize, minimize, and the closure constructions.
---

Conversions are functions named by what they do. Each takes an optional `method:` that names the
algorithm, and remembers its steps.

```delta
let R = re"(0 ∪ 1)* 0 1"
let N = to_nfa(R)                          // method: "textbook" (default), "thompson", "glushkov"
let D = determinize(N)                     // states are sets of N's states
let M = minimize(D)                        // method: "hopcroft", "moore", "brzozowski", "table"
check M ≡ R
assert M.size == 3
let R2 = to_regex(M, method: "state-elimination")
check R2 ≡ R
assert len(steps(D)) > 0                   // the construction's trace, for the Steps view
```

The closure constructions:

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
check intersect(A, B) ≡ A ∩ B
check union(A, B) accepts "", "ab"
check difference(A, B) rejects "b"
check concat(A, B) ≡ A ++ B
check star(B) accepts "", "bb"
check complement(A) ≡ ∁A
```
