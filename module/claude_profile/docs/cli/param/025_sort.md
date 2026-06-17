# Parameter :: 25. `sort::`

Controls row ordering in the `.usage` quota table.

- **Default:** `renew`
- **Constraints:** `name`, `renew`, `renews`
- **Purpose:** Select the row sorting strategy for the quota table.

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

**Examples:**

```text
sort::renew      → soonest quota event first — 7d reset or subscription renewal (default)
sort::name       → alphabetical A→Z
sort::renews     → soonest billing renewal first
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
