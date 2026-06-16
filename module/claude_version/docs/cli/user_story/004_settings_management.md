# Inspect and modify Claude Code settings

**Persona:** developer
**Goal:** Inspect all settings or read/write a specific setting — with type inference and atomic writes — without opening the settings file directly.
**Benefit:** Modifies Claude Code settings reliably without hand-editing JSON or risking partial writes from concurrent access.
**Priority:** Medium

### Acceptance Criteria

- [ ] `clv .settings.show` prints all key-value pairs from `~/.claude/settings.json`.
- [ ] `clv .settings.show format::json` returns the full settings object as JSON.
- [ ] `clv .settings.get key::X` prints the current value of key X; exits 2 if key absent.
- [ ] `clv .settings.set key::X value::V dry::1` previews the write without modifying the file.
- [ ] `clv .settings.set key::X value::V` writes the value with type inference and atomic rename.
- [ ] Boolean strings (`true`/`false`), integers, and floats are inferred to their JSON types.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.settings.show`](../command/settings.md#command--9-settingsshow) | Displays all current settings key-value pairs |
| 2 | [`.settings.get`](../command/settings.md#command--10-settingsget) | Reads a specific setting value by key |
| 3 | [`.settings.set`](../command/settings.md#command--11-settingsset) | Writes a setting with type inference and atomic rename |
| 4 | [`.help`](../command/root.md#command--1-help) | Provides discovery of available commands |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Execution Control](../param_group/02_execution_control.md) | Controls dry-run behavior for .settings.set |
| 2 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and output format |
| 3 | [Settings Identity](../param_group/03_settings_identity.md) | Identifies the setting key and value to read or write |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`key::`](../param/06_key.md) | Identifies the setting key to read or write |
| 2 | [`value::`](../param/07_value.md) | Provides the value to write with type inference |
| 3 | [`dry::`](../param/02_dry.md) | Previews write without modifying the file |
| 4 | [`v::`](../param/04_v.md) | Controls diagnostic detail level |
| 5 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
