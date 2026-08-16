# Parameter: 81. `max_age::`

Staleness eligibility threshold for [`stalest::`](080_stalest.md) — only accounts whose quota cache is **older than** SECS seconds are eligible for the reduced fetch set. A fully-fresh fleet fetches nothing.

### Summary

| Attribute | Value |
|-----------|-------|
| Type | `u64` (seconds) |
| Default | `0` — no eligibility threshold (every account eligible) |
| Commands | [`.usage`](../command/006_usage.md) |
| Group | [Fetch Behavior](../param_group/003_fetch_behavior.md) |
| Requires | `stalest::` — standalone `max_age::` (any value, including `0`) exits 1 |

### Semantics

Applied inside the stale-first ranking before the K-selection: accounts with cache age ≤ SECS are removed from eligibility. The fetch set is then the K oldest of the *remaining* accounts — possibly fewer than K, possibly empty.

- Age comparison is strict: an account exactly SECS old is **not** eligible.
- Missing cache = infinite age — always eligible.
- With an empty eligible set, the invocation performs zero HTTP fetches; every row renders from cache.

The combination `stalest::1 max_age::SECS` gives a background scheduler a self-terminating drain loop: each tick refreshes the single stalest account still over the threshold, and once the whole fleet is fresher than SECS the ticks become free (cache-only) until staleness re-accumulates.

### Validation

| Input | Result |
|-------|--------|
| `max_age::SECS` without `stalest::` | Exit 1 — the threshold filters the stalest selection's eligibility; it has no standalone meaning. Presence-based: `max_age::0` alone also exits 1 |
| Negative or non-integer | Exit 1 — must be a non-negative integer (seconds) |

Validation errors occur at parse time — zero HTTP calls, zero cache mutations.

### Examples

```bash
clp .usage stalest::1 max_age::7200         # refresh the stalest account only if older than 2h
clp .usage stalest::3 max_age::3600         # up to 3 accounts, each older than 1h
clp .usage stalest::1 max_age::7200 trace::1  # observe drain order and skips
```

### Valid Values

| Value | Meaning |
|-------|---------|
| `0` (default) | No eligibility threshold — `stalest::` selects from the whole fleet |
| `SECS ≥ 1` | Only accounts with cache age > SECS are fetch-eligible |

### Referenced Type

- **Fundamental Type:** `u64`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Eligibility threshold for the reduced quota fetch |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Self-terminating staleness drain — background ticks stop fetching once the fleet is fresh |

### See Also

| File | Relationship |
|------|--------------|
| [param/080_stalest.md](080_stalest.md) | The reducer this threshold composes with (required) |
| [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) | Fetch pipeline position relative to row filters |
