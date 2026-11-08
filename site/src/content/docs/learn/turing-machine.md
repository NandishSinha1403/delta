---
title: "What is a Turing machine?"
description: "Turing machines explained: tape, head, states and transitions, accepting, rejecting and looping, with a runnable machine for 0ⁿ1ⁿ."
sidebar:
  label: "What is a Turing machine?"
  order: 6
---

A **Turing machine** has a finite set of states and an unbounded **tape** with a **head** on it.
Each step reads the symbol under the head, writes a symbol, moves the head left or right, and
changes state. The input starts on the tape; the machine halts when it enters its accept or
reject state.

In Delta, `q0 -0 → x, R-> q1` means: in `q0` reading `0`, write `x`, move right, go to `q1`. This
machine decides `{ 0ⁿ1ⁿ }` by crossing off one `0` and one `1` per round:

```delta embed
tm M {
  alphabet 0, 1
  start q0
  accept qa
  q0 -0 → x, R-> q1;   q0 -y → R-> q3;   q0 -⊔ → R-> qa
  q1 -0 → R-> q1;      q1 -y → R-> q1;   q1 -1 → y, L-> q2
  q2 -0 → L-> q2;      q2 -y → L-> q2;   q2 -x → R-> q0
  q3 -y → R-> q3;      q3 -⊔ → R-> qa
}
check M accepts "", "01", "0011"
check M rejects "0", "10", "011"
run M on "0011"
```

Unlike a finite automaton, a Turing machine may **never halt**. A language is **decidable** if
some machine halts on every input and accepts exactly the words of the language, and
**recognizable** if some machine accepts exactly those words (it may loop on the others). The
**halting problem**, whether a given machine halts on a given input, is recognizable but not
decidable.

The Church–Turing thesis says these machines capture everything an algorithm can compute.
Variants with several tapes or nondeterminism compute the same functions.

## Frequently asked questions

### What does Delta do if my machine loops?
A repeated configuration gives the verdict `loops`; a run that hits the step budget gives
`unknown`. See the [Turing machine simulator](/tools/turing-machine-simulator/).

### Why is the halting problem undecidable?
A decider for it could be turned against itself: run on its own description, it would halt if
and only if it does not.

### Is a Turing machine more powerful than a PDA?
Yes. It recognizes every context-free language and more, such as `{ aⁿbⁿcⁿ }`.
