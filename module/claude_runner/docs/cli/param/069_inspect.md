# CLI Parameter: --inspect

Switch `clr ps` or `clr tools` output to key:value record format, showing all
available attributes for each session or tool as a multi-line block rather
than a table row.

- **Type:** bool
- **Default:** false
- **Command:** [`ps`](../command/06_ps.md), [`tools`](../command/08_tools.md)
- **JSON Key:** — (ps/tools subcommand; not supported by `--args-file`)

```sh
clr ps --inspect                    # inspect blocks for all sessions
clr ps -i                           # short form
clr ps --pid 1234567 --inspect      # inspect specific session
clr ps --mode print --inspect       # inspect all print-mode sessions
clr tools --inspect                 # inspect blocks for all tools
clr tools --category Web --inspect  # inspect blocks for tools in one category
```

**Output format — `ps`:**

One record block per session, separated by a blank line. Each block starts with a
separator line containing the PID, followed by left-aligned `key: value` pairs
with the values column-aligned:

```
──── PID 1234567 ─────────────────────────────────────────────
pid:      1234567
mode:     interactive
elapsed:  72h 18m
cpu:      0.6%
ram:      96M
state:    S
path:     $PRO/lib/wip_core/agent_kit/claude_runner/module/claude_runner/src
task:     explore current crate, its documents
binary:   /usr/local/bin/claude
cmd:      --effort max --dangerously-skip-permissions
cmdline:  /usr/local/bin/claude --effort max --dangerously-skip-permissions
started:  1750170000
```

**Attribute list (always shown in inspect mode, in display order):**

| Key | Description | Source |
|-----|-------------|--------|
| `pid` | Process ID | `ProcessInfo.pid` |
| `mode` | `interactive` or `print` | cmdline `--print`/`-p` detection |
| `elapsed` | Time since session started | `/proc/{pid}/stat` field 22 |
| `cpu` | Lifetime-average CPU percentage | `/proc/{pid}/stat` utime+stime |
| `ram` | Resident memory (K or M suffix) | `/proc/{pid}/status` VmRSS |
| `state` | Process state character | `/proc/{pid}/stat` field 3 |
| `path` | Working directory (`$PRO` prefix shortened when PRO is set) | `/proc/{pid}/cwd` |
| `task` | Last Form A user message from session JSONL (≤35 chars) | `~/.claude/projects/` JSONL |
| `binary` | Full executable path (args[0]) | `/proc/{pid}/cmdline` field 0 |
| `cmd` | Arguments after the binary (args[1..] joined with spaces) | `/proc/{pid}/cmdline` |
| `cmdline` | Full raw cmdline (all args joined with spaces) | `/proc/{pid}/cmdline` |
| `started` | Session start time as Unix epoch seconds | `/proc/{pid}/stat` field 22 |

**Interaction with other flags — `ps`:**

- `--pid`: inspect mode respects the `--pid` filter — only specified PIDs produce blocks.
- `--mode`: inspect mode respects the `--mode` filter — only matching execution-mode sessions produce blocks.
- `--columns`: ignored in inspect mode (all 12 attributes are always shown).
- `--wide`: ignored in inspect mode (all 12 attributes are always shown).
- **Queued table:** The Queued CLR Processes table is not shown when `--inspect` is active.

**Output format — `tools`:**

One record block per matching tool, separated by a blank line. Each block
starts with a separator line containing the tool name, followed by
left-aligned `key: value` pairs:

```
──── Tool Bash ─────────────────────────────────────────────
idx:      4
name:     Bash
category: Shell
desc:     Execute shell commands with timeout control
```

**Attribute list — `tools`** (always shown, in display order):

| Key | Description | Source |
|-----|-------------|--------|
| `idx` | Visible row number after filtering (1-based) | Counter |
| `name` | Tool name | `TOOLS` entry field 1 |
| `category` | Tool category | `TOOLS` entry field 2 |
| `desc` | Tool description | `TOOLS` entry field 3 |

**Interaction with other flags — `tools`:**

- `--name`/`--category`: inspect mode respects both filters — only matching tools produce blocks.
- `--columns`: ignored in inspect mode (all 4 attributes are always shown).
- `--value`: mutually exclusive with `--inspect` — specifying both is an error (exit 1).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Session Listing](../param_group/05_session_listing.md) | Full | `--mode`, `--columns`, `--wide`, `--pid` (ps context) |
| 7 | [Tool Listing](../param_group/07_tool_listing.md) | Full (shared) | `--name`, `--category`, `--columns`, `--value` (tools context) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`ps`](../command/06_ps.md) | false | Switches output to key:value record format; suppresses queued table |
| 8 | [`tools`](../command/08_tools.md) | false | Switches output to key:value record format; mutually exclusive with `--value` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
