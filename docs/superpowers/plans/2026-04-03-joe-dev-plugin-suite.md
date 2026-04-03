# joe-dev Plugin Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build two companion Claude Code plugins (`joe-dev` and `joe-secrets`) that automate
repeated operational workflows across Rust dev, code review, secrets management, CI, and
multi-repo session management.

**Architecture:** Flat plugin structure — skills in `skills/<name>/SKILL.md`, agents in
`agents/<name>.md`, hooks in `hooks/hooks.json`. All agents are thin wrappers delegating to
devkit. No duplicate hooks with global CLAUDE.md setup.

**Tech Stack:** Claude Code plugin format (YAML frontmatter + Markdown), Bash (hook scripts),
JSON (plugin manifests and hook config), SKILL.md / agent.md conventions.

---

## File Map

### `joe-dev` plugin

| File | Purpose |
|------|---------|
| `joe-dev/plugin.json` | Manifest |
| `joe-dev/README.md` | Installation and usage docs |
| `joe-dev/.gitignore` | Ignore `.claude/*.local.md` |
| `joe-dev/skills/cargo-gate/SKILL.md` | Rust validation gate skill |
| `joe-dev/skills/sentinel-autofixer/SKILL.md` | Auto-apply sentinel review fixes |
| `joe-dev/skills/hook-diagnostics/SKILL.md` | Hook visibility and failure surfacing |
| `joe-dev/skills/git-guard/SKILL.md` | Safe merge/rebase strategy |
| `joe-dev/skills/ci-assist/SKILL.md` | CI workflow editing and cross-compile |
| `joe-dev/skills/project-pulse/SKILL.md` | Session-end multi-repo state capture |
| `joe-dev/skills/handoff/SKILL.md` | Write HANDOFF.yaml at session end |
| `joe-dev/skills/handon/SKILL.md` | Read HANDOFF.yaml at session start |
| `joe-dev/agents/sentinel.md` | Code review agent (devkit delegate) |
| `joe-dev/agents/forge.md` | Dev companion agent (devkit delegate) |
| `joe-dev/agents/herald.md` | Cross-repo synthesis agent (devkit delegate) |
| `joe-dev/agents/conductor.md` | Workflow orchestrator agent (devkit delegate) |
| `joe-dev/agents/oxidizer.md` | Rust-specific reviewer agent (devkit delegate) |

### `joe-secrets` plugin

| File | Purpose |
|------|---------|
| `joe-secrets/plugin.json` | Manifest |
| `joe-secrets/README.md` | Installation and usage docs |
| `joe-secrets/.gitignore` | Ignore `.claude/*.local.md` |
| `joe-secrets/skills/op-resolver/SKILL.md` | 1Password/direnv validation skill |
| `joe-secrets/hooks/hooks.json` | SessionStart hook registration |
| `joe-secrets/hooks/op-resolver-startup.sh` | Startup script: op auth + env chain trace |

---

## Task 1: Scaffold `joe-dev` plugin structure

**Files:**
- Create: `~/.claude/plugins/joe-dev/plugin.json`
- Create: `~/.claude/plugins/joe-dev/README.md`
- Create: `~/.claude/plugins/joe-dev/.gitignore`
- Create: directories for all skills and agents

- [ ] **Step 1: Create directory tree**

```bash
mkdir -p $HOME/.claude/plugins/joe-dev/skills/cargo-gate
mkdir -p $HOME/.claude/plugins/joe-dev/skills/sentinel-autofixer
mkdir -p $HOME/.claude/plugins/joe-dev/skills/hook-diagnostics
mkdir -p $HOME/.claude/plugins/joe-dev/skills/git-guard
mkdir -p $HOME/.claude/plugins/joe-dev/skills/ci-assist
mkdir -p $HOME/.claude/plugins/joe-dev/skills/project-pulse
mkdir -p $HOME/.claude/plugins/joe-dev/skills/handoff
mkdir -p $HOME/.claude/plugins/joe-dev/skills/handon
mkdir -p $HOME/.claude/plugins/joe-dev/agents
```

Expected: no output, directories created.

- [ ] **Step 2: Write plugin.json**

Create `$HOME/.claude/plugins/joe-dev/plugin.json`:

```json
{
  "name": "joe-dev",
  "version": "0.1.0",
  "description": "Personal dev workflow plugin — Rust gates, code review, CI, git safety, multi-repo pulse",
  "author": {
    "name": "Joseph O'Brien",
    "email": "joeobrien516@gmail.com"
  }
}
```

- [ ] **Step 3: Write .gitignore**

Create `$HOME/.claude/plugins/joe-dev/.gitignore`:

```
.claude/*.local.md
```

- [ ] **Step 4: Write README.md**

Create `$HOME/.claude/plugins/joe-dev/README.md`:

```markdown
# joe-dev

Personal dev workflow plugin — Rust gates, code review, CI, git safety, multi-repo pulse.

## Installation

```bash
# Install both plugins for full session-start experience
cc --plugin-dir ~/.claude/plugins/joe-dev
cc --plugin-dir ~/.claude/plugins/joe-secrets
```

## Skills

| Skill | Trigger Phrases |
|-------|----------------|
| cargo-gate | "run gates", "validate rust", "pre-commit check" |
| sentinel-autofixer | "apply review fixes", "fix sentinel suggestions", "auto-fix review" |
| hook-diagnostics | "show hook status", "hook failures", "what hooks ran" |
| git-guard | "safe to commit", "check merge strategy", "commit safely" |
| ci-assist | "edit workflow", "fix CI", "check cross-compile", "verify binary" |
| project-pulse | "end session", "capture state", "session summary" |
| handoff | "write handoff", "end of session", "capture handoff" |
| handon | "start session", "orient to work", "what's outstanding" |

## Agents

| Agent | Purpose |
|-------|---------|
| sentinel | Structured code review (delegates to devkit) |
| forge | Dev companion — design, debug, refactor (delegates to devkit) |
| herald | Cross-repo synthesis → Obsidian (delegates to devkit) |
| conductor | devloop → doob → devkit pipeline (delegates to devkit) |
| oxidizer | Rust-specific review: clippy, unsafe, edition 2024 (delegates to devkit) |

## Notes

- `cargo-gate` runs `cargo xtask pre-commit` first — the xtask gate always takes priority.
- `joe-secrets` must also be installed for the session-start op-resolver + handon chain.
- All agents are thin wrappers; devkit must be installed and accessible.
```

