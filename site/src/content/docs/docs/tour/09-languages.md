---
title: 9. Languages and classes
description: lang values, their sources and their classes.
---

A `lang` is a set of words, possibly infinite. Its **class** is what Delta can guarantee about it:
Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any.

```delta
lang F = { "", "01", "0011" }                              // Finite
lang R = re"(0 ∪ 1)* 0 1"                                  // Regular
lang C = cfg { S -> 0 S 1 | ε }                            // CF
lang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') }     // Decidable
assert F.class == Finite and R.class == Regular
assert C.class == CF and P.class == Decidable
assert "0011" ∈ C and "0101" ∈ P and "10" ∉ C
```

Operations compute the class from the closure table:

```delta
lang R = re"(0 ∪ 1)* 0 1"
lang C = cfg { S -> 0 S 1 | ε }
lang C2 = cfg { S -> 1 S 0 | ε }
assert (C ∩ R).class == CF             // CF ∩ Regular stays CF
assert (C ∩ C2).class == CS            // CF ∩ CF is only known to be CS
assert (∁C).class == CS
assert (R ∪ C).class == CF             // mixed: the larger class
```
