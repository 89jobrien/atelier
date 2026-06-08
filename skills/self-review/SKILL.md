---
name: self-review
description: >
  Fill in the Patterns & Surprises section of the latest session reflect file and write
  the completed review to .ctx/logs/self-review/. Use when the user runs "/self-review",
  asks to "fill in the reflect", "write the session review", or "what patterns did we hit".
argument-hint: "[optional: repo path — defaults to cwd]"
allowed-tools: Read, Write, Edit, Bash, Glob
---

# Self-Review

Analyse the latest session reflect file, fill in the Patterns & Surprises section, and
write the result to `.ctx/logs/self-review/`.

## Step 1 — Find the reflect file

```bash
ls -t .ctx/reflect-*.md 2>/dev/null | head -1
```

If none found, report and stop.

## Step 2 — Read the reflect file

Read the full file. Extract:

- **Commits** from `## Shipped`
- **Diff stat** from `## Files changed`
- **Placeholder sections** under `## Patterns & Surprises`

## Step 3 — Analyse and fill in each section

For each of the four placeholder subsections, reason from the commit list and diff stat:

### Took longer than expected

Look for: repeated fix/retrigger commits on the same thing (e.g. multiple `ci: retrigger`
commits indicate a stuck workflow), large diff counts on unexpected files, or commits whose
subject suggests rework (`fix(ci)`, `fix: update ... regression`).

### Went smoothly

Look for: single-commit features that landed cleanly, quality gates that passed first try
(no follow-up fix commits), refactors with no test regressions.

### Discovered mid-session

Look for: commits that address something not in the original intent (e.g. `fix: close clone
closure and pipe fds` appearing after CI work suggests a latent bug surfaced during review),
doc-only fixes following code changes, unexpected breakages caught by hooks.

### Next session speedups

Reason forward: given what took long, what setup or guard would prevent it next time?
Examples: a CI lint job that catches drift earlier, a pre-push check that validates a
previously manual step, a rate-limit on version bumps that prevents noisy commits.

Keep each section to 2-4 bullet points. Be specific — reference commit SHAs or file names
where relevant. Do not use generic filler.

## Step 4 — Write the completed file

1. Replace the four `(fill in next session)` placeholders in the reflect file with the
   filled-in content (edit in-place).

2. Write a copy to `.ctx/logs/self-review/` using the same filename:

```bash
mkdir -p .ctx/logs/self-review
cp .ctx/reflect-<date>.md .ctx/logs/self-review/reflect-<date>.md
```

## Step 5 — Report

Print a one-line confirmation:

```
self-review: wrote .ctx/logs/self-review/reflect-<date>.md
```

Then print the filled-in `## Patterns & Surprises` section so the user can read it inline.

## Notes

- Never fabricate patterns. Only reference what is visible in the commit log and diff stat.
- If a section genuinely has nothing to note, write `- Nothing notable this session.`
- The `## Open questions` section is separate — leave it as-is unless the user asks to fill it.
