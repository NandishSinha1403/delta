# Delta for AI models (short version)

Delta writes finite automata and regular expressions exactly as Sipser's *Introduction to the
Theory of Computation* (3rd ed.) defines them. Every code block here is tested in CI. The full
guide, with recipes and 27 worked examples, is `docs/DELTA-FOR-AI.md`.

## Rules for your answer

1. Output one ```` ```delta ```` block, then a table of what each state means.
2. Follow Sipser exactly; cite the Definition/Theorem (DFA Def 1.5, NFA Def 1.37, regex Def 1.52,
   NFA→DFA Thm 1.39, regex→NFA Lemma 1.55, DFA→regex Lemma 1.60, union/∩ Thm 1.25,
   NFA union/concat/star Thms 1.45/1.47/1.49).
3. For a construction use the command, not a hand copy: `nfa(R)`, `dfa(N)`, `gnfa(D)`,
   `regex(G)`, `union`, `intersect`, `concat`, `star`. Never minimize or simplify unless asked.
4. Always write `check X accepts` and `check X rejects` lines (≥ 5 strings each, including `""`)
   and at least one `run` line. Trace every check string by hand before answering.
5. Exam `+` between expressions means union: write `∪` (`(0+1)*` → `(0 ∪ 1)*`). In Delta `R+` is
   one-or-more. `λ`/`e` → `ε`; `φ` → `∅`. `010*` is 0·1·0*, not (010)*.

## Syntax

```text
dfa NAME {                         nfa NAME {
  Q  = { q1, q2 }                    Q = { p, q }
  Σ  = { 0, 1 }                      Σ = { a, b }
  δ  = table {                       δ {
         | 0  | 1                      p -[a,b]-> p      // two arrows
      q1 | q1 | q2                     p -a-> q
      q2 | q1 | q2                     q -ε-> p          // ε only in an NFA
  }                                  }
  q0 = q1                            q0 = p
  F  = { q2 }                        F  = { q }
}                                  }

δ(q1, 0) = q2          one arrow per line (DFA); δ(p, a) = { p, q } in an NFA
regex R over { a, b } = (a ∪ b)*abb      ∪ (or |), juxtaposition = concat, * + ^k, ε ∅ Σ ( )
let N = nfa(R)         let D = dfa(N)    let G = gnfa(D)    let R2 = regex(G)
let D2 = dfa(N, full: true)              let R3 = regex(G, order: [2, 1], mode: simplified)
let U = union(A, B)    let I = intersect(D1, D2)    let C = concat(A, B)    let S = star(A)
check X accepts "", "ab"      check X rejects "b"      run X on "abb"      #mode strict
// line comment   /* block comment */   multi-character symbol: <RESET>
ASCII: eps = ε, empty = ∅, | = ∪, -> = →, Sigma = Σ, delta = δ, start = q0, accept = F
```

Not available yet (skipped with info E123): `show`, `layout`, `gnfa G { … }` declarations,
`minimize`, `complement`, `reverse`, `equivalent`, context-free grammars, PDAs, Turing machines.

## Errors to avoid

| Code | Rule |
|---|---|
| W201 / E201 | a DFA's δ must be total: one arrow per state and symbol (write the dead state) |
| E202 / E203 / E204 | DFA: two arrows on one symbol / an ε-arrow / a set of targets |
| E104 / E106 / E107 | a state not in Q (in δ / as start / in F) |
| E105 / E301 / R901 | a symbol not in Σ (DFA arrow / NFA arrow / check string) |
| E108 | ε listed in Σ |
| E114 | multi-character symbol written bare: use `<ab>` |
| W405 | `a+b` read as one-or-more: write `a ∪ b` for union |
| E801 | wrong type: `dfa(D)` on a DFA, `intersect` on NFAs, `union` on a regex |
| R910 | a `check` failed: trace it and fix the machine |

## Example 1: a DFA from a description

"Strings over {0, 1} that end in 01." Remember the longest suffix that is a prefix of 01.

```delta
// s0: no progress; s1: ends in 0; s2: ends in 01 (accept).
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
check Ends01 rejects "", "0", "1", "10", "011"
run Ends01 on "1101"
```

## Example 2: regex → NFA → DFA

"NFA for (0+1)*010*, then a DFA." `+` is union here. `nfa(R)` is the book's construction: 13
states q0 … q12 numbered left to right, start q0, F = {q10, q12}.

```delta
regex R over { 0, 1 } = (0 ∪ 1)* 0 1 0*

let N = nfa(R)   // Lemma 1.55
let D = dfa(N)   // Theorem 1.39

check N accepts "01", "010", "0010", "1101000", "1001"
check N rejects "", "0", "10", "0110", "011"
check D accepts "01", "010"
check D rejects "", "0110"
run N on "1101000"
```

## Example 3: DFA → regular expression

"Regular expression for the DFA 1 -a→ 1, 1 -b→ 2, 2 -a,b→ 2, F = {2}" (Example 1.66).

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

let G = gnfa(M)                                    // Lemma 1.60, special form
let R = regex(G, order: [2, 1], mode: simplified)  // a*b(a∪b)*

check R accepts "b", "ab", "ba", "aabba", "bbb"
check R rejects "", "a", "aa", "aaa", "aaaa"
run M on "aabba"
```
