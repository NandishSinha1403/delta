---
title: "TM simulation"
description: "How Delta runs Turing machines (and every other model): breadth-first over configurations, with loops detected and budgets."
sidebar:
  order: 8
---

Readers: **owner**. Sipser 3e: Definition 3.3 (TM), 3.5 (configurations); Theorem 3.13
(multitape ≡ single tape), Theorem 3.16 (nondeterministic ≡ deterministic, by breadth-first
search of the computation tree).

## Definition

A configuration is (state, tape contents, head position) — in Delta, generalized to (state, input
position, storages, output). A step applies one transition. A nondeterministic machine accepts if
**some** branch reaches an accepting configuration.

## How Delta simulates

`crates/delta-lang/src/sim.rs`, `Machine::run`. One algorithm for every model:

1. Start from the initial configuration; keep a frontier and a map `seen: Config → index`.
2. For each configuration in the frontier: accept if accepting; otherwise apply every outgoing
   edge's operation list; new configurations go to the next frontier.
3. Count steps against `fuel`; out of fuel → verdict `unknown`.
4. Frontier empty without accepting → `reject`, or `loops` for halting-acceptance machines whose
   configuration graph has a cycle.

<!-- excerpt: crates/delta-lang/src/sim.rs#sim-expand -->

<!-- excerpt: crates/delta-lang/src/sim.rs#sim-verdict -->

## Why it's correct

Breadth-first search visits every configuration reachable in k steps before any reachable only in
k + 1, so if any branch accepts within the fuel, it's found (Sipser's proof of Theorem 3.16 uses
exactly this order to avoid getting lost in an infinite branch). Deduplicating configurations
doesn't lose acceptance: two branches in the same configuration have the same futures. If the
reachable configuration graph is finite and fully explored with no acceptance, the machine
rejects; if that graph has a cycle, some branch runs forever — `loops`.

## Complexity

Exponential in the worst case for nondeterministic machines (the frontier can double each step),
and memory grows with the number of distinct configurations. For a deterministic TM the frontier
is one configuration, but `seen` stores every configuration with its tape — quadratic memory on
long runs (a known issue). Budgets bound both.

## Questions they'll ask

1. **"You can detect loops? Doesn't that solve the halting problem?"** — No: it detects only
   loops that **repeat a configuration**. A TM that writes forever to the right never repeats one;
   it ends as `unknown` when the budget runs out. That's exactly the boundary the halting problem
   draws.
2. **"Why breadth-first and not depth-first?"** — Depth-first can follow an infinite branch
   forever and miss an accepting one; BFS is complete within the budget.
3. **"How do multitape and two-way machines fit?"** — Configurations carry several tapes or a
   head that can move left on the input; the model descriptor says which operations exist. Same
   search.
4. **"What's the fast path?"** — Finite automata run as sets of states (`run_fa`), no
   configuration objects; tests check it matches the generic path.
5. **"How would you fix the quadratic memory?"** — Store configuration hashes instead of full
   tapes for loop detection (with a collision check on hit), or detect cycles with Floyd/Brent on
   deterministic machines (constant memory).
6. **"Is `loops` ever wrong?"** — For halting-acceptance machines, a cycle in the explored
   configuration graph means some branch never halts; with nondeterminism other branches might
   still reject, which is why the verdict is used only when nothing accepted.

## Self-test

<details class="selftest"><summary>1. What are the four verdicts and when does each happen?</summary>accept (accepting configuration reached), reject (all explored, none accepting), loops (all explored, a cycle exists, halting acceptance), unknown (fuel ran out).</details>
<details class="selftest"><summary>2. Why is deduplicating configurations safe?</summary>Equal configurations have identical futures.</details>
<details class="selftest"><summary>3. Which Sipser theorem uses BFS over the computation tree?</summary>Theorem 3.16 (NTM ≡ TM).</details>
<details class="selftest"><summary>4. Why doesn't loop detection contradict undecidability of halting?</summary>It only catches repeated configurations; non-repeating infinite runs end as unknown.</details>
<details class="selftest"><summary>5. What's the known memory issue?</summary>seen stores each configuration with its tape: quadratic growth on long deterministic runs.</details>
