# Parameter :: 44. `exclude_exhausted::`

Filters the `.usage` table to hide accounts in the 🟡 (exhausted) or 🔴 (invalid) tiers, showing only fully healthy 🟢 accounts.

- **Type:** `bool`
- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Show only fully healthy accounts (status = 🟢).
- **Group:** Display Control

**Behavior:** When `exclude_exhausted::1`, only 🟢 rows (valid token, `5h Left > 15%`, `7d Left > 5%`) are displayed. Both 🟡 (exhausted — hourly or weekly) and 🔴 (invalid token) rows are hidden. Strictly more aggressive than `only_valid::1`, which keeps 🟡 rows.

**Examples:**

```text
exclude_exhausted::1           -> only fully healthy accounts
exclude_exhausted::1 count::1  -> top healthy account (first after sort)
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/009_token_usage.md](../../feature/009_token_usage.md) for status emoji tier boundaries.
