# 08 — Error and Exception Handling

Every model in the book comes with rules: a DFA's δ must be total, a TM's input alphabet may not contain the blank, a GNFA's start state may not have incoming arrows, and so on. This document lists **every rule the engine enforces**, what happens when it is broken, and how errors flow from the Rust engine to the screen.

---

## 1. Philosophy

1. **Errors are values, not crashes.** Rust has no exceptions; every fallible function returns `Result<T, E>`. User mistakes are *data* that the UI displays. They never become a thrown exception, a blank screen or a frozen tab.
2. **Report everything at once.** The checker collects *all* problems in the file, not just the first, the way a good compiler does.
3. **Every error teaches.** Each message says what is wrong, which definition in the book it breaks (quoted on hover), and, whenever possible, offers a one-click fix.
4. **A broken line never blanks the screen.** The parser recovers, and the last valid version of each machine stays on the canvas (greyed) until the code is fixed.
5. **Honest about limits.** Running out of budget, or hitting an undecidable question, is reported as an *outcome* ("still running after 1 000 000 steps"), never as a wrong answer.
6. **The engine must never panic.** `unwrap()`/`expect()` are banned in `delta-core` and `delta-dsl` by Clippy (`clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` as errors). If a bug still causes a panic, it is contained (§8).

---

## 2. The error pipeline

```
 source text
    │
    ▼  LEXER ──────────► E0xx lexical errors      (bad character, unclosed string/comment)
    ▼  PARSER ─────────► E1xx syntax errors       (missing brace, unknown field) + recovery
    ▼  CHECKER ────────► E2xx–E7xx model rules    (violations of Definitions 1.5, 1.37, 2.2, 2.13, 3.3 …)
    ▼  EVALUATOR ──────► E8xx command/type errors (dfa(G4) where G4 is a grammar, unknown name …)
    ▼  RUNTIME ────────► R9xx outcomes/limits     (input symbol not in Σ, budget exceeded, cancelled)
    ▼
 Document { objects, diagnostics[], results[] }  ──► editor squiggles, problems panel, diagram marks
```

Each stage runs even when an earlier one reported errors, as far as it safely can. For example, if one machine has a syntax error, the other machines in the file are still checked and drawn.

---

## 3. Severity levels

| Severity | Meaning | Effect |
|---|---|---|
| **Error** | Breaks the formal definition or cannot be evaluated | Object is not built; dependent commands are skipped and show "depends on M1, which has errors" |
| **Warning** | Legal, but probably a mistake | Object builds normally |
| **Info** | Helpful observation | Object builds; shown in the problems panel only |
| **Hint** | Style/suggestion | Faint underline only |

### Strict vs. lenient mode
Instructors need exact book definitions, while beginners benefit from friendly defaults. A file-level pragma picks the mode:

```
#mode strict      // exactly the book's definitions (default for classroom links)
#mode lenient     // helpful defaults (default in the playground)
```

