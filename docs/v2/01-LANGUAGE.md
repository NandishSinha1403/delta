# 01 — The Delta 2 Language

Delta is a small, safe, complete language whose values are the objects of computation theory. This document is the specification draft for v2.0. Everything here must be implemented, tested and documented, or listed in §16 as deferred.

---

## 1. Taste

```delta
// A machine, written the way you'd draw it
dfa EndsAB {
  start q0
  accept q2
  q0 -a-> q1;   q0 -b-> q0
  q1 -a-> q1;   q1 -b-> q2
  q2 -a-> q1;   q2 -b-> q0
}

// A language, written the way you'd write it in math
lang L = { w ∈ {a,b}* | w ends with "ab" }

check EndsAB ≡ L up to 10            // bounded: L is only known to be Decidable
check EndsAB ≡ re"(a ∪ b)* a b"      // exact: both are Regular

// A construction, written like its definition
fn determinize(N) = explore {
  alphabet N.alphabet
  start  N.eclose({ N.start })
  step   R on a -> N.eclose(⋃ { N.δ(q, a) | q ∈ R })
  accept R if R ∩ N.accept ≠ ∅
}

let N = to_nfa(re"(a ∪ b)* a b")
check determinize(N) ≡ N
show determinize(N)
```

---

## 2. Lexical structure

| Thing | Syntax |
|---|---|
| Comments | `// line`, `/* block */`, `/// doc comment` (shown in hover and docs) |
| Statements | a **newline or `;`** ends a statement, everywhere (also inside machine and model bodies); a closing `}` ends the last statement of its block. A line continues if it ends with an operator, `,`, `(`, `[` or `{` |
| Blocks | `{ … }`. Indentation is free and the formatter decides it |
| Identifiers | letters, digits, `_`, `'` (`q0`, `p'`, `q_dead`); case-sensitive |
| Symbols | `'a'`, `'0'`, `'<RESET>'`. Inside machine and regex syntax a bare `a` or `<RESET>` is a symbol |
| Atoms | `:q0`, `:dead`. A `:` starts an atom only **where an expression starts** (at the start of a statement, after `(`, `[`, `{`, `,`, `=`, `:`, an operator or a keyword; never right after a name, literal, `)`, `]` or `}`) **and** only when a name follows with no space. So `x: T`, `{ from: p }` and `f(q:atom)` keep their `:`, while `{ from: :q0 }` and `{:q0:0}` contain atoms |
| Words | `"abba"`; the empty word is `""` or `ε` |
| Numbers | `42`, `1_000_000`; ranges `0..n` (exclusive), `0..=n` (inclusive) |
| Regex literal | `re"(0 ∪ 1)* 0 1"` |

### Unicode and ASCII (always both)

| Meaning | Unicode | ASCII | Editor shortcut |
|---|---|---|---|
| empty word | `ε` (also `λ`) | `eps` | `\epsilon` ⇥ |
| empty set | `∅` | `empty` | `\emptyset` ⇥ |
| union / intersection | `∪` `∩` | `union` `inter` (and `\|` in regexes) | `\cup` `\cap` ⇥ |
| difference / complement | `∖` / `∁L` | `minus` / `~L` | `\setminus` ⇥ |
| element / not / subset | `∈` `∉` `⊆` | `in` `not in` `subset` | `\in` ⇥ |
| equivalence of languages | `≡` | `equiv` | `\equiv` ⇥ |
| not equal / ≤ / ≥ | `≠` `≤` `≥` | `!=` `<=` `>=` | |
| big union | `⋃` | `union_all` | `\bigcup` ⇥ |
| arrow | `→` | `->` | `\to` ⇥ |
| blank | `⊔` | `_` | `\blank` ⇥ |
| Greek | `δ Σ Γ Δ` | `delta Sigma Gamma Delta` | `\delta` ⇥ |

The formatter can normalize a file to Unicode or to ASCII (a per-user setting). Meaning never depends on which spelling you used.

### Machine bodies have their own lexing mode

Inside the body of a machine literal (§5.1), the lexer works differently:

