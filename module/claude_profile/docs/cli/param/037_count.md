# Parameter :: 37. `count::`

Limits the number of rows displayed after all sorting, grouping, and filtering has been applied. Applied as the final truncation step after `offset::`.

- **Default:** `0` (all rows)
- **Constraints:** Non-negative integer; `0` means no limit
- **Purpose:** Show at most N rows from the filtered result set.

**Behavior:** `count::0` (default) shows all rows that survive the filter chain and offset. `count::N` (N ≥ 1) shows at most N rows. The table header and footer are always shown regardless of count.

**Examples:**

```text
count::3          -> show at most 3 rows
offset::1 count::3 -> skip first row, then show at most 3
count::1 get::account -> show account name of the first (sorted) row
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) for filter evaluation order.

### Referenced Type

- **Fundamental Type:** `u64`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Row count limit after filtering and sorting |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Show top-N accounts in focused quota view |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Limit output for script processing |
