# CLI User Story: Session Listing

### Scope

- **Purpose**: Document `clr ps` as a session inspection tool that lists all running Claude Code processes and queued `clr` waiters with per-session metadata in two plain-style tables.
- **Responsibility**: Define acceptance criteria for the session listing command: plain-style table output, elapsed column, queued-sessions table, empty-session state, column presence, help discoverability, typo guard, self-exclusion, mode filtering, column selection, and wide output.
- **In Scope**: `clr ps` plain-style table output, default 8 columns (`#`/`PID`/`Elapsed`/`CPU%`/`RAM`/`State`/`Absolute Path`/`Task`), optional 3 columns (`Mode`/`Command`/`Binary`), queued CLR processes table (`#`/`PID`/`CWD`/`Waiting`/`Attempt` columns), no-sessions message, `clr --help` listing, typo-guard for `clr p` / `clr pss`, self-PID exclusion, `$PRO` path shortening, gate state files, `CLR_GATE_DIR` override, `--mode` filtering, `--columns` selection, `--wide` expansion, `CLR_PS_MODE` / `CLR_PS_COLUMNS` env var fallbacks.
- **Out of Scope**: Watch/auto-refresh mode, non-Linux platforms. (Session termination implemented as `clr kill` in US-027 / TSK-201.)

### Persona

Developer or CI operator running multiple `clr` sessions who needs a quick
overview of active Claude Code processes — their PIDs, resource usage, working
directories, and current tasks — without reaching for system tools like `pgrep`
or `ps aux`.

### Goal

Inspect all running Claude Code sessions and queued `clr` waiters at a glance so
that the developer can understand which sessions are active, how long they have been
running, what they are doing, and whether any `clr` processes are blocked waiting
for a session slot — with full control over which rows and columns are displayed —
enabling them to identify stale, stuck, or piled-up sessions.

### Acceptance Criteria

- AC-001: `clr ps` with ≥1 running Claude processes exits 0 and prints a plain-style table containing the column header `PID` without `┌` border characters
- AC-002: `clr ps` with 0 running Claude processes and no queued processes exits 0 and prints exactly `No active Claude Code sessions.`
- AC-003: `clr --help` and `clr help` list `ps` among the known subcommands
- AC-004: `clr p` triggers the "Did you mean 'ps'?" guard and exits 1 with the message on stderr
- AC-005: The active sessions table contains column headers `PID`, `Elapsed`, `Absolute Path`, and `Task`
- AC-006: The `clr ps` process itself is not listed as a row in the table output
- AC-007: When the `PRO` environment variable is set and a session's working directory starts with that path, the `Absolute Path` column shows the path with the `$PRO` prefix replaced by the literal `"$PRO"` string
- AC-008: When one or more gate state files exist in `$CLR_GATE_DIR`, `clr ps` prints a queued CLR processes table containing column headers `PID`, `CWD`, and `Waiting`
- AC-009: When no gate state files exist, `clr ps` output does not contain a queued processes table
- AC-010: Each table rendered by `clr ps` is preceded by a titled caption rule line: the active sessions table shows `Active Sessions · N running` and the queued processes table shows `Queued · N waiting`
- AC-011: `clr ps --help` and `clr ps -h` print subcommand help to stdout and exit 0; the positional token `clr ps help` does the same
- AC-012: Active session rows are ordered by session start time (oldest first); when two or more sessions exist, the row with the longest elapsed time appears at row `#1`
- AC-013: `clr ps --mode interactive` shows only sessions without `--print`/`-p` in their cmdline arguments; `clr ps --mode print` shows only sessions with `--print`/`-p`; `clr ps --mode all` (default) shows both
- AC-014: `clr ps --mode bogus` exits 1 with an error message listing valid mode values on stderr
- AC-015: `clr ps --columns pid,path,task` shows exactly the specified columns in the specified order; column headers match the requested keys
- AC-016: `clr ps --columns bogus` exits 1 with an error message listing valid column keys on stderr
- AC-017: `clr ps --wide` shows all 11 columns including `Mode`, `Command`, and `Binary`
- AC-018: When both `--columns` and `--wide` are specified, `--columns` wins (explicit selection overrides the convenience flag)
- AC-019: `CLR_PS_MODE=print clr ps` filters to print-mode sessions (env var fallback); `clr ps --mode interactive` with `CLR_PS_MODE=print` shows interactive sessions only (CLI wins)
- AC-020: `CLR_PS_COLUMNS=pid,elapsed clr ps` shows only PID and Elapsed columns (env var fallback); `clr ps --columns pid,path` with `CLR_PS_COLUMNS=pid,elapsed` shows PID and Path (CLI wins)

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 6 | [`ps`](../command/06_ps.md) | The command being specified |

### Referenced Parameter Groups

| # | Group | Params |
|---|-------|--------|
| 5 | [Session Listing](../param_group/05_session_listing.md) | `--mode`, `--columns`, `--wide` |

### Referenced Parameters

| # | Parameter | AC |
|---|-----------|-----|
| 58 | [`--mode`](../param/058_mode.md) | AC-013, AC-014, AC-019 |
| 59 | [`--columns`](../param/059_columns.md) | AC-015, AC-016, AC-018, AC-020 |
| 60 | [`--wide`](../param/060_wide.md) | AC-017, AC-018 |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 25 | [025_concurrency_gate.md](025_concurrency_gate.md) | `--max-sessions` counts sessions to gate; `clr ps` lists them for inspection |
| 27 | [027_session_termination.md](027_session_termination.md) | `clr ps` discovers session PIDs; `clr kill` terminates them |
| 16 | [016_cli_discoverability.md](016_cli_discoverability.md) | AC-003 verifies `ps` appears in `clr help` output |
