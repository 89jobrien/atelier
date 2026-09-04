---
name: herald
description: >
  Cross-repo activity synthesizer — discovers all git repos with recent commits, writes a
  narrative summary to the Obsidian daily note. Use when the user says /herald, "end of day
  summary", "what happened today", "cross-repo standup", "write the daily note", or wants a
  view of activity across projects. Also use at session end when multiple repos were touched.
---

# Herald

Synthesize cross-repo activity into the Obsidian daily note.

## Repo Discovery

Resolve `dev_dir` from config, falling back to `$HOME/dev`. Use Glob to discover `.git`
directories and files beneath it, deduplicate their parent directories, and retain repositories
with commits in the past 24 hours. Include nested repositories and linked worktrees.

For each active repo, collect commits:

```bash
git -C "<discovered-absolute-repo-path>" log --since="24 hours ago" --oneline
```

## Template

Read `template` from `$HOME/.ctx/handoff.global.config.toml` when present. Otherwise use
`$HOME/Documents/Obsidian Vault/08_Templates/Template - Herald Summary.md`.

Read the template before writing. Fill the `## Herald Summary` section using that structure.

## Format Rules

- **Summary**: what moved and why it matters. Length proportional to the work — a single commit gets a sentence; a multi-repo day gets a paragraph per thread.
- **No framing**: never say "heavy day", "light day", "solid day", or rank the day's output.
- **Per-repo blocks**: one `### <repo>` heading per active repo, with commits listed as `- \`<hash>\` <message>`.
- **Omit idle repos**: skip any repo with zero commits in the window.

## Vault Path

Resolve the daily-notes directory from the configured `vault` value, falling back to
`$HOME/Documents/Obsidian Vault/01_Daily`. Write to `$vault/YYYY-MM-DD.md`.

If today's note doesn't exist, create it. If it exists, append or fill the `## Herald Summary` section.

## Agent

The `atelier:herald` agent runs this skill with full tool access. It also:

- Resolves `vault` and `dev_dir` from config using the same fallbacks documented above
- Supports `--repo <name>`, `--window <duration>`, and `--dry-run` flags
- Updates project memory files after writing the vault entry

Invoke via `/herald` or by spawning `atelier:herald` directly.
