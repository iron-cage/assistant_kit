# Parameter :: 43. `only_valid::`

Filters the `.usage` table to hide accounts with invalid or missing tokens (🔴 rows).

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Show only accounts with valid tokens (status ≠ 🔴).

**Behavior:** When `only_valid::1`, rows where the account has an invalid or missing `accessToken` (🔴 composite status) are hidden. 🟢 and 🟡 rows remain visible. The `→` marker and footer recommendation are unaffected by this filter (computed on the full set before filtering).

**Examples:**

```text
only_valid::1            -> hide expired/missing token accounts
only_valid::1 count::5   -> first 5 valid accounts
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/009_token_usage.md](../../feature/009_token_usage.md) for status emoji tiers.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Filter to valid-token accounts only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Hide expired accounts from usable quota view |
