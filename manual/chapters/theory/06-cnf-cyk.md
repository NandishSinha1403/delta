---
title: "CNF and CYK"
description: "Chomsky normal form and the CYK membership algorithm, as Delta builds them (cnf, parse with method cyk), next to Earley."
sidebar:
  order: 6
---

Readers: **owner**. Sipser 3e: Definition 2.8 and Theorem 2.9 (CNF); Theorem 7.16 (every CFL is
in P, by the dynamic-programming algorithm usually called CYK).

**Status in Delta: built (Phase 6).** `cnf(G)` converts with a step per phase; `parse(G, w,
method: "cyk")` fills the table (a `CykCell` step per non-empty cell) and returns the tree;
`parse(G, w)` (Earley) is the default and works on the grammar as written.

## Definitions

A grammar is in **Chomsky normal form** if every rule is A → BC (B, C variables, not the start)
or A → a, plus optionally S → ε. Conversion, in this order:

1. **New start** S₀ → S (so the start never appears on a right side).
2. **Remove ε-rules** A → ε (A ≠ S₀): for every rule with A on the right, add a copy without that
   occurrence (every subset of nullable occurrences).
3. **Remove unit rules** A → B: replace with A → u for every B → u.
4. **Shorten and separate**: A → u₁u₂…uₖ (k ≥ 3) becomes A → u₁A₁, A₁ → u₂A₂, …; a terminal a in
   a long rule becomes a new variable U_a → a.

**CYK** decides w ∈ L(G) for G in CNF: table T[i][j] = variables deriving w[i..j]. Length-1 cells
come from A → a; longer spans try every split point k: A ∈ T[i][j] if A → BC with B ∈ T[i][k]
and C ∈ T[k+1][j]. Accept iff S ∈ T[1][n].

## Why it's correct

CNF: each step preserves the language (ε-removal adds exactly the derivations that used the
removed ε; unit removal shortcuts unit chains; splitting is a renaming). CYK: by induction on
span length, T[i][j] contains A iff A ⇒* w[i..j], because in CNF a derivation of a span of length
≥ 2 starts with A → BC and splits the span in two.

## Complexity

CYK is O(n³ · |G|). CNF conversion can blow the grammar up: ε-removal is exponential in the
number of nullable symbols per rule if done naively (quadratic if long rules are split first).
Earley is O(n³) in general, O(n²) for unambiguous grammars and linear for many LR grammars, and
needs no normal form — which is why Delta uses it for membership.

## In Delta

`crates/delta-lang/src/cnf.rs`, id `cfg/cnf`. The phases run in Sipser's order (START, DEL,
UNIT, then TERM and BIN), then `trim`. DEL is the interesting one: for each rule it deletes every
subset of its nullable occurrences, never creating a new ε-rule except for the start:

<!-- excerpt: crates/delta-lang/src/cnf.rs#cnf-del -->

`crates/delta-lang/src/parse_tree.rs`, id `parse/cyk`. The table is `table[i][len-1]`, a map from
variable to its first back-pointer, filled by span length, split point, then rule order (that
order makes the tree deterministic, SEMANTICS §11.3). A grammar already in CNF is used as is.

<!-- excerpt: crates/delta-lang/src/parse_tree.rs#cyk-fill -->

The nullable fixpoint that DEL starts from (`crates/delta-lang/src/grammar.rs`):

<!-- excerpt: crates/delta-lang/src/grammar.rs#grammar-nullable -->

## Questions they'll ask

1. **"Why convert to CNF at all?"** — CYK needs binary rules; proofs (pumping lemma for CFLs,
   decidability of A_CFG) are easier; every derivation of a length-n word has exactly 2n − 1
   steps.
2. **"Why did you use Earley instead?"** — It works on any grammar as written, so error messages
   and parse trees refer to the student's rules, not a converted grammar.
3. **"What order must CNF steps go in, and why?"** — New start first (so ε-removal can't touch
   the start's role), ε before units (ε-removal creates unit rules), then splitting.
4. **"How do you get a parse tree from CYK?"** — Store back-pointers (A, k, B, C) per cell; read
   the tree from T[1][n] ∋ S.
5. **"Is A_CFG decidable?"** — Yes (Theorem 4.7): convert to CNF, try all derivations of 2n − 1
   steps — or run CYK.
6. **"Why does `parse(C, w, method: "cyk")` on a CNF grammar keep its variables?"** — Converting
   a grammar that is already in normal form would add a second start variable (`S01`), so a CNF
   grammar is used as it is (`Grammar::is_cnf`). That was a bug found while writing the tool page.

## Self-test

<details class="selftest"><summary>1. The two CNF rule shapes?</summary>A → BC and A → a (plus S → ε).</details>
<details class="selftest"><summary>2. CYK recurrence?</summary>A ∈ T[i][j] if A → BC, B ∈ T[i][k], C ∈ T[k+1][j] for some i ≤ k &lt; j.</details>
<details class="selftest"><summary>3. CYK complexity?</summary>O(n³ · |G|).</details>
<details class="selftest"><summary>4. Which parser is the default, and why?</summary>Earley (<code>parse/earley</code>): it works on the grammar as written, so trees use the student's own rules.</details>
<details class="selftest"><summary>5. Which CNF step needs the nullable set?</summary>Removing ε-rules.</details>
