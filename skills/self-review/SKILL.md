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

## Fast Path — Rust Script

If the repo has `scripts/self-review.rs`, prefer the script:

```bash
./scripts/self-review.rs
```

For a dry run:

```bash
./scripts/self-review.rs --dry-run
```

For a specific reflect file:

```bash
./scripts/self-review.rs --reflect .ctx/reflect-YYYY-MM-DD.md
```

The script should:

- find the latest `.ctx/reflect-*.md`
- create `.ctx/reflect-YYYY-MM-DD.md` if no reflect file exists
- fill the four `## Patterns & Surprises` placeholder subsections
- leave `## Open questions` unchanged
- write `.ctx/logs/self-review/<same-filename>`
- print `self-review: wrote ...`
- print the completed `## Patterns & Surprises` section

If `scripts/self-review.rs` fails, fall back to the manual workflow and briefly report the
script error.

## Manual Fallback

Use this workflow if `scripts/self-review.rs` does not exist or its execution fails.

## Step 1 — Find the reflect file

```bash
ls -t .ctx/reflect-*.md 2>/dev/null | head -1
```

If none found, create `.ctx/reflect-YYYY-MM-DD.md` with the standard sections:

- `## Shipped`
- `## Unfinished`
- `## Memory-bank source`
- `## Patterns & Surprises`
- `## Open questions`

Prefer seeding `Shipped` and `Unfinished` from `.ctx/opavs/memory-bank/active-context.md`.
If that is unavailable, try `.ctx/godmode/memory-bank/active-context.md`, then the matching
`progress.md` file in either memory bank. Legacy `.ctx/memory-bank/*.mbx.md` files are the final
fallback. If none has usable entries, use
`- Nothing recorded yet.` for `Shipped`, `Unfinished`, and `Open questions`. Use
`- Nothing notable this session.` for each `Patterns & Surprises` subsection. Record which
memory-bank file was used, or that no memory-bank seed was available, under
`## Memory-bank source`.

## Step 2 — Read the reflect file

Read the full file. Extract:

- **Commits** from `## Shipped`
- **Diff stat** from `## Files changed`
- **Placeholder sections** under `## Patterns & Surprises`

## Step 3 — Analyse and fill in each section

For each placeholder subsection, reason only from visible evidence in the reflect file:

### Took longer than expected

Look for repeated fix/retrigger commits, CI/fmt churn, large unexpected diffs, or rework
commits.

### Went smoothly

Look for single-commit features, clean merges, test additions, docs/code follow-through, and
absence of follow-up fixes.

### Discovered mid-session

Look for unexpected fixes, docs corrections after code work, newly visible unfinished work, or
breakages caught by hooks.

### Next session speedups

Reason forward from what took long or remained unfinished. Keep each section to 2-4 bullets.

Be specific — reference commit SHAs or file names where relevant. Do not use generic filler.

## Step 4 — Write the completed file

1. Replace the four `(fill in next session)` placeholders in `## Patterns & Surprises` with
   the filled-in content (edit in-place). Do not edit `## Open questions` unless explicitly
   asked.

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

## Common Failures

| Symptom                                    | Fix                                                                                    |
| ------------------------------------------ | -------------------------------------------------------------------------------------- |
| No `.ctx/reflect-*.md` files exist         | Create `.ctx/reflect-YYYY-MM-DD.md` and seed it from memory-bank context when possible |
| `Patterns & Surprises` is already filled   | Archive as-is and print the section                                                    |
| Placeholders remain under `Open questions` | Leave them alone unless the user asks to fill that section                             |
| `scripts/self-review.rs` fails             | Fall back to the manual workflow and report the script error briefly                   |
