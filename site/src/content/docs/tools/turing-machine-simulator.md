---
title: "Turing machine simulator online"
description: "Simulate a Turing machine online. See the tape and head at every step, and get one of four verdicts: accept, reject, loops or unknown."
sidebar:
  label: Turing machine simulator
  order: 8
---

Write a single-tape Turing machine and run it. Delta shows the tape and head after every step,
and detects machines that loop instead of hanging the page.

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
check M accepts "", "01", "0011", "000111"
check M rejects "0", "1", "10", "001", "011"
run M on "0011"
```

## How a Turing machine runs

The machine has a finite set of states and a tape that is infinite to the right (`⊔` is the
blank). Each step reads the symbol under the head and, from the current state, writes a symbol,
moves the head left (`L`) or right (`R`), or stays (`S`), and changes state. In Delta the edge
`q0 -0 → x, R-> q1` means: in `q0`, reading `0`, write `x`, move right, go to `q1`. When the
written symbol is omitted (`q1 -0 → R-> q1`), the machine leaves the symbol as it is.

The machine above decides `{ 0ⁿ1ⁿ | n ≥ 0 }` by crossing off matching symbols:

1. In `q0`, replace the leftmost `0` with `x` and move right.
2. In `q1`, skip `0`'s and `y`'s until the first `1`, replace it with `y`, and turn back.
3. In `q2`, move left to the `x` and step right: the next `0`, if any, is under the head.
4. When no `0` is left, `q3` checks that only `y`'s remain, then accepts at the blank.

A missing transition rejects: on `10`, `q0` has no edge for `1`.

## Four verdicts

A Turing machine may never halt, so Delta's runs end in one of four verdicts: `accept` and
`reject` when it halts, `loops` when a configuration repeats (the machine provably runs forever),
and `unknown` when the step budget runs out first. The cookbook's
[four verdicts recipe](/docs/cookbook/tm-verdicts/) shows a machine for each.

## Worked example

The run on `0011` takes 13 steps (the head is just after `▸`):

| step | state | tape |
|---|---|---|
| 0 | `q0` | `▸0011` |
| 1 | `q1` | `x▸011` |
| 3 | `q2` | `x▸0y1` |
| 5 | `q0` | `x▸0y1` |
| 6 | `q1` | `xx▸y1` |
| 8 | `q2` | `xx▸yy` |
| 10 | `q0` | `xx▸yy` |
| 11 | `q3` | `xxy▸y` |
| 13 | `qa` | `xxyy⊔▸⊔` |

Two rounds cross off one `0` and one `1` each; then `q3` finds only `y`'s and accepts.

## Frequently asked questions

### What happens if my machine never halts?
If a configuration repeats, the verdict is `loops`. Otherwise the run stops at the step budget
with the verdict `unknown`, and the Pad stays responsive; the Stop button cancels any run.

### Is the tape infinite in both directions?
It is infinite to the right. Moving left from the first cell keeps the head where it is.

### Can I write a nondeterministic Turing machine?
Nondeterministic and multi-tape machines are part of Delta 2.0. This simulator runs
deterministic single-tape machines.

### How do I write the blank symbol?
`⊔`, or type `\blank` in the Pad and it completes to `⊔`.

### What is the difference between deciding and recognizing?
A decider halts on every input. A recognizer must accept every word in the language but may
loop on words outside it; Delta's `loops` and `unknown` verdicts make that difference visible.

## Related tools

- [PDA simulator](/tools/pda-simulator/): a stack instead of a tape
- [NFA simulator](/tools/nfa-simulator/): finite memory only
- [Learn: Turing machines](/learn/turing-machine/)
- [Practice](/practice/): Turing machine problems with solutions
