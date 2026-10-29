# 05 — The Playground: UX, Layout, Responsiveness

Goal: feel like a **professional IDE on a laptop**, a **focused study tool on a tablet**, and a **genuinely usable app on a phone**: each layout designed for its screen, not shrunk down from the desktop one.

---

## 1. The four synchronized views

Every object on screen has the same four faces, always in sync (the code is the source of truth):

| View | Shows | Interaction |
|---|---|---|
| **Code** | the DSL text | type; diagnostics inline; quick fixes |
| **Diagram** | state diagram (book style: double circle = accept, arrow-in = start) | pan/zoom, drag states (writes `layout {}`), draw edges (writes δ entries), click to select |
| **Tuple** | the formal tuple rendered with KaTeX: M = (Q, Σ, δ, q₀, F) with each set listed | click any element to highlight it in code + diagram |
| **δ-table** | the transition table in the book's format (ε column for NFAs, Σ_ε × Γ_ε header for PDAs) | click a cell to jump to the code line |

Selection is shared: clicking `q2` anywhere highlights it in all four.

---

## 2. Layouts by screen size

### 2.1 Desktop (≥ 1280 px): three panes + bottom dock
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ δ Delta   [Examples ▾] [Share] [Export ▾]          object: [D ▾]   ☾  ⚙   │
├───────────────────┬──────────────────────────────────┬───────────────────────┤
│  CODE             │  DIAGRAM                         │  INSPECTOR            │
│  (CodeMirror)     │                                  │  [Tuple][δ][Steps]    │
│                   │     →(q1)──1──►((q2))            │  [Tests]              │
│  dfa M1 {         │       ↺0      ↺1  │0             │                       │
│    Q = {...}      │              ▲   ▼               │  M1 = (Q, Σ, δ, q1, F)│
│    ...            │              └─(q3)              │  Q = {q1, q2, q3}     │
│                   │                                  │  ...                  │
│                   │   [fit] [+] [−] [layout ▾]       │                       │
├───────────────────┴──────────────────────────────────┴───────────────────────┤
│  RUN  input: [0100      ] ▶ ⏸ ⏭ ⏮  speed ━━●━   | tape / fingers / stack     │
└──────────────────────────────────────────────────────────────────────────────┘
```
- All dividers are resizable and the layout is remembered (localStorage).
- **Compare mode**: the diagram pane splits to show two objects side by side (e.g. NFA and its DFA), with linked highlighting (hovering a DFA state highlights the NFA states in its subset).

### 2.2 Tablet (768–1279 px): two panes
- Landscape: Code | Diagram; Inspector and Run become a **bottom sheet** (drag up to expand).
- Portrait: Diagram on top, Code below (split adjustable); Inspector in a tab.
- Touch targets ≥ 44 px; pencil/stylus can draw edges.

### 2.3 Phone (< 768 px): one view at a time
```
┌─────────────────────────┐
│ ◆ M1 ▾            ⋯     │
├─────────────────────────┤
│                         │
│      (active view)      │
│                         │
├─────────────────────────┤
│ ε  ∅  ∪  *  →  ⊔  Σ  δ  │  ← symbol bar (above keyboard, Code view only)
├─────────────────────────┤
│ Code Diagram Tuple Run ⋯│  ← bottom tab bar
└─────────────────────────┘
```
- **Symbol bar**: phones can't type ε, ∅, ∪, →, ⊔ easily. One tap inserts them. Long-press shows the ASCII alternative.
- Diagram: pinch-zoom, double-tap to fit, long-press a state for a context menu (rename, make accept, delete).
- Run view: big step buttons at thumb height; the tape scrolls horizontally with the head centered.
- Conversion steps become a swipeable card stack.

### 2.4 Lecture / presentation mode (any size)
Large fonts, hidden chrome, step-through with keyboard or clicker, a laser-pointer cursor, and a high-contrast theme for projectors. For instructors.

**Breakpoints are tested** (Playwright) on: iPhone SE (375×667), Pixel 7, iPad (portrait/landscape), 1366×768 laptop, 1920×1080, 2560×1440.

---

## 3. Conversion Studio

Opened by any conversion (`dfa(N)`, `regex(G)`, `cnf(G4)` …), or from a right-click "Convert to ▸" menu.

```
 ┌──────── before ────────┐  ┌──────── after (step 7 / 15) ────────┐
 │  NFA N4                │  │  DFA under construction             │
 │  (subset {2,3}         │  │  new state {1,2,3} ← δ'({2,3}, a)    │
 │   highlighted)         │  │                                     │
 └────────────────────────┘  └─────────────────────────────────────┘
  Theorem 1.39:  δ'(R, a) = ⋃_{r∈R} E(δ(r, a))
  "From {2,3} on a: 2 → {2,3}, 3 → {1}; E(...) = {1,2,3}"
  ⏮ ◀ ━━━━━━━━●━━━━━━━━ ▶ ⏭     [faithful | optimized]    [explain more]
```
- A **timeline scrubber** over the trace. Every step shows the book formula (KaTeX) and a plain-English sentence.
- Toggle **faithful / optimized** to see the book's exact output vs the cleaned one.
- Algorithm-specific visuals:
  - Subset construction: the "worklist" of pending subsets; full-power-set mode greys unreachable subsets (Figure 1.43 → 1.44).
  - State elimination: the ripped state fades out; updated arrows show R1, R2, R3, R4 color-coded, then the new label (Figure 1.63).
  - Minimization: the triangular table fills; clicking a marked cell shows its distinguishing string.
  - CNF: rule list in the book's Example 2.10 style (new rules bold, removed rules struck through).
  - CYK: the triangular table fills cell by cell.
  - PDA → CFG: A_pq variables drawn as state pairs; pruned rules collapse.

---

## 4. Simulation views

| Model | What you see while stepping |
|---|---|
| DFA | input tape with head; current state glows; the path r₀…rₙ is drawn as a trail |
| NFA | **fingers**: every active state has a token; ε-closure expansion animates; optional computation-tree panel (Figure 1.29) |
| PDA | input tape + **vertical stack** beside the diagram; a branch selector when nondeterministic (or "explore all" tree view); budget meter |
| TM | infinite tape strip (scrollable), head, state; configuration log in the book's `u q v` format; step counter; "run to halt" with a progress bar and Stop |
| Multitape | k stacked tapes; toggle "single-tape encoding" (Theorem 3.13 view: `#` separators, dotted head symbols) |
| NTM | computation tree, breadth-first frontier highlighted |
| CFG | derivation stepper (uAv ⇒ uwv with the replaced variable underlined) and parse tree |

Controls everywhere: ▶ play, ⏸ pause, ⏭ step, ⏮ step back (history is kept), speed slider, "jump to end". Keyboard: `Space` play/pause, `→`/`←` step, `Home`/`End`.

**Batch test panel:** a table of strings → expected / actual / ✓✗, fed by the `check` commands in the code. Failing rows are clickable and open the run on that string.

---

## 5. Onboarding and content
- **Example gallery** organized by book chapter and example number: "Example 1.41 — NFA N4 → DFA", "Example 2.10 — CNF", "Example 3.7 — M2". Each opens as editable code. These double as the golden-test corpus.
- **First-run tour** (skippable, 4 steps): write → see diagram → run → convert.
- **Hover docs**: hovering `δ` in code shows Definition 1.5's line about δ; hovering `dfa(` shows Theorem 1.39's statement and complexity.
- **Empty state**: a blank editor with a one-click "Start from Definition 1.5 template".

---

## 6. Export and sharing
| Format | Use |
|---|---|
| Share link | whole program compressed into the URL hash; opens read-only-until-edited |
| SVG / PNG | slides, notes (PNG at 1×/2×/4×) |
| **LaTeX/TikZ** (`automata` library) | homework and papers: the diagram plus the tuple as LaTeX math |
| LaTeX tuple only | `M = (Q, \Sigma, \delta, q_0, F)` with the δ table as a `tabular` |
| Graphviz DOT | other tools |
| JSON | autograders, scripts |
| JFLAP `.jff` | interop with the most common existing tool |
| `.delta` file | save/open locally |

---

## 7. Visual design
- **Look:** calm, technical, textbook-like. Neutral background, one accent color for "active", a second for "accepting", a third for "rejected/dead". Diagrams use the book's conventions (start arrow, double circles, self-loops on top).
- **Type:** a clean sans for UI (e.g. Inter) and a monospace with good Unicode coverage for code (e.g. JetBrains Mono; it must render ε ∅ ∪ ⊔ Σ Γ δ correctly).
- **Themes:** light, dark (follows system, with manual override), high-contrast. Diagram colors are theme tokens, and exported SVGs use the light theme unless the user asks otherwise.
- **Motion:** 150–250 ms transitions; respects `prefers-reduced-motion` (animations become instant highlights).

---

## 8. Accessibility (WCAG 2.2 AA)
- Never color-only: active states get a thicker ring + glow; accept states keep the double circle; dead states are dashed.
- Full keyboard operation, including the diagram: Tab cycles states, arrows move between connected states, Enter opens an inspector.
- The **Tuple and δ-table views are the screen-reader-friendly representation** of the diagram. Each diagram has an accessible description generated from the tuple ("DFA M1, 3 states, start q1, accept q2; q1 on 0 goes to q1, …").
- Live region announces simulation steps ("read 1, moved to q2, 3 symbols left").
- Contrast checked in CI (axe).

---

## 9. Performance budgets (UI)
| Metric | Budget |
|---|---|
| First load (JS + CSS, gzipped, excluding lazy Graphviz) | ≤ 350 KB + WASM ≤ 600 KB |
| Time to interactive on a mid-range phone (4G) | ≤ 3 s |
| Keystroke → diagram update (≤ 50 states) | ≤ 100 ms |
| Animation | 60 fps for ≤ 200 states; above 300 states switch to canvas rendering or table view |
| Memory | engine work runs in a worker that can be recycled |
