# CLI User Story: Session Listing

### Scope

- **Purpose**: Document `clr ps` as a session inspection tool that lists all running Claude Code processes with per-session metadata in a unicode-box table.
- **Responsibility**: Define acceptance criteria for the session listing command: table output, empty-session state, column presence, help discoverability, typo guard, and self-exclusion.
- **In Scope**: `clr ps` unicode-box table output, `#`/`PID`/`Started`/`CPU%`/`RAM`/`State`/`Absolute Path`/`Task` columns, no-sessions message, `clr --help` listing, typo-guard for `clr p` / `clr pss`, self-PID exclusion, `$PRO` path shortening in Absolute Path column.
- **Out of Scope**: Session termination (kill — future task), filtering by state or path, watch/auto-refresh mode, non-Linux platforms.

### Persona

Developer or CI operator running multiple `clr` sessions who needs a quick
overview of active Claude Code processes — their PIDs, resource usage, working
directories, and current tasks — without reaching for system tools like `pgrep`
or `ps aux`.

### Goal

Inspect all running Claude Code sessions at a glance so that the developer can
understand which sessions are active, how long they have been running, and what
they are doing, enabling them to identify stale or stuck sessions.

### Acceptance Criteria

- AC-001: `clr ps` with ≥1 running Claude processes exits 0 and prints a unicode-box table whose first line contains `┌`
- AC-002: `clr ps` with 0 running Claude processes exits 0 and prints exactly `No active Claude Code sessions.`
- AC-003: `clr --help` and `clr help` list `ps` among the known subcommands
- AC-004: `clr p` triggers the "Did you mean 'ps'?" guard and exits 1 with the message on stderr
- AC-005: The unicode-box table contains column headers `PID`, `Absolute Path`, and `Task`
- AC-006: The `clr ps` process itself is not listed as a row in the table output
- AC-007: When the `PRO` environment variable is set and a session's working directory starts with that path, the `Absolute Path` column shows the path with the `$PRO` prefix replaced by the literal `"$PRO"` string

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 6 | [`ps`](../command/06_ps.md) | The command being specified |

### Referenced Parameter Groups

None. `ps` accepts no parameters.

### Referenced Parameters

None. `ps` accepts no parameters.

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 25 | [025_concurrency_gate.md](025_concurrency_gate.md) | `--max-sessions` counts sessions to gate; `clr ps` lists them for inspection |
| 16 | [016_cli_discoverability.md](016_cli_discoverability.md) | AC-003 verifies `ps` appears in `clr help` output |
