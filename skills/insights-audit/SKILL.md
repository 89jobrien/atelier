---
name: insights-audit
description:
  Use when the user runs "/insights-audit", asks to "validate the insights report", "audit
  the report against my projects", "check what the report got wrong", or wants to
  cross-check an /insights-generated HTML report against actual local project directories.
model: sonnet
effort: medium
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Edit
---

# insights-audit

Cross-check an `/insights`-generated HTML report against actual local project directories.
Flag inaccuracies, undercounting, and sycophantic language, then patch the report in place.

This audit is local-filesystem-only by design — it never shells out to `gh` or any network
call. GitHub state is not ground truth for this check; the workspace on disk is.

## Workflow

### 1. Locate the report

Default path: `~/.claude/usage-data/report.html`

If the user supplies a different path, use that instead. If the file does not exist, stop
and tell the user.

### 2. Collect ground truth

Local evidence only:

```bash
# Repos under the dev workspace
ls $HOME/dev | sort

# Claude Code session directories (proves session activity, not authorship)
ls $HOME/.claude/projects/ | sort
```

### 3. Extract report claims

Read the report HTML and extract:

- **Project areas** — names and session counts from `.project-area` elements or the "What
  You Work On" section
- **Tools and repos mentioned as authored** — any repo name, crate name, or tool mentioned
  as "built", "created", "your", or attributed to the user
- **Session counts and statistics** — numbers in the narrative and at-a-glance sections
- **Superlative/sycophantic language** — phrases like "impressive", "sophisticated",
  "superpower", "remarkably", "power user", "genuinely", "remarkably efficient"

### 4. Validate each claim

For each tool or repo the report attributes to the user:

1. Check if a directory exists under `$HOME/dev/<name>` with a `.git` → **confirmed local repo**
2. If not a top-level repo, search for it nested under a workspace's `crates/` (or
   equivalent) directory before giving up — see the `insightx` example in Notes
3. If no repo exists but a `~/.claude/projects/<name>` directory does → **local-only, session
   activity but no committed repo** (worth noting, not necessarily wrong)
4. If it's clearly third-party (e.g. installed via `cargo install`, `npm install`,
   `brew install`, or a well-known open-source tool not owned by the user): flag as
   **misattributed**

Check for undercounted projects: look for `$HOME/dev/` or `~/.claude/projects/` entries
that represent significant work (multiple directories with the same prefix, e.g.
`maestro`, `maestro-ao`, `maestro-dev`) but are collapsed into a single report area or
absent entirely.

### 5. Check for sycophancy

Scan the report text for:

- Superlatives: "impressive", "sophisticated", "remarkable", "superpower", "power user",
  "genuinely", "beautifully", "industrial-scale"
- Second-person flattery: "You've built an impressive...", "Your workflow is remarkably..."
- Inflated framing: "autonomous execution engine", "high-throughput contributor"

For each hit, draft a plain-language replacement that states the same fact without the
inflation.

### 6. Produce audit report

Print a structured audit to stdout:

```
## Insights Audit

### Misattributed tools
- <tool>: report says authored, but <evidence it's third-party>

### Undercounted projects
- <project-family>: N directories in ~/.claude/projects/, collapsed to M in report

### Missing projects
- <repo>: confirmed local project, not surfaced in any report area

### Sycophantic language
- "<original phrase>" → "<plain replacement>"

### Confirmed accurate
- <item>: <evidence>
```

### 7. Patch the report (if user confirms)

If the user says "patch it", "fix it", "apply", or similar:

1. Add or update a `<h2 id="section-validation">` section at the bottom of the report
   with the full audit findings as styled HTML, matching the existing report card style.
2. Replace sycophantic phrases inline throughout the report with the plain alternatives.
3. Fix any misattributed tool descriptions in-place.
4. Add a nav link for the validation section if a nav TOC exists.

Do not patch without confirmation.

## Notes

- A local clone proves project activity, not authorship. Do not infer ownership without explicit
  repository metadata or user confirmation.
- A `~/.claude/projects/<name>` directory proves session activity but not authorship.
- Absence of a top-level `$HOME/dev/<name>` repo does not disprove authorship — it could be
  nested inside another workspace's `crates/` directory. Always search before marking missing.
  Example: `insightx` is `checkup/crates/insightx/`, not a top-level repo.
- Never invoke `gh` or make any network call from this skill. If the user wants GitHub-side
  verification (public/private status, remote-only repos), that's a separate, explicit ask —
  not part of this audit.
- Do not remove the validation section if it already exists — append or update it.
