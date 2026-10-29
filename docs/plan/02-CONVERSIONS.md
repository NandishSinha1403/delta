# 02 — Conversions, Constructions and Algorithms

Each entry: **book reference → input/output → construction (pseudocode faithful to the book) → complexity → pitfalls → golden test**.
IDs (`C1`, `C2`, …) are used in the roadmap and in the code (`delta_core::convert::c02_subset`, etc.).

**Two modes for every conversion**
- **Faithful** — exactly the book's construction: same states, same labels, no clean-up. This is what students compare against homework.
- **Optimized** — same language, smaller/cleaner output (unreachable states removed, regexes simplified, useless variables pruned). Always labeled "beyond the book" in the UI.

**Every conversion returns `(Output, Trace)`.** The trace is a list of typed steps the UI can scrub through (see §T at the end).

---

## A. Regular languages (Chapter 1)

### C1 — DFA → NFA (p. 47, "every DFA is automatically an NFA")
- δ_N(q, a) = {δ_D(q, a)} for a ∈ Σ; δ_N(q, ε) = ∅. Q, Σ, q₀, F unchanged.
- O(|Q|·|Σ|). Exists so users can apply NFA-only constructions (C7) to a DFA.

---

### C2 — NFA → DFA, the subset construction (Theorem 1.39)

Given N = (Q, Σ, δ, q₀, F), build M = (Q′, Σ, δ′, q₀′, F′):

| Part | Book definition |
|---|---|
| Q′ | P(Q) |
| δ′(R, a) | { q ∈ Q \| q ∈ E(δ(r, a)) for some r ∈ R } |
| q₀′ | E({q₀}) |
| F′ | { R ∈ Q′ \| R contains an accept state of N } |

E(R) = states reachable from R along 0 or more ε arrows (including R itself).

```text
eps_closure(R):                       # iterative DFS, BitSet in/out
    stack ← elements of R; seen ← R
    while stack not empty:
        q ← pop(stack)
        for r in δ(q, ε): if r ∉ seen: seen ← seen ∪ {r}; push(stack, r)
    return seen

subset_construction(N, mode):
    start ← eps_closure({q0})
    if mode = FULL_POWERSET:          # faithful to Example 1.41: all 2^k subsets
        Q′ ← every subset of Q (refuse if k > 16)
    else:                              # REACHABLE (default): worklist
        Q′ ← {start}; work ← [start]
        while work not empty:
            R ← pop(work)
            for a in Σ:
                T ← eps_closure( ⋃_{r∈R} δ(r, a) )
                if T ∉ Q′: add T to Q′, push(work, T)
                δ′(R, a) ← T
    F′ ← { R ∈ Q′ | R ∩ F ≠ ∅ }
```

- **Complexity:** worst case O(2ⁿ · |Σ| · n) — unavoidable (there are NFAs whose minimal DFA has 2ⁿ states). Use `BitSet` keys in an `FxHashMap<BitSet, StateId>`; precompute ε-closure of each single state once and OR them.
- **Pitfalls:**
  - ∅ is a legitimate DFA state (the dead state). Keep it — Definition 1.5 needs δ′ total.
  - Apply E after every step *and* to the start state (the book applies ε-closure "after every step", p. 56).
  - Budget: stop at `max_states` (default 100 000) with a clear "too large" result, never hang.
- **Golden test (Example 1.41, N₄, Q = {1,2,3}, Σ = {a,b}, start 1, F = {1}; edges 1-b→2, 1-ε→3, 2-a→{2,3}, 2-b→3, 3-a→1):**
  - FULL mode: 8 states; start E({1}) = {1,3}; accept states {1}, {1,2}, {1,3}, {1,2,3}.
  - REACHABLE mode: 6 states (Figure 1.44, i.e. {1} and {1,2} removed): {1,3}, {2}, {2,3}, {3}, ∅, {1,2,3}; accept {1,3}, {1,2,3}.
  - Transitions: {1,3}-a→{1,3}, {1,3}-b→{2}, {2}-a→{2,3}, {2}-b→{3}, {2,3}-a→{1,2,3}, {2,3}-b→{3}, {3}-a→{1,3}, {3}-b→∅, {1,2,3}-a→{1,2,3}, {1,2,3}-b→{2,3}, ∅-a,b→∅. *(Verified with a prototype.)*

