# CLI Parameter: --columns

Select which columns to display in the `clr ps` active sessions table. Accepts a
comma-separated list of column keys. Columns are rendered in the order specified.

- **Type:** comma-separated string
- **Default:** `idx,pid,elapsed,cpu,ram,state,path,task`
- **Command:** [`ps`](../command/06_ps.md)

```sh
clr ps --columns pid,path,mode,task   # custom subset in specified order
clr ps --columns pid,elapsed,cmd      # minimal with full command line
CLR_PS_COLUMNS=pid,path,task clr ps   # env-var equivalent
```

**Available column keys:**

| Key | Header | Source | Default |
|-----|--------|--------|---------|
| `idx` | # | Counter | yes |
| `pid` | PID | `ProcessInfo.pid` | yes |
| `elapsed` | Elapsed | `/proc/{pid}/stat` field 22 | yes |
| `cpu` | CPU% | `/proc/{pid}/stat` utime+stime | yes |
| `ram` | RAM | `/proc/{pid}/status` VmRSS | yes |
| `state` | State | `/proc/{pid}/stat` field 3 | yes |
| `path` | Path | `/proc/{pid}/cwd` ($PRO shortened) | yes |
| `task` | Task | Session JSONL last user message | yes |
| `mode` | Mode | cmdline: `--print`/`-p` present → `print`, else `interactive` | no |
| `cmd` | Command | cmdline args[1..] joined | no |
| `binary` | Binary | cmdline args[0] (executable path) | no |

**Note:** Unknown column keys cause `clr ps` to exit 1 with an error message
listing valid keys to stderr.

**Note:** The `idx` column counter always reflects the visible row number
(1-based), regardless of filtering by `--mode`.

**Note:** When `--columns` and `--wide` are both specified, `--columns` wins
(explicit selection overrides the convenience flag).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Session Listing](../param_group/05_session_listing.md) | Full | `--mode`, `--wide` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`ps`](../command/06_ps.md) | 8 default columns | Controls active sessions table columns; queued table columns are fixed |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
