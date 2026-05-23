# Parameter :: 21. `interval::`

Sets the number of seconds between full refresh cycles in live mode. Ignored (and not validated) when `live::0`.

- **Type:** `u64`
- **Default:** `30` (seconds)
- **Constraints:** Must be ≥ 30 when `live::1`; values < 30 exit 1 with `"interval must be >= 30"`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Controls how frequently the live quota table refreshes. The minimum of 30 seconds prevents excessive API pressure on Anthropic's quota endpoint.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
interval::30    → refresh every 30 seconds (default)
interval::60    → refresh every minute
interval::120   → refresh every 2 minutes
interval::29    → exit 1: "interval must be >= 30" (only when live::1)
```
