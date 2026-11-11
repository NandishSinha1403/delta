---
title: "DFA: binary numbers divisible by n"
description: "States are remainders; reading a bit doubles the number and adds the bit."
sidebar:
  order: 2
---

States are remainders; reading a bit doubles the number and adds the bit.

```delta
fn divisible_by(n) = dfa {
  alphabet 0, 1
  start  [0]
  accept [0]
  for r ∈ 0..n {
    [r] -0-> [(2*r) % n]
    [r] -1-> [(2*r + 1) % n]
  }
}
check divisible_by(5) accepts "0", "101", "1010", "1111"
check divisible_by(5) rejects "1", "11", "110"
```
