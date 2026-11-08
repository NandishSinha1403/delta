---
title: Delta docs
description: Delta is a programming language whose values are the objects of computation theory.
---

Delta is a small, safe language whose values are **machines, languages, words, grammars and
computations**. You write a DFA the way you'd draw it, a language the way you'd write it in
math, and a construction the way it's defined, then run them.

```delta
dfa EndsAB {
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q2
  q2 -a-> q1;   q2 -b-> q0
}
lang L = { w ∈ {a,b}* | w ends with "ab" }
check EndsAB ≡ L up to 10
check EndsAB ≡ re"(a ∪ b)* a b"
```

Every example in these docs runs: press **Open in the Pad** under it.

- **[Tour](/docs/tour/01-first-program/)**: twelve short pages, from values to Turing machines.
- **[Library](/docs/library/)**: every built-in function and its methods.
- **[Models](/docs/models/)**: the prelude, and how to define your own model.
- **[Languages and classes](/docs/languages/)**: the class lattice, closure and decision tables.
- **[Cookbook](/docs/cookbook/)**: recipes for common exercises.
- **[Notation](/docs/notation/)**, **[Errors](/docs/errors/)** and **[AI](/docs/ai/)**.