- **Edges are tokens.** `-a->` (a bare label) and `-[ … ]->` (a bracketed label list) are each one token. A bare label runs from the `-` to the first `->`. It may contain `→` but **never an ASCII `->`**: `q0 -a, X -> YZ-> q1` is error E1xx, with the quick fix "add brackets: `q0 -[a, X -> YZ]-> q1`".
- **Inside `-[ … ]->`**, an ASCII `->` is the label arrow (`a, X -> YZ` is the pda short form), because only `]->` closes the edge. Brackets inside must **balance**, so labels can contain lists; an unbalanced `[` or `]` is E1xx. The list is read with the model's label grammar (§5.2): `-[a, b]->` is two dfa labels, `-[a, X -> YZ]->` is one pda label.
- **Arithmetic only inside `[ … ]`.** A state expression `[e]`, and the headers of `for` and `if` up to their `{`, are read as ordinary expressions (names are variables, `-` is minus). Everywhere else in the body, `-` after a state starts an edge, and a bare name is an atom (§3, §5.4).

Outside machine bodies, `-` is always minus and `->` is always the arrow.

---

## 3. Values and types

Types are **inferred**. Annotations are optional (`fn f(N: machine[nfa]) -> machine[dfa]`) and are checked.

| Type | Examples | Notes |
|---|---|---|
| `bool` | `true`, `false` | |
| `int` | `3`, `-7` | 64-bit, overflow is an error. `/` and `%` are Euclidean: the remainder is never negative (`-7 % 3 == 2`, `-7 / 3 == -3`) |
| `symbol` | `'a'` | |
| `atom` | `:q0`, `:dead` | a name used as a value; equal only to the same atom. A bare state name inside a machine is an atom, so `M.start == :q0` |
| `word` | `"0110"`, `ε` | `len(w)`, `w[i]`, `w ++ v`, `w^3`, `rev(w)`, `count(w, '0')`, `w ends with v`, `w starts with v`, `w contains v` (substring). `∈`/`in` always mean membership, never substring |
| `set[T]` | `{1, 2}`, `∅` | finite; canonical order for printing |
| `list[T]` | `[1, 2]` | |
| tuple | `(p, q)` | |
| `map[K, V]` | `{ :q0: 0, :q1: 1 }` | keys are any values; atom keys are written as atoms (`{ :q0: 0 }` is a map, while `{ q0: 0 }` is still Q-07) |
| record | `{ from: p, to: q }` | |
| `regex` | `re"ab*"` | |
| `grammar` | `cfg { S -> a S b \| ε }` | |
| `machine[model]` | `dfa M { … }` | §5 |
| `model` | `model pda { … }` | §4 |
| `lang[class]` | `lang L = …` | §6 |
| `trace` | `M.run("01")` | configurations step by step, and `.verdict`: `:accept`, `:reject`, `:loops` or `:unknown` (§13) |
| `fn` | `fn(x) = x + 1` | first-class, closures |
| `none` | | |

**Immutability.** Values never change. `let` binds once; `var` declares a local that can be *rebound*. Operations return new values.

### Set-builder notation (the only comprehension form)

```delta
{ q ∈ M.states | q ∉ M.accept }                 // filter
{ (p, q) | p ∈ A.states, q ∈ B.states }         // map, several generators
{ M.δ(q, a) | q ∈ R, a ∈ Σ, q ≠ :dead }         // generators and conditions mixed
[ len(w) | w ∈ words ]                          // list version keeps order
```

**Word sets:** `Σ^n` (all words of length n) and `Σ^≤n` (length up to n) are finite sets and can be iterated. `Σ*` is infinite, so it's allowed **only inside `lang`** (§6), where it's never materialized.

---

## 4. Models: what kind of machine

A **model** says what a machine is made of. Every machine in Delta is an instance of a model.

```delta
model <name> {
  input    one-way | two-way | on-tape      // how input is read
  storage  none | stack | tape | tapes(k) | queue | counters(k)
  accept   by final | by empty-stack | by halting | by final-and-empty
  output   none | per-transition | per-state   // transducers
  [deterministic] [total] [epsilon] [labels regex] [special-form]
  missing  reject | dead | error             // what an absent transition means
}
```

