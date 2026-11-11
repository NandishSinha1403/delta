---
title: "State elimination: from automaton to regular expression"
description: "How state elimination converts a DFA or NFA into a regular expression through a generalized NFA, with the rule for removing a state and an example."
sidebar:
  label: "State elimination"
  order: 8
---

**State elimination** turns a finite automaton into a regular expression. It works on a
**generalized NFA** (GNFA), whose edges carry regular expressions instead of symbols.

1. Add a new start state with an ε-edge to the old start, and a new accepting state with ε-edges
   from the old accepting states.
2. Remove the other states one at a time. Removing `q` replaces every path `p → q → r` by one edge
   `p → r` labelled `X Y* Z`, where `X` labels `p → q`, `Y` the loop on `q`, and `Z` labels
   `q → r`. Combine it with any existing `p → r` label using `∪`.
3. When only the new start and accept states are left, the label between them is the answer.

```delta embed
dfa M {
  start q0
  accept q0
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q0;   q1 -b-> q1
}
let G = to_gnfa(M)
check G ≡ M
let R = to_regex(M, method: "state-elimination")
check R ≡ M
print R
```

For this DFA (an even number of `a`'s), removing `q1` turns the path `q0 → q1 → q0` into the loop
`a b* a`, so `q0` loops on `b ∪ a b* a` and the answer is `(b ∪ a b* a)*`.

**Why it is correct.** Each removal keeps the set of labelled paths between the remaining states
the same: every way through `q` is described by `X Y* Z`. So the language never changes.

The order of removal changes the expression, not its language, and a bad order can make it much
longer. The [DFA to regex tool](/tools/dfa-to-regex/) also offers Arden's equations.

## Frequently asked questions

### Why add new start and accept states?
So that the start has no incoming edges and there is exactly one accepting state; then one edge
is left at the end.

### Which state should I remove first?
One with few incoming and outgoing edges keeps the expression short.

### Can the expression blow up?
Yes, exponentially in the worst case, which is one reason expressions and automata are both useful.
