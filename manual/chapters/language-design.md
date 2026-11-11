---
title: "Language design"
description: "Why Delta looks the way it does: braces, set-builder, atoms, models, a tree-walker, error recovery, class inference."
---

Readers: **owner** (PL and design interviews).

## Braces, newlines, no semicolons

Blocks are `{ … }`; a newline or `;` ends a statement everywhere, including machine bodies.
**Why:** braces survive copy-paste into chat apps, slides and LMS text boxes that mangle
indentation; one syntax means one parser, one formatter and one set of docs. Python's
vocabulary (`let`, `fn`, `for x in S`, `and`/`or`/`not`) keeps it familiar. **Cost:**
newline-terminated statements need continuation rules (an operator at line end continues) — the
parser keeps a newline context stack for that.

## Set-builder as the only comprehension

`{ x ∈ S | p(x) }` and `{ f(x) | x ∈ S, cond }`, defined by recursion on the generator list
(SEMANTICS §10.2). **Why:** it's the notation of the course, so a definition copied from the
board runs. With `lang`, the generator can be Σ* — never iterated, only used as a membership
predicate.

## Atoms and the two lexing modes

States are **atoms** (`:q0`), not strings, so `M.start == :q0` is a value comparison. Inside a
machine body a bare `q0` *is* the atom, and `-a->` is one edge token: the lexer has a **machine
mode**. **Why:** machines read like the diagram (`q0 -a-> q1`); outside bodies, `q0` is a
variable as usual. `[expr]` in state position allows computed states (`[(p, q)]` for product
states).

## Models = storage + acceptance

One `machine` construct over **models**. A model is a descriptor: input mode (read-once, two-way,
on tape), storages (stacks, counters, tapes, queue), acceptance (final state, empty stack,
halting), determinism, totality. `dfa`, `pda`, `tm` and 11 others are ordinary Delta in the
prelude. **Why:** one simulator and one checker for everything; new models (a two-counter
machine, a queue automaton) need no engine code; classes follow from storage (two stacks or two
counters → Recognizable).

## A tree-walking interpreter

The AST is evaluated directly, strictly, left to right; closures capture values. **Why:**
programs are small and the heavy work is in Rust built-ins (subset construction, decisions), so
a bytecode VM wouldn't pay off yet. Budgets are ticks in the walker.

## Error recovery

<!-- excerpt: crates/delta-lang/src/parser.rs#parser-recover -->

On a syntax error the parser records it and skips to the next statement boundary at the same
brace depth. **Why:** the Pad evaluates while you type; one half-typed line must not blank every
diagram and test. Machine bodies have their own line-level recovery (`recover_machine_line`).

## Type and class inference

Types are inferred; annotations are optional (`fn f(a: word, k: int) -> bool`). The checker also
infers each language's **class** from its source and operations, and rejects undecidable
questions. **Why:** "the theory is in the type system" — closure and decidability are checked,
not just taught.

## Questions they'll ask

1. **"Why not embed in Python or Haskell?"** — Notation (`-a->`, `∈`, `≡`, set-builder) and a
   checker that knows decidability. An embedded DSL would need string-quoted machines or heavy
   syntax, and couldn't reject `≡` on CF languages before running.
2. **"Why Unicode operators?"** — They're the course notation; every one has an ASCII spelling
   (`in`, `union`, `equiv`, `eps`) and the editor turns `\cup` into ∪.
3. **"Why are machines immutable values?"** — Constructions return new machines; sharing is
   cheap with `Rc`; equality and hashing are structural, so machines can be set elements.
4. **"How do you parse `ends with`?"** — Contextual words: `ends` followed by `with` is a binary
   operator in the Pratt table; elsewhere `ends` is an identifier. Same for `on`, `to`, `from`.
5. **"What's desugared?"** — Machine literals to a canonical record form, `run M on w` to
   `show M.run(w)`, `check M accepts …` to one test per word (SEMANTICS §10).
6. **"Static or dynamic typing?"** — Dynamically checked at run time, with static inference
   where it's cheap: names, arities, classes, machine well-formedness.

## Self-test

<details class="selftest"><summary>1. Why braces over indentation?</summary>Robust to copy-paste; one syntax, parser, formatter.</details>
<details class="selftest"><summary>2. What does machine mode change in the lexer?</summary>Edge tokens (<code>-a-></code>, <code>-[…]-></code>) and bare names as atoms.</details>
<details class="selftest"><summary>3. What is a model, in Delta?</summary>A descriptor: input mode, storages, acceptance, determinism, totality — defined in the prelude.</details>
<details class="selftest"><summary>4. Why a tree-walker?</summary>Programs are small; heavy work is in Rust built-ins.</details>
<details class="selftest"><summary>5. Where does recovery resume after an error?</summary>At the next newline/semicolon at brace depth 0, or a closing brace.</details>
