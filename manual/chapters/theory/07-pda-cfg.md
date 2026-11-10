---
title: "PDA ↔ CFG"
description: "Pushdown automata and context-free grammars recognize the same languages: both constructions."
sidebar:
  order: 7
---

Readers: **owner**. Sipser 3e: Theorem 2.20, with Lemma 2.21 (CFG → PDA) and Lemma 2.27
(PDA → CFG).

**Status in Delta: built (Phase 6).** `to_pda(G)` and `to_cfg(P)` (`crates/delta-lang/src/pda_cfg.rs`,
ids `cfg/to-pda` and `pda/to-cfg`). PDAs and DPDAs also run on the generic simulator, with
acceptance by final state or by empty stack, and the Pad's Run view shows the stack at each step.

## CFG → PDA

Three-state PDA: push `S$`, then loop in q_loop:
- top is a variable A: pop A, push the right side of some rule A → w (nondeterministically);
- top is a terminal a: read a from the input and pop it;
- top is `$`: pop it and go to the accept state.

It simulates a **leftmost derivation**, keeping the not-yet-matched part of the sentential form on
the stack.

## PDA → CFG

First normalize the PDA: one accept state, it empties its stack before accepting, and every
transition either pushes or pops one symbol (not both, not neither). Then for every pair of
states p, q add a variable A_pq generating exactly the words that take the PDA from p with an
empty stack to q with an empty stack:

- A_pp → ε;
- A_pq → A_pr A_rq for all r (the stack becomes empty in between);
- A_pq → a A_rs b when p pushes t reading a and goes to r, and s pops t reading b and goes to q
  (the first push is matched by the last pop).

Start variable A_{q₀, q_accept}.

## Why it's correct

CFG → PDA: a run accepts w iff the stack moves trace a leftmost derivation of w. PDA → CFG:
induction on derivation length (A_pq ⇒* x implies the PDA can go p → q on x with empty stack)
and on computation length (conversely), splitting on whether the stack becomes empty in the
middle.

## Complexity

CFG → PDA: linear in the grammar. PDA → CFG: O(n³) rules of the A_pq A_rq kind for n states,
plus one rule per matched push/pop pair — big and unreadable, which is the teaching point.

## In Delta

`to_pda` is a direct transcription: one edge per rule (pop the variable, push the right side; `push
YZ` leaves Y on top) and one per terminal (read it, pop it):

<!-- excerpt: crates/delta-lang/src/pda_cfg.rs#to-pda-rules -->

`to_cfg` normalizes first (SEMANTICS §11.6): a new start pushes `$`, every edge is split into a
chain of single pushes or pops through fresh states (an edge that does neither pushes and pops a
fresh `#`), and accepting runs drain the stack into one state `acc`. Then the matched push/pop
rule, the heart of the construction:

<!-- excerpt: crates/delta-lang/src/pda_cfg.rs#to-cfg-pairs -->

The ⟨p,r⟩⟨r,q⟩ rules are generated only over states that occur on a move, and the result is
trimmed; for a 3-state PDA that leaves about 40 rules. A cubic guard refuses machines whose
normalized form would need more rules than the states budget.

**Membership uses it.** The simulator's breadth-first search never ends on a PDA whose ε-moves can
grow the stack (`to_pda` of `S → S S | ε`). So `P.accepts(w)` and `check P accepts …` decide
membership through `to_cfg(P)` and Earley, cached per machine (`Machine::pda_grammar`); `P.run(w)`
still explores configurations so the Run view can show the stack.

## Questions they'll ask

1. **"Why must the PDA empty its stack before accepting for PDA → CFG?"** — A_pq is defined by
   empty-to-empty stack runs; a leftover stack would break the split argument.
2. **"Why can't a DPDA recognize every CFL?"** — {wwᴿ} needs to guess the middle; DCFLs are
   closed under complement, CFLs aren't, so they differ.
3. **"Accept by final state vs empty stack?"** — Equivalent for PDAs (convert either way); not
   for DPDAs. Delta has both (`accept: by-final-state`, `by empty-stack`).
4. **"How does simulation terminate with ε-loops?"** — The visited set catches repeated
   configurations, but ε-moves can grow the stack forever without repeating one. A run then ends
   at the step budget (`unknown`); membership questions avoid that by going through the grammar.
5. **"Why is the grammar from a PDA so big?"** — One variable per state pair and all splits;
   most are useless and can be trimmed (non-generating, unreachable).

## Self-test

<details class="selftest"><summary>1. What does the CFG → PDA stack hold?</summary>The unmatched suffix of a leftmost sentential form (variables and terminals), above $.</details>
<details class="selftest"><summary>2. What does A_pq generate?</summary>Words taking the PDA from p (empty stack) to q (empty stack).</details>
<details class="selftest"><summary>3. Name the three A_pq rule kinds.</summary>A_pp → ε; A_pq → A_pr A_rq; A_pq → a A_rs b (matched push/pop).</details>
<details class="selftest"><summary>4. Which Sipser theorem?</summary>Theorem 2.20 (Lemmas 2.21 and 2.27).</details>
<details class="selftest"><summary>5. How does Delta decide P.accepts(w) for a PDA, and why not by simulation?</summary>Through to_cfg(P) and the Earley recognizer, which always finishes; breadth-first simulation can run forever when ε-moves grow the stack.</details>
