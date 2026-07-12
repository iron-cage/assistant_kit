# CLI Parameter: --value

Print only the named column's value for each matching tool, one per line,
with no table formatting or heading. Intended for scripting and single-cell
extraction — combine with `--name`/`--category` to narrow to one row for a
true single-cell result.

- **Type:** string
- **Default:** — (table format; `--value` unset)
- **Command:** [`tools`](../command/08_tools.md)
- **JSON Key:** — (tools subcommand; not supported by `--args-file`)

```sh
clr tools --value name                       # every tool name, one per line
clr tools --category Web --value name        # names of tools in the Web category
clr tools --name Bash --value category       # single cell: "Shell" (one matching tool, one column)
```

**Accepted keys:** `idx`, `name`, `category`, `desc` (same vocabulary as `--columns`).

**Note:** `--value` is mutually exclusive with `--inspect` — specifying both
is an error (exit 1).

**Note:** `--columns` is ignored when `--value` is active (single-column
output overrides multi-column projection).

**Note:** Zero matches is not an error — produces no output and exits 0.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 7 | [Tool Listing](../param_group/07_tool_listing.md) | Full | `--name`, `--category`, `--columns`, `--inspect` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 8 | [`tools`](../command/08_tools.md) | — (table format) | Bare single-column output, one value per line |

### Referenced User Stories

*None — tool listing filtering is a CLI ergonomics improvement without a dedicated user story.*
