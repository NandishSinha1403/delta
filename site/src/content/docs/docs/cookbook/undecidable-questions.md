---
title: "What happens with an undecidable question"
description: "Equivalence of context-free languages is undecidable: the error offers a bounded check."
sidebar:
  order: 30
---

Equivalence of context-free languages is undecidable: the error offers a bounded check.

```delta
lang A = cfg { S -> a S b | ε }
lang B = cfg { S -> a B b | ε; B -> a B b | ε }
check A ≡ B up to 10
```
