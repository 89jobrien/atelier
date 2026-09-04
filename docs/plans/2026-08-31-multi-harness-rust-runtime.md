# Plan: Multi-Harness Rust Runtime

**Status:** Active. Task 1 landed in commit `317dae5`; Task 2 is next. Task sections preserve their
original red-green implementation instructions for reproducibility.

## Contents

- [Goal](#goal)
- [Approved Design](#approved-design)
- [Context Map](#context-map)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Requirement Traceability](#requirement-traceability)
- [Execution Protocol](#execution-protocol)
- [Tasks](#tasks)
- [Final Quality Gate](#final-quality-gate)
- [Pre-Save Checklist](#pre-save-checklist)

## Goal

Replace Atelier's duplicated script implementations with one Rust runtime that generates and
installs equivalent Claude Code, Codex, and OpenCode integrations while preserving every approved
legacy behavior profile.

## Approved Design

The normative design is `docs/designs/2026-08-31-multi-harness-rust-runtime-design.md`. This plan
uses its final names, paths, wire formats, compatibility profiles, ownership protocol, and safety
rules without introducing alternatives.

## Context Map

### Files to Add

| Path                                   | Purpose                                                           |
| -------------------------------------- | ----------------------------------------------------------------- |
| `Cargo.toml`, `Cargo.lock`             | Root `atelier-cli` package                                        |
| `atelier.toml`                         | Canonical capability, action, alias, and legacy-artifact manifest |
| `src/lib.rs`, `src/main.rs`            | Library/binary split                                              |
| `src/capability/mod.rs`                | Manifest model, parser, and validation                            |
| `src/compat/mod.rs`                    | Compatibility profiles and alias resolution                       |
| `src/host/mod.rs`                      | I/O ports and shared values                                       |
| `src/host/os_file_system/mod.rs`       | Filesystem adapter                                                |
| `src/host/os_process_runner/mod.rs`    | Process adapter                                                   |
| `src/host/os_environment/mod.rs`       | Environment adapter                                               |
| `src/host/system_clock/mod.rs`         | Clock adapter                                                     |
| `src/host/sqlite_handoff_store/mod.rs` | SQLite adapter                                                    |
| `src/harness/mod.rs`                   | Sealed adapter contract and parity                                |
| `src/harness/private/mod.rs`           | Sealing trait                                                     |
| `src/harness/generation/mod.rs`        | Generation planning and application                               |
| `src/harness/ownership/mod.rs`         | Hashes, receipts, journals, recovery                              |
| `src/harness/claude/mod.rs`            | Claude adapter and hook codec                                     |
| `src/harness/codex/mod.rs`             | Codex adapter and hook codec                                      |
| `src/harness/opencode/mod.rs`          | OpenCode adapter and hook codec                                   |
| `src/hooks/mod.rs`                     | Hook normalization and Rust action dispatch                       |
| `src/hooks/git/mod.rs`                 | Repository Git hooks                                              |
| `src/handoff/mod.rs`                   | HANDOFF document model                                            |
| `src/handoff/init/mod.rs`              | Initialization profiles                                           |
| `src/handoff/detect/mod.rs`            | Detection profiles                                                |
| `src/handoff/migrate/mod.rs`           | Migration profiles                                                |
| `src/handoff/database/mod.rs`          | Database command profiles                                         |
| `src/handoff/reconcile/mod.rs`         | Valerie reconciliation profiles                                   |
| `src/handoff/render/mod.rs`            | Context renderer profiles                                         |
| `src/handoff/diagrams/mod.rs`          | Diagram profiles                                                  |
| `src/schema/mod.rs`                    | jq-compatible schema inference                                    |
| `src/cli/mod.rs`                       | Canonical and multicall CLI dispatch                              |
| `templates/opencode/atelier.js`        | Transport-only OpenCode shim template                             |
| `tests/support/mod.rs`                 | Black-box fixture helpers                                         |
| `tests/fixtures/**`                    | Manifest, harness, hook, and compatibility goldens                |

### Files to Update

`justfile`, `.claude-plugin/plugin.json`, `README.md`, `CLAUDE.md`, `docs/design.md`, the active
skill callers listed in Task 35, and the 13 Codex metadata files listed in Task 36.

### Files to Remove After Parity

All eight current `bin/` files, three `.githooks/` files, five `skills/handoff/helpers/` scripts,
the Valerie reconciliation helper, both handover diagram helpers, the obsolete Python diagram test,
and the 13 generated Codex metadata files listed in Task 36.

### Dependency Edges

`cli` depends on domain modules and ports; harness adapters depend on capability and generation
types; production adapters implement ports; domain modules never depend on `cli` or production
adapters. The dependency chain is:

```text
package -> values/ports -> compatibility fixtures -> domain profiles
        -> capability manifest -> ownership-safe generation -> harness adapters
        -> hook codecs -> install/multicall CLI -> caller migration -> script deletion
```

### Risk Checklist

- Interpreter-specific invocations cannot survive script deletion.
- Existing variants disagree on output, timestamps, names, diagrams, and exit codes.
- SQL interpolation and unsafe paths must not be preserved.
- Canonical pre-push must not mutate Git history.
- Existing files and `core.hooksPath=.githooks` require explicit known-legacy adoption.
- Windows release is blocked until extensionless PE Git hooks execute natively.
- OpenCode's JavaScript file may transport events only; all decisions remain Rust-owned.
- Marketplace installation does not bootstrap the native runtime.

## Architecture

- **Crate affected**: new root package `atelier-cli`, library `atelier`, binary `atelier`.
- **Rust**: edition 2024, MSRV 1.88, `publish = false`.
- **Ports**: `FileSystem`, `ProcessRunner`, `Clock`, `Environment`, `HandoffStore`, `HookHandler`.
- **Core types**: `CapabilityManifest`, `Capability`, `ActionDefinition`, `AliasDefinition`,
  `GenerationContext`, `GenerationPlan`, `OwnershipReceipt`, `GenerationJournal`, `HookRequest`,
  `HookResponse`, `CompatibilityProfile`, `HandoffDocument`, `OperationInvocation`, and
  `OperationResult`.
- **Adapters**: `ClaudeAdapter`, `CodexAdapter`, `OpenCodeAdapter`, `OsFileSystem`,
  `OsProcessRunner`, `OsEnvironment`, `SystemClock`, and `SqliteHandoffStore`.
- **Data flow**: TOML manifest and source snapshot -> sealed adapter -> deterministic plan ->
  receipt-checked, journaled installation -> native harness files.
- **Hook flow**: native event -> adapter codec -> manifest action -> Rust handler -> event-aware
  native response.
- **Legacy flow**: argv-0 or `--compat` -> operation/profile -> shared Rust behavior -> exact
  profile output and status.

## Tech Stack

```toml
[package]
name = "atelier-cli"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"
publish = false

[lib]
name = "atelier"

[[bin]]
name = "atelier"
path = "src/main.rs"

[dependencies]
clap = { version = "4.6.6", features = ["derive"] }
rusqlite = { version = "0.40.2", features = ["bundled"] }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
serde_yaml_ng = "0.10.0"
sha2 = "0.11.0"
thiserror = "2.0.20"
time = { version = "0.3.55", features = ["formatting", "local-offset"] }
toml = "1.1.4"
walkdir = "2.5.0"

[dev-dependencies]
assert_cmd = "2.2.2"
tempfile = "3.27.0"
```

## Requirement Traceability

| Requirement                                     | Tasks         |
| ----------------------------------------------- | ------------- |
| Canonical capability model                      | 9-11, 32      |
| Deterministic, ownership-safe generation        | 12-15         |
| Claude, Codex, and OpenCode adapters            | 16-20         |
| Rust-owned hook behavior with transport-only JS | 19-20, 31     |
| Preserve all legacy variants                    | 2, 21-30      |
| Migrate every maintained script                 | 22-30, 35-36  |
| Alias plus explicit profile selection           | 2, 33-34      |
| Compatibility report and known-legacy adoption  | 14, 32        |
| Runtime bootstrap and Claude activation         | 34-35, 37, 39 |
| macOS, Linux, and Windows support               | 5, 34, 38-39  |
| Safe SQL and non-mutating canonical pre-push    | 8, 25, 30     |
| Documentation and release packaging             | 37-39         |

## Execution Protocol

Before Task 1, create and remain on `feat/multi-harness-rust-runtime`. For every task:

1. Add the named failing test and run the task's `Run` command; confirm failure is caused by the
   missing behavior, not compilation unrelated to the task.
2. Implement only the named behavior using the approved design contracts.
3. Run the task command, `cargo fmt --all -- --check`, and
   `cargo clippy --all-targets -- -D warnings`.
4. Run `git branch --show-current`; stop unless it prints `feat/multi-harness-rust-runtime`.
5. Stage only the task's paths and commit with the exact message shown.

## Tasks

### Task 1: Bootstrap the Rust package

**Crate**: `atelier-cli`
**File(s)**: `Cargo.toml`, `Cargo.lock`, `.gitignore`, `src/lib.rs`, `src/main.rs`, `src/cli/mod.rs`, `tests/cli_smoke.rs`
**Run**: `cargo nextest run -p atelier-cli --test cli_smoke`

1. Run the test command before creating `Cargo.toml`; expected failure is Cargo reporting that no
   package manifest exists.
2. Write `cli_smoke` to execute `atelier --help` and assert success plus `Usage:` and `Commands:`.
3. Add the exact manifest from Tech Stack.
4. Add `/target/` to `.gitignore` without changing existing ignore behavior.
5. Export `cli` from `src/lib.rs`. For this bootstrap commit only, expose
   `pub fn try_run<I, T>(arguments: I) -> ExitCode` with the final generic bounds and return success
   after `Cli::parse_from(arguments)`.
6. Implement `src/main.rs` exactly as `fn main() -> ExitCode {
atelier::try_run(std::env::args_os()) }` with the required `ExitCode` import.
7. Add the typed top-level CLI with `validate`, `generate`, `install`, `hook`, `handoff`, `schema`,
   and `repo-hook` command names. Do not dispatch commands in this bootstrap commit.
8. Commit: `build(atelier): bootstrap Rust CLI package`.

### Task 2: Define errors and compatibility values

**Crate**: `atelier-cli`
**File(s)**: `src/lib.rs`, `src/compat/mod.rs`, `src/capability/mod.rs`, `src/harness/mod.rs`, `src/harness/generation/mod.rs`, `src/harness/ownership/mod.rs`, `src/hooks/mod.rs`, `src/handoff/mod.rs`, `src/host/mod.rs`, `tests/compat_resolution.rs`
**Run**: `cargo nextest run -p atelier-cli`

1. Add module-local `#[cfg(test)]` model tests for every type and enum named in Architecture,
   including all private field names and enum variants from the approved design.
2. Define this complete supporting-type set before `AtelierError`:
   - Capability structs: `CapabilityManifest`, `PluginMetadata`, `PluginAuthor`, `Capability`,
     `CapabilityId`, `ActionId`, `LifecycleEventId`, `LifecycleBinding`, `Requirements`,
     `HarnessMetadataSet`, `HarnessMetadata`, `ActionDefinition`, `AliasDefinition`,
     `LegacyArtifact`, and `LegacyGitConfig` with every field shown in the approved design.
   - Capability enums: `RustOperation` with 13 designed operations; `CapabilityKind` with `Skill`,
     `Agent`, `Hook`, `Binary`, `RepositoryHook`; `HarnessId` with `Claude`, `Codex`, `OpenCode`;
     `Portability`; `VersionSource`; `LegacyRoot`; and `LegacyMatcher`.
   - Generation structs: `GenerationPlan`, `GenerationContext`, `OwnershipReceipt`,
     `OwnershipRecord`, `GenerationJournal`, `JournalOperation`, `ParityReport`, `ParityEntry`,
     `PathEntry`, and `FileMetadata`.
   - Generation enums: `MigrationMode`, `InstallScope`, `FileChange`, `ManagedTarget`,
     `DocumentFormat`, `JournalState`, `ParityStatus`, `PathEntryKind`, and `ApplyMode` with the exact
     designed variants and named variant fields.
   - Hook values: `HookRequest`, `HookResponse`, `HookDecision`, and `HookDecisionKind`.
   - Handoff values: `HandoffDocument`, `HandoffItem`, `HandoffExtraEntry`, `HandoffLogEntry`,
     `HandoffCommit`, `HandoffState`, `ItemKey`, and `ItemStatus`.
   - Runtime values: `CommandSpec`, `CommandOutput`, `OperationInvocation`, `OperationResult`, and
     generic `Runtime<F, P, C, E, S, H>`.
3. Define those values in their owning modules before defining `AtelierError`, then export only the
   types used by public function signatures from `src/lib.rs`.
4. Test all 20 `CompatibilityProfile` variants serialize as lowercase kebab-case.
5. Test selection precedence: explicit profile, full invocation path, basename, canonical default.
6. Test `/repo/bin/handoff-detect`, Windows separators, and one case-insensitive `.exe` suffix.
7. Implement `resolve_compatibility` with exact path lookup before basename normalization.
8. Map every invocation in the design's exact alias table; map `infer-json-schema` to
   `JqSchemaInference` and do not map direct execution of `infer-json-schema.jq`.
9. Add `AtelierError` with `Io`, `InvalidManifest`, `Toml`, `Json`, `Yaml`, `Process`,
   `UnsupportedCapability`, `OwnershipConflict`, `HookBinding`, `Recovery`, `Store`, and
   `Compatibility` variants and source-preserving error chains.
10. Replace the bootstrap `try_run` with the final `Result<ExitCode, AtelierError>` signature and
    update `main` to print the error to stderr and return failure.
11. Commit: `feat(atelier): add core models and compatibility selection`.

### Task 3: Define host ports and runtime injection

**Crate**: `atelier-cli`
**File(s)**: `src/host/mod.rs`, `src/lib.rs`, `tests/runtime_ports.rs`
**Run**: `cargo nextest run -p atelier-cli`

1. Add compile-failing inline tests that implement each port with a minimal fake.
2. Define `FileSystem`, `ProcessRunner`, `Clock`, `Environment`, `HandoffStore`, and `HookHandler`
   with the exact signatures in the approved design.
3. Add `PathEntry`, `PathEntryKind`, `FileMetadata`, `CommandSpec`, `CommandOutput`, `Runtime`,
   `OperationInvocation`, and `OperationResult` with named fields.
4. Implement `Runtime::new`; keep every dependency instance-owned and avoid global mutable state.
5. Add `run_with` output sinks and prove a fake runtime captures stdout and stderr independently.
6. Commit: `feat(atelier): establish runtime host ports`.

### Task 4: Implement deterministic test hosts

**Crate**: `atelier-cli`
**File(s)**: `src/host/mod.rs`, `tests/support/mod.rs`, `tests/support/fake_host.rs`, `tests/host_fakes.rs`
**Run**: `cargo nextest run -p atelier-cli --test host_fakes`

1. Test `MemoryFileSystem` stores bytes, metadata, links, executable state, and deterministic walks.
2. Test failure injection by operation index for write, rename, removal, and process calls.
3. Implement `FakeProcessRunner` with ordered expected `CommandSpec` values and captured calls.
4. Implement `FixedClock`, `FakeEnvironment`, `MemoryHandoffStore`, and `FakeHookHandler`.
5. Make unmet process expectations and unused expected calls fail with diagnostic values.
6. Commit: `test(atelier): add deterministic runtime fakes`.

### Task 5: Implement OS filesystem, clock, and environment adapters

**Crate**: `atelier-cli`
**File(s)**: `src/host/os_file_system/mod.rs`, `src/host/system_clock/mod.rs`, `src/host/os_environment/mod.rs`, `src/host/mod.rs`, `tests/os_host.rs`
**Run**: `cargo nextest run -p atelier-cli --test os_host`

1. Test atomic write replacement, recursive walk ordering, symlink inspection, executable copies,
   rename, recursive removal, and missing-path errors inside `tempfile::TempDir`.
2. Implement writes as create-neighbor-temp, flush, sync, then rename; remove the temp on failure.
3. Implement Unix executable bits with permissions mode and document Windows `set_executable` as a
   no-op.
4. Implement environment defaults from exact OS variables without a directory helper crate.
5. Test state, executable, Codex, and OpenCode root precedence from the design.
6. Commit: `feat(atelier): add operating-system host adapters`.

### Task 6: Implement subprocess execution

**Crate**: `atelier-cli`
**File(s)**: `src/host/os_process_runner/mod.rs`, `src/host/mod.rs`, `tests/os_process_runner.rs`
**Run**: `cargo nextest run -p atelier-cli --test os_process_runner`

1. Test exact argv, cwd, environment replacement, stdin bytes, stdout, stderr, and nonzero status.
2. Implement `OsProcessRunner` with `std::process::Command`; never invoke through a shell.
3. Return spawn/I/O failures as `AtelierError::Io`; return completed nonzero commands as
   `CommandOutput`, allowing each compatibility profile to choose status behavior.
4. Commit: `feat(atelier): add subprocess host adapter`.

### Task 7: Model and parse HANDOFF documents

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/mod.rs`, `tests/handoff_document.rs`, `tests/fixtures/handoff/current.yaml`, `tests/fixtures/handoff/legacy.yaml`
**Run**: `cargo nextest run -p atelier-cli --test handoff_document`

1. Test every field of `HandoffDocument`, `HandoffItem`, `HandoffExtraEntry`, `HandoffLogEntry`,
   `HandoffCommit`, `HandoffState`, `ItemKey`, and `ItemStatus`.
2. Include object commits, bare scalar commits, tagged `!Sha` scalars, unknown statuses, null optional
   values, `depends_on`, and unknown extension fields.
3. Use Serde flatten maps to preserve unknown fields and a custom commit deserializer for object,
   string, and tagged scalar forms.
4. Round-trip both fixtures without losing semantically represented data.
5. Commit: `feat(atelier): add tolerant handoff document model`.

### Task 8: Implement safe SQLite storage

**Crate**: `atelier-cli`
**File(s)**: `src/host/sqlite_handoff_store/mod.rs`, `src/host/mod.rs`, `tests/sqlite_handoff_store.rs`
**Run**: `cargo nextest run -p atelier-cli --test sqlite_handoff_store`

1. Test schema creation, upsert, query ordering, completion, status updates, and missing rows.
2. Include project and item values containing quotes and SQL syntax; assert they remain data.
3. Implement every query with `rusqlite` parameters and transactions for multi-item upserts.
4. Preserve nullable columns and designed status strings; never construct SQL with interpolation.
5. Commit: `feat(atelier): add parameterized handoff storage`.

### Task 9: Parse the canonical capability manifest

**Crate**: `atelier-cli`
**File(s)**: `src/capability/mod.rs`, `tests/capability_manifest.rs`, `tests/fixtures/manifests/minimal.toml`, `tests/fixtures/manifests/full.toml`
**Run**: `cargo nextest run -p atelier-cli --test capability_manifest`

1. Test every designed manifest type and kebab-case enum representation.
2. Test `Skill` directory, `Agent` file, source-less `Binary`, `Hook`, and `RepositoryHook` forms.
3. Parse harness metadata, lifecycle bindings, actions, aliases, legacy artifacts, and legacy Git
   config from `full.toml`.
4. Implement `load_manifest` through `FileSystem`; report path-aware TOML errors.
5. Commit: `feat(atelier): parse canonical capability manifests`.

### Task 10: Validate manifest invariants

**Crate**: `atelier-cli`
**File(s)**: `src/capability/mod.rs`, `tests/capability_validation.rs`, `tests/fixtures/manifests/invalid.toml`
**Run**: `cargo nextest run -p atelier-cli --test capability_validation`

1. Test schema version, lowercase IDs, duplicate IDs, absolute paths, parent traversal, missing
   sources, invalid source kinds, absent actions, duplicate aliases, and invalid SHA-256 strings.
2. Test each lifecycle binding references one action and at least one harness.
3. Test actions list all possible `HookDecisionKind` outcomes and aliases map an operation/profile.
4. Implement validation as an accumulating diagnostic pass, returning all violations in stable path
   order rather than stopping at the first error.
5. Commit: `feat(atelier): validate capability invariants`.

### Task 11: Load immutable generation snapshots

**Crate**: `atelier-cli`
**File(s)**: `src/capability/mod.rs`, `src/harness/generation/mod.rs`, `tests/generation_context.rs`
**Run**: `cargo nextest run -p atelier-cli --test generation_context`

1. Test `load_generation_context` walks only manifest-declared sources and sorts paths.
2. Reject symlinks escaping `source_root` and source changes during snapshot loading.
3. Resolve `git rev-parse HEAD` only for `GitHead`; preserve literal versions without Git.
4. Load prior receipt and `core.hooksPath` into `external_state` through injected ports.
5. Populate all designed roots and `MigrationMode` without adapter I/O.
6. Commit: `feat(atelier): load deterministic generation snapshots`.

### Task 12: Implement canonical ownership hashes

**Crate**: `atelier-cli`
**File(s)**: `src/harness/ownership/mod.rs`, `tests/ownership_hashes.rs`
**Run**: `cargo nextest run -p atelier-cli --test ownership_hashes`

1. Add fixed expected SHA-256 fixtures for file, executable, link, JSON key, and TOML key framing.
2. Test object-key sorting, array-order preservation, NUL-separated key paths, executable mode, and
   unrelated structured-document edits.
3. Implement the exact framing rules from the design with `sha2`; normalize TOML through the
   equivalent sorted JSON value model.
4. Commit: `feat(atelier): add canonical ownership hashing`.

### Task 13: Serialize generation plans and receipts

**Crate**: `atelier-cli`
**File(s)**: `src/harness/generation/mod.rs`, `src/harness/ownership/mod.rs`, `tests/generation_plan.rs`
**Run**: `cargo nextest run -p atelier-cli --test generation_plan`

1. Test every `FileChange`, `ManagedTarget`, `OwnershipRecord`, `OwnershipReceipt`,
   `GenerationJournal`, `JournalState`, and `JournalOperation` representation.
2. Test deterministic change ordering by normalized target path and document key.
3. Test receipt paths join the selected state root with `receipts/claude-user.json`,
   `receipts/codex-user.json`, and `receipts/opencode-project.json`.
4. Add stable Serde representations and constructors for the generation values introduced in Task
   2; reject unsupported receipt and journal schema versions.
5. Commit: `feat(atelier): model generation plans and receipts`.

### Task 14: Enforce strict, adopt, and known-legacy planning

**Crate**: `atelier-cli`
**File(s)**: `src/harness/ownership/mod.rs`, `src/harness/generation/mod.rs`, `tests/generation_migration.rs`, `tests/compatibility_report.rs`
**Run**: `cargo nextest run -p atelier-cli --test generation_migration --test compatibility_report`

1. Test strict conflict on every unreceipted target.
2. Test `AdoptMatching` accepts only exact canonical hashes.
3. Test `ReplaceKnownLegacy` accepts fixed hashes, version-ignored plugin JSON, source-root-relative
   Codex links, and exactly `core.hooksPath=.githooks`.
4. Test mismatched bytes, link targets, JSON fields, and Git config remain conflicts.
5. Ensure dry-run reports adoption/replacement decisions without mutation.
6. Emit a stable compatibility report naming each migrated path, selected profile, replacement
   command, adopted hash, unsupported interpreter invocation, and ownership conflict.
7. Commit: `feat(atelier): add safe legacy adoption modes`.

### Task 15: Apply and recover generation transactions

**Crate**: `atelier-cli`
**File(s)**: `src/harness/generation/mod.rs`, `src/harness/ownership/mod.rs`, `tests/generation_recovery.rs`
**Run**: `cargo nextest run -p atelier-cli --test generation_recovery`

1. Inject failure before and after every write, backup, replacement, Git config update, receipt
   replacement, and journal-state update.
2. Test reverse rollback, previous-receipt restoration, next-receipt commit-point detection, and
   committed-journal cleanup.
3. Stage and back up beside each target; store journals only under `state_root`.
4. Implement `recover_generation` before every mutation and `apply_generation` with receipt-last
   commit semantics.
5. Test idempotent second application and refusal to overwrite user-modified managed content.
6. Commit: `feat(atelier): add recoverable generation transactions`.

### Task 16: Seal harness adapters and parity reporting

**Crate**: `atelier-cli`
**File(s)**: `src/harness/mod.rs`, `src/harness/private/mod.rs`, `tests/harness_parity.rs`
**Run**: `cargo nextest run -p atelier-cli --test harness_parity`

1. Define sealed `HarnessAdapter` and unit structs `ClaudeAdapter`, `CodexAdapter`, and
   `OpenCodeAdapter`.
2. Test `Native`, `Adapted`, and `Unsupported` entries for each capability and metadata field.
3. Fail generation for unsupported `Universal` capabilities and unsupported required hook outcomes;
   retain report-only unsupported fields for `Adaptable` capabilities.
4. Sort parity entries by capability ID and metadata path.
5. Commit: `feat(atelier): establish harness adapter contract`.

### Task 17: Generate Claude Code artifacts

**Crate**: `atelier-cli`
**File(s)**: `src/harness/claude/mod.rs`, `.claude-plugin/plugin.json`, `tests/harness_claude.rs`, `tests/fixtures/harness/claude/plugin.json`, `tests/fixtures/harness/claude/hooks.json`
**Run**: `cargo nextest run -p atelier-cli --test harness_claude`

1. Test exact plugin keys, author fields, Git-hash version, canonical skill/agent paths, and optional
   `hooks/hooks.json`.
2. Generate the exact command-plus-args Claude hook handler schema from the design.
3. Group hooks by event/matcher and sort event, matcher, and action IDs deterministically.
4. Replace the checked-in `.claude-plugin/plugin.json` with the generated Claude manifest and test
   that regeneration produces no diff.
5. Preserve prompt bodies byte-for-byte from `GenerationContext.sources`.
6. Commit: `feat(atelier): generate Claude Code plugin artifacts`.

### Task 18: Generate Codex artifacts

**Crate**: `atelier-cli`
**File(s)**: `src/harness/codex/mod.rs`, `tests/harness_codex.rs`, `tests/compat_codex_sync.rs`, `tests/fixtures/harness/codex/plugin.json`, `tests/fixtures/harness/codex/hooks.json`, `tests/fixtures/harness/codex/config.toml`, `tests/fixtures/compat/codex-sync/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test harness_codex --test compat_codex_sync`

1. Test `.codex-plugin/plugin.json`, bundled hooks, all 27 `atelier-`-prefixed skill installations,
   13 explicit OpenAI interface records, and ten role configs.
2. Emit exact command/`commandWindows` fields and plugin-relative hooks path.
3. Own only `agents.atelier-`-prefixed TOML keys declared by the manifest; preserve every unrelated
   `config.toml` key.
4. Report unsupported agent tools/skills and unrepresentable permissions without prompt injection.
5. Freeze `CodexSync` stdout/stderr/status, plugin link, skill links, stale managed removal, and
   unmanaged collision behavior; preserve safe output while refusing its legacy broad deletion.
6. Commit: `feat(atelier): generate Codex plugin artifacts`.

### Task 19: Generate the OpenCode transport adapter

**Crate**: `atelier-cli`
**File(s)**: `src/harness/opencode/mod.rs`, `templates/opencode/atelier.js`, `tests/harness_opencode.rs`, `tests/fixtures/harness/opencode/atelier.js`
**Run**: `cargo nextest run -p atelier-cli --test harness_opencode`

1. Test user/project paths, 27 skills, ten agents, and callback omission when no binding exists.
2. Generate only `tool.execute.before` and `tool.execute.after` callbacks from declared bindings.
3. Spawn `ATELIER_BIN` or `atelier` without a shell; send exactly one JSON object and reject
   malformed/trailing child output.
4. Test continue, replacement, block, diagnostics, paths containing spaces, and nonzero child exit.
5. Add a source scan asserting the shim contains no policy terms, allow fallback, or duplicated
   compatibility behavior.
6. Commit: `feat(atelier): generate OpenCode transport shim`.

### Task 20: Normalize and encode lifecycle hooks

**Crate**: `atelier-cli`
**File(s)**: `src/hooks/mod.rs`, `src/harness/claude/mod.rs`, `src/harness/codex/mod.rs`, `src/harness/opencode/mod.rs`, `tests/hook_conformance.rs`, `tests/fixtures/hooks/claude.json`, `tests/fixtures/hooks/codex.json`, `tests/fixtures/hooks/opencode.json`
**Run**: `cargo nextest run -p atelier-cli --test hook_conformance`

1. Parse common session, cwd, event, and payload fields without dropping native extensions.
2. Test the complete approved event/decision matrix and event-aware `encode_hook(request, response)`.
3. Encode Claude/Codex deny, allow, updated input, and context using each event's exact native keys.
4. Encode OpenCode's exact decision envelope and reject unsupported decisions before execution.
5. Implement `resolve_hook_action`; reject absent, duplicate, wrong-harness, or wrong-event bindings.
6. Commit: `feat(atelier): add cross-harness hook codecs`.

### Task 21: Freeze the compatibility fixture protocol

**Crate**: `atelier-cli`
**File(s)**: `tests/support/compat_case.rs`, `tests/compat_fixture_schema.rs`, `tests/fixtures/compat/README.md`
**Run**: `cargo nextest run -p atelier-cli --test compat_fixture_schema`

1. Define each case with request YAML, stdin, expected stdout/stderr/status, before/after trees,
   expected process calls, links, modes, and database rows.
2. Make the runner reject missing expectations and normalize only platform path separators explicitly
   marked by the fixture.
3. Record security exceptions separately from expected bytes; never update a golden to reproduce SQL
   injection or unsafe traversal.
4. Commit: `test(atelier): define legacy compatibility fixtures`.

### Task 22: Port handoff initialization profiles

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/init/mod.rs`, `tests/compat_handoff_init.rs`, `tests/fixtures/compat/public-handoff-init/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_handoff_init`

1. Freeze unknown-argument, non-Git, existing-token, `--force`, package discovery, sanitization,
   deduplication, timestamp, branch, and `.gitignore` cases from `bin/handoff-init`.
2. Implement `HandoffInit` through ports using fixed clock/process fixtures.
3. Preserve exact stdout/stderr/status and managed-block behavior without shell dependencies.
4. Commit: `feat(atelier): port handoff initialization`.

### Task 23: Port both handoff detection profiles

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/detect/mod.rs`, `tests/compat_handoff_detect.rs`, `tests/fixtures/compat/public-handoff-detect/cases.yaml`, `tests/fixtures/compat/helper-handoff-detect/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_handoff_detect`

1. Freeze lazy init, helper no-init, nested manifests, workspace fallback, `.workspace` naming,
   migration fallback, metadata flags, missing-file status 2, and unknown arguments.
2. Dispatch both profiles through one discovery core with profile-specific naming and side effects.
3. Commit: `feat(atelier): port handoff detection variants`.

### Task 24: Port both handoff migration profiles

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/migrate/mod.rs`, `tests/compat_handoff_migrate.rs`, `tests/fixtures/compat/public-handoff-migrate/cases.yaml`, `tests/fixtures/compat/helper-handoff-migrate/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_handoff_migrate`

1. Freeze destination naming, `git mv` calls, byte-preserving helper behavior, public timestamp
   normalization, missing files, stderr notices, and usage statuses.
2. Reject destinations escaping `.ctx`; preserve all safe profile distinctions.
3. Commit: `feat(atelier): port handoff migration variants`.

### Task 25: Port handoff database command profiles

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/database/mod.rs`, `tests/compat_handoff_database.rs`, `tests/fixtures/compat/public-handoff-database/cases.yaml`, `tests/fixtures/compat/helper-handoff-database/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_handoff_database`

1. Freeze `init`, `upsert`, `query`, `complete`, `status`, ordering, empty items, unknown statuses,
   malformed logs, bare commits, usage failures, and both timestamp validators.
2. Route storage through `HandoffStore`; preserve accepted timestamp differences but not SQL
   interpolation.
3. Commit: `feat(atelier): port handoff database profiles`.

### Task 26: Port Valerie reconciliation profiles

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/reconcile/mod.rs`, `tests/compat_reconcile.rs`, `tests/fixtures/compat/cached-valerie-reconcile/cases.yaml`, `tests/fixtures/compat/bundled-valerie-reconcile/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_reconcile`

1. Freeze cache-version selection, argument/status forwarding, missing cache, sync/audit modes,
   detection, title normalization, blocked suffixes, priority mapping, all todo sets, reports, and
   audit status 1.
2. Parse doob JSON in Rust and call doob only through `ProcessRunner`.
3. Preserve external cache delegation only in `CachedValerieReconcile`; keep bundled behavior fully
   Rust-owned in `BundledValerieReconcile`.
4. Commit: `feat(atelier): port Valerie reconciliation profiles`.

### Task 27: Port context document renderers

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/render/mod.rs`, `tests/compat_context_docs.rs`, `tests/fixtures/compat/context-docs-dispatcher/cases.yaml`, `tests/fixtures/compat/shell-context-docs/cases.yaml`, `tests/fixtures/compat/nushell-context-docs/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_context_docs`

1. Freeze generated Markdown/state, stdout/stderr, nulls, malformed priorities, blocked sorting,
   five-log truncation, dependencies, commits, cache preference, and helper selection.
2. Implement one parsed document model with profile-specific renderers; do not invoke Nushell.
3. Preserve dispatcher selection outcomes through injected command/cache availability.
4. Commit: `feat(atelier): port context rendering profiles`.

### Task 28: Port both diagram profiles

**Crate**: `atelier-cli`
**File(s)**: `src/handoff/diagrams/mod.rs`, `tests/compat_diagrams.rs`, `tests/fixtures/compat/nushell-diagrams/cases.yaml`, `tests/fixtures/compat/python-diagrams/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_diagrams`

1. Freeze dependency inference, issue references, labels, node IDs, burn, velocity, hotspots,
   blockers, gates, `all`, invalid names, empty input, and trailing newlines.
2. Preserve Nushell `block-beta`, top-nine/three-column hotspots, and suppressed dependency output.
3. Preserve Python `xychart-beta`, top-eight hotspots, empty dependency block, and invalid-name status
   2 without retaining Python.
4. Commit: `feat(atelier): port diagram rendering profiles`.

### Task 29: Port jq-compatible schema inference

**Crate**: `atelier-cli`
**File(s)**: `src/schema/mod.rs`, `tests/compat_schema.rs`, `tests/fixtures/compat/jq-schema-inference/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_schema`

1. Freeze primitives, null, empty/heterogeneous arrays, object merge, required intersections, type
   sorting, property order, metadata text, malformed JSON, and final newline.
2. Implement recursive inference and merge over `serde_json::Value` with deterministic maps.
3. Preserve the description `Generated from sample JSON with jq.` under the compatibility profile.
4. Commit: `feat(atelier): port JSON Schema inference`.

### Task 30: Port repository Git-hook profiles

**Crate**: `atelier-cli`
**File(s)**: `src/hooks/git/mod.rs`, `tests/compat_git_hooks.rs`, `tests/fixtures/compat/git-pre-commit/cases.yaml`, `tests/fixtures/compat/git-pre-push/cases.yaml`, `tests/fixtures/compat/git-post-commit/cases.yaml`
**Run**: `cargo nextest run -p atelier-cli --test compat_git_hooks`

1. Freeze recursion guards, global delegation, manifest absence, version stamping, upstream/no-
   upstream diffs, Claude invocation, suppressed/fatal failures, and legacy pre-push commit creation.
2. Implement canonical pre-push separately and assert HEAD/index remain byte-identical.
3. Keep commit-during-push reachable only through `GitPrePush`; never use it during dry-run tests.
4. Immediately inspect temporary-repo log/status after every dry-run hook test.
5. Commit: `feat(atelier): port repository Git hooks safely`.

### Task 31: Wire hook actions and operation dispatch

**Crate**: `atelier-cli`
**File(s)**: `src/hooks/mod.rs`, `src/lib.rs`, `tests/operation_dispatch.rs`
**Run**: `cargo nextest run -p atelier-cli --test operation_dispatch`

1. Implement `RustHookHandler`, `dispatch_hook`, and `execute_operation` over the final operation
   enum.
2. Test every operation/profile pair reaches exactly one module and returns `OperationResult`.
3. Test hook decode -> action resolution -> Rust handler -> event-aware encode, including errors.
4. Commit: `feat(atelier): dispatch Rust-owned operations and hooks`.

### Task 32: Populate and validate atelier.toml

**Crate**: `atelier-cli`
**File(s)**: `atelier.toml`, `tests/repository_manifest.rs`
**Run**: `cargo nextest run -p atelier-cli --test repository_manifest`

1. Enumerate the exact 27 skills, ten agents, binary, aliases, actions, requirements, lifecycle
   bindings, and portability declared in the design.
2. Move all 13 OpenAI interface records into Codex metadata without changing their values.
3. Record exact current legacy hashes, version-ignored plugin JSON, source-relative Codex links, and
   `core.hooksPath=.githooks`.
4. Test every declared source exists, every active source is declared, and hidden/empty directories
   are excluded.
5. Commit: `feat(atelier): add canonical capability inventory`.

### Task 33: Complete canonical and multicall CLI dispatch

**Crate**: `atelier-cli`
**File(s)**: `src/cli/mod.rs`, `src/lib.rs`, `tests/multicall.rs`
**Run**: `cargo nextest run -p atelier-cli --test multicall`

1. Implement the exact canonical command tree from the design.
2. Resolve `--compat` before operation parsing and preserve legacy argv shapes for every alias.
3. Test all aliases by basename and full path, including Windows `.exe` normalization.
4. Render `OperationResult` bytes to the injected sinks unchanged and return its exact exit code.
5. Commit: `feat(atelier): complete multicall CLI dispatch`.

### Task 34: Install binaries, aliases, and repository hooks

**Crate**: `atelier-cli`
**File(s)**: `src/cli/mod.rs`, `src/harness/generation/mod.rs`, `tests/install_aliases.rs`, `tests/git_hook_install.rs`, `tests/claude_activation.rs`
**Run**: `cargo nextest run -p atelier-cli --test install_aliases --test git_hook_install --test claude_activation`

1. Test Unix symlinks, Windows `.exe` copies, exact extensionless Windows Git-hook copies, and
   compatibility selection from installed names.
2. Test strict/adopt/known-legacy installation and transaction rollback restoration.
3. Manage `core.hooksPath` with its prior value and refuse user-changed values.
4. Run real temporary Git repositories for pre-commit, pre-push, and post-commit on each native OS.
5. For Claude user installation, invoke `claude plugin marketplace update bazaar`, uninstall the
   prior `atelier` plugin with `--force`, then install `atelier@bazaar`; propagate the first fatal
   activation failure with captured stderr.
6. Commit: `feat(atelier): install managed aliases and Git hooks`.

### Task 35: Replace setup entry points and active callers

**Crate**: `atelier-cli`
**File(s)**: `justfile`, `skills/handoff/SKILL.md`, `skills/handon/SKILL.md`, `skills/handdown/SKILL.md`, `skills/handover/SKILL.md`, `skills/handup/SKILL.md`, `skills/eod/SKILL.md`, `skills/project-pulse/SKILL.md`, `skills/triage/SKILL.md`, `skills/onboard-atelier/SKILL.md`, `skills/handoff/references/schema.md`, `tests/no_legacy_callers.rs`, `tests/runtime_requirement.rs`
**Run**: `cargo nextest run -p atelier-cli --test no_legacy_callers --test runtime_requirement`

1. Replace interpreter/helper calls with canonical Rust commands and explicit compatibility profiles
   only where historical formatting is required.
2. Make `just init` build/install the Rust CLI before harness installation; keep it a convenience,
   not the Windows bootstrap contract.
3. Test maintained files contain no migrated `.sh`, `.nu`, `.py`, `.jq`, `sqlite3`, `yq`, or
   direct `sync-codex` invocation.
4. Require active skills to check `atelier` on PATH before runtime use and emit exactly
   `atelier runtime missing; run cargo install --path . --locked` when absent.
5. Keep historical docs and committed handoff history unchanged.
6. Commit: `refactor(atelier): route active workflows through Rust`.

### Task 36: Delete legacy scripts after parity

**Crate**: `atelier-cli`
**File(s)**: `bin/generate-ctx-docs`, `bin/handoff-db`, `bin/handoff-detect`, `bin/handoff-init`, `bin/handoff-reconcile`, `bin/infer-json-schema.jq`, `bin/migrate-handoff`, `bin/sync-codex`, `.githooks/pre-commit`, `.githooks/pre-push`, `.githooks/post-commit`, `skills/handoff/helpers/generate-ctx-docs.nu`, `skills/handoff/helpers/generate-ctx-docs.sh`, `skills/handoff/helpers/handoff-db.sh`, `skills/handoff/helpers/handoff-detect.sh`, `skills/handoff/helpers/migrate-handoff.sh`, `skills/handover/helpers/generate-diagrams.nu`, `skills/handover/helpers/generate-diagrams.py`, `skills/handover/scripts/test_generate_diagrams.py`, `skills/cargo-gate/agents/openai.yaml`, `skills/ci-assist/agents/openai.yaml`, `skills/git-guard/agents/openai.yaml`, `skills/handdown/agents/openai.yaml`, `skills/handoff/agents/openai.yaml`, `skills/handon/agents/openai.yaml`, `skills/handover/agents/openai.yaml`, `skills/handup/agents/openai.yaml`, `skills/hook-diagnostics/agents/openai.yaml`, `skills/minion/agents/openai.yaml`, `skills/onboard-atelier/agents/openai.yaml`, `skills/project-pulse/agents/openai.yaml`, `skills/sentinel-autofixer/agents/openai.yaml`, `.gitignore`, `tests/no_legacy_scripts.rs`
**Run**: `cargo nextest run -p atelier-cli --test no_legacy_scripts`

1. Run every compatibility golden against Rust twice and require identical results.
2. Delete every listed implementation and generated Codex metadata file.
3. Remove obsolete Python cache/test ignore entries while preserving unrelated ignore rules.
4. Test no maintained executable script remains except `templates/opencode/atelier.js`.
5. Test all removed public command names are recreated by `atelier install`.
6. Commit: `refactor(atelier): remove superseded script implementations`.

### Task 37: Update authoritative documentation

**Crate**: `atelier-cli`
**File(s)**: `README.md`, `CLAUDE.md`, `docs/design.md`, `docs/designs/2026-08-31-multi-harness-rust-runtime-design.md`, `tests/documented_commands.rs`
**Run**: `cargo nextest run -p atelier-cli --test documented_commands`

1. Replace the no-build architecture with Cargo build/install, bootstrap, harness roots, aliases,
   receipts, migration modes, compatibility profiles, hook safety, and platform packages.
2. Document marketplace limitation and exact remediation when `atelier` is absent.
3. Test every documented local command exists in `atelier --help` and every referenced path exists or
   is explicitly marked generated.
4. Commit: `docs(atelier): document the multi-harness Rust runtime`.

### Task 38: Add cross-platform CI

**Crate**: `atelier-cli`
**File(s)**: `.github/workflows/ci.yml`, `tests/native_aliases.rs`
**Run**: `cargo nextest run -p atelier-cli --test native_aliases`

1. Add native `macos-13`, `macos-14`, `ubuntu-latest`, and `windows-latest` jobs.
2. Run format, check, clippy, nextest, release build, deterministic generation, aliases, real Git
   hooks, clean-tree checks, and compatibility goldens.
3. On Windows, fail unless extensionless PE hooks execute under Git for Windows.
4. Use a Bash heredoc to create the workflow because repository hooks block direct workflow writes.
5. Commit: `ci(atelier): verify runtime across native platforms`.

### Task 39: Package native release archives

**Crate**: `atelier-cli`
**File(s)**: `.github/workflows/release.yml`, `tests/release_bundle.rs`
**Run**: `cargo nextest run -p atelier-cli --test release_bundle`

1. Package native archives for `x86_64-apple-darwin`, `aarch64-apple-darwin`,
   `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.
2. Include the native executable, `atelier.toml`, `skills/`, `agents/`, and SHA-256 checksums; the
   OpenCode template is compiled into the executable with `include_str!`. Use ZIP on Windows and
   tar.gz elsewhere.
3. Unpack on the native runner, install all three harnesses into temporary roots, rerun for
   idempotence, execute aliases/hooks, and prove modified/unmanaged files conflict safely.
4. Use a Bash heredoc to create the workflow.
5. Commit: `ci(atelier): package native runtime releases`.

## Final Quality Gate

```text
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo nextest run
cargo build --release
atelier validate --manifest atelier.toml
cargo nextest run --test harness_claude
cargo nextest run --test harness_codex
cargo nextest run --test harness_opencode
```

Harness tests create separate temporary roots; no quality gate may target the user's real home
directory.

After the commands pass, generate each harness output twice and compare complete trees, run all
three migration modes in isolated roots, verify every capability has a parity status, inspect Git
log/status after dry-run and pre-push cases, and confirm the working tree contains only intended
migration files. This gate verifies existing commits and must not create an empty commit.

## Pre-Save Checklist

- [x] Every approved requirement maps to one or more tasks.
- [x] Exact owner crate, module paths, type names, commands, and commit messages are specified.
- [x] Every implementation task starts with a failing test and ends with verification and a commit.
- [x] Legacy deletion is causally blocked by compatibility parity.
- [x] SQL, ownership, pre-push, OpenCode shim, and Windows-hook safety gates are explicit.
- [x] No implementation alternative or unresolved architectural choice remains.
