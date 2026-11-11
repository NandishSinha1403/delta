# 02 — Engine Architecture

One Rust engine, compiled to WebAssembly for the browser and the mobile apps, and natively for the CLI (and later the API and MCP server).

---

## 1. Crates

```
crates/
├─ delta-core      models, machines, languages, regexes, grammars, algorithms   (KEEP from v1, generalized)
├─ delta-lang      lexer, parser, resolver, checker, interpreter                 (NEW; replaces delta-dsl)
├─ delta-std       standard library: built-ins + the prelude (.delta source)     (NEW)
├─ delta-classes   the decidability-class lattice, closure and decision tables   (NEW)
├─ delta-explain   explain profiles (textbook citations as data)                 (NEW; content moved out of core)
├─ delta-render    layered layout, SVG, TikZ, ASCII (no dependencies)            (split out of core)
├─ delta-export    SVG → PNG/PDF (resvg, svg2pdf, embedded IBM Plex Mono)         (NEW, Phase 5b)
├─ delta-wasm      browser and app API                                           (KEEP; new surface)
├─ delta-export-wasm  the Pad's lazily loaded image export                       (NEW, Phase 5b)
└─ delta-cli       the `deltac` binary: run | repl | check | …                   (KEEP; extended)
```

**Rules, enforced in CI:**
- `delta-core` depends on no other Delta crate and contains **no textbook words** ("Theorem", "Lemma", "Sipser", "Definition"…). A test greps for them.
- `delta-explain` only reads stable algorithm and rule IDs.
- `delta-std`'s prelude is ordinary Delta source (`prelude.delta`), shown in the docs. Users can read how `dfa` or `pda` is defined.

---

## 2. The model system

Delta 1 had one struct per model. Delta 2 has **one generic machine** plus a **model descriptor** built from a handful of orthogonal building blocks.

```rust
pub struct Model {
    pub name: String,
    pub input: InputMode,          // OneWay | TwoWay | OnTape
    pub storage: Vec<Storage>,     // [] | [Stack] | [Tape] | [Tapes(k)] | [Queue] | [Counters(k)] | [Stack, Stack] …
    pub accept: Acceptance,        // ByFinal | ByEmptyStack | ByHalting | ByFinalAndEmpty
    pub output: OutputMode,        // None | PerTransition | PerState
    pub flags: ModelFlags,         // deterministic, total, epsilon, labels_regex, special_form, bounded
    pub missing: Missing,          // Reject | Dead | Error
}

pub struct Machine {
    pub model: Arc<Model>,
    pub states: IndexSet<Value>,     // any value can be a state
    pub alphabets: Alphabets,        // input, stack, tape, output
    pub edges: Vec<Edge>,            // from, Label, to
    pub start: StateIx,
    pub accept: BitSet,
    pub extra: Extra,                // reject state, blank, outputs per state
    pub provenance: Option<Provenance>,
}

pub struct Label {                   // generic: a sequence of operations
    pub ops: SmallVec<[Op; 3]>,      // Read(sym|ε), Pop(X), Push(word), Write(sym), Move(L|R|S),
}                                    // Inc(c), Dec(c), IfZero(c), Enqueue(word), Dequeue(X), Emit(word), Regex(id)
```

### 2.1 Semantics: one definition, every model

A **configuration** is `(state, input position, storage contents…)`. For each model property there is one short formal definition, written in `SEMANTICS.md` (Phase 0) and implemented once:

| Building block | Configuration part | A label's operations act on it |
|---|---|---|
| input one-way | position i | `Read a` needs w[i] = a, then i+1; `Read ε` leaves i |
| input two-way | position i with endmarkers ⊢ ⊣ | `Read a` + `Move` |
| input on-tape | the input is the initial tape | |
| stack | word (top first) | `Pop X` needs X on top; `Push YZ` |
| tape(s) | (left, right) per tape | `Write`, `Move`; the head stays at the left end |
| queue | word | `Dequeue X`, `Enqueue YZ` |
| counters(k) | k naturals | `Inc`, `Dec` (needs > 0), `IfZero` |
| output | emitted word | `Emit` |

**Acceptance** decides when a configuration accepts. **Determinism** is checked statically: at most one applicable move per configuration shape. **Totality** is checked statically: at least one move.

### 2.2 One simulator, plus fast paths

- **Generic simulator:** breadth-first search over configurations with a visited set and budgets. It handles every model, including user-defined ones.
- **Specialized fast paths**, chosen automatically when a model matches:
  - dense table for dfa;
  - bitsets for nfa;
  - two-stack tapes for tm;
  - CYK and Earley for grammars;
  - the v1 subset construction, Hopcroft minimization and state elimination.
- **Differential tests** run the generic simulator against every fast path on random machines, so the two can never disagree.

---

## 3. Languages and the class lattice (`delta-classes`)

```rust
pub enum Class { Finite, Regular, Dcf, Cf, Cs, Decidable, Recognizable, Any }   // a total order (lattice)

pub enum LangSource {
    FiniteSet(BTreeSet<Word>),
    Machine(Machine),          // class from the model
    Regex(ReId),
    Grammar(Grammar),
    Predicate(Closure),        // set-builder; membership runs the predicate under a budget
    Op(Box<LangOp>),           // ∪ ∩ ∁ ++ * rev of other languages (kept symbolic)
    Assumed { inner: Box<Lang>, claimed: Class, evidence: Box<Lang>, checked_upto: usize },
}

pub struct Lang { pub class: Class, pub source: LangSource }
```

