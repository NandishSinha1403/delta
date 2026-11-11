---
title: "PDA: even-length palindromes"
description: "Push the first half, guess the middle with ε, then pop the second half."
sidebar:
  order: 17
---

Push the first half, guess the middle with ε, then pop the second half.

```delta
pda P {
  start p
  accept f
  p -ε, ε → Z-> q
  q -0, ε → 0-> q;   q -1, ε → 1-> q
  q -ε-> r
  r -0, 0 → ε-> r;   r -1, 1 → ε-> r
  r -ε, Z → ε-> f
}
check P accepts "", "0110", "1001"
check P rejects "01", "010", "0111"
```
