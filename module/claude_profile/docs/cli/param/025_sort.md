# Parameter :: 25. `sort::`

Controls row ordering in the `.usage` quota table. Each value implements a distinct heuristic optimized for a specific operational workflow.

- **Type:** `enum`
- **Default:** `renew`
- **Constraints:** `name`, `endurance`, `drain`, `renew`, `next`, `expires`, `renews`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Select the row sorting strategy for the quota table.
- **Group:** Sort Control

**Values:**

| Value | Purpose | Default `desc::` |
|-------|---------|-------------------|
| `name` | Alphabetical — stable layout for `live::1` monitor | `0` (A→Z) |
| `endurance` | Sustained 5h+ session — qualified accounts first | `1` (best on top) |
| `drain` | Drain accounts with the lowest 7d weekly quota first | `0` (lowest on top) |
| `renew` | Use accounts whose next quota renewal event fires soonest (7d reset or billing cycle) | `0` (soonest on top) |
| `next` | Mirror the active `next::` strategy — `→` winner always appears first | inherits from resolved strategy |
| `expires` | Sort by token expiry time — accounts expiring soonest appear first | `0` (soonest on top) |
| `renews` | Sort by subscription renewal timer — accounts whose billing cycle renews soonest appear first | `0` (soonest on top) |

**Examples:**

```text
sort::renew      → use accounts whose next renewal event fires soonest (default)
sort::name       → alphabetical A→Z
sort::endurance  → best for uninterrupted agent run
sort::drain      → drain accounts with the lowest 7d weekly quota first
sort::next       → mirror active next:: strategy (→ winner first)
sort::expires    → accounts expiring soonest first
sort::renews     → accounts with soonest billing renewal first
```

**See Also:** [feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) for strategy algorithms.
