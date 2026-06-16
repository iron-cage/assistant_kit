# User Story :: 004. Settings Management

### Scope

- **Purpose**: Read and write Claude Code settings via clv.
- **Responsibility**: Persona, goal, and acceptance criteria for settings inspection and modification.

### Persona

Developer who needs to view current Claude Code settings or change a specific setting atomically without hand-editing JSON.

### Goal

Inspect all settings or read/write a specific setting — with type inference and atomic writes — without opening the settings file directly.

### Acceptance Criteria

- `clv .settings.show` prints all key-value pairs from `~/.claude/settings.json`.
- `clv .settings.show format::json` returns the full settings object as JSON.
- `clv .settings.get key::X` prints the current value of key X; exits 2 if key absent.
- `clv .settings.set key::X value::V dry::1` previews the write without modifying the file.
- `clv .settings.set key::X value::V` writes the value with type inference and atomic rename.
- Boolean strings (`true`/`false`), integers, and floats are inferred to their JSON types.

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.settings.show`](../command/settings.md#command--9-settingsshow) |
| 2 | [`.settings.get`](../command/settings.md#command--10-settingsget) |
| 3 | [`.settings.set`](../command/settings.md#command--11-settingsset) |
| 4 | [`.help`](../command/root.md#command--1-help) |

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
| 3 | [Settings Identity](../param_group/03_settings_identity.md) |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`key::`](../param/06_key.md) |
| 2 | [`value::`](../param/07_value.md) |
| 3 | [`dry::`](../param/02_dry.md) |
| 4 | [`v::`](../param/04_v.md) |
| 5 | [`format::`](../param/05_format.md) |
