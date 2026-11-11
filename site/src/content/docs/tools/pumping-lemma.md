---
title: "Pumping lemma for regular languages (split w = xyz)"
description: "Find the pumping split w = xyz of a word in a DFA's language online: see p, x, y and z, pump y up and down, and use it to show a language isn't regular."
sidebar:
  label: Pumping lemma
  order: 10
---

Give Delta a DFA (or a regex, NFA or regular `lang`) and a word in its language, and
`pump(D, w)` finds the split `w = xyz` that the pumping lemma promises: `|xy| ≤ p`, `|y| ≥ 1`,
and `x yⁱ z` stays in the language for every `i`. The example uses it to catch a DFA that claims
to recognize `{ 0ⁿ1ⁿ }` but can't.

```delta embed
dfa D {                     // 0*1*: no 0 after a 1
  alphabet 0, 1
  start q0
  accept q0, q1
  q0 -0-> q0
  q0 -1-> q1
  q1 -1-> q1
  q1 -0-> dead
  dead -0-> dead
  dead -1-> dead
}
let s = pump(D, "0011")
print s
check D accepts s.x ++ s.z, s.x ++ s.y ++ s.z, s.x ++ s.y^2 ++ s.z
let w2 = s.x ++ s.y^2 ++ s.z
assert count(w2, '0') != count(w2, '1')      // D accepts a word outside { 0ⁿ1ⁿ }
```

## How the split is found

Let `p` be the number of states of the DFA. Run it on `w` and write down the states
`r₀, r₁, …, r_|w|`. A word with `|w| ≥ p` visits `p + 1` states in its first `p` steps, so
some state repeats: `rᵢ = rⱼ` with `i < j ≤ p`. The symbols read between the two visits form a
loop, and that loop is `y`:

- `x = w[0..i]` leads from the start to the repeated state,
- `y = w[i..j]` goes around the loop and comes back to it,
- `z = w[j..]` finishes the run.

Going around the loop zero times (`xz`) or many times (`x yⁱ z`) ends in the same state as `w`,
so every pumped word is accepted exactly when `w` is. Delta picks the **first** repeat, which
gives the shortest `xy`. The result is a record `{p, x, y, z, state}`, where `state` is the
repeated state.

If `w` isn't in the language, or `|w| < p`, `pump` stops with error E504: the lemma promises
nothing for those words.

## Using it to show a language isn't regular

The pumping lemma is usually used the other way round: to prove that **no** DFA recognizes a
language. For `L = { 0ⁿ1ⁿ }`, suppose a DFA with `p` states did. Take `w = 0ᵖ1ᵖ`. Because
`|xy| ≤ p`, the loop `y` lies inside the 0s, so `x y² z` has more 0s than 1s, yet the DFA
accepts it. That contradiction shows `L` is not regular.

The example plays this game against one concrete DFA: `D` accepts `0011`, the split puts `y`
inside the 0s, and pumping it gives `00011`, which `D` accepts although it isn't in `L`. Any
other DFA you write for `L` fails the same way. Try replacing `D` with your own attempt.

## Worked example

`D` has `p = 3` states (`q0`, `q1` and `dead`). On `0011` the run is `q0 →0 q0 →0 q0 →1 q1 →1 q1`.
The first repeat is `q0`, at positions 0 and 1, so `x = ε`, `y = 0` and `z = 011`.

| i | x yⁱ z | in L(D)? | in { 0ⁿ1ⁿ }? |
|---|---|---|---|
| 0 | `011` | yes | no |
| 1 | `0011` | yes | yes |
| 2 | `00011` | yes | no |

## Frequently asked questions

### What is the pumping length p?
For a DFA it is the number of states. Delta uses the DFA you give it as it is; for a regex, NFA
or `lang` it first builds the minimal DFA, so `p` is as small as possible.

### Can pump prove that a language is not regular?
No program can do that for every language. `pump` shows how the lemma works on a real DFA: it
finds the split and lets you check the pumped words. The proof that a language isn't regular is
the argument for every DFA, as in the section above.

### Is there a pumping lemma for context-free languages?
Yes, with a split `uvxyz` that pumps two parts at once. Delta doesn't compute that split yet; for
grammars, try the [CYK parser](/tools/cyk-parser/) and `is_ambiguous(G, upto: n)`.

### What about Myhill–Nerode?
`myhill_nerode(D)` lists the classes of a regular language with their shortest words and the
suffixes that tell them apart. A language with infinitely many classes is not regular, which is
often an easier proof than pumping.

## Related tools

- [DFA minimization](/tools/dfa-minimization/): the minimal DFA gives the smallest `p`
- [NFA to DFA](/tools/nfa-to-dfa/): build a DFA to pump from an NFA
- [DFA equivalence](/tools/dfa-equivalence/): check a DFA against a language
- [Learn: DFAs](/learn/dfa/)
