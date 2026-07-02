# Format :: text

### Scope

- **Purpose**: Human-readable labeled output for interactive terminal use.
- **Responsibility**: Rendering rules, field catalog, and command applicability for the `text` format.
- **In Scope**: All aspects of the `text` output format.
- **Out of Scope**: Machine-readable output (→ `02_json.md`), format selection parameter (→ `../param/05_format.md`).

### Rendering Rules

- **Label style:** `key: value` pairs on separate lines; labels are human-readable words, not JSON keys.
- **Verbosity interaction:** `v::0` suppresses labels (raw values only); `v::1` shows labeled pairs (default); `v::2` adds diagnostic context lines.
- **No guaranteed structure:** Field order and presence can vary between commands; not suitable for parsing.
- **Default format:** Used when `format::` is not specified.

### Field Catalog

Output fields differ per command. Common patterns:

| Pattern | Example |
|---------|---------|
| Version string | `version: 2.1.78` |
| PID list | `pid: 4821` (one per line) |
| Key-value setting | `theme: dark` |
| Named alias table | `stable   2.1.78` |
| Status summary | `version: 2.1.78  sessions: 2  account: alice` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.status`](../command/root.md#command--2-status) | Default human-readable output |
| 2 | [`.version.show`](../command/version.md#command--3-versionshow) | Default human-readable output |
| 3 | [`.version.install`](../command/version.md#command--4-versioninstall) | Default human-readable output |
| 4 | [`.version.guard`](../command/version.md#command--5-versionguard) | Default human-readable output |
| 5 | [`.version.list`](../command/version.md#command--6-versionlist) | Default human-readable output |
| 6 | [`.processes`](../command/processes.md#command--7-processes) | Default human-readable output |
| 7 | [`.processes.kill`](../command/processes.md#command--8-processeskill) | Default human-readable output |
| 8 | [`.settings.show`](../command/settings.md#command--9-settingsshow) | Default human-readable output |
| 9 | [`.settings.get`](../command/settings.md#command--10-settingsget) | Default human-readable output |
| 10 | [`.version.history`](../command/version.md#command--12-versionhistory) | Default human-readable output |
| 11 | [`.config`](../command/config.md#command--13-config) | Default human-readable output |
| 12 | [`.params`](../command/params.md#command--14-params) | Default human-readable output |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine setup) |
| 2 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 5 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 6 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
| 7 | [007 Params Inspection](../user_story/007_params_inspection.md) | Developer (config inspector) |
