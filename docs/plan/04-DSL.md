# 04 — The DSL: "Code → Automaton"

The playground's language. Design goal: **the code *is* the formal tuple.** Someone who has read Definition 1.5 can read a Delta DFA without a manual, and someone who writes one has practised the definition.

File extension: `.delta`. Encoding: UTF-8. Every Unicode symbol has an ASCII spelling so it can be typed on any keyboard (and the playground has a symbol bar for phones).

---

## 1. Lexical basics

| Thing | Syntax |
|---|---|
| Comments | `// line`  `/* block */` |
| Identifiers (state/machine names) | `q1`, `q_accept`, `even`, `M1` — letters, digits, `_`, `'` (so `q0'` works like the book) |
| Symbols | single characters by default: `0`, `a`, `#`, `$`. Multi-character symbols in angle brackets: `<RESET>`, `<EXPR>` (the book writes ⟨RESET⟩) |
| Sets | `{ q1, q2, q3 }` |
| Strings (inputs) | `"0101"`, `""` (= ε); with multi-char symbols: `"10<RESET>22"` |

| Book | Unicode | ASCII |
|---|---|---|
| ε | `ε` | `eps` |
| ∅ | `∅` | `empty` |
| ∪ | `∪` | `\|` |
| Σ, Γ, δ | `Σ` `Γ` `δ` | `Sigma` `Gamma` `delta` |
| q₀ | `q0` | `start` |
| F | `F` | `accept` |
| → | `→` | `->` |
| ⊔ (blank) | `⊔` | `_` |

---

## 2. Finite automata

### 2.1 DFA — mirrors Definition 1.5 line by line
```delta
// Figure 1.6 — M1 accepts strings with at least one 1
// and an even number of 0s after the last 1
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
```

δ can be written in **three equivalent forms**, so users can pick whatever matches what they're copying from:

```delta
δ = table { ... }            // 1. the book's table (above)

δ {                          // 2. arrows, like a state diagram
  q1 -0-> q1
  q1 -1-> q2
  q2 -0-> q3
  q2 -1-> q2
  q3 -[0,1]-> q2             // several symbols on one arrow
}

δ(q1, 0) = q1                // 3. function notation, exactly as in the text
δ(q1, 1) = q2
```

**Inference (optional, convenient):** if `Q` or `Σ` is omitted, it is inferred from δ, and the inferred set is shown in the tuple panel. If `q0` is omitted, the first state mentioned is used, with a warning.

**Partial DFAs:** `dfa M partial { ... }` allows missing transitions. Without `partial`, a missing δ(q, a) is an **error** citing Definition 1.5, with the quick fix "complete with trap state".

### 2.2 NFA — Definition 1.37
```delta
// Example 1.38 — N1
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
```
Function form: `δ(q1, 1) = { q1, q2 }`, `δ(q2, ε) = { q3 }`. Table form uses set-valued cells and an `ε` column, exactly like Example 1.38.

### 2.3 GNFA — Definition 1.64 (usually produced by conversion, but can be written)
```delta
gnfa G {
  Σ = { a, b }
  start = s
  accept = a_
  δ {
    s  -[ε]->        1
    1  -[a]->        1
    1  -[b(a∪b)*]->  a_      // labels are regular expressions
  }
  // missing pairs are ∅ — special form is completed automatically
}
```

---

## 3. Regular expressions — Definition 1.52
```delta
regex R1 over { a, b } = (a ∪ b)* a b a
regex R2 over { 0, 1 } = Σ* 001 Σ*         // Σ shorthand (Example 1.53)
regex R3 over { 0, 1 } = 1* (0 1+)*        // + is "one or more" (R+ = RR*), NOT union
regex R4 over { 0, 1 } = (ΣΣ)^3            // R^k shorthand
regex R5 over { 0, 1 } = 0 ∪ ε             // union with ε
```
- Precedence: `*` `+` `^k` > concatenation > `∪`. Whitespace is ignored inside regexes. Concatenation is juxtaposition (`∘` is also accepted).
- `|` is accepted for `∪`. **`a+b` produces a warning**: "In Sipser, + means one-or-more (R⁺ = RR*). Did you mean a ∪ b?"
- `over {…}` is optional; it is inferred from the symbols used. Declare it when the expression doesn't mention every symbol (e.g. `∅` over {0,1}).

---

## 4. Context-free grammars — Definition 2.2
```delta
// Example 2.3 — G3, balanced parentheses (a = "(", b = ")")
cfg G3 {
  S -> aSb | SS | ε
}

// Example 2.4 — G4, arithmetic expressions
cfg G4 {
  <EXPR>   -> <EXPR> + <TERM> | <TERM>
  <TERM>   -> <TERM> x <FACTOR> | <FACTOR>
  <FACTOR> -> ( <EXPR> ) | a
}
```
**The book's conventions (p. 104):** variables = symbols appearing on a left-hand side; terminals = everything else; start = left-hand side of the first rule. Optional explicit form:
```delta
cfg G {
  V = { S, A, B }
  Σ = { a, b }
  S = S                      // start variable
  R {
    S -> ASA | aB
    A -> B | S
    B -> b | ε
  }
}
```
Inside rules, each character is one symbol unless wrapped in `< >`; spaces are ignored (`a S b` = `aSb`).

