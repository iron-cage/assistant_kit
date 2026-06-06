# Parameter :: 39. `only_active::`

Filters the `.usage` table to show only the row for the currently active account (the account matching the per-machine active marker).

- **Type:** `bool`
- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Show only the active account row.
- **Group:** Display Control
- **Pipeline Stage:** fetch — `is_active` from filesystem `_active_{hostname}_{user}` marker; account list reduced to ≤1 entry before HTTP fetch loop begins

**Behavior:** When `only_active::1`, the active account is identified from the `_active_{hostname}_{user}` filesystem marker before any HTTP call, reducing the fetch set to at most 1 account (Pipeline-Constraint rule: short-circuit after first match). Only the row whose account matches this marker is displayed. The footer is still shown. If the active account has no valid quota (🔴 row), it is still included — `only_active::1` does not filter by health status.

**Examples:**

```text
only_active::1            -> one row: the active account
only_active::1 get::5h_left -> bare 5h Left value for active account
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md).
