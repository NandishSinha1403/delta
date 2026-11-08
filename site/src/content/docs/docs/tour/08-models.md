---
title: 8. Models
description: Pushdown automata, options, and defining your own model.
---

A **model** says what a machine is made of: input, storage, acceptance. The prelude defines the
standard ones (see [Models](/docs/models/)); a pushdown automaton's labels read, pop and push:

```delta
pda P {
  start q0
  accept q3
  q0 -ε, ε → Z-> q1                  // push a bottom marker
  q1 -a, ε → A-> q1
  q1 -ε, Z → ε-> q3
  q1 -b, A → ε-> q2
  q2 -b, A → ε-> q2
  q2 -ε, Z → ε-> q3
}
check P accepts "", "ab", "aabb"
check P rejects "a", "ba", "aab"
```

The same label in the keyword form is `read a; pop X; push YZ`. Inside `-[ … ]->` you may
write the ASCII arrow: `-[a, X -> YZ]->`.

Define new models from the same blocks:

```delta
model qa { input one-way; epsilon; storage queue; accept by final }
qa Q {
  start s
  accept f
  s -read a; enqueue X-> s
  s -enqueue Z-> t               // a marker after the a's
  t -read b; dequeue X-> t
  t -dequeue Z-> f               // every a matched a b
}
check Q accepts "", "ab", "aabb"
check Q rejects "aab"
```
