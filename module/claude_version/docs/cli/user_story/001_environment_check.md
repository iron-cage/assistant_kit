# User Story :: 001. Environment Check

### Scope

- **Purpose**: Verify Claude Code installation state at a glance.
- **Responsibility**: Persona, goal, and acceptance criteria for environment verification via a single command.

### Persona

Developer on a new machine or after a system change who needs to confirm Claude Code is installed, running, and using the correct account.

### Goal

Run one command to see installed version, active session count, and active account — without digging into files or running multiple commands.

### Acceptance Criteria

- `cm .status` outputs version, session count, and active account in a single view.
- `cm .status format::json` returns the same fields as a JSON object for scripting.
- `cm .status v::2` shows additional diagnostic context.
- Missing HOME exits 2; all other failures also exit 2.

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.status`](../command/root.md#command--2-status) |
| 2 | [`.help`](../command/root.md#command--1-help) |

### Referenced Formats

| # | Format |
|---|--------|
| 1 | [text](../format/01_text.md) |
| 2 | [json](../format/02_json.md) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Output Control](../param_group/01_output_control.md) |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`v::`](../param/04_v.md) |
| 2 | [`format::`](../param/05_format.md) |
