---
title: 12. Grammars, Turing machines and transducers
description: cfg, tm verdicts, and Mealy machines.
---

Grammars are languages too; membership uses the Earley algorithm.

```delta
cfg Expr {
  start <E>
  <E> -> <E> + <T> | <T>
  <T> -> <T> * <F> | <F>
  <F> -> ( <E> ) | a
}
check Expr accepts "a+a*a", "(a+a)*a"
check Expr rejects "a+", "(a"
```

A Turing machine's labels write and move: `0 → x, R`, or `x → R` to move only.

```delta
tm Zeros {
  alphabet 0, 1
  start q0
  accept qa
  q0 -0 → x, R-> q0
  q0 -⊔ → R-> qa
}
check Zeros accepts "", "000"
check Zeros rejects "01"
```

Transducers emit output:

```delta
mealy Flip {
  start s
  s -0 / 1-> s;   s -1 / 0-> s
}
assert Flip.output("0110") == "1001"
```
