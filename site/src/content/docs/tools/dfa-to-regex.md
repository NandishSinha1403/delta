---
title: "DFA to regex converter (state elimination)"
description: "Convert a DFA or NFA to a regular expression online by state elimination, Arden's equations or Brzozowski–McCluskey, and check the result."
sidebar:
  label: DFA to regex
  order: 3
---

Give Delta a finite automaton and it returns an equivalent regular expression. The example
converts a DFA for "an even number of a's" and checks the answer against a hand-written one.

```delta embed
dfa M {
  start q0
  accept q0
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q0;   q1 -b-> q1
}
let R = to_regex(M, method: "state-elimination")
check R ≡ M
check re"(b ∪ a b* a)*" ≡ M
print R
```

## How state elimination works

State elimination turns the automaton into a **generalized NFA** (GNFA), whose edges are labelled
with whole regular expressions instead of single symbols, and then removes states one at a time
until only a start and an accept state are left. The label of the last edge is the answer.

1. Add a new start state with an ε-edge to the old start, and a new accept state with ε-edges
   from every old accepting state. Merge parallel edges with `∪`.
2. Pick a state `q` to remove. For every pair of states `p` and `r` with edges `p → q` labelled
   `X` and `q → r` labelled `Z`, where `q` loops on itself with `Y`, add the edge `p → r`
   labelled `X Y* Z` (combined with `∪` with any existing `p → r` label).
3. Delete `q` and repeat until only the new start and accept remain.

Every path through `q` is replaced by an expression that describes it, so the language never
changes. The order in which you remove states changes the shape of the answer, not its meaning;
Delta's `order: [...]` argument picks it.

Delta has two other methods. `"arden"` writes one equation per state and solves the system with
**Arden's rule** (`X = A X ∪ B` has the solution `X = A* B`). `"brzozowski-mccluskey"` is state
elimination that always removes next the state with the fewest incoming × outgoing edges, which
keeps the expression small.

## Worked example

For the DFA above, remove `q1` first. It is entered from `q0` by `a`, loops on `b`, and returns to
`q0` by `a`, so the loop `q0 → q1 → q0` becomes `a b* a`. Together with the `b` loop on `q0`, the
start state now loops on `b ∪ a b* a`, and the expression is `(b ∪ a b* a)*`.

Delta's elimination gives `b* ∪ b* a (b ∪ a b* a)* a b*`, which looks different but is the same
language. `check R ≡ M` proves that, and so does the second check for the hand result.

## Frequently asked questions

### Is the regular expression unique?
No. Every regular language has infinitely many expressions, and different elimination orders
give different ones. Compare two of them with `check R1 ≡ R2`.

### Why is the expression so long?
State elimination can produce expressions exponentially larger than the automaton. Try another
`order:` or another method; `simplify(R)` applies algebraic identities.

### Does it work for NFAs too?
Yes. The construction never needs determinism, so you can pass an NFA directly.

### What is Arden's rule?
If `A` does not contain the empty word, the equation `X = A X ∪ B` has the unique solution
`X = A* B`. Solving one equation per state with it gives the expression for the start state.

## Related tools

- [Regex to NFA](/tools/regex-to-nfa/): the conversion in the other direction
- [DFA equivalence](/tools/dfa-equivalence/): compare two expressions or machines
- [DFA minimization](/tools/dfa-minimization/): fewer states usually mean a shorter expression
- [Learn: state elimination](/learn/state-elimination/)
