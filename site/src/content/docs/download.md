---
title: Download deltac
description: "Install deltac, Delta's command-line tool, on macOS, Linux or Windows: Homebrew, a shell script, a .deb, an MSI, PowerShell or cargo."
---

`deltac` runs Delta programs in a terminal, draws machines as SVG, PNG, PDF or TikZ, and has a
REPL. It uses the same engine as the [Pad](/pad/). What it can do is on the
[Terminal](/docs/terminal/) page.

## macOS

With [Homebrew](https://brew.sh):

```sh
brew install nandishsinha1403/tap/delta-lang
```

Or with the install script (Apple Silicon and Intel):

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-installer.sh | sh
```

## Linux

The install script works on any distribution (x86-64 and ARM64; the binary is static):

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-installer.sh | sh
```

On Debian and Ubuntu, a `.deb` (use `arm64` instead of `amd64` on ARM):

```sh
curl -LO https://github.com/NandishSinha1403/delta/releases/latest/download/delta-lang-amd64.deb
sudo apt install ./delta-lang-amd64.deb
```

Homebrew on Linux works too: `brew install nandishsinha1403/tap/delta-lang`.

## Windows

In PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-installer.ps1 | iex"
```

Or download the installer:
[delta-cli-x86_64-pc-windows-msvc.msi](https://github.com/NandishSinha1403/delta/releases/latest/download/delta-cli-x86_64-pc-windows-msvc.msi).

## With cargo

With a recent stable Rust ([rustup](https://rustup.rs)), on any platform:

```sh
cargo install delta-cli
```

The crate is `delta-cli`; the command it installs is `deltac`.

## Check it works

```sh
deltac --version
deltac repl
```

## All files

Every release on [GitHub](https://github.com/NandishSinha1403/delta/releases/latest) has these,
each with a `.sha256` checksum:

| Platform | File |
|---|---|
| macOS, Apple Silicon | `delta-cli-aarch64-apple-darwin.tar.xz` |
| macOS, Intel | `delta-cli-x86_64-apple-darwin.tar.xz` |
| Linux x86-64 (static) | `delta-cli-x86_64-unknown-linux-musl.tar.xz`, `delta-lang-amd64.deb` |
| Linux ARM64 (static) | `delta-cli-aarch64-unknown-linux-musl.tar.xz`, `delta-lang-arm64.deb` |
| Windows x86-64 | `delta-cli-x86_64-pc-windows-msvc.zip`, `delta-cli-x86_64-pc-windows-msvc.msi` |

The scripts install `deltac` into `~/.cargo/bin` (Windows: `%USERPROFILE%\.cargo\bin`) and tell
you if that folder isn't on your `PATH` yet.
