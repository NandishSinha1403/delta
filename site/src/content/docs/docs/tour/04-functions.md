---
title: 4. Functions and control flow
description: fn, closures, if, for and while.
---

```delta
fn square(x) = x * x
fn fact(n: int) -> int {
  if n == 0 { return 1 }
  return n * fact(n - 1)
}
let add = fn(a, b) = a + b          // an anonymous function
fn adder(k) = fn(x) = x + k         // closures capture values
assert square(5) == 25
assert fact(10) == 3628800
assert adder(2)(40) == 42

var total = 0
for i ∈ 1..=10 { total = total + i }
var n = 0
while n < 5 { n = n + 1 }
assert total == 55 and n == 5
```

Annotations are optional and checked: calling `fact("x")` is an error before anything runs.
Every evaluation runs under a **budget** (steps, states, recursion depth, output), so a program
that loops stops with a budget outcome instead of hanging: `budget { steps: 1_000_000 }`.
