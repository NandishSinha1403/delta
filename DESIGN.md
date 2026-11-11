# Design

<!-- Documented from the built front page (site/src/pages/index.astro) and docs theme
     (site/src/styles/theme.css, site/ec.config.mjs). The Pad (web/) keeps its own look for now. -->

## World: Dark Tech

High-contrast text on black; every datum in a monospace; square, brutalist buttons against fully
rounded status chips; interaction shown by brighter 1px rings and faint radial glows, never by
color floods. The product's own artifacts (state diagrams, traces, step replays) are the imagery.

## Color

| Token | Value | Use |
|---|---|---|
| bg | `#000` | page ground |
| card | `#0a0a0a` | panels, code blocks |
| ring | `rgba(255,255,255,.145)` | 1px ring on every card (`box-shadow: 0 0 0 1px`) |
| chip | `#1f1f1f` | pills, inline code, selected list rows |
| white | `#fff` | headlines, CTA fill |
| text | `#ededed` | body on black; FAIL dot |
| sub | `#e7e7e7` | hero subtext |
| muted | `#999` | secondary text, data labels, WARN dot |
| accent | `#52a8ff` | active state only: current trace step, edges/operators in code, focus ring |
| ok | `#62c073` | PASS / accept dots, added lines |

No other hues. Red is not part of the system: failure is `#ededed` on a dark chip.

## Type

- **Schibsted Grotesk** (variable, self-hosted, OFL): headlines weight 500; hero `clamp(32px,
  2.6vw + 22px, 60px)` at `-0.045em`; section heads `clamp(28px, 3.2vw, 44px)` at `-0.04em`;
  metrics 56px at `-3.36px`; body 16px/1.6.
- **IBM Plex Mono** (400/500, self-hosted, OFL): code, chips, table data, metrics units, captions.
  Tabular numerals everywhere data is aligned.

### Code font (Q-32)

IBM Plex Mono is the one code font everywhere code appears: the Pad editor (400/600), code on the
site, docs and manual, exports (SVG/PNG/PDF), `deltac` terminal images and social cards. It has no
Greek and few maths symbols, so two subsets fill exactly the characters it lacks, in this order:

1. **Noto Sans Math**: Greek, ∪ ∩ ∈ ⊆ ≡ ⊔ ⊢ ▸ ⟨ ⟩ and the other maths symbols and arrows.
2. **Noto Sans Mono**: superscripts, subscripts and modifier letters (⁺ ⁿ ᵏ ᵐ ₖ).

Stack: `'IBM Plex Mono', 'Delta Math', 'Delta Sup', ui-monospace, …` on the web (the two
fallback faces carry `unicode-range`, so they load only when used); `IBM Plex Mono, Noto Sans
Math, Noto Sans Mono` in SVG. All files are self-hosted and cut by `scripts/subset-fonts.py`.
`crates/delta-export/fonts/symbols.txt` lists every symbol Delta code uses; a Rust test (exports)
and a Playwright test (the Pad) fail if any of them is drawn by another font.

### State names

State names in diagrams are set in **STIX Two Text italic** (OFL), as a maths text sets them:
`q₀` is an italic *q* with a smaller upright-baseline subscript. Everywhere `delta-render` draws
(the Pad's diagrams, SVG/PNG/PDF/TikZ exports, `deltac` images, social cards): 18px names,
subscripts at 0.65 of that, circles of radius 20–24px, an ellipse beyond that. Edge labels stay
IBM Plex Mono. The subset (ASCII, primes, subscript digits) and the width table delta-render
measures names with (`crates/delta-render/src/stix_widths.rs`) are both cut by
`scripts/subset-fonts.py`; the Pad loads it as `--font-state`.

### PNG watermark

Every PNG file a user gets (`save … to "x.png"`, `deltac --format png`, the Pad's PNG download)
has an 18px strip under the drawing, on the drawing's background, holding **Delta by Nandish**
right-aligned in IBM Plex Mono 10px (`#6b7884` on light, `#8c9199` on dark). The drawing itself
is unchanged; a drawing narrower than the mark is centred on a wider canvas. SVG, PDF and TikZ
files, terminal previews and social cards have no mark (`delta_export::png_file` vs `png`).

Everything that isn't code keeps its font: Schibsted Grotesk on the site and manual, IBM Plex
Sans for the Pad's UI, DotGothic16 for the logo, the maths serif for formulas.

## Components

- **btn-square**: radius 0, `#fff` on `#121212` text, 8×12px (large: 14×20px, 20px text), a 12px
  diagonal-arrow SVG; hover `#e6e6e6` plus a 4px white-8% ring.
- **chip**: inline-flex, `#1f1f1f`, radius 100px, 4×10px, gap 6px, 12px mono uppercase `#999`,
  6px status dot (`#62c073` pass, `#999` warn, `#ededed` fail).
- **card**: `#0a0a0a`, radius 12px, 1px ring; hover brightens the ring to white-24% and reveals a
  faint `#52a8ff` radial glow in the corner.
- **timeline bar**: 6px track, inner bar positioned by percentage (left = start, width = span),
  radius 2px, `rgba(255,255,255,.18)`, `#52a8ff` on the active row.
- **diff/replay list**: mono rows with a 2px inset side line, green for additions, blue for edits.
- **header**: absolute at 38px, 56px side padding; logo mark is a start arrow into an accepting
  (double) state; below 760px the nav becomes a full-screen black menu with 36px links.

## Layout

Max content width 1240px, side padding 56px (20px on phones). Sections 128px apart (88px on
phones). Section heads are a two-column grid: heading left, muted lede right. Hero is 100svh with
the image under `linear-gradient(to top, rgba(0,0,0,.76) 0%, rgba(0,0,0,0) 93.785%)` and a solid
black fade at the bottom.

## Motion

One authored moment: the trace steps row by row (650ms per step) when it scrolls into view, the
current step's head bar turning blue. Eases are `cubic-bezier(.16,1,.3,1)`. Reduced motion shows
the final step with no animation.

## Imagery

`site/public/img/hero-dfa.webp`: the Pad's own render of the divisible-by-5 DFA, captured at 2×
and cropped (provenance: produced by the Delta Pad from `samples`-style code, not generated).
