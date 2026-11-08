# 03 — Products: Front Page, Pad, Docs, Embeds, Mobile Apps

All products share **one engine** (WASM), **one Pad codebase** (React) and **one design system**.

---

## 1. Site map (static, GitHub Pages)

| Path | Page |
|---|---|
| `/` | **Front page** (§2) |
| `/pad` | The Pad (§3) |
| `/pad#code=…` | Shared program (compressed in the URL hash, no server) |
| `/pad#code=…&embed=1` | Embeddable Pad (§5) |
| `/docs` | Docs home → Tour, Library, Cookbook, Notation, Models, Languages and classes, Errors, AI |
| `/docs/errors/E701` | One page per error code |
| `/examples` | Gallery by topic; each opens in the Pad |
| `/download` | App store badges, PWA install instructions |
| `/llms.txt`, `/docs/ai` | AI guide (long and short), generated |
| `/schema/output-v2.json` | Output schema |

**Tech:** Astro + Starlight for the front page and docs (static, fast, search built in). The Pad is a React app at `/pad` and also an **island** inside the front page and docs, so every code sample is live.

---

## 2. The front page

Goal: within ten seconds a visitor understands what Delta is, and within thirty they've edited and run something.

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ δ Delta        Docs   Examples   Download   GitHub              [Open the Pad]│
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│   The language of computation.                                                │
│   Write machines, languages and constructions as code.                        │
│   See them run.                                                               │
│                                                                               │
│   [ Open the Pad ]   [ Take the tour ]        ▢ App Store   ▢ Google Play     │
│                                                                               │
│  ┌─ live, editable ───────────────────────┬─ output ────────────────────────┐ │
│  │ dfa EndsAB { … }                       │   (diagram, animated run of     │ │
│  │ lang L = { w ∈ {a,b}* | w ends "ab" }  │    "abab", ✓ EndsAB ≡ L up to   │ │
│  │ check EndsAB ≡ L up to 10   ▶ Run      │    10)                          │ │
│  └────────────────────────────────────────┴─────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────────────────────┤
│  Four feature sections, each = short text + a small live example:             │
│   1. Write it like math        set-builder languages, Unicode or ASCII        │
│   2. Any machine               models from storage + acceptance; define new   │
│   3. Ask only what's decidable the E701 example and its bounded alternative   │
│   4. Constructions as code     explore / fixpoint; step through any of them   │
├───────────────────────────────────────────────────────────────────────────────┤
│  "Learn by doing": cookbook cards (DFA for divisible-by-n, regex → NFA, PDA   │
│   for aⁿbⁿ, TM for palindromes, Mealy machine) → open in Pad                  │
├───────────────────────────────────────────────────────────────────────────────┤
│  For teachers: embeds, strict mode, explain profiles (Sipser, HMU, Linz)      │
│  For AI tools: JSON output, AI guide, (soon) API and MCP                      │
├───────────────────────────────────────────────────────────────────────────────┤
│  Footer: A project by Nandish Sinha · GitHub · Docs · License · v2.0          │
└───────────────────────────────────────────────────────────────────────────────┘
```

- **The hero is real.** The example is an embedded Pad that runs in the visitor's browser. The diagram animates the run on load, once, and stops; `prefers-reduced-motion` turns that off.
- The page uses the Pad's design system (near-black, the palette, Courier Prime for code). The big headline uses **IBM Plex Sans**, with the logo word in **DotGothic16**.
- **Performance:** the hero loads the engine lazily after first paint. Lighthouse ≥ 95, and LCP under 2 s on 4G.
- **On a phone** the hero stacks: headline, then buttons, then the editor and output as tabs.

---

## 3. The Pad

### 3.1 Desktop
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ δ Delta  [Samples ▾] [▶ Run ⌘↵] [Auto ○]  [Format] [Share] [Docs]        ☾ ⚙ │
├─────────────────────────────────┬────────────────────────────────────────────┤
│ EDITOR                          │ [Diagram] [Table] [Steps] [Tests] [Lang]   │
│                                 │  object [M ▾]                 Fit + − SVG  │
│                                 │                                            │
│                                 ├────────────────────────────────────────────┤
│                                 │ CONSOLE                                    │
├─────────────────────────────────┴────────────────────────────────────────────┤
│ ● 0 ▲ 0  tests 6/6  Ln 12 Col 4   notation default · explain off · ran 4 ms  │
└──────────────────────────────────────────────────────────────────────────────┘
```

| Panel | Shows |
|---|---|
| **Diagram** | Any machine: dotted grid, active-state glow while stepping, ε edges dashed, label pills; storage views (stack column, tape strip, counters, queue) beside it during runs |
| **Table** | Formal tuple (math typeset) + transition table; the screen-reader view of the diagram |
| **Steps** | Replay of any construction: built-in, `explore` or `fixpoint`. Scrubber, step text, optional citation |
| **Tests** | `check` and `assert` results; bounded results labeled; click → counterexample or trace |
| **Lang** | For a `lang` value: its class (with a lattice graphic), sample words in and out, which questions are decidable for it |
| **Console** | `print`, errors with line links, budget outcomes, timing |