| Situation | Strict | Lenient |
|---|---|---|
| DFA missing δ(q, a) | **Error** E201 | Warning; implicit trap state added (shown dashed) |
| `Q` or `Σ` not written | **Error** E101 | Info; inferred from δ and shown in the tuple panel |
| `⊔` missing from Γ | **Error** E703 | Info; added automatically |
| PDA push string longer than one symbol | **Error** E604 | Allowed (the book's shorthand, p. 118) |
| TM transition missing | Allowed: implicit reject, the book's own convention (p. 173) | Same |

---

## 4. The rule catalog, model by model

Code ranges: `E0xx` lexical · `E1xx` syntax · `E2xx` DFA · `E3xx` NFA · `E4xx` regex & GNFA · `E5xx` CFG · `E6xx` PDA/DPDA · `E7xx` TM · `E8xx` commands · `R9xx` runtime · `X999` internal. Warnings and infos share the ranges (`W2xx`, `I2xx`).

### 4.0 Rules shared by all machines
| Code | Severity | Rule | Source | Quick fix |
|---|---|---|---|---|
| E101 | error (strict) | Required tuple component missing (e.g. no `q0`) | each definition | insert template line |
| E102 | error | Component declared twice (`Q = …` twice) | — | remove duplicate |
| E103 | error | Alphabet is empty | §0.2: "an alphabet [is] any nonempty finite set" | — |
| E104 | error | State used in δ, q0 or F but not in Q | Q is the set of states | add to Q |
| E105 | error | Symbol used in δ but not in Σ (or Γ) | δ's domain | add to Σ |
| E106 | error | Start state not in Q | q₀ ∈ Q | — |
| E107 | error | F contains a state not in Q | F ⊆ Q | add to Q / remove |
| E108 | error | `ε` listed as a member of Σ | ε is the empty *string*, not a symbol | remove |
| E109 | error | Duplicate state or symbol name in a set | sets have no duplicates | remove |
| E110 | error | State name collides with a reserved word (`start`, `accept`, `ε`, `eps` …) | — | rename |
| W111 | warning | State unreachable from the start state | — | remove state |
| W112 | warning | F is empty, so the machine accepts nothing (legal, p. 36) | Def 1.5 allows F = ∅ | — |
| I113 | info | Dead state (no path to an accept state) | — | — |
| E114 | error | Multi-character symbol written bare (`ab` in Σ or a string) | symbols are single characters | write `<ab>` |
| W117 | warning (lenient) | `q0` not written; the first state of Q is used | q₀ ∈ Q is the start state | insert `q0 = …` |
| I115 | info (lenient) | Σ not written; inferred from δ | — | write Σ explicitly |
| I116 | info (lenient) | Q not written; inferred from δ, q0 and F | — | write Q explicitly |
| E120 | error | A state name is not a plain name (e.g. a string or a symbol) | — | — |
| E121 | error | A machine, set or δ block is missing its closing `}` | — | insert `}` |
| E122 | error | Unknown field in a machine (`Qs = …`), with an edit-distance suggestion | the fields are Q, Σ, δ, q0, F | rename |

### 4.1 DFA rules (Definition 1.5: δ : Q × Σ → Q)
| Code | Severity | Rule | Why | Quick fix |
|---|---|---|---|---|
| E201 | error (strict) | δ is not total: δ(q2, 1) undefined | "exactly one transition arrow exits every state for each possible input symbol" (p. 36) | add trap state / add transition / mark `partial` |
| E202 | error | δ is not a function: two transitions for (q1, 0) | δ maps each pair to *one* state | change `dfa` to `nfa` |
| E203 | error | ε-transition in a DFA | DFA δ takes Σ, not Σ_ε | change to `nfa` |
| E204 | error | Transition to a *set* of states, e.g. `δ(q1,0) = {q1,q2}` | DFA δ returns a state, not a set | change to `nfa` |
| E205 | error | δ-table has a wrong number of columns / unknown row | table shape must be \|Q\| × \|Σ\| | — |
| W201 | warning (lenient) | δ is not total; an implicit trap state is added and drawn dashed (E201 in strict mode) | p. 36 | add trap state / add transition / mark `partial` |
| I206 | info | DFA is not minimal (k states could be merged) | Myhill–Nerode | run `minimize` |

### 4.2 NFA rules (Definition 1.37: δ : Q × Σ_ε → P(Q))
| Code | Severity | Rule | Why | Quick fix |
|---|---|---|---|---|
| E301 | error | Label is neither a symbol of Σ nor ε | domain is Q × Σ_ε | add to Σ |
| W302 | warning | Same edge listed twice | δ returns a *set*, so the duplicate has no effect | remove |
| I303 | info | ε-cycle present (legal; the ε-closure handles it) | — | — |
| I304 | info | This NFA is deterministic and total, so it is a DFA | Def 1.5 | change to `dfa` |
| W305 | warning | No accept state reachable, so L(N) = ∅ | E_DFA-style marking | — |

### 4.3 Regular-expression rules (Definition 1.52)
| Code | Severity | Rule | Quick fix |
|---|---|---|---|
| E401 | error | Unbalanced parentheses | insert `)` |
| E402 | error | Operator with a missing operand: `∪ a`, `a ∪`, `* a`, `()` | — |
| E403 | error | Symbol not in the declared alphabet (`over {a,b}` but uses `c`) | add to alphabet |
| E404 | error | `R^k` with a non-integer or too large k (limit 64) | — |
| W405 | warning | `a+b`: in Sipser, `+` means one-or-more (R⁺ = RR*), not union | replace with `a ∪ b` |
| W406 | warning | Concatenation with ∅ makes the whole term ∅ (R∅ = ∅, Example 1.53) | — |
| I407 | info | Redundant: `R ∪ ∅` = R, `Rε` = R, `∅*` = ε (p. 66) | simplify |
| E408 | error | Expression nested more than 400 levels deep (keeps the engine's recursion bounded) | — |

### 4.4 GNFA rules (Definition 1.64 + the special form, p. 70)
| Code | Severity | Rule | Quick fix |
|---|---|---|---|
| E411 | error | Arrow *into* the start state | special form: no incoming arrows to q_start | remove |
| E412 | error | Arrow *out of* the accept state | special form: no outgoing arrows from q_accept | remove |
| E413 | error | Start state equals accept state | "the accept state is not the same as the start state" | add new accept state |
| E414 | error | More than one accept state | special form has exactly one | add new accept with ε-arrows |
| W415 | warning | Two arrows between the same pair of states | the book replaces them with one arrow labeled by their union | merge with ∪ |
| I416 | info | Missing arrows filled with ∅ automatically | — | — |
| E417 | error | Arrow label is not a valid regular expression | see §4.3 | — |

### 4.5 CFG rules (Definition 2.2)
| Code | Severity | Rule | Why | Quick fix |
|---|---|---|---|---|
| E501 | error | Left-hand side is not a single variable (`aS -> b`) | "each rule being a variable and a string…"; otherwise the grammar is not context-free | — |
| E502 | error | V and Σ overlap (same symbol used as both) | "Σ is a finite set, disjoint from V" | rename |
| E503 | error | Start variable not in V | S ∈ V | — |
| E504 | error | Empty rule body without ε (`S -> `) | write ε explicitly | insert ε |
| W505 | warning | Variable used on a right-hand side but never defined on a left-hand side, so it is treated as a terminal (book convention, p. 104) | — | add a rule |
| W506 | warning | Non-generating variable (can never produce a terminal string) | Thm 4.8 marking | remove rules |
| W507 | warning | Unreachable variable | — | remove |
| W508 | warning | ε mixed into a body (`aεb`) is redundant | ε is the empty string | remove ε |
| W509 | warning | Duplicate rule | R is a set | remove |
| I510 | info | L(G) = ∅ (start variable is non-generating) | Thm 4.8 | — |
| E511 | error | (`check is_cnf`) rule violates Definition 2.8: start variable on a right-hand side, A → ε with A ≠ S, unit rule, or long rule | Def 2.8 | run `cnf(G)` |

### 4.6 PDA rules (Definition 2.13: δ : Q × Σ_ε × Γ_ε → P(Q × Γ_ε))
| Code | Severity | Rule | Why | Quick fix |
|---|---|---|---|---|
| E601 | error | Read symbol not in Σ_ε | domain | add to Σ |
| E602 | error | Pop or push symbol not in Γ (classic: using `$` without declaring it) | Γ is the stack alphabet | add `$` to Γ |
| E603 | error | Label malformed: expected `a, b → c` | diagram notation p. 114 | — |
| E604 | error (strict) | Push string longer than one symbol | strict Def 2.13 pushes one symbol; longer strings are the p. 118 shorthand | expand shorthand |
| W605 | warning | ε-cycle that pushes symbols (state loop reading no input with net push) | the simulator may explore forever, so it uses budgets | — |
| W606 | warning | Stack is never popped / bottom marker never pushed | common design slip | — |
| I607 | info | Acceptance is by **final state**, not empty stack (book's definition) | students often assume empty-stack acceptance | — |

**DPDA additional rules (Definition 2.39)**
| Code | Severity | Rule |
|---|---|---|
| E611 | error | For some q, a ∈ Σ, x ∈ Γ, more than one of δ(q,a,x), δ(q,a,ε), δ(q,ε,x), δ(q,ε,ε) is non-empty (the message names the triple and both moves) |
| E612 | error | A single δ entry returns more than one move |
| W613 | warning | For some (q, a, x) *none* of the four is defined. Definition 2.39 requires exactly one, so the machine has no move there; lenient mode treats it as rejecting |

### 4.7 Turing-machine rules (Definition 3.3: δ : Q × Γ → Q × Γ × {L, R})
| Code | Severity | Rule | Why | Quick fix |
|---|---|---|---|---|
| E701 | error | Blank ⊔ in Σ | "Σ is the input alphabet not containing the blank symbol" | remove |
| E702 | error | Σ ⊄ Γ | "Σ ⊆ Γ" | add to Γ |
| E703 | error (strict) | ⊔ ∉ Γ | "⊔ ∈ Γ" | add ⊔ |
| E704 | error | q_accept = q_reject | "q_reject ≠ q_accept" | — |
| E705 | error | Accept/reject state missing from Q | — | add |
| E706 | error | Two transitions for the same (q, a) in a deterministic TM | δ is a function | change to `ntm` |
| E707 | error | Move other than L/R (e.g. `S`) without `stayput` | {L, R} only | add `stayput` |
| E708 | error | Written symbol not in Γ | δ's range is Q × Γ × {L,R} | add to Γ |
| W709 | warning | Transition out of q_accept or q_reject (ignored: halting states have no moves) | p. 169 | remove |
| I710 | info | Missing transitions go to q_reject moving R (book convention, p. 173) | — | show implicit transitions |
| E711 | error | Multitape arity mismatch: a 2-tape machine label must read 2, write 2, move 2 | Thm 3.13 model | — |

### 4.8 Command and type errors (the pipeline)
| Code | Severity | Example | Message idea |
|---|---|---|---|
| E801 | error | `let D = dfa(G4)` | "`dfa()` needs an NFA or a regex; `G4` is a CFG. A context-free language need not be regular." |
| E802 | error | `run X on "01"` | "No object named `X`. Did you mean `M1`?" (edit-distance suggestion) |
| E803 | error | `let D = …` twice | "`D` is already defined on line 12" |
| E804 | error | unknown option `regex(G, ordr: …)` | "Unknown option `ordr`. Valid: `order`, `mode`" |
| E805 | error | `regex(G, order: [q_start])` | "Rip order may only contain states other than q_start and q_accept (Lemma 1.60)" |
| E806 | error | `dfa(N, full: true)` with 20 states | "Full power set would have 2²⁰ states. Limit is 2¹⁶. Use the default (reachable subsets)" |
| W807 | warning | `intersect(D1, D2)` with different alphabets | "Alphabets differ, so both were extended to {0,1,2} with trap states" (strict: error) |
| I808 | info | `complement(N)` on an NFA | "Converted to a DFA first (Theorem 1.39). Swapping accept states of an NFA does not complement it" |
| E809 | error | `let A = union(B, A)` | "`A` refers to itself" |
| E810 | error | `check equivalent(G1, G2)` on two CFGs | "EQ_CFG is undecidable (Problem 5.1). Use `check equivalent(G1, G2) up to 10` for a bounded check" |
| I811 | info | `let D = dfa(N)` where `N` has errors | "Skipped: depends on `N`, which has errors" (one root error, not a cascade) |
| E123 | info | `let C = cnf(G)`, `pda P { … }` before Phase 3 | "`cnf()` arrives in Phase 3. This statement is skipped for now." (an `E` code, but reported as info so planned features never look like user mistakes) |

### 4.9 Runtime outcomes (R9xx): what happens while running
| Code | Kind | Situation | Behaviour |
|---|---|---|---|
| R901 | error | Input contains a symbol not in Σ: `run M1 on "012"` | **Error, not a silent reject.** Strings are defined over Σ. The message highlights the bad character |
| R902 | outcome | Conversion exceeded `max_states` | Partial result shown, marked incomplete; suggests minimizing first or raising the budget |
| R903 | outcome | TM exceeded `max_steps` | Outcome **RUNNING** (not "loops", which is undecidable, Thm 4.11), with Continue / Stop buttons |
| R904 | outcome | TM configuration repeated exactly | Outcome **LOOPS (proved)**: a repeated configuration of a deterministic TM means it never halts |
| R905 | outcome | PDA search exhausted the budget | Outcome **INCONCLUSIVE**, with an offer to use exact mode (PDA → CFG → CYK, always terminates) |
| R906 | outcome | Trace longer than `max_trace` | Result complete; trace truncated, with a note |
| R907 | outcome | User pressed Stop | **CANCELLED**: worker restarted, document re-evaluated, nothing lost |
| R908 | error | Input string too long for the chosen mode (e.g. NTM tree view > node limit) | Suggests the non-tree mode |
| R910 | warning | A `check` case fails: `check M1 accepts "10"` but M1 rejects it | Squiggle on the failing string; the Tests panel lists it, and clicking the row opens the run on that string |

### 4.10 Internal errors (X999)
A bug in the engine itself. See §8.

---

## 5. Representation in Rust

```rust
#[derive(Clone, Copy)]
pub enum Severity { Error, Warning, Info, Hint }

pub struct Span { pub start: u32, pub end: u32 }          // byte offsets into the source

pub struct BookRef { pub item: &'static str, pub quote: &'static str }
// e.g. BookRef { item: "Definition 1.5", quote: "δ : Q × Σ → Q is the transition function" }

pub struct Fix { pub title: String, pub edits: Vec<TextEdit> }  // applied by the editor

pub struct Diagnostic {
    pub code: &'static str,          // "E201"
    pub severity: Severity,
    pub span: Span,                  // where to underline
    pub message: String,             // "δ(q2, 1) is undefined"
    pub rule: Option<BookRef>,       // shown on hover
    pub related: Vec<(Span, String)>,// e.g. the other conflicting transition
    pub targets: Vec<Target>,        // states/edges to mark red on the diagram
    pub fixes: Vec<Fix>,
}

/// What the engine returns. Diagnostics are data, never a thrown exception.
pub struct Document {
    pub objects: Vec<NamedObject>,
    pub diagnostics: Vec<Diagnostic>,
    pub results: Vec<CommandResult>,      // check / run outputs, including R9xx outcomes
}

/// Errors that stop a single operation (as opposed to diagnostics about the file).
pub enum TlError {
    Invalid(Vec<Diagnostic>),             // precondition not met (E8xx)
    Budget { which: BudgetKind, partial: Option<Box<Object>> },
    Cancelled,
    Internal { message: String, context: String },   // X999
}
```

- The checker uses a `Diagnostics` collector (`push`, `has_errors`) rather than early `return Err`, so one pass reports everything.
- Algorithms use `Result<T, TlError>` with the `?` operator: Rust's equivalent of exceptions, but visible in every function signature.
- **Preconditions are re-checked inside every conversion.** For example, `complement` verifies totality even if the checker already did. The engine is also called from the CLI and tests, where no checker may have run.

---

## 6. Representation in TypeScript and the UI

The diagnostics come across the WASM boundary as plain typed JSON (types generated from the Rust structs).

| Where | What the user sees |
|---|---|
| Editor | red/yellow/blue squiggles; hover shows message + quoted definition; 💡 lightbulb applies a `Fix` |
| Problems panel | sortable list (severity, code, object, line); click jumps to code |
| Diagram | offending states/edges drawn **red dashed**; missing transitions shown as ghost arrows labeled `?` |
| Tuple panel | violating component highlighted ("δ(q2, 1) = ?") |
| Run panel | runtime outcomes (R9xx) as colored banners with action buttons (Continue, Use exact mode, Raise budget) |
| Error docs | every code has a help page (`/errors/E201`) with explanation, the book quote, a wrong example and a fixed example |

**TypeScript-side failures** (not user mistakes):
- Promise rejections are used only for *infrastructure* failures: worker crashed, WASM failed to load, out of memory.
- Each panel is wrapped in a **React error boundary**, so a rendering bug in, say, the CYK table shows "This panel hit a problem — reload panel" while the editor and diagram keep working.
- WASM fails to load (old browser, blocked by policy): a full-page message explains the requirement and offers a static read-only view of the code.

---

## 7. Parser error recovery

- **Synchronization points:** `}` closing a block, a newline at block level, or the next top-level keyword (`dfa`, `nfa`, `cfg`, `let`, `check` …). After an error the parser skips to the next sync point and continues.
- **Insert/delete repair** for common slips: a missing `}` at end of file, a missing `=` after `Q`, `->` vs `→`.
- **Last-good snapshot:** each named object keeps its last successfully built version. The canvas shows it greyed, with a banner "showing last valid version — 2 errors".
- **Typo suggestions:** unknown field `Sigam` → "did you mean `Sigma`?" (edit distance ≤ 2).

---

## 8. Panics, crashes and the worker

Even with `unwrap` banned, a bug can panic (e.g. an index out of bounds). In WebAssembly a Rust panic aborts the module, so:

1. `console_error_panic_hook` records the panic message and backtrace.
2. The worker catches the WASM trap and reports `X999 Internal error` to the main thread.
3. The main thread **automatically respawns the worker** and re-evaluates the last good document (~50 ms). The user's code is never lost because it lives in the editor, not the worker.
4. The UI shows: "Internal error in the engine (this is a bug, not your mistake). [Copy bug report]". The report contains the code (from the share link), the panic message and the version, with nothing personal.
5. If the same input crashes twice, the offending command is skipped and marked, so the rest of the file still works.

Natively (CLI/tests), `std::panic::catch_unwind` around each command gives the same isolation.

---

## 9. Testing the error system

- **One test file per error code** in `examples/errors/E201_dfa_not_total.<ext>` (etc.), with expected diagnostics as a snapshot (`insta` crate).
- **CI coverage check:** a script lists every code defined in `codes.rs` and fails if any code has no test file or no help page.
- **Fix round-trip tests:** applying each quick fix must remove the diagnostic and introduce no new errors.
- **Fuzzing:** `cargo-fuzz` on the parser and checker, asserting *no panics* on arbitrary input. Every fuzz-found crash becomes a regression test.
- **Mode tests:** each strict/lenient difference in §3 has a test for both modes.
