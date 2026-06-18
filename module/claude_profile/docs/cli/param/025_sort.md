# Parameter :: 25. `sort::`

Controls row ordering AND the `→` recommendation marker in the `.usage` quota table. Single parameter — no separate `next::` parameter.

- **Default:** `renew`
- **Constraints:** `name`, `renew`, `renews`
- **Purpose:** Select the row sorting strategy and `→` recommended next account.

**Values:**

| Value | Purpose | Primary key | Secondary key | Tertiary key | Exhausted floor | Default `desc::` |
|-------|---------|-------------|---------------|--------------|-----------------|-------------------|
| `name` | Alphabetical — stable layout for `live::1` monitor | account name | — | — | `0` (A→Z) |
| `renew` | Accounts whose next quota event fires soonest | `min(7d_reset, subscription_renewal)` | `prefer_weekly` (lowest %) | name | `0` (soonest on top) |
| `renews` | Accounts whose billing cycle renews soonest | `subscription_renewal_secs` | name | — | `0` (soonest on top) |

**4-tier grouping (applied before sorting):**

All accounts are partitioned into four status groups before any sort strategy runs. Group order is fixed — sorting only reorders rows within each group.

| Group | Status | 5h Left | 7d Left | Position | Condition |
|-------|--------|---------|---------|----------|-----------|
| 1 | 🟢 Green | available | available | top | Fully healthy — both 5h and 7d quota available |
| 2 | 🟡 Yellow | 🟡 exhausted | 🟢 available | second | 5h session depleted — recovers on short-cycle reset |
| 3 | 🟡 Yellow | 🟢 available | 🟡 exhausted | third | 7d weekly quota depleted — long recovery |
| 4 | 🔴 Red | — | — | bottom | Fully exhausted — no quota remaining |

`desc::1` reverses row order within each group but does not change group order. Green always appears above yellow, yellow always above red.

**Sort key details:**

- **`sort::renew` primary key** includes subscription renewal (Fix(BUG-229)) — not just 7d reset. Whichever fires sooner wins.
- **`sort::renew` secondary key** is governed by `prefer::`: `prefer::opus` → 7d Left %; `prefer::sonnet` → 7d(Son) Left %; `prefer::any` → min(7d, Son) Left %.
- **`sort::renews`**: accounts without subscription data (`renewal_at` absent and no `org_created_at`) are placed last.
- **Determinism** (Fix(BUG-259)): all strategies use account name as final tiebreaker.

**`→` recommendation marker:**

`sort::` also drives the `→` marker — the top eligible account in the sort order receives `→`. No separate `next::` parameter exists. Eligibility: non-current, non-active, non-occupied, not h-exhausted (`5h Left > 15%`), not weekly-exhausted (`prefer_weekly > 5.0`), valid quota data, `expires_in_secs > 0`. When no eligible account exists, no `→` is placed.

The footer shows one recommendation line for the active `sort::` strategy (omitted when 0 or 1 valid accounts).

**Examples:**

```text
sort::renew      → soonest quota event first; → on soonest-refill account (default)
sort::name       → alphabetical A→Z; → on first alphabetical eligible account
sort::renews     → soonest billing renewal first; → on soonest-renewal account
```

**See Also:** [feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) for strategy algorithms.

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Sort Control](../param_group/004_sort_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Row ordering in quota table |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Workflow-optimized account ordering in quota view |
