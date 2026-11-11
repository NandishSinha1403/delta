# Delta 2 — Semantics

The formal meaning of every model property, acceptance mode, label operation, `explore` and
`fixpoint`, and the core everything else desugars to (`01-LANGUAGE.md` §4, §7, §14;
`02-ENGINE.md` §2.1). Each numbered definition becomes executable tests (04 §3 "Semantics").

Where the spec is silent or contradicts itself, this document picks a default and tags it
**[Q-nn]**. The question is in `QUESTIONS.md`, and the default stands until you rule on it.

---

## 1. Preliminaries

**1.1 Words.** An *alphabet* is a finite, nonempty set of symbols. A *word* over Σ is a finite
sequence w = w₁…wₙ with wᵢ ∈ Σ. |w| = n, ε is the word of length 0, and Σ* is the set of all
words. Σ_ε = Σ ∪ {ε}. Indexing is 0-based in the language (`w[0]` is w₁) [Q-21].

**1.2 Canonical order.** Every value has a place in one total order ≼, used to print sets and
maps, to iterate them (`for`, generators) and to number the states of constructions. Values of
different types are ordered by type tag: `none < bool < int < symbol < atom < word < tuple < list < set <
map < record < regex < grammar < machine < lang < model < fn`. Within a type: `false < true`;
integers numerically; symbols and atoms by code points of their names; words **shortlex** (shorter first,
then lexicographic by symbol order); tuples, lists and records lexicographically by components (records
by sorted field names); sets and maps by their sorted element sequences. Functions are compared by
creation order (they are equal only to themselves). This order is part of the semantics:
deterministic output (02 §4) depends on it. [Q-22]

**1.3 Equality.** `==` is structural equality on values. Two machines are equal iff their models,
state sets, alphabets, edge *sets*, start states, accept sets and extras are equal. `==` is **not**
language equivalence; that is `≡` (§7.4).

**1.4 Budgets.** Every evaluation runs under a budget β = (steps, states, depth, output). An
evaluation either returns a value, raises an error (E5xx), or stops with a **budget outcome**
B6xx. A budget outcome is not an error in the program. Where a run of a machine is concerned it is
the verdict `unknown` (§6.2), and a decision that runs out of budget returns the budget outcome
instead of `true`/`false` (§7.3) [Q-46].

**1.5 Atoms.** An atom `:n` is a value made from a name n. Two atoms are equal iff their names are
equal; an atom is never equal to a symbol, word or other value (`:a ≠ 'a'`). A bare name in state
position inside a machine literal denotes the atom with that name (§10.3). Atoms can be map keys
(`{ :q0: 0 }`) and set elements like any other value.

---

## 2. Machines

**2.1 Machine.** A machine is a tuple

  M = (μ, Q, A, E, q₀, F, X)

where μ is a model (§5), Q is a finite set of values (the states), and A = (Σ, Γ, Δ) are the
input, storage and output alphabets (some may be ∅ when μ does not use them). E ⊆ Q × Λ × Q is a
finite set of **edges**, each labelled by a label from Λ (§4). q₀ ∈ Q is the start state, F ⊆ Q
the accept states, and X holds the extras: reject state q_r ∈ Q for tape models, blank ⊔ ∈ Γ, and
per-state outputs λ : Q → Δ* for `output per-state`.

**2.2 Well-formedness** (checked at construction; violations are E3xx): q₀ ∈ Q; F ⊆ Q; every edge's
endpoints are in Q; every symbol on a label is in the alphabet it belongs to (Read: Σ, or Γ for
on-tape input; Pop/Push/Write: Γ; Emit: Δ); and the model's properties hold (§5).

**2.3 Edges, not δ.** E is the primary object. `M.δ(q, a)` is derived:
- if μ is `deterministic` and one-way with no storage: the unique p with (q, Read a, p) ∈ E;
  if there is none, it is `none` [Q-23];
- otherwise the set { p | (q, ℓ, p) ∈ E, ℓ = Read a } (with `a = ε` selecting the label
  `Read ε`, i.e. a label with no Read in one-way mode).

For storage models, `M.δ(q, a)` is the set of (ℓ, p) pairs whose label reads a; it is mostly
useful for nfa-like models.

---

## 3. Configurations and computations

**3.1 Configuration.** For input w, a configuration is

  C = (q, h, s₁, …, s_m, o)

where q ∈ Q is the control state, h is the input head (§3.2), s₁…s_m are the contents of
the model's storages in order (§3.3), and o ∈ Δ* is the output so far (only when μ has output).

**3.2 Input modes.**
- **one-way:** h ∈ {0, …, |w|}. Initially 0. Only `Read` moves it, and only rightwards.
- **two-way:** the input is ⊢w⊣ with endmarkers ∉ Σ; h ∈ {0, …, |w|+1}, initially 0 (on ⊢)
  [Q-61]. `Read a` tests the symbol under the head without moving. `Move L|R|S` moves the head.
  A move off either endmarker is not applicable (the edge does not fire).
