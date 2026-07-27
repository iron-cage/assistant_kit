# CLI Parameter: --category

Filter the `clr tools` tool listing to tools whose category contains the
given substring (case-insensitive). Combines with `--name` using AND logic.

- **Type:** string
- **Default:** — (no filter; all tools shown)
- **Command:** [`tools`](../command/08_tools.md)
- **JSON Key:** — (tools subcommand; not supported by `--args-file`)

```sh
clr tools --category Web                     # tools in the Web category
clr tools --category "File Operations"        # multi-word category (quote it)
clr tools --name cron --category Scheduling   # combine with --name (AND)
```

**Note:** Substring match, not exact match — `--category task` matches both
`Background Tasks` entries; use a longer substring (e.g. `Background Tasks`)
to disambiguate from unrelated categories that happen to share a word.

**Note:** Zero matches is not an error — renders an empty table (or no output
for `--value`/`--inspect`) and exits 0.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 7 | [Tool Listing](../param_group/07_tool_listing.md) | Full | `--name`, `--columns`, `--value`, `--inspect` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 8 | [`tools`](../command/08_tools.md) | — (no filter) | Case-insensitive substring match on tool category |

### Referenced User Stories

*None — tool listing filtering is a CLI ergonomics improvement without a dedicated user story.*
