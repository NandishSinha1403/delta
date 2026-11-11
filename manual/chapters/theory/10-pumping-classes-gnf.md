---
title: "Pumping, classes, GNF, ambiguity, one tape"
description: "The five functions that closed Phase 6: pump, myhill_nerode, gnf, bounded is_ambiguous and to_single_tape — how each works and where its limits are."
sidebar:
  order: 10
---

Readers: **owner**. Sipser 3e: Theorem 1.70 (pumping lemma for regular languages), Problem 1.52
(Myhill–Nerode), Definition 2.7 (ambiguity), Theorem 3.13 (multitape ≡ single tape). Greibach
normal form is not in Sipser; Hopcroft–Ullman cover it. Spec: SEMANTICS §11.8–11.12.

## `pump(D, w)` — `regular/pump`

Run the DFA on w and stop at the **first** state that comes round a second time. With p states,
that happens within p steps (pigeonhole), so `|xy| ≤ p` and `|y| ≥ 1`; the symbols between the
two visits are a loop, so any number of copies of y leads to the same final state.

<!-- excerpt: crates/delta-core/src/fa/pump.rs#pump-split -->

p is the size of the DFA **as given** (the owner chose this, 2026-11-11); for a regex, NFA or
`lang` it's the minimal DFA. A word outside L(D), or shorter than p, is E504: the lemma makes no
promise for it. The pumping-lemma tool page (`/tools/pumping-lemma/`) uses `pump` to catch a DFA
that claims to recognize `{ 0ⁿ1ⁿ }`.

## `myhill_nerode(D)` — `regular/myhill-nerode`

The classes of u ≡ v ⟺ (∀z: uz ∈ L ⟺ vz ∈ L) are exactly the states of the minimal complete
DFA (`minimize/hopcroft`, then `crates/delta-core/src/fa/myhill_nerode.rs`). For each class:
breadth-first search from the start, symbols in order, gives the shortlex-least word. For each
pair of classes: breadth-first search over **pairs** of states gives the shortlex-least suffix
that leads one into F and the other out. Result: `{index, classes, separators}`; for a `dfa`
argument each class also lists the original states it merges.

## `gnf(G)` — `cfg/gnf`

Start from `cnf(G)` (so rules are A → BC or A → a), order the variables A₁…Aₙ with the start
first, and make every Aᵢ rule start with a terminal or a **later** variable: substitute earlier
variables away, then remove immediate left recursion with a fresh Zᵢ:

<!-- excerpt: crates/delta-lang/src/gnf.rs#gnf-left-recursion -->

Finally substitute back from Aₙ down to A₁, then into the Z rules; every rule now starts with a
terminal. The result can be large (a three-variable grammar became 853 rules in a test), so rule count is
charged to the states budget (B602).

## `is_ambiguous(G, upto: n)` — `cfg/ambiguity-bounded`

Ambiguity is undecidable, so this is a **search**, and the result says so: the record's `note`
always states the bound. For each word up to length n (shortlex), count the parse trees of every
span **in G itself** (a normal form would change the trees), saturating at 2. A cycle A ⇒⁺ A
gives infinitely many trees, so counts are a least fixed point computed in rounds:

<!-- excerpt: crates/delta-lang/src/ambiguity.rs#ambiguity-rounds -->

The round in which a count first reached 1 and 2 is kept. A tree for a count reached in round r
only uses children reached before r, so reading two trees off the chart always ends, even with
cycles and ε-rules. The two trees are Pad objects with diagrams.

## `to_single_tape(M)` — `tm/single-tape`

The owner chose the **multi-track** construction (2026-11-11) over the textbook `#`-separated
one: each cell holds `[s₁,…,sₖ]` with `^` after sᵢ where tape i's head is, and `▸` on cell 0. A
plain symbol c means `[c,⊔,…,⊔]` with no heads, so the input needs no conversion pass. One step
of M: sweep right from cell 0 collecting the k head symbols (the state remembers them), pick M's
edge, sweep left writing each tape's symbol and moving its mark, walk back to `▸`:

<!-- excerpt: crates/delta-lang/src/single_tape.rs#single-tape-apply -->

Only reachable (state, symbol) pairs are built, by a worklist. The 0ⁿ1ⁿ two-tape example becomes
a 68-state one-tape machine; conformance case 66 checks both accept the same words up to length 6.

Gotcha found while testing: in the keyword label form, `on 2:` covers only the operation right
after it (`;` resets to tape 1), so `on 2: write $; on 2: move R` needs `on 2:` twice.

## Questions they'll ask

1. **"Does `pump` prove a language isn't regular?"** — No. It shows the split for one DFA and
   one word. The proof is the argument for every DFA; the tool page walks through it.
2. **"Why count trees in G and not in CNF?"** — Converting to CNF can add or remove ambiguity
   (DEL and UNIT merge derivations), so the answer would be about a different grammar.
3. **"Can `is_ambiguous` say a grammar is unambiguous?"** — Only "no ambiguous word up to
   length n". The `note` field says exactly that, and longer words may still be ambiguous.
4. **"Why multi-track instead of the textbook proof?"** — No shifting when a tape grows, a smaller
   machine, and the Run view shows tracks directly. Same result: L(S) = L(M), S halts iff M does.
5. **"Why is p the given DFA's size, not the minimum?"** — It's the DFA the student is reasoning
   about; a regex or NFA has no DFA yet, so it gets the minimal one.

## Self-test

<details class="selftest"><summary>1. Why is |xy| ≤ p guaranteed?</summary>The first p steps visit p + 1 states of a p-state DFA, so one repeats within them.</details>
<details class="selftest"><summary>2. What are the Myhill–Nerode classes of a regular language, concretely?</summary>The states of its minimal complete DFA; each has a shortest word, and each pair a shortest separating suffix.</details>
<details class="selftest"><summary>3. Why does is_ambiguous keep the round in which each count reached 1 and 2?</summary>So trees are read only from children counted earlier, which keeps them finite even with cycles.</details>
<details class="selftest"><summary>4. What does a plain symbol mean on the single tape?</summary>That symbol on track 1, blanks on the others, no heads.</details>
<details class="selftest"><summary>5. Which budget does gnf use, and why?</summary>The states budget (B602), charged per rule, because substitution can blow up the grammar.</details>
