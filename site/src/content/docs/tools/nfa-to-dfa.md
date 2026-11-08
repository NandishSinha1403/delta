---
title: "NFA to DFA converter (subset construction)"
description: "Convert an NFA to an equivalent DFA online with the subset construction. Edit the NFA, see the DFA's diagram and its states as sets, step by step."
sidebar:
  label: NFA to DFA
  order: 1
---

Paste or edit a nondeterministic finite automaton below and Delta builds the equivalent DFA with
the **subset construction**, checks that the two accept the same language, and draws the result.

```delta embed
nfa N {
  alphabet 0, 1
  start q0
  accept q2
  q0 -[0, 1]-> q0
  q0 -1-> q1
  q1 -[0, 1]-> q2
}
let D = determinize(N)
check D ≡ N
assert D.size == 4
check D accepts "10", "011", "1110"
check D rejects "", "1", "01", "100"
show D
```

## How the subset construction works

An NFA can be in several states at once: after reading part of the input, the "current state" is
really a *set* of states. The subset construction makes that set the state. Each DFA state is a
set of NFA states, and the DFA's transition on a symbol `a` takes a set `S` to the set of every
NFA state reachable from some state in `S` by one `a`-edge, followed by any number of ε-edges
(the **ε-closure**).

1. The DFA's start state is the ε-closure of the NFA's start state.
2. Take a DFA state `S` not yet expanded. For each symbol `a`, compute
   `closure(move(S, a))`. If that set is new, add it as a DFA state.
3. Repeat until no new sets appear. Only sets reachable from the start are ever built, so the
   result is usually far smaller than all `2ⁿ` subsets.
4. A DFA state accepts when its set contains at least one accepting NFA state.

The DFA accepts a word exactly when *some* path of the NFA accepts it, because the set tracks
every path at once. In Delta, `determinize(N)` runs this construction; `check D ≡ N` then proves
the two machines equivalent rather than testing a few words.

## Worked example

The NFA above accepts binary words whose **second-to-last symbol is 1**. It guesses, at some `1`,
that exactly one symbol remains. The construction explores four sets:

| DFA state | on 0 | on 1 | accepting? |
|---|---|---|---|
| `{q0}` (start) | `{q0}` | `{q0, q1}` | no |
| `{q0, q1}` | `{q0, q2}` | `{q0, q1, q2}` | no |
| `{q0, q2}` | `{q0}` | `{q0, q1}` | yes |
| `{q0, q1, q2}` | `{q0, q2}` | `{q0, q1, q2}` | yes |

So the DFA has 4 states, which the program asserts with `D.size == 4`. The DFA remembers the last
two symbols: that is all it needs, and no DFA for this language has fewer states.

## Frequently asked questions

### Can every NFA be converted to a DFA?
Yes. For every NFA, the subset construction builds a DFA that accepts the same language, so NFAs
and DFAs recognize exactly the regular languages.

### How many states can the DFA have?
At most `2ⁿ` for an NFA with `n` states, and some languages need nearly all of them: for "the
k-th symbol from the end is 1" the smallest DFA has `2ᵏ` states. Delta only builds the reachable
sets, so typical results are much smaller.

### What happens to ε-transitions?
They are absorbed by the ε-closure. Every set the construction builds is already closed under
ε-moves, so the DFA itself has no ε-transitions.

### Is the DFA from the subset construction minimal?
Not always. Run [DFA minimization](/tools/dfa-minimization/) on the result (`minimize(D)`) to get
the smallest equivalent DFA.

### What does the empty set mean?
If no NFA state can read a symbol, the move is the empty set: a dead state that rejects
everything after it. Delta keeps it so that the DFA is total.

## Related tools

- [Regex to NFA](/tools/regex-to-nfa/): build the NFA from a regular expression first
- [DFA minimization](/tools/dfa-minimization/): shrink the DFA you just built
- [NFA simulator](/tools/nfa-simulator/): watch the set of active states change symbol by symbol
- [Learn: the subset construction](/learn/subset-construction/)
