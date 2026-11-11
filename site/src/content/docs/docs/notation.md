---
title: Notation
description: Unicode and ASCII spellings, and notation profiles for other textbooks.
---

Every symbol has an ASCII spelling, and they mean the same thing:

| Meaning | Unicode | ASCII | Pad shortcut |
|---|---|---|---|
| empty word | `ε` (also `λ`) | `eps` | `\epsilon` |
| empty set | `∅` | `empty` | `\emptyset` |
| union / intersection | `∪` `∩` | `union` `inter` | `\cup` `\cap` |
| difference / complement | `∖` / `∁L` | `minus` / `~L` | `\setminus` |
| element / not / subset | `∈` `∉` `⊆` | `in` `not in` `subset` | `\in` |
| equivalence | `≡` | `equiv` | `\equiv` |
| arrow | `→` | `->` | `\to` |
| blank | `⊔` | `_` (in labels) | `\blank` |
| Greek | `δ Σ Γ Δ` | `delta Sigma Gamma Delta` | `\delta` |

## Profiles

A notation profile changes how regexes are read and printed, never what a program means:

```delta
use notation "hmu"
lang L = re"a+b"          // under hmu, + is union
assert "a" ∈ L and "b" ∈ L
```

| Profile | `+` in regexes | Empty word | Blank |
|---|---|---|---|
| `default` | one-or-more | ε | ⊔ |
| `hmu` | union | ε | B |
| `linz` | union | λ | □ |
| custom | `use notation { union: "+", epsilon: "λ", empty: "φ" }` | | |

Under the default profile `a+b` means one-or-more, and the Pad warns with two quick fixes:
`a ∪ b` (union) or `a⁺b` (one-or-more).

## Explain profiles

`use explain "sipser3e"` adds textbook citations to constructions and errors. It never changes a
result; citations live only in the profile, keyed by algorithm ids such as `determinize/subset`.
