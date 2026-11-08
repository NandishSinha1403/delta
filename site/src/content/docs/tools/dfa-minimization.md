---
title: "DFA minimization (Hopcroft and table filling)"
description: "Minimize a DFA online with Hopcroft's algorithm or the table-filling method. See which states merge and get the smallest equivalent DFA."
sidebar:
  label: DFA minimization
  order: 4
---

Delta merges the states of a DFA that no input can tell apart and returns the smallest DFA for
the same language. The example minimizes a six-state DFA two ways and checks that they agree.

```delta embed
dfa M {
  start a
  accept c, e
  a -0-> b;   a -1-> c
  b -0-> a;   b -1-> d
  c -0-> e;   c -1-> f
  d -0-> e;   d -1-> f
  e -0-> e;   e -1-> f
  f -0-> f;   f -1-> f
}
let H = minimize(M, method: "hopcroft")
let T = minimize(M, method: "table")
check H ≡ M
assert H.size == T.size
assert H.size == 5
show H
```

## How minimization works

Two states are **equivalent** when every word leads both to acceptance or both to rejection.
Equivalent states can be merged without changing the language, and once every class of
equivalent states is merged (and unreachable states are dropped) the DFA is the unique minimal
one, up to renaming states.

**Partition refinement** (Moore's method, and Hopcroft's faster version) starts from two groups,
accepting and non-accepting states, and splits a group whenever two of its states move to
different groups on the same symbol. When no group splits any more, each group is one state of
the minimal DFA. Hopcroft's algorithm processes the splits from a worklist and always reuses the
smaller half, which gives `O(n · |Σ| · log n)` time.

**Table filling** marks pairs of states that are distinguishable. First mark every pair with one
accepting and one non-accepting state. Then repeatedly mark a pair `(p, q)` if some symbol takes
it to a pair already marked. Pairs that stay unmarked are equivalent. It is easy to do by hand
and takes `O(n² · |Σ|)` time.

Delta also has `method: "moore"` and `method: "brzozowski"` (reverse, determinize, twice).

## Worked example

Start from accepting `{c, e}` and non-accepting `{a, b, d, f}`.

| state | on 0 goes to | on 1 goes to |
|---|---|---|
| `a` | non-accepting | accepting |
| `b` | non-accepting | non-accepting |
| `d` | accepting | non-accepting |
| `f` | non-accepting | non-accepting |

`a`, `d` and the pair `b, f` behave differently, so the group splits into `{a}`, `{d}`, `{b, f}`.
Next round: `b` moves to `{a}` on 0, while `f` stays in `{b, f}`, so `b` and `f` split as well.
`c` and `e` both go to `e` on 0 and to `f` on 1, so they stay together. The minimal DFA has
**5 states**: `{a}`, `{b}`, `{d}`, `{f}` and `{c, e}`.

## Frequently asked questions

### Is the minimal DFA unique?
Yes. For every regular language there is exactly one minimal DFA up to renaming states (the
Myhill–Nerode theorem), so any correct algorithm gives the same machine.

### Do I need to remove unreachable states first?
Delta does it for you. Unreachable states never affect the language, so they are dropped before
the states are merged.

### Can I minimize an NFA?
Minimizing NFAs is much harder (PSPACE-complete). Determinize it first with
[NFA to DFA](/tools/nfa-to-dfa/), then minimize the DFA.

### Which method is fastest?
Hopcroft's, in `O(n log n)` for a fixed alphabet. The table method is quadratic but is the one to
use when you check a minimization by hand.

### Why does the result keep a dead state?
Delta's DFAs are total: a state that rejects everything after it is part of the minimal total DFA.

## Related tools

- [NFA to DFA](/tools/nfa-to-dfa/): minimize the result of the subset construction
- [DFA equivalence](/tools/dfa-equivalence/): two DFAs are equivalent iff their minimal DFAs match
- [DFA to regex](/tools/dfa-to-regex/): fewer states usually give a shorter expression
- [Practice: minimize a DFA](/practice/)
