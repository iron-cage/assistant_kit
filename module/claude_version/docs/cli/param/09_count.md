# Parameter :: 9. `count::`

-- **Summary:** Limit the number of releases shown by .version.history.
-- **Type:** u64
-- **Default:** 10
-- **Commands:** `.version.history`
-- **Group:** Output Control

Default is 10, showing the most recent releases first. Values exceeding
available releases return all available.

- **Type:** u64 (unsigned integer)
- **Default:** 10
- **Validation:** must be a non-negative integer; values exceeding available releases return all available

```sh
clv.version.history count::1       # most recent release only
clv.version.history count::3       # 3 most recent releases
clv.version.history count::0       # empty output (valid, exit 0)
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.version.history`](../command/version.md#command--12-versionhistory) | 10 | Values exceeding available releases return all available |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `v::`, `format::` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
