---
title: "Accept, reject, loops, unknown"
description: "A run's verdict says how it ended: loops means a configuration repeated."
sidebar:
  order: 22
---

A run's verdict says how it ended: loops means a configuration repeated.

```delta
budget { steps: 2_000 }
tm T {
  alphabet a, b, c
  start q0
  accept qa
  reject qr
  q0 -a → R-> qa
  q0 -b → S-> q0
  q0 -c → R-> q1
  q1 -⊔ → R-> q1
  q0 -⊔ → R-> qr
}
assert T.run("a").verdict == :accept
assert T.run("").verdict == :reject
assert T.run("b").verdict == :loops
assert T.run("c").verdict == :unknown
```
