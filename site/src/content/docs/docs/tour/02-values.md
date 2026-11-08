---
title: 2. Values
description: Numbers, symbols, words, sets, lists, tuples, records, maps and atoms.
---

Every value is immutable. `let` binds a name once; `var` declares one you can rebind.

```delta
let n = 7 * 6
let s = 'a'                        // a symbol
let w = "abba"                     // a word: a sequence of symbols
let S = {3, 1, 2}                  // sets print in canonical order
let t = (1, "x")                   // a tuple
let r = { from: :q0, to: :q1 }     // a record; :q0 is an atom
let m = { :q0: "even", :q1: "odd" }    // a map keyed by atoms
assert n == 42
assert w[0] == s                   // indexing a word gives a symbol
assert s != "a"                    // a symbol is not a word
assert S == {1, 2, 3}
assert m[r.to] == "odd"
assert ε == ""
print S, w, t
```

Integers are 64-bit and overflow is an error. `/` and `%` are Euclidean: `-7 % 3 == 2`.

Unicode and ASCII spellings always mean the same thing: `∪`/`union`, `∈`/`in`, `ε`/`eps`,
`≠`/`!=`, `→`/`->`. Type `\cup`, `\in`, `\delta`… in the Pad to insert the symbols.
