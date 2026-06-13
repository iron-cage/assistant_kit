# CLI User Story: Session Listing

### Scope

- **Purpose**: Document `clr ps` as a session inspection tool that lists all running Claude Code processes and queued `clr` waiters with per-session metadata in two plain-style tables.
- **Responsibility**: Define acceptance criteria for the session listing command: plain-style table output, elapsed column, queued-sessions table, empty-session state, column presence, help discoverability, typo guard, and self-exclusion.
- **In Scope**: `clr ps` plain-style table output, `#`/`PID`/`Elapsed`/`CPU%`/`RAM`/`State`/`Absolute Path`/`Task` columns, queued CLR processes table (`#`/`PID`/`CWD`/`Waiting`/`Attempt` columns), no-sessions message, `clr --help` listing, typo-guard for `clr p` / `clr pss`, self-PID exclusion, `$PRO` path shortening in Absolute Path and CWD columns, gate state files written by `gate.rs`, `CLR_GATE_DIR` override.
- **Out of Scope**: Session termination (kill — future task), filtering by state or path, watch/auto-refresh mode, non-Linux platforms.

### Persona

Developer or CI operator running multiple `clr` sessions who needs a quick
overview of active Claude Code processes — their PIDs, resource usage, working
directories, and current tasks — without reaching for system tools like `pgrep`
or `ps aux`.

### Goal

Inspect all running Claude Code sessions and queued `clr` waiters at a glance so
that the developer can understand which sessions are active, how long they have been
running, what they are doing, and whether any `clr` processes are blocked waiting
for a session slot — enabling them to identify stale, stuck, or piled-up sessions.

### Acceptance Criteria

- AC-001: `clr ps` with ≥1 running Claude processes exits 0 and prints a plain-style table whose first line contains the header `PID` without `┌` border characters
- AC-002: `clr ps` with 0 running Claude processes and no queued processes exits 0 and prints exactly `No active Claude Code sessions.`
- AC-003: `clr --help` and `clr help` list `ps` among the known subcommands
- AC-004: `clr p` triggers the "Did you mean 'ps'?" guard and exits 1 with the message on stderr
- AC-005: The active sessions table contains column headers `PID`, `Elapsed`, `Absolute Path`, and `Task`
- AC-006: The `clr ps` process itself is not listed as a row in the table output
- AC-007: When the `PRO` environment variable is set and a session's working directory starts with that path, the `Absolute Path` column shows the path with the `$PRO` prefix replaced by the literal `"$PRO"` string
- AC-008: When one or more gate state files exist in `$CLR_GATE_DIR`, `clr ps` prints a queued CLR processes table containing column headers `PID`, `CWD`, and `Waiting`
- AC-009: When no gate state files exist, `clr ps` output does not contain a queued processes table

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
