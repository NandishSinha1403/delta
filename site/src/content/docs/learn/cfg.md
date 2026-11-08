---
title: "What is a context-free grammar (CFG)?"
description: "Context-free grammars explained: variables, rules, derivations and parse trees, with a runnable grammar and the languages CFGs can describe."
sidebar:
  label: "What is a CFG?"
  order: 4
---

A **context-free grammar** (CFG) generates words by rewriting. It has **variables** (written
`<S>` in Delta), **terminals** (the symbols of the words), a **start variable**, and **rules**
`A → w` that replace a variable with a string of variables and terminals. A word is in the
language when the start variable can be rewritten into it.

```delta embed
cfg Expr {
  start <E>
  <E> -> <E> + <T> | <T>
  <T> -> <T> * <F> | <F>
  <F> -> ( <E> ) | a
}
check Expr accepts "a", "a+a", "a*a", "(a+a)*a"
check Expr rejects "", "+", "a+", "(a"
```

A **derivation** of `a+a` is `E ⇒ E + T ⇒ T + T ⇒ F + T ⇒ a + T ⇒ a + F ⇒ a + a`. The same steps
drawn as a tree, with `E` at the root, form the **parse tree**. A grammar is **ambiguous** if some
word has two parse trees; this one is not, and the layering `E → T → F` is what gives `*`
priority over `+`.

CFGs describe the **context-free languages**, for example balanced brackets and `{ aⁿbⁿ }`,
which no finite automaton recognizes. They are exactly the languages of
[pushdown automata](/learn/pda/). Some languages, like `{ aⁿbⁿcⁿ }`, are not context-free.

## Frequently asked questions

### Why "context-free"?
A rule `A → w` applies to `A` wherever it appears, whatever surrounds it.

### Is every regular language context-free?
Yes. A DFA becomes a grammar with one variable per state and a rule `A → a B` per transition.

### Can I check whether two grammars are equivalent?
Not in general: it is undecidable. Delta checks it for words up to a length you give, with
`check G ≡ H up to n`.
