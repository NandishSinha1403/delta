---
title: 6. Members and runs
description: Machine members, δ, runs, traces and verdicts.
---

```delta
nfa N {
  alphabet a, b
  start q0
  accept q2
  q0 -[a, b]-> q0;   q0 -a-> q1;   q1 -b-> q2
}
assert N.start == :q0
assert N.accept == {:q2}
assert N.δ(:q0, 'a') == {:q0, :q1}       // a set for an NFA
assert N.size == 3
assert N.accepts("aab")
assert N.run("aab").verdict == :accept
run N on "abab"
```

A run ends with one of four **verdicts**: `accept`, `reject`, `loops` (a configuration repeats,
so it never halts) or `unknown` (the budget ran out first). `M.accepts(w)` is true for accept and
false for reject and loops.

Other members: `M.states`, `M.alphabet`, `M.edges`, `M.eclose(S)`, `M.rename(f)`,
`M.with(accept: …)`, `M.lang`, `M.output(w)` for transducers.