- **Run** (⌘/Ctrl + Enter) is the main action. **Auto** (re-run while typing) is off by default, because programs can loop. **Stop** appears when a run takes longer than 300 ms.
- **Editor:**
  - CodeMirror 6 with VS Code Dark+/Light+ syntax colors;
  - autocomplete for keywords, the library, names in scope, the current machine's states, and **LaTeX names** (`\delta` ⇥ δ);
  - hover docs, signature help, squiggles and quick fixes;
  - formatter, bracket matching, find/replace.
- **Symbol bar** (ε ∅ ∪ ∩ ∈ → ⊔ Σ δ ≡) with long-press for the ASCII alternatives.
- **Samples:** about 20 programs covering every major feature.

### 3.2 Tablet and phone
- **Tablet:** editor on top, output below, draggable split.
- **Phone:** tabs **Code / Output**, a floating ▶ Run button, the symbol bar above the keyboard, and thumb-sized step controls. The Output tab holds a segmented control for Diagram / Table / Steps / Tests / Lang / Console.

---

## 4. Docs

| Section | Content |
|---|---|
| Tour | 12 short pages, each with live examples: values → machines → models → languages and classes → explore/fixpoint → grammars → TMs → transducers |
| Library | **Generated** from signature records: every built-in with signature, methods and live examples |
| Models | The prelude explained, and how to define your own model |
| Languages and classes | The lattice, closure tables, decidability tables, bounded checks, `as … by` |
| Cookbook | ≥ 30 recipes ("DFA for strings with an even number of 0s and odd number of 1s", "regex → NFA, textbook construction", "minimize and explain why states merged", "PDA for palindromes", "TM that adds binary numbers", "Mealy machine for parity"…) |
| Notation | Profiles and how exam notation maps to Delta |
| Errors | One page per code |
| AI | How to use Delta with AI assistants; the long and short guides; JSON schema |

**Every code block in the docs runs in CI.** A broken example fails the build.

---

## 5. Embeds

```html
<iframe src="https://delta.nandish.dev/pad/#code=…&embed=1&panel=diagram&run=1"
        width="100%" height="420" loading="lazy" title="Delta example"></iframe>
```

Options (in the hash): `embed=1`, `panel` (diagram, tests, lang, console), `run`, `readonly`. The `code` value is the program, deflated and base64url-encoded. A "Copy embed" button lives in the Pad's Share menu. The front page and docs use the same embed internally.

---

## 6. Mobile apps (Android and iOS)

### 6.1 Approach: Capacitor, not Flutter

| Option | Verdict |
|---|---|
| **Capacitor** (wraps the web Pad in a native app shell) | **Chosen.** Same React code and same WASM engine, plus native plugins for files, share sheet, haptics and storage. Mature and widely used for this purpose |
| Flutter | Rewriting the entire UI in Dart; a second codebase to keep in sync; weaker code editor |
| Tauri 2 mobile | Promising (Rust-native), but its mobile support is younger. Revisit later for desktop apps |
| React Native | Would still need a WebView for CodeMirror; no real gain |

**First, a PWA** (Phase 3): Delta installs to the home screen on Android and iOS straight from the browser and works offline. Store apps come later (Phase 8) and add discoverability, file handling and a native feel.

### 6.2 What the apps add beyond the website
- **Offline by default:** engine, docs and examples all bundled.
- **Files:** open and save `.delta` files from the Files app or Drive; the app registers the `.delta` file type, so tapping a file opens it in Delta.
- **Native share sheet:** share a program as a link, a `.delta` file, or an SVG/PNG of the diagram.
- **Haptics** on run, and on test pass/fail.
- **Deep links:** `delta.nandish.dev/pad#…` links open in the app if it's installed.
- **Safe-area aware layout,** with the keyboard and symbol bar tuned for phones and tablets (iPad split view).

### 6.3 Practical requirements
- **Accounts and fees:** an **Apple Developer Program** membership (about US$99 per year) and a **Google Play Console** account (about US$25, one time). Check current prices when you sign up.
- **Building for iOS needs a Mac with Xcode** (you have one). Android builds use Android Studio.
- **App Store review:** Apple can reject apps that are "just a website" (minimum-functionality guideline). Offline mode, file handling, the share sheet and the phone-optimized UI are what make Delta a real app. Build them **before** submitting.
- **Releases:** GitHub Actions builds signed Android (AAB) and iOS (IPA) artifacts; uploads go through Play Console and App Store Connect (TestFlight first).
- **One version number** shared by web, apps and engine.

---

## 7. Design system (shared by every product)

| Token | Value |
|---|---|
| Background / panels / raised | `#0D0E10` / `#111215` / `#17191C` |
| Borders | `#23262B`, `#2A2D33` |
| Text / muted | `#F1FAEE` / `#8C9199` |
| Active (selection, current cell, active state, Run) | `#A8DADC` (text on it `#0D0E10`) |
| Brand red (logo tile, Share, primary CTA on the front page) and errors | `#E63946` |
| Tints | navy `#1D3557`, active fill `#16263D`, steel blue `#457B9D` |
| Accept states | `#F1FAEE` double ring |
| Fonts | code: Courier Prime (fallback Courier New → STIX Two Math); UI: IBM Plex Sans; logo: DotGothic16; math: STIX Two Text |
| Syntax | VS Code Dark+ / Light+ |
| Light theme | paper `#F1FAEE`, ink `#1D3557`, active `#457B9D` |
| Motion | 150–250 ms; respects reduced motion |
| Accessibility | WCAG 2.2 AA; full keyboard use; Table panel as the diagram's text alternative |
