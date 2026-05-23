# Parameter :: 3. `force::`

-- **Summary:** Bypass safety guards.
-- **Type:** bool
-- **Default:** false (0)
-- **Commands:** `.version.install`, `.version.guard`, `.processes.kill`
-- **Group:** Execution Control

For `.version.install`/`.version.guard`: skip the "already installed" idempotency check.
For `.processes.kill`: SIGKILL directly (no SIGTERM first).

- **Type:** bool
- **Default:** false (0)
- **Validation:** strictly `0` or `1`; `true`, `yes`, `TRUE` etc. rejected with exit 1

```sh
cm .version.install force::1          # reinstall even if current
cm .version.guard force::1            # reinstall even if matching
cm .processes.kill force::1           # SIGKILL immediately
```

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 2 | [`.version.guard`](../command/version.md#command--5-versionguard) |
| 3 | [`.processes.kill`](../command/processes.md#command--8-processeskill) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Execution Control](../param_group/02_execution_control.md) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 3 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
