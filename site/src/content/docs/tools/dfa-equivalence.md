---
title: "DFA equivalence checker with counterexample"
description: "Check online whether two DFAs, NFAs or regular expressions accept the same language, and get the shortest word that tells them apart."
sidebar:
  label: DFA equivalence
  order: 5
---

Are two machines, or a machine and a regular expression, the same language? Delta decides it
exactly, for every word, and when they differ it finds the shortest word that proves it.

```delta embed
dfa A {
  start e
  accept e
  e -a-> o;   e -b-> e
  o -a-> e;   o -b-> o
}
let B = re"b* (a b* a b*)*"
check A ≡ B
let C = re"(a a)*"
assert not equivalent(A, C)
assert shortest((A ∖ C) ∪ (C ∖ A)) == "b"
```

## How equivalence is decided

Two regular languages `A` and `B` are equal exactly when their **symmetric difference**,
`(A ∖ B) ∪ (B ∖ A)`, is empty. That gives an algorithm:

1. Build DFAs for both sides (regular expressions and NFAs are converted first).
2. Build the **product automaton**: its states are pairs `(p, q)`, one state from each DFA, and it
   reads each symbol in both at once.
3. A pair is "bad" when exactly one of `p` and `q` accepts. Search the product breadth first from
   the start pair. If no bad pair is reachable, the languages are equal.
4. If a bad pair is reachable, the path to it spells a word in one language and not in the
   other. Breadth-first search finds a shortest such word.

This takes time proportional to the size of the product, `|A| · |B| · |Σ|`. Another way is to
minimize both DFAs and compare them: the minimal DFA of a language is unique, so equal languages
give the same minimal machine.

In Delta, `check A ≡ B` runs this decision and reports a counterexample when it fails;
`equivalent(A, B)` returns the answer as a value, and `shortest(L)` returns the shortlex-least
word of a language.

## Worked example

`A` accepts words with an even number of `a`'s. `B = b* (a b* a b*)*` describes the same
words: any number of `b`'s, then pairs of `a`'s with `b`'s around them. The product of their
DFAs never reaches a bad pair, so `check A ≡ B` passes.

`C = (a a)*` also has an even number of `a`'s, but it has no `b`'s. The shortest word in the
symmetric difference is `b`: `A` accepts it and `C` does not.

## Frequently asked questions

### Is DFA equivalence decidable?
Yes, for DFAs, NFAs and regular expressions. For context-free grammars it is undecidable, and
Delta reports that at compile time (error E701) instead of running forever.

### What is a counterexample here?
A word accepted by one side and rejected by the other. Delta reports the shortest one, and among
the shortest the first in alphabetical order.

### Can I compare an NFA with a regular expression?
Yes. Both are converted to DFAs before the product is built, so any mix of models works.

### How do I check inclusion instead of equality?
Use `check A ⊆ B`. It is empty-checking `A ∖ B` instead of the symmetric difference.

## Related tools

- [DFA minimization](/tools/dfa-minimization/): equal languages have the same minimal DFA
- [Regex to NFA](/tools/regex-to-nfa/) and [DFA to regex](/tools/dfa-to-regex/): convert, then compare
- [NFA to DFA](/tools/nfa-to-dfa/): the first step of an NFA comparison
- [Practice](/practice/): equivalence questions with solutions
