---
title: "Reverse a language"
description: "Reverse every arrow; the old accepting states become the start."
sidebar:
  order: 11
---

Reverse every arrow; the old accepting states become the start.

```delta
let R = re"a b*"
let V = reverse(to_nfa(R))
check V ≡ re"b* a"
check V ≡ rev(R)
```
