---
title: "Regex to NFA, three ways"
description: "The textbook, Thompson and Glushkov constructions give different NFAs for one language."
sidebar:
  order: 5
---

The textbook, Thompson and Glushkov constructions give different NFAs for one language.

```delta
let R = re"(a ∪ b)* a b b"
let T = to_nfa(R, method: "textbook")
let H = to_nfa(R, method: "thompson")
let G = to_nfa(R, method: "glushkov")
check T ≡ R
check H ≡ R
check G ≡ R
assert G.size == 6                 // the start state plus one per symbol position
show T
```
