---
title: "DFA to regex by state elimination"
description: "Remove states one by one, relabelling the arrows around each with regexes."
sidebar:
  order: 8
---

Remove states one by one, relabelling the arrows around each with regexes.

```delta
dfa M {
  start q0
  accept q1
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q0
}
let R = to_regex(M, method: "state-elimination")
let A = to_regex(M, method: "arden")
check R ≡ M
check A ≡ M
print R
```
