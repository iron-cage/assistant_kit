# Group :: 5. Display Control

**Parameters:** `cols::`
**Pattern:** Per-invocation column visibility for the `.usage` quota table
**Purpose:** Controls which columns appear in the text-format quota table.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`cols::`](../param/033_cols.md) | `string` | (default set) | Comma-separated `+col_id` / `-col_id` modifiers relative to the default column set |

**Used By (1 command):** [`.usage`](../command/006_usage.md#command--9-usage)

**Typical Patterns:**

```bash
# Show Sub column (off by default)
clp .usage cols::+sub

# Show Sub and 7d Son Reset (both off by default)
clp .usage cols::+sub,+7d_son_reset

# Hide Renews and 7d(Son) from the default set
clp .usage cols::-renews,-7d_son

# Show Sub and hide 7d(Son) simultaneously
clp .usage cols::+sub,-7d_son
```

**Semantic Coherence Test**

> "Does parameter X control **which columns appear** in the `.usage` text-format quota table?"

`cols::` passes: it selects visible columns via `+col_id` / `-col_id` modifiers. All other `.usage` parameters (sort::, desc::, prefer::, next::, format::, live::) fail — they control ordering, recommendation strategy, serialization format, or fetch behavior, not column visibility.

**Invariants**

- The `flag` (first column) and `account` (name) columns are structural and always visible — `cols::` modifiers cannot remove them.
- `cols::` has no effect on `format::json` output — JSON always includes all schema fields. See [../004_parameter_interactions.md](../004_parameter_interactions.md) Interaction 8.
- `Sub` and `7d Son Reset` columns are off by default; all other quota columns are on by default.
- Invalid column IDs cause exit 1 with an error naming the valid column IDs.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — Interaction 8: `cols::` does not affect `format::json` output
- [../../feature/009_token_usage.md](../../feature/009_token_usage.md) — column definitions, AC-22, AC-23; three-tier grouping
- [../param/033_cols.md](../param/033_cols.md) — complete column registry with IDs and defaults
