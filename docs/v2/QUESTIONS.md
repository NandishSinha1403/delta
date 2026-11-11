# Delta 2 — Open questions and contradictions (Phase 0)

Found while auditing v1 and writing `SEMANTICS.md` and the conformance suite. Each row gives a
**recommended default**. SEMANTICS.md and `conformance/` already use these defaults, tagged
`[Q-nn]`. Accept, change or strike each one, and the spec freezes.

**Blocks** = the first phase that can't start or finish without an answer. ✅ = confirmed: the
final answer is in the next section, and the row keeps the original default for history.

## Confirmed answers (2026-11-08)

The owner's answers, with the refinements agreed in the same session. They are applied to
`01-LANGUAGE.md`, `SEMANTICS.md` (their `[Q-nn]` tags are removed) and `conformance/`.

| # | Topic | Answer | Status |
|---|---|---|---|
| Q-01 | Statements per line | A newline **or** `;` ends a statement, everywhere (machine and model bodies too); `}` ends the last statement of a block. The spec examples (§1, §8) use `;` or separate lines. This also answers Q-02 | CONFIRMED |
| Q-03 | Transition arrow | Machine bodies have their own lexing mode. `-a->` and `-[ … ]->` are edge tokens. A bare label runs to the first `->` and can't contain an ASCII `->` (E1xx, quick fix: add brackets). Inside `-[ … ]->` an ASCII `->` is the label arrow, because only `]->` closes the edge; brackets inside must balance; the list is read with the model's label grammar. Arithmetic only inside `[ … ]` (and in `for`/`if` headers) | CONFIRMED |
| Q-06 | What `dfa` means | Model names start machine literals, appear in types and name the model value (`M.model == dfa`); they are never called. Conversions are verbs: `determinize(N)`, `to_nfa(R)`, `to_gnfa(M)`. `dfa(N)`, `nfa(R)`, `gnfa(M)` are E2xx with a quick fix | CONFIRMED |
| Q-08 | Substring | `w contains "ab"`. `in`/`∈` always mean membership | CONFIRMED |
| Q-26 | State names | Bare names inside a machine are **atoms**; outside they are written `:q0` (`M.start == :q0`). `:` starts an atom only where an expression starts and when a name follows with no space, so `x: T` and `{ from: p }` are unaffected. Maps can be keyed by atoms: `{ :q0: 0, :q1: 1 }` | CONFIRMED |
| Q-44 | Citations | `delta-core` doc comments name the algorithm id; citations live only in `delta-explain` profiles; the CI grep covers comments | CONFIRMED |
| Q-62 | Initial stack | Empty by default; machine option `initial stack Z0` (Hopcroft–Motwani–Ullman convention) | CONFIRMED |
| Q-69 | Accept by halting | Accept on entering the accept state; reject on entering the reject state or when there is no move. Verdicts everywhere: **accept \| reject \| loops** (proved by a repeated configuration) **\| unknown** (budget exhausted); in traces they are the atoms `:accept`, … | CONFIRMED |
| Q-75 | Closure table | Finite column (∪ ∩ ++ rev → Finite; ∁ and * → Regular) and Any column (everything Any). Mixed operands take the larger class. `F ∩ L` and `F ∖ L` are Finite when L ≤ Decidable; for a Recognizable or Any L, `F ∩ L` → Recognizable and `F ∖ L` → Any. Otherwise `L ∖ K` = `L ∩ ∁K` (SEMANTICS §7.5). See Q-92 | CONFIRMED |
| Q-80 | `explore` alphabet | The `alphabet` clause is required (E1xx without it) | CONFIRMED |

