# joe-dev Plugin Suite — Design Spec

**Date:** 2026-04-03
**Author:** Joseph O'Brien
**Status:** Approved

---

## Overview

Two companion plugins that automate the repeated operational workflows identified across active projects
(minibox, maestro, devloop, doob, magi, mcpipe). The split separates secrets/env concerns from dev
workflow concerns, allowing independent installation and versioning.

---

## Plugin 1: `joe-dev`

Personal dev workflow plugin — Rust gates, code review, CI, git safety, multi-repo pulse.

### Directory Structure

```
joe-dev/
├── plugin.json
├── README.md
├── .gitignore
├── docs/
│   └── design.md
├── skills/
│   ├── cargo-gate/SKILL.md
│   ├── sentinel-autofixer/SKILL.md
│   ├── hook-diagnostics/SKILL.md
│   ├── git-guard/SKILL.md
│   ├── ci-assist/SKILL.md
│   ├── project-pulse/SKILL.md
│   ├── handoff/SKILL.md
│   └── handon/SKILL.md
└── agents/
    ├── sentinel.md
    ├── forge.md
    ├── herald.md
    ├── conductor.md
    └── oxidizer.md
```

### Skills

| Skill | Trigger Phrases | Responsibility | Delegates To |
|-------|----------------|----------------|-------------|
| `cargo-gate` | "run gates", "validate rust", "pre-commit check" | Runs `cargo xtask pre-commit` first (always takes priority), then layers unified pass/fail report across fmt/clippy/test/check stages. Auto-fix clippy suggestions with opt-out. | `cargo xtask pre-commit` |
| `sentinel-autofixer` | "apply review fixes", "fix sentinel suggestions", "auto-fix review" | Reads sentinel report, triages blocking vs suggestion-level issues. Applies suggestion-level fixes as dry-run diff, then optionally commits in one batch. Never auto-applies blocking issues. | `sentinel` agent (local) |
| `hook-diagnostics` | "show hook status", "hook failures", "what hooks ran" | Lists all active hooks with last-run status. Surfaces failures from post-tool tracking logs. Reports hook overhead per session. | hook log files |
| `git-guard` | "safe to commit", "check merge strategy", "commit safely" | Runs `git log --oneline main..branch`, detects unsafe rebase candidates (branches with merge commits), recommends merge vs rebase, confirms 1Password SSH signing agent is available. | `git` |
| `ci-assist` | "edit workflow", "fix CI", "check cross-compile", "verify binary" | Heredoc template builder for `.github/workflows/` files (Edit tool blocked). Target triple validator (`file <binary>`). `gh run view` CI diagnostics aggregator. | `gh`, `file` |
| `project-pulse` | "end session", "capture state", "session summary" | Captures branch/commit/PR state per repo. Diffs session-start vs end state. Writes to memory files (`~/.claude/projects/.../memory/`) and Obsidian daily note. | `herald` agent (local), memory system |
| `handoff` | "write handoff", "end of session", "capture handoff" | Writes `HANDOFF.yaml` with completed work, newly discovered gaps, and current project state. Intended as explicit session-end action. | memory system |
| `handon` | "start session", "orient to work", "what's outstanding" | Scans for `HANDOFF.yaml` files across active repos, triages items by priority, presents outstanding work. Invoked automatically by `joe-secrets` session-start hook after secrets validation. | memory system |

### Agents

All agents are thin wrappers that delegate to `devkit` agents. System prompts are ≤5 lines.
No assumptions are made about agents installed on the host machine.

| Agent | Purpose | Delegates To | Trigger Examples |
|-------|---------|-------------|-----------------|
| `sentinel` | Structured code review: hexagonal arch, Rust/Go conventions | devkit sentinel agent | After implementing features; before PRs; when reviewing diffs |
| `forge` | Primary dev companion: design, debug, refactor, prototype | devkit forge agent | Design discussions; debugging sessions; ad-hoc dev work |
| `herald` | Cross-repo activity synthesis → Obsidian daily note | devkit herald agent | End of session; cross-project summaries |
| `conductor` | Workflow orchestrator: devloop → doob → devkit pipeline | devkit conductor agent | After significant commits; when CI fails |
| `oxidizer` | Rust-specific review: clippy, unsafe usage, edition 2024 conventions | devkit sentinel agent (Rust-focused prompt) | After Rust file edits; before cargo-gate; unsafe block additions |

### Manifest

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

---

## Plugin 2: `joe-secrets`

Secrets, environment, and 1Password session management.

### Directory Structure

```
joe-secrets/
├── plugin.json
├── README.md
├── .gitignore
├── skills/
│   └── op-resolver/SKILL.md
└── hooks/
    ├── hooks.json
    └── op-resolver-startup.sh
```

### Skills

| Skill | Trigger Phrases | Responsibility |
|-------|----------------|----------------|
| `op-resolver` | "resolve secrets", "check 1password", "debug env", `/joe-secrets:op-resolver` | Validates `op account list`. Traces `.envrc` `source_up` chain from CWD. Detects `op://` URI conflicts between Toptal and personal 1Password accounts. Reports missing vars. Available as slash command for on-demand mid-session use. |

### Hooks

| Hook | Event | Behavior |
|------|-------|----------|
| `session-start` | `SessionStart` | Shells to `op-resolver-startup.sh`. Runs op-resolver logic (secrets validation, `.envrc` chain trace, conflict detection), then invokes `joe-dev` `handon` skill to surface outstanding HANDOFF items. Sequential: secrets first, orientation second. |

`op-resolver-startup.sh` steps:
1. Run `op account list` — fail fast if not authed
2. Trace `source_up` chain from CWD `.envrc`
3. Detect literal `op://` URIs that need `op run` wrapping
4. Detect account conflicts (toptal vs personal vault refs in same env)
5. Hand off to `handon` skill

### Slash Command

`/joe-secrets:op-resolver` — on-demand invocation of the same op-resolver logic outside of session start.

### Manifest

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

---

## Installation

Both plugins must be installed together for the full session-start experience
(joe-secrets hook chains into joe-dev handon):

```bash
claude --plugin-dir ~/.claude/plugins/joe-dev
claude --plugin-dir ~/.claude/plugins/joe-secrets
```

---

## Key Design Decisions

- **Flat structure**: 8 skills + 5 agents in joe-dev is within the range where a flat layout is
  navigable. No sub-grouping needed.
- **xtask priority**: `cargo-gate` always runs `cargo xtask pre-commit` first. The skill adds
  reporting on top — it does not replace the xtask gate.
- **Thin agents**: All agents delegate to devkit. No domain knowledge is embedded locally; devkit
  is the source of truth for agent behavior.
- **No duplicate hooks**: `joe-dev` has no hooks. Global CLAUDE.md hooks (rtk-rewrite,
  course-correct, cargo-fmt, cargo-check, etc.) are not duplicated here.
- **Secrets split**: `joe-secrets` is independently installable. Projects that don't use 1Password
  can install `joe-dev` alone.
- **Session-start chain**: `joe-secrets` SessionStart hook → op-resolver → handon. The chain is
  defined in `joe-secrets` so the ordering is explicit and the secrets plugin owns session init.
