---
title: "Mealy machine: running parity"
description: "Each step outputs the parity of the 1s read so far."
sidebar:
  order: 23
---

Each step outputs the parity of the 1s read so far.

```delta
mealy Parity {
  start even
  even -0 / 0-> even;   even -1 / 1-> odd
  odd -0 / 1-> odd;     odd -1 / 0-> even
}
assert Parity.output("1101") == "1001"
assert Parity.output("000") == "000"
```
