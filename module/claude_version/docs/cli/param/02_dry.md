# Parameter :: 2. `dry::`

-- **Summary:** Preview the action that would be performed without executing side effects.
-- **Type:** bool
-- **Default:** false (0)
-- **Commands:** `.version.install`, `.version.guard`, `.processes.kill`, `.settings.set`
-- **Group:** Execution Control

Output is prefixed with `[dry-run] would ...`.

- **Type:** bool
- **Default:** false (0)
- **Validation:** strictly `0` or `1`; `true`, `yes`, `TRUE` etc. rejected with exit 1

```sh
cm .version.install dry::1
cm .version.guard dry::1
cm .processes.kill dry::1
cm .settings.set key::theme value::dark dry::1
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 2 | [`.version.guard`](../command/version.md#command--5-versionguard) |
| 3 | [`.processes.kill`](../command/processes.md#command--8-processeskill) |
| 4 | [`.settings.set`](../command/settings.md#command--11-settingsset) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Execution Control](../param_group/02_execution_control.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 3 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 4 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
