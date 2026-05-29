# Parameter :: 38. `offset::`

Skips the first N rows from the filtered result before applying `count::` and rendering. Enables pagination through the quota table output.

- **Type:** `u64`
- **Default:** `0` (no skip)
- **Constraints:** Non-negative integer
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Skip first N rows from the filtered and sorted result set.
- **Group:** Display Control

**Behavior:** `offset::0` (default) starts from the first row. `offset::N` skips the first N matching rows. Combined with `count::`, implements a sliding window: `offset::2 count::3` shows rows 3–5 (0-indexed).

**Examples:**

```text
offset::2             -> skip first 2 rows, show remainder
offset::1 count::1    -> show only the 2nd row in sorted order
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) for filter evaluation order and interaction with `count::`.
