---
title: "Equivalence: does this DFA accept (a+b)*ab?"
description: "Solved GATE-style problem: prove that a 3-state DFA accepts exactly the strings ending in ab, and find the shortest string that separates it from (a+b)*ab(a+b)*."
sidebar:
  label: "8. A DFA and a regex"
  order: 8
---

**Two parts.** The DFA `A` below has states `p` (start), `q` and `r` (accepting).
(i) Is `L(A) = (a ∪ b)* a b`? (ii) What is the shortest string on which `A` and
`(a ∪ b)* a b (a ∪ b)*` disagree?

| state | on `a` | on `b` |
|---|---|---|
| `p` | `q` | `p` |
| `q` | `q` | `r` |
| `r` (accept) | `q` | `p` |

## Solution

(i) State `q` means "the last symbol was `a`", `r` means "the last two symbols were `ab`", and
`p` covers everything else. Each transition keeps that meaning true, so `A` accepts exactly the
strings that **end in** `ab`. **Yes.**

(ii) `(a ∪ b)* a b (a ∪ b)*` is the set of strings that **contain** `ab`. A string that contains
`ab` but does not end in it needs at least one symbol after the `ab`, so it has length 3 or more.
Of length 3, `aba` and `abb` contain `ab` without ending in it (while `aab` and `bab` end in it,
so both sides accept them). In shortlex order the first is **`aba`**.

## Check it in Delta

```delta
dfa A {
  start p
  accept r
  p -a-> q;   p -b-> p
  q -a-> q;   q -b-> r
  r -a-> q;   r -b-> p
}
let R = re"(a ∪ b)* a b"
check A ≡ R
let R2 = re"(a ∪ b)* a b (a ∪ b)*"
assert shortest((A ∖ R2) ∪ (R2 ∖ A)) == "aba"
```
