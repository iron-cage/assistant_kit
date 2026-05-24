# Parameter :: 25. `sort::`

Controls row ordering in the `.usage` quota table. Each value implements a distinct heuristic optimized for a specific operational workflow.

- **Type:** `enum`
- **Default:** `reset`
- **Constraints:** `name`, `endurance`, `drain`, `reset`
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

**Examples:**

```text
sort::reset      → use accounts resetting soonest (default)
sort::name       → alphabetical A→Z
sort::endurance  → best for uninterrupted agent run
sort::drain      → drain almost-exhausted accounts
```

**See Also:** [feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) for strategy algorithms.
