---
title: Models
description: The standard prelude, model properties, and defining your own models.
---

A **model** says what a machine is made of. Every machine is an instance of one. The prelude is
ordinary Delta code:

```delta
model dfa2   { input one-way; deterministic; total; accept by final }
model nfa2   { input one-way; epsilon; accept by final }
model pda2   { input one-way; epsilon; storage stack; accept by final }
model dpda2  extends pda2 { deterministic }
model tm2    { input on-tape; storage tape; deterministic; accept by halting; missing reject }
model ntm2   extends tm2 { not deterministic }
model lba2   extends tm2 { bounded }
print dfa, nfa, gnfa, pda, dpda, tm, ntm, lba, mtm(2), twdfa, mealy, moore, fst, ctr(2)
```

(The names above end in 2 so they don't replace the prelude's own models.)

| Property | Values |
|---|---|
| `input` | `one-way`, `two-way`, `on-tape` |
| `storage` | `none`, `stack`, `tape`, `tapes(k)`, `queue`, `counters(k)`, or several (`stack, stack`) |
| `accept` | `by final`, `by empty-stack`, `by halting`, `by final-and-empty` |
| `output` | `none`, `per-transition`, `per-state` |
| flags | `deterministic`, `total`, `epsilon`, `labels regex`, `special-form`, `bounded` |
| `missing` | `reject`, `dead`, `error`: what an absent transition means |

The checker enforces each property: a `deterministic` model rejects two edges that can apply in
the same configuration, and `total` requires a move for every state and symbol.

## Labels by model

| Model | Short form | Keyword form |
|---|---|---|
| dfa / nfa | `a`, `ε` | `read a` |
| pda | `a, X → YZ` | `read a; pop X; push YZ` |
| tm | `0 → x, R` · `x → R` | `read 0; write x; move R` |
| mealy | `a / 1` | `read a; emit 1` |
| two stacks | | `read a; pop X; push2 Y` |
| queue | | `read a; dequeue X; enqueue YZ` |
| counters | | `read a; inc c1; if c2 == 0` |

`push YZ` leaves `Y` on top. A stack starts empty unless the machine has the option
`(initial stack Z)`.
