# Parameter :: 5. `format::`

-- **Summary:** Select output format.
-- **Type:** `OutputFormat`
-- **Default:** text
-- **Commands:** all commands with format:: support
-- **Group:** Output Control

Case-sensitive: `text` and `json` only.

- **Type:** [`OutputFormat`](../type/02_output_format.md)
- **Default:** text
- **Validation:** `text` or `json` only; `TEXT`, `Json` etc. -> exit 1

```sh
clv.status format::json
clv.settings.show format::text
```

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

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Output Control](../param_group/01_output_control.md) |

### Referenced Types

| # | Type |
|---|------|
| 1 | [`OutputFormat`](../type/02_output_format.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine setup) |
| 2 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 5 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 6 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
