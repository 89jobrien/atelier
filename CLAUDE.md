# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

**Atelier** is a Claude Code plugin — a collection of skills and agents for personal dev workflow automation.
It also contains the staged Rust `atelier-cli` runtime. Plugin content remains Markdown and YAML,
while `Cargo.toml`, `src/`, and `tests/` implement and verify the native command surface.

Plugin version: auto-managed (git-hash based) | Installed via the bazaar marketplace.

## Setup & Reinstall

```bash
just init        # One-time setup: wire git hooks, verify tools, install plugin
just reinstall   # Reinstall plugin without re-running full init
```

The pre-push hook stamps the manifest version and reinstalls the plugin when plugin sources changed.
The post-commit hook is intentionally a no-op.

## Architecture

```
atelier/
├── .claude-plugin/plugin.json   # Plugin metadata
├── skills/                      # 27 procedural workflow skills
├── agents/                      # 10 agent definitions
├── bin/                         # 8 legacy commands and compatibility utilities
├── src/                         # Native atelier library and CLI parser
├── tests/                       # Rust CLI integration tests
├── docs/designs/                # Current architectural designs
├── docs/plans/                  # Current implementation plans
├── docs/design.md               # Historical plugin-suite design
└── justfile                     # Setup automation
```

`bin/` is added to PATH automatically by Claude Code when the plugin is installed. Its current
commands are `generate-ctx-docs`, `handoff-db`, `handoff-detect`, `handoff-init`,
`handoff-reconcile`, `infer-json-schema.jq`, `migrate-handoff`, and `sync-codex`.

### Skills

| Skill                | Trigger examples                                                     |
| -------------------- | -------------------------------------------------------------------- |
| `ai-review`          | "review AI changes", "audit generated code"                          |
| `cap`                | "/cap", "commit and push", "ship it", "save progress"                |
| `cargo-gate`         | "run gates", "validate rust", "pre-commit check"                     |
| `ci-assist`          | "edit workflow", "fix CI", "check cross-compile"                     |
| `cleanup`            | "clean merged branches", "prune worktrees"                           |
| `commit-msg`         | "write commit message", "summarize staged changes"                   |
| `eod`                | "/eod", "end of day", "wrap up session"                              |
| `git-guard`          | "safe to commit", "check merge strategy"                             |
| `harbor-adapter`     | "adapt Harbor workflow", "translate Harbor tasks"                    |
| `handoff`            | "write handoff", "end of session"                                    |
| `handon`             | "start session", "orient to work", "what's outstanding"              |
| `handdown`           | "write back analysis", "annotate handoffs", "persist handup context" |
| `handover`           | "visualize the handoff", "show handoff"                              |
| `handup`             | "survey all projects", "what's open across repos"                    |
| `herald`             | "cross-project summary", "write daily note"                          |
| `hook-diagnostics`   | "show hook status", "what hooks ran"                                 |
| `insights-audit`     | "validate insights report", "audit the report"                       |
| `merge`              | "merge this branch", "integrate changes"                             |
| `minion`             | "dispatch subagent", "run in parallel", "fast subtask"               |
| `onboard`            | "onboard me", "how do I set up atelier"                              |
| `project-pulse`      | "end session", "capture state", "session summary"                    |
| `sadd`               | "subagent-driven development", "execute tasks with agents"           |
| `self-review`        | "fill in the reflect", "write the session review"                    |
| `sentinel-autofixer` | "apply review fixes", "fix sentinel suggestions"                     |
| `triage`             | "triage", "what needs fixing"                                        |
| `using-gkg`          | "use gkg", "query the code graph"                                    |
| `using-rslm`         | "use rslm", "Rhai language server"                                   |

### Agents

Agents define routing and execution procedures. Prefer companion tools for reusable domain
implementations, while keeping required policy explicit in each agent definition.

| Agent        | Purpose                                                           |
| ------------ | ----------------------------------------------------------------- |
| `sentinel`   | Structured code review (hexagonal arch, Rust/Go conventions)      |
| `forge`      | Primary dev companion: design, debug, refactor                    |
| `herald`     | Cross-repo activity → Obsidian daily note                         |
| `conductor`  | devloop → doob → devkit workflow pipeline                         |
| `oxidizer`   | Rust-specific review (clippy, unsafe, edition 2024)               |
| `minion`     | General-purpose parallel worker for independent subtasks          |
| `maxion`     | Structured task planner for complex or ambiguous items            |
| `midion`     | Parallel worker dispatched by handon for backlog items            |
| `workshop`   | Full-suite test agent — verifies skill loading and plugin surface |
| `rslm-agent` | RSLM architecture and implementation specialist                   |

