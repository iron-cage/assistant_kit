# Parameter :: 10. `.help`

-- **Summary:** Display help listing and exit; overrides any command when present anywhere in argv.
-- **Type:** bool (standalone)
-- **Default:** false
-- **Commands:** all commands
-- **Group:** none

Present anywhere in argv triggers help display and exit, regardless of
other commands or parameters.

- **Type:** bool (standalone)
- **Default:** false
- **Commands:** all commands (universal override)

```sh
clv.help
clv.version.install .help    # still shows help, ignores install
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.help`](../command/root.md#command-1-help) | false | Universal override — triggers help and exits |
| 2 | [`.status`](../command/root.md#command-2-status) | false | Universal override |
| 3 | [`.version.show`](../command/version.md#command-3-versionshow) | false | Universal override |
| 4 | [`.version.install`](../command/version.md#command-4-versioninstall) | false | Universal override |
| 5 | [`.version.guard`](../command/version.md#command-5-versionguard) | false | Universal override |
| 6 | [`.version.list`](../command/version.md#command-6-versionlist) | false | Universal override |
| 7 | [`.processes`](../command/processes.md#command-7-processes) | false | Universal override |
| 8 | [`.processes.kill`](../command/processes.md#command-8-processeskill) | false | Universal override |
| 9 | [`.settings.show`](../command/settings.md#command-9-settingsshow) | false | Universal override |
| 10 | [`.settings.get`](../command/settings.md#command-10-settingsget) | false | Universal override |
| 11 | [`.settings.set`](../command/settings.md#command-11-settingsset) | false | Universal override |
| 12 | [`.version.history`](../command/version.md#command-12-versionhistory) | false | Universal override |
| 13 | [`.config`](../command/config.md#command-13-config) | false | Universal override |
| 14 | [`.params`](../command/params.md#command-14-params) | false | Universal override |

### Referenced Type

| # | Type |
|---|------|
| 1 | `bool` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
