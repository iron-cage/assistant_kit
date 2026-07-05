# Parameter: 21. `interval::`

Sets the number of seconds between full refresh cycles in live mode. Ignored (and not validated) when `live::0`.

- **Default:** `30` (seconds)
- **Constraints:** Must be ≥ 30 when `live::1`; values < 30 exit 1 with `"interval must be >= 30"`
- **Purpose:** Controls how frequently the live quota table refreshes. The minimum of 30 seconds prevents excessive API pressure on Anthropic's quota endpoint.

**Examples:**

```text
interval::30    → refresh every 30 seconds (default)
interval::60    → refresh every minute
interval::120   → refresh every 2 minutes
interval::29    → exit 1: "interval must be >= 30" (only when live::1)
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
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Live mode refresh cycle duration |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Tunable refresh cadence for live monitoring |
