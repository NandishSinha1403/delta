---
title: "10. Security and safety"
description: A short threat model, what is never published, and how to report a problem.
---

Readers: **operator**, **owner**.

## Threat model (short)

| Threat | Where | Mitigation |
|---|---|---|
| A program that runs forever or allocates without limit | Engine | Budgets (steps, states, depth, output); the worker is killed after 10 s |
| A malicious share link | Pad | The program only runs in the engine, which has no file, network or DOM access; output is rendered as text, not HTML |
| `save` writing outside the folder | `deltac` | The engine rejects absolute paths, drive letters and `..` (E515); the CLI checks again after resolving symlinks and refuses to write through a link |
| A crafted SVG reaching the exporter | export | Only SVG that Delta's renderer produced is exported; resvg reads no system fonts or external images (features off) |
| Leaked Cloudflare credentials | Operator's Mac | wrangler OAuth stays on the Mac; CI tokens will be GitHub secrets scoped to Pages; no tokens in the repo or this manual |
| The manual becoming public | Pages | A password gate on every hostname (Pages Function; HMAC cookie, constant-time checks, fails closed without the secret), noindex headers and meta, robots `Disallow: /`, no links, a build check for secret-shaped strings |
| Copyright | Repo | The textbook PDF is gitignored; quotations live only in explain profiles and this private manual |
| Supply chain | Dependencies | Few, pinned dependencies, each approved; `pnpm` blocks unknown install scripts (`@ibm/plex-mono`'s telemetry script is denied) |

## Never published

Secrets or parts of them, account ids, the owner's email, the manual's password (a Pages secret),
personal data of any kind, the textbook, and details of any unfixed security problem.

## Reporting a problem

Open a private security advisory on the GitHub repository (Security → Advisories), or email the
owner. Don't open a public issue for a vulnerability.

## Questions they'll ask

1. **"Can a Delta program attack the user?"** — It runs in a WASM sandbox inside a worker with no
   imports for files, network or DOM; the worst it can do is use CPU until a budget or the
   10-second watchdog stops it.
2. **"How do you prevent path traversal in `save`?"** — Validation twice: the engine's
   `save_path` (relative, no empty, `.` or `..` parts, no drive letter) and the CLI's
   `write_inside`, which canonicalizes the deepest existing folder and refuses symlinked targets.
3. **"Is one password enough for the manual?"** — For one reader and non-secret content, yes,
   and it costs nothing to run. Trade-offs: no per-person identity, no audit log, and no rate
   limit on guesses (a long password matters). Cloudflare Access would add those; it was
   dropped because it needs a Zero Trust enrolment with billing consent.
4. **"What about XSS through program output?"** — React escapes text; outputs are strings or
   structured JSON; the diagram is SVG built from data, not from user HTML.
5. **"What's your biggest residual risk?"** — Operator credentials on one laptop, and no
   monitoring yet.

## Self-test

<details class="selftest"><summary>1. Which two layers validate save paths?</summary>The engine (E515 via save_path) and the CLI (write_inside: canonical parent inside cwd, no symlink target).</details>
<details class="selftest"><summary>2. What stops an infinite loop in the browser?</summary>Budgets inside the engine; the 10-second worker watchdog outside it.</details>
<details class="selftest"><summary>3. What does the manual's cookie contain?</summary>HMAC-SHA256 of a fixed string keyed by MANUAL_PASS — never the password; changing the secret invalidates it.</details>
<details class="selftest"><summary>4. Where is the owner's email allowed to appear?</summary>Nowhere in the repo or the manual (git commit metadata aside).</details>
<details class="selftest"><summary>5. How should a vulnerability be reported?</summary>A private GitHub security advisory or email to the owner — never a public issue.</details>
