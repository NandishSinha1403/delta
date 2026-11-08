---
title: "Grammar: arithmetic expressions"
description: "Precedence comes from the layers E, T, F."
sidebar:
  order: 19
---

Precedence comes from the layers E, T, F.

```delta
cfg Expr {
  start <E>
  <E> -> <E> + <T> | <T>
  <T> -> <T> * <F> | <F>
  <F> -> ( <E> ) | a
}
check Expr accepts "a", "a+a*a", "(a+a)*a"
check Expr rejects "", "+a", "a**a"
assert Expr.terminals == {'(', ')', '*', '+', 'a'}
```
