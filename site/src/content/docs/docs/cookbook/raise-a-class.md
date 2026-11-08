---
title: "Raise a class with evidence"
description: "Claim a class and give a machine; Delta checks the claim up to a bound."
sidebar:
  order: 29
---

Claim a class and give a machine; Delta checks the claim up to a bound.

```delta
lang E = { w ∈ {0,1}* | count(w,'1') % 3 == 0 } as Regular by re"(0* 1 0* 1 0* 1)* 0*"
assert E.class == Regular and E.assumed
check E ≡ re"(0* 1 0* 1 0* 1)* 0*"
check E accepts "", "111", "10101"
```
