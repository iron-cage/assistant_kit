# Parameter :: 32. `next::`

Controls which recommendation strategy places the `→` marker on the recommended next account in the `.usage` table. The footer always shows all three strategy recommendations; `next::` affects only which account receives the `→` flag.

- **Default:** `renew`
- **Constraints:** `renew`, `endurance`, `drain`
- **Purpose:** Select which strategy's winner receives the `→` table marker.

**Values:**

| Value | Behavior |
|-------|----------|
| `renew` (default) | First eligible account from renew sort order — the account whose quota will refill soonest; `→` marks winner in table |
| `endurance` | Top of endurance sort; `→` marks winner in table |
| `drain` | Lowest `prefer_weekly > 5.0` account (skips weekly-exhausted accounts where `prefer_weekly ≤ 5.0`); `→` marks winner in table |

The footer always shows one recommendation line per strategy (renew, endurance, drain) regardless of which `next::` value is active.

**Examples:**

```text
next::renew       -> arrow on account with soonest quota refill (default)
next::drain       -> arrow on lowest weekly-healthy (> 5%) quota account
next::endurance   -> arrow on best for long agent run
```

**See Also:** [feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md) for strategy algorithms.

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Sort Control](../param_group/004_sort_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Strategy for `→` recommended next account marker |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Workflow-driven next account recommendation |
