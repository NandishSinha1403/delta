---
title: 3. Sets and words
description: Set-builder notation, word operations and word sets.
---

Set-builder notation is the only comprehension form.

```delta
let Q = {0, 1, 2, 3}
assert { q ∈ Q | q % 2 == 0 } == {0, 2}                   // filter
assert { (p, q) | p ∈ {0, 1}, q ∈ {'a'} } == {(0, 'a'), (1, 'a')}
assert [ len(w) | w ∈ ["", "a", "ab"] ] == [0, 1, 2]       // the list form keeps order
```

Words have their own operations:

```delta
let w = "0110"
assert len(w) == 4 and w[0] == '0'
assert w ++ "1" == "01101"
assert "ab"^3 == "ababab"
assert rev(w) == w
assert count(w, '1') == 2
assert w starts with "01" and w ends with "10" and w contains "11"
assert w[1..3] == "11"
```

`Σ^n` is every word of length n, and `Σ^≤n` every word up to length n:

```delta
let Σ = {'a', 'b'}
assert Σ^2 == {"aa", "ab", "ba", "bb"}
assert len(Σ^≤3) == 15
```

`Σ*` is infinite, so it is allowed only inside `lang` (see [languages](/docs/tour/09-languages/)).