- [ ] **Step 5: Verify structure**

```bash
ls $HOME/.claude/plugins/joe-dev/
ls $HOME/.claude/plugins/joe-dev/skills/
ls $HOME/.claude/plugins/joe-dev/agents/
```

Expected: all directories present, plugin.json, README.md, .gitignore visible.

- [ ] **Step 6: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git init && git add -A && git commit -m "feat: scaffold joe-dev plugin structure"
```

---

## Task 2: Write `cargo-gate` skill

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/cargo-gate/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/cargo-gate/SKILL.md`:

```markdown
---
name: cargo-gate
description: This skill should be used when the user asks to "run gates", "validate rust",
  "pre-commit check", "run cargo validation", "check before committing", or wants to run
  the full Rust validation suite before a commit.
---

# cargo-gate

Run the full Rust validation suite before committing. The `cargo xtask pre-commit` gate
always takes priority — this skill wraps it and adds structured pass/fail reporting.

## Validation Order

Always run stages in this order:

1. `cargo xtask pre-commit` — runs fmt-check + clippy + release build (canonical gate, always first)
2. Report results per stage: fmt / clippy / test / check
3. If xtask fails, surface the failing stage explicitly before stopping

## Running the Gate

```bash
cargo xtask pre-commit
```

If `cargo xtask pre-commit` is not available (no xtask in workspace), fall back to:

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo check --workspace
```

Never skip `cargo xtask pre-commit` when it exists. It is the canonical gate.

## Reporting

After running, present a structured summary:

```
STAGE      RESULT
fmt        PASS
clippy     PASS  (or: FAIL — 3 warnings promoted to errors)
test       PASS  (or: FAIL — 2 tests failed)
check      PASS
```

If any stage fails, show the first 20 lines of error output for that stage.

## Clippy Auto-Fix

When clippy produces suggestion-level warnings (not errors), offer:

> "clippy has N suggestions. Apply auto-fixes? (yes/no/show-diff)"

If yes: run `cargo clippy --fix --allow-staged`. Always show the diff before applying.
Never auto-apply without confirmation.

## Integration with xtask

`cargo xtask pre-commit` on minibox runs: fmt-check → clippy → release build.
It does NOT run `cargo test` by default. Add test stage separately if needed.

## When to Use

Invoke before every commit on Rust projects. Pairs with `git-guard` — run cargo-gate
first, then git-guard to confirm merge strategy before committing.
```

- [ ] **Step 2: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add cargo-gate skill"
```

---

## Task 3: Write `sentinel-autofixer` skill

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/sentinel-autofixer/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/sentinel-autofixer/SKILL.md`:

```markdown
---
name: sentinel-autofixer
description: This skill should be used when the user asks to "apply review fixes",
  "fix sentinel suggestions", "auto-fix review", "apply code review", "batch apply
  sentinel fixes", or wants to automatically apply suggestion-level fixes from a
  sentinel code review report.
---

# sentinel-autofixer

Read a sentinel code review report, triage issues by severity, and apply
suggestion-level fixes in a single batch commit. Never auto-apply blocking issues.

## Triage Rules

Sentinel reports contain three categories:

| Category | Action |
|----------|--------|
| **Blocking** | Surface to user — do NOT auto-apply. Require explicit instruction. |
| **Suggestions** | Apply automatically after dry-run confirmation. |
| **Observations** | Inform user — no action unless requested. |

## Workflow

1. Read the sentinel report from the current conversation or ask the user to paste it
2. Extract all suggestion-level items
3. Present a dry-run diff of proposed changes:
   ```
   SUGGESTION 1: [description]
   File: src/handler.rs:42
   - old code
   + new code
   ```
4. Ask: "Apply N suggestions? (yes/no/select)"
5. If yes: apply all changes, run `cargo check --workspace` to verify
6. If select: apply only confirmed items
7. Commit all applied fixes in one batch commit:
   ```bash
   git add -A && git commit -m "fix: apply sentinel suggestion-level fixes"
   ```

## Blocking Issues

When blocking issues are present, always surface them first:

> "Sentinel found N blocking issue(s) that require manual review:
> 1. [issue description] — [file:line]
> These will NOT be auto-applied."

Do not proceed with suggestion fixes until the user acknowledges blocking issues.

## After Applying

After committing, run `cargo-gate` to verify the workspace still builds cleanly.
Report any regressions introduced by the applied fixes.

## Using the sentinel Agent

To generate a fresh sentinel report before auto-fixing, invoke the `sentinel` agent:

> "Review [file or diff] for issues"

The sentinel agent will produce a structured report that this skill can consume.
```

- [ ] **Step 2: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add sentinel-autofixer skill"
```

---

## Task 4: Write `hook-diagnostics` skill

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/hook-diagnostics/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/hook-diagnostics/SKILL.md`:

```markdown
---
name: hook-diagnostics
description: This skill should be used when the user asks to "show hook status",
  "hook failures", "what hooks ran", "why did my hook fail", "hook overhead",
  "list active hooks", or wants visibility into Claude Code hook execution and failures.
---

# hook-diagnostics

