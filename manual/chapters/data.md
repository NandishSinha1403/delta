---
title: "5. Data"
description: What Delta stores, where, for how long, and who can see it. (Almost nothing, on purpose.)
---

Readers: **operator**, **contributor**.

Delta has **no server and no database**. Everything below lives on the user's device or in a
URL, or is static content.

| Data | Where | Shape | Kept | Who sees it |
|---|---|---|---|---|
| Open files in the Pad | Browser `localStorage`, key `delta.v2.files` | `{ files: [{ id, name, source }], activeId }` | Until the user clears site data | The user only |
| Old single source (v1) | `localStorage` `delta.source` | string | Read once to migrate into `delta.v2.files` | The user |
| Theme, Auto switch | `localStorage` `delta.theme`, `delta.auto` | strings | Until cleared | The user |
| Offline copy of the Pad | Service worker cache `delta-v2` | HTTP responses | Replaced on each visit; old caches deleted on activate | The user |
| Share links | The URL fragment: `#code=…` | deflate-raw, then base64url | Wherever the link is pasted | Anyone with the link. The fragment is never sent to the server |
| REPL history | `~/.deltac_history` | one entry per line | Until deleted | The local user |
| Files from `save` | The folder `deltac` runs in | svg/png/pdf/tex/txt/json | User's choice | The local user |
| Site and Pad | Cloudflare Pages project `delta` | static files | Every deployment is kept by Cloudflare | Public |
| This manual | Pages project `delta-manual` | static files | Every deployment | Whoever has the password |

**Backups.** The source of truth for everything published is the git repository (GitHub,
branch `v2`). Any deployment can be rebuilt from a commit. Cloudflare keeps previous deployments,
so a bad deploy can be rolled back in the dashboard (Pages → project → Deployments → "Rollback").
User programs are not backed up by Delta; share links are the way to keep or move them.

**Storage failures are expected.** Private windows and blocked site data make `localStorage`
throw; every read and write in `web/src/state/ui.ts` is wrapped, so the Pad still works, just
without remembering files.

## Questions they'll ask

1. **"How do share links work without a server?"** — The program is compressed with the
   browser's `CompressionStream('deflate-raw')`, base64url-encoded and put after `#code=`. The
   fragment never reaches the server; opening the link decompresses it in the page.
2. **"What's the limit on a share link?"** — Browsers accept very long URLs, but chat apps and
   QR codes don't; typical programs compress to a few hundred characters. A server-side short
   link is planned with accounts.
3. **"Any personal data?"** — None collected. Web Analytics (cookieless, Cloudflare) is planned
   for the public site only; the manual has none.
4. **"How would you add accounts?"** — A small backend (Cloudflare D1 or Supabase) for saved
   projects and short links, keeping the engine client-side. It's in the roadmap's "later" list.
5. **"What happens to saved files when the format changes?"** — The key is versioned
   (`delta.v2.files`), and the v1 key is migrated on first load; a v3 would do the same.

## Self-test

<details class="selftest"><summary>1. Where does the Pad keep open files?</summary><code>localStorage["delta.v2.files"]</code>.</details>
<details class="selftest"><summary>2. Does a share link send the program to Cloudflare?</summary>No: it's in the URL fragment (after #), which browsers don't send.</details>
<details class="selftest"><summary>3. How is a bad deploy undone?</summary>Roll back to a previous deployment in the Pages dashboard, or redeploy an older commit's build.</details>
<details class="selftest"><summary>4. What happens in a private window where storage throws?</summary>Reads and writes are wrapped in try/catch; the Pad works but doesn't remember files.</details>
<details class="selftest"><summary>5. Where does the REPL keep history?</summary><code>~/.deltac_history</code>.</details>
