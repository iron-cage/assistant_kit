# Parameter :: 25. `sort::`

Controls row ordering in the `.usage` quota table. Each value implements a distinct heuristic optimized for a specific operational workflow.

- **Type:** `enum`
- **Default:** `drain`
- **Constraints:** `name`, `endurance`, `drain`, `reset`, `next`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Select the row sorting strategy for the quota table.
- **Group:** Sort Control

**Values:**

| Value | Purpose | Default `desc::` |
|-------|---------|-------------------|
| `name` | Alphabetical — stable layout for `live::1` monitor | `0` (A→Z) |
| `endurance` | Sustained 5h+ session — qualified accounts first | `1` (best on top) |
| `drain` | Cherry-pick low-quota accounts to finish them off | `0` (lowest on top) |
| `reset` | Use accounts whose quota refills soonest | `0` (soonest on top) |
| `next` | Mirror the active `next::` strategy — `→` winner always appears first | inherits from resolved strategy |

**Examples:**

```text
sort::drain      → drain almost-exhausted accounts (default)
sort::name       → alphabetical A→Z
sort::endurance  → best for uninterrupted agent run
sort::reset      → use accounts resetting soonest
sort::next       → mirror active next:: strategy (→ winner first)
```

**See Also:** [feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) for strategy algorithms.
