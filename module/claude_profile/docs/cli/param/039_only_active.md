# Parameter :: 39. `only_active::`

Filters the `.usage` table to show only the row for the currently active account (the account matching the per-machine active marker).

- **Type:** `bool`
- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Show only the active account row.
- **Group:** Display Control

**Behavior:** When `only_active::1`, only the row whose account name matches `{credential_store}/_active_{hostname}_{user}` is displayed. The footer is still shown. If the active account has no valid quota (🔴 row), it is still included — `only_active::1` does not filter by health status.

**Examples:**

```text
only_active::1            -> one row: the active account
only_active::1 get::5h_left -> bare 5h Left value for active account
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md).
