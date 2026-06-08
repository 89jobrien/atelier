---
name: using-rslm
description: >
  Use when working on the rslm project — Recursive Language Model inference engine in Rust.
  Triggers on "rslm", "RLM", "recursive language model", "rhai built-ins", "ctx_grep",
  "ctx_slice", "rlm_call", "rslm-core", "rslm-providers", "rslm-store", or any question about
  running queries, ingesting documents, or writing Rhai scripts for the RLM loop.
allowed-tools: Read, Bash, Glob, Grep
---

# using-rslm — Recursive Language Model Inference

rslm is a Rust implementation of the Recursive Language Model (RLM) inference strategy
(Zhang & Khattab, 2025). The model never sees the full context directly — instead, it writes
Rhai scripts that query the context via registered functions, then calls `final_answer` once
it has grounded its answer in the results.

## Architecture

6-crate Cargo workspace at `/Users/joe/dev/rslm`:

| Crate            | Role                                          |
| ---------------- | --------------------------------------------- |
| `rslm-core`      | RLM loop, Rhai engine, registered functions   |
| `rslm-providers` | `LlmProvider` trait, OpenAI + Anthropic impls |
| `rslm-store`     | SQLite chunk store, BM25, HNSW, hybrid search |
| `rslm-cli`       | `rslm` binary — query / interactive / ingest  |
| `rslm-bench`     | Criterion benchmarks + accuracy eval harness  |
| `rslm-harness`   | Eval loop for golden-set accuracy testing     |

Key files:

- `crates/rslm-core/src/rlm.rs` — `Rlm` struct, the loop, `rlm_call` dispatch
- `crates/rslm-core/src/env.rs` — `build_engine`, all Rhai function registrations
- `crates/rslm-core/src/protocol.rs` — `RlmError`, `Notebook`, `Cell`
- `crates/rslm-providers/src/provider.rs` — `LlmProvider` trait, `Message`

## Build & Test

```bash
cargo check --workspace
cargo test --workspace
cargo test --workspace --features live-tests   # real API calls, needs keys
cargo bench -p rslm-bench                       # criterion benchmarks
```

## Rhai Built-ins (always available)

Every script the model writes executes in a fresh `Scope::new()`. No state persists between
iterations. The model MUST call `ctx_grep` or `ctx_slice` before `final_answer`.

| Function       | Signature                | Purpose                              |
| -------------- | ------------------------ | ------------------------------------ |
| `ctx_len`      | `() -> int`              | Byte length of context               |
| `ctx_slice`    | `(start, end) -> String` | Byte-range slice, UTF-8 safe         |
| `ctx_grep`     | `(pattern) -> String`    | Regex search, returns matching lines |
| `print_cell`   | `(msg)`                  | Append to cell output buffer         |
| `final_answer` | `(answer)`               | Signal final answer, exit loop       |
| `rlm_call`     | `(query, ctx) -> String` | Spawn child RLM at depth+1, blocking |

## Store Rhai Built-ins (with `--store`)

| Function     | Signature                      | Purpose                          |
| ------------ | ------------------------------ | -------------------------------- |
| `doc_id`     | `() -> String`                 | Current document ID              |
| `ctx_chunks` | `(doc_id) -> int`              | Number of stored chunks          |
| `ctx_chunk`  | `(doc_id, i) -> String`        | i-th chunk (0-indexed)           |
| `ctx_search` | `(doc_id, query, k) -> String` | BM25 keyword search              |
| `ctx_hybrid` | `(doc_id, query, k) -> String` | Hybrid semantic+BM25 (preferred) |

## Environment Variables

| Variable            | Default                                              | Purpose                           |
| ------------------- | ---------------------------------------------------- | --------------------------------- |
| `RSLM_PROVIDER`     | `openai`                                             | Provider: `openai` or `anthropic` |
| `RSLM_MODEL`        | `gpt-4o` (openai) or `claude-sonnet-4-6` (anthropic) | Model ID override                 |
| `OPENAI_API_KEY`    | —                                                    | Required for OpenAI + embeddings  |
| `ANTHROPIC_API_KEY` | —                                                    | Required for Anthropic            |
| `RUST_LOG`          | —                                                    | Tracing filter (output to stderr) |

## CLI Quick Reference

```bash
# Single query
rslm query "Who wrote the preface?" --context-file book.txt

# With store and hybrid search
rslm query "Summarise chapter 3" \
    --context-file book.txt --store book.db \
    --provider anthropic --verbose

# Ingest a document
rslm ingest --context-file book.txt --store book.db --strategy paragraph

# Interactive REPL
rslm interactive --verbose
```

## Known Gotchas

- `rlm_call` bridges async→sync via `block_in_place` (multi-thread runtime) or a fresh
  current-thread runtime (nested/blocking context). Tests that call `rlm_call` end-to-end
  must use `#[ignore]` or `spawn_blocking` — not bare `#[tokio::test]`.
- `rhai` requires `features = ["sync"]` in `rslm-core` for `Send + Sync` engine closures.
- `thiserror` must be added explicitly — it is not auto-included in the workspace.
- Each Rhai script runs in `Scope::new()` — no variable state persists between model turns.

## Error Types

| Variant                 | Cause                          | Fix                                          |
| ----------------------- | ------------------------------ | -------------------------------------------- |
| `MaxDepthExceeded`      | `rlm_call` at max depth        | Increase `--max-depth` or restructure        |
| `MaxIterationsExceeded` | No `final_answer` in N iters   | Use `--verbose`, increase `--max-iterations` |
| `ScriptError`           | 3 consecutive bad Rhai scripts | Model failing to self-correct                |
| `ProviderError`         | Network / auth / rate limit    | Check API keys                               |

## Additional Resources

- **`docs/assets/rslm.ref.md`** — full reference: all CLI flags, Rhai built-ins, Rust API,
  workflow patterns, error guidance
- **`docs/superpowers/specs/2026-04-23-rslm-design.md`** — original design spec
- **`docs/plans/2026-05-06-rslm-harness.md`** — eval harness plan
