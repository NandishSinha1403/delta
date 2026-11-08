---
title: "What is a regular expression? Regex in automata theory"
description: "Regular expressions in the theory of computation: union, concatenation and star, how they describe regular languages, and how they become automata."
sidebar:
  label: "What is a regular expression?"
  order: 3
---

A **regular expression** describes a language with three operations on smaller expressions:

| Expression | Language |
|---|---|
| `a` | the one-symbol word `a` |
| `ε` | the empty word |
| `∅` | no words |
| `R ∪ S` | words of `R` or of `S` (union) |
| `R S` | a word of `R` followed by a word of `S` (concatenation) |
| `R*` | zero or more words of `R` joined together (star) |

`R+` is shorthand for `R R*`. So `(a ∪ b)* a b` is "any word, then `ab`": the words that end in
`ab`.

```delta embed
let R = re"(a ∪ b)* a b"
check R accepts "ab", "bab", "aab"
check R rejects "", "ba", "abb"
let N = to_nfa(R)
check N ≡ R
check re"(a ∪ b)* a b" ≡ re"(a* b)* a+ b"
```

**Kleene's theorem** says regular expressions and finite automata describe exactly the same
languages. Every expression becomes an NFA by [Thompson's or Glushkov's construction](/tools/regex-to-nfa/),
and every automaton becomes an expression by [state elimination](/learn/state-elimination/).

Theory regular expressions are smaller than the "regex" of programming languages: features like
back-references (`(a*)\1`) describe languages that are not regular.

## Frequently asked questions

### Are two regular expressions with different text the same language?
Often. Decide it exactly with `check R ≡ S` or the [equivalence tool](/tools/dfa-equivalence/).

### What is the difference between ε and ∅?
`ε` matches the empty word; `∅` matches nothing. So `R ∪ ∅ = R` while `R ε = R`, and `∅* = ε`.

### Why can't a regular expression match aⁿbⁿ?
Its language would be regular, and a finite automaton cannot count `a`'s without bound.
