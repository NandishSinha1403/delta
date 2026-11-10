---
title: "CFG to PDA converter (and PDA to CFG)"
description: "Convert a context-free grammar to a pushdown automaton online, run it with its stack, and convert a PDA back to a grammar."
sidebar:
  label: CFG to PDA
  order: 8
---

Write a grammar and Delta builds a **pushdown automaton** that accepts the same language, draws it,
and lets you run it with the stack shown at every step. The reverse construction, `to_cfg`, turns
any pushdown automaton back into a grammar.

```delta embed
cfg G {
  S -> a S b | ε
}
let P = to_pda(G)
check P accepts "", "ab", "aabb"
check P rejects "a", "ba", "abb"
check to_cfg(P) ≡ G up to 6
show P
run P on "aabb"
```

## How CFG → PDA works

The automaton has three states. It pushes `S$` and moves to `loop`, where it repeats two moves:

- if the top of the stack is a variable A, pop it and push the right side of a rule `A → u`
  (choosing the rule nondeterministically);
- if the top is a terminal a, read a from the input and pop it.

When `$` reaches the top, the whole input has been matched and the automaton accepts. The stack
always holds what is left of a **leftmost derivation**, so the automaton accepts exactly the words
the grammar generates.

## How PDA → CFG works

`to_cfg(P)` first normalizes the automaton so that every move either pushes or pops exactly one
symbol and every accepting run ends with an empty stack in a single state. Then each pair of states
p, q gets a variable ⟨p,q⟩ that generates the words taking the automaton from p to q with the stack
as it was. The rules say a run either stays put (⟨p,p⟩ → ε), passes through an empty stack in the
middle (⟨p,q⟩ → ⟨p,r⟩⟨r,q⟩), or pushes a symbol at the start and pops it at the end
(⟨p,q⟩ → a ⟨r,s⟩ b). Useless variables are removed at the end.

## Worked example

For `S → a S b | ε`, `to_pda(G)` has these moves on `loop`: `ε, S → aSb`, `ε, S → ε`, `a, a → ε`,
`b, b → ε`, plus `start —ε, ε → S$→ loop` and `loop —ε, $ → ε→ accept`. On `aabb` the stack goes
`S$` → `aSb$` → `Sb$` → `aSbb$` → `Sbb$` → `bb$` → `b$` → `$`, and the automaton accepts.

## Frequently asked questions

### Do pushdown automata and context-free grammars describe the same languages?
Yes. Every context-free grammar has an equivalent pushdown automaton, and every pushdown automaton
has an equivalent grammar. Both constructions are on this page.

### Is the PDA deterministic?
No. It chooses a rule whenever a variable is on top of the stack. Some context-free languages, such
as even-length palindromes, have no deterministic pushdown automaton at all.

### Why is the grammar from `to_cfg` so large?
It has a variable for every pair of states and a rule for every way of splitting a run, so it grows
with the cube of the number of states. Delta removes useless variables, but the result is still
much bigger than a hand-written grammar.

### How does Delta decide whether a PDA accepts a word?
For a yes-or-no answer Delta converts the automaton to a grammar and parses the word, which always
finishes. A step-by-step `run` explores the automaton's configurations instead, so you can watch
the stack.

## Related tools

- [PDA simulator](/tools/pda-simulator/): run any pushdown automaton step by step
- [CFG to CNF](/tools/cfg-to-cnf/) and the [CYK parser](/tools/cyk-parser/)
