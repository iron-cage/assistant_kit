# Inspect and modify Claude Code settings via .config

**Persona:** developer
**Goal:** Inspect all settings or read/write a specific setting with 4-layer resolution visibility, type inference, and atomic writes.
**Benefit:** Manages Claude Code settings reliably without hand-editing JSON, with full visibility into which config layer is active.
**Priority:** Medium

### Acceptance Criteria

- [ ] `clv .config` prints all resolved settings with source layer annotations (env/project/user/catalog).
- [ ] `clv .config key::X` shows the effective value for key X with source layer.
- [ ] `clv .config key::X format::json` returns the value as structured JSON.
- [ ] `clv .config key::X value::V` writes V to user settings with type inference and atomic rename.
- [ ] `clv .config key::X value::V scope::project` writes to project settings.
- [ ] `clv .config key::X value::V dry::1` previews the write without modifying any file.
- [ ] `clv .config key::X unset::1` removes key X from user settings.
- [ ] `clv .config key::X unset::1 scope::project` removes from project settings.
- [ ] Type inference: `"true"`/`"false"` → JSON bool; integer/float strings → JSON number; else → JSON string.
- [ ] Invalid combinations (`value::` + `unset::`, `scope::` without write op) exit 1 with message.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.help`](../command/root.md#command--1-help) | Provides discovery of config modes and parameters |
| 2 | [`.config`](../command/config.md#command--13-config) | Primary command: inspect and modify settings |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output with source annotations |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and output format |
| 2 | [Execution Control](../param_group/02_execution_control.md) | Controls dry-run behavior for set/unset operations |
| 3 | [Config Identity](../param_group/04_config_identity.md) | Identifies the config key, value, scope, and unset mode |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`dry::`](../param/02_dry.md) | Previews write or unset without modifying any file |
| 2 | [`v::`](../param/04_v.md) | Controls diagnostic detail level |
| 3 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
| 4 | [`key::`](../param/06_key.md) | Identifies the setting key to read or write |
| 5 | [`value::`](../param/07_value.md) | Provides the value to write with type inference |
| 6 | [`.help`](../param/10_help.md) | Universal help override for any command |
| 7 | [`scope::`](../param/11_scope.md) | Selects write target: user or project |
| 8 | [`unset::`](../param/12_unset.md) | Deletes the key from target scope instead of writing |

### Workflow Steps

**Step 1 — Show all resolved settings with source annotations:**

```bash
clv .config
# model:       claude-sonnet-5   (catalog default)
# theme:       dark              (user)
# autoUpdates: false             (user)
```

**Step 2 — Get the effective value for one key:**

```bash
clv .config key::model
# claude-sonnet-5  (catalog default)
```

**Step 3 — Preview a write to user scope:**

```bash
clv .config key::theme value::light dry::1
# [dry-run] Would write to ~/.claude/settings.json: theme = "light"  (string)
```

**Step 4 — Write a key to project scope:**

```bash
clv .config key::theme value::dark scope::project
# Written to .claude/settings.json: theme = "dark"  (string)
```
