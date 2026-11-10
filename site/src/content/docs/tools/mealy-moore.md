---
title: "Mealy to Moore converter (and Moore to Mealy)"
description: "Convert a Mealy machine to an equivalent Moore machine and back, online, and compare their outputs on any input."
sidebar:
  label: Mealy ↔ Moore
  order: 9
---

Delta converts **Mealy machines** (output on transitions) to **Moore machines** (output on states)
and back, and checks that the outputs agree.

```delta embed
mealy M {
  alphabet 0, 1
  start q0
  q0 -0 / 0-> q0; q0 -1 / 0-> q1
  q1 -0 / 1-> q0; q1 -1 / 1-> q1
}
let N = mealy_to_moore(M)
assert N.output("0110") == M.output("0110")
assert M.output("0110") == "0011"
show N
```

This Mealy machine outputs the previous input symbol (starting with 0): a one-step delay.

## How the conversions work

- **Mealy → Moore.** A Moore state has to know which output to show, so each new state is a pair
  `(q, o)`: the old state q together with the output o of the transition that entered it. Its Moore
  output is o. The start state is `(q0, ε)`, whose output is empty, so the two machines produce the
  same word.
- **Moore → Mealy.** Each transition emits the output of the state it enters. A Moore machine also
  outputs λ(q₀) before reading anything, which a Mealy machine can't, so the Mealy output is the
  Moore output without its first symbol.

Moore outputs are written `λ(q) = w` (or `lambda(q) = w`) inside the machine body.

```delta
moore D {
  alphabet a, b
  start q0
  λ(q0) = 0
  λ(q1) = 1
  q0 -a-> q1; q0 -b-> q0
  q1 -a-> q1; q1 -b-> q0
}
assert D.output("ab") == "010"
assert D.output("ab") == "0" ++ moore_to_mealy(D).output("ab")
```

## Frequently asked questions

### Are Mealy and Moore machines equally powerful?
Yes, up to the first output: a Moore machine always emits λ(q₀) before reading input. Otherwise
each kind can simulate the other with the constructions above.

### How many states does the Moore machine need?
At most one for each pair (state, output word on an entering transition), plus the start: up to
|Q| · |Δ| + 1 for single-symbol outputs.

### Can the output be a word, not a symbol?
Yes. Both models in Delta emit words: `q0 -a / xy-> q1` for Mealy and `λ(q) = "xy"` for Moore.

## Related tools

- [NFA simulator](/tools/nfa-simulator/) and [DFA minimization](/tools/dfa-minimization/)
- [All tools](/tools/)
