---
title: "Is a language finite?"
description: "A regular language is infinite iff its useful states have a cycle."
sidebar:
  order: 15
---

A regular language is infinite iff its useful states have a cycle.

```delta
dfa M {
  alphabet a, b
  start s
  accept t
  s -a-> t;   t -b-> u;   u -[a, b]-> u;   s -b-> u;   t -a-> u
}
assert M.lang.is_finite()
assert not re"a*".is_finite()
assert not M.lang.is_empty()
```
