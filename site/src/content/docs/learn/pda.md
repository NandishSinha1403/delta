---
title: "What is a pushdown automaton (PDA)?"
description: "Pushdown automata explained: a finite automaton with a stack, transitions that push and pop, acceptance, and the link to context-free grammars."
sidebar:
  label: "What is a PDA?"
  order: 5
---

A **pushdown automaton** (PDA) is a finite automaton with a **stack**. Each transition may read an
input symbol (or nothing), pop the top of the stack (or nothing), and push a string. The stack
is unbounded, so a PDA can count, and match brackets, which a finite automaton cannot.

In Delta the edge `q1 -a, ε → A-> q1` reads `a`, pops nothing and pushes `A`. This PDA accepts
`{ aⁿbⁿ | n ≥ 0 }`:

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
check P accepts "", "ab", "aaabbb"
check P rejects "a", "ba", "aab"
run P on "aabb"
```

It pushes a bottom marker `Z`, then one `A` per `a`, pops one `A` per `b`, and accepts when it
sees `Z` again. The [PDA simulator](/tools/pda-simulator/) traces every step with its stack.

PDAs are **nondeterministic** in general, and that matters: deterministic PDAs recognize fewer
languages (even-length palindromes need the guess "this is the middle"). Nondeterministic PDAs
recognize exactly the [context-free languages](/learn/cfg/).

## Frequently asked questions

### Acceptance by final state or by empty stack?
Both give the same class of languages. Delta accepts by final state; pop a bottom marker into an
accepting state to accept by empty stack.

### Can a PDA recognize aⁿbⁿcⁿ?
No. After matching the `b`'s against the `a`'s the count is gone. That language needs a
[Turing machine](/learn/turing-machine/).

### Are deterministic PDAs weaker?
Yes. The language of even-length palindromes has a PDA but no deterministic one.