- **on-tape:** there is no separate input head. The input initializes the first tape (§3.3), and
  `Read a` tests the symbol under that tape's head.

**3.3 Storages** (initial contents in brackets):
- **stack** — a word over Γ, written top first [ε, or Z when the machine has the option
  `initial stack Z`; Z ∈ Γ (an inferred Γ includes it), otherwise E3xx].
- **tape** — a pair (l, r) with l ∈ Γ* (cells left of the head, nearest last) and r ∈ Γ⁺ (the
  head's cell and everything to its right; trailing blanks are implicit). For on-tape input the
  first tape starts as (ε, w) if w ≠ ε, else (ε, ⊔); other tapes start as (ε, ⊔). The tape is
  infinite to the right only: a `Move L` at the left end leaves the head where it is (02 §2.1)
  [Q-63].
- **tapes(k)** — k tapes as above, numbered **1…k** in labels (`on 1: …`) [Q-64].
- **queue** — a word over Γ, written front first [ε].
- **counters(k)** — (n₁, …, n_k) ∈ ℕᵏ, named `c1`…`ck` in labels [all 0].
- Several storages may be listed (`storage stack, stack`). Ops address them by position when the
  kind is ambiguous: `pop₁`, `push₂`, … [Q-65].

**3.4 Steps.** An edge (q, ℓ, p) is *applicable* in C = (q, …) iff ⟦ℓ⟧ (§4) is defined on C. Then
C ⊢ C′ where C′ = ⟦ℓ⟧(C) with control state p. ⊢* is the reflexive-transitive closure. A
configuration with no applicable edge is **stuck**. What being stuck means is decided by `missing`
(§5.8) and by the acceptance mode (§6).

The **computation tree** on w has the initial configuration C₀ = (q₀, 0, initial storages, ε) at
its root and the successors of each node as its children. The generic simulator explores it
breadth first with a visited set on configurations, under the budget (02 §2.2). The visited set
makes ε-loops and other configuration cycles terminate whenever the reachable configuration space
is finite. Otherwise the budget stops it with the verdict `unknown` (§6.2).

---

## 4. Labels and their operations

A label is a sequence of operations ℓ = o₁; o₂; …; o_k. Its meaning ⟦ℓ⟧ is the composition
⟦o_k⟧ ∘ … ∘ ⟦o₁⟧ of partial functions on configurations. ℓ is applicable iff every step is defined.
Composition is left to right, so `read a; pop X; push YZ` tests the input, then the stack, then
pushes.

| Op | Defined when | Effect |
|---|---|---|
| `Read a` (one-way) | h < \|w\| and w[h] = a | h ← h+1 |
| `Read a` (two-way) | the cell under h (of ⊢w⊣) is a | none |
| `Read a` (on-tape, tape t) | head cell of tape t is a | none |
| *no Read* (one-way, written `ε`) | always | none (an ε-move) |
| `Pop X` | stack = Xβ | stack ← β |
| `Push β` | always | stack ← β·stack (`push YZ` leaves Y on top) |
| `Write x` (tape t) | always | head cell of t ← x |
| `Move d` (tape t), d ∈ {L, R, S} | always | t's head moves; L at the left end stays; R past the last cell appends ⊔ |
| `Move d` (two-way input) | the new position is in 0…\|w\|+1 | h ← h±1 or h |
| `Enqueue β` | always | queue ← queue·β |
| `Dequeue X` | queue = Xβ | queue ← β |
| `Inc c` | always | n_c ← n_c + 1 |
| `Dec c` | n_c > 0 | n_c ← n_c − 1 |
| `IfZero c` | n_c = 0 | none |
| `IfPos c` | n_c > 0 | none [Q-66] |
| `Emit β` | always | o ← o·β |
| `Regex r` (one-way, `labels regex`) | some u ∈ L(r) with w[h…h+\|u\|) = u | h ← h+\|u\| (one successor per such u) |

**4.1 Well-formed labels.** In a one-way model, a label has at most one `Read`, and it comes first.
On-tape and two-way labels have at most one `Read` per tape/input, before any `Write`/`Move` on it.
A TM short form `0 → x, R` desugars to `read 0; write x; move R`, and `x → R` to `read x; move R`.
A PDA short form `a, X → YZ` desugars to `read a; pop X; push YZ`, where `ε` in any position means
"omit that op".

**4.2 Guards.** The *guard* of ℓ is the set of tests it makes on the configuration *before* any
effect: the input symbol (from `Read`), the stack top (from `Pop`), the queue front (`Dequeue`),
counter tests (`Dec` ⇒ n>0, `IfZero` ⇒ n=0, `IfPos` ⇒ n>0) and tape head cells (on-tape `Read`).
A test is on a **dimension** (input, stack 1, counter c2, tape 1, …). Two labels' guards are
**compatible** iff they agree on every dimension both of them test. A dimension tested by only one
label is compatible. `Read a` vs `Read b` with a ≠ b is incompatible, `IfZero c` vs `Dec c` is
incompatible, and `Regex r` is compatible with every Read [Q-67].

---

## 5. Model properties

A model μ is a record of properties (01 §4). `extends m { … }` copies m's properties and then
applies the body: a property written replaces the inherited one, and `not p` removes flag p.
Parameters (`mtm(k)`) are substituted before checking.

Each property is either a **static rule** on the machine (checked when it is built, giving E3xx)
or a **dynamic rule** (it changes ⊢ or acceptance).

**5.1 `input one-way | two-way | on-tape`** (dynamic): selects §3.2.

**5.2 `storage …`** (dynamic): selects §3.3; `none` means m = 0.

**5.3 `deterministic`** (static): for every state q, any two distinct edges (q, ℓ₁, p₁),
(q, ℓ₂, p₂) ∈ E have **incompatible** guards (§4.2). Equivalently, at most one edge is applicable in
every configuration. Consequence: the computation tree is a path.
*Note:* in one-way mode, an ε-label (no Read) is compatible with every Read on the input
dimension, so it is incompatible with another label only through other dimensions. This is the
usual DPDA condition.

**5.4 `total`** (static; meaningful only together with `deterministic`): for every state q that is
not halting (§6), and every combination of values on the dimensions μ tests, some edge from q has a
guard satisfied by it. The dimensions are: the input symbol a ∈ Σ (one-way: positions h < |w| only;
two-way: Σ ∪ {⊢, ⊣}), stack top ∈ Γ ∪ {empty}, counter zero-ness, and head cells ∈ Γ. For `dfa`
this is exactly "δ(q, a) is defined for every q ∈ Q and a ∈ Σ" (01 §4).
`total` without `deterministic` is the same condition with "at least one" instead of "exactly one".

**5.5 `epsilon`** (static): permits labels with no `Read` in one-way models. Without it, every
label of a one-way model contains exactly one `Read a` with a ∈ Σ. (It also permits `Regex r` with
ε ∈ L(r) when `labels regex` is set.)

**5.6 `labels regex` and `special-form`** (static): with `labels regex`, labels are `Regex r` with r
over Σ. `special-form` requires: (i) F = {q_a} with q_a ≠ q₀; (ii) q₀ has no incoming edges;
(iii) q_a has no outgoing edges; (iv) for every qᵢ ≠ q_a and qⱼ ≠ q₀ there is **exactly one** edge
qᵢ → qⱼ. An edge labelled ∅ may be left out of the source; the checker inserts it, so after building
(iv) holds literally.

**5.7 `bounded`** (dynamic; on-tape only): the head stays within the input cells 0…max(|w|,1)−1. A
`Move R` from the last input cell, or `Move L` from cell 0, leaves the head where it is.

**5.8 `missing reject | dead | error`** (dynamic; what a stuck, non-halting configuration means):
- `reject` — the branch ends and does not accept. A deterministic machine rejects w.
- `dead` — as if every missing (state, guard) combination led to an implicit non-accepting sink
  state q_⊥ with self-loops on everything. For one-way models the language is the same as with
  `reject`. The difference shows in `M.δ` (returns q_⊥), in diagrams and traces (q_⊥ is drawn
  dashed), and for two-way input (the head keeps moving right to ⊣).
- `error` — reaching a stuck configuration is a runtime error E5xx (with the configuration).
- **Default:** `reject` [Q-60].

The machine option `(partial)` means: `total` is removed for this machine and `missing` is set to
`dead`. `(missing: m)` and `(accept: …)` override that one property for this machine [Q-60].

**5.9 `output per-transition | per-state`** (dynamic): per-transition — the output is the
concatenation of the `Emit` ops along the run. Per-state — o starts as λ(q₀), and each step into
state p appends λ(p). λ is given by `λ(q) = w` lines (01 §5.2); λ(q) = ε for a state without one. So a Moore machine on w of length n emits n+1 outputs.
`M.output(w)` is defined for deterministic machines and returns o at the end of the run. For a
nondeterministic transducer (`fst`) it returns the **set** of outputs of accepting runs [Q-68].

---

## 6. Acceptance

Let C range over configurations reachable from C₀ on w (C₀ ⊢* C). Halting states are q ∈ F and q_r
for `by halting`, and none otherwise.

| Mode | M accepts w iff some reachable C … |
|---|---|
| `by final` (one-way) | … has q ∈ F and h = \|w\| |
| `by final` (two-way) | … has q ∈ F (at any head position; such a configuration halts) [Q-61] |
| `by empty-stack` | … has h = \|w\| and the (first) stack empty (q arbitrary) |
| `by final-and-empty` | … has q ∈ F, h = \|w\| and the stack empty |
| `by halting` | … has q ∈ F. Entering q ∈ F or q_r halts the branch immediately |

**6.1 Language.** L(M) = { w ∈ Σ* | M accepts w }.

**6.2 Verdict of a run.** The run of M on w has exactly one of four verdicts, written the same way
in traces (`M.run(w).verdict`, as the atoms `:accept`, `:reject`, `:loops`, `:unknown`), in JSON
output (`"verdict": "accept"`, …) and in the playground:

| Verdict | When |
|---|---|
| `accept` | an accepting configuration (per the table above) is reachable from C₀ |
| `reject` | not accept, and every branch **ends**: it reaches a configuration that halts (q_r, under `by halting`) or is stuck (no applicable edge, with `missing reject`) |
| `loops` | not accept, the reachable configurations are all explored, and some branch is proved never to end because a configuration repeats on it (a cycle in the configuration graph) |
| `unknown` | the budget runs out before one of the above is established (B6xx) |

`loops` arises only under `by halting`. In the other modes acceptance is reachability of an
accepting configuration, so a repeated configuration is simply not explored again: when everything
reachable has been explored without acceptance, the verdict is `reject`. A stuck configuration under
`missing error` is a runtime error E5xx, not a verdict.

`M.accepts(w)` (and `w ∈ M.lang`) is `true` for `accept`, `false` for `reject` and `loops`, and the
budget outcome for `unknown`.

**6.3 Models without `accept`.** `mealy` and `moore` declare no acceptance. Their `accept` is
treated as `by final` with F = Q, so L(M) = Σ* (for total machines) and `M.accepts` is always true
[Q-68].

**6.4 Language class of a model** (01 §6.1), extended to all models by rule [Q-70]:

| Model properties | Class |
|---|---|
| storage none, any input mode | Regular |
| one stack, deterministic, one-way | DCF |
| one stack, or counters(1), one-way | CF |
| on-tape, `bounded` | CS |
| on-tape, tape(s), queue, ≥ 2 stacks, counters(k ≥ 2) | Recognizable |

---

## 7. Languages

**7.1 Lang values.** `lang(c, s)` denotes a set ⟦s⟧ ⊆ Σ_s* together with a class c that is a
**guaranteed upper bound**: ⟦s⟧ ∈ c. Each source has an alphabet Σ_s [Q-71]:

| Source | ⟦s⟧ | Σ_s |
|---|---|---|
| finite set of words | the set | symbols occurring in it |
| regex r (`over S`) | L(r) | S, or the symbols of r |
| machine M | L(M) | M.alphabet |
| grammar G | L(G) | terminals of G |
| `{ w ∈ S* \| p(w) }` | { w ∈ S* \| p(w) = true } | S |
| `L op K` | §7.2 | Σ_L ∪ Σ_K |
| `L as c by E` | ⟦L⟧ (claimed equal to ⟦E⟧) | Σ_L |

**7.2 Operations.** ⟦L ∪ K⟧ = ⟦L⟧ ∪ ⟦K⟧, ⟦L ∩ K⟧ = ⟦L⟧ ∩ ⟦K⟧, ⟦∁L⟧ = Σ_L* ∖ ⟦L⟧,
⟦L ∖ K⟧ = ⟦L⟧ ∩ ⟦∁K⟧, ⟦L ++ K⟧ = { uv | u ∈ ⟦L⟧, v ∈ ⟦K⟧ }, ⟦L*⟧ = ⋃ₙ ⟦L⟧ⁿ, and
⟦rev L⟧ = { rev(u) | u ∈ ⟦L⟧ }. Complement is taken relative to Σ_L, so the alphabet matters:
∁(re"a*" over {a,b}) contains "b" [Q-71].

**7.3 Membership.** `w ∈ L` is `true`/`false` when decided, and the budget outcome (verdict
`unknown`) otherwise.
Decided means one of: finite set lookup, a machine run (§6.2), a grammar parse, or the predicate
returning a bool. A predicate that errors is an E5xx error. One that returns a non-bool is E2xx at
check time (or E5xx at runtime).

**7.4 Questions.** For a question Φ ∈ {∈, is_empty, is_finite, is_universal, ≡, ⊆} and operand
classes c₁, c₂, let c = c₁ ⊔ c₂ (the join, i.e. max in the chain) [Q-72]. Φ is allowed iff
`decidable(Φ, c)` is `Yes` in the decision table (01 §6.3). Otherwise it is error E7xx **at check
time** if c is known statically, and **at run time** with the same code if not [Q-73].
- `L ≡ K` is true iff ⟦L⟧ = ⟦K⟧. On failure, the witness is the ≼-least word (shortlex, §1.2)
  in the symmetric difference.
- `L ⊆ K` is true iff ⟦L⟧ ⊆ ⟦K⟧. The witness is the least word in ⟦L⟧ ∖ ⟦K⟧.
- **Bounded:** `L ≡ K up to n` is true iff for all w ∈ (Σ_L ∪ Σ_K)^≤n, w ∈ L ⇔ w ∈ K. It is
  allowed for every class (each membership is budgeted), and the result carries `bounded: n`.
  `L ⊆ K up to n` is defined the same way [Q-74].

**7.5 Class propagation.** Classes form the chain Finite < Regular < DCF < CF < CS < Decidable <
Recognizable < Any, and ⊔ is the max. T is the closure table of 01 §6.2. The class of a result is:

1. `F ∩ L` or `L ∩ F` with class(F) = Finite (if both are Finite, either one): Finite if
   class(L) ≤ Decidable, otherwise Recognizable.
2. `F ∖ L` with class(F) = Finite: Finite if class(L) ≤ Decidable, otherwise Any.
3. `L ∩ R` or `R ∩ L` with class(R) = Regular: T[∩R][class(L)].
4. `L ∖ K` otherwise: the class of `L ∩ K′`, where class(K′) = T[∁][class(K)], by rules 1, 3, 5.
5. Every other binary op: T[op][c₁ ⊔ c₂]. Unary ops: T[op][c].

The rules are tried in this order. Rules 1 and 2 stop at Decidable because computing F ∩ L or
F ∖ L asks `w ∈ L` for every w ∈ F, which is guaranteed to finish only for a Decidable or lower L.
Unary ops on Finite: ∁ and * give Regular, rev gives Finite. Every op with an `Any` operand gives
`Any`, except rule 1. ⟦L ∖ K⟧ = ⟦L⟧ ∩ ⟦∁K⟧ (§7.2) holds for the values in every case.

**7.6 Assumed classes.** `L as c by E` has class c and source `Assumed{L, c, E, n}`. At
construction it checks ⟦L⟧ ≡ ⟦E⟧ up to n (default n = 8) [Q-76]. If the check fails, the
declaration is an error E7xx with the witness. If it passes, every value computed from it carries
the `assumed` mark, and decisions that rely on c use ⟦E⟧ (a class-c object) as the representative.

---

## 8. `explore`

```
explore [μ] { alphabet S; start s₀; step clauses; accept π if φ }
```

**8.1 Meaning.** Let S be the value of the `alphabet` clause, which is required (a missing clause is
E1xx; a value that is not a nonempty finite set of symbols is E2xx/E5xx). The step clauses define a
successor relation on values:
- `step π on a -> e` (deterministic): for state v matching π and each symbol a ∈ S, one edge
  v →ᵃ ⟦e⟧.
- `step π on a -> any e`: edges v →ᵃ u for each u ∈ ⟦e⟧ (a set; ∅ adds none).
- `step π on ε -> any e`: ε-edges v →ᵉ u for each u ∈ ⟦e⟧.
- `edges π -> e`: e is a list of (label, target) pairs, with the label in keyword or short form.

The variable after `on` (here `a`) is bound to each symbol of S in turn. Patterns bind as in `let`.
If no clause's pattern matches a state, that state gets no edges from those clauses. If several
clauses match, the edges of all of them are added [Q-81].

Q is the least set with s₀ ∈ Q that is closed under successors. It is computed by BFS from s₀,
taking symbols in ≼ order and states in discovery order. F = { v ∈ Q | v matches π and φ is true }.
The result is the machine (μ, Q, (S, …), E, s₀, F, ∅).

**8.2 Model.** If μ is given, the result must satisfy μ's static properties (E3xx otherwise). If it
is omitted: `dfa` when there are only deterministic `step … on a -> e` clauses, and `nfa` otherwise.
A deterministic `step` that evaluates to `none` for some (v, a) creates no edge, so the result is
not total, and with μ = `dfa` it gets the `(partial)` treatment (§5.8) [Q-82].

**8.3 Budget and trace.** Each new state counts against `states`, and each evaluation of a clause
against `steps`. The trace records `AddState(v)` when v is discovered and `AddEdge(i)` when edge i
is added, in BFS order. Hitting the budget yields the B6xx outcome and keeps the partial machine
in the trace.

**8.4 Termination.** explore terminates iff the reachable set is finite (or the budget stops it).

---

## 9. `fixpoint`

```
fixpoint X from e₀ { e }
```

**9.1 Meaning.** X₀ = ⟦e₀⟧ and X_{k+1} = ⟦e⟧[X := X_k]. The result is X_k for the least k with
X_{k+1} = X_k (structural equality, §1.3). Each iteration counts against the `steps` budget. If no
such k is reached within budget, the result is the B6xx outcome (with the last X_k in the trace).

**9.2 Why "least".** If f(X) = ⟦e⟧[X] is monotone on a finite lattice of sets and X₀ ⊆ f(X₀), the
iteration is increasing and stops at the least fixed point of f that contains X₀ (Kleene). The
idiom `X ∪ …` in 01 §7.2 makes f inflationary, so this holds. For a non-monotone e, the result is
the first repeated value, if there is one, and is not a least fixed point. The checker warns when
it can see that the body does not contain `X ∪` [Q-83].

**9.3 Trace.** Each iteration records `Closure(k, X_k ∖ X_{k−1})` when both are sets, and
`Closure(k, X_k)` otherwise.

---

## 10. The core and desugaring (01 §14)

**10.1 Evaluation.** Programs evaluate top to bottom, strictly (call by value), left to right inside
expressions. `let x = e` binds x immutably in the rest of the block. `var x = e` declares a
rebindable local, and `x = e` rebinds the nearest enclosing `var x`. Inner blocks may rebind outer
`var`s (01 §7.3 relies on this); closures capture **values**, not variables [Q-24]. Assigning to a
`let` name or an undeclared name is E2xx. Shadowing with `let` in an inner scope is allowed and
warned (02 §4).

**10.2 Set-builder.** `{ e | g₁, …, gₙ }`, where each gᵢ is a generator `π ∈ S` or a condition `c`,
is defined by recursion on the list:
- `{ e | }` = { ⟦e⟧ };
- `{ e | π ∈ S, rest }` = ⋃_{v ∈ S in ≼ order, v matches π} `{ e | rest }`[π := v];
- `{ e | c, rest }` = `{ e | rest }` if ⟦c⟧ = true, else ∅.

`{ x ∈ S | c }` (the filter form) = `{ x | x ∈ S, c }`. The list form `[ e | gens ]` is the same
with ⋃ replaced by concatenation in generator order. `Σ*` and other infinite sets may appear as a
generator source only inside a `lang` set-builder, where they are never iterated (§7.1). Elsewhere
this is E2xx [Q-25].

**10.3 Machine literals** desugar to `machine μ { states …; alphabet …; start …; accept …;
edges […] }` (01 §5.6):
- `q -ℓ-> p` adds edge (q, ℓ, p); `q -[ℓ₁, …, ℓₖ]-> p` adds k edges (the list is read with the
  model's label grammar, so `-[a, X -> YZ]->` is one pda edge); `δ(q, a) = p` adds (q, a, p);
  `δ(q, a) = {p₁, …}` adds one edge per pᵢ; a `table` adds one edge per non-∅ cell.
- `for`/`if` inside a body expand to their edges (in iteration order). `[e]` in state position is
  the value of e, and a bare identifier `n` is the atom `:n` (§1.5). The headers of `for` and `if`
  are ordinary expressions (01 §2, "Machine bodies").
- In lenient mode, omitted `states` = all states mentioned in start, accept and edges, and omitted
  `alphabet` = all input symbols on edges. In strict mode, omitting either is E3xx.

**10.4 Statements.**
- `run M on w` ≡ `show M.run(w)`.
- `check M accepts w₁, …, wₙ` ≡ n tests `assert wᵢ ∈ M.lang` (and `rejects` with `∉`); each is one
  `test` output.
- `check A ≡ B [up to n]` ≡ one test of §7.4, recording the witness on failure. Machines, regexes
  and grammars are coerced to their `lang` [Q-27].
- `check A ⊆ B`, `check L.is_empty()` ≡ one test each.
- `assert c` ≡ a test that passes iff ⟦c⟧ = true. A non-bool is E2xx/E5xx.
- `save e to p` evaluates e, then p. p must be a string; its extension (after the last `.`, ASCII
  case-insensitive) is one of `svg`, `png`, `pdf`, `tex`, `txt`, `json`, else E514. p is split on
  `/` (and `\`); it must not be empty, start with `/` or `\`, contain a drive letter (`C:`), or
  have an empty, `.` or `..` part, else E515. The image formats and `tex` need a machine, else
  E516. The result is one output `{ kind: "save", line, path, format, value, text }`, where `path`
  is p with `\` turned into `/`, `format` the lower-case extension, `value` the `show` view of e
  and `text` what `print e` would print (the contents of a `.txt` file).
  Nothing is written by the engine: the host writes `path` relative to its working folder. Two
  `save`s to the same path both appear, in order; the host keeps the last.

**10.5 Statement separators.** A newline or `;` ends a statement in every context, including
machine and model bodies; `}` ends the last statement of a block (01 §2).

**10.6 Notation profiles** act only in the lexer and printer (01 §10): they map spellings to the
same core tokens (`+`→union under `hmu`, `λ`→ε, `B`/`□`→⊔), so no core value depends on the profile.

## 11. Grammar, transducer, regular and machine built-ins (01 §12)

Let G = (V, Σ, R, S), with rules in source order. Every result records its algorithm id and steps.

**11.1 `nullable(G)`** (`cfg/nullable`) is the least N ⊆ V with A ∈ N whenever A → X₁…Xₖ ∈ R and
every Xᵢ ∈ N (k = 0 included). **`useless(G)`** is V minus the variables that are both
generating (derive a terminal word) and reachable from S through generating rules only; `trim(G)`
removes them and every rule that mentions one. If S itself is useless, `trim(G)` has the single
variable S and no rules (L = ∅).

**11.2 `cnf(G)`** (`cfg/cnf`) applies, in order: **START** add S0 → S; **DEL** for each nullable
A ≠ S0 remove A → ε and add every rule obtained by deleting a subset of the nullable occurrences
on a right side (never producing B → ε for B ≠ S0; S0 → ε stays iff ε ∈ L(G)); **UNIT** replace
A → B by A → u for every non-unit B → u reachable through unit rules, then drop unit rules;
**TERM** in a right side of length ≥ 2, replace each terminal a by U_a with U_a → a; **BIN**
replace A → B₁…Bₖ (k ≥ 3) by A → B₁X₁, X₁ → B₂X₂, …, X_{k−2} → B_{k−1}Bₖ (numbering continues
across rules); finally `trim`. L(cnf(G)) = L(G). Each phase adds a `GrammarEdit` step.

**11.3 `parse(G, w)`.** `earley` (default, `parse/earley`): the Earley chart (items over G), then the tree
found by trying, for a variable over a span, its rules in source order and split points from left
to right, never re-entering the same (variable, span) on one path. `cyk` (`parse/cyk`): G' = G if
G is already in Chomsky normal form, else cnf(G); T[i][j] = { A | A ⇒* w[i..j] } by spans of increasing length, splits left to right, rules
in order, keeping the first back-pointer for each variable; the tree is read from S0 (or S) in
T[0][n]. Each filled cell adds a `CykCell` step. A tree node is a variable with its children; a
leaf is a terminal, or ε for an empty right side. `T.yield` is the word of the leaves.

**11.4 `derive(G, w)`** is the leftmost derivation of the `earley` tree: the list of sentential
forms, from [S] to w, each a word whose variables are single symbols.

**11.5 `to_pda(G)`** (`cfg/to-pda`): states :start, :loop, :accept; Γ = V ∪ Σ ∪ {$}; edges
`:start -ε, ε → S$-> :loop`, `:loop -ε, A → u-> :loop` for each A → u, `:loop -a, a → ε-> :loop`
for each a ∈ Σ, `:loop -ε, $ → ε-> :accept`; accept {:accept} by final state. L(to_pda(G)) = L(G).

**11.6 `to_cfg(P)`** (`pda/to-cfg`) for a one-stack, one-way machine P. First P is normalized to
P′: a new start :s₀ pushes $ (then P's initial stack symbol, if any) and moves to q₀; each edge is
split into a chain through fresh states so that every edge pushes exactly one symbol or pops
exactly one symbol (an edge that does neither pushes and pops a fresh symbol #); from each
accepting configuration (by P's acceptance mode) an ε-edge goes to :drain, which pops every
symbol except $, and `:drain -ε, $ → ε-> :acc`. Then the grammar has variables ⟨p,q⟩ for states p,
q of P′ and rules ⟨p,p⟩ → ε; ⟨p,q⟩ → ⟨p,r⟩⟨r,q⟩; ⟨p,q⟩ → a ⟨r,s⟩ b whenever p reads a (or ε),
pushes t and goes to r, and s reads b (or ε), pops t and goes to q. Start ⟨:s₀,:acc⟩; the result
is trimmed. L(to_cfg(P)) = L(P). Membership in a one-stack, one-way machine's language
(`P.accepts(w)`, `w ∈ P.lang`, `check P accepts …`) is decided this way, through to_cfg(P) and the
Earley recognizer, so it always ends; `P.run(w)` still explores configurations (§3).

**11.7 Transducers.** `mealy_to_moore(M)` (`mealy/to-moore`) for a deterministic Mealy machine:
states (q₀, ε) and (q, o) for every edge p -a / o-> q; λ((q, o)) = o, λ((q₀, ε)) = ε; an edge
(p, o′) -a-> (q, o) for each such edge and each state (p, o′). `moore_to_mealy(M)`
(`moore/to-mealy`): the same states, and p -a / λ(q)-> q for each edge p -a-> q. Both preserve
`output` as stated in 01 §12. Either on another model is E504.

**11.8 `gnf(G)`** (`cfg/gnf`): every rule becomes A → a B₁…Bₖ (k ≥ 0, a ∈ Σ, Bᵢ ∈ V), plus
S0 → ε iff ε ∈ L(G). Start from G′ = cnf(G); number its variables A₁…Aₙ in their order in G′
(start first). For i = 1…n: for each j < i replace every Aᵢ → Aⱼ γ by Aᵢ → δ γ for each Aⱼ → δ;
then remove immediate left recursion: if Aᵢ → Aᵢ α₁ | … | Aᵢ αₘ | β₁ | … | βₗ, add a fresh Zᵢ with
Aᵢ → βⱼ | βⱼ Zᵢ and Zᵢ → αⱼ | αⱼ Zᵢ. Then for i = n−1 down to 1 replace every Aᵢ → Aⱼ γ (j > i) by
the Aⱼ rules, and finally each Zᵢ → Aⱼ γ the same way. Fresh variables are Z1, Z2, … (skipping
names in use); the result is trimmed, and L(gnf(G)) = L(G). More rules than the states budget is
B602. Each phase adds a `GrammarEdit` step.

**11.9 `is_ambiguous(G, upto: n)`** (`cfg/ambiguity-bounded`) is a **bounded search**, never a
decision (ambiguity is undecidable). For each word w over Σ with |w| ≤ n, in shortlex order, it
counts the parse trees of w in G itself (not a normal form), saturating at 2: c(A, i, j) for each
variable and span is the least fixed point of c(A, i, j) = Σ over A → X₁…Xₖ and splits i = p₀ ≤ … ≤
pₖ = j of Π c(Xᵢ, pᵢ₋₁, pᵢ) (a terminal counts 1 on its one-symbol span), computed by rounds with
all values capped at 2, so infinitely many trees (a cycle A ⇒⁺ A) also count 2. The first w with
c(S, 0, |w|) = 2 is the witness. The result is a record: `ambiguous` (bool), `upto` (n), `word`
(the witness or none), `trees` (two distinct parse trees of the witness, or []) and `note`, which
always states the bound: "the word w has two parse trees (checked words up to length n)" or "no
ambiguous word up to length n (bounded check: longer words were not tested)". The two trees are
read from the rounds in which each count first reached 1 and 2, so they are finite. `upto:` is
required (E507); the work is bounded by the step budget (B601).

**11.10 `pump(D, w)`** (`regular/pump`) for a regular D (a dfa machine is used as given; any other
regular language is first made a minimal complete DFA) and a word w: p = |Q| of that DFA. If
w ∉ L(D), or |w| < p, it is E504 (the split is only guaranteed for words in L with |w| ≥ p). Let
r₀ … r_{|w|} be the run on w and j the least index with rⱼ = rᵢ for some i < j (j ≤ p by
pigeonhole). The result is the record `{p, x, y, z, state}` with x = w[0..i], y = w[i..j],
z = w[j..], and `state` = rᵢ (D's state value, or the class number for a constructed DFA). So
|xy| ≤ p, |y| ≥ 1 and x yᵏ z ∈ L(D) for every k ≥ 0.

**11.11 `myhill_nerode(D)`** (`regular/myhill-nerode`) for a regular D: the classes of the
right-congruence u ≡ v ⟺ ∀z (uz ∈ L ⟺ vz ∈ L), one per state of the minimal complete DFA
(reachable states only). Result: the record `{index, classes, separators}`. `index` is the number
of classes; `classes` lists, in shortlex order of their words, records `{word, accepting,
states}`: the shortlex-least word in the class, whether it is in L, and, when D is a dfa machine,
the set of D's reachable states in the class (else {}). `separators` maps each pair (u, v) of class
words (u before v) to the shortlex-least z with exactly one of uz, vz in L.

**11.12 `to_single_tape(M)`** (`tm/single-tape`) for a deterministic k-tape machine M (`mtm(k)`;
a one-tape `tm` is returned unchanged). The result is a `tm` whose cells hold either a symbol of
M's tape alphabet Γ (read as that symbol on track 1 and ⊔ on the others, no head marks) or a
track symbol `[s₁,…,sₖ]` with a head mark `^` after sᵢ when tape i's head is there, prefixed `▸`
on cell 0. States are tuples: (:init), (q, "collect", seen) with seen the symbols under the heads
found so far (`_` for not yet), (q, "apply", e, done) for M's edge e and the set of tapes already
updated, (q, "place", e, done, i) while moving tape i's mark one cell, (q, "return") and
:accept / :reject. One step of M is: from cell 0 sweep right collecting the k head symbols; if q
is an accept state go to :accept; if no edge of M applies go to :reject; else sweep left from
the rightmost mark, at each mark i (not yet done) write tape i's symbol and move the mark (L on
cell 0 stays), then walk back to cell 0 and collect again. L(to_single_tape(M)) = L(M); M halts
iff the result halts. Only reachable states and tape symbols are built; more states than the states
budget is B602.

