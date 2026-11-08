---
title: "Turing machine trace: how many steps on 0011?"
description: "Solved GATE-style problem: trace a Turing machine for 0ⁿ1ⁿ on the input 0011 and count its moves, then check the count in Delta."
sidebar:
  label: "7. Tracing a Turing machine"
  order: 7
---

**Numerical answer type.** The Turing machine below decides `{ 0ⁿ1ⁿ | n ≥ 0 }` by crossing off a
`0` (as `x`) and a matching `1` (as `y`) in each round. How many moves does it make on input
`0011` before it accepts?

## Solution

Trace it, writing the head position with `▸`:

| moves | configuration | what happens |
|---|---|---|
| 0 | `q0 ▸0011` | |
| 1 | `q1 x▸011` | cross off the first `0` |
| 2–3 | `q2 x▸0y1` | skip `0`, cross off the first `1`, turn left |
| 4–5 | `q0 x▸0y1` | back to the `x`, step right |
| 6 | `q1 xx▸y1` | cross off the second `0` |
| 7–8 | `q2 xx▸yy` | skip `y`, cross off the second `1` |
| 9–10 | `q0 xx▸yy` | back to the `x`, step right |
| 11–12 | `q3 xxyy▸⊔` | no `0` left: check that only `y`'s remain |
| 13 | `qa` | blank reached: accept |

**Answer: 13.**

## Check it in Delta

`M.run(w)` returns the run: its verdict and every configuration, including the start one.

```delta
tm M {
  alphabet 0, 1
  start q0
  accept qa
  q0 -0 → x, R-> q1;   q0 -y → R-> q3;   q0 -⊔ → R-> qa
  q1 -0 → R-> q1;      q1 -y → R-> q1;   q1 -1 → y, L-> q2
  q2 -0 → L-> q2;      q2 -y → L-> q2;   q2 -x → R-> q0
  q3 -y → R-> q3;      q3 -⊔ → R-> qa
}
let r = M.run("0011")
assert r.verdict == :accept
assert len(r.steps) == 14        // the start configuration plus 13 moves
```
