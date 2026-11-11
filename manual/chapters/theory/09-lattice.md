---
title: "The decidability lattice"
description: "Language classes as types: closure tables, decision tables, and how Delta rejects undecidable questions."
sidebar:
  order: 9
---

Readers: **owner**. Sipser 3e: closure (Theorems 1.25, 1.26, 2.x exercises), decidable problems
(Theorems 4.1–4.8), undecidable ones (Theorem 4.11 A_TM; Theorems 5.1–5.4; EQ_CFG in the
exercises of chapter 5), Theorem 4.22 (decidable = recognizable ∩ co-recognizable).

## Definition

Every `lang` value has a **class**, a point in the chain

Finite ⊂ Regular ⊂ DCF ⊂ CF ⊂ CS ⊂ Decidable ⊂ Recognizable ⊂ Any.

Classes come from sources (a `dfa` is Regular, a `cfg` CF, a set-builder over Σ* Decidable, a
`tm` Recognizable, an `lba` CS) and from operations through **closure tables**. Questions
(membership, emptiness, finiteness, universality, equivalence, inclusion) are allowed only where a
**decision table** says they're decidable. A class can be raised with evidence: `lang E = … as
Regular by re"…"`, checked up to a bound.

## The tables

Closure — the class of an operation's result, by the operand's class column:

<!-- excerpt: crates/delta-lang/src/lang.rs#closure-table -->

Decision — when a question can be answered:

<!-- excerpt: crates/delta-lang/src/lang.rs#decidable -->

## Why it's correct (sketches)

- Regular is closed under ∪, ∩, complement, concatenation, star, reverse (product, subset,
  NFA constructions). Everything is decidable for regular languages (Theorems 4.1–4.5).
- CF is closed under ∪, concatenation, star, reverse, ∩ with Regular, but **not** ∩ or
  complement (so ∩ of two CF is only CS, complement of CF is CS). Emptiness and finiteness of CF
  are decidable (generating variables; cycles), equivalence and universality are not.
- DCF is closed under complement (so complement stays DCF); DCF universality is decidable (it's
  emptiness of the complement); DCF equivalence is decidable (Sénizergues) but not implemented →
  E702.
- Beyond CF, the meaningful questions are undecidable (Rice's theorem for recognizable
  languages); Decidable ⊂ Recognizable because a decider is a recognizer, and complement of
  Recognizable is only "Any" (co-recognizable).

## In Delta

The checker computes classes statically (`check.rs` `class_of`) and reports E701 before running;
the interpreter re-checks at run time (`hooks.rs` `decide`). Every E701 offers a bounded
alternative (`up to 12` by default). Every table entry has a test (`closure_table_rules` and
friends in `lang.rs`).

## Questions they'll ask

1. **"Isn't this just documentation?"** — No: it's enforced. `check A ≡ B` with A, B context-free
   doesn't compile (E701); the quick fix makes it a bounded check, and the output says so.
2. **"Why is the class an upper bound, not exact?"** — Computing the exact class is undecidable
   (is this CFL regular? undecidable). Delta tracks a sound upper bound; `as C by …` lets the user
   lower it with evidence.
3. **"Why is ∩ of two CF languages CS and not 'Decidable'?"** — CS is closed under ∩ (LBAs run
   in sequence), and every CF language is CS; the table gives the tightest class it can justify.
4. **"Where does `up to 12` come from?"** — A setting (default 12) chosen so bounded checks over
   small alphabets finish quickly; 2¹³ − 1 = 8,191 words over {a, b}.
5. **"What's the relationship between Decidable and Recognizable?"** — L is decidable iff L and
   its complement are both recognizable (Theorem 4.22).
6. **"How does a class flow through `let`?"** — The checker binds each name with its class; a
   function result without a known class falls back to run-time checking.

## Self-test

<details class="selftest"><summary>1. Class of the complement of a CF language?</summary>CS (CF isn't closed under complement).</details>
<details class="selftest"><summary>2. Class of CF ∩ Regular?</summary>CF.</details>
<details class="selftest"><summary>3. Which questions are decidable for CF?</summary>Membership, emptiness, finiteness.</details>
<details class="selftest"><summary>4. What error for an undecidable question, and for decidable-but-unimplemented?</summary>E701 and E702.</details>
<details class="selftest"><summary>5. State Theorem 4.22.</summary>A language is decidable iff it is Turing-recognizable and co-Turing-recognizable.</details>
