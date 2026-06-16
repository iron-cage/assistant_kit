# Terminate unresponsive Claude Code processes

**Persona:** developer
**Goal:** Identify all running Claude Code processes and terminate them safely — with a dry-run preview before executing.
**Benefit:** Cleanly restarts stuck sessions without leaving orphaned processes or sending signals blindly.
**Priority:** Medium

### Acceptance Criteria

- [ ] `clv .processes` lists all running Claude Code PIDs and their working directories.
- [ ] `clv .processes format::json` returns the same as a JSON array.
- [ ] `clv .processes.kill dry::1` prints what would be killed without sending signals.
- [ ] `clv .processes.kill` sends SIGTERM, waits 2 seconds, then SIGKILLs survivors.
- [ ] `clv .processes.kill force::1` sends SIGKILL directly.
- [ ] After a successful kill, `clv .processes` returns an empty list (exit 0).

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.processes`](../command/processes.md#command--7-processes) | Lists all running Claude Code PIDs |
| 2 | [`.processes.kill`](../command/processes.md#command--8-processeskill) | Terminates detected processes via signal sequence |
| 3 | [`.help`](../command/root.md#command--1-help) | Provides discovery of available commands |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Execution Control](../param_group/02_execution_control.md) | Controls dry-run and force kill behavior |
| 2 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and output format |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`dry::`](../param/02_dry.md) | Previews kill action without sending signals |
| 2 | [`force::`](../param/03_force.md) | Sends SIGKILL directly, skipping SIGTERM wait |
| 3 | [`v::`](../param/04_v.md) | Controls diagnostic detail level |
| 4 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
