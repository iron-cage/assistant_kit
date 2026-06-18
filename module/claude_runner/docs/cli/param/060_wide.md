# CLI Parameter: --wide

Show all available columns in the `clr ps` active sessions table, including the
optional `mode`, `cmd`, and `binary` columns that are hidden by default.

- **Type:** bool
- **Default:** false
- **Command:** [`ps`](../command/06_ps.md)

```sh
clr ps --wide            # show all 11 columns
clr ps -w                # short form
clr ps -w --mode print   # wide output filtered to print-mode sessions
```

**Note:** `--wide` is a convenience shorthand for
`--columns idx,pid,elapsed,cpu,ram,state,path,task,mode,cmd,binary`.

**Note:** When `--columns` is also specified, `--columns` wins — explicit column
selection overrides the `--wide` flag.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Session Listing](../param_group/05_session_listing.md) | Full | `--mode`, `--columns` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`ps`](../command/06_ps.md) | false | Expands active sessions table to all columns |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
