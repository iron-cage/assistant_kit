# CLI Command: kill

### Description

Terminate a running Claude Code session by sending SIGTERM to the target process. Use `clr kill <PID>` after `clr ps` to stop a stuck, stale, or unwanted session with confidence that only a verified `claude` process will be targeted.

-- **Parameters:** `<PID>`
-- **Exit Codes:** 0 (SIGTERM delivered) | 1 (error)

### Syntax

```sh
clr kill <PID>
```

### Parameters

| # | Name | Required | Purpose |
|---|------|----------|---------|
| 1 | `<PID>` | Yes | Process ID of the Claude Code session to terminate |

**Algorithm (4 steps):**
1. Validate `<PID>` is present and a positive integer; exit 1 with error message on failure.
2. Scan `/proc` via `find_claude_processes()` to verify PID belongs to a running `claude` process.
3. Send `SIGTERM` to the target process.
4. Print `"Sent SIGTERM to Claude Code session <PID>."` and exit 0; on signal failure, exit 1 with error.

### Examples

```sh
# Inspect running sessions first
clr ps

# Terminate session with PID 12345
clr kill 12345

# Show kill-specific help
clr kill --help
```

### Notes

`kill` verifies the given PID belongs to a currently running `claude` process (via `find_claude_processes()` scanning `/proc`) before sending the signal. This prevents accidental termination of unrelated processes. Use `clr ps` to discover active session PIDs.

Sends `SIGTERM` (graceful termination request). Claude Code handles SIGTERM by attempting to finish any in-progress operation before exiting. For immediate unconditional termination, use `kill -KILL <PID>` directly.

`kill` reads `/proc` on Linux to enumerate active `claude` processes. The command is not compiled on non-Linux platforms.

The current `clr kill` process itself cannot appear in the active session list (self-exclusion in `find_claude_processes()`), so attempting `clr kill <own-pid>` always fails with a "not a running Claude Code session" error.

`clr kil` triggers the "Did you mean 'kill'?" typo guard and exits 1.

**Error messages:**
- `Error: missing PID argument.` — No PID was provided.
- `Error: invalid PID '<value>': must be a positive integer` — Non-numeric argument.
- `Error: PID <N> is not a running Claude Code session.` — PID exists but is not a `claude` process.
- `Error: failed to send SIGTERM to PID <N>: <reason>` — Signal delivery failed (e.g. permission denied).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`ps`](06_ps.md) | Discovers active session PIDs used as input to `kill` |

### Referenced Parameter Groups

None. `kill` accepts a single positional argument, not named parameters.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 27 | [027_session_termination.md](../user_story/027_session_termination.md) | Developer / CI operator |

---

**Category:** Session management
**Complexity:** 3
**API Requirement:** None
**Idempotent:** No
**Risk Level:** High
