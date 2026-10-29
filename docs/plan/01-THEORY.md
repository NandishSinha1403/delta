# 01 — Theory: The Models, Their Tuples, and How We Code Them

This document is the contract between the textbook and the code. For each model it gives:

- the **formal definition** (quoted from the book's numbered definition),
- the **acceptance condition** (what it means to accept a string),
- the **configuration** (the snapshot we animate in the simulator),
- **validation rules** the engine enforces,
- the **Rust type** that represents it (full data-structure details are in `03-ARCHITECTURE.md`).

Notation: Σ_ε = Σ ∪ {ε}, Γ_ε = Γ ∪ {ε}, P(Q) = power set of Q.

---

## 0. Shared building blocks

| Concept | Book | Code |
|---|---|---|
| Alphabet | "a nonempty finite set" (§0.2) | `Alphabet { symbols: Vec<String> }`, interned to `SymId(u32)`. Symbols may be multi-character (`<EXPR>`, `RESET`) — Example 1.13 uses ⟨RESET⟩ as one symbol. |
| String | finite sequence of symbols | `Vec<SymId>` (never raw `&str` inside the engine) |
| Language | set of strings | Not stored. Represented by a machine; questions answered by algorithms |
| State | element of Q | `StateId(u32)` + `StateInfo { name, origin }` |
| Set of states | element of P(Q) | `BitSet` (fixed-width bit vector, fast union/hash) |

**Provenance (`origin`).** Every state created by a conversion remembers why it exists — e.g. `Subset({1,3})`, `NewStart("Theorem 1.45")`, `Pair(q1, p2)`, `Triple(A_pq)`. The UI uses this to explain each state ("this DFA state is E({1}) = {1,3}").

---

## 1. Deterministic finite automaton (DFA) — Definition 1.5

> A finite automaton is a 5-tuple (Q, Σ, δ, q₀, F), where
> 1. Q is a finite set called the states,
> 2. Σ is a finite set called the alphabet,
> 3. δ : Q × Σ → Q is the transition function,
> 4. q₀ ∈ Q is the start state, and
> 5. F ⊆ Q is the set of accept states.

**Acceptance (p. 40).** M accepts w = w₁…wₙ if there is a sequence of states r₀,…,rₙ with
1. r₀ = q₀, 2. δ(rᵢ, wᵢ₊₁) = rᵢ₊₁ for i = 0..n−1, 3. rₙ ∈ F.

**Configuration:** `(state, position in input)`. A run is the sequence r₀…rₙ — exactly what we highlight.

**Validation (strict mode):**
- δ must be **total**: exactly one transition for every (q, a). The book is explicit (p. 36): "exactly one transition arrow exits every state for each possible input symbol".
- F = ∅ is allowed (p. 36). Q must be non-empty (q₀ ∈ Q).
- No ε-transitions.

**Lenient mode:** many instructors draw partial DFAs with an implicit "dead" state. We allow `partial` DFAs only when explicitly marked, and offer a one-click **"complete with trap state"** fix. Complement (swap F) is only correct on a *complete* DFA — the engine auto-completes first and says so.

**Code:**
```rust
pub struct Dfa {
    pub states: Vec<StateInfo>,        // Q
    pub sigma: Alphabet,               // Σ
    pub delta: Vec<StateId>,           // δ as a dense |Q|×|Σ| row-major table
    pub start: StateId,                // q0
    pub accept: BitSet,                // F
}
// δ(q, a) == delta[q.0 as usize * sigma.len() + a.0 as usize]
```
Dense table = one array index per input symbol → simulating a DFA is a tight loop with no hashing.

---

## 2. Nondeterministic finite automaton (NFA) — Definition 1.37

> A nondeterministic finite automaton is a 5-tuple (Q, Σ, δ, q₀, F), where
> 1. Q is a finite set of states, 2. Σ is a finite alphabet,
> 3. δ : Q × Σ_ε → P(Q) is the transition function,
> 4. q₀ ∈ Q is the start state, and 5. F ⊆ Q is the set of accept states.

**Acceptance (p. 54).** N accepts w if w = y₁…yₘ with each yᵢ ∈ Σ_ε and states r₀…rₘ exist with r₀ = q₀, rᵢ₊₁ ∈ δ(rᵢ, yᵢ₊₁), rₘ ∈ F.

**Configuration:** the *set* of active states after each symbol (the book's "fingers", §1.2), always ε-closed. For the computation-tree view (Figure 1.28/1.29) we also keep each branch.

**Validation:** δ(q, a) may be ∅ (no arrow) — that branch dies. ε-loops are legal (closure handles them).

**DFA ⊆ NFA.** "Every deterministic finite automaton is automatically a nondeterministic finite automaton" (p. 47). Formally δ_N(q, a) = {δ_D(q, a)}, δ_N(q, ε) = ∅. That is the **DFA → NFA** conversion — trivial but we expose it so the user can then apply NFA operations (union, concat, star).

**Code:**
```rust
pub enum Label { Eps, Sym(SymId) }
pub struct Nfa {
    pub states: Vec<StateInfo>,
    pub sigma: Alphabet,
    pub delta: Vec<Vec<(Label, StateId)>>,   // per-state outgoing edges
    pub start: StateId,
    pub accept: BitSet,
}
```
For heavy algorithms we build a compressed (CSR) index per label once, so the subset construction does not scan edge lists repeatedly.

---

## 3. Generalized NFA (GNFA) — Definition 1.64

> A generalized nondeterministic finite automaton is a 5-tuple (Q, Σ, δ, q_start, q_accept), where
> 1. Q is the finite set of states, 2. Σ is the input alphabet,
> 3. δ : (Q − {q_accept}) × (Q − {q_start}) → R is the transition function (R = all regular expressions over Σ),
> 4. q_start is the start state, and 5. q_accept is the accept state.

**Special form (p. 70)** — the engine *always* keeps GNFAs in it:
- q_start has arrows to every other state and **no incoming** arrows.
- Exactly one accept state, q_accept ≠ q_start, with arrows **in** from every other state and **no outgoing** arrows.
- Every other pair (including self-loops) has exactly one arrow. Missing arrows are labeled ∅ (drawn hidden in the UI, as the book does in Figure 1.67).

**Acceptance (p. 73).** w = w₁…w_k with each wᵢ ∈ Σ*, states q₀ = q_start, …, q_k = q_accept, and wᵢ ∈ L(δ(qᵢ₋₁, qᵢ)).

**Code:**
```rust
pub struct Gnfa {
    pub states: Vec<StateInfo>,
    pub sigma: Alphabet,
    pub start: StateId,
    pub accept: StateId,
    pub label: Vec<ReId>,          // dense |Q|×|Q| matrix of regex ids; ∅ where no arrow
    pub arena: ReArena,            // hash-consed regex nodes (shared)
}
```
Dense matrix because the special form is *complete* by definition.

---

## 4. Regular expressions — Definition 1.52

> R is a regular expression if R is
> 1. a for some a ∈ Σ, 2. ε, 3. ∅, 4. (R₁ ∪ R₂), 5. (R₁ ∘ R₂), 6. (R₁*).

**Precedence (p. 64):** star, then concatenation, then union.
**Shorthands (p. 65):** R⁺ = RR*, Rᵏ = R concatenated k times, Σ = (a₁ ∪ … ∪ aₙ).
**Danger:** in Sipser, `+` is the *plus* operator (R⁺), **not union**. Many tools use `+` for union. Our parser follows the book: `∪` or `|` = union, `+` (postfix) = one-or-more. A diagnostic warns when `a+b` looks like intended union.

**Identities used by the simplifier (p. 66):** R ∪ ∅ = R, Rε = R, R∅ = ∅ (Example 1.53 #11), ∅* = ε (#12). Note that R ∪ ε ≠ R and R∅ ≠ R in general — the simplifier must never apply those.

**Code:** a hash-consed arena so equal sub-expressions are stored once (essential: state elimination produces heavily shared expressions).
```rust
pub enum Re { Empty, Eps, Sym(SymId), Union(Box<[ReId]>), Concat(Box<[ReId]>), Star(ReId) }
pub struct ReArena { nodes: Vec<Re>, dedup: FxHashMap<Re, ReId> }
```
Union and Concat are n-ary (flattened) — makes simplification and printing easy. Printing uses precedence to emit **minimal parentheses** (`a*b(a∪b)*`, not `a*(b((a∪b)*))`).

---

## 5. Context-free grammar (CFG) — Definition 2.2

> A context-free grammar is a 4-tuple (V, Σ, R, S), where
> 1. V is a finite set called the variables,
> 2. Σ is a finite set, disjoint from V, called the terminals,
> 3. R is a finite set of rules, each a variable and a string of variables and terminals, and
> 4. S ∈ V is the start variable.

**Derivation (p. 104):** uAv ⇒ uwv if A → w ∈ R; L(G) = {w ∈ Σ* | S ⇒* w}.

**Book conventions we adopt (p. 104):** when only rules are written, the variables are the symbols on left-hand sides, the terminals are everything else, and the start variable is the left-hand side of the first rule.

**Chomsky normal form — Definition 2.8:** every rule is A → BC or A → a, where B, C are not the start variable; additionally S → ε is permitted.

**Code:**
```rust
pub enum GSym { Var(VarId), Term(SymId) }
pub struct Rule { pub lhs: VarId, pub rhs: Vec<GSym> }   // empty rhs = ε
pub struct Cfg { pub vars: Alphabet, pub terms: Alphabet, pub rules: Vec<Rule>, pub start: VarId }
```
Derivations and parse trees are separate structures (`ParseTree { node: GSym, children }`).

---

## 6. Pushdown automaton (PDA) — Definition 2.13

> A pushdown automaton is a 6-tuple (Q, Σ, Γ, δ, q₀, F), where Q, Σ, Γ, F are finite sets and
> 1. Q is the set of states, 2. Σ is the input alphabet, 3. Γ is the stack alphabet,
> 4. δ : Q × Σ_ε × Γ_ε → P(Q × Γ_ε) is the transition function,
> 5. q₀ ∈ Q is the start state, 6. F ⊆ Q is the set of accept states.

**Diagram label "a, b → c" (p. 114):** read a, pop b, push c; any of them may be ε.

**Acceptance (p. 114).** w = w₁…wₘ (wᵢ ∈ Σ_ε), states r₀…rₘ and stack strings s₀…sₘ ∈ Γ* with
1. r₀ = q₀ and s₀ = ε (the stack starts **empty** — there is no built-in bottom marker; machines push `$` themselves),
2. (rᵢ₊₁, b) ∈ δ(rᵢ, wᵢ₊₁, a) where sᵢ = at and sᵢ₊₁ = bt for some a, b ∈ Γ_ε, t ∈ Γ*,
3. rₘ ∈ F. **Acceptance is by final state at end of input**, not by empty stack.

**Configuration:** `(state, input position, stack)`. Nondeterministic → the simulator explores a configuration graph (see `02-CONVERSIONS.md` §S3).

**Shorthand (p. 118):** (r, u) ∈ δ(q, a, s) with |u| > 1 means "push the whole string u" (u₁ ends up on top). We store push strings natively and *expand* to intermediate states only when the user asks for strict Definition 2.13 form.

**Code:**
```rust
pub struct PdaEdge { pub from: StateId, pub read: Option<SymId>, pub pop: Option<StkId>,
                     pub to: StateId, pub push: Vec<StkId> }   // push[0] = new top
pub struct Pda { pub states: Vec<StateInfo>, pub sigma: Alphabet, pub gamma: Alphabet,
                 pub edges: Vec<PdaEdge>, pub start: StateId, pub accept: BitSet }
```

### 6.1 Deterministic PDA — Definition 2.39
δ : Q × Σ_ε × Γ_ε → (Q × Γ_ε) ∪ {∅}, with the condition: for every q ∈ Q, a ∈ Σ, x ∈ Γ, **exactly one** of δ(q,a,x), δ(q,a,ε), δ(q,ε,x), δ(q,ε,ε) is not ∅. The engine implements this as a checker (`is_dpda()`), reporting the offending (q, a, x) triple.

---

## 7. Turing machine (TM) — Definition 3.3

> A Turing machine is a 7-tuple (Q, Σ, Γ, δ, q₀, q_accept, q_reject), where Q, Σ, Γ are finite sets and
> 1. Q is the set of states,
> 2. Σ is the input alphabet not containing the blank symbol ⊔,
> 3. Γ is the tape alphabet, where ⊔ ∈ Γ and Σ ⊆ Γ,
> 4. δ : Q × Γ → Q × Γ × {L, R} is the transition function,
> 5. q₀ ∈ Q is the start state, 6. q_accept ∈ Q is the accept state, and
> 7. q_reject ∈ Q is the reject state, where q_reject ≠ q_accept.

**Computation (p. 168–169):**
- Input on the leftmost n cells, rest blank, head on the leftmost cell.
- Moving L at the left end: **the head stays put** for that move.
- Halts on entering q_accept or q_reject; otherwise may run forever (outcomes: accept / reject / loop).
- **Configuration** u q v: tape is uv, head on first symbol of v. "ua qᵢ bv yields u qⱼ acv if δ(qᵢ,b) = (qⱼ,c,L)" and "yields uac qⱼ v if δ(qᵢ,b) = (qⱼ,c,R)". We display configurations in exactly this notation.

**Diagram conventions (p. 172–173):**
- `0→⊔,R` means read 0, write ⊔, move R. Shorthand `0→R` means write back the same symbol. `0,1→R` = both symbols.
- "We don't show the reject state or the transitions going to the reject state… the head moves right in each of these transitions." → **missing transitions implicitly go to q_reject, moving R.** The engine fills these in and can show/hide them.

**Validation:** ⊔ ∉ Σ; Σ ⊆ Γ; q_accept ≠ q_reject; no outgoing transitions from halting states (warning, they are ignored).

**Code:**
```rust
pub enum Move { L, R, S }      // S only for the stay-put variant
pub struct Tm {
    pub states: Vec<StateInfo>, pub sigma: Alphabet, pub gamma: Alphabet, pub blank: SymId,
    pub delta: Vec<Option<(StateId, SymId, Move)>>,   // dense |Q|×|Γ|; None = implicit reject
    pub start: StateId, pub accept: StateId, pub reject: StateId,
}
pub struct Tape { left: Vec<SymId>, right: Vec<SymId> }   // two stacks; head = top of `right`
```

### 7.1 Variants (§3.2) — all supported by the simulator
| Variant | Book | Model change | Engine note |
|---|---|---|---|
| Stay-put | §3.2 | moves ∈ {L, R, S} | convertible to plain TM (S → R then L), the book's argument |
| Multitape | Theorem 3.13 | δ : Q × Γᵏ → Q × Γᵏ × {L,R,S}ᵏ | simulate natively; optional view of the single-tape encoding (`#`-separated, dotted head symbols) |
| Nondeterministic | Theorem 3.16 | δ : Q × Γ → P(Q × Γ × {L,R}) | breadth-first search over the computation tree (as the proof requires — DFS can get stuck on an infinite branch) |
| Enumerator | §3.2 | TM with a printer | output panel lists printed strings |
| Linear bounded automaton | Definition 5.6 | head may not leave the input region | head stays put at both ends; finite configuration count = q·n·gⁿ (Lemma 5.8) → loop detection is exact |

---

## 8. Model hierarchy (what converts to what)

```
 DFA ⊂ NFA ⊂ GNFA  ≡  regex          (all = regular languages, Thm 1.39, Thm 1.54)
 regular ⊂ DCFL ⊂ CFL                (DFA ⊂ DPDA ⊂ PDA ≡ CFG, Thm 2.20)
 CFL ⊂ decidable ⊂ Turing-recognizable  (Thm 4.9; TM ≡ multitape ≡ NTM, Thm 3.13/3.16)
```

The engine enforces this with *explicit* conversion functions — there is no silent coercion. If the user runs `union(D1, D2)` on two DFAs, they choose between the product construction (DFA result, Theorem 1.25) and the NFA construction (NFA result, Theorem 1.45).

---

## 9. Configurations summary (what the simulator animates)

| Model | Configuration | Deterministic? | Termination guarantee |
|---|---|---|---|
| DFA | (q, i) | yes | always halts after n steps |
| NFA | (S ⊆ Q, i) | set-simulation | always halts |
| PDA | (q, i, stack) | no — graph search | **not guaranteed** (ε-moves can push forever) → budgets + cycle detection |
| TM | u q v | yes | **not guaranteed** (halting problem, Thm 4.11) → step budget |
| NTM | tree of u q v | no — BFS | not guaranteed → budget |
| LBA | u q v, bounded | yes | decidable (Thm 5.9) → exact loop detection |