**Agent Permissions:**

- `workshop` and `minion` have `permissionMode: acceptEdits` — they can write/edit files without
  prompting. `oxidizer` has `permissionMode: default`. All other agents omit the field and
  prompt before file changes.

## Handoff System

Three-file model per project:

| File                                           | Committed | Purpose                                        |
| ---------------------------------------------- | --------- | ---------------------------------------------- |
| `.ctx/HANDOFF.<name>.<base>.yaml` (in `.ctx/`) | YES       | Source of truth — tasks, log, metadata         |
| `.ctx/HANDOFF.<name>.<base>.state.yaml`        | NO        | Project snapshot (branch, build status, tests) |
| `.ctx/HANDOFF.md`                              | NO        | Rendered human-readable reference              |

`<name>` is derived from the nearest `Cargo.toml`/`pyproject.toml`/`go.mod`; `<base>` is the
repo root directory name. `handoff-init` creates stubs and manages the `.gitignore` block on
first use — it runs lazily via `handoff-detect` and never needs to be called directly.

Items have immutable `id`/`title`/`description`/`priority` (P0/P1/P2) and mutable `status`
(open/done/parked/blocked). The log section prepends newest-first. Items also sync to
`~/.local/share/atelier/handoff.db` (SQLite) for cross-session queries, and `handoff-reconcile`
is the scripted bridge that captures open HANDOFF items into the authoritative `doob` backlog.

## Key Design Rules

- **Shared implementations** — prefer companion tools for reusable domain logic while keeping
  required routing and execution policy explicit in agent definitions.
- `.ctx/HANDOFF.*.state.yaml` is intentionally gitignored because it tracks local session state.
- **No duplicate hooks** — global hooks (`rtk-rewrite.sh`, `cargo-fmt.nu`, etc.) live in
  `~/.claude/hooks/`; never copy them here.
- **`cargo-gate` runs xtask first** — always calls `cargo xtask pre-commit`; the skill adds
  reporting on top, not a replacement.
- **Secrets split** — 1Password / `.envrc` logic belongs in the companion `sanctum` plugin, not here.

## Companion Plugins

| Plugin        | Role                                                                                   |
| ------------- | -------------------------------------------------------------------------------------- |
| `sanctum`     | 1Password auth + `.envrc` chain tracing (required for `git-guard` SSH signing)         |
| `hand`        | Standalone session handoff (optional; atelier's handoff skills are the preferred path) |
| `orca-strait` | Parallel TDD orchestrator for Rust workspaces (optional)                               |

`atelier`, `sanctum`, and `orca-strait` install from the `bazaar` marketplace. Register it once
per machine before any `@bazaar` install: `claude plugin marketplace add https://github.com/89jobrien/bazaar`
`hand` and `vault-keeper` have no GitHub remote — `@local` only.

## Session Flow

1. Session starts → `sanctum` validates 1Password auth, traces `.envrc`
2. `sanctum` hands off to `atelier:handon` → triages `.ctx/HANDOFF.<name>.<base>.yaml` by priority
3. Work happens using skills (`cargo-gate`, `git-guard`, `ci-assist`, etc.)
4. Session ends → `project-pulse` captures state snapshot → `handoff` writes `.ctx/HANDOFF` files

## Native Runtime

`cargo run -- --help` is the source of truth for the staged native CLI. The parser currently
exposes `validate`, `generate`, `install`, `hook`, `handoff`, `schema`, and `repo-hook`.
Implementation proceeds through `docs/plans/2026-08-31-multi-harness-rust-runtime.md`.

## Adding or Editing Skills

Each skill lives at `skills/<name>/SKILL.md`. After editing:

1. Run `just reinstall` to reload into Claude Code.
2. Skills are auto-discovered from `skills/` — no `plugin.json` changes needed for new skills.
3. Keep skills as procedural guides — steps Claude follows, not implementations.

## Editing `plugin.json`

The manifest at `.claude-plugin/plugin.json` registers plugin metadata. After any change:

```bash
just reinstall
```

The `version` field is auto-set by the pre-push hook to the source HEAD hash before the hook's
version-stamp commit — do not set it manually.

@OPAVS.md
