---
title: "Strict mode for assignments"
description: "use strict requires every field, so a missing alphabet is an error."
sidebar:
  order: 31
---

use strict requires every field, so a missing alphabet is an error.

```delta
use strict
dfa M {
  states q0, q1
  alphabet 0, 1
  start q0
  accept q1
  q0 -0-> q0;   q0 -1-> q1
  q1 -0-> q0;   q1 -1-> q1
}
check M accepts "1", "011"
check M rejects "", "10"
```
