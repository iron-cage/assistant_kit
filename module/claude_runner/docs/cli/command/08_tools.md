# CLI Command: tools

### Description

List all Claude Code built-in tools with name, category, and description. Use `clr tools` to discover available tools before constructing `--allowed-tools` or `--disallowed-tools` flags for a `run` invocation. Supports filtering by name/category, column projection, single-column value extraction, and a key:value record format for inspecting all fields at once.

-- **Exit Codes:** 0 (success, including zero matches after filtering); 1 (`--value` and `--inspect` both specified; unknown `--columns`/`--value` key; unexpected argument)

### Syntax

```sh
clr tools [--name SUBSTRING] [--category SUBSTRING] [--columns KEYS | --value KEY | --inspect]
```

### Parameters

| Flag | Type | Default | Description |
|------|------|---------|--------------|
| [`--name`](../param/078_name.md) | string | — | Filter by tool name (case-insensitive substring match) |
| [`--category`](../param/079_category.md) | string | — | Filter by category (case-insensitive substring match) |
| [`--columns`](../param/059_columns.md) | string | `idx,name,category,desc` | Comma-separated column keys to display in table format |
| [`--value`](../param/080_value.md) | string | — | Print only the named column's value, one per line, no table decoration |
| [`--inspect`](../param/069_inspect.md) | bool | false | Switch to key:value record format (all fields, one block per tool) |

Universal help flags (`--help`, `-h`) are also supported.

**Combining rules:** `--name` and `--category` combine with AND logic (both must match). `--value` and `--inspect` are mutually exclusive output-format switches — specifying both is an error (exit 1). `--columns` is ignored when `--value` or `--inspect` is active.

**Algorithm (4 steps):** (1) Read tool definitions from `contract/claude_code/docs/tool/`; (2) apply `--name`/`--category` filters (AND, case-insensitive substring) if present; (3) select output mode — table (default, honoring `--columns`), single-column bare value list (`--value`), or key:value record blocks (`--inspect`); (4) render and exit 0. Zero matching tools after filtering is not an error — renders an empty table (or no output for `--value`/`--inspect`) and still exits 0.

### Examples

```sh
# List all Claude Code tools
clr tools

# Filter by category
clr tools --category "File Operations"

# Filter by name substring
clr tools --name task

# Combine filters (AND)
clr tools --name cron --category Scheduling

# Narrow columns
clr tools --columns name,category

# Print only tool names, one per line
clr tools --value name

# Print a single cell: the category of one tool
clr tools --name Bash --value category

# Full key:value record for every tool
clr tools --inspect

# Pipe to grep for a specific tool
clr tools | grep "Bash"
```

### Notes

Tool data is sourced from the contract documentation at `contract/claude_code/docs/tool/`. The table is rendered in plain style matching `clr ps` output conventions. The hardcoded `TOOLS` array in `tools.rs` must be kept in sync with the contract doc's tool count and names — see [invariant/015](../../invariant/015_tools_array_doc_sync.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`run`](01_run.md) | `--allowed-tools` and `--disallowed-tools` flags reference tool names listed here |
| 2 | [`help`](02_help.md) | Complementary discovery: `help` lists commands; `tools` lists tools |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 7 | [Tool Listing](../param_group/07_tool_listing.md) | Full |

### Referenced User Stories

*None — tool listing filtering is a CLI ergonomics improvement without a dedicated user story.*

---

**Category:** CLI discoverability
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