### The standard prelude (ordinary Delta code, readable in the docs)

```delta
model dfa   { input one-way; deterministic; total; accept by final }
model nfa   { input one-way; epsilon; accept by final }
model gnfa  { input one-way; labels regex; special-form; accept by final }
model pda   { input one-way; epsilon; storage stack; accept by final }
model dpda  extends pda { deterministic }
model tm    { input on-tape; storage tape; deterministic; accept by halting; missing reject }
model ntm   extends tm { not deterministic }
model lba   extends tm { bounded }                        // head stays within the input
model mtm(k) { input on-tape; storage tapes(k); deterministic; accept by halting; missing reject }
model twdfa { input two-way; deterministic; total; accept by final }
model mealy { input one-way; deterministic; output per-transition }
model moore { input one-way; deterministic; output per-state }
model fst   { input one-way; epsilon; output per-transition }
model ctr(k) { input one-way; storage counters(k); accept by final }
```

As everywhere (§2), `;` separates properties written on one line. Each property has a precise meaning in the semantics document (`02-ENGINE.md` §2), and the checker enforces it. For example, `deterministic` + `total` means δ maps every (state, symbol) pair to exactly one move.

**Users can define new models** with the same building blocks: a 2-stack PDA (`storage stack, stack`), a queue automaton (`storage queue`), a counter machine.

---

## 5. Machines

### 5.1 Literal form

```delta
<model> <Name> [ (options) ] {
  <fields>
  <transitions>
}
```

Fields use readable names **or** textbook letters:

| Field | Aliases | Notes |
|---|---|---|
| `states` | `Q =` | inferred if omitted (lenient mode) |
| `alphabet` | `Σ =`, `input` | inferred if omitted |
| `stack` | `Γ =` (stack models) | |
| `tape` | `Γ =` (tape models) | must contain the blank; inferred blank `⊔` |
| `output` | `Δ =` | transducers |
| `start` | `q0 =`, `s =` | |
| `accept` | `F =`, `final` | |
| `reject` | | tape models |

Fields can be written `start q0`, `start: q0` or `q0 = q0`. All three are accepted.

### 5.2 Transitions

Four equivalent forms:

```delta
q0 -a-> q1                 // arrow
q0 -[a, b]-> q1            // several labels on one arrow
δ(q0, a) = q1              // function (the formal definition)
δ = table {                // table: header row is the labels
     | a  | b
  q0 | q1 | q0
  q1 | q1 | q2
}
```

**Labels** follow the model's storage. Each prelude model has a short textbook form, and every model also accepts the **keyword form**, which works for any storage:

| Model | Short form | Keyword form |
|---|---|---|
| dfa / nfa | `a`, `ε` | `read a` |
| pda | `a, X → YZ` | `read a; pop X; push YZ` |
| tm | `0 → x, R` · `x → R` | `write x; move R` |
| mtm(2) | `(0, ⊔) → (0, 0), (R, R)` | `on 1: write 0; move R` … |
| mealy | `a / 1` | `read a; emit 1` |
| fst | `a : xy` | `read a; emit "xy"` |
| ctr(2) | | `read a; inc c1; if c2 == 0` |
| queue | | `read a; dequeue X; enqueue YZ` |
| gnfa | `ab* ∪ c` (a regex) | |

`push YZ` leaves `Y` on top. Moves are `L`, `R` and `S`.

### 5.3 Options

```delta
dfa M (partial) { … }               // missing transitions → implicit dead state
tm  T (missing: error) { … }        // make a missing transition an error, not a reject
pda P (accept: by empty-stack) { … }
pda P (initial stack Z0) { … }      // start with Z0 on the stack (Hopcroft–Motwani–Ullman convention)
```

A stack starts **empty** unless the machine has the option `initial stack Z`; then it starts as `Z`, which must be in Γ.

### 5.4 States are values; machines can be generated

