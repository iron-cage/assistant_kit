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

| # | Command |
|---|---------|
| 1 | [`.help`](../command/root.md#command--1-help) |
| 2 | [`.status`](../command/root.md#command--2-status) |
| 3 | [`.version.show`](../command/version.md#command--3-versionshow) |
| 4 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 5 | [`.version.guard`](../command/version.md#command--5-versionguard) |
| 6 | [`.version.list`](../command/version.md#command--6-versionlist) |
| 7 | [`.version.history`](../command/version.md#command--12-versionhistory) |
| 8 | [`.processes`](../command/processes.md#command--7-processes) |
| 9 | [`.processes.kill`](../command/processes.md#command--8-processeskill) |
| 10 | [`.settings.show`](../command/settings.md#command--9-settingsshow) |
| 11 | [`.settings.get`](../command/settings.md#command--10-settingsget) |
| 12 | [`.settings.set`](../command/settings.md#command--11-settingsset) |
| 13 | [`.config`](../command/config.md#command--13-config) |

