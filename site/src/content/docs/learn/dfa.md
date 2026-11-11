---
title: "What is a DFA? Deterministic finite automata explained"
description: "A deterministic finite automaton (DFA) explained with a runnable example: states, transitions, acceptance, and what DFAs can and cannot recognize."
sidebar:
  label: "What is a DFA?"
  order: 1
---

A **deterministic finite automaton** (DFA) reads a word one symbol at a time. It is always in
exactly one of finitely many states, and for every state and symbol there is exactly one next
state. When the word ends, the DFA accepts if it is in an accepting state.

Formally a DFA is a 5-tuple `(Q, Σ, δ, q₀, F)`: a finite set of states `Q`, an alphabet `Σ`, a
transition function `δ : Q × Σ → Q`, a start state `q₀` and a set of accepting states `F ⊆ Q`.
In Delta you write the same five parts directly:

```delta embed
dfa EndsAB {
  alphabet a, b
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q2
  q2 -a-> q1;   q2 -b-> q0
}
check EndsAB accepts "ab", "aab", "abab"
check EndsAB rejects "", "a", "ba", "abb"
check EndsAB ≡ re"(a ∪ b)* a b"
run EndsAB on "abab"
```

Each state stands for what the machine remembers: `q1` means "the last symbol was `a`", `q2`
"the last two were `ab`", and `q0` everything else. Designing a DFA is choosing those meanings,
then checking that each transition keeps them true.

A DFA has only finite memory, so it cannot count without bound. The languages DFAs recognize are
the **regular languages**, the same ones that NFAs and regular expressions describe. A language
like `{ aⁿbⁿ }` is not regular: it needs the stack of a [pushdown automaton](/learn/pda/).

## Frequently asked questions

### What is the difference between a DFA and an NFA?
A DFA has exactly one move per state and symbol. An [NFA](/learn/nfa/) may have several or none,
and ε-moves. Both recognize the same languages.

### Does every state need a transition for every symbol?
In a DFA, yes: δ is total, and Delta reports error E306 when a transition is missing. Add a dead
state for the symbols that should reject, or write an [NFA](/learn/nfa/), which may leave
transitions out.

### How do I find the smallest DFA?
Minimize it: see the [DFA minimization](/tools/dfa-minimization/) tool.
