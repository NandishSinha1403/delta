# 07 — Suggestions from the Principal Developer

Opinionated advice, roughly in order of importance.

## Do these

1. **Ship Phase 0 first: deploy before you build.** Get an empty React page that calls one Rust function live at `delta.nandish.dev` in week one. Every later feature then ships the moment it works, and deployment never becomes a final-week surprise.

2. **Make "faithful to the book" your identity.** Many automata tools exist (JFLAP, various web simulators), but none reproduce *Sipser's* constructions step by step with the theorem number on every step. "Check your homework against the book" is the hook for students and instructors. Never let an optimization silently change the book's output.

3. **Build Chapter 1 completely before touching Chapter 2.** Regular languages are 80% of what students use this for, and they exercise every part of the architecture (DSL, engine, traces, diagram, worker, export). Chapters 2–3 then reuse it all. The DK-test even reuses the subset construction.

4. **Treat the trace as a first-class product.** The step-by-step trace is what makes this a *teaching* tool and not just a converter. Design `StepKind` carefully in Phase 1. Retrofitting traces later is painful.

5. **Write the brute-force oracles early.** A naive "does M accept w?" straight from the definition takes an afternoon and catches most bugs in the clever algorithms through property tests. That's what lets a solo developer trust exponential-time, fiddly code.

6. **Counterexamples, always.** Whenever the tool says "not equal", "not in the language" or "not minimal", show the shortest string that proves it. That's the single most useful feedback for a student.

7. **LaTeX/TikZ export early (Phase 2).** Students write homework in LaTeX. "Draw it here, paste into Overleaf" will spread the tool by word of mouth more than any other feature.

8. **Keep the Rust boring.** Index IDs, `Vec`s, owned data, one algorithm per file. You're learning Rust, and boring Rust is fast Rust anyway. Save clever generics for never.

## Avoid these

- **Don't build two-way diagram editing before v1.** Code as the single source of truth is the right call. Diagram edits as text patches come in Phase 5.
- **Don't promise answers to undecidable questions.** Label bounded checks honestly ("no difference found up to length 10"). Instructors will trust the tool *because* it is honest about Theorem 4.11.
- **Don't add accounts or a backend in v1.** Share links in the URL hash and local files cover sharing. A backend brings cost, privacy duties and maintenance for a solo project.
- **Don't support every notation.** Follow Sipser; warn on conflicts (`+` as union, missing ε). Dialect switches can come later if instructors ask.
- **Don't use Monaco** (bad on phones), and don't render diagrams with a generic graph library. Automata need book conventions (start arrow, double circles, self-loops, curved parallel edges).

## Ideas that would make it stand out

- **"Explain this state."** Click any state of a converted machine to see its provenance: "DFA state {1,3} = E({1}), the start state, because 1 has an ε-arrow to 3 (Theorem 1.39)."
- **Pumping-lemma game** with a CFG/PDA as the membership oracle (Phase 3). Students struggle most with these proofs, and no mainstream tool does this well.
- **Assignment links (classroom mode, v1.x):** an instructor shares a link with a language description and hidden test strings. The student builds a DFA, and the page shows pass/fail and a counterexample. Still static: the tests are hashed into the link.
- **Lexer-spec mode:** define tokens as regexes and get one combined DFA. That connects the theory to compilers, the real-world use the book mentions on p. 66.
- **Name: decided — Delta (δ).** Every machine in the book is defined by its δ. The language, the playground and the site (`delta.nandish.dev`) share the name.

## Practical setup tips for you
- Install Rust via `rustup`; add the WASM target: `rustup target add wasm32-unknown-unknown`.
- Use **VS Code + rust-analyzer**. It shows types inline, which is the fastest way to learn Rust.
- Read *The Rust Book* chapters 1–10 (ownership, structs, enums, collections, error handling). That's all this engine needs.
- Keep a `scratch/` folder of tiny Rust experiments. Port one algorithm from the prototype mindset (Python-like pseudocode in 02-CONVERSIONS.md) at a time.
- Start the Rust side with C2 (subset construction). It's small, central, and its golden test (Example 1.41) is already verified.
