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
| 11 | [`.settings.set`](../command/settings.md#command--11-settingsset) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine setup) |
| 2 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 5 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