Any value can be a state. A bare name in state position is an **atom**: inside the machine you write `q0`, outside it `:q0` (`EndsAB.start == :q0`, `EndsAB.δ(:q0, 'a') == :q1`). `[expr]` writes any other value in state position, and `for` / `if` generate transitions:

```delta
fn divisible_by(n) = dfa {
  alphabet 0, 1
  start  [0]
  accept [0]
  for r ∈ 0..n {
    [r] -0-> [(2*r) % n]
    [r] -1-> [(2*r + 1) % n]
  }
}
check divisible_by(3) accepts "0", "11", "110", "1001"
check divisible_by(3) rejects "1", "10", "111"
```

### 5.5 Members

`M.model`, `M.states`, `M.alphabet`, `M.start`, `M.accept`, `M.edges` (list of `{from, label, to}`), `M.δ(q, a)` (a state or a set, by model), `M.eclose(S)`, `M.accepts(w)`, `M.run(w)` (trace, with `.verdict`), `M.output(w)` (transducers), `M.size`, `M.rename(f)`, `M.with(field: value)`, `M.lang` (the language, §6).

### 5.6 Canonical record form

Every literal desugars to this, and it's also what JSON output and `M.edges` use:

```delta
let N = machine nfa {
  states   { 0, 1, 2, 3 }
  alphabet { 0, 1 }
  start    0
  accept   { 3 }
  edges    [ (0, '1', 1), (1, ε, 2), (2, '0', 3) ]
}
```

---

## 6. Languages and their classes

A `lang` is a (possibly infinite) set of words. Its **class** is what Delta can guarantee about it:

```
Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any
```

### 6.1 Where languages come from

```delta
lang F = { "", "01", "0011" }                       // Finite
lang R = re"(0 ∪ 1)* 0 1"                           // Regular
lang D = EndsAB                                      // a machine: class from its model (dfa → Regular)
lang C = cfg { S -> 0 S 1 | ε }                     // CF
lang P = { w ∈ {0,1}* | count(w,'0') == count(w,'1') }   // Decidable (predicate; runs under a budget)
lang T = SomeTuringMachine                           // Recognizable
```

