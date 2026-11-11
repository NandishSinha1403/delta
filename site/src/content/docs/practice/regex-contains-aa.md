---
title: "Equivalent regular expressions: strings containing aa"
description: "Solved GATE-style MCQ: are (a+b)*aa(a+b)* and (b+ab)*aa(a+b)* equivalent? Reasoning and a runnable Delta check."
sidebar:
  label: "3. Strings containing aa"
  order: 3
---

**Multiple choice.** Which of these regular expressions denote the set of strings over `{a, b}`
that contain `aa` as a substring?

- P: `(a ∪ b)* a a (a ∪ b)*`
- Q: `(b ∪ a b)* a a (a ∪ b)*`

(A) only P  (B) only Q  (C) both P and Q  (D) neither

## Solution

P says it directly: anything, then `aa`, then anything.

For Q, look at the **first** occurrence of `aa` in a word. The part before it contains no `aa`
and does not end in `a` (otherwise the `aa` would start one position earlier). The words with no
`aa` that do not end in `a` are exactly `(b ∪ a b)*`: every `a` in them is followed by a `b`. So
every word containing `aa` matches Q, and every word Q matches contains `aa`.

**Answer: (C).**

## Check it in Delta

`check P ≡ Q` decides equality for every word, not just the samples.

```delta
let P = re"(a ∪ b)* a a (a ∪ b)*"
let Q = re"(b ∪ a b)* a a (a ∪ b)*"
check P ≡ Q
check Q accepts "aa", "baa", "abaa", "aab"
check Q rejects "", "a", "ab", "abab"
```
