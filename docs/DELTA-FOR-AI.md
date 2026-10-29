# Delta: syntax guide for AI models

Delta writes finite automata and regular expressions exactly as Sipser's *Introduction to the
Theory of Computation* (3rd ed.) defines them. Every code block below is run by the test suite
(`crates/delta-dsl/src/guide_tests.rs`), so the examples are guaranteed to be valid.

**Instructions for the model**
1. Output only Delta code, in one block.
2. **Always include `check … accepts` and `check … rejects` lines** for every machine or expression
   you write, with at least three strings each, including the empty string `""` on the correct side.
3. Use only the syntax shown here. Anything else is an error or is skipped.

Statements go one per line (machine bodies span lines): `dfa`, `nfa`, `regex` declare objects;
`let` converts or combines them; `check` and `run` test them. Comments: `// …` and `/* … */`.

## 1. DFA (Definition 1.5)

```delta
// M1 from Figure 1.6: at least one 1, and an even number of 0s after the last 1.
dfa M1 {
  Q  = { q1, q2, q3 }
  Σ  = { 0, 1 }
  δ  = table {
          | 0  | 1
       q1 | q1 | q2
       q2 | q3 | q2
       q3 | q2 | q2
  }
  q0 = q1
  F  = { q2 }
}

check M1 accepts "1", "01", "100", "0101"
check M1 rejects "", "0", "10", "1000"
run M1 on "1101"
```

- Fields: `Q`, `Σ`, `δ`, `q0`, `F` (any order). `F = q2` (one state, no braces) and `F = ∅` are allowed.
- **δ must be total**: exactly one arrow for every state and every symbol of Σ (see §6, W201/E201).
- The table's first row starts with `|` and lists the symbols; each later row is `state | target | …`.

δ can also be written as arrows, `δ { a -0-> b … }` (see §2), or one entry per line as below.
All three forms work in both `dfa` and `nfa`.

```delta
dfa Parity {
  Q = { even, odd }
  Σ = { 0, 1 }
  δ(even, 0) = odd
  δ(even, 1) = even
  δ(odd, 0) = even
  δ(odd, 1) = odd
  q0 = even
  F  = { even }
}

check Parity accepts "", "1", "00", "0110"
check Parity rejects "0", "01", "000"
```

Arrow lists: `p -[0,1]-> q` is two arrows. State names may be words or numbers (`q1`, `even`, `1`).

## 2. NFA (Definition 1.37)

```delta
// N1 from Example 1.38: contains 101 or 11 as a substring.
nfa N1 {
  Q  = { q1, q2, q3, q4 }
  Σ  = { 0, 1 }
  δ {
    q1 -[0,1]-> q1
    q1 -1-> q2
    q2 -[0,ε]-> q3
    q3 -1-> q4
    q4 -[0,1]-> q4
  }
  q0 = q1
  F  = { q4 }
}

check N1 accepts "11", "101", "010110"
check N1 rejects "", "0", "10", "100100"
run N1 on "010110"
```

- Arrows may be labelled `ε`; δ may be partial (missing arrows mean ∅); a state may have several
  arrows on the same symbol. In a table, an NFA cell is a set: `{q1, q2}` or `∅`, and the header may
  include an `ε` column:

```delta
nfa N {
  Q = { p, q }
  Σ = { a }
  δ = table {
      | a      | ε
    p | {p, q} | ∅
    q | ∅      | ∅
  }
  q0 = p
  F  = { q }
}

check N accepts "a", "aaa"
check N rejects ""
```

## 3. Regular expressions (Definition 1.52)

```delta
regex R1 over { a, b } = (ab ∪ a)*
regex R2 over { 0, 1 } = Σ*1Σ^2
regex R3 over { 0, 1 } = 1*(01+)*
regex R4 = (a ∪ b)*abb

check R1 accepts "", "a", "ab", "aab", "abab"
check R1 rejects "b", "ba", "abb"
check R2 accepts "100", "0111"
check R2 rejects "", "10", "1"
check R3 accepts "", "1", "01", "0111011"
check R3 rejects "0", "00", "10"
check R4 accepts "abb", "aabb", "babb"
check R4 rejects "", "ab", "abba"
```

- Operators, tightest first: `*` (star), `+` (one or more: R⁺ = RR*), `^k` (k copies, k ≤ 64), then
  concatenation (juxtaposition, or `∘`), then union `∪` (ASCII `|`).
- `ε`, `∅` and `Σ` (any one symbol) are atoms. **Each character is one symbol**: `ab` is a ∘ b.
- `over { … }` sets Σ; without it Σ is the set of symbols the expression uses. Write it whenever Σ has
  symbols the expression never mentions.

## 4. Conversions and closure operations (`let`)

`let NAME = function(arguments)` names the result. Arguments are names or nested calls.

| Call | Input → output | Book |
|---|---|---|
| `dfa(N)` | NFA or regex → DFA (reachable subsets; `dfa(N, full: true)` = all 2ᵏ) | Theorem 1.39 |
| `nfa(R)` / `nfa(D)` | regex → NFA, or DFA viewed as an NFA | Lemma 1.55, p. 47 |
| `gnfa(D)` | DFA or NFA → GNFA in special form | Lemma 1.60 |
| `regex(G)` | GNFA, DFA or NFA → regex; options `mode: faithful` (default) or `simplified`, `order: [q2, q1]` (rip order) | Lemma 1.60 |
| `union(A, B)` | two DFAs → DFA by the product construction; with an NFA → NFA | Thm 1.25 / 1.45 |
| `intersect(A, B)` | two DFAs → DFA, F = F₁ × F₂ | Thm 1.25 footnote |
| `concat(A, B)` | DFAs/NFAs → NFA | Theorem 1.47 |
| `star(A)` | DFA/NFA → NFA (new accepting start state) | Theorem 1.49 |

