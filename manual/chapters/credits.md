---
title: "13. Credits and licences"
description: Delta's licence, its dependencies and fonts, and the works it draws on.
---

Readers: **all**.

**Delta** is MIT-licensed (`Cargo.toml` `[workspace.package] license = "MIT"`).

## Main dependencies

| Dependency | Used for | Licence |
|---|---|---|
| serde, serde_json | JSON everywhere | MIT / Apache-2.0 |
| wasm-bindgen, console_error_panic_hook | The WASM boundary | MIT / Apache-2.0 |
| resvg 0.45, svg2pdf 0.13 | PNG and PDF export | Apache-2.0 / MIT (resvg: Apache-2.0 or MIT) |
| rustyline 19 | The REPL's line editing | MIT |
| icy_sixel 0.7 | Sixel images in terminals | MIT / Apache-2.0 |
| proptest | Property tests | MIT / Apache-2.0 |
| React 19, Zustand, Comlink | The Pad | MIT / MIT / Apache-2.0 |
| CodeMirror 6, Lezer | The editor | MIT |
| @hpcc-js/wasm-graphviz | On-screen diagram layout | Apache-2.0 |
| KaTeX | Math in the Pad | MIT |
| Astro, Starlight | Site and this manual | MIT |
| cargo-dist (tool) | Release packages | MIT / Apache-2.0 |

## Fonts (all SIL Open Font License 1.1)

- **IBM Plex Mono** 2.5.0 (IBM): the one code font: the Pad, the site, this manual, exports,
  `deltac` images and social cards (subsets cut by `scripts/subset-fonts.py`).
- **Noto Sans Math** 3.000 (the Noto Project Authors): Greek and the maths symbols Plex lacks.
- **Noto Sans Mono** (the Noto Project Authors): the superscripts Plex lacks (⁺ ⁿ ᵏ).
- **Schibsted Grotesk**: the site's headlines and prose.
- **IBM Plex Sans, DotGothic16**: the Pad's UI and logo.

## Works it draws on

The course structure follows Michael Sipser, *Introduction to the Theory of Computation*, 3rd
edition (Cengage, 2013). The engine contains no text from it; citations appear only in explain
profiles and in this private manual. Algorithms are the standard ones, each credited in its
deep dive (Rabin–Scott, Hopcroft, Brzozowski, McNaughton–Yamada, Thompson, Glushkov, Earley,
Cocke–Younger–Kasami).

## Questions they'll ask

1. **"Can you use Sipser's examples?"** — Ideas and standard constructions aren't copyrightable;
   text and figures are. The engine has neutral names and ids; citations are data in profiles.
2. **"Why MIT?"** — Maximum reuse by teachers and tools; nothing to protect commercially.
3. **"Are the fonts safe to embed in the binary?"** — Yes, OFL allows embedding and
   redistribution with the licence; the licences ship next to the font files.
4. **"Any GPL dependencies?"** — No; the licence column is MIT/Apache/OFL throughout.
5. **"Who else contributed?"** — Name honestly: the owner, and AI coding assistants under the
   owner's direction.

## Self-test

<details class="selftest"><summary>1. Delta's licence?</summary>MIT.</details>
<details class="selftest"><summary>2. Which fonts are embedded in exports, and why three?</summary>IBM Plex Mono, plus Noto Sans Math and Noto Sans Mono subsets for the characters Plex lacks (Greek, ⊔ ⊢ ▸, and superscripts like ⁿ).</details>
<details class="selftest"><summary>3. Which library lays out on-screen diagrams?</summary>Graphviz, via @hpcc-js/wasm-graphviz.</details>
<details class="selftest"><summary>4. Where may Sipser be quoted?</summary>Only in explain profiles and this private manual.</details>
<details class="selftest"><summary>5. What does the REPL use for line editing?</summary>rustyline 19.</details>
