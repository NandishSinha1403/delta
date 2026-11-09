# MANUAL.md: how to write a project manual (universal)

> **For the AI reading this:** you are about to write the manual for a software project.
> This file tells you what a good manual contains, how to write it, how to keep it true,
> and where it lives. Read it all, then read the project, then follow §12 step by step.
> Where this file and the project's own documents disagree, the project wins: say so in your report.

**Companion file:** if the project has a `DESIGN.md`, the manual follows its look (fonts, colours,
voice, banned words). If not, use the plain defaults in §11.

---

## 0. What a manual is

A manual explains **how the project works, how to use it, how to run it and how to change it**,
for someone who wasn't there when it was built. It's the book you'd hand a new teammate, a
curious visitor, or yourself in a year.

It is **not**:

- marketing (no "revolutionary", no promises);
- an API dump (reference tables belong in an appendix, generated from code);
- a changelog or a diary of how it was built (link to those instead);
- a copy of the whole codebase (the code lives in the repo; the manual explains it).

**One test for every paragraph:** would a reader who has never seen this project understand it,
and could they act on it?

---

## 1. Project brief (fill this in first)

Fill what you can from the repo. Ask the owner only for what you can't find, at most five
questions, all at once.

| Field | Answer |
|---|---|
| Project name | |
| One sentence: what it does, for whom | |
| Owner (person or team) | |
| Stage | idea / prototype / in use / mature / retired |
| Kind of project | library / web app / CLI / service or API / data pipeline / AI agents / mobile / embedded / other |
| Repo URL, default branch, commit the manual describes | |
| Readers, in order of importance (see §2) | |
| Public or private manual? Parts that must stay private? | |
| Where the manual lives (see §10) | e.g. `manual.<project>.nandish.dev`, `<project>.nandish.dev/manual/`, `docs/` in the repo, or a single `MANUAL.md` |
| Design system | `DESIGN.md` present? Which kit? |
| Sensitive areas | secrets, personal data, security work, paid services, legal limits |
| Facts that change often | prices, quotas, model names, third-party limits, versions |

---

## 2. Readers

Write for real people. Most projects have three or four of these; mark each chapter with who
it's for.

| Reader | They want | Time they'll give |
|---|---|---|
| **Visitor** | What is this, why does it exist, is it any good? | 5 minutes |
| **User** | How do I use it to get my result? | 30 minutes |
| **Operator** | How do I run, deploy, watch, back up and fix it? | An afternoon |
| **Contributor** | How is it built, and how do I change it safely? | A few days |
| **Owner** | Why is it this way, what are the rules, what's next? | Ongoing |

Put the visitor's material first and keep it short. Depth goes later.

---

## 3. The chapters

Use this order. Drop a chapter only if it truly doesn't apply, and say so in one line
("This project has no server, so there's no Operating chapter"). Renumber nothing: a reader
should find "How it works" in the same place in every manual you write.

