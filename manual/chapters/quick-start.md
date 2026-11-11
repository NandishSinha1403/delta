---
title: "2. Quick start"
description: From nothing to a first result in about ten minutes, in the browser or in a terminal.
---

Readers: **user**, **contributor**.

## In the browser (1 minute)

1. Open `https://delta.nandish.dev/pad/`.
2. The tour program is already there. Look at the **Tests** view: all three checks pass.
3. Change line `q2 -b-> q0` to `q2 -b-> q2` and wait a moment.

You should see: both `EndsAB ≡ …` checks fail with `counterexample: "abb"` (the bounded one and
the exact one); `subsets(N) ≡ N` still passes. Undo the change.

**If it fails:** an old browser without WebAssembly or module workers shows an error banner; use a
current Chrome, Firefox or Safari.

## In a terminal (10 minutes, first build)

**Before you start:** git, a recent stable Rust (from rustup), and about 2 GB of disk for the
build.

1. Get the code and build the CLI:

   ```sh
   git clone https://github.com/NandishSinha1403/delta
   cd delta
   git checkout v2
   cargo install --path crates/delta-cli
   ```

   You should see `Installed package delta-cli … (executable deltac)` after a few minutes.

2. Save this as `ends-ab.delta`:

   ```delta
   dfa EndsAB {
     start q0
     accept q2
     q0 -a-> q1;   q0 -b-> q0
     q1 -a-> q1;   q1 -b-> q2
     q2 -a-> q1;   q2 -b-> q0
   }
   check EndsAB ≡ re"(a ∪ b)* a b"
   check EndsAB accepts "ab", "bab"
   save EndsAB to "ends-ab.png"
   ```

3. Run it:

   ```sh
   deltac run ends-ab.delta
   ```

   You should see:

   ```text
   ✓ line 8: EndsAB ≡ re"(a ∪ b)* a b"
   ✓ line 9: EndsAB accepts "ab"
   ✓ line 9: EndsAB accepts "bab"
   saved ends-ab.png
   ```

   The PNG has a small "Delta by Nandish" line under the drawing, as SVG and PDF files do; TikZ
   files don't.

4. Try the REPL: run `deltac repl`, type `:load ends-ab.delta` (it runs the file and keeps its
   definitions), then `EndsAB.accepts("aab")`. It prints `true`. Leave with `:quit`.

**If it fails:**
- `error: linker 'cc' not found` — install the platform C tools (Xcode command-line tools on
  macOS, `build-essential` on Debian/Ubuntu).
- `deltac: command not found` — `~/.cargo/bin` isn't on `PATH`; open a new terminal or run
  `source "$HOME/.cargo/env"`.

**How to undo:** `cargo uninstall delta-cli`.

## The Pad locally (contributors)

**Before you start:** Node 22 (via `fnm`) and pnpm.

```sh
./scripts/build-wasm.sh
cd web
pnpm install
pnpm dev
```

You should see Vite print `Local: http://localhost:5173/`; open it and the tour appears.
If the page says the engine failed to load, the WASM wasn't built: run `./scripts/build-wasm.sh`
again from the repo root and check it printed no error.

## Questions they'll ask

1. **"How long does a cold build take?"** — A few minutes for the CLI (resvg and svg2pdf are the
   big dependencies); the WASM build is similar. Incremental builds are seconds.
2. **"Why `deltac` and not `delta`?"** — `delta` is git-delta, a popular diff pager; installing
   both would clash on `PATH`. See [War stories](/war-stories/).
3. **"Is there a binary download?"** — Not yet. cargo-dist is configured (shell and PowerShell
   installers, Homebrew, MSI) but releases need GitHub Actions, which is off. See
   [Operating it](/operating/).
4. **"Does the Pad need a server?"** — No. It's static files; the engine runs in your browser.

## Self-test

<details class="selftest"><summary>1. Which command installs the CLI from a checkout?</summary><code>cargo install --path crates/delta-cli</code> (the binary is <code>deltac</code>).</details>
<details class="selftest"><summary>2. What must you run before <code>pnpm dev</code> in web/?</summary><code>./scripts/build-wasm.sh</code>, which builds the engine and export WASM into <code>web/src/engine/pkg</code> and <code>export-pkg</code>.</details>
<details class="selftest"><summary>3. Where does <code>save … to "x.png"</code> write in the terminal, and in the Pad?</summary>Terminal: inside the current folder. Pad: nowhere — the Console shows a Download button.</details>
<details class="selftest"><summary>4. What does a failed ≡ check print?</summary>A ✗ line with "they differ on" and the shortest word in exactly one of the two languages.</details>
<details class="selftest"><summary>5. Which Node version does the web build need?</summary>22 (the repo uses fnm: <code>fnm use 22</code>).</details>
