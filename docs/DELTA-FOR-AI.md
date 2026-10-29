# Delta: the complete guide for AI models

Delta is a small language for finite automata and regular expressions, written exactly as Michael
Sipser's *Introduction to the Theory of Computation* (3rd ed.) defines them, and run in the Delta
playground at `delta.nandish.dev`. This guide teaches an AI model to turn a request such as
"make an NFA for (0+1)*010*" or "DFA for strings with an even number of 0s" into correct, valid
Delta code, built the way the book builds it.

**This guide is tested.** Every ```` ```delta ```` block below is run by the test suite
(`crates/delta-dsl/src/guide_tests.rs`, in CI): it must have no errors or warnings, all of its
`check` lines must pass, and it must follow the rule in §0 (at least 5 accepted and 5 rejected
strings including `""`, and a `run` line). Blocks marked ```` ```delta-fails ```` are deliberate
mistakes; the test checks that Delta reports the listed error code. The syntax here was taken from
the actual parser, not from a plan: if something is not in this guide, assume Delta does not
support it yet.

A short version for small context windows is in `docs/DELTA-FOR-AI-SHORT.md`.

## Contents

- [0. Instructions to the AI](#0-instructions-to-the-ai)
- [1. Notation translation](#1-notation-translation)
- [2. Syntax reference](#2-syntax-reference)
  - [2.1 Program structure](#21-program-structure)
  - [2.2 DFA](#22-dfa-definition-15)
  - [2.3 NFA](#23-nfa-definition-137)
  - [2.4 Regular expressions](#24-regular-expressions-definition-152)
  - [2.5 GNFA](#25-gnfa-definition-164)
  - [2.6 Commands and functions](#26-commands-and-functions)
  - [2.7 Symbols, strings and spellings](#27-symbols-strings-and-spellings)
  - [2.8 Not available yet](#28-not-available-yet)
- [3. Rules that cause errors](#3-rules-that-cause-errors)
- [4. Design recipes](#4-design-recipes)
  - [4.1 DFA recipes](#41-dfa-recipes)
  - [4.2 NFA recipes](#42-nfa-recipes)
  - [4.3 Regular expression → NFA](#43-regular-expression--nfa-lemma-155)
  - [4.4 NFA → DFA](#44-nfa--dfa-theorem-139)
  - [4.5 DFA → GNFA → regular expression](#45-dfa--gnfa--regular-expression-lemma-160)
- [5. Worked exam-style examples](#5-worked-exam-style-examples)
- [6. Common mistakes and self-checks](#6-common-mistakes-and-self-checks)
- [7. Response template](#7-response-template)

---

## 0. Instructions to the AI

You write Delta code. Follow these rules in every answer.

1. **Output one ```` ```delta ```` block**, then a short explanation of what each state means
   (see the template in §7).
2. **Follow Sipser 3rd edition exactly**, and cite the Definition, Theorem or Lemma you used
   (DFA: Definition 1.5; NFA: Definition 1.37; regular expression: Definition 1.52; GNFA:
   Definition 1.64; constructions: Theorems 1.25, 1.39, 1.45, 1.47, 1.49, Lemmas 1.55 and 1.60).
3. **When the user asks for a construction, use the Delta command**, not a hand-built machine:
   | Asked for | Write |
   |---|---|
   | regex → NFA | `regex R over {…} = …` then `let N = nfa(R)` |
   | NFA → DFA | `let D = dfa(N)` |
   | DFA/NFA → GNFA | `let G = gnfa(D)` |
   | DFA/NFA/GNFA → regex | `let R = regex(G)` (or `regex(D)`) |
   | union / intersection of DFAs | `union(D1, D2)` / `intersect(D1, D2)` (Theorem 1.25) |
   | union / concatenation / star of NFAs | `union(N1, N2)`, `concat(N1, N2)`, `star(N1)` |

   Build a machine by hand only when the user explicitly asks for a hand-built one, or when they
   describe a language in words (then design a DFA or NFA with the recipes in §4).
4. **Never minimize or simplify unless asked.** `nfa(R)` is already the book's construction; do
   not remove ε-arrows or merge states. Use `regex(…, mode: simplified)` only if the user wants a
   shorter expression; the default `faithful` mode is the book's.
5. **Always include `check … accepts` and `check … rejects` lines**: at least 5 strings each,
   including `""` (ε) on the correct side and edge cases (single symbols, the shortest string in
   the language, strings that almost match). **Include at least one `run` line.**
6. **Before answering, trace every check string through your machine** by hand, state by state.
   If a trace disagrees with your check line, fix the machine (or the check), not the trace.
7. If the request uses exam notation (`+` for union, `λ` for ε, `φ` for ∅), translate it with §1
   and say so in one sentence.

---

## 1. Notation translation

Exam papers, other textbooks and Sipser write the same things differently. Translate to Sipser,
then to Delta.

| Seen in a question | Meaning | Sipser | Delta |
|---|---|---|---|
| `a + b`, `a \| b` (as an operator between expressions) | union | a ∪ b | `a ∪ b` or `a \| b` |
| `a+` (superscript or trailing plus) | one or more | a⁺ = aa* | `a+` |
| `ε`, `λ`, `e`, `Λ` | empty string | ε | `ε` or `eps` |
| `φ`, `Ø`, `{}` (as a language) | empty language | ∅ | `∅` or `empty` |
| `a·b`, `ab`, `a∘b` | concatenation | ab | `ab` or `a∘b` |
| `a*` | star | a* | `a*` |
| `aᵏ`, `a^k` | k copies | aᵏ | `a^k` (k ≤ 64) |
| `Σ` in an expression | any one symbol | Σ | `Σ` or `Sigma` |
| `Σ*` | all strings | Σ* | `Σ*` |
| `→`, `->` | arrow / transition | | `p -a-> q` or `p -a→ q` |
| `δ(q, a) = r` | transition | δ(q, a) = r | `δ(q, a) = r` |

**The most common trap: `+`.** In many exam papers `(0+1)*` means "any string of 0s and 1s".
In Sipser, and in Delta, `+` is *one or more*. So `(0+1)*` must be written `(0 ∪ 1)*`. Delta
warns (W405) when it sees `a+b` written tightly, because that is almost always meant as union.

**Precedence (tightest first):** `*`, `+`, `^k` → concatenation → `∪`.

- `010*` means `0 · 1 · (0*)`, **not** `(010)*`.
- `ab ∪ c` means `(ab) ∪ c`.
- `ab*` means `a(b*)`; to star both, write `(ab)*`.
- Every character in a regex is one symbol: `ab` is a ∘ b. Spaces are ignored inside a regex, so
  `0 1 0*` = `010*`.

**ASCII spellings** (Delta accepts both forms everywhere):

| Unicode | ASCII | Unicode | ASCII |
|---|---|---|---|
| `ε` | `eps` | `Σ` | `Sigma` |
| `∅` | `empty` | `δ` | `delta` |
| `∪` | `\|` | `q0` | `start` |
| `→` | `->` | `F` | `accept` |
| `∘` | (just write the symbols next to each other) | `⊔` | `_` is an ordinary symbol (Turing machines are not in Delta yet) |

---

## 2. Syntax reference

### 2.1 Program structure

A Delta program is a list of statements, one per line (machine bodies span several lines):

- `dfa NAME { … }` and `nfa NAME { … }` declare machines.
- `regex NAME over { … } = EXPRESSION` declares a regular expression.
- `let NAME = function(arguments)` converts or combines objects.
- `check NAME accepts "…", …` and `check NAME rejects "…", …` test any object.
- `run NAME on "…"` records a step-by-step run of a DFA or NFA.
- `#mode strict` (first line) turns lenient warnings into errors (see §3).

Comments are `// to the end of the line` and `/* block */`. Names start with a letter and may
contain letters, digits, `_` and `'` (`M1`, `q0'`, `even_a`). State names may also be numbers.

```delta
// A complete program: one machine, its tests, and a run.
/* Strings over {a, b} that end in b. */
dfa EndsB {
  Q  = { s, t }
  Σ  = { a, b }
  δ  = table {
         | a | b
       s | s | t
       t | s | t
  }
  q0 = s
  F  = { t }
}

check EndsB accepts "b", "ab", "bb", "aab", "abab"
check EndsB rejects "", "a", "ba", "bba", "abba"
run EndsB on "abab"
```

### 2.2 DFA (Definition 1.5)

> A finite automaton is a 5-tuple (Q, Σ, δ, q₀, F), where Q is a finite set of states, Σ is the
> alphabet, δ : Q × Σ → Q is the transition function, q₀ ∈ Q is the start state, and F ⊆ Q is the
> set of accept states.

The fields `Q`, `Σ`, `δ`, `q0`, `F` may come in any order. **δ must be total**: exactly one arrow
for every state and every symbol. `F = q2` (one state, no braces) and `F = ∅` are allowed.

There are three ways to write δ, and they may be mixed in one machine.

**Table form** (the book's table, Example 1.7). The header row starts with `|` and lists the
symbols; each row is `state | target | target …`.

```delta
// Figure 1.6, M1: at least one 1, and an even number of 0s after the last 1.
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

check M1 accepts "1", "01", "11", "100", "0101010101", "110000"
check M1 rejects "", "0", "10", "1000", "101000"
run M1 on "1101"
```

**Arrow form**: `from -symbol-> to`, one arrow per line; `p -[0,1]-> q` is two arrows.

```delta
// Example 1.9, M2: strings that end in 1.
dfa M2 {
  Q = { q1, q2 }
  Σ = { 0, 1 }
  δ {
    q1 -0-> q1
    q1 -1-> q2
    q2 -0-> q1
    q2 -1-> q2
  }
  q0 = q1
  F  = { q2 }
}

check M2 accepts "1", "01", "11", "0101", "1001"
check M2 rejects "", "0", "10", "110", "0110"
run M2 on "0101"
```

**Function form**: one `δ(state, symbol) = state` per line.

```delta
// Strings over {0, 1} with an even number of 0s.
dfa Even0 {
  Q = { even, odd }
  Σ = { 0, 1 }
  δ(even, 0) = odd
  δ(even, 1) = even
  δ(odd, 0)  = even
  δ(odd, 1)  = odd
  q0 = even
  F  = { even }
}

check Even0 accepts "", "1", "00", "0110", "1001"
check Even0 rejects "0", "10", "01", "000", "1101111"
run Even0 on "0110"
```

**Partial DFAs.** If some arrows are missing, Delta (in its default lenient mode) adds a trap
state and warns (W201); in `#mode strict` it is an error (E201). If you *mean* the missing arrows
to go to an implicit dead state, say so with `partial`, which silences the warning. The book
always draws total DFAs, so prefer writing the dead state yourself (§4.1).

```delta
// "partial": missing arrows go to an implicit trap state (drawn dashed).
dfa StartsAB partial {
  Q = { s, a1, ok }
  Σ = { a, b }
  δ {
    s  -a-> a1
    a1 -b-> ok
    ok -[a,b]-> ok
  }
  q0 = s
  F  = { ok }
}

check StartsAB accepts "ab", "aba", "abb", "abab", "abba"
check StartsAB rejects "", "a", "b", "ba", "aab"
run StartsAB on "abb"
```

### 2.3 NFA (Definition 1.37)

> A nondeterministic finite automaton is a 5-tuple (Q, Σ, δ, q₀, F), where δ : Q × Σ_ε → P(Q).

An NFA may have ε-arrows, several arrows on the same symbol from one state, and missing arrows
(they mean δ(q, a) = ∅). All three δ forms work; in a table, a cell is a set `{…}` or `∅`, and the
header may have an `ε` column. In the function form write `δ(q, a) = { … }`.

```delta
// Example 1.38, N1: contains 101 or 11 as a substring.
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

check N1 accepts "11", "101", "010110", "0011", "1010"
check N1 rejects "", "0", "10", "100100", "1001"
run N1 on "010110"
```

```delta
// The same kind of NFA as a table with set-valued cells and an ε column.
nfa EndsA {
  Q = { p, q }
  Σ = { a, b }
  δ = table {
      | a      | b   | ε
    p | {p, q} | {p} | ∅
    q | ∅      | ∅   | ∅
  }
  q0 = p
  F  = { q }
}

check EndsA accepts "a", "aa", "ba", "aba", "bba"
check EndsA rejects "", "b", "ab", "abb", "bab"
run EndsA on "bba"
```

```delta
// Function form with sets, and an ε-arrow: strings that contain at least one 1
// (guess the last 1, then read only 0s).
nfa Fn {
  Q = { p, q, r }
  Σ = { 0, 1 }
  δ(p, 0) = { p }
  δ(p, 1) = { p, q }
  δ(q, ε) = { r }
  δ(r, 0) = { r }
  q0 = p
  F  = { r }
}

check Fn accepts "1", "10", "100", "0110", "1000"
check Fn accepts "101", "1001"
check Fn rejects "", "0", "00", "000", "0000"
run Fn on "0110"
```

### 2.4 Regular expressions (Definition 1.52)

> R is a regular expression if R is a for some a in Σ, ε, ∅, (R₁ ∪ R₂), (R₁ ∘ R₂), or (R₁*).

```text
regex NAME over { SYMBOLS } = EXPRESSION
regex NAME = EXPRESSION            (Σ = the symbols the expression uses)
```

| Write | Means |
|---|---|
| `a` | the one-symbol string a |
| `ε` / `eps` | the empty string |
| `∅` / `empty` | the empty language |
| `R ∪ S` / `R \| S` | union |
| `RS` / `R ∘ S` | concatenation |
| `R*` | star |
| `R+` | one or more, R⁺ = RR* |
| `R^k` | k copies (k ≤ 64; `R^0` = ε) |
| `Σ` / `Sigma` | any one symbol of the alphabet |
| `( … )` | grouping |

Always write `over { … }` when the language's alphabet has symbols the expression never uses
(for example Σ = {0, 1} but the expression is `0*`). Without it, Σ is only the symbols that appear.

```delta
// Example 1.53-style expressions.
regex R1 over { 0, 1 } = 0*10*
regex R2 over { 0, 1 } = Σ*1Σ*
regex R3 over { 0, 1 } = (ΣΣ)*
regex R4 over { 0, 1 } = 1*(01+)*
regex R5 over { 0, 1 } = (0 ∪ ε)(1 ∪ ε)
regex R6 over { 0, 1 } = ∅          // (1*∅ is also ∅, but Delta warns W406: R∅ = ∅)

let N1 = nfa(R1)

check R1 accepts "1", "01", "10", "0010", "00100"
check R1 rejects "", "0", "11", "0110", "101"
check R2 accepts "1", "01", "10", "111", "0010"
check R2 rejects "", "0", "00", "000", "0000"
check R3 accepts "", "00", "01", "1010", "111111"
check R3 rejects "0", "1", "010", "11111", "00000"
check R4 accepts "", "1", "01", "011", "0101", "1101"
check R4 rejects "0", "00", "10", "100", "0110"
check R5 accepts "", "0", "1", "01"
check R5 rejects "10", "00", "11", "001", "011"
check R6 rejects "", "1", "11", "0", "10"
run N1 on "0010"
```

### 2.5 GNFA (Definition 1.64)

A GNFA is an NFA whose arrows are labelled by regular expressions, in the **special form** of
Lemma 1.60: one start state with arrows to every other state and no arrows in; one accept state
with arrows in from every other state and no arrows out; one arrow between every other pair.

**Delta cannot declare a GNFA directly** (`gnfa G { … }` is skipped with info E123). Make one from
a DFA or NFA with `gnfa(…)`, then turn it into a regular expression with `regex(…)`.

```delta
// Example 1.66: a two-state DFA → GNFA (special form) → regular expression.
dfa M {
  Q = { 1, 2 }
  Σ = { a, b }
  δ {
    1 -a-> 1
    1 -b-> 2
    2 -[a,b]-> 2
  }
  q0 = 1
  F  = { 2 }
}

let G = gnfa(M)                                   // Lemma 1.60: add qstart and qaccept
let R = regex(G, order: [2, 1], mode: simplified) // the book's answer: a*b(a∪b)*

check R accepts "b", "ab", "ba", "aabba", "bbb"
check R rejects "", "a", "aa", "aaa", "aaaa"
check G accepts "b", "ab", "ba", "aabba", "bbb"
check G rejects "", "a", "aa", "aaa", "aaaa"
run M on "aabba"
```

### 2.6 Commands and functions

| Statement | Meaning |
|---|---|
| `let X = dfa(N)` | Theorem 1.39, subset construction (reachable subsets). `N` may also be a regex (Lemma 1.55 first). Option `full: true` builds all 2ⁿ subsets (Figure 1.43). |
| `let X = nfa(R)` | Lemma 1.55 for a regex; for a DFA, the same machine viewed as an NFA (p. 47). |
| `let X = gnfa(M)` | Lemma 1.60's GNFA in special form, from a DFA or NFA. |
| `let X = regex(G)` | Lemma 1.60, CONVERT(G), from a GNFA, DFA or NFA. Options: `mode: faithful` (default) or `mode: simplified`; `order: [q2, q1]` sets the rip order. |
| `let X = union(A, B)` | Two DFAs: product construction, Theorem 1.25. With an NFA: Theorem 1.45. |
| `let X = intersect(A, B)` | Two DFAs: product with F = F₁ × F₂ (Theorem 1.25 footnote). |
| `let X = concat(A, B)` | Theorem 1.47 (DFA arguments are used as NFAs). |
| `let X = star(A)` | Theorem 1.49 (new accepting start state). |
| `check X accepts "w", …` | Each string must be in L(X). Works on any object. |
| `check X rejects "w", …` | Each string must not be in L(X). |
| `run X on "w"` | Records a step-by-step run of a DFA or NFA (shown in the playground). |

Calls nest: `let D = dfa(star(N))`. Arguments are names or calls. When two machines have
different alphabets, `union`, `intersect` and `concat` work over Σ₁ ∪ Σ₂ and warn (W807).

```delta
// Every command at once.
regex R over { a, b } = (a ∪ b)*b
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

let N  = nfa(R)               // Lemma 1.55
let D  = dfa(N)               // Theorem 1.39
let G  = gnfa(D)              // Lemma 1.60
let R2 = regex(G)             // Lemma 1.60, CONVERT(G), faithful
let U  = union(D, EvenA)      // Theorem 1.25, product
let I  = intersect(D, EvenA)  // Theorem 1.25 footnote
let C  = concat(N, N)         // Theorem 1.47
let S  = star(N)              // Theorem 1.49
let DS = dfa(star(N))         // calls nest

check D accepts "b", "ab", "bb", "aab", "abab"
check D rejects "", "a", "ba", "aba", "bba"
check R2 accepts "b", "ab", "bb", "aab", "abab"
check R2 rejects "", "a", "ba", "aba", "bba"
check U accepts "", "b", "aa", "ab", "aba"
check U rejects "a", "ba", "aaa", "bba", "aaaaa"
check I accepts "b", "aab", "abab", "bb", "baab"
check I rejects "", "ab", "a", "aa", "bab"
check C accepts "bb", "abb", "bab", "abab", "aabb"
check C rejects "", "b", "ab", "ba", "bba"
check S accepts "", "b", "bb", "ab", "abab"
check S rejects "a", "ba", "aa", "bba", "aba"
check DS accepts "", "b", "abab"
check DS rejects "a", "ba"
run N on "aab"
run D on "aab"
```

### 2.7 Symbols, strings and spellings

- A symbol is one character: `0`, `a`, `#`, `$`. For a multi-character symbol write `<NAME>`
  both in Σ and in strings: `Σ = { <RESET>, 0, 1 }`, `"0<RESET>1"` (Example 1.13).
- Strings are written in double quotes; `""` is ε. Every symbol of a string must be in Σ (R901).
- `ε` may label NFA arrows and appear in regexes, but never inside Σ (E108) or a DFA (E203).

```delta
// Example 1.13: the sum of the numbers read, mod 3, reset by <RESET>; accept when it is 0.
dfa Sum3 {
  Q = { q0, q1, q2 }
  Σ = { <RESET>, 0, 1, 2 }
  δ = table {
        | <RESET> | 0  | 1  | 2
     q0 | q0      | q0 | q1 | q2
     q1 | q0      | q1 | q2 | q0
     q2 | q0      | q2 | q0 | q1
  }
  q0 = q0
  F  = { q0 }
}

check Sum3 accepts "", "0", "12", "111", "2<RESET>", "11<RESET>0"
check Sum3 rejects "1", "2", "11", "<RESET>1", "12<RESET>2"
run Sum3 on "12<RESET>2"
```

### 2.8 Not available yet

These parse but are **skipped** with info E123 (they belong to later phases). Do not use them:
`show`, `layout { }`, `gnfa G { … }` declarations, `cfg`, `pda`, `tm`, `minimize(…)`,
`complement(…)`, `reverse(…)`, `equivalent(…)`, `check empty(…)` and other decision procedures.
For a complement, build it by hand (§4.1, "complement").

---
## 3. Rules that cause errors

Delta checks every machine against its definition. Each diagnostic has a code; **E** is an error
(the object is not built), **W** a warning, **I** information, **R** a runtime outcome. Delta runs
in **lenient** mode by default; `#mode strict` on the first line turns the lenient warnings W117,
W201, I115 and I116 into errors. Each example below is a deliberate mistake, and the test suite
checks that Delta reports the code shown.

### 3.1 Rules for every machine

| Code | Rule | Fix |
|---|---|---|
| E101 | (strict) a field is missing, e.g. no `q0` | write the field |
| W117 | (lenient) `q0` missing, so the first state of Q is used | write `q0 = …` |
| I115 / I116 | (lenient) Σ / Q missing, so it is inferred from δ | write Σ and Q explicitly |
| E102 | a field is written twice | keep one |
| E103 | Σ is empty ("an alphabet is any nonempty finite set", §0.2) | list the symbols |
| E104 | a state used in δ, q0 or F is not in Q | add it to Q (or fix the typo) |
| E106 | the start state is not in Q | fix q0 |
| E107 | F contains a state not in Q | fix F or Q |
| E108 | `ε` listed in Σ (ε is the empty *string*, not a symbol) | remove it |
| E109 | a set lists the same element twice | remove the duplicate |
| E110 | a state is named with a reserved word (`eps`, `empty`, `start`, `accept`, …) | rename it |
| E114 | a multi-character symbol written bare (`ab`) | write `<ab>` |
| E120 / E121 / E122 | syntax: a bad statement, a missing `}`, an unknown field | follow the message |
| W111 | a state is unreachable from the start state | remove it or add an arrow |
| W112 | F is empty, so the machine accepts nothing (legal, p. 36) | usually a mistake |

```delta-fails
// expect: E104
dfa M {
  Q = { q1, q2 }
  Σ = { 0 }
  δ {
    q1 -0-> q3
    q2 -0-> q2
  }
  q0 = q1
  F  = { q2 }
}
```

```delta-fails
// expect: E107
dfa M {
  Q = { q1 }
  Σ = { 0 }
  δ { q1 -0-> q1 }
  q0 = q1
  F  = { q9 }
}
```

```delta-fails
// expect: E108
nfa N {
  Q = { p }
  Σ = { a, ε }
  δ { p -a-> p }
  q0 = p
  F  = { p }
}
```

```delta-fails
// expect: E109
dfa M {
  Q = { p, p }
  Σ = { a }
  δ { p -a-> p }
  q0 = p
  F  = { p }
}
```

```delta-fails
// expect: E101
#mode strict
dfa M {
  Q = { p }
  Σ = { a }
  δ { p -a-> p }
  F = { p }
}
```

```delta-fails
// expect: W111
dfa M {
  Q = { p, lost }
  Σ = { a }
  δ {
    p -a-> p
    lost -a-> p
  }
  q0 = p
  F  = { p }
}
```

### 3.2 DFA rules (Definition 1.5: δ : Q × Σ → Q)

| Code | Rule | Fix |
|---|---|---|
| W201 / E201 | **δ must be total**: "exactly one transition arrow exits every state for each possible input symbol" (p. 36). Lenient: warning + implicit trap state. Strict: error. | add the missing arrows, usually to a dead state |
| E202 | two arrows from the same state on the same symbol: δ is a function | fix the arrow, or use `nfa` |
| E203 | an ε-arrow in a DFA: a DFA's δ takes Σ, not Σ_ε | remove it, or use `nfa` |
| E204 | a set of targets `{…}` in a DFA | one target only, or use `nfa` |
| E205 | a δ table with the wrong number of cells | one cell per symbol |
| E105 | an arrow on a symbol not in Σ | add the symbol to Σ or fix the arrow |

```delta-fails
// expect: W201
dfa M {
  Q = { p, q }
  Σ = { a, b }
  δ {
    p -a-> q
    q -[a,b]-> q
  }
  q0 = p
  F  = { q }
}
```

```delta-fails
// expect: E201
#mode strict
dfa M {
  Q = { p, q }
  Σ = { a, b }
  δ {
    p -a-> q
    q -[a,b]-> q
  }
  q0 = p
  F  = { q }
}
```

```delta-fails
// expect: E202
dfa M {
  Q = { p, q }
  Σ = { a }
  δ {
    p -a-> p
    p -a-> q
    q -a-> q
  }
  q0 = p
  F  = { q }
}
```

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
// expect: E204
dfa M {
  Q = { p, q }
  Σ = { a }
  δ(p, a) = { p, q }
  δ(q, a) = q
  q0 = p
  F  = { q }
}
```

```delta-fails
// expect: E205
dfa M {
  Q = { p }
  Σ = { a, b }
  δ = table {
       | a | b
     p | p
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
  δ {
    p -a-> p
    p -b-> p
  }
  q0 = p
  F  = { p }
}
```

### 3.3 NFA rules (Definition 1.37: δ : Q × Σ_ε → P(Q))

| Code | Rule | Fix |
|---|---|---|
| E301 | an arrow label that is neither in Σ nor ε | add it to Σ |
| W302 | the same arrow written twice (δ is a set, so it has no effect) | remove the copy |
| I304 | the NFA is deterministic and total, so it is really a DFA | consider `dfa` |

Missing arrows are fine in an NFA: δ(q, a) = ∅ means that branch dies.

```delta-fails
// expect: E301
nfa N {
  Q = { p }
  Σ = { a }
  δ { p -c-> p }
  q0 = p
  F  = { p }
}
```

```delta-fails
// expect: W302
nfa N {
  Q = { p, q }
  Σ = { a }
  δ {
    p -a-> q
    p -a-> q
  }
  q0 = p
  F  = { q }
}
```

### 3.4 Regular-expression rules (Definition 1.52)

| Code | Rule | Fix |
|---|---|---|
| E401 | unbalanced parentheses | add the missing `(` or `)` |
| E402 | an operator with a missing operand (`a ∪`, `* a`, `()`) | complete it; `()` is not ε, write `ε` |
| E403 | a symbol not in the `over { … }` alphabet | add it to the alphabet |
| E404 | `R^k` with k missing or above 64 | use a number ≤ 64 |
| W405 | `a+b` written tightly: in Sipser `+` means one-or-more, not union | write `a ∪ b` (or `a+ b` if you meant a⁺b) |
| W406 | concatenation with ∅ makes the term ∅ (R∅ = ∅, Example 1.53) | probably a mistake |
| E408 | nested more than 400 levels | simplify the expression |

```delta-fails
// expect: E401
regex R over { a, b } = (a ∪ b*
```

```delta-fails
// expect: E402
regex R over { a, b } = a ∪
```

```delta-fails
// expect: E403
regex R over { a, b } = a*c
```

```delta-fails
// expect: W405
regex R over { 0, 1 } = (0+1)*
```

```delta-fails
// expect: W406
regex R over { 0, 1 } = 0∅1
```

### 3.5 GNFA rules (Definition 1.64 and the special form, p. 70)

Delta builds GNFAs only through `gnfa(…)`, which always produces the special form: a new start
state with no arrows in, a new accept state with no arrows out, start ≠ accept, and exactly one
arrow (∅ when absent, hidden in the diagram) between every other pair. So there are no GNFA
errors to make. `regex(G, order: [...])` may only list states other than q_start and q_accept
(E805).

```delta-fails
// expect: E805
dfa M {
  Q = { p, q }
  Σ = { a }
  δ {
    p -a-> q
    q -a-> p
  }
  q0 = p
  F  = { q }
}
let G = gnfa(M)
let R = regex(G, order: [qstart])
```

### 3.6 Command and runtime rules

| Code | Rule | Fix |
|---|---|---|
| E801 | wrong argument type: `dfa(D)` on a DFA, `nfa(N)` on an NFA, `intersect` on NFAs, `union` on a regex | use the right function (§2.6), e.g. `intersect(dfa(N1), dfa(N2))` |
| E802 | no object with that name (with a suggestion) | fix the name |
| E803 | a name defined twice | rename one |
| E804 | an unknown option, e.g. `regex(G, ordr: …)` | `dfa`: `full`; `regex`: `order`, `mode` |
| E805 | a bad rip order | only inner states of the GNFA |
| E806 | `dfa(N, full: true)` on a large NFA | drop `full: true` |
| E809 | an object defined in terms of itself | |
| W807 | the alphabets of a union/intersection/concatenation differ | intended? the result is over Σ₁ ∪ Σ₂ |
| I811 | skipped because an object it uses has errors | fix the first error |
| R901 | a check/run string uses a symbol not in Σ | strings are over Σ |
| R902 | a construction ran out of its state budget (10 000) | use a smaller machine |
| R910 | a `check` failed | trace the string; fix the machine or the check |
| E123 (info) | a feature from a later phase (`show`, `layout`, `minimize`, …) was skipped | do not use it (§2.8) |

```delta-fails
// expect: E801
dfa D {
  Q = { p }
  Σ = { a }
  δ { p -a-> p }
  q0 = p
  F  = { p }
}
let X = dfa(D)
```

```delta-fails
// expect: E802
regex R over { a } = a*
let N = nfa(S)
```

```delta-fails
// expect: R901
regex R over { a, b } = a*
check R rejects "abc"
```

```delta-fails
// expect: R910
regex R over { a, b } = a*
check R accepts "b"
```

---
## 4. Design recipes

Sipser's advice (p. 41, "Designing finite automata"): **put yourself in the place of the machine**.
You read the input one symbol at a time and cannot go back, and you have finitely many states of
memory. Ask: *what is the least I must remember about the part read so far to decide later?* Each
possibility is one state. Then fill δ by asking, for every state and every symbol, "what do I
remember now?". Finally mark as accepting the states whose memory means "yes".

Write the meaning of every state as a comment. A DFA needs **every** arrow (§3.2); the state that
means "it can never be accepted any more" is the **dead state** (also called a trap).

### 4.1 DFA recipes

#### Ends with a string w

**Method.** Remember the longest suffix of the input that is a prefix of w (as in the KMP
algorithm). With w = 01: states "nothing useful" (s0), "just read 0" (s1), "just read 01" (s2).
On a symbol that breaks the match, fall back to the longest prefix of w that is still a suffix.

| State | Meaning |
|---|---|
| s0 | the input does not end in a prefix of 01 (it is ε or ends in 1 but not 01) |
| s1 | the input ends in 0 |
| s2 | the input ends in 01 (accept) |

```delta
dfa Ends01 {
  Q = { s0, s1, s2 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     s0 | s1 | s0
     s1 | s1 | s2
     s2 | s1 | s0
  }
  q0 = s0
  F  = { s2 }
}

check Ends01 accepts "01", "001", "101", "1101", "0101"
check Ends01 rejects "", "0", "1", "10", "011", "0110"
run Ends01 on "1101"
```

#### Starts with a string w

**Method.** Follow w symbol by symbol from the start; any mismatch goes to a dead state; after
the whole of w, stay in an accepting state forever.

| State | Meaning |
|---|---|
| s | nothing read yet |
| a1 | read "a" so far (a prefix of ab) |
| ok | the input starts with ab (accept, absorbing) |
| dead | the input cannot start with ab |

```delta
dfa StartsAB {
  Q = { s, a1, ok, dead }
  Σ = { a, b }
  δ = table {
          | a    | b
        s | a1   | dead
       a1 | dead | ok
       ok | ok   | ok
     dead | dead | dead
  }
  q0 = s
  F  = { ok }
}

check StartsAB accepts "ab", "aba", "abb", "abab", "abbb"
check StartsAB rejects "", "a", "b", "ba", "aab", "bab"
run StartsAB on "abab"
```

#### Contains a substring w

**Method.** Like "ends with", but once all of w has been seen, stay accepting (the last state is
absorbing). For w = aba the states remember the longest suffix that is a prefix of aba.

| State | Meaning |
|---|---|
| p0 | no progress |
| p1 | ends in a |
| p2 | ends in ab |
| p3 | aba has occurred (accept, absorbing) |

Note the fallbacks: from p1 on a stay in p1 (the new a starts a match); from p2 on b go to p0
("abb" has no useful suffix).

```delta
dfa HasABA {
  Q = { p0, p1, p2, p3 }
  Σ = { a, b }
  δ = table {
        | a  | b
     p0 | p1 | p0
     p1 | p1 | p2
     p2 | p3 | p0
     p3 | p3 | p3
  }
  q0 = p0
  F  = { p3 }
}

check HasABA accepts "aba", "aaba", "abab", "babab", "abaa", "bbabab"
check HasABA rejects "", "a", "ab", "abb", "bbaab", "baab"
run HasABA on "aabab"
```

#### Exactly, at least, at most k occurrences of a symbol

**Method.** Count the symbol up to k + 1 (or up to k when "at least"), ignoring the others.

| State | Meaning |
|---|---|
| c0, c1, c2 | 0, 1, 2 ones read so far |
| c3 | more than two ones read (dead for "exactly" and "at most") |

```delta
// Exactly two 1s: accept c2; c3 is dead.
dfa Exactly2 {
  Q = { c0, c1, c2, c3 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     c0 | c0 | c1
     c1 | c1 | c2
     c2 | c2 | c3
     c3 | c3 | c3
  }
  q0 = c0
  F  = { c2 }
}

// At least two 1s: c2 is absorbing.
dfa AtLeast2 {
  Q = { c0, c1, c2 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     c0 | c0 | c1
     c1 | c1 | c2
     c2 | c2 | c2
  }
  q0 = c0
  F  = { c2 }
}

// At most two 1s: accept c0, c1, c2.
dfa AtMost2 {
  Q = { c0, c1, c2, c3 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     c0 | c0 | c1
     c1 | c1 | c2
     c2 | c2 | c3
     c3 | c3 | c3
  }
  q0 = c0
  F  = { c0, c1, c2 }
}

check Exactly2 accepts "11", "011", "101", "110", "0101", "1001"
check Exactly2 rejects "", "0", "1", "111", "0111", "1011"
check AtLeast2 accepts "11", "101", "111", "0110", "11111"
check AtLeast2 rejects "", "0", "1", "10", "0001"
check AtMost2 accepts "", "0", "1", "11", "0101"
check AtMost2 rejects "111", "0111", "1011", "1101", "11111"
run Exactly2 on "0101"
```

#### Even or odd number of a symbol

**Method.** Remember only the parity: two states that swap on the symbol and ignore the rest.

```delta
// Odd number of a's over {a, b}.
dfa OddA {
  Q = { even, odd }
  Σ = { a, b }
  δ = table {
          | a    | b
     even | odd  | even
      odd | even | odd
  }
  q0 = even
  F  = { odd }
}

check OddA accepts "a", "ab", "ba", "aaa", "bab"
check OddA rejects "", "b", "aa", "abab", "bbbb"
run OddA on "babab"
```

#### Two counts at once (product of parities)

**Method.** Remember both facts: one state per *pair* (parity of 0s, parity of 1s). This is the
product construction of Theorem 1.25 done by hand: four states, each symbol flips one coordinate.

| State | 0s | 1s |
|---|---|---|
| ee | even | even |
| eo | even | odd (accept: "even 0s and odd 1s") |
| oe | odd | even |
| oo | odd | odd |

```delta
dfa Even0Odd1 {
  Q = { ee, eo, oe, oo }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     ee | oe | eo
     eo | oo | ee
     oe | ee | oo
     oo | eo | oe
  }
  q0 = ee
  F  = { eo }
}

check Even0Odd1 accepts "1", "001", "100", "010", "111", "11100"
check Even0Odd1 rejects "", "0", "11", "01", "0011", "000"
run Even0Odd1 on "11100"
```

#### Length mod n

**Method.** n states r0 … r(n−1) in a cycle; every symbol moves one step. Accept the remainders
asked for.

```delta
// |w| is a multiple of 3, over {a, b}.
dfa Len3 {
  Q = { r0, r1, r2 }
  Σ = { a, b }
  δ {
    r0 -[a,b]-> r1
    r1 -[a,b]-> r2
    r2 -[a,b]-> r0
  }
  q0 = r0
  F  = { r0 }
}

check Len3 accepts "", "aaa", "abb", "bab", "ababab"
check Len3 rejects "a", "ab", "bbbb", "aaaaa", "b"
run Len3 on "ababab"
```

#### Binary numbers divisible by n (state = remainder)

**Method.** Reading a binary number most significant bit first, the value v becomes 2v + b on bit
b. So remember only v mod n: from remainder r on bit b go to (2r + b) mod n. Accept remainder 0.
By this convention the empty string has value 0 and is accepted; say so in the answer (if the
question excludes ε, intersect with "at least one symbol" or add a separate start state).

| State | Remainder | on 0 | on 1 |
|---|---|---|---|
| r0 | 0 (accept) | r0 | r1 |
| r1 | 1 | r2 | r0 |
| r2 | 2 | r1 | r2 |

```delta
// Binary numbers divisible by 3, most significant bit first.
dfa Div3 {
  Q = { r0, r1, r2 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     r0 | r0 | r1
     r1 | r2 | r0
     r2 | r1 | r2
  }
  q0 = r0
  F  = { r0 }
}

check Div3 accepts "", "0", "11", "110", "1001", "1100"
check Div3 rejects "1", "10", "100", "101", "111"
run Div3 on "1001"
```

#### No two consecutive equal symbols of a kind ("no 11")

**Method.** Remember whether the last symbol was the dangerous one; a second one in a row goes to
the dead state.

| State | Meaning |
|---|---|
| s | the last symbol was not 1 (or nothing read) |
| t | the last symbol was 1 |
| dead | 11 has occurred |

```delta
dfa No11 {
  Q = { s, t, dead }
  Σ = { 0, 1 }
  δ = table {
          | 0    | 1
        s | s    | t
        t | s    | dead
     dead | dead | dead
  }
  q0 = s
  F  = { s, t }
}

check No11 accepts "", "0", "1", "010", "10101", "0100"
check No11 rejects "11", "011", "110", "0110", "1011"
run No11 on "10101"
```

#### Every a is immediately followed by b

**Method.** After an a you *owe* a b: a waiting state that is not accepting; a second a while
waiting is fatal.

| State | Meaning |
|---|---|
| ok | every a so far was followed by b (accept) |
| owe | the last symbol was an a still waiting for its b |
| dead | an a was followed by another a (can never be fixed) |

```delta
dfa AThenB {
  Q = { ok, owe, dead }
  Σ = { a, b }
  δ = table {
          | a    | b
       ok | owe  | ok
      owe | dead | ok
     dead | dead | dead
  }
  q0 = ok
  F  = { ok }
}

check AThenB accepts "", "b", "ab", "abab", "bab", "abbab"
check AThenB rejects "a", "aa", "ba", "aab", "abba"
run AThenB on "abbab"
```

#### Complement

**Method (p. 85, "the class of regular languages is closed under complement").** Make sure the DFA
is **complete** (total δ, dead state written out), then swap accepting and non-accepting states.
This only works for a DFA: swapping F in an NFA does *not* complement it. Delta has no
`complement(…)` yet (§2.8), so write the swapped machine.

```delta
// Strings over {a, b} that do NOT contain aba: HasABA with F swapped.
dfa NoABA {
  Q = { p0, p1, p2, p3 }
  Σ = { a, b }
  δ = table {
        | a  | b
     p0 | p1 | p0
     p1 | p1 | p2
     p2 | p3 | p0
     p3 | p3 | p3
  }
  q0 = p0
  F  = { p0, p1, p2 }
}

check NoABA accepts "", "a", "ab", "abb", "bbaab", "baab"
check NoABA rejects "aba", "aaba", "abab", "babab", "abaa"
run NoABA on "abba"
```

#### Union and intersection: the product construction (Theorem 1.25)

**Method.** Run both DFAs at once: states are pairs (r₁, r₂), the start is (q₁, q₂), and
δ((r₁, r₂), a) = (δ₁(r₁, a), δ₂(r₂, a)). For union F = (F₁ × Q₂) ∪ (Q₁ × F₂); for intersection
(the footnote) F = F₁ × F₂. **Use the commands** `union(D1, D2)` and `intersect(D1, D2)`: they
build exactly this, keeping only reachable pairs.

```delta
dfa EvenZeros {
  Q = { e, o }
  Σ = { 0, 1 }
  δ = table {
       | 0 | 1
     e | o | e
     o | e | o
  }
  q0 = e
  F  = { e }
}

dfa EndsIn01 {
  Q = { s0, s1, s2 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     s0 | s1 | s0
     s1 | s1 | s2
     s2 | s1 | s0
  }
  q0 = s0
  F  = { s2 }
}

let U = union(EvenZeros, EndsIn01)      // Theorem 1.25
let I = intersect(EvenZeros, EndsIn01)  // Theorem 1.25, footnote

check U accepts "", "01", "00", "1", "0101", "0110"
check U rejects "0", "10", "011", "000", "1110", "0111"
check I accepts "001", "1001", "10101", "11001", "100001"
check I rejects "", "01", "00", "0001", "1", "0110"
run U on "0101"
```

### 4.2 NFA recipes

An NFA may **guess**. Think of every possible guess running in parallel as a "finger" on a state
(p. 48); the NFA accepts if some finger ends on an accept state. Missing arrows simply kill a
finger, so an NFA needs no dead state.

#### Guess the position: "ends with w" and "the k-th symbol from the end"

**Method.** Loop in the start state on every symbol, and **guess** the moment where the
interesting part begins; then check it with a chain of states.

```delta
// Ends with 01: guess that the final 01 starts now.
nfa Ends01N {
  Q = { q0, q1, q2 }
  Σ = { 0, 1 }
  δ {
    q0 -[0,1]-> q0
    q0 -0-> q1
    q1 -1-> q2
  }
  q0 = q0
  F  = { q2 }
}

// The third symbol from the end is 1 (Example 1.30's language with k = 3): guess that this 1
// is it, then exactly two more symbols must follow. The equivalent DFA needs 2³ = 8 states.
nfa Third {
  Q = { q0, q1, q2, q3 }
  Σ = { 0, 1 }
  δ {
    q0 -[0,1]-> q0
    q0 -1-> q1
    q1 -[0,1]-> q2
    q2 -[0,1]-> q3
  }
  q0 = q0
  F  = { q3 }
}

let D = dfa(Third)   // Theorem 1.39: 8 reachable subsets

check Ends01N accepts "01", "001", "101", "1101", "0101"
check Ends01N rejects "", "0", "1", "10", "011"
check Third accepts "100", "111", "0100", "1100", "01101"
check Third rejects "", "1", "10", "000", "011", "1011"
check D accepts "100", "111", "0100"
check D rejects "", "011"
run Third on "01101"
```

#### Union by a new start state (Theorem 1.45)

**Method.** Q = {q0} ∪ Q₁ ∪ Q₂ with a **new** start q0 and ε-arrows from q0 to both old starts;
F = F₁ ∪ F₂. **Use `union(N1, N2)`.** Delta names the new start q0 and primes N₂'s states where
names clash (q1 → q1′), because the proof assumes Q₁ and Q₂ are disjoint. (On two DFAs,
`union` uses the product construction of Theorem 1.25 instead, which keeps the result a DFA.)

```delta
// N1: contains 11.  N2: odd length.  N = N1 ∪ N2.
nfa N1 {
  Q = { q1, q2, q3 }
  Σ = { 0, 1 }
  δ {
    q1 -[0,1]-> q1
    q1 -1-> q2
    q2 -1-> q3
    q3 -[0,1]-> q3
  }
  q0 = q1
  F  = { q3 }
}

nfa N2 {
  Q = { q1, q2 }
  Σ = { 0, 1 }
  δ {
    q1 -[0,1]-> q2
    q2 -[0,1]-> q1
  }
  q0 = q1
  F  = { q2 }
}

let N = union(N1, N2)   // Theorem 1.45: new start q0, ε-arrows to q1 and q1'

check N accepts "0", "11", "011", "0110", "10101"
check N rejects "", "00", "01", "1010", "0101"
run N on "0110"
```

#### Concatenation with ε-arrows (Theorem 1.47)

**Method.** Q = Q₁ ∪ Q₂; the start is N₁'s start; add ε-arrows from **every** accept state of N₁
to N₂'s start; F = F₂ (N₁'s accept states stop accepting). **Use `concat(N1, N2)`.**

```delta
// N1: exactly "1".  N2: odd length.  N1 ∘ N2: a 1 followed by an odd-length string.
nfa One {
  Q = { q1, q2 }
  Σ = { 0, 1 }
  δ { q1 -1-> q2 }
  q0 = q1
  F  = { q2 }
}

nfa Odd {
  Q = { r1, r2 }
  Σ = { 0, 1 }
  δ {
    r1 -[0,1]-> r2
    r2 -[0,1]-> r1
  }
  q0 = r1
  F  = { r2 }
}

let C = concat(One, Odd)   // Theorem 1.47

check C accepts "10", "11", "1000", "1011", "111111"
check C rejects "", "1", "0", "01", "100", "11111"
run C on "1011"
```

#### Star with a new accepting start state (Theorem 1.49), and the Exercise 1.15 pitfall

**Method.** Q = {q0} ∪ Q₁ with a **new** start q0 that **is accepting** (so ε is in L*), an
ε-arrow q0 → q₁, and ε-arrows from every accept state of N₁ back to q₁; F = {q0} ∪ F₁.
**Use `star(N1)`.**

**The pitfall (Exercise 1.15).** "Just make the old start state accepting and add ε-arrows from
F₁ back to it" is wrong when the old start state has arrows coming back into it: strings that
only loop at the start become accepted. The example below shows the wrong machine accepting "0".

```delta
// N1 accepts (0*1)+: strings ending in 1. Its start q1 has a loop on 0.
nfa N1 {
  Q = { q1, q2 }
  Σ = { 0, 1 }
  δ {
    q1 -0-> q1
    q1 -1-> q2
    q2 -ε-> q1
  }
  q0 = q1
  F  = { q2 }
}

let S = star(N1)   // Theorem 1.49: L(S) = ε plus the strings ending in 1

check S accepts "", "1", "01", "0011", "11"
check S rejects "0", "00", "10", "110", "100"
run S on "0011"
```

```delta-fails
// expect: R910
// WRONG (Exercise 1.15): the old start q1 made accepting instead of adding a new start.
nfa Wrong {
  Q = { q1, q2 }
  Σ = { 0, 1 }
  δ {
    q1 -0-> q1
    q1 -1-> q2
    q2 -ε-> q1
  }
  q0 = q1
  F  = { q1, q2 }
}
// L(N1)* does not contain "0", but this machine accepts it:
check Wrong rejects "0"
```

### 4.3 Regular expression → NFA (Lemma 1.55)

**Method.** Write the regular expression (translate exam notation first, §1), then **use
`nfa(R)`**. Delta builds exactly the book's NFA by structural recursion over Definition 1.52:

1. `a`: two states q -a→ q′, accepting q′.
2. `ε`: one accepting state.
3. `∅`: one non-accepting state.
4. `R₁ ∪ R₂`: Theorem 1.45 (new start, ε-arrows to both).
5. `R₁R₂`: Theorem 1.47 (ε-arrows from F₁ to R₂'s start).
6. `R₁*`: Theorem 1.49 (new accepting start, ε-arrow in, ε-arrows back from F₁).

No ε-arrows are removed and no states are merged or minimized. States are numbered q0, q1, …
**left to right along the expression**: the new start of a union or star gets its number before
its operands. Concatenations and unions of more than two parts are built left to right.

**Worked by hand: `(0 ∪ 1)* 0 1 0*`.** (An exam would write `(0+1)*010*`; `+` is union here.)
The parts, in order: the star `(0∪1)*` (new start q0) containing the union `0∪1` (new start q1)
of `0` (q2 -0→ q3) and `1` (q4 -1→ q5); then `0` (q6 -0→ q7); then `1` (q8 -1→ q9); then the star
`0*` (new start q10) of `0` (q11 -0→ q12). The steps:

| Step | Rule | What it adds |
|---|---|---|
| 1 | Lemma 1.55 (1), `0` | q2 -0→ q3 |
| 2 | Lemma 1.55 (1), `1` | q4 -1→ q5 |
| 3 | Theorem 1.45, `0∪1` | new start q1; q1 -ε→ q2, q1 -ε→ q4; F = {q3, q5} |
| 4 | Theorem 1.49, `(0∪1)*` | new accepting start q0; q0 -ε→ q1; q3 -ε→ q1, q5 -ε→ q1; F = {q0, q3, q5} |
| 5 | Lemma 1.55 (1), `0` | q6 -0→ q7 |
| 6 | Theorem 1.47, `(0∪1)*0` | q0, q3, q5 -ε→ q6; F = {q7} |
| 7 | Lemma 1.55 (1), `1` | q8 -1→ q9 |
| 8 | Theorem 1.47, `(0∪1)*01` | q7 -ε→ q8; F = {q9} |
| 9 | Lemma 1.55 (1), `0` | q11 -0→ q12 |
| 10 | Theorem 1.49, `0*` | new accepting start q10; q10 -ε→ q11; q12 -ε→ q11; F = {q10, q12} |
| 11 | Theorem 1.47, `(0∪1)*010*` | q9 -ε→ q10; F = {q10, q12} |

The result has 13 states q0 … q12, start q0, F = {q10, q12}, 12 ε-arrows and 5 symbol arrows.
Below, the same NFA is written by hand **and** produced by `nfa(R)`; both pass the same checks.
In an answer, write only the `regex` and `nfa(R)` lines unless the user asked for the hand-built
machine.

```delta
regex R over { 0, 1 } = (0 ∪ 1)* 0 1 0*

let N = nfa(R)   // Lemma 1.55: exactly the machine below

// The same NFA built by hand, arrow for arrow.
nfa Hand {
  Q = { q0, q1, q2, q3, q4, q5, q6, q7, q8, q9, q10, q11, q12 }
  Σ = { 0, 1 }
  δ {
    q0 -ε-> q1
    q1 -ε-> q2
    q1 -ε-> q4
    q2 -0-> q3
    q4 -1-> q5
    q3 -ε-> q1
    q5 -ε-> q1
    q0 -ε-> q6
    q3 -ε-> q6
    q5 -ε-> q6
    q6 -0-> q7
    q7 -ε-> q8
    q8 -1-> q9
    q9 -ε-> q10
    q10 -ε-> q11
    q11 -0-> q12
    q12 -ε-> q11
  }
  q0 = q0
  F  = { q10, q12 }
}

check N accepts "01", "010", "0010", "1101000", "1001"
check N rejects "", "0", "10", "0110", "011"
check Hand accepts "01", "010", "0010", "1101000", "1001"
check Hand rejects "", "0", "10", "0110", "011"
run N on "1101000"
```

### 4.4 NFA → DFA (Theorem 1.39)

**Method.** **Use `dfa(N)`.** The subset construction: each DFA state is a *set* of NFA states,
the set of places a finger can be.

- **ε-closure.** E(R) = the states reachable from R along zero or more ε-arrows (p. 56).
- **Start state.** q₀′ = E({q₀}).
- **Transitions.** δ′(R, a) = E(⋃_{r ∈ R} δ(r, a)).
- **Accept states.** The sets that contain an accept state of N.
- **The ∅ state.** It is the dead state: once every finger has died, nothing can be accepted.

Delta builds only the subsets reachable from the start (Figure 1.44); `dfa(N, full: true)` builds
all 2ⁿ (Figure 1.43).

**Example 1.41.** N₄ has states 1, 2, 3, start 1, F = {1}, and an ε-arrow 1 → 3.

| DFA state | Why | on a | on b |
|---|---|---|---|
| {1,3} (start, accept) | E({1}) = {1,3} | δ(1,a) ∪ δ(3,a) = ∅ ∪ {1}, E = {1,3} | {2} |
| {2} | | {2,3} | {3} |
| {2,3} | | δ(2,a) ∪ δ(3,a) = {2,3} ∪ {1}, E = {1,2,3} | {3} |
| {3} | | {1,3} | ∅ |
| {1,2,3} (accept) | | {1,2,3} | {2,3} |
| ∅ | dead | ∅ | ∅ |

```delta
// Example 1.41: N4 and its DFA (Figure 1.44: six reachable subsets).
nfa N4 {
  Q = { 1, 2, 3 }
  Σ = { a, b }
  δ {
    1 -b-> 2
    1 -ε-> 3
    2 -a-> 2
    2 -a-> 3
    2 -b-> 3
    3 -a-> 1
  }
  q0 = 1
  F  = { 1 }
}

let D4     = dfa(N4)              // reachable subsets, Figure 1.44
let D4full = dfa(N4, full: true)  // all 2³ = 8 subsets, Figure 1.43

check D4 accepts "", "a", "aa", "baba", "baa"
check D4 rejects "b", "bb", "ba", "aba", "abb", "babba"
check D4full accepts "", "a", "baa"
check D4full rejects "b", "ba"
run D4 on "baba"
```

### 4.5 DFA → GNFA → regular expression (Lemma 1.60)

**Method.** **Use `gnfa(D)` then `regex(G)`** (or `regex(D)` directly). The proof of Lemma 1.60:

1. **Special form** (`gnfa`): add a new start state q_start with an ε-arrow to the old start, and
   a new accept state q_accept with ε-arrows from the old accept states (which stop accepting).
   Parallel arrows merge into one arrow labelled by their union; missing arrows are ∅.
2. **CONVERT(G)** (`regex`): while more than two states remain, **rip** a state q_rip. For every
   pair (qᵢ, qⱼ) the new label is

   > δ′(qᵢ, qⱼ) = (R₁)(R₂)*(R₃) ∪ (R₄),

   where R₁ = δ(qᵢ, q_rip), R₂ = δ(q_rip, q_rip), R₃ = δ(q_rip, qⱼ), R₄ = δ(qᵢ, qⱼ).
3. When only q_start and q_accept remain, the label between them is the answer.

The rip order changes the expression (not the language). `order: [2, 1]` rips 2 first. The
default `mode: faithful` writes the book's raw expression (with ε and ∅ terms); `mode: simplified`
applies the identities of p. 66 (R ∪ ∅ = R, Rε = R, ∅* = ε). Use faithful unless the user wants
the short form.

**Example 1.66.** States 1 (start) and 2 (accept): 1 -a→ 1, 1 -b→ 2, 2 -a,b→ 2.

- **Rip 2.** R₁ = b, R₂ = a∪b, R₃ = ε, R₄ = ∅, so δ′(1, q_accept) = b(a∪b)*.
- **Rip 1.** R₁ = ε, R₂ = a, R₃ = b(a∪b)*, R₄ = ∅, so δ′(q_start, q_accept) = **a*b(a∪b)***.

```delta
dfa M {
  Q = { 1, 2 }
  Σ = { a, b }
  δ {
    1 -a-> 1
    1 -b-> 2
    2 -[a,b]-> 2
  }
  q0 = 1
  F  = { 2 }
}

let G  = gnfa(M)                                     // Lemma 1.60, special form
let R  = regex(G, order: [2, 1], mode: simplified)   // a*b(a∪b)*, the book's answer
let Rf = regex(G, order: [2, 1])                     // the faithful, unsimplified expression

check R accepts "b", "ab", "ba", "aabba", "bbb"
check R rejects "", "a", "aa", "aaa", "aaaa"
check Rf accepts "b", "ab", "ba", "aabba", "bbb"
check Rf rejects "", "a", "aa", "aaa", "aaaa"
run M on "aabba"
```

---
## 5. Worked exam-style examples

Each example: the **question**, the **reasoning** (what each state remembers), the **Delta code**
with its checks, and the book reference. The first seven are the book's own examples.

### 5.1 Figure 1.6: M1

**Question.** Give a DFA for M1 = {w | w contains at least one 1 and an even number of 0s follow
the last 1} (Example 1.7).

**Reasoning.** q1: no 1 seen yet. q2: a 1 has been seen and an even number of 0s follow the last
1 (accept). q3: a 1 has been seen and an odd number of 0s follow the last 1. A 1 always returns
to q2; a 0 toggles between q2 and q3. (Definition 1.5.)

```delta
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

check M1 accepts "1", "01", "11", "0101010101", "100", "0100", "110000"
check M1 rejects "", "0", "10", "1000", "101000"
run M1 on "1101"
```

### 5.2 Example 1.38: N1, then its DFA

**Question.** N1 accepts the strings containing 101 or 11 as a substring. Write N1 and give the
equivalent DFA.

**Reasoning.** q1 loops on everything and guesses where 101 or 11 starts: on 1 it may go to q2.
From q2, a 0 continues "101" and an ε skips straight on for "11"; q3 needs the final 1; q4 is
accepting and absorbing. The DFA comes from the subset construction (Theorem 1.39), so use
`dfa(N1)`.

```delta
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

let D1 = dfa(N1)   // Theorem 1.39

check N1 accepts "11", "101", "010110", "0011", "1010"
check N1 rejects "", "0", "10", "100100", "1001"
check D1 accepts "11", "101", "010110", "0011", "1010"
check D1 rejects "", "0", "10", "100100", "1001"
run N1 on "010110"
```

### 5.3 Example 1.35: N3, a unary NFA with ε-arrows

**Question.** Over Σ = {0}, give an NFA for {0ᵏ | k is a multiple of 2 or of 3}.

**Reasoning.** The NFA guesses with ε-arrows which test to run: a 2-cycle counting mod 2 or a
3-cycle counting mod 3. In each cycle the "remainder 0" state accepts. ε (k = 0) is accepted
because 0 is a multiple of both.

```delta
nfa N3 {
  Q = { s, a0, a1, b0, b1, b2 }
  Σ = { 0 }
  δ {
    s -ε-> a0
    s -ε-> b0
    a0 -0-> a1
    a1 -0-> a0
    b0 -0-> b1
    b1 -0-> b2
    b2 -0-> b0
  }
  q0 = s
  F  = { a0, b0 }
}

check N3 accepts "", "00", "000", "0000", "000000"
check N3 rejects "0", "00000", "0000000", "00000000000", "0000000000000"
run N3 on "000"
```

### 5.4 Example 1.41: NFA → DFA

**Question.** Convert N4 to a DFA (Theorem 1.39).

**Reasoning.** See §4.4: the start state is E({1}) = {1,3}; six subsets are reachable; the
accepting ones are those containing 1: {1,3} and {1,2,3}; ∅ is the dead state.

```delta
nfa N4 {
  Q = { 1, 2, 3 }
  Σ = { a, b }
  δ {
    1 -b-> 2
    1 -ε-> 3
    2 -a-> 2
    2 -a-> 3
    2 -b-> 3
    3 -a-> 1
  }
  q0 = 1
  F  = { 1 }
}

let D4 = dfa(N4)   // Theorem 1.39, Figure 1.44

check D4 accepts "", "a", "aa", "baba", "baa"
check D4 rejects "b", "bb", "ba", "aba", "babba"
run D4 on "baa"
```

### 5.5 Example 1.56: (ab ∪ a)* → NFA

**Question.** Convert (ab ∪ a)* to an NFA (Lemma 1.55).

**Reasoning.** Use `nfa(R)`. The book's construction has 8 states: 2 for `a`, 2 for `b`, their
concatenation `ab` (no new state), 2 for the second `a`, a new start for the union, and a new
accepting start for the star.

```delta
regex R over { a, b } = (ab ∪ a)*

let N = nfa(R)   // Lemma 1.55: 8 states

check N accepts "", "a", "ab", "aab", "abab", "aaba"
check N rejects "b", "ba", "abb", "bb", "abba"
run N on "aab"
```

### 5.6 Example 1.58: (a ∪ b)*aba → NFA → DFA

**Question.** Convert (a ∪ b)*aba to an NFA, then to a DFA.

**Reasoning.** `nfa(R)` gives the book's 12-state NFA; `dfa(N)` the subset construction.

```delta
regex R over { a, b } = (a ∪ b)*aba

let N = nfa(R)   // Lemma 1.55: 12 states
let D = dfa(N)   // Theorem 1.39

check D accepts "aba", "aaba", "baba", "abaaba", "babaaba"
check D rejects "", "a", "ab", "abab", "abba"
run D on "babaaba"
```

### 5.7 Example 1.66: DFA → regular expression

**Question.** Convert the two-state DFA (1 -a→ 1, 1 -b→ 2, 2 -a,b→ 2; F = {2}) to a regular
expression by ripping state 2 and then state 1.

**Reasoning.** §4.5: rip 2 gives b(a∪b)*, then rip 1 gives a*b(a∪b)*: "at least one b".

```delta
dfa M {
  Q = { 1, 2 }
  Σ = { a, b }
  δ {
    1 -a-> 1
    1 -b-> 2
    2 -[a,b]-> 2
  }
  q0 = 1
  F  = { 2 }
}

let G = gnfa(M)
let R = regex(G, order: [2, 1], mode: simplified)   // a*b(a∪b)*

check R accepts "b", "ab", "ba", "aabba", "bbb"
check R rejects "", "a", "aa", "aaa", "aaaa"
run M on "aab"
```

### 5.8 Even number of a's and even number of b's

**Question.** Σ = {a, b}. DFA for strings with an even number of a's and an even number of b's.

**Reasoning.** Remember both parities (§4.1, product of parities). ee = (even a, even b) is the
start and the only accept state; a flips the first letter, b the second.

```delta
dfa EvenEven {
  Q = { ee, eo, oe, oo }
  Σ = { a, b }
  δ = table {
        | a  | b
     ee | oe | eo
     eo | oo | ee
     oe | ee | oo
     oo | eo | oe
  }
  q0 = ee
  F  = { ee }
}

check EvenEven accepts "", "aa", "bb", "abab", "aabb", "baab"
check EvenEven rejects "a", "b", "ab", "aab", "bbb"
run EvenEven on "abab"
```

### 5.9 Contains 00

**Question.** Σ = {0, 1}. DFA for strings containing 00 as a substring.

**Reasoning.** s0: the last symbol was not 0 (or nothing read). s1: the last symbol was 0.
s2: 00 has occurred (accept, absorbing).

```delta
dfa Has00 {
  Q = { s0, s1, s2 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     s0 | s1 | s0
     s1 | s2 | s0
     s2 | s2 | s2
  }
  q0 = s0
  F  = { s2 }
}

check Has00 accepts "00", "100", "001", "1001", "0100"
check Has00 rejects "", "0", "1", "010", "10101"
run Has00 on "1001"
```

### 5.10 Does not contain 00 (complement)

**Question.** Σ = {0, 1}. DFA for strings that do not contain 00.

**Reasoning.** The complement of 5.9: the DFA is complete (s2 is its dead state), so swap F.

```delta
dfa No00 {
  Q = { s0, s1, s2 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     s0 | s1 | s0
     s1 | s2 | s0
     s2 | s2 | s2
  }
  q0 = s0
  F  = { s0, s1 }
}

check No00 accepts "", "0", "1", "010", "10101"
check No00 rejects "00", "100", "001", "1001", "0100"
run No00 on "10101"
```

### 5.11 Binary numbers divisible by 5

**Question.** DFA accepting binary numbers (most significant bit first) divisible by 5.

**Reasoning.** State = value mod 5; on bit b go from r to (2r + b) mod 5 (§4.1). ε counts as 0.

| r | on 0 | on 1 |
|---|---|---|
| 0 | 0 | 1 |
| 1 | 2 | 3 |
| 2 | 4 | 0 |
| 3 | 1 | 2 |
| 4 | 3 | 4 |

```delta
dfa Div5 {
  Q = { r0, r1, r2, r3, r4 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
     r0 | r0 | r1
     r1 | r2 | r3
     r2 | r4 | r0
     r3 | r1 | r2
     r4 | r3 | r4
  }
  q0 = r0
  F  = { r0 }
}

check Div5 accepts "", "0", "101", "1010", "1111", "10100"
check Div5 rejects "1", "10", "11", "100", "110", "1001"
run Div5 on "1111"
```

### 5.12 Number of a's ≡ 1 (mod 3)

**Question.** Σ = {a, b}. Strings whose number of a's leaves remainder 1 when divided by 3.

**Reasoning.** m0, m1, m2 = number of a's mod 3; a moves round the cycle, b stays. Accept m1.

```delta
dfa AMod3 {
  Q = { m0, m1, m2 }
  Σ = { a, b }
  δ = table {
        | a  | b
     m0 | m1 | m0
     m1 | m2 | m1
     m2 | m0 | m2
  }
  q0 = m0
  F  = { m1 }
}

check AMod3 accepts "a", "ab", "ba", "aaaa", "bab"
check AMod3 rejects "", "b", "aa", "aaa", "abab"
run AMod3 on "aaaa"
```

### 5.13 Length at least 3

**Question.** Σ = {0, 1}. DFA for strings of length at least 3.

**Reasoning.** Count up to 3: l0, l1, l2, then l3 (accept, absorbing).

```delta
dfa Len3Plus {
  Q = { l0, l1, l2, l3 }
  Σ = { 0, 1 }
  δ {
    l0 -[0,1]-> l1
    l1 -[0,1]-> l2
    l2 -[0,1]-> l3
    l3 -[0,1]-> l3
  }
  q0 = l0
  F  = { l3 }
}

check Len3Plus accepts "000", "101", "111", "0000", "10101"
check Len3Plus rejects "", "0", "1", "01", "11"
run Len3Plus on "0000"
```

### 5.14 Begins and ends with the same symbol

**Question.** Σ = {a, b}. Strings that begin and end with the same symbol.

**Reasoning.** Remember the first symbol (it never changes) and whether the last symbol equals
it. a1: began with a, last is a (accept). a2: began with a, last is b. b1: began with b, last is
b (accept). b2: began with b, last is a. ε has no first symbol, so it is rejected (state it in the
answer; a one-symbol string such as "a" begins and ends with a, so it is accepted).

```delta
dfa SameEnds {
  Q = { s, a1, a2, b1, b2 }
  Σ = { a, b }
  δ = table {
        | a  | b
      s | a1 | b1
     a1 | a1 | a2
     a2 | a1 | a2
     b1 | b2 | b1
     b2 | b2 | b1
  }
  q0 = s
  F  = { a1, b1 }
}

check SameEnds accepts "a", "b", "aa", "aba", "bab", "abba"
check SameEnds rejects "", "ab", "ba", "aab", "bba"
run SameEnds on "abba"
```

### 5.15 At least one a and at least one b

**Question.** Σ = {a, b}. Strings containing at least one a and at least one b.

**Reasoning.** n: neither seen. A: only a's seen. B: only b's seen. both: accept, absorbing.

```delta
dfa BothAB {
  Q = { n, A, B, both }
  Σ = { a, b }
  δ = table {
          | a    | b
        n | A    | B
        A | A    | both
        B | both | B
     both | both | both
  }
  q0 = n
  F  = { both }
}

check BothAB accepts "ab", "ba", "aab", "bba", "abab"
check BothAB rejects "", "a", "b", "aaa", "bbbb"
run BothAB on "aab"
```

### 5.16 Contains 0101 or 111 (NFA)

**Question.** Σ = {0, 1}. Give an NFA for strings containing 0101 or 111, and the equivalent DFA.

**Reasoning.** Loop in s and guess which pattern starts here: a chain a1–a4 checks 0101, a chain
b1–b3 checks 111. The last state of each chain accepts and loops. Then `dfa(N)` (Theorem 1.39).

```delta
nfa N {
  Q = { s, a1, a2, a3, a4, b1, b2, b3 }
  Σ = { 0, 1 }
  δ {
    s -[0,1]-> s
    s -0-> a1
    a1 -1-> a2
    a2 -0-> a3
    a3 -1-> a4
    a4 -[0,1]-> a4
    s -1-> b1
    b1 -1-> b2
    b2 -1-> b3
    b3 -[0,1]-> b3
  }
  q0 = s
  F  = { a4, b3 }
}

let D = dfa(N)

check N accepts "111", "0101", "00101", "1110", "10101"
check N rejects "", "0", "11", "0110", "1001", "01001"
check D accepts "111", "0101", "00101", "1110", "10101"
check D rejects "", "0", "11", "0110", "1001", "01001"
run N on "00101"
```

### 5.17 The second-to-last symbol is a (NFA)

**Question.** Σ = {a, b}. NFA for strings whose second-to-last symbol is a.

**Reasoning.** Guess the position (§4.2): loop in q0; on a guess "this is it" and go to q1; one
more symbol of any kind reaches q2 (accept). No arrows leave q2, so the guess dies if the input
continues.

```delta
nfa SecondLastA {
  Q = { q0, q1, q2 }
  Σ = { a, b }
  δ {
    q0 -[a,b]-> q0
    q0 -a-> q1
    q1 -[a,b]-> q2
  }
  q0 = q0
  F  = { q2 }
}

check SecondLastA accepts "aa", "ab", "baa", "bab", "aab"
check SecondLastA rejects "", "a", "b", "ba", "abb", "bb"
run SecondLastA on "bab"
```

### 5.18 (a ∪ b)*abb: regex → NFA → DFA

**Question.** Construct an NFA for (a+b)*abb by the standard construction, then a DFA.

**Reasoning.** `+` is union in this exam notation (§1), so the expression is (a ∪ b)*abb. Use
`nfa(R)` (Lemma 1.55) and `dfa(N)` (Theorem 1.39); do not build or simplify by hand.

```delta
regex R over { a, b } = (a ∪ b)*abb

let N = nfa(R)   // Lemma 1.55
let D = dfa(N)   // Theorem 1.39

check D accepts "abb", "aabb", "babb", "ababb", "abbabb"
check D rejects "", "a", "ab", "abba", "bbb"
run N on "babb"
```

### 5.19 Exam notation: (0+1)*1(0+1)*

**Question.** Give an NFA for (0+1)*1(0+1)*.

**Reasoning.** Translate: (0 ∪ 1)*1(0 ∪ 1)*, "contains at least one 1". Writing `(0+1)` would be
read by Delta as 0⁺1 with warning W405; use ∪.

```delta
regex R over { 0, 1 } = (0 ∪ 1)*1(0 ∪ 1)*

let N = nfa(R)   // Lemma 1.55

check N accepts "1", "01", "10", "0110", "111"
check N rejects "", "0", "00", "000", "0000"
run N on "0110"
```

### 5.20 Exam notation: 1+0 as "one or more 1s, then 0"

**Question.** Give a regular expression for "one or more 1s followed by a 0".

**Reasoning.** In Sipser this is 1⁺0 = 11*0. In Delta write `1+ 0` (a space after `+` shows it is
not a union, and Delta does not warn) or `11*0`. If an exam's `1+0` meant "1 or 0", it is
`1 ∪ 0` instead: always say which reading you used.

```delta
regex R  over { 0, 1 } = 1+ 0
regex R2 over { 0, 1 } = 11*0

let N = nfa(R2)

check R accepts "10", "110", "1110", "11110", "111110"
check R rejects "", "0", "1", "100", "1101"
check R2 accepts "10", "110", "1110"
check R2 rejects "", "0", "100"
run N on "110"
```

### 5.21 DFA → regular expression: an even number of 0s

**Question.** Give a regular expression for the strings over {0, 1} with an even number of 0s,
by converting a DFA.

**Reasoning.** Build the two-state parity DFA, then `regex(…)` (Lemma 1.60). Ask for
`mode: simplified` only if a short expression is wanted; the result is equivalent to
1*(01*01*)*.

```delta
dfa Even0 {
  Q = { e, o }
  Σ = { 0, 1 }
  δ = table {
       | 0 | 1
     e | o | e
     o | e | o
  }
  q0 = e
  F  = { e }
}

let G = gnfa(Even0)                    // Lemma 1.60
let R = regex(G, mode: simplified)     // CONVERT(G)

check R accepts "", "1", "00", "0110", "1001"
check R rejects "0", "10", "01", "000", "1101111"
run Even0 on "1001"
```

### 5.22 Union by the product construction: even length or contains 11

**Question.** Using the product construction, give a DFA for strings of even length or containing
11.

**Reasoning.** Two small DFAs, then `union` (Theorem 1.25); the result's states are pairs.

```delta
dfa EvenLen {
  Q = { e, o }
  Σ = { 0, 1 }
  δ {
    e -[0,1]-> o
    o -[0,1]-> e
  }
  q0 = e
  F  = { e }
}

dfa Has11 {
  Q = { n, one, yes }
  Σ = { 0, 1 }
  δ = table {
         | 0   | 1
       n | n   | one
     one | n   | yes
     yes | yes | yes
  }
  q0 = n
  F  = { yes }
}

let U = union(EvenLen, Has11)   // Theorem 1.25

check U accepts "", "11", "00", "011", "0110", "111"
check U rejects "0", "1", "010", "10101", "00100"
run U on "011"
```

### 5.23 Intersection: starts with a and ends with b

**Question.** Σ = {a, b}. DFA for strings that start with a and end with b, as an intersection.

**Reasoning.** StartA: s → yes on a (absorbing accept), s → dead on b. EndB: remembers whether the
last symbol was b. Then `intersect` (Theorem 1.25 footnote, F = F₁ × F₂).

```delta
dfa StartA {
  Q = { s, yes, dead }
  Σ = { a, b }
  δ = table {
          | a    | b
        s | yes  | dead
      yes | yes  | yes
     dead | dead | dead
  }
  q0 = s
  F  = { yes }
}

dfa EndB {
  Q = { x, y }
  Σ = { a, b }
  δ = table {
       | a | b
     x | x | y
     y | x | y
  }
  q0 = x
  F  = { y }
}

let I = intersect(StartA, EndB)   // Theorem 1.25, footnote

check I accepts "ab", "aab", "abb", "abab", "aaab"
check I rejects "", "a", "b", "ba", "aba", "bab"
run I on "abab"
```

### 5.24 Concatenation: a* followed by b*

**Question.** Given NFAs for a* and b*, build an NFA for a*b* by Theorem 1.47.

**Reasoning.** `concat(As, Bs)` adds an ε-arrow from As's accept state to Bs's start. The regex
a*b* describes the same language.

```delta
nfa As {
  Q = { p }
  Σ = { a, b }
  δ { p -a-> p }
  q0 = p
  F  = { p }
}

nfa Bs {
  Q = { q }
  Σ = { a, b }
  δ { q -b-> q }
  q0 = q
  F  = { q }
}

let C = concat(As, Bs)   // Theorem 1.47
regex R over { a, b } = a*b*

check C accepts "", "a", "b", "ab", "aabbb"
check C rejects "ba", "aba", "bab", "bba", "abab"
check R accepts "", "a", "b", "ab", "aabbb"
check R rejects "ba", "aba", "bab", "bba", "abab"
run C on "aabbb"
```

### 5.25 Star: (ab)*

**Question.** Given an NFA for {ab}, build an NFA for (ab)* by Theorem 1.49.

**Reasoning.** `star(AB)` adds a new accepting start state (so ε is accepted) with an ε-arrow to
the old start, and ε-arrows from AB's accept state back to the old start.

```delta
nfa AB {
  Q = { p, q, r }
  Σ = { a, b }
  δ {
    p -a-> q
    q -b-> r
  }
  q0 = p
  F  = { r }
}

let S = star(AB)   // Theorem 1.49

check S accepts "", "ab", "abab", "ababab", "abababab"
check S rejects "a", "b", "ba", "aba", "abb"
run S on "abab"
```

### 5.26 Odd number of 1s and ends in 0

**Question.** Σ = {0, 1}. DFA for strings with an odd number of 1s that end in 0.

**Reasoning.** Remember the parity of 1s and whether the last symbol was 0: four states.
E (even, last not 0), E0 (even, last 0), O (odd, last 1), O0 (odd, last 0: accept).

```delta
dfa Odd1End0 {
  Q = { E, E0, O, O0 }
  Σ = { 0, 1 }
  δ = table {
        | 0  | 1
      E | E0 | O
     E0 | E0 | O
      O | O0 | E
     O0 | O0 | E
  }
  q0 = E
  F  = { O0 }
}

check Odd1End0 accepts "10", "100", "0010", "1110", "1000"
check Odd1End0 rejects "", "0", "1", "110", "101", "1100"
run Odd1End0 on "1110"
```

### 5.27 The third symbol is 1

**Question.** Σ = {0, 1}. DFA for strings whose third symbol is 1.

**Reasoning.** Count two symbols (s0, s1, s2), then look at the third: 1 goes to yes (accept,
absorbing), 0 goes to dead.

```delta
dfa Third1 {
  Q = { s0, s1, s2, yes, dead }
  Σ = { 0, 1 }
  δ = table {
          | 0    | 1
       s0 | s1   | s1
       s1 | s2   | s2
       s2 | dead | yes
      yes | yes  | yes
     dead | dead | dead
  }
  q0 = s0
  F  = { yes }
}

check Third1 accepts "001", "111", "0010", "1011", "00111"
check Third1 rejects "", "0", "00", "000", "110", "1100"
run Third1 on "1011"
```

---

## 6. Common mistakes and self-checks

**Partial δ in a DFA.** Every state needs an arrow on every symbol (Definition 1.5). Delta adds a
trap and warns (W201), but the book's answer writes the dead state. *Self-check:* count the arrows:
|Q| × |Σ|.

**Forgetting the dead state.** "Starts with ab": after "b" nothing can help. Without a dead state
the DFA is partial (W201); with an accepting state used as a sink, it accepts wrong strings.

**Wrong precedence.** `010*` is 0·1·0*, not (010)*. `ab ∪ c` is (ab) ∪ c. *Self-check:* check the
shortest strings: `010*` accepts "01" but not "010010".

**`+` as union.** `(0+1)*` from an exam is `(0 ∪ 1)*` in Delta. Delta warns W405.

```delta-fails
// expect: W405
regex R over { 0, 1 } = (0+1)*
```

**An NFA where a DFA was asked.** An ε-arrow, two arrows on one symbol, or a missing arrow make
it an NFA. If a DFA was asked, give the DFA (design one, or write the NFA and `dfa(N)`).

```delta-fails
// expect: E203
dfa NotReallyADfa {
  Q = { p, q }
  Σ = { a }
  δ {
    p -ε-> q
    p -a-> p
    q -a-> q
  }
  q0 = p
  F  = { q }
}
```

**Accepting ε by mistake.** If the start state is accepting, ε is accepted. "At least one 1" must
not have an accepting start. *Self-check:* always include `""` in a check line.

```delta-fails
// expect: R910
// "At least one a", but the start state was made accepting.
dfa AtLeastOneA {
  Q = { none, some }
  Σ = { a, b }
  δ = table {
          | a    | b
     none | some | none
     some | some | some
  }
  q0 = none
  F  = { none, some }
}
check AtLeastOneA rejects ""
```

**N₁'s accept states left accepting after concatenation.** In Theorem 1.47, F = F₂ only; N₁'s
accept states stop accepting. `concat(N1, N2)` does this for you; a hand-built version often
forgets.

```delta-fails
// expect: R910
// "a then b", built by hand, but q (N1's accept state) was left accepting.
nfa AThenB {
  Q = { p, q, r, s }
  Σ = { a, b }
  δ {
    p -a-> q
    q -ε-> r
    r -b-> s
  }
  q0 = p
  F  = { q, s }
}
check AThenB rejects "a"
```

**Star by making the old start accepting.** Exercise 1.15: wrong when the start state has
incoming arrows (§4.2). Use `star(N)`.

**Swapping F in an NFA to complement it.** Only works for a complete DFA (§4.1, complement).

**Symbols outside Σ.** Every arrow label and every check string must use symbols of Σ (E105,
E301, R901). Declare Σ before writing arrows.

**Multi-character symbols.** `ab` in Σ is two symbols' worth of text; write `<ab>` for one
symbol (E114).

**Reading `run` as `check`.** `run M on "w"` only records a run; it does not test anything. The
tests are the `check` lines.

**Before answering, run the self-check:**

1. Every state's meaning is written down, and the start state's meaning matches "nothing read".
2. A DFA has |Q| × |Σ| arrows; an NFA's guesses are explained.
3. ε is on the correct side of a check line.
4. The shortest accepted string and the longest "almost" string are checked.
5. Every check string has been traced by hand through the machine, state by state.
6. Constructions use `nfa(R)`, `dfa(N)`, `gnfa(D)`, `regex(G)`, `union`, `intersect`, `concat`,
   `star`, not hand copies, unless a hand-built machine was asked for.

---

## 7. Response template

Answer in exactly this shape.

```text
<one sentence: the language, any notation you translated (e.g. "+ read as ∪"), and the
Definition/Theorem/Lemma you used>

~~~delta
<the machine or construction, each state's meaning in a comment>

check NAME accepts "…", "…", "…", "…", "…"     (≥ 5, the shortest member first)
check NAME rejects "", "…", "…", "…", "…"      (≥ 5, ε on the correct side)
run NAME on "…"
~~~

| State | Meaning |
|---|---|
| q0 | … (start) |
| q1 | … (accept) |

Construction: <Definition 1.5 / Theorem 1.39 / Lemma 1.55 / …>, built by <command or by hand>.
```

A complete answer for "DFA for strings with an even number of 0s":

> The language of strings over {0, 1} with an even number of 0s; a DFA (Definition 1.5).

```delta
// e: an even number of 0s read so far (start, accept); o: an odd number.
dfa Even0 {
  Q = { e, o }
  Σ = { 0, 1 }
  δ = table {
       | 0 | 1
     e | o | e
     o | e | o
  }
  q0 = e
  F  = { e }
}

check Even0 accepts "", "1", "00", "0110", "1001"
check Even0 rejects "0", "10", "01", "000", "1101111"
run Even0 on "0110"
```

| State | Meaning |
|---|---|
| e | an even number of 0s so far (start, accept) |
| o | an odd number of 0s so far |

Construction: Definition 1.5, designed by hand (two states remember the parity of 0s).
