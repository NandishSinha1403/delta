---
title: Languages and classes
description: The class lattice, the closure table, the decision table and bounded checks.
---

A `lang` value is a set of words with a **class**, the most Delta can guarantee about it:

Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any

| Source | Class |
|---|---|
| a finite set of words | Finite |
| a regex, dfa, nfa, gnfa, twdfa | Regular |
| a dpda | DCF |
| a pda, a cfg | CF |
| an lba | CS |
| a set-builder predicate `{ w ∈ Σ* \| … }` | Decidable |
| a tm, ntm | Recognizable |

## Closure table

| Operation | Finite | Regular | DCF | CF | CS | Decidable | Recognizable | Any |
|---|---|---|---|---|---|---|---|---|
| `L ∪ K` | Finite | Regular | CF | CF | CS | Decidable | Recognizable | Any |
| `L ∩ K` | Finite | Regular | CS | CS | CS | Decidable | Recognizable | Any |
| `L ∩ R` (R regular) | Finite | Regular | DCF | CF | CS | Decidable | Recognizable | Any |
| `F ∩ L` (F finite) | Finite | Finite | Finite | Finite | Finite | Finite | Recognizable | Recognizable |
| `∁L` | Regular | Regular | DCF | CS | CS | Decidable | Any | Any |
| `F ∖ L` (F finite) | Finite | Finite | Finite | Finite | Finite | Finite | Any | Any |
| `L ++ K` | Finite | Regular | CF | CF | CS | Decidable | Recognizable | Any |
| `L*` | Regular | Regular | CF | CF | CS | Decidable | Recognizable | Any |
| `rev(L)` | Finite | Regular | CF | CF | CS | Decidable | Recognizable | Any |

Mixed operands take the larger class. `L ∖ K` is `L ∩ ∁K`.

```delta
lang F = { "", "01" }
lang R = re"(0 ∪ 1)* 0 1"
lang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') }
assert (F ∩ P).class == Finite
assert (F ∖ P).is_empty()
assert (R ∪ F).class == Regular
```

## Decision table

| Question | Finite / Regular | DCF | CF | CS | Decidable | Recognizable |
|---|---|---|---|---|---|---|
| `w ∈ L` | ✓ | ✓ | ✓ | ✓ | ✓ (budgeted) | may not halt: `unknown` |
| `L.is_empty()` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
| `L.is_finite()` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
| `L.is_universal()` | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `L ≡ K` | ✓ | decidable, not implemented | ✗ | ✗ | ✗ | ✗ |
| `L ⊆ K` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |

An ✗ question is error [E701](/docs/errors/e701/) before the program runs, and its quick fix
adds `up to 12`. A bounded check (`check L ≡ K up to n`) works for every class and is labeled
bounded in the results.