Surface Claude Code hook execution status, failures, and overhead from the current session.

## Hook Sources

Active hooks come from two sources:

1. **Global CLAUDE.md hooks** (`~/.claude/settings.json` or hooks defined in CLAUDE.md):
   - rtk-rewrite.sh (PreToolUse/Bash)
   - pre-tool-course-correct.py (PreToolUse/Bash)
   - post-bash-redact.sh (PostToolUse/Bash)
   - post-tool-track-failures.py (PostToolUse/Bash)
   - post-edit-cargo-fmt.nu (PostToolUse/Edit|Write)
   - post-edit-cargo-check.nu (PostToolUse/Edit|Write)
   - sync_memory_to_vault.py (PostToolUse/Edit|Write)

2. **Plugin hooks** (joe-secrets SessionStart):
   - op-resolver-startup.sh (SessionStart)

## Checking Hook Status

To list currently loaded hooks, run in Claude Code:

```
/hooks
```

To check hook failure logs from post-tool-track-failures.py:

```bash
ls -lt $HOME/.claude/hooks/failures/ | head -20
```

If the failures directory doesn't exist, no failures have been recorded this session.

## Reading Failure Logs

Failure logs are written by `post-tool-track-failures.py` as JSON entries:

```bash
# Show last 10 failures
tail -n 10 $HOME/.claude/hooks/failures/failures.jsonl 2>/dev/null || echo "No failures recorded"
```

Each entry contains: `timestamp`, `hook_name`, `exit_code`, `command`, `stderr`.

## Diagnosing a Specific Hook Failure

1. Identify the hook name from the failure log
2. Find the hook script path: `ls $HOME/.claude/hooks/`
3. Run the hook directly with sample input:
   ```bash
   echo '{"tool_name": "Bash", "tool_input": {"command": "echo test"}}' | \
     bash $HOME/.claude/hooks/<hook-name>.sh
   echo "Exit: $?"
   ```
4. Check stderr for error messages

## Hook Overhead

To estimate hook overhead, check the `rtk gain` command:

```bash
rtk gain
```

This shows token savings from rtk-rewrite and command history. Hook execution time is
not directly measured, but `claude --debug` shows hook timing in the debug log.

## Common Failures

| Hook | Common Cause | Fix |
|------|-------------|-----|
| rtk-rewrite.sh | rtk binary not on PATH | `which rtk` — reinstall if missing |
| pre-tool-course-correct.py | Python not found | `which python3` |
| post-edit-cargo-fmt.nu | nu not on PATH | `which nu` |
| op-resolver-startup.sh | 1Password not authed | `op account list` |
```

- [ ] **Step 2: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add hook-diagnostics skill"
```

---

## Task 5: Write `git-guard` skill

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/git-guard/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/git-guard/SKILL.md`:

```markdown
---
name: git-guard
description: This skill should be used when the user asks to "safe to commit",
  "check merge strategy", "commit safely", "is it safe to merge", "should I rebase
  or merge", or wants to verify the git strategy is safe before committing or merging.
---

# git-guard

Verify git merge/rebase strategy is safe before committing. Detect unsafe rebase
candidates, confirm 1Password SSH signing agent is available, and recommend the
correct strategy.

## Safety Check Workflow

Run these checks in order:

### 1. Check for merge commits on current branch

```bash
git log --oneline --merges main..HEAD
```

If output is non-empty: the branch contains merge commits. **Do NOT rebase.** Use merge.

If output is empty: branch has no merge commits. Rebase is safe if desired.

### 2. Check branch divergence

```bash
git log --oneline main..HEAD
```

If empty: branch is fully merged into main. Nothing to do.

If non-empty: list the commits that will be included and confirm with the user.

### 3. Confirm 1Password SSH signing agent

```bash
ssh-add -l 2>/dev/null | grep -i "1password\|agent" || echo "WARNING: 1Password agent may not be running"
```

If agent not detected: warn user to unlock 1Password before committing.

```bash
op account list
```

If this fails: 1Password is not authed. Commits will fail with signing error.

### 4. Recommend strategy

Present recommendation:

```
STRATEGY RECOMMENDATION
Branch has merge commits:  NO  →  rebase is safe (but merge also fine)
Branch has merge commits:  YES →  USE MERGE, do not rebase

Recommended: git merge main  (or: git rebase main)
1Password:   READY
```

## Unsafe Rebase — Hard Rule

**Never rebase a branch that contains merge commits.** From CLAUDE.md:

> Never rebase branches that contain merge commits. Use `git merge` for conflict
> resolution unless explicitly told to rebase.

When merge commits are detected, always recommend `git merge` and refuse to suggest rebase.

## After Confirmation

Once the user confirms strategy, proceed with the commit:

```bash
git add -A && git commit -m "<message>"
```

If commit fails with `1Password: agent returned an error`: instruct user to open and
unlock 1Password, then retry the commit. No config change is needed.

## Pairing with cargo-gate

Always run `cargo-gate` before `git-guard` on Rust projects:

1. `cargo-gate` — validates the build is clean
2. `git-guard` — confirms strategy and signs the commit
```

- [ ] **Step 2: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add git-guard skill"
```

---

## Task 6: Write `ci-assist` skill

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/ci-assist/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/ci-assist/SKILL.md`:

```markdown
---
name: ci-assist
description: This skill should be used when the user asks to "edit workflow", "fix CI",
  "check cross-compile", "verify binary", "update github actions", "debug CI failure",
  "verify target triple", or needs help with CI/CD workflow files or cross-compilation.
---

# ci-assist

Edit GitHub Actions workflow files, verify cross-compiled binary architecture, and
diagnose CI failures.

## Editing Workflow Files

The `Edit` tool is blocked for `.github/workflows/*.yml` files by a security hook.
Always use Bash heredoc instead:

