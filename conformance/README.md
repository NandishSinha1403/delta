# Conformance suite

Every example in `docs/v2/01-LANGUAGE.md` as an executable test (04-ROADMAP.md §3, "Language").
These cases are the definition of done for the language. The implementation follows them, and the
spec is changed first when a case has to change.

## Layout

```
cases/NN-name.delta         the program: spec examples verbatim where possible
cases/NN-name.expect.json   what `deltac run --json` must print (partial match)
xfail.txt                   cases expected to fail for now
```

Lines marked `// Context` are not in the spec. They define the machines and values that a spec
fragment refers to. Expectations that depend on a provisional answer in `docs/v2/QUESTIONS.md`
say so in a comment.

## Running

```bash
cargo test -p delta-cli --test conformance                 # passes iff exactly xfail.txt fails
cargo test -p delta-cli --test conformance -- --ignored    # requires every case to pass
```

The harness (`crates/delta-cli/tests/conformance.rs`) runs `deltac run --json FILE` and kills it
after 10 s. It fails a case that is not in `xfail.txt` and doesn't match, and it also fails when a
case listed in `xfail.txt` now passes, so the list only shrinks.

## Matching rules

The output schema is `02-ENGINE.md` §7: `{ ok, diagnostics: [{ code, severity, line, col, message,
fixes }], outputs: [{ kind, … }] }`.

| In the expectation | Matches |
|---|---|
| object | an object with **at least** these keys, each matching recursively |
| array | an array of the **same length**, element by element |
| `{ "$len": n }` | an array of length n |
| `{ "$each": p }` | an array whose every element matches p (combine with `$len`) |
| `"E7*"` | a string starting with `E7` (codes whose exact number isn't in the spec yet) |
| `null` | null **or absent** (e.g. `"bounded": null` = an exact, unbounded result) |
| anything else | equal |

`"diagnostics": []` means no diagnostics at all, including warnings.
