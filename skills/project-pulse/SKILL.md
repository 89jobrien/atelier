---
name: project-pulse
description:
  This skill should be used when the user asks to "end session", "capture state",
  "session summary", "what changed this session", "summarize repos", "write session notes",
  or wants to capture multi-repo state at the end of a work session.
model: sonnet
effort: medium
allowed-tools:
  - Read
  - Write
  - Bash
---

# project-pulse

Capture branch/commit/PR state across all active repos at session end. Write to both
memory files and the Obsidian daily note.

See `references/active-repos.md` for discovery method, output paths, and session diff
format.

## Repo Discovery

Scan all subdirectories of `$HOME/dev/` that contain a `.git/` directory. By default,
only include repos that have a GitHub remote (public). Repos with only a Gitea
remote are private/employer projects and excluded unless the user explicitly asks
for all repos.

```bash
for dir in $HOME/dev/*/; do
  [ -d "$dir/.git" ] || continue
  # Check for a github.com remote (origin or github)
  git -C "$dir" remote -v 2>/dev/null | grep -q 'github\.com' && basename "$dir"
done
```

To include all repos (public + private), the user can say "pulse all" or
"include private repos".

## State Capture Per Repo

Run for each discovered repo. Skip directories where `git -C <path> status` fails.

### Core state (always collected)

```bash
git -C ~/dev/<repo> branch --show-current
git -C ~/dev/<repo> log --oneline -5
git -C ~/dev/<repo> status --short
gh -C ~/dev/<repo> pr list --state open --limit 3 2>/dev/null || true
```

### Extended context (24h and 7d activity)

Collect these for every repo to provide historical context beyond the current session:

```bash
# Last commit age (human-readable + ISO)
git -C ~/dev/<repo> log -1 --format="%ar (%ai)"

# Commits in the last 24 hours (all branches)
git -C ~/dev/<repo> log --oneline --since="24 hours ago" --all

# Commits in the last 7 days (all branches, count only)
git -C ~/dev/<repo> log --oneline --since="7 days ago" --all | wc -l

# Recent tags
git -C ~/dev/<repo> tag --sort=-creatordate | head -3

# Stash count
git -C ~/dev/<repo> stash list | wc -l

# Worktree count (>1 means active worktrees beyond the main checkout)
git -C ~/dev/<repo> worktree list | wc -l

# All local branches (cap at 10)
git -C ~/dev/<repo> branch -a | head -10
```

### Session commits

To count commits made this session, use the earliest log entry from the current
session as a boundary. If session start time is unknown, use last 4 hours:

```bash
git -C ~/dev/<repo> log --oneline --since="4 hours ago"
```

## Writing to Memory

Write session state to the project memory file:

```
~/.claude/projects/-Users-joe-dev-<repo>/memory/session_YYYY-MM-DD.md
```

Format:

```markdown
---
name: session-2026-04-03
description: Session state snapshot for 2026-04-03
type: project
---

## Session State — 2026-04-03

**Branch:** feat/gc-images
**Last commit:** abc1234 fix: clean up GC loop (3 days ago)
**Latest tag:** v0.31.0

### Activity

| Window   | Commits |
| -------- | ------- |
| Session  | 3       |
| 24 hours | 5       |
| 7 days   | 28      |

**Stashes:** 4 | **Worktrees:** 2 | **Local branches:** 8

**Open PRs:**

- #42 feat: image GC (draft)

**Uncommitted changes:** none
```

Only write memory files for repos where something actually happened — commits
(session, 24h, or 7d), open PRs, uncommitted changes, or notable stash/worktree
counts. Skip repos that are completely dormant (zero activity across all windows,
clean tree, no stashes, single worktree).

## Writing to Obsidian

Append under `## Session Pulse` in today's daily note:

```
$HOME/Documents/Obsidian Vault/Daily Notes/YYYY-MM-DD.md
```

```markdown
## Session Pulse

| Repo    | Branch         | Session | 24h | 7d  | Last Commit | Stashes | Status |
| ------- | -------------- | ------- | --- | --- | ----------- | ------- | ------ |
| minibox | feat/gc-images | +3      | +5  | 113 | 3d ago      | 12      | clean  |
| doob    | main           | 0       | +3  | 3   | 15h ago     | 4       | clean  |
| devloop | main           | 0       | 0   | 0   | 4w ago      | 3       | clean  |
```

If the daily note doesn't exist, create it with the pulse section only — do not add
other content or headers beyond `# YYYY-MM-DD`.

If a `## Session Pulse` section already exists in the daily note (from an earlier pulse
run today), replace it entirely rather than appending a second table.

Sort rows by activity: repos with session commits first, then 24h commits, then 7d
commits, then by last commit recency. This puts the most active repos at the top.

## Repos with No Activity

Include repos with zero commits across all windows in the table — this confirms the
repo was checked, not skipped. Omit repos entirely only if the git command fails.

## Using the herald Agent

For narrative synthesis, invoke the `herald` agent after pulse:

> "Synthesize today's session into the Obsidian daily note"

`herald` produces prose narrative; `project-pulse` produces structured state tables.
Run pulse first to generate the table, then herald to wrap it in context.

## Pairing with handoff

After `project-pulse`, run `handoff` to write `HANDOFF.yaml` with actionable next steps.
Pulse captures what happened; handoff captures what's next.

## Additional Resources

- **`references/active-repos.md`** — discovery method, session diff format, output paths.
