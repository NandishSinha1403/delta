---
title: "NFA simulator: step through a run online"
description: "Simulate an NFA on any input online. See the set of active states after every symbol, ε-closures included, and whether the word is accepted."
sidebar:
  label: NFA simulator
  order: 6
---

Run a nondeterministic finite automaton on a word and watch every branch at once. The Run view
of the Pad below steps through the input and highlights the active states on the diagram.

```delta embed
nfa N {
  alphabet 0, 1
  start q0
  accept q2
  q0 -[0, 1]-> q0
  q0 -1-> q1
  q1 -[0, 1]-> q2
}
run N on "0110"
check N accepts "0110", "11"
check N rejects "0101", "1"
```

## How an NFA runs

A DFA has exactly one next state for each symbol. An NFA may have several, or none, and may also
move on ε without reading anything. You can think of it as trying every choice in parallel: a
word is accepted when **at least one** sequence of choices reads the whole word and ends in an
accepting state.

A simulator does not need to explore the choices one path at a time. It keeps the **set** of
states the NFA could be in:

1. Start with the ε-closure of the start state: the start state and everything reachable from it
   by ε-moves.
2. For each input symbol, move every state in the set along its edges for that symbol, then take
   the ε-closure again.
3. If the set becomes empty, every branch has died and the word is rejected.
4. After the last symbol, accept if the set contains an accepting state.

Each step costs at most `O(n²)` for an `n`-state NFA, so a word of length `m` takes `O(m · n²)`
time, with no exponential blow-up. This is the same idea as the subset construction, computed
only for the sets the input actually reaches.

The same set view explains why NFAs are convenient to design: you describe one lucky path (here,
"guess that this `1` is second to last") and the simulator, or the subset construction, takes care
of every other path for you.

## Worked example

The NFA accepts words whose second-to-last symbol is `1`. On input `0110`:

| step | read | active states |
|---|---|---|
| 0 | | `{q0}` |
| 1 | `0` | `{q0}` |
| 2 | `1` | `{q0, q1}` |
| 3 | `1` | `{q0, q1, q2}` |
| 4 | `0` | `{q0, q2}` |

The last set contains `q2`, so the word is accepted. On `0101` the run ends in `{q0, q1}`,
which has no accepting state: rejected.

## Frequently asked questions

### What does it mean for an NFA to accept a word?
At least one way of making the choices reads the whole word and ends in an accepting state. Other
branches may die or end in rejecting states; they do not matter.

### How are ε-transitions handled?
After the start and after every symbol, the simulator adds every state reachable by ε-moves (the
ε-closure). An ε-edge is written `-ε->` in Delta.

### Can I see a single accepting path?
The Run view shows every active branch. `run N on "…"` in the program prints the same steps as
text, and the diagram highlights the active states at the selected step.

### Is simulating an NFA slower than a DFA?
Each step handles a set of states instead of one, so it is up to `n` times slower per symbol, but
it never needs the exponential number of states a DFA may need.

## Related tools

- [NFA to DFA](/tools/nfa-to-dfa/): turn the sets into the states of a DFA
- [PDA simulator](/tools/pda-simulator/): nondeterminism with a stack
- [Turing machine simulator](/tools/turing-machine-simulator/): a tape instead of a set
- [Learn: NFAs](/learn/nfa/)
