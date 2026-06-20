# Parameter :: 40. `only_next::`

Filters the `.usage` table to show only the row selected as the recommended next account by the active `sort::` strategy.

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Show only the recommended next account row.
- **Pipeline Stage:** process — `sort::` strategy evaluation requires per-account quota data; all accounts matching upstream constraints are fetched before the recommendation is computed

**Behavior:** When `only_next::1`, the result contains at most one row — the top eligible account in the active `sort::` order. When no eligible candidate exists (all accounts are current/active, or no qualifying accounts), the result is empty (0 data rows) and exits 0.

**Examples:**

```text
only_next::1              -> one row: the recommended next account
only_next::1 get::7d_left -> bare 7d Left value for the recommended next account
only_next::1 sort::renews -> recommended row from renews strategy
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md).

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Filter to recommended next account row only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Identify next rotation target from strategy |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Extract recommended account name for automation |
