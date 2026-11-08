---
title: "Turing machine: 0ⁿ1ⁿ"
description: "Cross off a 0, find a matching 1, return, and repeat."
sidebar:
  order: 21
---

Cross off a 0, find a matching 1, return, and repeat.

```delta
tm M {
  alphabet 0, 1
  start q0
  accept qa
  q0 -0 → x, R-> q1;   q0 -y → R-> q3;   q0 -⊔ → R-> qa
  q1 -0 → R-> q1;      q1 -y → R-> q1;   q1 -1 → y, L-> q2
  q2 -0 → L-> q2;      q2 -y → L-> q2;   q2 -x → R-> q0
  q3 -y → R-> q3;      q3 -⊔ → R-> qa
}
check M accepts "", "01", "0011", "000111"
check M rejects "0", "1", "10", "001", "011"
run M on "0011"
```
