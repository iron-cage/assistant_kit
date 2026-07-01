# Parameter: 39. `only_active::`

Filters the `.usage` table to show only the row for the currently active account (the account matching the per-machine active marker).

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Show only the active account row.
- **Pipeline Stage:** fetch — `is_active` from filesystem `_active_{hostname}_{user}` marker; account list reduced to ≤1 entry before HTTP fetch loop begins

**Behavior:** When `only_active::1`, the active account is identified from the `_active_{hostname}_{user}` filesystem marker before any HTTP call, reducing the fetch set to at most 1 account (Pipeline-Constraint rule: short-circuit after first match). Only the row whose account matches this marker is displayed. The footer is still shown. If the active account has no valid quota (🔴 row), it is still included — `only_active::1` does not filter by health status.

**Examples:**

```text
only_active::1            -> one row: the active account
only_active::1 get::5h_left -> bare 5h Left value for active account
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md), [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md).

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Filter to active account row only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Single-account quota check before rotation |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Extract active account quota value for scripts |
