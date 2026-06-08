# Active Repos

## Discovery

Scan all subdirectories of `$HOME/dev/` that contain a `.git/` directory. By default,
only include repos with a `github.com` remote (public). Repos with only Gitea
remotes are private/employer projects and excluded unless explicitly requested.

```bash
for dir in $HOME/dev/*/; do
  [ -d "$dir/.git" ] || continue
  git -C "$dir" remote -v 2>/dev/null | grep -q 'github\.com' && basename "$dir"
done
```

To include all repos, the user says "pulse all" or "include private repos".

## State Capture Per Repo

```bash
git branch --show-current          # current branch
git log --oneline -5               # last 5 commits
git log -1 --format="%ar (%ai)"   # last commit age
git log --oneline --since="24 hours ago" --all  # 24h activity
git log --oneline --since="7 days ago" --all | wc -l  # 7d count
git status --short                 # uncommitted changes
git stash list | wc -l             # stash count
git worktree list | wc -l          # worktree count
git tag --sort=-creatordate | head -3  # recent tags
gh pr list --state open --limit 3  # open PRs (if gh available)
```

## Session Diff Format

```
REPO     BRANCH         SESSION  24H  7D   LAST COMMIT  STASHES  STATUS
minibox  feat/gc-images +3       +5   113  3d ago       12       clean
doob     main           0        +3   3    15h ago      4        clean
devloop  main           0        0    0    4w ago       3        clean (7 wt)
```

## Output Paths

| Destination         | Path                                                                    |
| ------------------- | ----------------------------------------------------------------------- |
| Memory file         | `~/.claude/projects/-Users-joe-dev-<repo>/memory/session_YYYY-MM-DD.md` |
| Obsidian daily note | `~/Documents/Obsidian Vault/Daily Notes/YYYY-MM-DD.md`                  |

Append under a `## Session Pulse` heading in the daily note. Create the note if absent.
