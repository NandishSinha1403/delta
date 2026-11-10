---
title: "6. Using it"
description: The Pad and deltac task by task — write a machine, check it, run it, convert it, share it, save it.
---

Readers: **user**. The public docs (`/docs/`) are the full reference; this chapter is the tasks
you'll demo.

## How to write and check a machine

1. In the Pad, write a machine literal: `dfa M { start q0; accept q1; q0 -a-> q1 }`.
2. Add claims: `check M accepts "a"`, `check M rejects "", "aa"`.
3. Look at **Tests**: one line per word. The **Diagram** draws the machine; **Tuple** shows
   (Q, Σ, δ, q0, F) and the δ-table.

If the machine is missing a transition, the checker says so (E3xx) with quick fixes "add a dead
state" or "mark `(partial)`".

## How to compare a machine with a specification

- Exact, for regular languages: `check M ≡ re"(a ∪ b)* a b"`.
- Bounded, when one side is only known to be decidable: `check M ≡ L up to 10`.
- On failure the Tests view shows `counterexample: "…"`, the shortest word where they differ.

## How to run a machine step by step

1. `run M on "abab"` in code, or pick the machine and type a word in the **Run** view.
2. Scrub the timeline: the diagram highlights the active states ("fingers") and the edge just
   taken; the tape shows the head.
3. The verdict is accept, reject, loops or unknown.

## How to convert and replay a construction

`let D = determinize(N)`, `minimize(D)`, `to_regex(D)`, `to_nfa(re"…")`. The **Steps** view
replays the construction (each new subset, each split, each removed state). The command palette
(⌘K) and the **Convert** buttons insert these lines for you.

## How to write your own construction

```delta
fn subsets(N) = explore {
  alphabet N.alphabet
  start  N.eclose({ N.start })
  step   R on a -> N.eclose(⋃ { N.δ(q, a) | q ∈ R })
  accept R if R ∩ N.accept ≠ ∅
}
```

Then `check subsets(N) ≡ N`.

## How to work with grammars

`cnf(G)`, `parse(G, w)` (or `method: "cyk"`), `derive(G, w)`, `to_pda(G)`, `to_cfg(P)`,
`nullable(G)`, `useless(G)`, `trim(G)`. A parse tree shows as an indented tree; `steps(cnf(G))`
and `steps(parse(G, w, method: "cyk"))` replay the phases and the table in the Steps view.

## How to write a Moore machine

Give each state's output with `λ(q) = w` (or `lambda(q) = w`); `mealy_to_moore` and
`moore_to_mealy` convert between the two kinds.

## How to share

**Share** copies a link with the program in it; **Embed** copies an `<iframe>` snippet for a
teaching page (`&embed=1&panel=…`).

## How to save pictures

- Pad: `save M to "m.png"`, then the Console's **Download m.png** button.
- Terminal: `deltac run file.delta` writes the file; `deltac run --show file.delta` draws
  machines inline in kitty, Ghostty, iTerm2, WezTerm or sixel terminals;
  `--format pdf --out figs` writes every shown machine.

## Questions they'll ask

1. **"Can students cheat with it?"** — It checks claims; a teacher can grade by running a
   student's machine against a hidden reference with `check ≡`. `use strict` makes students write
   every field explicitly.
2. **"How do you explain errors to beginners?"** — Every code has a docs page with an example;
   messages name the problem in course terms ("no transition for (q2, '1'). A total deterministic
   model needs one") and offer quick fixes.
3. **"Does it work on a phone?"** — Yes; the Pad has phone, tablet and desktop layouts, tested
   with Playwright at each size.
4. **"What happens with a Turing machine that never halts?"** — The run stops at the step budget
   with verdict `unknown`, or `loops` if a configuration repeats on every branch.
5. **"Can I use it offline?"** — After one visit, yes (PWA with a service worker).

## Self-test

<details class="selftest"><summary>1. How do you get the shortest counterexample between M and a regex?</summary><code>check M ≡ re"…"</code>; on failure the Tests view shows it.</details>
<details class="selftest"><summary>2. Which view replays a construction?</summary>Steps.</details>
<details class="selftest"><summary>3. What keyword builds a machine from successors?</summary><code>explore</code>.</details>
<details class="selftest"><summary>4. How does a teacher put a live example in a page?</summary>The Embed button: an iframe with <code>#code=…&amp;embed=1</code>.</details>
<details class="selftest"><summary>5. Which terminals show images with <code>--show</code>?</summary>kitty, Ghostty (kitty protocol), iTerm2, WezTerm (iTerm2 protocol), foot/mlterm/contour (sixel).</details>
