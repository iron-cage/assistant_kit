# Parameter :: 38. `offset::`

Skips the first N rows from the filtered result before applying `count::` and rendering. Enables pagination through the quota table output.

- **Default:** `0` (no skip)
- **Constraints:** Non-negative integer
- **Purpose:** Skip first N rows from the filtered and sorted result set.

**Behavior:** `offset::0` (default) starts from the first row. `offset::N` skips the first N matching rows. Combined with `count::`, implements a sliding window: `offset::2 count::3` shows rows 3–5 (0-indexed).

**Examples:**

```text
offset::2             -> skip first 2 rows, show remainder
offset::1 count::1    -> show only the 2nd row in sorted order
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) for filter evaluation order and interaction with `count::`.

### Referenced Type

- **Fundamental Type:** `u64`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Row skip before count truncation |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Pagination through quota table |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Sliding window for script processing |
