---
title: "List, count and find the shortest words"
description: "words, count and shortest work on any regular language."
sidebar:
  order: 14
---

words, count and shortest work on any regular language.

```delta
let L = re"(a ∪ b)* b"
assert shortest(L) == "b"
assert words(L, upto: 2) == {"b", "ab", "bb"}
assert count(L, length: 5) == 16
assert is_finite(re"a ∪ b b") and not is_finite(L)
```
