---
title: 1. Your first program
description: Write a DFA, test it, and run it step by step.
---

A machine is written the way you'd draw it: a start state, accepting states, and arrows.

```delta
dfa EvenAs {
  start even
  accept even
  even -a-> odd;   even -b-> even
  odd -a-> even;   odd -b-> odd
}
check EvenAs accepts "", "aa", "abba"
check EvenAs rejects "a", "ab"
run EvenAs on "abab"
```

- `check M accepts "…"` turns into one test per word; results appear in the **Tests** view.
- `run M on "w"` shows the run step by step in the **Run** view.
- Statements end at the end of a line, or at `;`.

The alphabet and the set of states were not written: Delta infers them from the arrows. Write
`states` and `alphabet` yourself when you want the checker to catch typos (or use
[strict mode](/docs/tour/05-machines/)).
