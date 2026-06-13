# CLI Command: kill

Terminate a running Claude Code session by sending SIGTERM to the target process.

**Syntax:**

```sh
clr kill <PID>
```

**Parameters:**

| # | Name | Required | Description |
|---|------|----------|-------------|
| 1 | `<PID>` | Yes | Process ID of the Claude Code session to terminate |

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

**Examples:**

```sh
# Inspect running sessions first
clr ps

# Terminate session with PID 12345
clr kill 12345

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
