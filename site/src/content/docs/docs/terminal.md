---
title: Terminal
description: "deltac: run Delta programs in a terminal, draw machines as pictures, save SVG, PNG, PDF and TikZ, and use the REPL."
---

`deltac` runs Delta outside the browser. It uses the same engine as the [Pad](/pad/), and draws
pictures with the same code, so a PNG saved in a terminal and one downloaded from the Pad are the
same file. (It is called `deltac` because `delta` is already the name of a popular git tool.)

## Install

Every way to install it, with the files for each platform, is on [Download](/download/).

| Where | Command |
|---|---|
| macOS (Homebrew) | `brew install nandishsinha1403/tap/delta-lang` |
| macOS, Linux (script) | `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-installer.sh \| sh` |
| Debian, Ubuntu (.deb) | `curl -LO https://github.com/NandishSinha1403/delta/releases/latest/download/delta-lang-amd64.deb && sudo apt install ./delta-lang-amd64.deb` (`arm64` on ARM) |
| Windows (PowerShell) | `powershell -ExecutionPolicy Bypass -c "irm https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-installer.ps1 \| iex"` |
| Windows (installer) | [delta-cli-x86_64-pc-windows-msvc.msi](https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-x86_64-pc-windows-msvc.msi) |
| Any platform (Rust) | `cargo install delta-cli` |

The Linux builds are static, so they run on any distribution. Then check that it works:

```sh
deltac --version
```

## Run a program

```sh
deltac run even-zeros.delta
```

`print` lines, test results (✓ and ✗) and shown values are printed in program order. Errors are
printed as `file:line:col: error[CODE]: message`, and `deltac` exits with status 1 if there are
errors or failing tests, so it works in scripts and CI.

| Option | What it does |
|---|---|
| `--show` | Draws shown machines as pictures, in terminals that can show images (kitty, Ghostty, iTerm2, WezTerm, foot and other sixel terminals). Elsewhere it lists the arrows as text. |
| `--format F` | Also writes every shown machine as a file: `svg`, `png`, `pdf`, `tikz` or `txt`, named `line-N.ext` after the line of its `show`. |
| `--out DIR` | The folder for those files (default: the current folder). |
| `--theme light\|dark` | Light (default) or dark pictures; a `save … (theme: …)` in the program wins. |
| `--json` | Prints the whole result as JSON (the [output schema](/schema/output-v2.json)). Writes no files. |

`deltac run -` reads the program from standard input. Set `DELTAC_IMAGES` to `kitty`, `iterm`,
`sixel` or `none` if the terminal is guessed wrong.

## Save files from a program

`save` writes a value to a file. The extension picks the format:

```delta
dfa M {
  start q0; accept q0
  q0 -0-> q1; q0 -1-> q0
  q1 -0-> q0; q1 -1-> q1
}
save M to "even-zeros.svg"
save M to "pictures/even-zeros.pdf"
save M to "even-zeros.tex"
save M.states to "states.txt"
```

| Extension | Contents |
|---|---|
| `.svg`, `.png`, `.pdf` | The diagram (machines only). PDFs keep the text as text. |
| `.tex` | A TikZ picture of the same diagram; needs only `\usepackage{tikz}`. |
| `.txt` | What `print` shows. |
| `.json` | The value as JSON. |

Paths are relative and stay inside the folder you run `deltac` in: no leading `/`, no drive
letter, no `..` ([E515](/docs/errors/e515/)). Folders are created as needed. In the Pad, each
`save` line in the Console has a **Download** button instead.

## The REPL

```sh
deltac repl
```

Type statements one at a time; an expression on its own prints its value. A machine or block
can span several lines: the REPL waits until the brackets close. A line with an error is shown
and forgotten, so the session always runs cleanly.

| Command | What it does |
|---|---|
| `:show on` / `:show off` | Draw shown machines as pictures |
| `:load FILE` | Run a file and keep its definitions |
| `:program` | Print the session so far |
| `:reset` | Start again |
| `:quit` | Leave (or Ctrl-D) |