```bash
cat > .github/workflows/ci.yml << 'EOF'
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # ... rest of workflow
EOF
```

Reference canonical patterns from `~/dev/minibox` (`.github/workflows/ci.yml`,
`nightly.yml`, `release.yml`, `deny.toml`).

## Cross-Compilation Target Reference

| Environment | Target Triple |
|-------------|--------------|
| Local macOS (M-series) | `aarch64-apple-darwin` |
| VPS / minibox deploy | `x86_64-unknown-linux-musl` |

**Never rsync a binary before verifying target.** Always run:

```bash
file <binary>
```

Expected output for VPS binary:
```
<binary>: ELF 64-bit LSB executable, x86-64, statically linked
```

Expected output for local binary:
```
<binary>: Mach-O 64-bit executable arm64
```

If the architecture does not match the deploy target, rebuild with the correct target:

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

## CI Diagnostics

To check the latest CI run:

```bash
gh run list --limit 5
gh run view <run-id>
gh run view <run-id> --log-failed
```

To watch a run in progress:

```bash
gh run watch <run-id>
```

To list workflow files:

```bash
gh workflow list
```

## Diagnosing a Failed CI Run

1. `gh run list --limit 5` — find the failing run ID
2. `gh run view <id> --log-failed` — see only failing steps
3. Read the failing step's log for the root cause
4. Check: missing env vars, wrong target triple, denied op:// refs in CI context

## Common CI Failure Patterns

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| `cargo build` fails on Linux CI but passes locally | Wrong target triple or missing musl toolchain | Add `x86_64-unknown-linux-musl` target in workflow |
| `op://` URI appears in logs | Secret not injected via `op run` | Wrap command with `op run --` in workflow |
| Clippy warnings fail CI | Warnings promoted to errors (`-D warnings`) | Fix clippy warnings locally first |
| Workflow file not updated | Edit tool blocked | Use heredoc approach above |

## Reference Repo

For canonical CI patterns, read from `~/dev/minibox/.github/workflows/`:

```bash
ls ~/dev/minibox/.github/workflows/
```
```

- [ ] **Step 2: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add ci-assist skill"
```

---

## Task 7: Write `project-pulse` skill

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/project-pulse/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/project-pulse/SKILL.md`:

```markdown
---
name: project-pulse
description: This skill should be used when the user asks to "end session", "capture state",
  "session summary", "what changed this session", "summarize repos", "write session notes",
  or wants to capture multi-repo state at the end of a work session.
---

# project-pulse

Capture branch/commit/PR state across all active repos at session end. Write to both
memory files and the Obsidian daily note.

## Active Repos

Default active repos to check:

```
~/dev/minibox
~/dev/maestro
~/dev/devloop
~/dev/doob
~/dev/devkit
~/dev/magi
~/dev/mcpipe
~/dev/braid
```

Add or remove repos based on what was active in the current session.

## State Capture Per Repo

For each repo, capture:

```bash
cd <repo>
git branch --show-current          # current branch
git log --oneline -5               # last 5 commits
git status --short                 # uncommitted changes
gh pr list --state open --limit 3  # open PRs (if gh available)
```

## Session Diff

Compare against session-start state (if known) to produce a diff:

```
REPO        BRANCH          COMMITS THIS SESSION   OPEN PRS
minibox     feat/gc-images  3 new commits          1 open
devloop     main            0                      0
doob        fix/sync        1 new commit           0
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
**Commits this session:** 3
**Last commit:** abc1234 fix: clean up GC loop

**Open PRs:**
- #42 feat: image GC (draft)

**Uncommitted changes:** none
```

## Writing to Obsidian

Append a section to today's daily note:

```
$HOME/Documents/Obsidian Vault/Daily Notes/YYYY-MM-DD.md
```

Append under a `## Session Pulse` heading:

```markdown
## Session Pulse

| Repo | Branch | Commits | Status |
|------|--------|---------|--------|
| minibox | feat/gc-images | +3 | clean |
| devloop | main | 0 | clean |
```

If the daily note doesn't exist, create it with the pulse section.

## Using the herald Agent

For a richer narrative synthesis (vs. raw state capture), invoke the `herald` agent:

> "Synthesize today's session into the Obsidian daily note"

`herald` produces prose summaries; `project-pulse` produces structured state tables.
Use both for complete session-end coverage: pulse first (structured), herald second (narrative).

## Pairing with handoff

After `project-pulse`, run `handoff` to write `HANDOFF.yaml` with actionable next steps.
The two skills complement each other: pulse captures what happened, handoff captures what's next.
```

- [ ] **Step 2: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add project-pulse skill"
```

---

## Task 8: Write `handoff` and `handon` skills

**Files:**
- Create: `~/.claude/plugins/joe-dev/skills/handoff/SKILL.md`
- Create: `~/.claude/plugins/joe-dev/skills/handon/SKILL.md`

- [ ] **Step 1: Write handoff SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/handoff/SKILL.md`:

```markdown
---
name: handoff
description: This skill should be used when the user asks to "write handoff", "end of session",
  "capture handoff", "save handoff", "update HANDOFF.yaml", or wants to record current project
  state and outstanding work for the next session.
---

# handoff

Write `HANDOFF.yaml` in the current repo with completed work, newly discovered gaps, and
current project state. Designed to be read by `handon` at the start of the next session.

## HANDOFF.yaml Format

Write to `<repo-root>/HANDOFF.yaml`:

