# Parameter: 80. `stalest::`

Stale-first fetch reduction — restrict the HTTP fetch set to the K accounts whose quota cache is oldest. Every other account stays in the output, rendered from cache via `approximate_quota()`. Designed for staggered background refresh: repeated invocations drain staleness oldest-first without full-fleet API bursts.

### Summary

| Attribute | Value |
|-----------|-------|
| Type | `u32` (integer ≥ 1) |
| Default | *(omit)* — no reduction; full-fleet fetch |
| Commands | [`.usage`](../command/006_usage.md) |
| Group | [Fetch Behavior](../param_group/003_fetch_behavior.md) |
| Mutual exclusion | `only_active::1` (exits 1 — both are pre-fetch reducers with contradictory selection intents) |
| Bypassed by | `rotate::1` — rotation needs a complete fresh ranking, so the reducer is disabled entirely |

### Semantics

Before the HTTP fetch loop, accounts are ranked by quota-cache age:

- Age = now − cache `fetched_at`, per account.
- **Missing cache (or unparseable `fetched_at`) ranks infinitely stale** — an account without usable cache data is refreshed first.
- Equal ages tie-break by account list position (deterministic across invocations).

The K oldest become the fetch set; only they take the HTTP path. Every other account is gated exactly like the `solo::`/occupied-elsewhere degradation paths — `approximate_quota()` renders its row from cache. **`stalest::` controls token consumption, not display**: row count, order, and all display filters are unchanged; only fetch provenance differs.

With [`max_age::`](081_max_age.md), only accounts staler than the threshold are eligible — the fetch set may then hold fewer than K accounts, possibly zero (a fully-fresh fleet fetches nothing).

### Validation

| Input | Result |
|-------|--------|
| `stalest::0` | Exit 1 — an empty fetch set is a contradiction; omit the parameter for full-fleet fetch |
| Negative or non-integer | Exit 1 — must be a positive integer |
| `stalest::K only_active::1` | Exit 1 — mutually exclusive pre-fetch reducers |
| `stalest::K rotate::1` | Allowed; reducer bypassed (full-fleet fetch so rotation ranks fresh data) |

Validation errors occur at parse time — zero HTTP calls, zero cache mutations.

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| K ≥ fleet size | Every account selected — equivalent to no reduction |
| Account with no cache | Ranks oldest; selected before any cached account |
| All caches equally old | First K accounts in list order selected |
| Selected account's token locally expired | Fetch path still taken; the expired-skip/refresh machinery applies as in a full run |

### Trace Output

When `trace::1`, non-selected accounts log a dedicated skip line:

```
2026-08-16 · 12:00:04 · fetch    alice@work.pro   stale-skip: not in stalest set (cache-rendered)
```

### Examples

```bash
clp .usage stalest::1                       # refresh only the single stalest account
clp .usage stalest::3                       # refresh the 3 stalest accounts
clp .usage stalest::1 max_age::7200         # refresh the stalest account only if older than 2h
clp .usage stalest::1 format::tsv           # reduced fetch; output shape unchanged
clp .usage stalest::2 trace::1              # show which accounts were stale-skipped
```

### Valid Values

| Value | Meaning |
|-------|---------|
| *(omit)* | No reduction — all eligible accounts fetched (existing behavior) |
| `K ≥ 1` | Fetch only the K stalest accounts; others render from cache |

### Referenced Type

- **Fundamental Type:** `u32`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Stale-first reduced quota fetch |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Staggered background refresh — one stale account per tick instead of full-fleet bursts |

### See Also

| File | Relationship |
|------|--------------|
| [param/081_max_age.md](081_max_age.md) | Eligibility threshold composing with `stalest::` |
| [param/060_solo.md](060_solo.md) | Sibling fetch reducer (current+owned selection instead of staleness ranking) |
| [param/059_rotate.md](059_rotate.md) | `rotate::1` bypasses the reducer |
| [param/039_only_active.md](039_only_active.md) | Mutually exclusive pre-fetch reducer |
| [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) | Fetch pipeline position relative to row filters |
