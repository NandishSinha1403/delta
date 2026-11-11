---
title: "PDA simulator: pushdown automaton online"
description: "Simulate a pushdown automaton online. Step through every branch of a nondeterministic PDA with its stack, and test which words it accepts."
sidebar:
  label: PDA simulator
  order: 7
---

A pushdown automaton is a finite automaton with a stack. Delta runs it breadth first over all its
branches and shows each configuration, state, input position and stack, at every step.

```delta embed
pda P {
  start q0
  accept q3
  q0 -ε, ε → Z-> q1
  q1 -a, ε → A-> q1
  q1 -ε, Z → ε-> q3
  q1 -b, A → ε-> q2
  q2 -b, A → ε-> q2
  q2 -ε, Z → ε-> q3
}
check P accepts "", "ab", "aabb"
check P rejects "a", "aab", "ba"
assert P.class == CF
run P on "aabb"
```

## How a PDA runs

Each transition reads one input symbol or nothing (ε), pops one stack symbol or nothing, and
pushes a string. In Delta the edge `q1 -a, ε → A-> q1` means: in `q1`, read `a`, pop nothing, push
`A`, stay in `q1`.

A **configuration** is the triple (state, remaining input, stack). Because a PDA can be
nondeterministic, one configuration may have several successors, and ε-moves can apply without
reading. The simulator keeps the set of configurations reachable after each step, breadth first,
and accepts as soon as one of them has read the whole input in an accepting state.

The machine above recognizes `{ aⁿbⁿ | n ≥ 0 }`:

1. Push a bottom marker `Z`, so the machine can tell when the stack is empty.
2. Push an `A` for every `a`.
3. Pop an `A` for every `b`.
4. When `Z` is on top again, the counts matched: pop it and accept.

No finite automaton can do this, because it would have to remember an unbounded count. The
stack is exactly the memory a context-free language needs: `P.class == CF` confirms which class
Delta assigns to it.

To design your own PDA, decide what the stack should hold at each point of a word that is in the
language (here: one `A` per unmatched `a`), then add the transitions that keep it that way.

## Worked example

The run on `aabb` (the stack is written top first):

| step | state | read so far | stack |
|---|---|---|---|
| 0 | `q0` | | ε |
| 1 | `q1` | | `Z` |
| 2 | `q1` | `a` | `AZ` |
| 3 | `q1` | `aa` | `AAZ` |
| 4 | `q2` | `aab` | `AZ` |
| 5 | `q2` | `aabb` | `Z` |
| 6 | `q3` | `aabb` | ε |

At step 2 a second branch also pops `Z` and moves to `q3` without reading; it dies because input
is left over. The main branch ends in `q3` with all input read, so `aabb` is accepted.

## Frequently asked questions

### Does Delta accept by final state or by empty stack?
By final state: a run accepts when it has read the whole input and is in an accepting state. To
accept by empty stack, pop a bottom marker into an accepting state, as this example does.

### Can the PDA be nondeterministic?
Yes. Delta explores every branch breadth first, so nondeterministic PDAs such as the palindrome
recognizer in the cookbook work as written.

### What if a PDA loops on ε-moves forever?
The run is bounded by the step budget; when the budget runs out, the verdict is `unknown` instead
of a hang. Raise it with `budget { steps: … }`.

### What do the stack symbols mean?
Anything you choose. Here `Z` marks the bottom and `A` counts the `a`'s. Stack symbols are
declared implicitly by the edges that push them.

## Related tools

- [NFA simulator](/tools/nfa-simulator/): the same idea without a stack
- [Turing machine simulator](/tools/turing-machine-simulator/): a tape you can read and rewrite
- [Learn: pushdown automata](/learn/pda/)
- [Learn: context-free grammars](/learn/cfg/)
