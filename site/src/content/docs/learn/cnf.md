---
title: "Chomsky normal form (CNF) explained"
description: "Chomsky normal form for context-free grammars: the rule shapes, the conversion steps, why CNF matters for CYK parsing, and a worked example."
sidebar:
  label: "Chomsky normal form"
  order: 9
---

A context-free grammar is in **Chomsky normal form** (CNF) when every rule has one of two shapes:

- `A → B C`, two variables (neither of them the start variable), or
- `A → a`, one terminal,

plus, if the empty word is in the language, the single rule `S → ε` for the start variable.

Every context-free grammar can be converted to CNF:

1. Add a new start variable `S₀ → S`, so the start never appears on the right.
2. Remove **ε-rules** `A → ε`, adding copies of rules with `A` left out.
3. Remove **unit rules** `A → B`, copying `B`'s rules to `A`.
4. Replace terminals inside long rules by new variables (`A → a B` becomes `A → X B`, `X → a`).
5. Split rules longer than two into chains of two (`A → B C D` becomes `A → B Y`, `Y → C D`).

For `S → a S b | a b` this gives the grammar `C` below, which a bounded check compares with the
original:

```delta
cfg G {
  start <S>
  <S> -> a <S> b | a b
}
cfg C {
  start <S>
  <S> -> <A> <T> | <A> <B>
  <T> -> <S> <B>
  <A> -> a
  <B> -> b
}
check C ≡ G up to 8
check C accepts "ab", "aabb", "aaabbb"
check C rejects "", "ba", "aab"
```

**Why it matters.** In CNF every derivation of a word of length `n` takes exactly `2n - 1` steps,
and every parse tree is binary. That is what the **CYK algorithm** relies on to decide membership
in `O(n³)` time with a table of substrings. CNF is also the starting point of the pumping lemma for
context-free languages.

Delta's own `cnf(G)` and the CYK parser arrive with Delta 2.0; until then, write the grammar in CNF
yourself and compare it as above.

## Frequently asked questions

### Can every context-free grammar be put in CNF?
Yes, every grammar for a language without ε, and with the extra rule `S → ε` otherwise.

### Does CNF change the language?
No. Each step keeps the language the same; only the grammar's shape changes.

### How much bigger does the grammar get?
Polynomially: the usual conversion is at most quadratic in the size of the grammar if ε-rules are
removed carefully.
