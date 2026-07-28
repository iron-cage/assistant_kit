# CLI Parameter: --name

Filter the `clr tools` tool listing to tools whose name contains the given
substring (case-insensitive). Combines with `--category` using AND logic.

- **Type:** string
- **Default:** — (no filter; all tools shown)
- **Command:** [`tools`](../command/08_tools.md)
- **JSON Key:** — (tools subcommand; not supported by `--args-file`)

```sh
clr tools --name task                       # tools whose name contains "task" (case-insensitive)
clr tools --name cron --category Scheduling # combine with --category (AND)
```

**Note:** Substring match, not exact match — `--name task` matches `TaskCreate`,
`TaskGet`, `TaskList`, `TaskOutput`, `TaskStop`, and `TaskUpdate`.

**Note:** Zero matches is not an error — renders an empty table (or no output
for `--value`/`--inspect`) and exits 0.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 7 | [Tool Listing](../param_group/07_tool_listing.md) | Full | `--category`, `--columns`, `--value`, `--inspect` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 8 | [`tools`](../command/08_tools.md) | — (no filter) | Case-insensitive substring match on tool name |

### Referenced User Stories

*None — tool listing filtering is a CLI ergonomics improvement without a dedicated user story.*
