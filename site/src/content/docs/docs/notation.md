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
| custom | `use notation { union: "+", epsilon: "λ", blank: "□", empty: "φ" }` | | |

Printing follows the profile too: under `linz`, `print ""` prints `λ`, a regex prints as
`a + b`, and the blank shows as `□` in diagrams, tables and tapes. Every spelling is always read
(`ε`, `λ`, `eps`; `⊔`, `_`, `□`). `B` is the blank only under `hmu`, and only in the labels of a
machine with a tape:

```delta
use notation "hmu"
tm T {
  alphabet 0
  start q0; accept qa; reject qr
  q0 -0 → R-> q0
  q0 -B → R-> qa      // B is the blank here
}
```

In a DFA with `alphabet A, B`, `B` is still an ordinary symbol.

Under the default profile `a+b` means one-or-more, and the Pad warns with two quick fixes:
`a ∪ b` (union) or `a⁺b` (one-or-more).

## Explain profiles

`use explain "sipser3e"` adds textbook citations to constructions and errors. It never changes a
result; citations live only in the profile, keyed by algorithm ids such as `determinize/subset`.
