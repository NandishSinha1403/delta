---
title: "Languages from predicates"
description: "A set-builder language is Decidable; compare it with a machine up to a bound."
sidebar:
  order: 28
---

A set-builder language is Decidable; compare it with a machine up to a bound.

```delta
lang Pal = { w ∈ {a,b}* | w == rev(w) }
lang EvenLen = { w ∈ {a,b}* | len(w) % 2 == 0 }
assert "abba" ∈ Pal and "ab" ∉ Pal
check EvenLen ≡ re"((a ∪ b)(a ∪ b))*" up to 8
assert (Pal ∩ EvenLen).class == Decidable
```
