# Parameter :: 37. `count::`

Limits the number of rows displayed after all sorting, grouping, and filtering has been applied. Applied as the final truncation step after `offset::`.

- **Type:** `u64`
- **Default:** `0` (all rows)
- **Constraints:** Non-negative integer; `0` means no limit
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Show at most N rows from the filtered result set.
- **Group:** Display Control

**Behavior:** `count::0` (default) shows all rows that survive the filter chain and offset. `count::N` (N ≥ 1) shows at most N rows. The table header and footer are always shown regardless of count.

**Examples:**

```text
count::3          -> show at most 3 rows
offset::1 count::3 -> skip first row, then show at most 3
count::1 get::account -> show account name of the first (sorted) row
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) for filter evaluation order.