---

### C3 — Regular expression → NFA (Lemma 1.55)

Structural recursion over Definition 1.52, using **Sipser's** constructions (not Thompson's — they differ, and the state count must match the book's figures):

| Case | NFA |
|---|---|
| R = a | ({q₁,q₂}, Σ, δ, q₁, {q₂}) with δ(q₁, a) = {q₂} |
| R = ε | ({q₁}, Σ, δ, q₁, {q₁}), no transitions |
| R = ∅ | ({q}, Σ, δ, q, ∅), no transitions |
| R₁ ∪ R₂ | **Theorem 1.45:** new start q₀ with δ(q₀, ε) = {q₁, q₂}; F = F₁ ∪ F₂ |
| R₁ ∘ R₂ | **Theorem 1.47:** start q₁; F = F₂; for q ∈ F₁: δ(q, ε) = δ₁(q, ε) ∪ {q₂} |
| R₁* | **Theorem 1.49:** new start q₀ which is *also accepting*, δ(q₀, ε) = {q₁}; for q ∈ F₁: δ(q, ε) = δ₁(q, ε) ∪ {q₁}; F = {q₀} ∪ F₁ |

- Shorthands are desugared first: R⁺ → RR*, Rᵏ → R…R, Σ → (a₁ ∪ … ∪ aₙ). n-ary unions/concats fold **left** (matches the book's figures).
- **Complexity:** O(|R|) states and edges.
- **Pitfall (Exercise 1.15):** the "just make the start state accepting" shortcut for star is **wrong** — it can add unwanted strings. Our implementation adds the new start state exactly as Theorem 1.49 does; a unit test documents a counterexample for the wrong version.
- **Golden tests (verified with a prototype):**
  - Example 1.56: (ab ∪ a)* → **8 states** ("the procedure gives an NFA with eight states").
  - Example 1.58: (a ∪ b)*aba → **12 states**.

---

### C4 — DFA (or NFA) → GNFA in special form (proof of Lemma 1.60, p. 71)

```text
to_gnfa(M):
    add new q_start with an ε arrow to M's start state
    add new q_accept with ε arrows from every old accept state
    merge parallel arrows: label(qi, qj) ← union of all symbols (and ε) on arrows qi → qj
    every remaining pair (qi, qj) with qi ≠ q_accept, qj ≠ q_start gets label ∅
```
- Works for NFAs too (ε edges become the regex ε). O(|Q|² + |δ|).
- UI hides ∅ arrows by default (as the book does in Figure 1.67) with a toggle to show them.

### C5 — GNFA → Regular expression: `CONVERT(G)` (Lemma 1.60, p. 73)

```text
CONVERT(G):
    k ← |Q|
    if k = 2: return label(q_start, q_accept)
    choose q_rip ∈ Q − {q_start, q_accept}
    for each qi ∈ Q′ − {q_accept}, qj ∈ Q′ − {q_start}   (Q′ = Q − {q_rip}, includes qi = qj):
        R1 ← δ(qi, q_rip); R2 ← δ(q_rip, q_rip); R3 ← δ(q_rip, qj); R4 ← δ(qi, qj)
        δ′(qi, qj) ← (R1)(R2)*(R3) ∪ (R4)
    return CONVERT(G′)
```
- Implemented as a loop (no recursion), each iteration one trace step `Rip { q_rip, updates: [(qi, qj, R1, R2, R3, R4, new)] }`.
- **Rip order** changes the size of the output (not the language). Options: `book` (user-specified order, e.g. Example 1.66 rips 2 then 1), `index` (state order), `heuristic` (default in optimized mode: rip the state minimizing (in-degree × out-degree) and label size first).
- **Faithful vs simplified:** faithful mode builds (R1)(R2)*(R3) ∪ (R4) literally, so the student sees `(b)(a∪b)*(ε) ∪ ∅`. Simplified mode applies only the identities from p. 66 (R∪∅=R, Rε=R, R∅=∅, ∅*=ε, ε*=ε) plus idempotence R∪R=R and (R*)*=R*. Both are shown side by side for each step.
- **Complexity:** O(|Q|³) regex constructions; regex *size* can grow exponentially in |Q| (inherent). Hash-consing keeps memory proportional to distinct sub-expressions.
- **Golden test (Example 1.66):** DFA 1-a→1, 1-b→2, 2-a,b→2, F={2}, rip 2 then 1 ⇒ after ripping 2 the label 1→a is b(a∪b)*; final result **a\*b(a∪b)\***. *(Verified with a prototype.)*
- **Pitfall:** a pretty-printer that does not respect precedence produces `a*(b((a∪b)*))`. Printing must use minimal parentheses by precedence (star > concat > union).

---

### C6 — Product construction: ∪, ∩, −, ⊕ on DFAs (Theorem 1.25)

M = (Q₁ × Q₂, Σ, δ, (q₁, q₂), F) with δ((r₁, r₂), a) = (δ₁(r₁, a), δ₂(r₂, a)).

| Operation | F |
|---|---|
| Union (Thm 1.25) | (F₁ × Q₂) ∪ (Q₁ × F₂) |
| Intersection (footnote to Thm 1.25: "F = F₁ × F₂ … would be the intersection") | F₁ × F₂ |
| Difference A − B | F₁ × (Q₂ − F₂) |
| Symmetric difference (used by Thm 4.5) | pairs where exactly one is accepting |

- Build only reachable pairs (worklist). O(|Q₁|·|Q₂|·|Σ|). Alphabets must match — if not, extend both to Σ₁ ∪ Σ₂ by completing with trap states (and tell the user).

### C7 — NFA closure constructions (Theorems 1.45, 1.47, 1.49)
Exposed directly: `union(N1,N2)`, `concat(N1,N2)`, `star(N1)` — the same constructions C3 uses internally. Also `plus(N) = concat(N, star(N))`.

### C8 — Complement and reverse
- **Complement:** complete the DFA (C-complete: add trap state), then F ← Q − F. Applying it to an NFA requires C2 first — the UI explains *why* swapping accept states on an NFA is wrong.
- **Reverse (Problem 1.31):** reverse every arrow; new start state with ε-arrows to every old accept state; the old start state becomes the only accept state. Output: NFA.

### C9 — DFA minimization (beyond the book's main text; theory in Problems 1.51–1.52, Myhill–Nerode)
Myhill–Nerode says the minimal DFA has exactly *index(L)* states. Algorithms:
1. **Remove unreachable states** (BFS from q₀).
2. **Table-filling** (teaching mode): mark pairs (p, q) with exactly one in F; repeat: mark (p, q) if some a has (δ(p,a), δ(q,a)) marked; merge unmarked pairs. O(n²·|Σ|) per pass; the UI animates the triangular table and shows, for each marked pair, the **distinguishing string** — that string is exactly the Myhill–Nerode witness of Problem 1.51.
3. **Hopcroft's algorithm** (fast mode): partition refinement, O(n·|Σ|·log n).
- Both must produce isomorphic results (property test).
- Output is **canonically numbered** (BFS order from start, symbols in Σ order) so two minimal DFAs can be compared with `==`.

### C10 — Regex → DFA directly (beyond the book, optional)
Brzozowski derivatives (with the simplifier from C5 to keep the state set finite) or the followpos/position-automaton construction. Used as an **independent cross-check** of the C3 → C2 → C9 path in property tests, and offered to users as an alternative.

### C11 — DFA → CFG (p. 107)
Variable Rᵢ per state qᵢ; rule Rᵢ → aRⱼ when δ(qᵢ, a) = qⱼ; rule Rᵢ → ε when qᵢ ∈ F; start R₀. Shows students that every regular language is context-free.

---

## D. Decision procedures and analysis (Chapter 4, plus Theorem 1.70)

| ID | Problem | Book | Algorithm | Output |
|---|---|---|---|---|
| D1 | A_DFA | Thm 4.1 | simulate | accept/reject + run |
| D2 | A_NFA | Thm 4.2 | C2 then D1 (or direct set simulation) | + subset trace |
| D3 | A_REX | Thm 4.3 | C3 then D2 | |
| D4 | E_DFA | Thm 4.4 | marking: mark start, repeatedly mark successors; empty iff no accept state marked | marked set animation; if non-empty, a **shortest witness string** (BFS parent pointers) |
| D5 | EQ_DFA | Thm 4.5 | C6 symmetric difference, then D4 | equal, or a **shortest distinguishing string** (counterexample) |
| D6 | A_CFG | Thm 4.7 | C12 to CNF, then CYK (C13) — not the book's 2n−1-step enumeration, which is exponential; Theorem 7.16 justifies CYK | parse tree |
| D7 | E_CFG | Thm 4.8 | mark variables that generate terminal strings (fixpoint) | marked variables |
| D8 | Language enumeration | — | BFS in shortlex order over a DFA (or CYK-filter for CFGs) | first N strings in L, first N strings not in L |
| D9 | Pumping lemma (regular) | Thm 1.70 | for DFA with p = \|Q\|, s ∈ L, \|s\| ≥ p: run s, find first repeated state among r₀…r_p (pigeonhole, as in the proof); x = s[0..i], y = s[i..j], z = rest | split highlighted on the diagram as a loop |
| D10 | Pumping lemma (CFL) | Thm 2.34 | p = b^{\|V\|+1} (b = max RHS length; b = 2 in CNF). Take a parse tree of s, find a variable repeated among the lowest \|V\|+1 variables of the longest path → u v x y z | parse tree with the two occurrences highlighted |

**Pumping lemma "game" mode** (for *non*-regular languages, which have no DFA): the language is supplied by any membership oracle the engine can decide — a CFG (via CYK), a PDA (via PDA→CFG→CNF→CYK), an LBA, or a step-bounded TM. The user plays the prover (picks s), the engine plays the adversary (all splits xyz with |xy| ≤ p, |y| > 0), and the user must find an i that pumps out of the language for every split.

**Explicitly undecidable — we say so in the UI instead of pretending:** EQ_CFG (Problem 5.1), ALL_CFG (Thm 5.13), AMBIG_CFG (Problem 5.21), A_TM / HALT_TM (Thm 4.11, 5.1). We offer *bounded* checks ("no difference found among strings of length ≤ 10") clearly labeled as such.

---

## B. Context-free languages (Chapter 2)

### C12 — CFG → Chomsky normal form (Theorem 2.9)

Faithful order (the book's):
```text
1. START  add new start S0 and rule S0 → S
2. DEL    while some ε-rule A → ε exists with A ≠ S0:
              remove it; for each rule R → uAv add R → uv for every subset of A-occurrences
              (R → uAvAw adds R → uvAw, R → uAvw, R → uvw);
              a rule R → A adds R → ε unless R → ε was previously removed
3. UNIT   while some unit rule A → B exists:
              remove it; for each B → u add A → u unless it is a unit rule previously removed
4. BIN    replace A → u1u2…uk (k ≥ 3) by A → u1A1, A1 → u2A2, …, A(k−2) → u(k−1)uk
   TERM   in rules of length 2, replace terminal ui by new variable Ui with Ui → ui
```
- Each removed/added rule is a trace step; the UI renders the book's style from Example 2.10 (bold = just added, gray = just removed).
- Keep a `removed_eps: Set<Var>` and `removed_units: Set<(Var,Var)>` — the "unless previously removed" clauses are what make the loop terminate.
- **Complexity pitfall:** DEL on a rule with k nullable occurrences adds up to 2ᵏ rules. *Optimized mode* runs BIN before DEL (standard textbook-independent ordering), which keeps growth polynomial; it is labeled as a different order than the book.
- **Optimized extras:** remove non-generating then unreachable variables (D7 + reachability), merge duplicate Uᵢ (the book itself merges them into one U in Example 2.10).
- **Golden test (Example 2.10, G₆: S → ASA | aB, A → B | S, B → b | ε):** final grammar (with the book's U merge) S₀ → AA₁ | UB | a | SA | AS; S → AA₁ | UB | a | SA | AS; A → b | AA₁ | UB | a | SA | AS; A₁ → SA; U → a; B → b. Compare up to renaming of the fresh variables.

### C13 — CYK membership and parsing (Theorem 7.16)
Book algorithm D: table(i, j) = variables generating wᵢ…wⱼ; length-1 entries from rules A → b; for l = 2..n, i, k: if B ∈ table(i,k), C ∈ table(k+1,j), A → BC then add A; accept iff S ∈ table(1,n); w = ε handled by S → ε.
- O(n³·|R|). Store variable sets as `BitSet`; store back-pointers to rebuild **all** parse trees (count them → ambiguity *for this string*).
- UI: the triangular table filling cell by cell, click a cell to see its derivation.

### C14 — Grammar hygiene
Generating variables (D7), reachable variables, useless-symbol removal, ε-free check, left-recursion detection (informational).

### C15 — CFG → PDA (Lemma 2.21)
States {q_start, q_loop, q_accept} ∪ E (E = helper states for the push-a-string shorthand):
```text
δ(q_start, ε, ε) = {(q_loop, S$)}                      # push $ then S (S on top)
δ(q_loop, ε, A)  = {(q_loop, w) | A → w ∈ R}           # expand a variable
δ(q_loop, a, a)  = {(q_loop, ε)} for each terminal a   # match a terminal
δ(q_loop, ε, $)  = {(q_accept, ε)}                     # stack empty → accept
```
- Shorthand expansion (Figure 2.23): (r, u₁…u_l) ∈ δ(q, a, s) becomes δ(q, a, s) ∋ (q₁, u_l), δ(q₁, ε, ε) = {(q₂, u_{l−1})}, …, δ(q_{l−1}, ε, ε) = {(r, u₁)}. Faithful mode expands; compact mode keeps string pushes and draws them like Figure 2.24.

### C16 — PDA → CFG (Lemma 2.27)
1. **Normalize P** to the three features:
   - single accept state q_accept (ε,ε→ε arrows from old accept states to a new one);
   - empties its stack before accepting (new start pushes a bottom marker, old accept states go to a drain state that pops everything, then pops the marker into q_accept);
   - every transition is *either* a push *or* a pop: split pop+push through a new state; replace "neither" with push-then-pop of a fresh dummy symbol; expand string pushes first.
2. **Build G** with variables A_pq for all p, q ∈ Q, start A_{q0, q_accept}:
   - (1) for p,q,r,s ∈ Q, u ∈ Γ, a,b ∈ Σ_ε: if (r,u) ∈ δ(p,a,ε) and (q,ε) ∈ δ(s,b,u), add A_pq → a A_rs b
   - (2) for p,q,r ∈ Q: A_pq → A_pr A_rq
   - (3) for p ∈ Q: A_pp → ε
- **Complexity:** |Q|³ rules from (2) alone; rule (1) iterate push-edges × pop-edges with matching u (not all 4-tuples). Faithful mode shows the raw grammar; optimized mode prunes (C14) — usually from thousands of rules to a handful.
- **Golden test:** PDA M₁ of Example 2.14 (0ⁿ1ⁿ) → CFG → CNF → CYK must agree with direct PDA simulation on every string over {0,1} of length ≤ 10.

### C17 — DPDA check (Definition 2.39)
For all q, a ∈ Σ, x ∈ Γ: exactly one of δ(q,a,x), δ(q,a,ε), δ(q,ε,x), δ(q,ε,ε) is non-∅. Report each violating triple. (Plus: each non-∅ value is a single move.)

### C18 — DK-test and DCFG → DPDA (§2.4, Theorem 2.52) — stretch
- Build NFA K over **dotted rules** B → u.v: start ε-moves to S₁ → .u for each start rule; shift move on symbol X from B → u.Xv to B → uX.v; ε-move from B → u.Cv to C → .r for every rule of C; accept states are completed rules B → u. .
- DK = C2(K) with unreachable states removed — **reuses our subset construction**.
- DK-test: every accept state of DK contains (1) exactly one completed rule and (2) no dotted rule with a terminal immediately after the dot. Report the failing state with its dotted rules (this is LR(0) conflict reporting).
- Then visualize the DPDA/LR parse of a string: stack of DK states, shift/reduce actions, handles underlined as in Example 2.45.

### C19 — Derivations and parse trees
Leftmost/rightmost derivation from a parse tree; derivation step-through (uAv ⇒ uwv highlighting); **bounded** ambiguity search (two distinct parse trees for some w with |w| ≤ n, via CYK counts) — labeled bounded, since AMBIG_CFG is undecidable.

---

## C. Turing machines (Chapter 3) and simulations

### S1 — DFA run
Tight loop over the dense table; records r₀…rₙ. O(n).

### S2 — NFA run
Set simulation with BitSets: S₀ = E({q₀}); Sᵢ₊₁ = E(⋃_{q∈Sᵢ} δ(q, wᵢ₊₁)). O(n·|δ|). Optional computation-tree mode (Figure 1.29) with a node limit.

### S3 — PDA run (nondeterministic)
Breadth-first search over configurations (state, input index, stack), visited-set dedupe (hash the stack). PDAs may loop on ε-moves while growing the stack, so:
- **Fast mode:** BFS with budgets (max configurations, max stack depth — default derived from |w|, |Q|, |Γ|); result is ACCEPT, REJECT (search space exhausted) or **INCONCLUSIVE (budget)** — never a wrong answer.
- **Exact mode:** C16 → C12 → C13 (PDA → CFG → CNF → CYK) decides membership exactly. Slower to set up, but always terminates — a nice demonstration of Theorem 4.7.

### S4 — TM run (Definition 3.3)
- Tape = two stacks (`left`, `right`), head = top of `right`; extending with ⊔ on demand. Left move at the left end: head stays (book rule).
- Missing transitions → q_reject, moving R (book convention p. 173).
- Step budget (default 10⁶, user-adjustable). Outcomes: ACCEPT, REJECT, **RUNNING (budget exhausted)** — never "loops", because that is undecidable (Thm 4.11) — except:
- **Exact repeat detection:** hash each configuration; if one repeats, the machine provably loops (sound but incomplete). For **LBAs** this is complete: there are only q·n·gⁿ configurations (Lemma 5.8), so the simulator decides A_LBA (Thm 5.9).
- Configuration log printed as `u q v` exactly like the book's sample runs (e.g. `⊔q₂000`).

### S5 — TM variants
- Stay-put → standard: replace each S move by R to a fresh state, then L back (book's argument, §3.2).
- Multitape: native k-tape simulation; **"show single-tape encoding"** view renders `#t1#t2#…#` with dotted head symbols (Theorem 3.13). Full construction of the equivalent single-tape δ is a stretch goal.
- Nondeterministic TM: **breadth-first** search of the computation tree (Theorem 3.16 — depth-first could follow an infinite branch forever). Visualize the tree; node addresses as in the proof's address tape (strings over {1..b}).
- Enumerator: run, collect printed strings, show duplicates.

---

## T. Trace format (shared by all algorithms)

```rust
pub struct Trace { pub steps: Vec<Step>, pub truncated: bool }
pub struct Step {
    pub rule: &'static str,         // e.g. "Theorem 1.39: δ'(R,a) = ⋃ E(δ(r,a))"
    pub message: String,            // human sentence for the step panel
    pub kind: StepKind,             // typed payload for highlighting
}
pub enum StepKind {
    AddState { id: StateId, origin: Origin },
    AddEdge  { from: StateId, label: String, to: StateId },
    RemoveState { id: StateId },
    Closure  { from: BitSet, result: BitSet },
    Rip      { q_rip: StateId, updates: Vec<RipUpdate> },   // R1, R2, R3, R4, new
    MarkPair { p: StateId, q: StateId, witness: Vec<SymId> },
    GrammarEdit { added: Vec<Rule>, removed: Vec<Rule>, phase: CnfPhase },
    CykCell  { i: usize, j: usize, vars: BitSet },
    Config   { text: String },      // simulations: "1011 q7 01111"
}
```
- Traces are capped (`max_trace_steps`); after the cap, the result is still computed, the trace is marked `truncated`.
- The UI can replay any prefix of the trace to reconstruct the intermediate machine (so "step 37 of 120" shows the half-built DFA).

---

## Complexity summary

| Algorithm | Time | Output size | Notes |
|---|---|---|---|
| C2 subset | O(2ⁿ·\|Σ\|·n) worst | ≤ 2ⁿ | typical cases far smaller; budgeted |
| C3 regex→NFA | O(\|R\|) | O(\|R\|) | |
| C5 state elimination | O(n³) regex ops | regex can be exponential | rip-order heuristic |
| C6 product | O(n₁n₂\|Σ\|) | ≤ n₁n₂ | reachable only |
| C9 Hopcroft | O(n\|Σ\|log n) | ≤ n | |
| C12 CNF | book order: exponential worst case (DEL); BIN-first: O(\|G\|²) | | |
| C13 CYK | O(n³\|R\|) | | |
| C16 PDA→CFG | O(\|Q\|³ + pushes×pops) | O(\|Q\|³) | prune afterwards |
| S3 PDA BFS | budgeted | | exact mode via C16+C13 |
| S4 TM | budgeted | | LBA: exact |