## A. Lexing and syntax (01 §2, §5, §15)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-01 ✅ | §2 vs §1, §8 | "One statement per line", yet §1 puts two transitions per line (`q0 -a-> q1    q0 -b-> q0`) and §8 has `let x = 3        var y = 0      y = y + 1` | Transitions are self-delimiting, so several may share a line **inside machine bodies**. Elsewhere use a newline or `;`. Rewrite the §8 line | P1 |
| Q-02 | §2 vs §4 | §2: "`;` is allowed but never needed" (everywhere). §4: "`;` separates properties on one line, **inside a model body only**" | **Answered by Q-01** (CONFIRMED): `;` is a statement separator everywhere | P1 |
| Q-03 ✅ | §2, §5.2 | `-label->` clashes with minus and negative numbers (`[r] -0-> …`), with `- [list]` (`-[a, b]->`), and in ASCII with the short-form arrow inside labels: `q0 -a, X -> YZ-> q1` | In machine bodies, after a state, `-` opens a label that runs to the **last** `->` before the target. The formatter writes inner arrows as `→`. Outside machine bodies `-` is minus | P1 |
| Q-04 | §5.4, §5.6 | Are `alphabet 0, 1` and §5.6's `alphabet { 0, 1 }` symbols or ints? §5.6's edges use `'1'`, and its states `{0,1,2,3}` are ints | `alphabet`/`stack`/`tape`/`output` fields read their elements in **symbol context** (bare `0` = `'0'`). `states`, `start` and `accept` read values | P1 |
| Q-05 | §5.4 vs §15 | `fn divisible_by(n) = dfa { … }` is an anonymous literal; the EBNF requires `ident ident` | The name is optional | P1 |
| Q-06 ✅ | §4, §12 | `dfa` is a model value, a literal keyword **and** a function (`dfa(N)` = determinize). The same goes for `nfa(R)` and `gnfa(M)` | Keep all three, separated by syntax: `dfa X {`/`dfa {` = literal, `dfa(` = call, otherwise the model value | P1 |
| Q-07 | §3 | `{ q0: 0, q1: 1 }` (map) and `{ from: p, to: q }` (record) look the same. `{}` is ambiguous (empty set, map, block) | `{ ident: e }` = record; maps are `map { k: v }`; `{}` = the empty set (also `∅`) | P1 |
| Q-08 ✅ | §3 | `v in w` is **substring**, but `in` is the ASCII `∈` (membership). `"11" ∈ w`? | `∈`/`in` always mean membership. Substring is `w contains v`. Fix §3 (conformance 40 uses `in` until decided) | P1 |
| Q-09 | §3, §15 | `{ x ∈ S \| y ∈ T }` parses as filter (x over S, condition y ∈ T) or as map (the bool `x ∈ S` for each y) | If the head is exactly `ident ∈ expr`, it's the filter form. Write `{ (x ∈ S) \| … }` for the map | P1 |
| Q-10 | §2 | `'` is both a prime in identifiers (`p'`) and a symbol quote (`'a'`): `f(p','a')` | A `'` right after an identifier character is a prime | P1 |
| Q-11 | §10 | Profile tokens collide with names: `hmu` makes `B` the blank, `_` is the ASCII blank and a likely wildcard pattern | Profile and blank tokens act only inside labels and regexes | P1 |
| Q-12 | §6.2 | Postfix `L*` on expressions clashes with `*` multiplication (`2*r` in §5.4) | Postfix `*` on a non-regex expression only right before `)`, `.`, `,` or end of line. Otherwise use `star(L)` | P1 |
| Q-13 | §3, §7.3 | `todo[1..]` (open slice) and list `++` are used, but only word ops are listed | Slices `[a..b]`, `[a..]`, `[..b]` for lists and words; `++` for lists and words | P1 |
| Q-14 | §15 | The EBNF has no `cfg`, `table`, machine options, `machine … { edges }`, `mfor`/`mif`, `checkspec`, patterns, `extends`/`not`, `as … by` placement, `run … on` vs `run M.run`, or the lang operators | Complete the EBNF as part of the freeze (Phase 1, first task) | P1 |
| Q-15 | §9, §10 | `+` under the default profile: is postfix `a+` (ASCII one-or-more) allowed? And which quick fixes go with the `a+b` warning? (conformance 31 assumes 2) | Yes. Fixes: "union: `a ∪ b`" and "one-or-more: `a⁺b`" | P1 |
| Q-16 | §11 | Grammar terminals: `\|` and `->` can't be terminals, and multi-character terminals and spaces have no syntax | Quoted symbols in grammars: `'\|'`, `'->'`, `'if'` | P6 |
| Q-17 | §2, §9 | How do you write a **word** containing a multi-character symbol? (`"<RESET>a"`) | Inside word literals `<NAME>` is one symbol; `\<` escapes | P1 |
| Q-18 | §1, 02 §4 | §1 defines `fn determinize`, which shadows a built-in, and "shadowing warnings" exist (02 §4) | Top-level user definitions shadow the prelude and built-ins **silently**. Only local shadowing warns | P1 |
| Q-19 | §5.5, §2 | Is `M.delta(q, a)` an alias of `M.δ`? Is `Sigma` an alias in `let Sigma = …`? | ASCII Greek names alias each other everywhere (`delta` ≡ `δ`) | P1 |