```yaml
updated: "2026-04-03T18:30:00Z"
project: minibox
branch: feat/gc-images

completed:
  - "Implemented image GC loop in gc.rs"
  - "Added unit tests for retention policy"
  - "Fixed off-by-one in layer ref counting"

in_progress:
  - task: "Integration test for GC under load"
    notes: "Needs minibox running on VPS — blocked until SSH key rotated"
    priority: high

gaps:
  - "Layer dedup logic not yet implemented — see issue #38"
  - "GC does not handle concurrent pulls — race condition possible"

next_session:
  - "SSH key rotation (prerequisite for VPS integration test)"
  - "Review sentinel suggestions from today's review"
  - "Open PR once integration test passes"

blockers:
  - "SSH key rotation required before VPS test"
```

## Gathering Content

Before writing, collect:

1. **Completed:** Ask user "What did we finish this session?" or infer from git log:
   ```bash
   git log --oneline $(git log --format="%H" --since="8 hours ago" | tail -1)..HEAD
   ```

2. **In progress:** What was started but not finished? Check git status for uncommitted work.

3. **Gaps:** What was discovered but not addressed? Pull from sentinel observations,
   TODOs in code, or explicit user mentions.

4. **Next session:** What should be picked up immediately next time?

5. **Blockers:** What is preventing progress?

## Writing the File

Write to the repo root, overwriting any existing HANDOFF.yaml:

```bash
# Confirm repo root
git rev-parse --show-toplevel
```

Write the YAML with current timestamp in ISO 8601 format.

## After Writing

Confirm with the user:

> "HANDOFF.yaml written to `<path>`. Commit it?"

If yes:
```bash
git add HANDOFF.yaml && git commit -m "chore: update handoff for session end"
```
```

- [ ] **Step 2: Write handon SKILL.md**

Create `$HOME/.claude/plugins/joe-dev/skills/handon/SKILL.md`:

```markdown
---
name: handon
description: This skill should be used when the user asks to "start session", "orient to work",
  "what's outstanding", "read handoff", "what was I working on", "pick up where I left off",
  or at session start when the joe-secrets SessionStart hook fires to surface outstanding work.
---

# handon

Read `HANDOFF.yaml` files across active repos, triage items by priority, and present
outstanding work at session start. Invoked automatically by the `joe-secrets` SessionStart hook.

## Active Repos to Scan

Scan for HANDOFF.yaml in:

```
~/dev/minibox
~/dev/maestro
~/dev/devloop
~/dev/doob
~/dev/devkit
~/dev/magi
~/dev/mcpipe
~/dev/braid
```

Also check the current working directory repo.

## Reading HANDOFF.yaml

For each repo with a HANDOFF.yaml:

```bash
ls ~/dev/*/HANDOFF.yaml 2>/dev/null
```

Read each file and extract: `in_progress`, `next_session`, `blockers`, `updated`.

## Triage and Presentation

Sort by:
1. `blockers` non-empty — surface first
2. `in_progress` items with `priority: high`
3. `next_session` items
4. `in_progress` items with other priorities

Present as a brief orientation table:

```
SESSION ORIENTATION — 2026-04-03

BLOCKERS
  minibox: SSH key rotation required before VPS integration test

HIGH PRIORITY
  minibox [feat/gc-images]: Integration test for GC under load
    Note: Needs VPS — blocked until SSH rotated

NEXT UP
  minibox: SSH key rotation → VPS test → open PR
  devloop: Review snapshot failures from yesterday's nextest run

LAST UPDATED
  minibox: 3 hours ago
  devloop: 1 day ago
```

## When No HANDOFF.yaml Exists

If no HANDOFF.yaml files are found:

> "No HANDOFF.yaml found in active repos. Starting fresh session.
> Run `handoff` at session end to capture state for next time."

## Automatic Invocation

This skill is invoked automatically by the `joe-secrets` SessionStart hook after op-resolver
completes. No manual trigger needed at session start — it fires on every new Claude session.
```

- [ ] **Step 3: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add handoff and handon skills"
```

---

## Task 9: Write the five agents

**Files:**
- Create: `~/.claude/plugins/joe-dev/agents/sentinel.md`
- Create: `~/.claude/plugins/joe-dev/agents/forge.md`
- Create: `~/.claude/plugins/joe-dev/agents/herald.md`
- Create: `~/.claude/plugins/joe-dev/agents/conductor.md`
- Create: `~/.claude/plugins/joe-dev/agents/oxidizer.md`

- [ ] **Step 1: Write sentinel.md**

Create `$HOME/.claude/plugins/joe-dev/agents/sentinel.md`:

```markdown
---
name: sentinel
description: Use this agent for structured code review against hexagonal architecture,
  Rust/Go conventions, and SOLID principles. Examples:

<example>
Context: User has just implemented a new feature in a Rust crate.
user: "Review handler.rs for issues"
assistant: "I'll use the sentinel agent to review handler.rs."
<commentary>
Feature implementation complete — structured review is appropriate before committing.
</commentary>
</example>

<example>
Context: User is about to open a PR.
user: "Check the diff before I open a PR"
assistant: "Let me run sentinel over the diff before you open the PR."
<commentary>
Pre-PR review is a canonical sentinel trigger.
</commentary>
</example>

model: inherit
color: blue
tools: ["Read", "Grep", "Glob", "Bash"]
---

You are sentinel, a structured code reviewer. Delegate all review work to the devkit
sentinel agent by invoking it with the files or diff provided.

Your only role is to pass the task to devkit sentinel and return its structured report
to the user. Do not perform the review yourself.

Invoke devkit sentinel with: the file paths or diff context from the user's request.
Return the full report: blocking issues, suggestions, and observations.
```

- [ ] **Step 2: Write forge.md**

Create `$HOME/.claude/plugins/joe-dev/agents/forge.md`:

