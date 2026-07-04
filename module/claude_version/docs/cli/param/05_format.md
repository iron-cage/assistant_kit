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

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.status`](../command/root.md#command--2-status) | text | |
| 2 | [`.version.show`](../command/version.md#command--3-versionshow) | text | |
| 3 | [`.version.install`](../command/version.md#command--4-versioninstall) | text | |
| 4 | [`.version.guard`](../command/version.md#command--5-versionguard) | text | |
| 5 | [`.version.list`](../command/version.md#command--6-versionlist) | text | |
| 6 | [`.processes`](../command/processes.md#command--7-processes) | text | |
| 7 | [`.processes.kill`](../command/processes.md#command--8-processeskill) | text | |
| 8 | [`.settings.show`](../command/settings.md#command--9-settingsshow) | text | |
| 9 | [`.settings.get`](../command/settings.md#command--10-settingsget) | text | |
| 10 | [`.version.history`](../command/version.md#command--12-versionhistory) | text | |
| 11 | [`.config`](../command/config.md#command--13-config) | text | |
| 12 | [`.params`](../command/params.md#command--14-params) | text | |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `v::`, `count::` |

### Referenced Type

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
| 7 | [007 Params Inspection](../user_story/007_params_inspection.md) | Developer (config inspector) |
