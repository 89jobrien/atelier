# Diagram Patterns — handover skill

## Node Label Rules (strict)

Every node label in every diagram must satisfy all of:

- **2–3 words** — trim or abbreviate aggressively
- **No newlines** — `\n` or literal line breaks break Mermaid rendering
- **No HTML** — no `<br>`, `<b>`, `<i>`, or any tags
- **No bare parentheses** — Mermaid parses `()` as a special node shape; use `[]` or omit
- **No colons in node IDs** — use camelCase or underscores for IDs, colons only in edge labels

Good: `Auth Gate`, `CI Enforce`, `Bench Fix`
Bad: `Add daemon-side policy gate for mounts`, `Fix phased-deployment CI — HTTP 403 on gh run list`

### Abbreviation guide

| Full title pattern                | Abbreviated   |
| --------------------------------- | ------------- |
| "Add X for Y"                     | "Add X"       |
| "Fix X — reason"                  | "Fix X"       |
| "Improve X coverage"              | "X Coverage"  |
| "Enforce X with CI"               | "X Enforce"   |
| "VZ.framework VZErrorInternal..." | "VZ Bug"      |
| "Tier N Linux-only..."            | "Linux Tests" |

## Flowchart Patterns

### Priority → Item → Status flow

```
flowchart LR
  subgraph P0
    p0a[Auth Gate]
  end
  subgraph P1
    p1a[CI Enforce]
    p1b[Bench Toggles]
    p1c[Deploy Fix]
    p1d[Handler Cov]
  end
  subgraph P2
    p2a[Dual License]
    p2b[E2E Serial]
    p2c[VZ Bug]
    p2d[Dashbox Fixes]
    p2e[Dagu Fixes]
    p2f[Linux Tests]
  end
  p0a --> blocked_or_open
  p1a & p1b & p1c & p1d --> Open
  p2a & p2b & p2d & p2e & p2f --> Open
  p2c --> Blocked
```

Use subgraphs only when there are ≥3 items per priority level. Otherwise use plain nodes.

### Simple flow (few items)

```
flowchart LR
  P0[Auth Gate] --> Open
  P1a[CI Enforce] --> Open
  P1b[Bench Fix] --> Open
  P2a[VZ Bug] --> Blocked
```

## State Diagram Patterns

### Basic status machine with counts

```
stateDiagram-v2
  [*] --> Open : 9 items
  Open --> Done : completed
  Open --> Blocked : 1 item
  Open --> Parked : optional
  Done --> [*]
  Blocked --> Open : unblocked
  Parked --> Open : resumed
```

Annotate with real counts from the HANDOFF data.

## Sequence Diagram Patterns

### Session log timeline

```
sequenceDiagram
  participant Apr03 as 2026-04-03
  participant Apr02 as 2026-04-02
  participant Apr01 as 2026-04-01
  Note over Apr03: Bench timeout fix
  Note over Apr02: Tier 1 wins
  Note over Apr01: test-linux infra
```

Use `as` alias to shorten date participant names. Keep Note text ≤4 words.

### With commit hashes (optional, when available)

```
sequenceDiagram
  participant Apr03 as 2026-04-03
  Note over Apr03: Bench fix 2b6e0d9
```

Include hash only for the most recent entry.

## ER Diagram Patterns

### Item → File relationship

```
erDiagram
  ITEM ||--o{ FILE : references
  ITEM {
    string id
    string priority
    string status
  }
  FILE {
    string path
  }
```

Follow with a compact mapping table:

| Item      | Files                                   |
| --------- | --------------------------------------- |
| minibox-1 | .github/workflows/phased-deployment.yml |
| minibox-4 | crates/daemonbox/src/handler.rs         |

Truncate at 10 rows; add `... N more items` if needed.

## Quadrant Chart Patterns

Only emit when ≥6 items exist. Map:

- X axis: P2 (low) → P0 (high)
- Y axis: done/parked (inactive) → open/blocked (active)

```
quadrantChart
  title Items Status
  x-axis Low Priority --> High Priority
  y-axis Inactive --> Active
  quadrant-1 Act Now
  quadrant-2 Monitor
  quadrant-3 Backlog
  quadrant-4 Done
  Auth Gate: [0.9, 0.9]
  CI Enforce: [0.7, 0.8]
  VZ Bug: [0.4, 0.3]
  Dual License: [0.3, 0.7]
```

Map priorities: P0 → x=0.8–1.0, P1 → x=0.5–0.7, P2 → x=0.2–0.4
Map statuses: open → y=0.6–1.0, blocked → y=0.3–0.5, done/parked → y=0.0–0.2

## Common Mistakes

### Mistake: Long node labels

```
flowchart LR
  A[Add daemon-side policy gate for mounts and privileged containers] --> Open
```

Fix: `A[Auth Gate] --> Open`

### Mistake: Newline in label

```
flowchart LR
  A["Auth\nGate"] --> Open
```

Fix: `A[Auth Gate] --> Open`

### Mistake: Colon in node ID

```
flowchart LR
  minibox-1[Auth Gate] --> Open
```

Fix: `minibox1[Auth Gate] --> Open` (use camelCase or underscore IDs)

### Mistake: Parentheses in label

```
flowchart LR
  A[Auth (Gate)] --> Open
```

Fix: `A[Auth Gate] --> Open`
