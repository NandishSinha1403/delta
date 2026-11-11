---
title: "The subset construction: NFA to DFA explained"
description: "How the subset (powerset) construction turns an NFA into an equivalent DFA, step by step, with ε-closures, an example and its worst case."
sidebar:
  label: "Subset construction"
  order: 7
---

The **subset construction** (or powerset construction) converts an NFA into a DFA that accepts the
same language. Its idea: after reading part of the input, an NFA can be in a *set* of states, so
let each DFA state be one such set.

1. Start with the ε-closure of the NFA's start state.
2. For each new set `S` and symbol `a`, the next set is the ε-closure of every state reachable from
   `S` by an `a`-edge.
3. Stop when no new sets appear. A set is accepting if it contains an accepting NFA state.

```delta embed
nfa N {
  alphabet 0, 1
  start q0
  accept q2
  q0 -[0, 1]-> q0;   q0 -0-> q1;   q1 -1-> q2
}
let D = determinize(N)
check D ≡ N
assert D.size == 3
show D
```

This NFA (words ending in `01`) gives the sets `{q0}`, `{q0, q1}` and `{q0, q2}`. The DFA tracks
"no progress", "just read 0" and "just read 01".

**Why it is correct.** By induction on the length of the input, the DFA's state after a word `w`
is exactly the set of NFA states reachable by reading `w`. The DFA accepts when that set contains
an accepting state, which is exactly when some run of the NFA accepts.

**Cost.** There are at most `2ⁿ` sets, and some languages need them all: "the k-th symbol from the
end is 1" has a `(k + 1)`-state NFA and no DFA with fewer than `2ᵏ` states. Building only the
reachable sets keeps most practical cases small. The [NFA to DFA tool](/tools/nfa-to-dfa/) shows
the table of sets for your own NFA.

## Frequently asked questions

### Why only the reachable subsets?
Unreachable sets can never be entered, so they do not affect the language.

### Is the result minimal?
Not always; minimize it afterwards.

### What is the empty set state?
The dead state: no NFA state survives, so nothing after it is accepted.
