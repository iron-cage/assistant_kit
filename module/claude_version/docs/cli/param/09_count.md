# Parameter :: 9. `count::`

-- **Summary:** Limit the number of releases shown by `.version.list mode::history`.
-- **Type:** u64
-- **Default:** 10
-- **Commands:** `.version.list` (`mode::history` only)
-- **Group:** Output Control

Default is 10, showing the most recent releases first. Values exceeding
available releases return all available.

- **Type:** u64 (unsigned integer)
- **Default:** 10
- **Validation:** must be a non-negative integer; values exceeding available releases return all available

```sh
clv.version.list mode::history count::1       # most recent release only
clv.version.list mode::history count::3       # 3 most recent releases
clv.version.list mode::history count::0       # empty output (valid, exit 0)
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.version.list`](../command/version.md#command-6-versionlist) | 10 | Meaningful only under `mode::history`; ignored under `mode::aliases`. Values exceeding available releases return all available. |

### Referenced Type

| # | Type |
|---|------|
| 1 | `u64` |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|-----------|-----------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `v::`, `format::` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