## B. Values and evaluation (01 §3, §5.5, §8)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-20 | §3, §5.6 | `ε` is both the empty word and the ε-label (`(1, ε, 2)`, `M.δ(q, ε)`) | `ε` is the empty word value. As a label, the empty word means "no read" | P1 |
| Q-21 | §3 | `w[i]` and `todo[0]`: 0- or 1-based? | 0-based | P1 |
| Q-22 | §3, 02 §4 | "Canonical order" for printing sets of mixed values isn't defined | SEMANTICS §1.2 (type tag, then structural; words shortlex) | P1 |
| Q-23 | §5.5 | `M.δ(q, a)` when the transition is missing (a `(partial)` dfa)? | `none` (or the implicit dead state when `missing dead`; see Q-60) | P2 |
| Q-24 | §3, §7.3 | "`var` declares a *local*", but §7.3 rebinds outer `var`s from inner blocks. And do closures capture variables or values? | Inner blocks may rebind enclosing `var`s; closures capture values | P1 |
| Q-25 | §3 | Which error does `Σ*` outside `lang` give? | E2xx (a type error: an infinite set) | P1 |
| Q-26 ✅ | §5 | A bare state name `q0` inside a machine: **what value is it**, and how does code outside the machine refer to it (`M.start == ?`)? | It's a symbol. Allow multi-character symbol literals `'q0'` (keep `<…>` for multi-character symbols in labels and regexes) | P1 |
| Q-27 | §8, §6 | Are machines, regexes and grammars implicitly langs? (`check G accepts …`, `"ab" ∈ re"…"`, `check re"a" ⊆ M`) | Yes: wherever a `lang` is expected, they coerce to `.lang` | P1 |
| Q-28 | §3 | `Σ^n` uses a free `Σ`. Is there a global Σ? | No. `Σ` is an ordinary name the user binds | P1 |
| Q-29 | §5.5 | `M.size` means? | \|Q\| | P2 |

## C. Products and UI (03)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-30 | 01 §8 vs 03 §3.1 | `run M on "w"` goes to the "Trace panel", but the Pad's panels have no Trace panel (v1 had a Run panel) | Add a **Trace** panel (tape/storage strip and timeline), or name Diagram + storage views as its home | P4 |
| Q-31 | README §3, 03 §6.1, 04 §2 | PWA "comes first" / "(Phase 3)" / Phase 4 | Phase 4 | P4 |
| Q-32 | 03 §7 | Courier Prime for code: it lacks most math glyphs (∪ ∩ ∈ ⊆ ≡ ⋃ ∁ ⊔ ⁺ ⊣ δ Σ …), which then all fall back to STIX Two Math. v1 found Plex's ∪ unreadable and had to patch glyphs | **CONFIRMED 2026-11-11 (owner):** IBM Plex Mono is the one code font everywhere code appears (Pad editor, site and docs code, manual, exports, `deltac` images, social cards), self-hosted. Fallbacks for exactly what Plex lacks: a Noto Sans Math subset (Greek, ∪ ∩ ∈ ⊆ ≡ ⊔ ⊢ ▸ …) and a Noto Sans Mono subset (⁺ ⁿ ᵏ ᵐ …). Commit Mono, JetBrains Mono and the KaTeX ∪ are gone. Non-code text keeps its font. `scripts/subset-fonts.py` cuts every file; `fonts/symbols.txt` in delta-export is checked in the Pad (e2e) and in exports | P4 |
| Q-33 | 04 §1 | `delta migrate` (v1 → v2 syntax) isn't in any phase | Phase 2 (once machines parse), so the v1 golden tests convert through it | P2 |
| Q-34 | 02 §1 vs 02 §8 | Layout: `delta-render` (Rust: "layout hints, DOT, SVG") or `layout()` "Graphviz in the worker" (JS, as in v1)? | Rust emits DOT and other exports. Graphviz-WASM in the worker lays out, as in v1. TS draws the SVG | P4 |

