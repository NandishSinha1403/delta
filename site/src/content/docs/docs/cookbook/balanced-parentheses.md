---
title: "PDA and grammar: balanced parentheses"
description: "The same language as a pushdown automaton and as a grammar."
sidebar:
  order: 18
---

The same language as a pushdown automaton and as a grammar.

```delta
pda P (accept: by empty-stack) {
  start q
  q -(, ε → X-> q
  q -), X → ε-> q
}
cfg G { S -> ( S ) S | ε }
check P accepts "", "()", "(())()"
check P rejects "(", ")(", "(()"
check P ≡ G up to 8
```
