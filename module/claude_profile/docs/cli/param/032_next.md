# Parameter :: 32. `next::`

Controls which recommendation strategy places the `→` marker on the recommended next account in the `.usage` table. The footer always shows both strategy recommendations; `next::` affects only which account receives the `→` flag.

- **Type:** `enum`
- **Default:** `drain`
- **Constraints:** `endurance`, `drain`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Select which strategy's winner receives the `→` table marker.
- **Group:** Sort Control

**Values:**

| Value | Behavior |
|-------|----------|
| `endurance` | Top of endurance sort; `→` marks winner in table |
| `drain` (default) | Lowest `prefer_weekly > 5.0` account (skips weekly-exhausted accounts where `prefer_weekly ≤ 5.0`); `→` marks winner in table |

The footer always shows one recommendation line per strategy (endurance, drain) regardless of which `next::` value is active.

**Examples:**

```text
next::drain      -> arrow on lowest weekly-healthy (> 5%) quota account (default)
next::endurance  -> arrow on best for long agent run
```

**See Also:** [feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md) for strategy algorithms.