| Source | Class |
|---|---|
| finite set of words | Finite |
| regex, dfa, nfa, gnfa, twdfa | Regular |
| dpda | DCF |
| pda, cfg | CF |
| lba | CS |
| set-builder predicate | Decidable (the predicate must terminate; it's budgeted) |
| tm, ntm | Recognizable |

### 6.2 Operations compute the class (closure properties)

| Operation | Finite | Regular | DCF | CF | CS | Decidable | Recognizable | Any |
|---|---|---|---|---|---|---|---|---|
| `L ∪ K` | Finite | Regular | CF | CF | CS | Decidable | Recognizable | Any |
| `L ∩ K` | Finite | Regular | CS | CS | CS | Decidable | Recognizable | Any |
| `L ∩ R` (R regular) | Finite | Regular | DCF | CF | CS | Decidable | Recognizable | Any |
| `F ∩ L` (F finite) | Finite | Finite | Finite | Finite | Finite | Finite | Recognizable | Recognizable |
| `∁L` | Regular | Regular | DCF | CS | CS | Decidable | Any | Any |
| `L ∖ K` | as `L ∩ ∁K` | | | | | | | |
| `F ∖ L` (F finite) | Finite | Finite | Finite | Finite | Finite | Finite | Any | Any |
| `L ++ K` | Finite | Regular | CF | CF | CS | Decidable | Recognizable | Any |
| `L*` | Regular | Regular | CF | CF | CS | Decidable | Recognizable | Any |
| `rev(L)` | Finite | Regular | CF | CF | CS | Decidable | Recognizable | Any |

How to read it:
- **Mixed operands** take the larger class (the later one in the chain), then use that column. So `F ∪ C` with F Finite and C CF is CF.
- **Rows with a finite or regular operand come first.** If one operand of `∩` is Finite, use the `F ∩ L` row, with the column of the *other* operand's class. Otherwise, if one is Regular, use `L ∩ R` the same way. `F ∖ L` works like `F ∩ L`.
- **Why `F ∩ L` and `F ∖ L` stop at Decidable:** computing them means asking `w ∈ L` for each word of F, which is only guaranteed to finish when L is Decidable or lower. For a Recognizable or Any L, `F ∩ L` is Recognizable and `F ∖ L` is Any.
- **`∖` in general** is `L ∩ ∁K`: compute the class of `∁K`, then use the `∩` rows.

The result's class is an **upper bound** on what Delta can guarantee. For example, the intersection of two CF languages might happen to be regular, but Delta can't know that in general, so it says CS.

### 6.3 Questions are allowed only where they're decidable

| Question | Finite / Regular | DCF | CF | CS | Decidable | Recognizable |
|---|---|---|---|---|---|---|
| `w ∈ L` | ✓ | ✓ | ✓ | ✓ | ✓ (budgeted) | semi: `unknown` when the budget runs out |
| `L.is_empty()` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
| `L.is_finite()` | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ |
| `L.is_universal()` | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| `L ≡ K` | ✓ | decidable, not implemented* | ✗ | ✗ | ✗ | ✗ |
| `L ⊆ K` | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |

\* Equivalence of deterministic CF languages is decidable, but the known algorithm is impractical to implement. Delta says exactly that and offers a bounded check.

Asking an undecidable question is a **compile-time error** that explains why and offers an alternative:

```
error[E701]: equivalence of context-free languages is undecidable
  --> main.delta:12:7
   |
12 | check C ≡ P
   |       ^^^^^ C is CF, P is Decidable
   = help: use a bounded check:  check C ≡ P up to 12
```

Bounded results are labeled in every output: `≡ for all words of length ≤ 12 (bounded)`.

### 6.4 Raising a class with evidence

```delta
lang E = { w ∈ {0,1}* | count(w,'0') % 2 == 0 } as Regular by re"(1* 0 1* 0)* 1*"   // claim: E is regular, with this regex as evidence
```

Delta checks the claim **up to a bound** and marks the value *assumed* everywhere it's used. The playground shows an "assumed" badge, so a claim is never confused with a proof.

---

## 7. Constructions: `explore` and `fixpoint`

### 7.1 `explore`: build a machine from successors

```delta
explore [model] {
  alphabet <set>                             // required: the input alphabet
  start  <expr>                              // the start state (any value)
  step   <pattern> on <symbol> -> <expr>     // deterministic successor
  step   <pattern> on <symbol> -> any <set>  // nondeterministic successors (nfa)
  step   <pattern> on ε -> any <set>         // ε-moves (nfa)
  accept <pattern> if <cond>
}
```

The `alphabet` clause is required: an `explore` without one is error E1xx. Only states reachable from `start` are created, and creation is budgeted. The default model is `dfa` when every `step` is deterministic, and `nfa` otherwise.

**Product construction:**
```delta
fn product(A, B, keep) = explore {
  alphabet A.alphabet ∪ B.alphabet
  start  (A.start, B.start)
  step   (p, q) on a -> (A.δ(p, a), B.δ(q, a))
  accept (p, q) if keep(p ∈ A.accept, q ∈ B.accept)
}
let both   = product(A, B, fn(x, y) = x and y)
let either = product(A, B, fn(x, y) = x or y)
```

For constructions on other models (PDA, TM), `explore` takes a general `edges` clause: `edges q -> [ (label, target), … ]`.

### 7.2 `fixpoint`: least fixed points

```delta
fixpoint X from <init> { <expr using X> }   // repeat X ← expr until it stops changing
```

```delta
fn eclose(N, S) = fixpoint X from S { X ∪ { r | q ∈ X, r ∈ N.δ(q, ε) } }
fn reachable(M) = fixpoint X from { M.start } { X ∪ { M.δ(q, a) | q ∈ X, a ∈ M.alphabet } }
fn nullable(G)  = fixpoint X from ∅ { X ∪ { A | (A, rhs) ∈ G.rules, set(rhs) ⊆ X } }   // ε-rules have rhs = []
```

Budgeted. If the value stops growing in a finite domain, the fixpoint is reached; otherwise the budget stops it with a clear message.

### 7.3 Ordinary control flow is still there

```delta
var todo = [M.start]
var seen = { M.start }
while todo ≠ [] {
  let q = todo[0]
  todo = todo[1..]
  for a ∈ M.alphabet {
    let r = M.δ(q, a)
    if r ∉ seen {
      seen = seen ∪ { r }
      todo = todo ++ [r]
    }
  }
}
```

---

## 8. Functions and statements

```delta
fn name(a, b) = expr                       // expression body
fn name(a: word, k: int) -> bool {         // block body, optional types
  if k == 0 { return true }
  return name(a, k - 1)
}
let f = fn(x) = x + 1                      // anonymous

let x = 3
var y = 0;  y = y + 1                      // a newline or ";" ends a statement
if c { … } else if d { … } else { … }
for x ∈ S { … }  (or: for x in S { … })
while c { … }
return e
```

### Output and testing

| Statement | Goes to |
|---|---|
| `print a, b` | Console |
| `show x` | Visual panel: a diagram for machines, a tree for grammars and parses, a tape for traces, a table for sets and maps, samples for languages |
| `run M on "w"` | Trace panel (sugar for `show M.run("w")`) |
| `assert cond` | Tests (shows the failing expression and its values) |
| `check M accepts "…", …` / `rejects` | Tests, one per word |
| `check A ≡ B` / `check A ≡ B up to n` | Tests; on failure, the shortest word that tells them apart |
| `check A ⊆ B`, `check L.is_empty()` | Tests |
| `save x to "file.svg"` | A file: the CLI writes it, the Pad downloads it |

`save` writes a value to a file whose extension picks the format:

| Extension | Contents | Values |
|---|---|---|
| `.svg`, `.png`, `.pdf` | The diagram, drawn the same way by the Pad and the CLI | machines |
| `.tex` | A TikZ picture of the diagram | machines |
| `.txt` | The text that `print` shows | any |
| `.json` | The value as JSON (the `show` value of the output schema) | any |

```delta
save M to "even-zeros.svg"
save M.minimize() to "out/min.pdf"
save M.states to "states.txt"
```

The path is relative and stays inside the folder the program runs in: no leading `/`, no drive
letter, no `..`. The engine itself never writes files. It emits a `save` output and the host
writes it: `deltac` writes inside the current folder (creating subfolders), the Pad offers a
download. Another extension, a path that leaves the folder, or a value with no diagram saved to
an image is an error (E514, E515, E516).

---

## 9. Regular expressions

`re"…"`, with optional `over {…}`. Operators:

| | Syntax | Precedence |
|---|---|---|
| group | `( )` | highest |
| postfix | `R*`, `R⁺`, `R?`, `R^n` | 1 |
| concatenation | juxtaposition, `∘`, `·` | 2 |
| union | `∪`, `\|` (or `+` under a notation profile, §10) | 3 |

Atoms: a symbol, `<multi>`, `ε`, `∅`, `Σ` (any single symbol), `[abc]`. Spaces are ignored. So `010*` means `0 · 1 · (0*)`.

## 10. Notation profiles

```delta
use notation "hmu"      // + is union, ε as usual
```

| Profile | `+` in regexes | Empty word | Blank |
|---|---|---|---|
| `default` | one-or-more | ε | ⊔ |
| `hmu` (Hopcroft–Motwani–Ullman) | union | ε | B |
| `linz` | union | λ | □ |
| custom | `notation { union: "+", epsilon: "λ", empty: "φ" }` | | |

Profiles change **reading and printing only**. Ambiguous input (for example `a+b` under the default profile) is a warning with quick fixes, never a silent guess.

## 11. Grammars

```delta
cfg G {
  S -> a S b | S S | ε
}
cfg Expr {
  start <E>
  <E> -> <E> + <T> | <T>
  <T> -> <T> * <F> | <F>
  <F> -> ( <E> ) | a
}
```

Variables are the left-hand sides unless declared. Symbols are space-separated **or** single characters. `<Name>` is always one symbol. Grammars are languages too (`lang L = G` is CF).

---

## 12. Standard library (Rust built-ins; overview, full reference generated)

Every built-in takes an optional `method:` naming **the algorithm** (never a book), and returns a value that remembers its **provenance and step trace** (`steps(x)`).

**What the states of a result are.** `determinize` gives sets of the input's states; `minimize` keeps a state that merges with nothing and makes a merged class the set of its states; `union`/`intersect`/`difference`/`product` give pairs `(p, q)`; `concat` tags states `(1, q)` and `(2, q)`; `complete` adds `:dead`, and `reverse`, `star` and `to_gnfa` add `:start` (and `:accept`), renamed `:dead1`, `:start1`, … if taken; `to_nfa` numbers its states `0, 1, …`. Every result remembers its algorithm id, and `steps(x)` lists its trace.

**Conversions are verbs**. A model name (`dfa`, `nfa`, `pda`, …) only starts a machine literal (`dfa M { … }`, `dfa { … }`), appears in types (`machine[dfa]`), and names the model value (`M.model == dfa`). It is never called: `dfa(N)` is error E2xx with the quick fix "`determinize(N)`" (likewise `nfa(R)` → `to_nfa(R)`, `gnfa(M)` → `to_gnfa(M)`).

| Area | Functions |
|---|---|
| Regex ↔ automata | `to_nfa(R, method: "textbook" \| "thompson" \| "glushkov")`, `to_regex(M, method: "state-elimination" \| "arden" \| "brzozowski-mccluskey", order: […])`, `to_gnfa(M)`, `simplify(R)` |
| Automata | `determinize`, `minimize(method: "hopcroft" \| "moore" \| "brzozowski" \| "table")`, `complete`, `trim`, `remove_epsilon`, `reverse`, `complement`, `union`, `intersect`, `difference`, `concat`, `star`, `product` |
| Decisions | `equivalent` (with witness), `subset_of`, `is_empty`, `is_finite`, `is_universal`, `shortest`, `words(L, upto: n)`, `count(L, length: n)`, `myhill_nerode(D)`, `pump(D, w)` |
| Grammars | `cnf`, `gnf`, `nullable`, `useless`, `trim`, `parse(G, w, method: "cyk" \| "earley")`, `derive`, `to_pda`, `to_cfg`, `is_ambiguous(G, upto: n)` (bounded) |
| Machines | `M.run(w, fuel:)`, `to_single_tape`, `mealy_to_moore`, `moore_to_mealy`, `F.transduce(w)` |
| Words and sets | `len`, `count`, `rev`, `prefixes`, `suffixes`, `substrings`, `Σ^n`, `Σ^≤n`, `power_set`, `sort` |

---

## 13. Errors, strictness, budgets

| Range | Kind | Example |
|---|---|---|
| E0xx | lexical | unterminated string |
| E1xx | syntax (with recovery) | expected `}` |
| E2xx | names and types | unknown name `Mx`, did you mean `M`? |
| E3xx | machine and model rules | `dfa M`: no transition for (q2, '1'). A total deterministic model needs one. Quick fixes: add a dead state / mark `(partial)` |
| E4xx | regex and grammar | unbalanced `(` |
| E5xx | runtime | assert failed, with values |
| B6xx | budget outcomes | stopped after 10,000,000 steps |
| E7xx | **decidability** | equivalence of CF languages is undecidable (offers `up to n`) |
| X999 | internal | engine restarts; your code is kept |

- **`use strict`** requires every field to be written explicitly and disables inference. Teachers use it for assignments. The default is **lenient**.
- **Budgets:** steps, states created, recursion depth and output size. Defaults are generous; adjust with `budget { steps: 50_000_000 }`. Hitting a budget is an *outcome*, not a crash: the program stops there, the outputs it already produced stay, and the result has `ok: false` with a B6xx diagnostic. (A machine *run* that runs out of budget instead gets the verdict `unknown` and the program goes on.)
- **Verdicts.** A run of a machine on a word ends with one of four verdicts, the same words everywhere (traces, JSON, the playground):
  - **accept**: an accepting configuration was reached;
  - **reject**: every branch ended without accepting;
  - **loops**: no branch accepts, and a branch is proved never to halt because a configuration repeats;
  - **unknown**: the budget ran out first.

  For a decision (`w ∈ L`, `M.accepts(w)`), accept means `true`, reject and loops mean `false`, and unknown is the budget outcome.
- **Explain profiles** (`use explain "sipser3e"`) add textbook citations to errors and steps. They're off by default.

---

## 14. The formal core (what everything desugars to)

The checker and interpreter work on a small core:

```
values   v ::= bool | int | symbol | word | set | list | tuple | map | record | fn
               | machine(model, Q, alphabets, edges, start, accept, extra)
               | lang(class, source)        // source: machine | regex | grammar | predicate | finite set
               | model(properties)
exprs    e ::= literals | x | e(e…) | e.f | op(e…) | { e | gens } | if | let/var | while | for
               | explore{…} | fixpoint{…}
```

- Machine literals, `δ` tables, label short forms, `run`, `check` and notation profiles are all **syntax sugar** over this core.
- The semantics of each model property (`deterministic`, `total`, `by final`, …) is a short formal definition over configurations, in `docs/v2/SEMANTICS.md`, written in Phase 0.

---

## 15. Grammar (EBNF, abridged)

```ebnf
program    = { directive } { item } ;
directive  = "use" ( "notation" ( string | "{" pairs "}" ) | "strict" | "explain" string )
           | "budget" "{" pairs "}" ;
item       = modeldecl | machinedecl | langdecl | fndecl | stmt ;

modeldecl  = "model" ident [ "(" params ")" ] [ "extends" ident ] "{" { property } "}" ;
machinedecl= ident ident [ "(" options ")" ] "{" { field | transition | mfor | mif } "}"   (* first ident = model *)
           | "machine" ident "{" { field } "}" ;
field      = fieldname ( [":"] exprlist | "=" expr ) ;
transition = state edge state                (* edge: one token in machine-body mode, §2 *)
           | ( "δ" | "delta" ) "(" exprlist ")" "=" expr
           | ( "δ" | "delta" ) "=" table ;
state      = ident | "[" expr "]" ;              (* ident = an atom *)
edge       = "-" barelabel "->" | "-[" labels "]->" ;   (* barelabel has no ASCII "->" *)

langdecl   = "lang" ident "=" ( expr | setbuilder ) [ "as" class "by" expr ] ;
fndecl     = "fn" ident "(" params ")" [ "->" type ] ( "=" expr | block ) ;
stmt       = "let" pattern "=" expr | "var" ident "=" expr | ident "=" expr
           | "if" expr block { "else" "if" expr block } [ "else" block ]
           | "for" pattern ( "∈" | "in" ) expr block | "while" expr block
           | "return" [ expr ] | "print" exprlist | "show" expr | "assert" expr
           | "save" expr "to" expr
           | "check" checkspec | "run" expr "on" expr | expr ;
expr       = … | atom | setbuilder | "re" string [ "over" expr ]
           | "explore" [ ident ] "{" { exclause } "}" | "fixpoint" ident "from" expr block ;
setbuilder = "{" expr "|" gen { "," ( gen | expr ) } "}"
           | "{" ident ( "∈" | "in" ) expr "|" expr "}" ;
gen        = pattern ( "∈" | "in" ) expr ;
atom       = ":" ident ;                         (* only where an expression starts, no space, §2 *)
```

---

## 16. Deferred (reserved now, built later)

| Feature | Plan |
|---|---|
| `choose` (nondeterministic programs, guess-and-verify) | keyword reserved; v2.1 |
| ω-automata (Büchi, Muller), weighted and probabilistic automata | new `accept` and `labels` kinds; v2.x |
| Reductions (`reduce A to B via f`, checked on bounded instances) | v2.x |
| Modules across files (`import`) | pad tabs first, then `import`; v2.1 |
| Custom short label forms for user models | v2.1 |
| Cellular automata, PCP explorer | community or v2.x |
