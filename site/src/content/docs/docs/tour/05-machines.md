---
title: 5. Machines
description: Machine literals, the four transition forms, fields and options.
---

There are four ways to write transitions; they build the same machine.

```delta
dfa A {
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q2
  q2 -[a, b]-> q2                   // several labels on one arrow
}
dfa B {
  start q0
  accept q2
  δ = table {
       | a  | b
    q0 | q1 | q0
    q1 | q1 | q2
    q2 | q2 | q2
  }
}
assert A == B
```

Fields have readable names or textbook letters: `states`/`Q`, `alphabet`/`Σ`, `start`/`q0`,
`accept`/`F`. Options go in parentheses:

```delta
dfa M (partial) {                    // missing transitions go to an implicit dead state
  alphabet a, b
  start q0
  accept q1
  q0 -a-> q1
}
check M accepts "a"
check M rejects "b", "aa"
```

Without `(partial)`, a `dfa` must have a move for every state and symbol; the error offers both
fixes (add a dead state, or mark it partial):

```delta error
dfa Bad {
  alphabet a, b
  start q0
  accept q0
  q0 -a-> q0
}
```

States can be any value: `[expr]` writes one, and `for`/`if` generate transitions.

```delta
fn mod_counter(n) = dfa {
  alphabet a
  start [0]
  accept [0]
  for r ∈ 0..n { [r] -a-> [(r + 1) % n] }
}
check mod_counter(3) accepts "", "aaa"
check mod_counter(3) rejects "a", "aaaa"
```

Outside a machine, a bare state name is written as an atom: `A.start == :q0`.
