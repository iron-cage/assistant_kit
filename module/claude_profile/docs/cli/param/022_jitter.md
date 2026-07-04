# Parameter: 22. `jitter::`

Adds a random number of seconds in the range `[0, jitter]` to each outer cycle delay, preventing synchronized refreshes when multiple users run `.usage live::1` with the same `interval::`. Ignored (and not validated) when `live::0`.

- **Default:** `0` (no jitter — exact `interval::` timing)
- **Constraints:** Must satisfy `jitter ≤ interval` when `live::1`; violation exits 1 with `"jitter must not exceed interval"`
- **Purpose:** Thunder-herd mitigation — when many users share the same refresh cadence, jitter spreads the API call bursts across a wider time window.

**Examples:**

```text
jitter::0    → no jitter, exact interval timing (default)
jitter::10   → each cycle waits interval + random[0..=10] seconds
jitter::30   → each cycle waits interval + random[0..=30] seconds
jitter::70   → exit 1: "jitter must not exceed interval" (when interval::60)
```

### Referenced Type

- **Fundamental Type:** `u64`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Spreads live mode refresh calls across time window |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | API pressure mitigation for shared live monitoring |
