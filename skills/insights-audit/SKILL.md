---
name: insights-audit
description:
  Use when the user runs "/insights-audit", asks to "validate the insights report", "audit
  the report against my projects", "check what the report got wrong", or wants to
  cross-check an /insights-generated HTML report against actual GitHub repos and local
  project directories.
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

Cross-check an `/insights`-generated HTML report against actual GitHub repos and local
`~/.claude/projects/` directories. Flag inaccuracies, undercounting, and sycophantic
language, then patch the report in place.

## Workflow

### 1. Locate the report

Default path: `~/.claude/usage-data/report.html`

If the user supplies a different path, use that instead. If the file does not exist, stop
and tell the user.

### 2. Collect ground truth

First, determine the GitHub username:

1. `gh api user --jq .login`
2. Fallback: `git config --global user.name`

Then run in parallel:

```bash
# GitHub repos (substitute discovered username for GH_USER)
GH_USER=$(gh api user --jq .login)
gh repo list "$GH_USER" --limit 100 --json name,isPrivate \
  --jq '.[] | [.name, (if .isPrivate then "private" else "public" end)] | @tsv'

# Local project dirs
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

1. Check if it exists in the GitHub repo list → **confirmed public**, **confirmed private**,
   or **not on GitHub**
2. For tools not on GitHub: check if a `~/.claude/projects/<name>` directory exists →
   **local only**
3. For tools that are clearly third-party (e.g. installed via `cargo install`, `npm install`,
   `brew install`): flag as **misattributed**

Check for undercounted projects: look for GitHub repos or `~/.claude/projects/` entries
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
- <repo>: on GitHub, not surfaced in any report area

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

- `rustqual`, `cargo-nextest`, `cargo-deny`, `cargo-machete` and similar tools installed
  via `cargo install` are third-party unless a GitHub repo exists under the user's account.
- A `~/.claude/projects/<name>` directory proves session activity but not authorship.
- Private repos on GitHub confirm authorship; absence from GitHub does not disprove it
  (could be local-only, pre-published, or a crate nested inside another workspace).
- For tools not found as standalone repos, search workspace `crates/` directories before
  marking as missing. Example: `insightx` is `checkup/crates/insightx/`, not a top-level repo.
- Do not remove the validation section if it already exists — append or update it.