## D. Engine and architecture (02)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-39 | 02 §2, §4 | New dependencies `indexmap`, `smallvec`, `im` (and probably `schemars`) need your OK (CLAUDE.md). `Arc<Model>` and persistent collections go beyond v1's "Simple Rust" rule | Approve indexmap and im. Skip smallvec (`Vec` is fine). Rewrite the "Simple Rust" rule for v2 | P1 |
| Q-40 | 02 §7 | Output JSON has only `line`, but the editor needs UTF-16 ranges (v1 `Utf16Map`) | Diagnostics and outputs carry `line`, `col` (1-based, UTF-16) and `from`/`to` (UTF-16 offsets). Conformance uses `line`/`col` | P1 |
| Q-41 | 02 §7 | One source of truth for the schema: ts-rs (v1) or the published JSON Schema? | Rust structs are the source; generate TS (ts-rs) **and** `/schema/output-v2.json` (schemars) from them | P1 |
| Q-42 | AUDIT §2.6 | v1 `run(src, name, input)` re-evaluates the whole program per run. That's costly once user code is budgeted | Traces are program outputs. "Run this machine on this word" from the UI appends a synthetic `run` statement | P4 |
| Q-43 | 02 §2, §3 | Crate cycle: `Machine.states: IndexSet<Value>` puts the interpreter's `Value` in `delta-core`, and `LangSource::Predicate(Closure)` puts an interpreter closure in `delta-classes`, which `delta-lang` depends on | `Value` lives in `delta-core` (functions as opaque ids). `delta-classes` asks for predicate membership through a trait that `delta-lang` implements | P1 |
| Q-44 ✅ | CLAUDE.md vs 02 §1 | CLAUDE.md: "every algorithm cites its Definition/Theorem/Lemma number in its doc comment". 02 §1: a CI grep forbids "Theorem/Lemma/Sipser/Definition" in `delta-core` | v2 doc comments describe the algorithm neutrally; citations live only in `delta-explain` profiles. The grep covers comments too. Update CLAUDE.md for v2 | P2 |
| Q-45 | 01 §13 | Only E/B/X ranges are defined, but v1 and v2 also have warnings and infos (`a+b` warning, shadowing, inferred Q/Σ) | `W` and `I` share the E range digits: W1xx syntax, W2xx names/shadowing, W3xx machine (e.g. inferred Q), W4xx regex | P1 |
| Q-46 | 01 §13 | After a budget outcome: does the statement or the whole program stop? Is `ok` false? Is the verdict three-valued (`unknown`)? | The program stops; already-produced outputs stay. B6xx is in `diagnostics`; `ok: false`; verdicts: Q-69 | P1 |
| Q-47 | 01 §8 | `print a, b` format: separator, quoting of words, set syntax | Space-separated; words unquoted; sets/maps canonical (`{1, 2, 3}`); `none` | P1 |
| Q-48 | 02 §7 | What does `ok` mean? Does a failing test make it false? | `ok` = no error diagnostics. Test results are in outputs; the CLI exits nonzero on errors **or** failed tests | P1 |
| Q-49 | 01 §13 vs v1 | v1 lenient mode made a missing DFA transition a **warning** (with an implicit trap). v2 §13 shows it as an E3xx error with "mark (partial)" as a fix | Error (`(partial)` is the opt-in). Conformance 37 assumes this | P2 |

