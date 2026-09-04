# atelier

Personal dev workflow plugin — Rust gates, code review, CI, git safety,
multi-repo pulse.

## Installation

```bash
claude plugin marketplace add 89jobrien/bazaar
claude plugin install atelier@bazaar
```

Requires `sanctum` for the session-start hook chain:

```bash
claude plugin install sanctum@bazaar
```

For local repo development, use:

```bash
cd ~/dev/atelier
just init
```

That keeps the Claude plugin installed and also mirrors `skills/`, `bin/`, and the repo root
into Codex-visible locations on machines that have `~/.codex/`.

## Skills

Atelier currently ships 27 skills:

| Workflow             | Skills                                                                                           |
| -------------------- | ------------------------------------------------------------------------------------------------ |
| Session and handoff  | `eod`, `handoff`, `handon`, `handdown`, `handover`, `handup`, `project-pulse`                    |
| Git and delivery     | `cap`, `cleanup`, `commit-msg`, `merge`, `git-guard`, `ci-assist`, `cargo-gate`                  |
| Review and triage    | `ai-review`, `insights-audit`, `self-review`, `sentinel-autofixer`, `hook-diagnostics`, `triage` |
| Agent workflows      | `minion`, `sadd`, `harbor-adapter`, `herald`                                                     |
| Tools and onboarding | `onboard`, `using-gkg`, `using-rslm`                                                             |

## Agents

| Agent        | Purpose                                         |
| ------------ | ----------------------------------------------- |
| `sentinel`   | Structured code review                          |
| `forge`      | Development companion and task router           |
| `herald`     | Cross-repository synthesis into Obsidian        |
| `conductor`  | Devloop, doob, and devkit workflow pipeline     |
| `oxidizer`   | Rust-specific review                            |
| `minion`     | General-purpose parallel worker                 |
| `midion`     | Backlog worker dispatched by handon             |
| `maxion`     | Structured task planner                         |
| `workshop`   | Plugin-surface and skill-loading verification   |
| `rslm-agent` | RSLM architecture and implementation specialist |

## Runtime CLI

The staged `atelier-cli` Rust runtime currently exposes these command groups:

```text
atelier validate
atelier generate
atelier install
atelier hook
atelier handoff <init|detect|migrate|db|reconcile|render|diagrams>
atelier schema infer
atelier repo-hook <pre-commit|pre-push|post-commit>
```

The command surface is bootstrapped; most command behavior remains under implementation.

## Documentation

- [Current Rust runtime design](docs/designs/2026-08-31-multi-harness-rust-runtime-design.md)
- [Current implementation plan](docs/plans/2026-08-31-multi-harness-rust-runtime.md)
- [Historical plugin-suite design](docs/design.md)

## Notes

- `cargo-gate` runs `cargo xtask pre-commit` first — the xtask gate always takes priority.
- `sanctum` must also be installed for the session-start op-resolver + handon chain.
- Agent definitions contain their current routing and execution procedures; some delegate to devkit.
- When `${CODEX_HOME:-$HOME/.codex}` exists, `just reinstall` runs `bin/sync-codex`, which
  symlinks the repo, skills, and commands into Codex-visible locations.
- The plugin manifest is `.claude-plugin/plugin.json`; its version is stamped by the pre-push hook.
