# Parameter :: 40. `only_next::`

Filters the `.usage` table to show only the row that received the `→` marker from the active `next::` strategy.

- **Type:** `bool`
- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Show only the recommended next account row.
- **Group:** Display Control
- **Pipeline Stage:** process — `next::` strategy evaluation requires per-account quota data; all accounts matching upstream constraints are fetched before the `→` marker is assigned

**Behavior:** When `only_next::1`, the result contains at most one row — the account selected by the active `next::` strategy. When no eligible candidate exists (all accounts are current/active, or no qualifying accounts for the strategy), the result is empty (0 data rows) and exits 0.

**Examples:**

```text
only_next::1              -> one row: the → account
only_next::1 get::7d_left -> bare 7d Left value for the recommended next account
only_next::1 next::drain  -> → row from drain strategy
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md).
