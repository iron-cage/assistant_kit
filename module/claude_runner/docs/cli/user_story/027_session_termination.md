# CLI User Story: Session Termination

### Scope

- **Purpose**: Document `clr kill <PID>` as a session management tool that sends SIGTERM to a
  running Claude Code process identified by its PID.
- **Responsibility**: Define acceptance criteria for the session termination command: PID
  validation, SIGTERM delivery, non-Claude PID rejection, missing PID error, help
  discoverability, and typo guard.
- **In Scope**: `clr kill <PID>` command, SIGTERM delivery, PID validation via
  `find_claude_processes()`, error handling for invalid/missing/non-Claude PIDs, `clr
  --help` listing, typo guard for `clr kil`.
- **Out of Scope**: SIGKILL support (escalation), kill-all mode, kill by path/pattern,
  non-Linux platforms.

### Persona

Developer or CI operator running multiple `clr` sessions who needs to cleanly stop a
specific Claude Code process — identified by its PID from `clr ps` — without resorting
to system-level `kill` commands and without risking terminating the wrong process.

### Goal

Terminate a specific Claude Code session by PID so the developer can free up a session
slot, stop a stuck task, or cancel an unwanted run — with confidence that only a verified
`claude` process will be targeted.

### Acceptance Criteria

- AC-001: `clr kill <PID>` with a PID belonging to a running Claude Code process exits 0 and prints `"Sent SIGTERM to Claude Code session <PID>."`
- AC-002: `clr kill <PID>` with a PID that is not a running Claude Code process exits 1 and stderr contains the PID and a message indicating it is not a Claude session
- AC-003: `clr kill` with no argument exits 1 and stderr contains `"missing PID"`
- AC-004: `clr kill --help` and `clr kill -h` exit 0 and print usage information including `SIGTERM` and `<PID>`
- AC-005: `clr --help` and `clr help` list `kill` among the known subcommands
- AC-006: `clr kil` (one-character truncation typo) triggers the "Did you mean 'kill'?" guard and exits 1 with the message on stderr

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 7 | [`kill`](../command/07_kill.md) | The command being specified |

### Referenced Parameter Groups

None. `kill` accepts a single positional argument, not named parameters.

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 26 | [026_session_listing.md](026_session_listing.md) | `clr ps` discovers session PIDs; `clr kill` terminates them |
| 25 | [025_concurrency_gate.md](025_concurrency_gate.md) | Terminating a session frees a slot that was counted by the gate |
| 16 | [016_cli_discoverability.md](016_cli_discoverability.md) | AC-005 verifies `kill` appears in `clr help` output |