| # | Chapter | Reader | What goes in it |
|---|---|---|---|
| 0 | **Cover** | All | Name, one sentence, status, owner, "Describes commit `abc1234`, updated DD MMM YYYY". |
| 1 | **The idea in one page** | Visitor | The problem, who has it, what this project does about it, what it deliberately doesn't do. One diagram at most. |
| 2 | **Quick start** | User | From nothing to a first result in about 10 minutes. Exact commands, expected output, what to do if it fails. |
| 3 | **How it works** | All | The big-picture diagram. The life of one request, job or task from start to finish. A short glossary of the project's own words. |
| 4 | **Tour of the parts** | Contributor | One section per component: what it does, why it exists, where it lives in the code, what goes in and out, how it fails. |
| 5 | **Data** | Operator, Contributor | What's stored, where, the shape of it (schema summary), how long it's kept, who can see it, how it's backed up. |
| 6 | **Using it** | User | Every screen, command or endpoint a user touches, task by task ("How to …"), not menu by menu. |
| 7 | **Operating it** | Operator | Install and deploy; configuration (names of settings and secrets, never their values); monitoring; backups and restore; runbooks (§5.4); the stop switch, if any. |
| 8 | **Changing it** | Contributor | Dev setup, code layout, conventions, tests, recipes ("How to add a new …"), review and release process. |
| 9 | **Rules and decisions** | Owner, Contributor | Hard rules and why each exists; an index of decision records; ideas that were tried and dropped, and why. |
| 10 | **Security and safety** | Operator, Owner | What could go wrong (a short threat model), what is never published, how to report a problem. |
| 11 | **Limits and costs** | Owner, Operator | Known issues, quotas, rate limits, what it costs to run (even if ₹0, say so and say how that's enforced). |
| 12 | **What's next** | Owner, Visitor | The roadmap in plain words, with what's done and what isn't. |
| 13 | **Credits and licences** | All | Licence, dependencies and their licences, data sources and their terms, people and works it draws on. |
| A | **Appendices** | As needed | Command reference, configuration reference, error messages and fixes, FAQ, full glossary. Generate these from code where you can (§8). |

### 3.1 Emphasis by kind of project

| Kind | Make these chapters strongest |
|---|---|
| Library | 2 Quick start, 6 Using it (as a cookbook), A API reference (generated) |
| Web app | 3 How it works, 6 Using it (by screen and by task), 7 Operating it |
| CLI tool | 2 Quick start, 6 Using it (by task), A command reference (generated from `--help`) |
| Service or API | 3, 5 Data, 7 Operating it, A endpoint reference |
| Data pipeline | 3 (follow one record end to end), 5 Data, 7 runbooks for bad input and reruns |
| AI agents | 3 (one task end to end), 4 (each agent or role), 9 Rules, 10 Safety, 11 Limits and costs |
| Mobile app | 6 Using it (by screen), 7 release and store process |
| Embedded or hardware | 2 (wiring and flashing), 5, 7 (field fixes), 10 (physical safety) |

---

## 4. Writing rules

- **Plain words.** Short sentences, active voice. Write the way a knowledgeable colleague would talk.
- **Define before use.** The first time a project-specific word appears, explain it in one line and add it to the glossary.
- **Second person for instructions.** "Run `make test`." Not "the user should run".
- **Every claim checkable.** Numbers come with a source and a date. "It's fast" becomes "a frame draws in about 3 ms on a 2022 MacBook Air (measured 31 OCT 2026)".
- **Say why, not just what.** A rule without its reason gets broken; one line of reason is enough.
- **No marketing words.** If the project's `DESIGN.md` has a banned-words list, follow it.
- **Honest about gaps.** Write "Not built yet" or "Untested on Safari" plainly, not around it.
- **Dates as `DD MMM YYYY`**, with the time zone when time matters.
- **Headings in sentence case.** One idea per section. If a section needs more than three levels of heading, split it.

### 4.1 Procedures (anything the reader does)

Every procedure has the same shape:

1. **When to use it**, in one line.
2. **Before you start:** what you need (access, tools, a backup).
3. **Numbered steps**, one action each, with the exact command or click.
4. **What you should see** after the steps that matter.
5. **If it goes wrong:** the likely failures and fixes.
6. **How to undo it**, or a plain warning that it can't be undone.

### 4.2 Runbooks (when something breaks)

Title each one by the symptom the operator sees, not the cause: "The site shows old numbers",
not "Cache invalidation". Then: how to confirm it, the fix, how to check it's fixed, and how to
stop it happening again.

---

## 5. Code in the manual

- **Excerpts, not dumps.** Show the 5 to 30 lines that matter, never whole files.
- **Always cite the source:** file path, and the commit or tag, so the reader can open the full file.
- **Explain every excerpt** in the sentence before it: what to look at and why it matters.
- **Keep excerpts true.** Pull them from the source by marker or line range at build time, rather than pasting by hand (§8). A pasted excerpt that drifts from the code is worse than none.
- **Never** show secrets, tokens, real keys, private hostnames, personal data, or details of an undisclosed security problem, even in examples. Use clearly fake values like `EXAMPLE_KEY_123`.
- Commands are copy-pasteable: no prompt characters inside the code block, one command per line, and the expected output in a separate block.

---

## 6. Diagrams

- Draw them as text that renders: **Mermaid** or hand-written **SVG**. No screenshots of diagrams, and no AI-generated images.
- **One idea per diagram:** the big picture, one flow, or one data shape.
- Label every box with the name used in the code.
- Give each diagram a one-paragraph text version underneath, for screen readers and for search.
- Keep them small enough to read on a phone; split a big one into an overview and close-ups.

---

## 7. Screens and examples

- Screenshots are allowed for UI the reader will see, taken from the real app, with fake data.
- Crop to what matters; add a short caption saying what to look at.
- Mark example data as examples. Never use real people's names, emails or accounts.

---

## 8. Keeping it true

A manual that lies is worse than no manual. Build freshness in.

- **Pin it to a commit.** The cover says which commit the manual describes.
- **Generate what you can from code:** command help, config keys and defaults, endpoint lists, schema tables, error codes. Mark generated sections so nobody edits them by hand.
- **Volatile facts** (prices, quotas, third-party limits, model names, versions) carry a `verified on DD MMM YYYY` note and a link to the source. Anything older than 30 days is shown as possibly out of date.
- **Automatic checks**, run with the project's tests:
  - every code excerpt still matches its source;
  - every command in Quick start runs on a clean machine (or in CI);
  - every internal link and anchor resolves; external links are checked weekly;
  - every glossary word used is defined;
  - no secret-shaped strings appear anywhere in the manual.
- **When the code changes, the manual changes in the same change.** Reviews ask: "Which manual section does this touch?"

---

## 9. Safety and privacy

- **Public or private.** Decide per chapter. Operating details, internal hostnames, security runbooks and anything about undisclosed vulnerabilities belong in a private manual, or nowhere.
- **Never include:** secrets or their partial values, personal data, private keys, customer data, exact internal addresses, details of security problems that aren't fixed and disclosed.
- **Settings and secrets by name only:** `GITHUB_APP_KEY_PATH`, never its contents.
- If the project has a public view and a private view (for example a public site and a private admin area), the manual says which one each section describes.

---

## 10. Where the manual lives

Pick one and record it in the brief. If unsure, ask the owner once.

| Option | When | How |
|---|---|---|
| **A. Its own subdomain** | The project is public and the owner has a domain (often `nandish.dev`) | `manual.<project>.nandish.dev`, or `<project>.nandish.dev/manual/` if the project already has a site there. |
| **B. A folder in the repo** | Open source, or the readers live on GitHub | `docs/` with an `index.md`; GitHub renders it. Optionally published as static pages. |
| **C. One file** | Small project, few readers | A single `MANUAL.md` at the repo root, with a contents list at the top. |
| **D. Private only** | Internal tools, or chapters that must stay private | Behind the project's login (for example Cloudflare Access), or kept in the repo and never published. |

Whatever the host, the manual is:

- **static** (plain HTML, or Markdown that renders without a build server);
- **free of tracking,** analytics and outside requests, unless the owner asks;
- **readable on a phone** (360 px wide, no sideways scrolling) and printable;
- **linkable,** with a stable anchor on every heading;
- **searchable** (a small client-side index is enough), when it's more than one page.

---

## 11. Look and navigation

- **If `DESIGN.md` exists:** use its fonts, colours, voice and components, and treat the manual as one of its surfaces.
- **If not, plain defaults:** a readable serif or system sans for text, a monospace for code, a line length near 70 characters, generous spacing, light and dark themes that follow the device, and one accent colour for links.
- **Navigation** for multi-page manuals: a contents sidebar (a menu on phones), "On this page" links for long chapters, previous and next links, and breadcrumbs.
- **Footer on every page:** "Describes commit `abc1234` · Updated DD MMM YYYY · Source in `docs/…`".
- Code blocks with a copy button; tables that scroll inside their own box on phones.

---

## 12. Process for the AI

1. **Read.** The README, any `CLAUDE.md` or contributor guide, `DESIGN.md`, the decision log or ADRs, the task list or roadmap, the config files, the tests, and the code's top-level layout.
2. **Fill the brief** (§1) from what you read. List what you couldn't find.
3. **Propose an outline:** the chapters from §3 that apply, with one line each on what they'll say, plus where the manual will live (§10). Ask your questions (at most five, all at once) together with the outline. If nobody answers, choose the most reasonable option and say which you chose.
4. **Write chapter by chapter,** in order 1, 2, 3, then the rest. Cite file paths as you go.
5. **Verify** every command, path, number and link against the repo. Run Quick start for real if you can.
6. **Set up freshness:** generated sections, excerpt markers and the checks in §8.
7. **Check safety** (§9): search the result for secrets, personal data, internal hostnames and private material.
8. **Report:** what you wrote, where it lives, what you couldn't verify, what's missing from the project itself (things the manual exposed as unclear), and how to keep it current.

---

## 13. Definition of done

- [ ] The brief (§1) is filled and stored with the manual.
- [ ] Every chapter in §3 is written or explicitly marked "not applicable" with a reason.
- [ ] A newcomer can get a first result from Quick start in about 10 minutes, and the steps were actually run.
- [ ] "How it works" follows one real request, job or task from start to finish.
- [ ] Every code excerpt cites its file and commit, and a check keeps it matching.
- [ ] Every volatile fact has a source and a `verified on` date.
- [ ] No secrets, personal data or undisclosed security details anywhere; public and private parts are separated.
- [ ] Works on a 360 px phone and prints cleanly; every heading has an anchor.
- [ ] Follows `DESIGN.md` if the project has one; otherwise the defaults in §11.
- [ ] The report lists what couldn't be verified and what's still missing.

---

## 14. Skeleton

Start from this. Replace every line in angle brackets.

```markdown
# <Project> manual

<One sentence: what it does, for whom.>
Status: <stage> · Owner: <name> · Describes commit `<sha>` · Updated <DD MMM YYYY>

## Contents
1. The idea in one page
2. Quick start
3. How it works
4. Tour of the parts
5. Data
6. Using it
7. Operating it
8. Changing it
9. Rules and decisions
10. Security and safety
11. Limits and costs
12. What's next
13. Credits and licences
Appendices

## 1. The idea in one page
<The problem. Who has it. What this does about it. What it deliberately doesn't do.>

## 2. Quick start
Before you start: <requirements>
1. <command>
   You should see: <output>
If it fails: <likely cause and fix>

## 3. How it works
<Big-picture diagram + text version.>
### The life of one <request/job/task>
<Step by step, naming the real components.>
### Glossary
- **<term>**: <one-line meaning>

## 4. Tour of the parts
### <Component>
What it does · Why it exists · Where: `<path>` · In and out · How it fails

## 5. Data
<What's stored, where, schema summary, retention, access, backup.>

## 6. Using it
### How to <task>
<Procedure, §4.1 shape.>

## 7. Operating it
<Deploy, configuration by name, monitoring, backup and restore.>
### Runbook: <symptom>
Confirm · Fix · Check · Prevent

## 8. Changing it
<Setup, layout, conventions, tests, recipes, release.>

## 9. Rules and decisions
- **<rule>**: <why>
Decision records: <index or link>
Dropped ideas: <what and why>

## 10. Security and safety
<Threat model in brief, what's never published, how to report a problem.>

## 11. Limits and costs
<Known issues, quotas (verified on <date>), running cost and how it's capped.>

## 12. What's next
<Done / in progress / later.>

## 13. Credits and licences
<Licence, dependencies, data sources, acknowledgements.>

## Appendices
<Generated references, FAQ, error messages.>
```
