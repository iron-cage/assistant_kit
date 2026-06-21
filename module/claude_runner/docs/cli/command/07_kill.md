# CLI Command: kill

Terminate a running Claude Code session by sending SIGTERM to the target process.

**Syntax:**

```sh
clr kill <PID>
clr kill --stale
```

**Parameters:**

| # | Name | Required | Description |
|---|------|----------|-------------|
| 1 | `<PID>` | Yes (unless `--stale`) | Process ID of the Claude Code session to terminate |
| - | `--stale` | No | Kill all print-mode sessions older than `DEFAULT_PRINT_TIMEOUT_SECS` (3600 s) |

**Validation:**

`kill` verifies the given PID belongs to a currently running `claude` process
(via `find_claude_processes()` scanning `/proc`) before sending the signal.
This prevents accidental termination of unrelated processes.

Use `clr ps` to discover active session PIDs.

**Signal:**

Sends `SIGTERM` (graceful termination request).  Claude Code handles SIGTERM
by attempting to finish any in-progress operation before exiting.  For
immediate unconditional termination, use `kill -KILL <PID>` directly.

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | SIGTERM delivered successfully |
| 1 | Error: missing PID, invalid PID format, PID is not a running Claude session, or signal delivery failed |

**Batch Mode — `--stale`:**

`clr kill --stale` identifies all print-mode Claude Code sessions whose elapsed time exceeds `DEFAULT_PRINT_TIMEOUT_SECS` (3600 seconds = 1 hour) and sends SIGTERM to each. Interactive sessions are never targeted — they are user-attended and deliberately have no default timeout.

This provides an external enforcement mechanism for sessions started before the TSK-227 watchdog was deployed, or for sessions whose internal watchdog failed. The threshold is not configurable; it matches `DEFAULT_PRINT_TIMEOUT_SECS` from `src/cli/execution.rs`.

Output: one `"Killed PID <N>"` line per terminated session, followed by a summary count. If no stale sessions are found: `"No stale print-mode sessions found."`.

`--stale` is mutually exclusive with `<PID>`. Passing both exits 1.

**Examples:**

```sh
# Inspect running sessions first
clr ps

# Terminate session with PID 12345
clr kill 12345

# Kill all stale print-mode sessions (> 1 hour)
clr kill --stale

# Show kill-specific help
clr kill --help
```

**Error Messages:**

- `Error: missing PID argument.` — No PID was provided.
- `Error: invalid PID '<value>': must be a positive integer` — Non-numeric argument.
- `Error: PID <N> is not a running Claude Code session.` — PID exists but is not a `claude` process.
- `Error: failed to send SIGTERM to PID <N>: <reason>` — Signal delivery failed (e.g. permission denied).

**Notes:**

`kill` reads `/proc` on Linux to enumerate active `claude` processes.  The
command is not compiled on non-Linux platforms.

The current `clr kill` process itself cannot appear in the active session list
(self-exclusion in `find_claude_processes()`), so attempting `clr kill
<own-pid>` always fails with a "not a running Claude Code session" error.

`clr kil` triggers the "Did you mean 'kill'?" typo guard and exits 1.

### Referenced Parameter Groups

None. `kill` accepts a single positional argument, not named parameters.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 27 | [027_session_termination.md](../user_story/027_session_termination.md) | Developer / CI operator |
