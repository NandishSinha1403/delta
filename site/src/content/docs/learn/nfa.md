---
title: "What is an NFA? Nondeterministic finite automata explained"
description: "Nondeterministic finite automata (NFAs) explained: several choices at once, ε-transitions, acceptance, and why NFAs are no more powerful than DFAs."
sidebar:
  label: "What is an NFA?"
  order: 2
---

A **nondeterministic finite automaton** (NFA) is like a DFA, but a state may have several
transitions on the same symbol, none at all, or **ε-transitions** that move without reading
input. An NFA accepts a word if **some** sequence of choices reads the whole word and ends in an
accepting state.

Formally the transition function returns a set: `δ : Q × (Σ ∪ {ε}) → P(Q)`. NFAs are often much
smaller and easier to design than DFAs, because they can guess. This one guesses where the
suffix `01` starts:

```delta embed
nfa N {
  alphabet 0, 1
  start q0
  accept q2
  q0 -[0, 1]-> q0
  q0 -0-> q1
  q1 -1-> q2
}
check N accepts "01", "001", "1101"
check N rejects "", "10", "011"
run N on "1101"
```

To run an NFA you do not need to guess: keep the **set** of states it could be in, and update the
whole set on each symbol (see the [NFA simulator](/tools/nfa-simulator/)). Making that set a
state gives a DFA, which is the [subset construction](/learn/subset-construction/). So NFAs
recognize exactly the regular languages, the same as DFAs, though the DFA may need up to `2ⁿ`
states.

## Frequently asked questions

### Is an NFA more powerful than a DFA?
No. Every NFA can be converted to a DFA for the same language. NFAs are only more concise.

### What is an ε-transition?
A move that reads no input. Written `-ε->` in Delta. NFAs with ε-moves are no more powerful
either: the ε-closure removes them.

### When does an NFA reject?
When every possible run either gets stuck or ends in a non-accepting state.
