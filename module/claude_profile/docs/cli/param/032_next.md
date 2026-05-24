# Parameter :: 32. `next::`

Controls which recommendation strategy selects the `->` Next account in the `.usage` footer. The default `all` shows recommendations from every strategy simultaneously.

- **Type:** `enum`
- **Default:** `all`
- **Constraints:** `all`, `session`, `endurance`, `drain`, `reset`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Select the recommendation strategy for the footer and `->` table marker.
- **Group:** Sort Control

**Values:**

| Value | Behavior |
|-------|----------|
| `all` (default) | Footer shows one recommendation per strategy; `->` suppressed in table |
| `session` | Best `(5h_left, expires_in_secs, 7d_left)` composite; `->` marks winner in table |
| `endurance` | Top of endurance sort; `->` marks winner in table |
| `drain` | Top of drain sort; `->` marks winner in table |
| `reset` | Top of reset sort; `->` marks winner in table |

**Examples:**

```text
next::all        -> multi-strategy footer (default)
next::session    -> single recommendation: best session composite
next::endurance  -> single recommendation: best for long agent run
next::drain      -> single recommendation: drain lowest-quota account
next::reset      -> single recommendation: soonest session reset
```

**See Also:** [feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md) for strategy algorithms.
