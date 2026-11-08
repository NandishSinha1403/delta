---
title: "Why the subset construction can blow up"
description: "The NFA for 'the 4th symbol from the end is 1' has 5 states; its DFA has 16."
sidebar:
  order: 12
---

The NFA for 'the 4th symbol from the end is 1' has 5 states; its DFA has 16.

```delta
nfa N {
  alphabet 0, 1
  start q0
  accept q4
  q0 -[0, 1]-> q0;   q0 -1-> q1
  q1 -[0, 1]-> q2;   q2 -[0, 1]-> q3;   q3 -[0, 1]-> q4
}
let D = determinize(N)
assert N.size == 5
assert D.size == 16
assert minimize(D).size == 16
```
