# Rhai Script Patterns for RLM

Canonical patterns for Rhai scripts the model writes (or that tests exercise).
Each script runs in an isolated `Scope::new()` — no state persists between iterations.

## Rule: Always Ground Before Answering

The system prompt enforces this. A script that calls `final_answer` without first calling
`ctx_grep` or `ctx_slice` will be rejected by the model's own loop constraints.

## Pattern 1: Grep-Then-Answer (minimal correct form)

```rhai
let r = ctx_grep("(?i)author");
final_answer(r);
```

## Pattern 2: Multi-Grep With Fallback

Try progressively broader patterns before calling `final_answer`.

```rhai
let r = ctx_grep("(?i)written by");
if r == "" {
    r = ctx_grep("(?i)author");
}
if r == "" {
    r = ctx_slice(0, 512);
}
final_answer(r);
```

## Pattern 3: Page-Through Large Context

Scan in windows when grep returns nothing useful.

```rhai
let step = 2048;
let total = ctx_len();
let i = 0;
let found = "";
while i < total && found == "" {
    let page = ctx_slice(i, i + step);
    if page.contains("Chapter 3") {
        found = page;
    }
    i += step;
}
final_answer(if found != "" { found } else { ctx_slice(0, 512) });
```

## Pattern 4: Recursive Split-and-Delegate

Divide the context and delegate each half to a child RLM.

```rhai
let mid = ctx_len() / 2;
let a = rlm_call("find the publication date", ctx_slice(0, mid));
let b = rlm_call("find the publication date", ctx_slice(mid, ctx_len()));
final_answer(a + " | " + b);
```

## Pattern 5: Store Hybrid Search (preferred with --store)

```rhai
let id = doc_id();
let results = ctx_hybrid(id, "transformer attention mechanism", 6);
final_answer(results);
```

## Pattern 6: BM25 Keyword Search

Use when `ctx_hybrid` is not available (no embedder / no `OPENAI_API_KEY`).

```rhai
let results = ctx_search(doc_id(), "license terms", 5);
final_answer(results);
```

## Pattern 7: Chunk Enumeration Fallback

When search returns nothing, iterate chunks directly.

```rhai
let id = doc_id();
let n = ctx_chunks(id);
let i = 0;
let found = "";
while i < n && found == "" {
    let c = ctx_chunk(id, i);
    if c.contains("license") {
        found = c;
    }
    i += 1;
}
final_answer(if found != "" { found } else { "not located in chunks" });
```

## Pattern 8: Multi-Step Inspection With print_cell

Use `print_cell` to emit intermediate results to the cell output buffer. The output
is fed back to the model as "Cell output: …" and visible with `--verbose`.

```rhai
print_cell("scanning for author …");
let r = ctx_grep("Author:");
print_cell(r);
let r2 = ctx_grep("(?i)written by");
print_cell(r2);
final_answer(if r != "" { r } else { r2 });
```

## Anti-Patterns to Avoid

**Never call final_answer with empty string:**

```rhai
// BAD — rejected by system prompt rules
final_answer("");
```

**Never skip grounding:**

```rhai
// BAD — no ctx_grep or ctx_slice before final_answer
final_answer("I don't know");
```

**Do not assume state persists:**

```rhai
// BAD — `found` from a previous iteration is NOT available here
// Each script is a fresh Scope::new()
final_answer(found);  // ReferenceError: found not defined
```
