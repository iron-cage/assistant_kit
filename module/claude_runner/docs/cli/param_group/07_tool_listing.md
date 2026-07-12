# CLI Parameter Group: Tool Listing

**Pattern:** Consumed by `dispatch_tools()` to filter rows, select columns, and control output format in the `clr tools` tool listing table. Not forwarded to any subprocess.

**Purpose:** Control tool listing display — filter by name, filter by category, select visible columns, extract a single column's bare values, or switch to key:value inspect format.
**Order:** 7

### Semantic Coherence Test

"Is this flag consumed by `clr tools`, not by `run`/`ask` or the claude subprocess?" — YES for all 5.

### Why NOT Runner Control

- `--name`, `--category`, `--columns`, `--value`, `--inspect`: apply only to `clr tools`; never affect subprocess execution, retry behavior, or command construction — they are output display controls for the tool listing command exclusively.

### Invariants

All 5 parameters are consumed by `dispatch_tools()` in `tools.rs` before table rendering. None affect subprocess execution or command construction.

### Notes

`--columns` and `--inspect` are shared with the [Session Listing](05_session_listing.md) group — each parameter's own doc file (`059_columns.md`, `069_inspect.md`) documents both commands' vocabularies via a per-command Variant Table. `--name`, `--category`, and `--value` are exclusive to this group.

### Typical Patterns

```sh
clr tools
clr tools --category Web
clr tools --name task --category Background
clr tools --columns name,category
clr tools --value name
clr tools --name Bash --inspect
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 8 | [`tools`](../command/08_tools.md) | Full | — | All 5 params apply; tool listing command |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--name`](../param/078_name.md) | string | — | Row filter | Filter tools by name (case-insensitive substring) |
| [`--category`](../param/079_category.md) | string | — | Row filter | Filter tools by category (case-insensitive substring) |
| [`--columns`](../param/059_columns.md) | string | 4 default cols | Column selector | Comma-separated list of column keys to display |
| [`--value`](../param/080_value.md) | string | — | Output mode | Print one column's bare values, one per line |
| [`--inspect`](../param/069_inspect.md) | bool | false | Output mode | Switch to key:value record format showing all 4 attributes |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 7 | [07_tool_listing.md](../../../tests/docs/cli/param_group/07_tool_listing.md) | Tool Listing group behavior |

### Referenced User Stories

*None — tool listing filtering is a CLI ergonomics improvement without a dedicated user story.*