---

## 5. Pushdown automata — Definition 2.13
Edge label = the book's **`a, b → c`** (read a, pop b, push c; any may be ε):
```delta
// Example 2.14 — M1 recognizes { 0ⁿ1ⁿ | n ≥ 0 }
pda M1 {
  Q  = { q1, q2, q3, q4 }
  Σ  = { 0, 1 }
  Γ  = { 0, $ }
  δ {
    q1 -[ε, ε → $]-> q2
    q2 -[0, ε → 0]-> q2
    q2 -[1, 0 → ε]-> q3
    q3 -[1, 0 → ε]-> q3
    q3 -[ε, $ → ε]-> q4
  }
  q0 = q1
  F  = { q1, q4 }
}
```
- Several labels on one arrow: separate with `;` → `q -[a, ε → a; b, ε → b]-> q`.
- Push strings (the book's shorthand, p. 118): `q -[ε, A → aSb]-> q` pushes `aSb` with `a` on top. Strict mode flags it and offers "expand shorthand".
- `dpda M { ... }` additionally runs the Definition 2.39 check.
- Function form: `δ(q2, 0, ε) = { (q2, 0) }`.

---

## 6. Turing machines — Definition 3.3
Edge label = the book's diagram notation: **`a → b, R`**, shorthand **`a → R`** (write back the same symbol), several reads **`0,1 → R`**:
```delta
// Example 3.7 — M2 decides { 0^(2^n) | n ≥ 0 }
tm M2 {
  Q  = { q1, q2, q3, q4, q5, qaccept, qreject }
  Σ  = { 0 }
  Γ  = { 0, x, ⊔ }
  δ {
    q1 -[0 → ⊔, R]-> q2
    q1 -[⊔ → R]->    qreject
    q1 -[x → R]->    qreject
    q2 -[x → R]->    q2
    q2 -[⊔ → R]->    qaccept
    q2 -[0 → x, R]-> q3
    q3 -[x → R]->    q3
    q3 -[0 → R]->    q4
    q3 -[⊔ → L]->    q5
    q4 -[x → R]->    q4
    q4 -[0 → x, R]-> q3
    q4 -[⊔ → R]->    qreject
    q5 -[0,x → L]->  q5
    q5 -[⊔ → R]->    q2
  }
  q0 = q1
  accept = qaccept
  reject = qreject
}
```
- **Missing transitions go to `reject`, moving R** (book convention, p. 173). The diagram hides them by default, with a toggle to show them.
- Variants: `tm M stayput { … }` allows `S` moves; `tm M tapes 2 { q -[0,⊔ → 0,0, R,R]-> r }` multitape; `ntm M { … }` allows several transitions for the same (q, a); `lba M { … }` keeps the head inside the input.
- Function form: `δ(q1, 0) = (q2, ⊔, R)`.

*(Golden test, verified with a prototype: on `0000` this machine passes through exactly the book's 22 configurations, `q1 0000` … `⊔xxx⊔ qaccept`, and accepts. It accepts `0`, `00`, `00000000` and rejects ε, `000`, `00000`.)*

---

## 7. Commands — the pipeline

Top-level statements that build, convert, test and display objects. They run in order, results appear in the Run panel, and every named result can be shown on the canvas.

```delta
let N  = nfa(R1)                       // C3  Lemma 1.55
let D  = dfa(N)                        // C2  Theorem 1.39 (reachable subsets)
let Df = dfa(N, full: true)            // C2  all 2^k subsets, like Example 1.41
let M  = minimize(D)                   // C9  (method: hopcroft | table)
let G  = gnfa(M)                       // C4
let r  = regex(G, order: [q2, q1])     // C5  rip order optional; mode: faithful | simplified
let N2 = nfa(M1)                       // C1  DFA → NFA

let U  = union(D, M1)                  // C6 product (DFA)  — union(N, N2) uses Thm 1.45 (NFA)
let I  = intersect(D, M1)
let C  = complement(M1)                // auto-completes first, and says so
let Rv = reverse(M1)                   // Problem 1.31
let S  = star(N)                       // Thm 1.49;  concat(N, N2) Thm 1.47

let Gc = cnf(G4)                       // C12 Theorem 2.9 (order: book | fast)
let P  = pda(G3)                       // C15 Lemma 2.21
let H  = cfg(M1pda)                    // C16 Lemma 2.27 (prune: true)
let Gr = cfg(M1)                       // C11 DFA → CFG

// tests (Chapter 4 decision procedures)
check M1 accepts "1", "0100", "110000"
check M1 rejects "", "0", "1000"
check equivalent(D, r)                 // D5 EQ_DFA → counterexample if not
check empty(I)                         // D4 E_DFA
check G4 generates "a+axa"             // D6 via CNF + CYK
check is_dpda(M1pda)                   // C17

// simulation
run M2 on "0000" limit 10_000          // trace of configurations u q v
run N1 on "010110" tree                // NFA computation tree (Figure 1.29)
parse G4 "(a+a)xa"                     // parse tree(s) + CYK table

// analysis
pump M1 on "0100"                      // D9 regular pumping lemma split
enumerate D 20                         // first 20 strings of L(D) in shortlex order

// display
show D, M                              // pin to canvas (side by side)
export M tikz                          // also: dot, json, jff, svg, png
```

Options use `name: value` syntax. Unknown options are errors, with a list of valid ones.

---

## 8. Layout block (diagram positions live in the code)
```delta
layout M1 {
  q1 at (0, 0)
  q2 at (160, 0)
  q3 at (320, 0)
  q2 -> q3 bend 30          // optional curve
}
```
Written automatically when the user drags states on the canvas (code is the single source of truth). Without a layout block, Graphviz lays the diagram out.

---

## 9. Diagnostics

The full rule catalog (every rule for DFA, NFA, regex, GNFA, CFG, PDA/DPDA and TM, plus command and runtime errors), strict vs. lenient mode, the Rust/TypeScript representation and crash handling are specified in **[`08-ERRORS.md`](08-ERRORS.md)**.

A few examples of what users see:

| Code | Example message | Quick fix |
|---|---|---|
| E201 | `δ(q2, 1)` is undefined. Definition 1.5 requires δ : Q × Σ → Q to be total | add trap state / mark `partial` |
| E202 | DFA has two transitions for (q1, 0). A DFA's δ returns one state | change to `nfa` |
| W405 | `a+b`: in Sipser, + means one-or-more (R⁺ = RR*). Did you mean `a ∪ b`? | replace |
| E602 | `$` is pushed but not declared in Γ | add `$` to Γ |
| E701 | blank `⊔` is in Σ. Definition 3.3: Σ must not contain the blank | remove |
| R901 | input `"012"`: symbol `2` is not in Σ = {0, 1} | — |

## 10. Grammar of the DSL (EBNF, abridged)

```ebnf
program     = { item } ;
item        = machine | regexdecl | cfgdecl | letstmt | command | layout ;

machine     = kind ident [ "partial" | "stayput" | "tapes" int ] "{" { field } "}" ;
kind        = "dfa" | "nfa" | "gnfa" | "pda" | "dpda" | "tm" | "ntm" | "lba" ;
field       = setfield | deltafield | startfield | acceptfield | rejectfield ;
setfield    = ( "Q" | "Σ" | "Sigma" | "Γ" | "Gamma" ) "=" set ;
startfield  = ( "q0" | "start" ) "=" ident ;
acceptfield = ( "F" | "accept" ) "=" ( set | ident ) ;
rejectfield = "reject" "=" ident ;
deltafield  = ( "δ" | "delta" ) ( "=" table | "{" { edge } "}" )
            | ( "δ" | "delta" ) "(" args ")" "=" value ;
edge        = ident ( "-" sym "->" | "-[" labels "]->" ) ident ;
labels      = label { ";" label } ;
label       = symlist                                  (* DFA/NFA: 0,1 or ε *)
            | sym "," sym "→" symstring                (* PDA: a, b → c *)
            | symlist "→" [ sym "," ] move             (* TM: a → b, R | a → R *)
            | regex ;                                  (* GNFA *)
move        = "L" | "R" | "S" ;
table       = "table" "{" header { row } "}" ;

regexdecl   = "regex" ident [ "over" set ] "=" regex ;
regex       = term { ( "∪" | "|" ) term } ;
term        = factor { factor } ;
factor      = atom { "*" | "+" | "^" int } ;
atom        = sym | "ε" | "eps" | "∅" | "empty" | "Σ" | "(" regex ")" ;

cfgdecl     = "cfg" ident "{" ( { rule } | explicitcfg ) "}" ;
rule        = gsym ( "->" | "→" ) alt { "|" alt } ;
alt         = "ε" | gsym { gsym } ;
gsym        = char | "<" name ">" ;

letstmt     = "let" ident "=" call ;
call        = ident "(" [ arg { "," arg } ] ")" ;
arg         = expr | ident ":" expr ;
command     = check | run | parse | pump | enumerate | show | export ;
layout      = "layout" ident "{" { ident "at" "(" num "," num ")" | ident "->" ident "bend" num } "}" ;
```
The parser dispatches on the machine `kind`, so the same `→` token means "push" in a PDA label and "write" in a TM label without ambiguity.

---

## 11. Import / interop
- **JFLAP `.jff`** import/export (FA, PDA, TM). JFLAP is the most widely used existing tool, so interop lowers the switching cost for courses.
- **JSON** canonical form (stable, versioned `"delta_version": 1`), useful for autograders.
- Future: **lexer-spec mode**: `tokens { NUM = D+ (.D*)? ; ID = L (L ∪ D)* }` → one combined DFA with priorities (the book mentions lexical analyzers on p. 66).
