---
title: "Two stacks: aⁿbⁿcⁿ"
description: "A model with two stacks is as strong as a Turing machine."
sidebar:
  order: 24
---

A model with two stacks is as strong as a Turing machine.

```delta
model pda2 { input one-way; epsilon; storage stack, stack; accept by final }
pda2 P {
  start s
  accept r
  s -push Z; push2 Z-> p
  p -read a; push A-> p
  p -ε-> q
  q -read b; pop A; push2 B-> q
  q -pop Z-> c
  c -read c; pop2 B-> c
  c -pop2 Z-> r
}
check P accepts "", "abc", "aabbcc"
check P rejects "aabbc", "abcc"
assert P.class == Recognizable
```
