---
title: "7. Operating it"
description: Deploying the site, the Pad and this manual on Cloudflare; configuration by name; runbooks.
---

Readers: **operator**. This chapter describes private operating details; it stays in the
private manual.

## What runs where

| Thing | Host | How it gets there |
|---|---|---|
| Front page, docs, Pad (`/pad/`) | Cloudflare Pages project `delta`, `delta-17m.pages.dev`, custom domain `delta.nandish.dev` | Direct upload with wrangler from a Mac |
| This manual | Pages project `delta-manual`, `delta-manual.pages.dev` | Direct upload, behind a password gate (Pages Function) |
| v1 | GitHub Pages from `main` (no longer on the domain) | — |
| DNS | name.com: one CNAME `delta` → `delta-17m.pages.dev` | Manual |

## How to deploy the site

**When:** after a phase or a docs change, once all checks pass. **Before you start:** a wrangler
login (`npx wrangler@4.42.0 login`), Rust, Node 22.

1. Run every check in `CLAUDE.md` (fmt, clippy, tests, WASM build and size, web lint,
   typecheck, tests, build, e2e).
2. `./scripts/build-site.sh` — builds the Pad into `site/public/pad`, runs `deltac gen-docs`,
   builds the site, runs the SEO check.
3. `npx wrangler@4.42.0 pages deploy site/dist --project-name=delta --branch=main`
4. If `~/.wrangler` now exists, move it away (see the runbook below) — **also between two
   deploys in a row** (site, then manual): the second one otherwise fails without saying much.

**You should see:** `✨ Deployment complete!` and a preview URL. Check `/`, `/pad/`, `/tools/`,
`/robots.txt`, `/sitemap-index.xml`. New pages can 404 for a few seconds while the deployment
propagates; retry with `?v=1` before worrying.

**Undo:** Pages → `delta` → Deployments → pick the previous one → Rollback.

## How to deploy this manual

1. `./scripts/build-manual.sh` (fails on a missing excerpt marker or a secret-shaped string).
2. `npx wrangler@4.42.0 pages deploy manual/dist --project-name=delta-manual --branch=main`
   Run it **from `manual/`** (`cd manual && npx wrangler@4.42.0 pages deploy dist
   --project-name=delta-manual --branch=main`): wrangler only uploads `functions/` — the password
   gate — when it's in the current folder.
3. Open the URL in a private window: you must see the password page, not the manual.

**Never** deploy the manual without `functions/_middleware.js`: it is the only thing between
the pages and the internet. The gate runs on every hostname of the project, including each
deployment's own `<hash>.delta-manual.pages.dev` URL, and fails closed (500) if `MANUAL_PASS` is
missing. Never add a custom domain.

## Configuration (names only)

| Name | Where | What |
|---|---|---|
| wrangler OAuth login | `~/Library/Preferences/.wrangler/config/default.toml` on the Mac | Pages deploys and secrets |
| `MANUAL_PASS` | Pages secret on `delta-manual` (production) | The manual's password. Set with `npx wrangler@4.42.0 pages secret put MANUAL_PASS --project-name=delta-manual`; never in the repo |
| `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` | GitHub secrets (not set yet) | For `deploy.yml` once Actions is on |
| `HOMEBREW_TAP_TOKEN` | GitHub secret (not set yet) | cargo-dist's Homebrew publish job |
| `BEACON_TOKEN` | `site/src/lib/analytics.mjs` | Cloudflare Web Analytics site token (public by design, empty for now) |
| `DELTAC_IMAGES` | user's shell | Forces the terminal image protocol |

## Releases of `deltac`

`dist-workspace.toml` configures cargo-dist 0.34.0: five targets, shell/PowerShell installers,
Homebrew (`NandishSinha1403/homebrew-tap`), MSI. Locally:
`dist build --artifacts=local --target aarch64-apple-darwin` → `target/distrib/`. The generated
`.github/workflows/release.yml` builds and publishes on a `v*` tag once GitHub Actions is on.

## Monitoring

None yet beyond Cloudflare's dashboard. Planned: Cloudflare Web Analytics on the public site
(cookieless). The Pad reports engine errors to the console only.

