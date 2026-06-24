# Parameter :: 43. `only_valid::`

Filters the `.usage` table to hide accounts with invalid or missing tokens, or cancelled subscriptions (🔴 rows).

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Show only accounts with valid, active subscriptions (status ≠ 🔴).

**Behavior:** When `only_valid::1`, rows where the account has an invalid or missing `accessToken` OR a cancelled subscription (`billing_type = "none"`) are hidden. Both conditions produce 🔴 composite status. 🟢 and 🟡 rows remain visible. The footer recommendation is unaffected by this filter (computed on the full set before filtering). Fix(BUG-317): cancelled accounts with `result = Ok(...)` were previously not excluded — the filter only checked `result.is_ok()` without inspecting `billing_type`.

**Examples:**

```text
only_valid::1            -> hide expired/missing token accounts and cancelled subscriptions
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