## E. The plan vs the v1 code (04 §1, 02 §2.2)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-50 | 02 §2.2, 04 §1, P2 exit | "The v1 … **Hopcroft minimization**" and "algorithms ported from v1": v1 has **no minimization at all** | All four `minimize` methods are new work. Schedule them in P2 (fast paths) and raise P2's estimate | P2 |
| Q-51 | 01 §12 | v1 has no equivalence, inclusion, emptiness, finiteness, universality, shortest, words, count, Myhill–Nerode or pump; no complement, difference, reverse, trim, complete, remove_epsilon; no Thompson, Glushkov, Arden, Brzozowski–McCluskey or simplify | All new. Many conformance cases (§1 Taste among them) need equivalence, so it moves early in P3 or late in P2. The estimates look low | P3 |
| Q-52 | v1 c02 | v1's subset construction has a `full` mode (all 2^k subsets, Example 1.41). It isn't in the v2 library | `determinize(N, method: "full")` | P2 |
| Q-53 | 01 §12 | Are `M ∪ K` (operator) and `union(M, K)` (function) the same thing? `count` and `trim` are overloaded | Operators on machines give **langs** (symbolic). Functions give **machines**. Overloads are fine with signature records | P3 |
| Q-54 | 01 §12 | `words(L, upto: n)`: a set or a list? | A set (it prints in shortlex anyway, §1.2) | P3 |
| Q-55 | 01 §7.2, §7.3 | `G.rules`, `set(…)` and `len` on lists/sets are used but not listed (grammar members aren't specified at all) | Add `len` (words, lists, sets, maps), `set(list)`, `list(set)`, and `G.rules`/`G.variables`/`G.terminals`/`G.start` | P1/P6 |
| Q-56 | 01 §12, 02 §5 | `steps(x)` result shape; default `method` for `nfa(R)` | A list of typed steps; `to_nfa(R)` defaults to `"textbook"` (the v1 construction) | P2 |
| Q-57 | 01 §12 | `simplify(R)`: which identities? | v1's safe set (∅/ε identities, idempotence, (R*)* = R*) | P2 |
| Q-58 | 01 §5.5 | `M.run(w)` for nondeterministic and storage models: a tree, or configuration sets per step? v1's NFA "fingers" view doesn't extend to stacks | A list of configuration sets per step (BFS frontier), capped by the budget | P2 |
| Q-59 | 01 §4 | `lba extends tm` is **deterministic**, but CS = nondeterministic LBAs (deterministic LBAs ⊆ CS; equality is open) | `lba extends ntm { bounded }` | P6 |

## F. Models, configurations and acceptance (01 §4, 02 §2.1)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-60 | §4, §5.3 | Default for `missing`? How do `(partial)`, `total` and `missing dead` relate? Are options general property overrides? | Default `reject`. `(partial)` = drop `total` + `missing dead`. `(k: v)` overrides property k for that machine (SEMANTICS §5.8) | P2 |
| Q-61 | 02 §2.1 | Two-way input: initial head position, moving off the endmarkers, and acceptance (F anywhere? at ⊣? moving off the right end?) | Start on ⊢. Moves past the endmarkers don't apply. Accept on entering F anywhere (halts) | P6 |
| Q-62 ✅ | 02 §2.1 | Initial stack: empty (Sipser) or Z₀ (HMU)? Profiles "change reading and printing only", so they can't decide it | Empty. Machines push their own bottom marker; an optional `stack start Z` field | P6 |
| Q-63 | 02 §2.1 | Tape: infinite to the right only, head stays at the left end (Sipser), or infinite both ways (HMU)? | Right-infinite by default; `storage tape(two-way)` as an option | P6 |
| Q-64 | §5.2 | `on 1:` suggests tapes numbered from 1 | 1-based | P6 |
| Q-65 | §4 | `storage stack, stack`: how do labels address the second stack? | `on s2: pop X` (like `on 2:` for tapes) | P6 |
| Q-66 | §5.2, 02 §2 | Counters: only `IfZero` exists, so "test > 0 without changing" isn't expressible, which deterministic counter machines need. `if c2 == 0` is written as an expression inside a label | Add `IfPos` (`if c > 0`). Label tests are a fixed form, not general expressions | P6 |
| Q-67 | 02 §2.1 | Determinism over `labels regex` and other mixed guards | Compatibility rules in SEMANTICS §4.2 | P2 |
| Q-68 | §4, §12 | `mealy`/`moore` have no `accept`: what are `M.lang` and `M.accepts`? Does Moore emit λ(q₀)? `M.output(w)` vs `F.transduce(w)` (two names for one thing?). Is the output of a nondeterministic `fst` a set? | Accept by final with F = Q. Moore emits λ(q₀) first. Keep one name (`output`). `fst` outputs a set | P6 |
| Q-69 ✅ | §4 | `accept by halting`: halt in an accept state (Sipser), or halt at all? | Entering an accept state (F) accepts and halts; entering `reject` rejects and halts | P6 |
| Q-70 | §6.1 | Class from model is listed only for the prelude. What about user models, `ctr`, queues, `fst`? | The rule table in SEMANTICS §6.4 | P3 |

## G. Languages and classes (01 §6)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-71 | §6.2 | Complement needs an alphabet: which Σ does `∁L` use? What do `"d" ∈ re"[abc]"` and other symbols outside Σ give? | Each lang has Σ_L (SEMANTICS §7.1). ∁ is relative to Σ_L. A word with symbols outside Σ_L is simply not a member | P3 |
| Q-72 | §6.3 | Mixed operands (`check C ≡ P`, CF vs Decidable): which column? | The join (max) of the two classes | P3 |
| Q-73 | §6.3, 02 §3 | "Compile-time E701", but classes are unknown statically inside generic functions (`fn eq(L, K) = L ≡ K`) and across `if` branches | Static where known (join over branches). Otherwise the **same** E7xx code at run time | P3 |
| Q-74 | §6.3, 04 §3 | Only `≡ up to n` is specified, but "every E7xx error has a test with its bounded alternative", which ⊆, is_universal and is_empty also need | `up to n` is allowed on every `check` question | P3 |
| Q-75 ✅ | §6.2 | The closure table has no **Finite** column, no **Any** column, no rule for **mixed** classes, and no **difference** row (`∖`/`minus` and `difference` exist) | Join rule; Finite: ∪ ∩ ++ rev → Finite, ∁ and * → Regular, ∩ with anything → Finite; Any absorbs; ∖ = ∩ ∁ | P3 |
| Q-76 | §6.4 | ~~The spec's own example was false (`P as Regular by re"(01)*"`)~~ **RESOLVED 2026-11-08:** §6.4 now uses `lang E = { w ∈ {0,1}* \| count(w,'0') % 2 == 0 } as Regular by re"(1* 0 1* 0)* 1*"`. Still open: the bound used and what a failed claim does | Default bound 8. A failed claim is an E7xx error with the witness (conformance 19) | P3 |
| Q-77 | §6.3 | The E701 message says "context-free languages", although the join is Decidable. Where does the `up to 12` in the help come from? | The message names the question and both classes. The suggested bound is a setting, default 12 | P3 |

## H. `explore` and `fixpoint` (01 §7)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-80 ✅ | §7.1 | "alphabet: inferred from the inputs used" is undefined (in `determinize`, `a` is never "used" on an alphabet) | The union of `.alphabet` over the machine values the clauses refer to. If there are none and no clause, E2xx | P3 |
| Q-81 | §7.1 | Several `step` clauses that match the same state; patterns that match no state; literal patterns | All matching clauses add edges. A state no pattern matches gets no edges. Literal patterns are allowed | P3 |
| Q-82 | §7.1 | A deterministic `step` that yields `none` (partial) when the default model is `dfa` (total) | No edge; the result is treated as `(partial)` | P3 |
| Q-83 | §7.2 | "Least fixed point" holds only for a monotone, inflationary body; otherwise the result is just the first repeat | SEMANTICS §9.2. Rename the section "fixpoint iteration" and drop the warning idea if it's noisy | P3 |
| Q-84 | §7.1 | `edges q -> [(label, target), …]` for PDA/TM: labels as values need a `label` type, which §3 doesn't have (and `M.edges` records carry labels) | Add a `label` type, written in keyword or short form inside `label"…"` | P6 |

## H2. Raised while applying the confirmed answers (2026-11-08)

| # | Where | Problem | Recommended default | Blocks |
|---|---|---|---|---|
| Q-92 | 01 §6.2, SEMANTICS §7.5 | With L = ∁T (T Recognizable, so L is Any), `F ∩ L` gets **Recognizable** but `F ∖ T`, the same language, gets **Any**. Recognizable over-promises: a member w needs a proof that T does *not* accept w, which a run may never give | `F ∩ L` → Recognizable only for a Recognizable L, and → Any for an Any L. Conformance case 15 tests neither cell, so this changes one table cell only | P3 |
| Q-93 | 01 §5.3 | Option syntax: `(initial stack Z0)` has no colon, while `(missing: error)` and `(accept: by empty-stack)` do. How are several options combined? | Options are `,`-separated; `initial stack Z0` stays as written. Settle in the EBNF (Q-14) | P1 |

## H3. Raised in Phase 6 (2026-11-11)

| # | Where | Question | Answer |
|---|---|---|---|
| Q-94 | 01 §5.2, SEMANTICS §5.9 | How is a Moore state's output λ(q) written? `λ` already lexes as ε | **CONFIRMED 2026-11-11 (owner):** `λ(q) = w` lines (ASCII `lambda(q) = w`), like `δ(q, a) = p`; `λ` followed by `(` in a machine body is the output function, elsewhere it stays ε |
| Q-95 | SEMANTICS §11.6 | Breadth-first simulation never ends on a PDA whose ε-moves grow the stack, so `P.accepts(w)` could only end as `unknown` | Decided (C): membership in a one-stack, one-way machine's language goes through `to_cfg(P)` and Earley (always ends); `P.run(w)` still explores configurations |

## I. Process

| # | Problem | Recommended default |
|---|---|---|
| Q-90 | Where conformance tests live and how "failing" coexists with "all checks pass before committing" | `conformance/` + `crates/delta-cli/tests/conformance.rs` + `xfail.txt`: CI is green while every listed case fails, and goes red if a listed case starts passing. Strict run: `-- --ignored` (done in Phase 0) |
| Q-91 | Local `main` was 13 commits behind `origin/main` | Fast-forwarded to `origin/main` (`50bf153`) before tagging `v1.0-final` (done in Phase 0) |
