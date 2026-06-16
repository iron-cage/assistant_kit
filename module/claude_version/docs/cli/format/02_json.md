# Format :: json

### Scope

- **Purpose**: Machine-readable structured output for scripting and pipeline integration.
- **Responsibility**: Rendering rules, field catalog, and command applicability for the `json` format.
- **In Scope**: All aspects of the `json` output format.
- **Out of Scope**: Human-readable output (→ `01_text.md`), format selection parameter (→ `../param/05_format.md`).

### Rendering Rules

- **Encoding:** Standard JSON; all strings properly escaped.
- **Top-level shape:** Object `{}` for single-result commands; array `[]` for list commands (`.processes`, `.version.list`, `.version.history`).
- **Verbosity interaction:** `v::0` omits optional fields; `v::1` includes standard fields (default); `v::2` includes all available fields including diagnostics.
- **Required keys not stripped:** Even at `v::0`, the primary payload key is always present.
- **Case-sensitive:** The format value is `json` (lowercase only); `JSON` or `Json` are rejected with exit 1.

### Field Catalog

Field names are stable snake_case JSON keys. Common fields:

| Command | Key | Type | Notes |
|---------|-----|------|-------|
| `.status` | `version` | string | Installed version or `null` |
| `.status` | `sessions` | number | Running process count |
| `.status` | `account` | string \| null | Active account name |
| `.version.show` | `version` | string | |
| `.version.list` | `[].alias` | string | Array of alias objects |
| `.version.list` | `[].version` | string | |
| `.processes` | `[].pid` | number | Array of process objects |
| `.processes` | `[].cwd` | string | |
| `.settings.get` | `key` | string | |
| `.settings.get` | `value` | any | JSON-typed value |
| `.version.history` | `[].version` | string | Array of release objects |
| `.version.history` | `[].date` | string | ISO 8601 |
| `.version.history` | `[].summary` | string | One-line description |

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.status`](../command/root.md#command--2-status) |
| 2 | [`.version.show`](../command/version.md#command--3-versionshow) |
| 3 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 4 | [`.version.guard`](../command/version.md#command--5-versionguard) |
| 5 | [`.version.list`](../command/version.md#command--6-versionlist) |
| 6 | [`.version.history`](../command/version.md#command--12-versionhistory) |
| 7 | [`.processes`](../command/processes.md#command--7-processes) |
| 8 | [`.processes.kill`](../command/processes.md#command--8-processeskill) |
| 9 | [`.settings.show`](../command/settings.md#command--9-settingsshow) |
| 10 | [`.settings.get`](../command/settings.md#command--10-settingsget) |
| 11 | [`.config`](../command/config.md#command--13-config) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine setup) |
| 2 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 5 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 6 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
