---
title: "Complement a language"
description: "Complete the DFA, then swap accepting and non-accepting states."
sidebar:
  order: 10
---

Complete the DFA, then swap accepting and non-accepting states.

```delta
let R = re"(a ∪ b)* a a (a ∪ b)*"
let C = complement(to_nfa(R))
check C ≡ ∁R
check C accepts "", "abab"
check C rejects "aa", "baab"
```
