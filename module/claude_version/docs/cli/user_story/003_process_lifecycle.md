# User Story :: 003. Process Lifecycle

### Scope

- **Purpose**: Inspect and terminate running Claude Code processes.
- **Responsibility**: Persona, goal, and acceptance criteria for process management via cm.

### Persona

Developer whose Claude Code session has become unresponsive or who needs to restart all sessions cleanly.

### Goal

Identify all running Claude Code processes and terminate them safely — with a dry-run preview before executing.

### Acceptance Criteria

- `cm .processes` lists all running Claude Code PIDs and their working directories.
- `cm .processes format::json` returns the same as a JSON array.
- `cm .processes.kill dry::1` prints what would be killed without sending signals.
- `cm .processes.kill` sends SIGTERM, waits 2 seconds, then SIGKILLs survivors.
- `cm .processes.kill force::1` sends SIGKILL directly.
- After a successful kill, `cm .processes` returns an empty list (exit 0).

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.processes`](../command/processes.md#command--7-processes) |
| 2 | [`.processes.kill`](../command/processes.md#command--8-processeskill) |
| 3 | [`.help`](../command/root.md#command--1-help) |

### Referenced Formats

| # | Format |
|---|--------|
| 1 | [text](../format/01_text.md) |
| 2 | [json](../format/02_json.md) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Execution Control](../param_group/02_execution_control.md) |
| 2 | [Output Control](../param_group/01_output_control.md) |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`dry::`](../param/02_dry.md) |
| 2 | [`force::`](../param/03_force.md) |
| 3 | [`v::`](../param/04_v.md) |
| 4 | [`format::`](../param/05_format.md) |
