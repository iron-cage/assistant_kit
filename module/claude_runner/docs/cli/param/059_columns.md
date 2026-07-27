# CLI Parameter: --columns

Select which columns to display in the `clr ps` active sessions table or the
`clr tools` tool listing table. Accepts a comma-separated list of column keys.
Columns are rendered in the order specified. Available keys and defaults are
command-specific — see the two Variant Tables below.

- **Type:** comma-separated string
- **Default:** `idx,pid,elapsed,cpu,ram,state,mode,path,task` (`ps`); `idx,name,category,desc` (`tools`)
- **Command:** [`ps`](../command/06_ps.md), [`tools`](../command/08_tools.md)
- **JSON Key:** — (ps/tools subcommand; not supported by `--args-file`)

```sh
clr ps --columns pid,path,mode,task   # ps: custom subset in specified order
clr ps --columns pid,elapsed,cmd      # ps: minimal with full command line
CLR_PS_COLUMNS=pid,path,task clr ps   # ps: env-var equivalent
clr tools --columns name,category     # tools: narrow to two columns
clr tools --columns name              # tools: single column (still table-formatted; see --value for bare output)
```

**Available column keys — `ps`:**

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
| `mode` | Mode | cmdline: `--print`/`-p` present → `print`, else `interactive` | yes |
| `cmd` | Command | cmdline args[1..] joined | no |
| `binary` | Binary | cmdline args[0] (executable path) | no |

**Available column keys — `tools`:**

| Key | Header | Source | Default |
|-----|--------|--------|---------|
| `idx` | # | Counter | yes |
| `name` | Tool | `TOOLS` entry field 1 | yes |
| `category` | Category | `TOOLS` entry field 2 | yes |
| `desc` | Description | `TOOLS` entry field 3 | yes |

**Note:** Unknown column keys cause the invoking command to exit 1 with an
error message listing valid keys to stderr.

**Note:** The `idx` column counter always reflects the visible row number
(1-based), regardless of filtering (`--mode` for `ps`; `--name`/`--category`
for `tools`).

**Note:** When `--columns` and `--wide` are both specified on `ps`, `--columns`
wins (explicit selection overrides the convenience flag). `tools` has no
`--wide` flag — its default view already includes all 4 available columns;
use `--columns` alone to narrow.

**Note:** On `tools`, `--columns` is ignored when `--value` or `--inspect` is
active (both switch to a non-table output mode).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Session Listing](../param_group/05_session_listing.md) | Full | `--mode`, `--wide` (ps context) |
| 7 | [Tool Listing](../param_group/07_tool_listing.md) | Full (shared) | `--name`, `--category`, `--value`, `--inspect` (tools context) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`ps`](../command/06_ps.md) | 9 default columns | Controls active sessions table columns; queued table columns are fixed |
| 8 | [`tools`](../command/08_tools.md) | 4 default columns (all) | Controls tool listing table columns; ignored when `--value`/`--inspect` active |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
