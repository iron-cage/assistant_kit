# Parameter :: 22. `jitter::`

Adds a random number of seconds in the range `[0, jitter]` to each outer cycle delay, preventing synchronized refreshes when multiple users run `.usage live::1` with the same `interval::`. Ignored (and not validated) when `live::0`.

- **Type:** `u64`
- **Default:** `0` (no jitter — exact `interval::` timing)
- **Constraints:** Must satisfy `jitter ≤ interval` when `live::1`; violation exits 1 with `"jitter must not exceed interval"`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Thunder-herd mitigation — when many users share the same refresh cadence, jitter spreads the API call bursts across a wider time window.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
jitter::0    → no jitter, exact interval timing (default)
jitter::10   → each cycle waits interval + random[0..=10] seconds
jitter::30   → each cycle waits interval + random[0..=30] seconds
jitter::70   → exit 1: "jitter must not exceed interval" (when interval::60)
```
