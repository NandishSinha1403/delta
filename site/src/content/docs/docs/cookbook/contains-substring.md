---
title: "DFA: words that contain aba"
description: "Remember the longest suffix that is a prefix of aba."
sidebar:
  order: 3
---

Remember the longest suffix that is a prefix of aba.

```delta
dfa M {
  start s
  accept f
  s -a-> a;    s -b-> s
  a -a-> a;    a -b-> ab
  ab -a-> f;   ab -b-> s
  f -[a, b]-> f
}
check M ≡ re"(a ∪ b)* a b a (a ∪ b)*"
check M accepts "aba", "babab"
check M rejects "ab", "baab"
```