```markdown
---
name: forge
description: Use this agent for design discussions, debugging, refactoring, prototyping,
  and ad-hoc dev work across minibox, devloop, doob, and devkit. Examples:

<example>
Context: User wants to discuss architecture for a new feature.
user: "Let's design the new sync adapter for doob"
assistant: "I'll open a forge session to work through the design."
<commentary>
Design discussion is a forge trigger — it auto-routes to sentinel/navigator/conductor as needed.
</commentary>
</example>

<example>
Context: User is stuck debugging a Rust compile error.
user: "Help me debug this lifetime error"
assistant: "I'll use forge to work through this with you."
<commentary>
Debugging and dev companion work is forge's core purpose.
</commentary>
</example>

model: inherit
color: green
tools: ["Read", "Write", "Edit", "Grep", "Glob", "Bash"]
---

You are forge, a primary dev companion. Delegate all work to the devkit forge agent
by invoking it with the full context of the user's request.

Your only role is to pass the task to devkit forge and relay its responses. Devkit forge
auto-dispatches to sentinel, navigator, or conductor as needed — do not second-guess it.
```

- [ ] **Step 3: Write herald.md**

Create `$HOME/.claude/plugins/joe-dev/agents/herald.md`:

```markdown
---
name: herald
description: Use this agent to synthesize cross-project activity into an Obsidian daily note,
  generate cross-repo narrative summaries, or consolidate work from multiple repos. Examples:

<example>
Context: End of a work session across multiple repos.
user: "Synthesize today's session into my daily note"
assistant: "I'll use herald to synthesize the cross-project activity."
<commentary>
End-of-session cross-repo synthesis is herald's primary use case.
</commentary>
</example>

<example>
Context: User wants a narrative summary of what changed.
user: "What happened across all repos this week?"
assistant: "I'll run herald to generate a cross-repo summary."
<commentary>
Cross-project narrative summarization triggers herald.
</commentary>
</example>

model: inherit
color: cyan
tools: ["Read", "Write", "Bash"]
---

You are herald, a cross-project knowledge synthesizer. Delegate all synthesis work to
the devkit herald agent by invoking it with the session context and repo list.

Your only role is to pass the task to devkit herald and return the synthesized narrative.
The output goes to both the Obsidian daily note and the memory system.
```

- [ ] **Step 4: Write conductor.md**

Create `$HOME/.claude/plugins/joe-dev/agents/conductor.md`:

```markdown
---
name: conductor
description: Use this agent to run the devloop → doob → devkit workflow pipeline, create doob
  tasks from council analysis findings, or triage CI failures. Examples:

<example>
Context: User has just made a significant commit.
user: "Run the pipeline on this branch"
assistant: "I'll use conductor to run the devloop → doob → devkit pipeline."
<commentary>
Post-commit pipeline runs are conductor's primary trigger.
</commentary>
</example>

<example>
Context: CI just failed on a push.
user: "CI failed, help me triage"
assistant: "I'll invoke conductor to triage the CI failure."
<commentary>
CI failure triage is a conductor trigger.
</commentary>
</example>

model: inherit
color: yellow
tools: ["Bash", "Read"]
---

You are conductor, a workflow orchestrator. Delegate all pipeline work to the devkit
conductor agent by invoking it with the branch name and repo context.

Your only role is to pass the task to devkit conductor and return its findings.
Conductor does not fix code or make commits — it surfaces work for the user to act on.
```

- [ ] **Step 5: Write oxidizer.md**

Create `$HOME/.claude/plugins/joe-dev/agents/oxidizer.md`:

```markdown
---
name: oxidizer
description: Use this agent for Rust-specific code review focused on clippy lints, unsafe
  block usage, Rust edition 2024 conventions, and memory safety. Examples:

<example>
Context: User has just added an unsafe block to a Rust file.
user: "Review this unsafe block"
assistant: "I'll use oxidizer to review the unsafe usage."
<commentary>
Unsafe block additions are a primary oxidizer trigger — Rust-specific review needed.
</commentary>
</example>

<example>
Context: User edited Rust files and wants a quick pre-commit review.
user: "Quick Rust review before I gate"
assistant: "I'll run oxidizer over the changed Rust files."
<commentary>
Pre-gate Rust review — oxidizer before cargo-gate.
</commentary>
</example>

<example>
Context: Clippy is producing warnings the user doesn't understand.
user: "What does this clippy lint mean and should I fix it?"
assistant: "I'll use oxidizer to explain and resolve the clippy issue."
<commentary>
Clippy explanation and resolution is an oxidizer use case.
</commentary>
</example>

model: inherit
color: red
tools: ["Read", "Grep", "Glob", "Bash"]
---

You are oxidizer, a Rust-specific code reviewer. Delegate all review work to the devkit
sentinel agent, providing a Rust-focused prompt:

Focus on: clippy lint compliance, unsafe block justification and soundness,
Rust edition 2024 conventions (unsafe set_var/remove_var, match ergonomics),
memory safety, and error handling patterns (anyhow vs thiserror usage).

Pass the Rust files or diff to devkit sentinel with this Rust-specific focus.
Return the structured report with particular attention to unsafe and edition 2024 issues.
```

