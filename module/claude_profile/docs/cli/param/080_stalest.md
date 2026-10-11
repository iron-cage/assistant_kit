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

Before the HTTP fetch loop, accounts with an empty or `anthropic` `inference_provider` are ranked by quota-cache age. An account the fetch loop skips whatever the selection says is never ranked:
a `backend: redirect` account, one another machine owns, one occupied on another machine unless it's the live session here, and with `solo::1` any account but the live session here.

Ranking:

- Age = now − the newest of the cache's `fetched_at` (last success), `last_error_at` (last no-subscription verdict) and `last_attempt_at`
  (last failed fetch of any kind), per account: the time since the last fetch *attempt*, not the last success. It's the cache age `max_age::` compares against.
  <!-- BUG-591 — every failed live fetch stamps last_attempt_at, so an account that keeps failing waits its turn instead of taking the slot every tick -->
- **Missing cache (or unparseable `fetched_at`) ranks infinitely stale** — an account without usable cache data is selected first.
- Equal ages tie-break by account list position (deterministic across invocations).

The K oldest become the fetch set; only they take the HTTP path. Every other account is gated exactly like the `solo::`/occupied-elsewhere degradation paths — `approximate_quota()` renders its row from cache. **`stalest::` controls token consumption, not display**: row count, order, and all display filters are unchanged; only fetch provenance differs.
<!-- BUG-583 — the fetch loop's skip gates apply before ranking, so no pick is an account they skip -->

With [`max_age::`](081_max_age.md), only accounts staler than the threshold are eligible — the fetch set may then hold fewer than K accounts, possibly zero (a fully-fresh fleet fetches nothing).

### Validation

| Input | Result |
|-------|--------|
| `stalest::0` | Exit 1 — an empty fetch set is a contradiction; omit the parameter for full-fleet fetch |
| Negative or non-integer | Exit 1 — must be a positive integer |
| `stalest::K only_active::1` | Exit 1 — mutually exclusive pre-fetch reducers |
| `stalest::K rotate::1` | Allowed; reducer bypassed (full-fleet fetch so rotation ranks fresh data) |
| `stalest::K solo::1` | Allowed; only the live session here is ranked, since the solo gate renders every other account from cache <!-- BUG-583 — selection applies the solo gate too --> |

Validation errors occur at parse time — zero HTTP calls, zero cache mutations.

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| K ≥ fleet size | Every ranked account selected — equivalent to no reduction |
| Account with no cache | Ranks oldest; selected before any cached account |
| All caches equally old | First K ranked accounts in list order selected |
| Selected account's token locally expired | Fetch path still taken; the expired-skip/refresh machinery applies as in a full run |
| Oldest account occupied on another machine | Not ranked; its slot goes to the next ranked account |
| Occupied elsewhere, but the live session here | Ranked as usual; the occupied-elsewhere gate exempts the live session |
| Account another machine owns | Not ranked; the ownership gate renders it from cache |
| `backend: redirect` account | Not ranked, whatever its `inference_provider`; it has no Anthropic quota |

### Trace Output

When `trace::1`, the selected set is logged once, before the fetch loop, with names sorted. An empty set prints `none`. Non-selected accounts then log a dedicated skip line:

```
2026-08-16 · 12:00:04 UTC · stalest  selected (k=1, max_age=7200s): bob@work.pro
2026-08-16 · 12:00:04 UTC · fetch  alice@work.pro  stale-skip: not in stalest set (cache-rendered)
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
