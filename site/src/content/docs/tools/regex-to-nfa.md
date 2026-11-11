---
title: "Regex to NFA converter (Thompson and Glushkov)"
description: "Convert a regular expression to an NFA online. Compare Thompson's construction with Glushkov's ε-free automaton and see both diagrams."
sidebar:
  label: Regex to NFA
  order: 2
---

Write a regular expression and Delta builds an NFA for it, with the construction you choose.
The example builds two NFAs for the same expression and proves both correct.

```delta embed
let R = re"(a ∪ b)* a b"
let T = to_nfa(R, method: "thompson")
let G = to_nfa(R, method: "glushkov")
check T ≡ R
check G ≡ R
assert T.size == 12
assert G.size == 5
check G accepts "ab", "aab", "babab"
check G rejects "", "a", "ba", "abb"
show G
```

## How the constructions work

A regular expression is built from symbols with three operations: union `∪`, concatenation and
star `*`. Both constructions follow that structure, but they produce very different machines.

**Thompson's construction** builds a small NFA for each symbol and glues the pieces together
with ε-transitions. Every piece has one start and one accept state. A union adds a new start
that branches by ε into both parts; a concatenation links the accept of the first part to the
start of the second; a star adds ε-edges that skip the part or loop back to its start. The
result has at most two states per symbol and operator, and each state has at most two outgoing
ε-edges, which makes it easy to simulate.

**Glushkov's construction** (also called the position automaton) has no ε-transitions at all.
Number every occurrence of a symbol in the expression: these are its *positions*. The NFA has
one state per position plus a start state. From the expression it computes which positions can
come first, which can come last, and which can follow which; an edge goes from position `i` to
position `j`, labelled with `j`'s symbol, whenever `j` can follow `i`. An expression with `n`
symbol occurrences always gives exactly `n + 1` states.

Delta also offers `method: "textbook"` (the default), the ε-NFA construction most courses
present. It works like Thompson's but reuses states where it can: 10 states for this example.

## Worked example

For `(a ∪ b)* a b` the positions are `a₁ b₂ a₃ b₄`: four positions, so Glushkov's NFA has
**5 states**, as the program asserts.

- First positions: `a₁, b₂, a₃` (the star may be skipped).
- Last position: `b₄`.
- Follows: `a₁` and `b₂` can be followed by `a₁, b₂, a₃`; `a₃` by `b₄`.

Thompson's NFA for the same expression has **12 states** joined by ε-edges. Both accept
exactly the words that end in `ab`; `check T ≡ R` and `check G ≡ R` decide that, for every word.

## Frequently asked questions

### Which construction should I use?
Thompson's is the easiest to derive by hand and to explain. Glushkov's gives a smaller NFA with
no ε-transitions, which is usually the better input for the subset construction.

### Why does Thompson's NFA have ε-transitions?
They connect the sub-automata without merging their states, so each operator adds a constant
number of states and edges. The ε-edges disappear when you convert to a DFA.

### How many states does each NFA have?
Glushkov's has exactly one state per symbol occurrence plus one. Thompson's grows linearly with
the size of the expression, usually two to three times larger.

### Can I get a DFA directly?
Convert the NFA with `determinize(G)`, then `minimize` it. See the
[NFA to DFA](/tools/nfa-to-dfa/) tool.

### What syntax does Delta use for regular expressions?
`re"…"` with `∪` (or `|`) for union, juxtaposition with spaces for concatenation, `*` and `+`
for repetition and `ε` for the empty word. The [notation page](/docs/notation/) lists them all.

## Related tools

- [DFA to regex](/tools/dfa-to-regex/): the conversion in the other direction
- [NFA to DFA](/tools/nfa-to-dfa/): determinize the NFA you just built
- [NFA simulator](/tools/nfa-simulator/): run it on a word
- [Learn: regular expressions](/learn/regular-expressions/)