- [ ] **Step 6: Commit**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "feat: add all five agents"
```

---

## Task 10: Scaffold and build `joe-secrets` plugin

**Files:**
- Create: `~/.claude/plugins/joe-secrets/plugin.json`
- Create: `~/.claude/plugins/joe-secrets/README.md`
- Create: `~/.claude/plugins/joe-secrets/.gitignore`
- Create: `~/.claude/plugins/joe-secrets/skills/op-resolver/SKILL.md`
- Create: `~/.claude/plugins/joe-secrets/hooks/hooks.json`
- Create: `~/.claude/plugins/joe-secrets/hooks/op-resolver-startup.sh`

- [ ] **Step 1: Create directory structure**

```bash
mkdir -p $HOME/.claude/plugins/joe-secrets/skills/op-resolver
mkdir -p $HOME/.claude/plugins/joe-secrets/hooks
```

- [ ] **Step 2: Write plugin.json**

Create `$HOME/.claude/plugins/joe-secrets/plugin.json`:

```json
{
  "name": "joe-secrets",
  "version": "0.1.0",
  "description": "1Password and direnv session management — secrets validation and env chain tracing",
  "author": {
    "name": "Joseph O'Brien",
    "email": "joeobrien516@gmail.com"
  }
}
```

- [ ] **Step 3: Write .gitignore**

Create `$HOME/.claude/plugins/joe-secrets/.gitignore`:

```
.claude/*.local.md
```

- [ ] **Step 4: Write op-resolver SKILL.md**

Create `$HOME/.claude/plugins/joe-secrets/skills/op-resolver/SKILL.md`:

```markdown
---
name: op-resolver
description: This skill should be used when the user asks to "resolve secrets", "check
  1password", "debug env", "why isn't my op:// ref working", "trace direnv chain",
  "fix secret not loading", or invokes /joe-secrets:op-resolver. Also fires automatically
  at session start via the SessionStart hook.
---

# op-resolver

Validate 1Password authentication, trace the `.envrc` `source_up` chain, detect `op://`
URI conflicts between accounts, and report missing environment variables.

## Step 1: Validate 1Password Auth

```bash
op account list
```

Expected output: table showing at least two accounts (toptal.1password.com and my.1password.com).

If the command fails or shows no accounts: 1Password CLI is not authed. Instruct user:

> "Run `op signin` or open 1Password and unlock it, then retry."

## Step 2: Trace .envrc source_up Chain

Starting from the current working directory, trace upward:

```bash
# Find all .envrc files from CWD to $HOME
d="$PWD"; while [ "$d" != "$HOME" ] && [ "$d" != "/" ]; do
  [ -f "$d/.envrc" ] && echo "$d/.envrc"
  d=$(dirname "$d")
done
```

For each `.envrc` found, check for `source_up` calls and `op://` references:

```bash
# Show op:// refs in each .envrc
grep -h "op://" <each envrc path>
```

Report the chain:

```
DIRENV CHAIN (CWD → HOME)
  /Users/joe/dev/minibox/.envrc  — 3 op:// refs
  /Users/joe/dev/.envrc          — 1 op:// ref (source_up)
  /Users/joe/.envrc              — 2 op:// refs (source_up)
```

## Step 3: Detect op:// Account Conflicts

Scan all `.envrc` files in the chain for `op://` refs and identify which account they target:

- `op://toptal.1password.com/...` → Toptal account
- `op://my.1password.com/...` or `op://Personal/...` → Personal account
- `op://<uuid>/...` — check UUID against `op account list` to identify account

If refs target multiple accounts in the same chain, flag as a potential conflict:

> "WARNING: .envrc chain references both Toptal and Personal 1Password accounts.
> Commands using `op run` may need `--account` flag to disambiguate."

## Step 4: Detect Literal op:// URIs in Shell

Claude's shell context cannot resolve `op://` URIs directly. If the environment
contains literal `op://` values (not resolved secrets), warn:

> "WARNING: Environment variable FOO contains a literal op:// URI.
> Use `op run -- <command>` to inject resolved values into commands."

## Step 5: Report Summary

```
1PASSWORD AUTH     OK (2 accounts)
DIRENV CHAIN       3 files found, 6 op:// refs total
ACCOUNT CONFLICTS  None detected
LITERAL OP:// REFS None in current environment
```

## Common Issues

| Issue | Fix |
|-------|-----|
| `op account list` fails | Run `op signin` or unlock 1Password |
| `source_up` not loading parent | Run `direnv reload` in each parent dir |
| Wrong account selected | Add `--account <uuid>` to `op run` |
| Literal op:// in env | Wrap command with `op run --` |
| op item not found | Use UUID not item name in op:// path |

## Always Use UUIDs

Never use item names in `op://` paths — they may not resolve correctly across accounts.
Use `op item list --vault <vault>` to get exact item UUIDs.
```

- [ ] **Step 5: Write hooks.json**

Create `$HOME/.claude/plugins/joe-secrets/hooks/hooks.json`:

```json
{
  "description": "Session-start hook: validates 1Password auth and orients to outstanding work",
  "hooks": {
    "SessionStart": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "bash $CLAUDE_PLUGIN_ROOT/hooks/op-resolver-startup.sh",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

- [ ] **Step 6: Write op-resolver-startup.sh**

Create `$HOME/.claude/plugins/joe-secrets/hooks/op-resolver-startup.sh`:

```bash
#!/usr/bin/env bash
# op-resolver-startup.sh
# SessionStart hook: validate 1Password auth and trace direnv chain.
# Outputs a system message for Claude summarizing session readiness.
set -euo pipefail

OUTPUT=""
WARNINGS=""

# Step 1: Check 1Password auth
if ! op account list &>/dev/null; then
  WARNINGS="${WARNINGS}WARNING: 1Password CLI not authed — run 'op signin' before using op:// refs.\n"
else
  ACCOUNT_COUNT=$(op account list 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
  OUTPUT="${OUTPUT}1Password: ${ACCOUNT_COUNT} account(s) authed.\n"
fi

# Step 2: Trace .envrc chain from CWD
ENVRC_COUNT=0
OP_REF_COUNT=0
d="${CLAUDE_PROJECT_DIR:-$PWD}"
while [ "$d" != "$HOME" ] && [ "$d" != "/" ]; do
  if [ -f "$d/.envrc" ]; then
    ENVRC_COUNT=$((ENVRC_COUNT + 1))
    refs=$(grep -c "op://" "$d/.envrc" 2>/dev/null || true)
    OP_REF_COUNT=$((OP_REF_COUNT + refs))
  fi
  d=$(dirname "$d")
done

OUTPUT="${OUTPUT}Direnv chain: ${ENVRC_COUNT} .envrc file(s) found, ${OP_REF_COUNT} op:// refs.\n"

# Step 3: Check for literal op:// URIs in environment
LITERAL_COUNT=$(env | grep -c "op://" 2>/dev/null || true)
if [ "$LITERAL_COUNT" -gt 0 ]; then
  WARNINGS="${WARNINGS}WARNING: ${LITERAL_COUNT} env var(s) contain literal op:// URIs — use 'op run --' to resolve.\n"
fi

# Compose system message
MSG="[joe-secrets session-start]\n${OUTPUT}"
if [ -n "$WARNINGS" ]; then
  MSG="${MSG}${WARNINGS}"
fi

printf '{"systemMessage": "%s"}' "$(echo -e "$MSG" | sed 's/"/\\"/g' | tr -d '\n')"
```

Make executable:

```bash
chmod +x $HOME/.claude/plugins/joe-secrets/hooks/op-resolver-startup.sh
```

- [ ] **Step 7: Write README.md**

Create `$HOME/.claude/plugins/joe-secrets/README.md`:

```markdown
# joe-secrets

1Password and direnv session management — secrets validation and env chain tracing.

## Installation

```bash
cc --plugin-dir ~/.claude/plugins/joe-secrets
```

Install alongside `joe-dev` for the full session-start experience.

## Skills

| Skill | Trigger |
|-------|---------|
| op-resolver | "resolve secrets", "check 1password", "debug env", `/joe-secrets:op-resolver` |

## Hooks

`SessionStart` — automatically runs on every new Claude session:
1. Validates `op account list`
2. Traces `.envrc` `source_up` chain
3. Counts `op://` refs and detects literal URIs in environment
4. Returns system message to Claude with readiness summary

## Slash Command

`/joe-secrets:op-resolver` — invoke op-resolver on demand mid-session.

## Prerequisites

- 1Password CLI (`op`) installed and on PATH
- `direnv` installed (optional — chain tracing degrades gracefully without it)
```

- [ ] **Step 8: Init git and commit**

```bash
cd $HOME/.claude/plugins/joe-secrets && git init && git add -A && git commit -m "feat: initial joe-secrets plugin"
```

---

## Task 11: Validate both plugins

- [ ] **Step 1: Check joe-dev structure**

```bash
ls $HOME/.claude/plugins/joe-dev/skills/
ls $HOME/.claude/plugins/joe-dev/agents/
```

Expected: 8 skill directories, 5 agent files.

- [ ] **Step 2: Check joe-secrets structure**

```bash
ls $HOME/.claude/plugins/joe-secrets/skills/
ls $HOME/.claude/plugins/joe-secrets/hooks/
```

Expected: `op-resolver/` skill dir, `hooks.json`, `op-resolver-startup.sh`.

- [ ] **Step 3: Validate hooks.json is valid JSON**

```bash
python3 -c "import json; json.load(open('$HOME/.claude/plugins/joe-secrets/hooks/hooks.json')); print('hooks.json: valid')"
```

Expected: `hooks.json: valid`

- [ ] **Step 4: Validate all plugin.json files**

```bash
python3 -c "import json; json.load(open('$HOME/.claude/plugins/joe-dev/plugin.json')); print('joe-dev plugin.json: valid')"
python3 -c "import json; json.load(open('$HOME/.claude/plugins/joe-secrets/plugin.json')); print('joe-secrets plugin.json: valid')"
```

Expected: both print valid.

- [ ] **Step 5: Check all SKILL.md files have frontmatter**

```bash
for f in $HOME/.claude/plugins/joe-dev/skills/*/SKILL.md; do
  head -1 "$f" | grep -q "^---" && echo "OK: $f" || echo "MISSING FRONTMATTER: $f"