## Runbook: wrangler says I'm not logged in, right after a deploy worked

- **Confirm:** `ls ~/.wrangler` shows a folder.
- **Fix:** move it away (`mv ~/.wrangler ~/wrangler-moved`). Newer wrangler versions create it
  and then ignore the OAuth login stored in `~/Library/Preferences/.wrangler`.
- **Check:** `npx wrangler@4.42.0 whoami` shows the account.
- **Prevent:** keep wrangler pinned at 4.42.0 in scripts and docs.

## Runbook: the site shows old pages after a deploy

- **Confirm:** the deployment's own preview URL shows the new page, the domain doesn't.
- **Fix:** wait a minute; request with a query string to skip caches. Pages serves HTML with
  `max-age=0, must-revalidate`, so it clears itself.
- **Check:** `curl -sI https://delta.nandish.dev/<page>` returns 200.
- **Prevent:** verify new routes after each deploy (the deploy procedure lists them).

## Runbook: the WASM size check fails

- **Confirm:** `./scripts/check-wasm-size.sh` prints which module is over (engine 700 KB,
  export 1,000 KB gzipped).
- **Fix:** find what grew (`twiggy top`, or compare `cargo tree`); move heavy code behind the
  lazy export module or a feature flag.
- **Prevent:** the check runs before every commit.

## Runbook: the domain stops serving

- **Confirm:** `dig +short delta.nandish.dev CNAME` should be `delta-17m.pages.dev.`; the Pages
  project's custom-domain status should be `active`.
- **Fix:** restore the CNAME at name.com. Rollback to v1 is the CNAME back to
  `nandishsinha1403.github.io`.

## Runbook: the manual's password doesn't work

- **Confirm:** the page says "Wrong password" for the right one, or shows "The manual is not
  configured" (the secret is missing).
- **Fix:** set the secret again (`npx wrangler@4.42.0 pages secret put MANUAL_PASS
  --project-name=delta-manual`), then redeploy from `manual/` so the new value is used. Changing
  the secret signs every browser out (the cookie is derived from it).
- **Last resort:** the source is in the repo: `./scripts/build-manual.sh`, then open
  `manual/dist/` with any static server.

## Questions they'll ask

1. **"Why direct upload instead of Git integration?"** — GitHub Actions was locked on the
   account, and Pages' Git integration would build on Cloudflare's machines without the Rust/WASM
   toolchain set up. Building on the Mac and uploading the result is simple and reproducible.
2. **"How do you roll back?"** — Pages keeps every deployment; rollback is one click, or
   redeploy an older commit's build.
3. **"How is the manual kept private?"** — A Pages Function in front of every request: a
   password page until the right password is posted, then a 30-day Secure, HttpOnly cookie
   holding an HMAC of the secret (not the password), compared in constant time; plus noindex
   headers and robots, no links from the public site.
4. **"What would CI look like?"** — `ci.yml` (checks) on every push, `deploy.yml` on push to
   `main`, `release.yml` on tags. They exist; Actions has to be turned on.
5. **"What's your stop switch?"** — Rolling back to a previous deployment, or the CNAME rollback
   to v1. There is no server to shut down.

## Self-test

<details class="selftest"><summary>1. Which wrangler version, and why pinned?</summary>4.42.0; newer ones create <code>~/.wrangler</code> and ignore the stored OAuth login.</details>
<details class="selftest"><summary>2. What must be deployed with the manual, every time?</summary><code>manual/functions/_middleware.js</code> (deploy from <code>manual/</code>), with the <code>MANUAL_PASS</code> secret set.</details>
<details class="selftest"><summary>3. What does build-site.sh do?</summary>Builds the Pad into site/public/pad, runs gen-docs, builds the site, runs the SEO check.</details>
<details class="selftest"><summary>4. How do you roll the domain back to v1?</summary>Point the name.com CNAME <code>delta</code> back at <code>nandishsinha1403.github.io</code>.</details>
<details class="selftest"><summary>5. Which GitHub secrets will CI need?</summary>CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID, HOMEBREW_TAP_TOKEN.</details>
