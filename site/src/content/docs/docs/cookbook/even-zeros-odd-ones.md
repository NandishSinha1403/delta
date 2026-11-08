---
title: "DFA: an even number of 0s and an odd number of 1s"
description: "Track both parities: four states, one per combination."
sidebar:
  order: 1
---

Track both parities: four states, one per combination.

```delta
dfa M {
  start ee
  accept eo
  ee -0-> oe;   ee -1-> eo
  oe -0-> ee;   oe -1-> oo
  eo -0-> oo;   eo -1-> ee
  oo -0-> eo;   oo -1-> oe
}
check M accepts "1", "001", "10101"
check M rejects "", "0", "11", "01"
lang L = { w ∈ {0,1}* | count(w,'0') % 2 == 0 and count(w,'1') % 2 == 1 }
check M ≡ L up to 8
```