done
for f in $HOME/.claude/plugins/joe-secrets/skills/*/SKILL.md; do
  head -1 "$f" | grep -q "^---" && echo "OK: $f" || echo "MISSING FRONTMATTER: $f"
done
```

Expected: all print `OK:`.

- [ ] **Step 6: Check all agent files have required frontmatter fields**

```bash
for f in $HOME/.claude/plugins/joe-dev/agents/*.md; do
  name=$(grep "^name:" "$f" | head -1)
  model=$(grep "^model:" "$f" | head -1)
  color=$(grep "^color:" "$f" | head -1)
  echo "$f: name=$name model=$model color=$color"
done
```

Expected: all agents show name, model, and color.

- [ ] **Step 7: Commit validation pass**

```bash
cd $HOME/.claude/plugins/joe-dev && git add -A && git commit -m "chore: validation pass complete" --allow-empty
```

---

## Self-Review Against Spec

Spec requirements vs. plan coverage:

| Spec Requirement | Task |
|-----------------|------|
| joe-dev: 8 skills | Tasks 2–8 |
| joe-dev: 5 agents (sentinel, forge, herald, conductor, oxidizer) | Task 9 |
| joe-dev: no hooks | Confirmed — no hooks tasks for joe-dev |
| joe-secrets: op-resolver skill | Task 10 |
| joe-secrets: SessionStart hook | Task 10 |
| joe-secrets: /joe-secrets:op-resolver slash command | Documented in SKILL.md frontmatter (auto-registered) |
| cargo-gate: xtask priority | Task 2 |
| sentinel-autofixer: never auto-apply blocking | Task 3 |
| All agents: thin wrappers delegating to devkit | Task 9 |
| oxidizer delegates to devkit sentinel with Rust focus | Task 9 step 5 |
| project-pulse: writes to memory + Obsidian | Task 7 |
| handon: invoked by joe-secrets hook | Task 8 + Task 10 step 5 |
| hooks.json: plugin wrapper format | Task 10 step 5 |

No gaps found.