- **Closure table** (`01-LANGUAGE.md` §6.2) is data: `fn result_class(op, &[Class]) -> Class`.
- **Decision table** (§6.3) is data: `fn decidable(question, Class) -> Decidability { Yes(algo) | YesNotImplemented | No(reason) }`.
- **The checker** propagates classes through expressions, so `check C ≡ P` is rejected **before running**, with error E701 and the bounded alternative.
- **Operations stay symbolic** until a question needs an answer. Then they're compiled to the cheapest machine that can answer it: for example, `R ∩ C` with R regular and C context-free becomes a PDA via the product construction.
- **Bounded checks** compare membership on all words up to length n, on both sides, budgeted. Results carry `bounded: n`.

---

## 4. Interpreter (`delta-lang`)

| Stage | Notes |
|---|---|
| Lexer | Unicode-aware; notation profiles change the token meaning of `+`, `λ`, `B`, `□` |
| Parser | recursive descent + Pratt parsing; **error recovery**: one bad line never blanks the output; newline-terminated statements with continuation rules |
| Resolver | scopes, shadowing warnings, "did you mean" |
| Checker | type inference (Hindley–Milner-lite over a small type language) + **class propagation** + model rules (E3xx) |
| Interpreter | tree-walking over a desugared core; persistent immutable values (`im`); `explore` and `fixpoint` are native nodes with budgets |
| Prelude | `prelude.delta` loaded first: the standard models, helper functions |
| Determinism | canonical ordering of sets and maps; no clock; randomness only seeded |

A tree-walker is enough because heavy work happens in built-ins. A bytecode VM is a later optimization, if profiling ever calls for one.

---

## 5. Standard library and traces (`delta-std`)

- Every built-in has a **signature record**: name, parameters, types, defaults, `method:` options, docs and examples. One record feeds:
  - the generated library reference;
  - editor autocomplete, hover and signature help;
  - the AI guide's reference section;
  - CI example tests.
- Conversions emit **typed trace steps** (`AddState`, `AddEdge`, `Closure`, `Rip`, `MarkPair`, `GrammarEdit`, `CykCell`, …) with neutral messages. Results keep **provenance**: algorithm, method and inputs. The Steps panel replays any of them.
- `explore` and `fixpoint` record traces too, so **user-written constructions are steppable** just like built-ins.

---

## 6. Explain profiles (`delta-explain`)

Textbook content is data, never code:

```toml
# profiles/sipser3e.toml
[profile]
name = "Sipser, Introduction to the Theory of Computation, 3rd ed."
[algorithm."determinize/subset"]
cite = "Theorem 1.39"
formula = "δ′(R, a) = ⋃ E(δ(r, a)), r ∈ R"
[rule."model/total"]
cite = "Definition 1.5"
[question."equivalent/cf"]
cite = "Exercise 5.1 (compare Theorem 5.13)"
```

Profiles: **off** (default), `sipser3e` (migrated from v1), `hmu`, `linz`, plus community pull requests. A profile only *adds* citations and wording; it never changes results.

---

## 7. Output model (pad, apps, CLI, future API: all the same)

```json
{
  "delta_version": "2.0.0",
  "ok": true,
  "diagnostics": [],
  "outputs": [
    { "kind": "print", "line": 14, "text": "true" },
    { "kind": "show",  "line": 15, "value": { "type": "machine", "model": "dfa", "name": "M",
        "states": ["q0","q1","q2"], "alphabet": ["a","b"], "start": "q0", "accept": ["q2"],
        "edges": [["q0","a","q1"]] } },
    { "kind": "test",  "line": 17, "name": "M ≡ L up to 10", "pass": true, "bounded": 10 },
    { "kind": "trace", "line": 18, "machine": "M", "input": "abab", "verdict": "accept",
        "steps": [{ "config": "q0 ▸abab" }, { "config": "q1 a▸bab" }] },
    { "kind": "save",  "line": 19, "path": "m.svg", "format": "svg", "value": { "type": "machine", … } }
  ],
  "stats": { "steps": 1342, "ms": 3 }
}
```

The schema is versioned and published at `/schema/output-v2.json`. A `save` output carries the
value, not the file: drawing it (SVG, TikZ, ASCII) is `delta-render`, and PNG/PDF is
`delta-export` (resvg + svg2pdf), compiled into `deltac` and into a second WASM module the Pad loads
only when an image is exported, so both produce the same bytes from the same view.

## 8. Platform boundary (`delta-wasm`)

```ts
interface DeltaEngine {
  run(src: string, opts?: { budget?: Budget; explain?: string }): Promise<RunResult>;
  check(src: string): Promise<Diagnostic[]>;            // on every keystroke (debounced)
  complete(src: string, offset: number): Promise<Completion[]>;
  hover(src: string, offset: number): Promise<Hover | null>;
  format(src: string, style?: "unicode" | "ascii"): Promise<string>;
  layout(machine: MachineJson): Promise<Layout>;         // Graphviz in the worker
  export(machine: MachineJson, fmt: "svg" | "dot" | "tikz" | "jff" | "json"): Promise<string>;
  importJflap(xml: string): Promise<string>;
}
```

The same WASM runs in the browser, inside the Android and iOS apps' WebViews, and (later) on Cloudflare Workers for the API. **One engine, four places.**

## 9. Performance targets

| Case | Target |
|---|---|
| Keystroke → diagnostics | < 50 ms (files < 300 lines) |
| Typical teaching program run | < 30 ms |
| Subset construction to 10⁵ states | < 1 s |
| User code loop | ≥ 5 M steps/s (WASM) |
| TM simulation | ≥ 20 M steps/s (WASM) |
| Diagram drawing | ≤ 300 states; above that, table view plus a message |
| WASM size | ≤ 900 KB gzipped (lazy-load Graphviz separately) |
