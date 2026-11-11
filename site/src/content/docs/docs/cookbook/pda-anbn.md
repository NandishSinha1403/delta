---
title: "PDA: aⁿbⁿ"
description: "Push for each a, pop for each b, and check the bottom marker at the end."
sidebar:
  order: 16
---

Push for each a, pop for each b, and check the bottom marker at the end.

```delta
pda P {
  start q0
  accept q3
  q0 -ε, ε → Z-> q1
  q1 -a, ε → A-> q1
  q1 -ε, Z → ε-> q3
  q1 -b, A → ε-> q2
  q2 -b, A → ε-> q2
  q2 -ε, Z → ε-> q3
}
check P accepts "", "ab", "aaabbb"
check P rejects "a", "abab", "aab"
run P on "aabb"
```
