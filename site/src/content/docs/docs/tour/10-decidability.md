---
title: 10. Decidability
description: Only decidable questions are allowed; bounded checks work for any class.
---

Asking an undecidable question is an error **before the program runs**, with a quick fix:

```delta error
lang C = cfg { S -> 0 S 1 | ε }
lang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') }
check C ≡ P
```

A bounded check compares the two languages on every word up to a length, for any class, and
is labeled as bounded in the results:

```delta
lang C = cfg { S -> 0 S 1 | ε }
lang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') }
check C ⊆ P up to 8
```

You can raise a language's class with evidence. Delta checks the claim up to a bound and marks
the language *assumed* wherever it is used:

```delta
lang E = { w ∈ {0,1}* | count(w,'0') % 2 == 0 } as Regular by re"(1* 0 1* 0)* 1*"
assert E.assumed
check E ≡ re"(1* 0 1* 0)* 1*"
```
