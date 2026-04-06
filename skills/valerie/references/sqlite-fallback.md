# SQLite Fallback — Schema and Wrapper

When the backend is `sqlite`, valerie scaffolds a thin wrapper that mirrors the doob
command surface. The wrapper lives at `~/.local/bin/doob-sqlite` (sh) or
`~/.local/bin/doob-sqlite.nu` (nu).

## Schema

```sql
CREATE TABLE IF NOT EXISTS todos (
    id          TEXT PRIMARY KEY,   -- random UUID
    content     TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending',
    priority    INTEGER DEFAULT 1,
    project     TEXT,
    tags        TEXT,               -- comma-separated
    due         TEXT,               -- YYYY-MM-DD or NULL
    created_at  TEXT NOT NULL       -- ISO 8601
);
```

DB path: `~/.local/share/valerie/todos.db`

## JSON Output Shape (mirrors doob --json)

```json
{
  "count": 1,
  "todos": [
    {
      "id": "abc123",
      "content": "Fix auth bug",
      "status": "pending",
      "priority": 3,
      "project": "myproject",
      "tags": ["bug", "urgent"],
      "due": null,
      "created_at": "2026-04-06T10:00:00Z"
    }
  ]
}
```

## Wrapper Commands (sh)

Scaffold at `~/.local/bin/doob` when backend=sqlite:

```sh
#!/bin/sh
# doob-sqlite — sqlite wrapper mimicking doob CLI surface
DB="$HOME/.local/share/valerie/todos.db"
sqlite3 "$DB" "CREATE TABLE IF NOT EXISTS todos (
  id TEXT PRIMARY KEY, content TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending',
  priority INTEGER DEFAULT 1, project TEXT, tags TEXT, due TEXT, created_at TEXT NOT NULL
);" 2>/dev/null

CMD="$1"; shift
case "$CMD" in
  todo)
    SUBCMD="$1"; shift
    case "$SUBCMD" in
      add)
        # parse --priority, --project, --tags from "$@"
        CONTENT="$1"; shift
        PRIORITY=1; PROJECT=""; TAGS=""
        while [ $# -gt 0 ]; do
          case "$1" in
            --priority) PRIORITY="$2"; shift 2 ;;
            --project|-p) PROJECT="$2"; shift 2 ;;
            --tags|-t) TAGS="$2"; shift 2 ;;
            *) shift ;;
          esac
        done
        ID=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || date +%s%N)
        NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        sqlite3 "$DB" "INSERT INTO todos VALUES ('$ID','$CONTENT','pending',$PRIORITY,'$PROJECT','$TAGS',NULL,'$NOW');"
        echo "Created: $CONTENT"
        ;;
      list)
        STATUS="pending"; PROJECT_FILTER=""
        while [ $# -gt 0 ]; do
          case "$1" in
            --status) STATUS="$2"; shift 2 ;;
            --project|-p) PROJECT_FILTER="AND project='$2'"; shift 2 ;;
            *) shift ;;
          esac
        done
        sqlite3 -json "$DB" "SELECT * FROM todos WHERE status='$STATUS' $PROJECT_FILTER ORDER BY priority DESC, created_at ASC;"
        ;;
      complete)
        for ID in "$@"; do
          sqlite3 "$DB" "UPDATE todos SET status='completed' WHERE id='$ID';"
        done
        echo "Completed $# todo(s)"
        ;;
      remove)
        for ID in "$@"; do
          sqlite3 "$DB" "DELETE FROM todos WHERE id='$ID';"
        done
        echo "Removed $# todo(s)"
        ;;
      undo)
        for ID in "$@"; do
          sqlite3 "$DB" "UPDATE todos SET status='pending' WHERE id='$ID';"
        done
        echo "Undone $# todo(s)"
        ;;
      due)
        ID="$1"; DATE="$2"
        if [ "$DATE" = "clear" ]; then DATE="NULL"; else DATE="'$DATE'"; fi
        sqlite3 "$DB" "UPDATE todos SET due=$DATE WHERE id='$ID';"
        ;;
    esac
    ;;
esac
```

## Limitations vs doob

| Feature | doob | sqlite fallback |
|---------|------|----------------|
| SurrealKV embedded | yes | no |
| Batch add | yes | single item only |
| Dependency tracking | yes | no |
| HANDOFF sync | yes | no |
| Full-text search | yes | sqlite LIKE only |
| `--json` flag | yes | sqlite3 -json |
| Archive | yes | no |

The sqlite fallback covers the core add/list/complete/remove/undo/due surface
sufficient for valerie's workflows. Missing features are noted when encountered.