```delta
nfa N {
  Q = { q1, q2, q3 }
  Σ = { a, b }
  δ {
    q1 -[a,b]-> q1
    q1 -a-> q2
    q2 -b-> q3
  }
  q0 = q1
  F  = { q3 }
}

dfa EvenA {
  Q = { e, o }
  Σ = { a, b }
  δ = table {
       | a | b
     e | o | e
     o | e | o
  }
  q0 = e
  F  = { e }
}

regex Bs over { a, b } = b*

let D  = dfa(N)                          // Theorem 1.39
let G  = gnfa(D)                         // Lemma 1.60, special form
let R  = regex(G, mode: simplified)      // Lemma 1.60, CONVERT(G)
let NB = nfa(Bs)                         // Lemma 1.55
let U  = union(D, EvenA)                 // Theorem 1.25 (both DFAs: product)
let I  = intersect(D, EvenA)             // Theorem 1.25 footnote
let C  = concat(N, NB)                   // Theorem 1.47
let S  = star(N)                         // Theorem 1.49
let DS = dfa(star(N))                    // calls nest

check D accepts "ab", "aab", "bab"
check D rejects "", "a", "abb"
check R accepts "ab", "bbab", "abab"
check R rejects "", "ba", "aba"
check U accepts "", "b", "ab", "aa", "bab"
check U rejects "a", "ba", "aaa"
check I accepts "aab", "abab", "baab"
check I rejects "", "ab", "bab"
check C accepts "ab", "abbb", "aabb"
check C rejects "", "a", "aba"
check S accepts "", "ab", "abab", "bab"
check S rejects "a", "b", "aba"
check DS accepts "", "abab"
check DS rejects "ba"
```

A GNFA cannot be declared directly yet (`gnfa G { … }` is skipped with E123); make one with `gnfa(…)`.

## 5. Commands

- `check NAME accepts "w1", "w2", …` and `check NAME rejects …`: works on any object (DFA, NFA,
  regex, or a `let` result). `""` is ε. A failing case is warning R910.
- `run NAME on "w"`: records a step-by-step run of a DFA or NFA (shown in the playground).
- Strings are sequences of symbols of Σ; write a multi-character symbol as `<NAME>`, e.g.
  `"<RESET>a"`, and declare it the same way: `Σ = { <RESET>, a }`.
- `#mode strict` on its own line turns lenient warnings (W201, W117, I115, I116) into errors.

## 6. Rules that cause errors (and their codes)

| Code | Rule |
|---|---|
| E201 / W201 | **DFA δ must be total.** Missing arrows: W201 warning plus an implicit trap state (lenient, the default); E201 error with `#mode strict`. Write `dfa M partial { … }` to accept the trap silently. |
| E202 | DFA δ is a function: two arrows from the same state on the same symbol. Use `nfa`. |
| E203 | **ε-arrows only in an NFA.** A DFA's δ takes Σ, not Σ_ε. |
| E204 | A DFA cell/target is one state, not a set `{…}`. |
| E104 | A state used in δ, q0 or F is not in Q. |
| E105 / E301 | **Every arrow symbol must be in Σ** (E105 in a DFA, E301 in an NFA). |
| E108 | `ε` listed as a member of Σ (ε is the empty string, not a symbol). |
| E114 | Multi-character symbol written bare: write `<ab>`, not `ab`. |
| E401 / E402 | Unbalanced parentheses / operator with a missing operand (`a ∪`, `()`). |
| E403 | Regex uses a symbol not in its `over { … }` alphabet. |
| W405 | **`+` means one-or-more, not union** (p. 65). `a+b` is a⁺b; for union write `a ∪ b` or `a | b`. |
| E801 | Wrong argument type, e.g. `dfa(D)` on a DFA, `intersect` on NFAs, `union` on a regex. |
| E802 | Unknown name (E803: a name defined twice). |
| R901 | A `check`/`run` string contains a symbol not in Σ. |
| R910 | A `check` case fails. |

```delta-fails
// expect: E203
dfa M {
  Q = { p }
  Σ = { a }
  δ {
    p -a-> p
    p -ε-> p
  }
  q0 = p
  F  = { p }
}
```

```delta-fails
// expect: E105
dfa M {
  Q = { p }
  Σ = { a }
  δ { p -b-> p }
  q0 = p
  F  = { p }
}
```

```delta-fails
// expect: E201
#mode strict
dfa M {
  Q = { p, q }
  Σ = { a, b }
  δ { p -a-> q }
  q0 = p
  F  = { q }
}
```

```delta-fails
// expect: W405
regex R over { a, b } = a+b
check R accepts "ab", "aab"
```

## 7. Unicode symbols and ASCII spellings

| Symbol | ASCII | Where |
|---|---|---|
| `Σ` | `Sigma` | field name; regex atom (any symbol) |
| `δ` | `delta` | field name |
| `ε` | `eps` | NFA arrow labels, table header, regexes |
| `∅` | `empty` | empty set (F, table cells), empty-language regex |
| `∪` | `\|` | regex union |
| `→` | `->` | arrows: `p -a-> q` or `p -a→ q` |
| `∘` | (juxtaposition) | explicit concatenation |
| `q0` | `start` | start-state field |
| `F` | `accept` | accept-states field |

ASCII-only version of an NFA (equivalent to writing the Unicode forms):

```delta
nfa A {
  Q = { p, q }
  Sigma = { a, b }
  delta {
    p -eps-> q
    q -a-> q
  }
  start = p
  accept = { q }
}

regex E over { a, b } = (a | b)*b | empty

check A accepts "", "a", "aaa"
check A rejects "b", "ab", "aab"
check E accepts "b", "ab", "aab"
check E rejects "", "a", "ba"
```
