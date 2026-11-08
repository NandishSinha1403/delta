---
title: "Which strings does this grammar generate?"
description: "Solved GATE-style MCQ on context-free grammars: the grammar S → aSb | SS | ε generates the balanced strings. Reasoning and a runnable Delta check."
sidebar:
  label: "6. A grammar's language"
  order: 6
---

**Multiple choice.** Consider the grammar `S → a S b | S S | ε`. Which of these strings does it
generate?

- `abab`
- `aababb`
- `abba`

(A) only `abab`  (B) `abab` and `aababb`  (C) all three  (D) only `aababb`

## Solution

Read `a` as an opening and `b` as a closing bracket. `a S b` wraps a balanced string, `S S` puts
two side by side, and `ε` is the empty string: the grammar generates exactly the **balanced**
strings (every prefix has at least as many `a`'s as `b`'s, and the totals are equal).

- `abab` = `(ab)(ab)`: balanced. `S ⇒ S S ⇒ aSb S ⇒ ab S ⇒ ab aSb ⇒ abab`.
- `aababb` = `a(ab)(ab)b`: balanced.
- `abba`: after `abb` there are more `b`'s than `a`'s, so it is not balanced.

**Answer: (B).**

## Check it in Delta

A context-free grammar is a language too: `check G accepts …` parses each word.

```delta
cfg G {
  start <S>
  <S> -> a <S> b | <S> <S> | ε
}
check G accepts "", "ab", "aabb", "abab", "aababb"
check G rejects "ba", "aab", "abba"
```
