# Parameter :: 44. `exclude_exhausted::`

Filters the `.usage` table to hide accounts in status groups 2–4 (🟡 h-exhausted, 🟡 weekly-exhausted, 🔴 Red), showing only 🟢 Green accounts.

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Show only fully healthy accounts (status = 🟢).

**Behavior:** When `exclude_exhausted::1`, only 🟢 rows (valid token, `5h Left > 15%`, `7d Left > 5%`) are displayed. Both 🟡 (exhausted — hourly or weekly) and 🔴 (invalid token) rows are hidden. Strictly more aggressive than `only_valid::1`, which keeps 🟡 rows.

**Examples:**

```text
exclude_exhausted::1           -> only fully healthy accounts
exclude_exhausted::1 count::1  -> top healthy account (first after sort)
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/009_token_usage.md](../../feature/009_token_usage.md) for status groups, [dictionary](../002_dictionary.md#status-groups) for canonical definitions.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Filter to fully healthy (🟢) accounts only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Show only usable accounts for active workloads |
