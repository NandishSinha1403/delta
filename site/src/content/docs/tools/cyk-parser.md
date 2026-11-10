---
title: "CYK parser: parse a word with a context-free grammar"
description: "Run the CYK algorithm online: fill the table of variables for every substring, get the parse tree, or see why a word is not in the language."
sidebar:
  label: CYK parser
  order: 7
---

Enter a grammar and a word, and Delta runs the **Cocke–Younger–Kasami (CYK)** algorithm: it fills
a table with the variables that derive each substring, then reads the parse tree off the table.
The Steps view lists every non-empty cell.

```delta embed
cfg G {
  S -> a S b | ε
}
let T = parse(G, "aabb", method: "cyk")
assert T.yield == "aabb"
assert parse(G, "aab", method: "cyk") == none
for cell in steps(T) { print cell }
show T
```

## How CYK works

CYK needs a grammar in [Chomsky normal form](/tools/cfg-to-cnf/); Delta converts the grammar first
if it isn't. For a word w of length n:

1. **Length 1.** For each position i, the cell for `w[i]` gets every variable A with a rule
   `A → w[i]`.
2. **Longer spans.** For each span w[i..j], from short to long, and each split point k, add A to
   the cell whenever `A → B C` with B in the cell for w[i..k] and C in the cell for w[k+1..j].
3. **Answer.** w is in the language exactly when the start variable is in the cell for the whole
   word. Back-pointers stored with each entry give the parse tree.

There are about n²/2 cells and up to n split points for each, so CYK takes O(n³ · |G|) time.

## Worked example

With `S → a S b | ε` converted to CNF, the word `aabb` fills these cells:

| Substring | Positions | Variables |
|---|---|---|
| `a`, `a` | 1, 2 | `U_a` |
| `b`, `b` | 3, 4 | `U_b` |
| `ab` | 2–3 | `S`, `S0` |
| `abb` | 2–4 | `X1`, `X2` |
| `aabb` | 1–4 | `S`, `S0` |

`S0` is in the top cell, so `aabb` is in the language. For `aab` the top cell has no `S0`, and
`parse` returns `none`.

## Frequently asked questions

### What is the difference between CYK and Earley?
Both decide membership for any context-free language in O(n³) time. CYK needs Chomsky normal
form and builds a full table; Earley works on the grammar as written and is faster on many
practical grammars. `parse(G, w)` uses Earley by default; pass `method: "cyk"` for CYK.

### Why is the tree in terms of new variables?
CYK parses the CNF grammar, so its tree uses that grammar's variables (`S0`, `U_a`, `X1`, …). For a
tree over your own rules, use the default Earley parser: `parse(G, w)`.

### How does CYK handle the empty word?
The table has no cells for the empty word. Instead, ε is in the language exactly when the CNF
grammar has the rule `S0 → ε`.

### Can CYK tell whether a grammar is ambiguous?
A cell can record several ways to derive the same variable, which shows that the word has more
than one parse tree. Delta keeps the first one found, so use `derive` and `parse` to inspect a
particular derivation.

## Related tools

- [CFG to CNF](/tools/cfg-to-cnf/): the conversion CYK runs first
- [CFG to PDA](/tools/cfg-to-pda/): recognize the same language with a pushdown automaton
- [PDA simulator](/tools/pda-simulator/)
