---
title: "CFG to CNF converter (Chomsky normal form)"
description: "Convert a context-free grammar to Chomsky normal form online, step by step: new start variable, ε-rules, unit rules, terminals and long rules."
sidebar:
  label: CFG to CNF
  order: 6
---

Write a context-free grammar below and Delta converts it to **Chomsky normal form** (CNF), shows
every phase of the conversion in the Steps view, and checks that the new grammar generates the
same words.

```delta embed
cfg G {
  S -> a S b | ε
}
let C = cnf(G)
check C ≡ G up to 8
check C accepts "", "ab", "aabb"
show C
```

## What Chomsky normal form is

A grammar is in CNF when every rule has one of two shapes: `A → B C` (two variables, neither of
them the start variable) or `A → a` (one terminal). The start variable may also have `S₀ → ε`
when the language contains the empty word. CNF is what the [CYK parser](/tools/cyk-parser/)
needs, and it makes every derivation of a word of length n take exactly 2n − 1 steps.

## How the conversion works

Delta's `cnf(G)` runs five phases in this order, then removes useless variables:

1. **START**: add a new start variable `S0 → S`, so the start never appears on a right side.
2. **DEL**: remove each ε-rule `A → ε`. For every rule that mentions a nullable variable, add
   copies with that occurrence deleted.
3. **UNIT**: replace each unit rule `A → B` with `A → u` for every non-unit rule `B → u`.
4. **TERM**: in right sides of length two or more, replace each terminal `a` with a new variable
   `U_a → a`.
5. **BIN**: split each right side longer than two into a chain: `A → B₁ X1`, `X1 → B₂ X2`, …

The order matters. Removing ε-rules creates unit rules, so UNIT has to come after DEL, and the
new start has to exist before DEL so that `S0 → ε` can keep ε in the language.

## Worked example

For `S → a S b | ε` the phases are:

| Phase | Change |
|---|---|
| START | add `S0 → S` |
| DEL | remove `S → ε`; add `S → a b` and `S0 → ε` |
| UNIT | replace `S0 → S` by `S0 → a S b` and `S0 → a b` |
| TERM | add `U_a → a`, `U_b → b` |
| BIN | `S → U_a X1`, `X1 → S U_b`, `S0 → U_a X2`, `X2 → S U_b` |

The result is `S0 → ε | U_a X2 | U_a U_b`, `S → U_a X1 | U_a U_b`, `X1 → S U_b`,
`X2 → S U_b`, `U_a → a`, `U_b → b`, and `check C ≡ G up to 8` confirms that both grammars
generate the same words up to length 8.

## Frequently asked questions

### Does every context-free grammar have a CNF equivalent?
Yes. Every context-free language has a grammar in Chomsky normal form (with `S₀ → ε` allowed
when ε is in the language), and the five phases construct it.

### Can the grammar get much bigger?
Yes. Removing ε-rules adds a copy of a rule for every subset of its nullable occurrences, so a
rule with k nullable variables can become 2ᵏ rules. Splitting long rules adds one variable per
extra symbol.

### Why is `check C ≡ G` bounded?
Equivalence of context-free grammars is undecidable, so Delta compares all words up to a length
you choose. The conversion itself is proven correct; the bounded check is a sanity check.

### What are U_a and X1?
`U_a` stands for the terminal `a` inside a long rule, and `X1`, `X2`, … hold the rest of a right
side when it's split. If your grammar already uses one of those names, Delta picks the next free
number.

## Related tools

- [CYK parser](/tools/cyk-parser/): parse a word with the CNF grammar and see the table
- [CFG to PDA](/tools/cfg-to-pda/): turn a grammar into a pushdown automaton
- [PDA simulator](/tools/pda-simulator/): run the result step by step
