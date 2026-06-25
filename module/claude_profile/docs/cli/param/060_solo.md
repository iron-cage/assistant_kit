# Parameter :: `solo::`

Token conservation mode — restrict all credential-consuming operations (HTTP fetch, refresh subprocess, touch subprocess) to the account that is both current and owned. All other accounts display approximated historical data.

### Summary

| Attribute | Value |
|-----------|-------|
| Type | `bool` (0 or 1) |
| Default | `0` |
| Commands | [`.usage`](../command/006_usage.md) |
| Group | [Fetch Behavior](../param_group/003_fetch_behavior.md) |
| Mutual exclusion | `rotate::1` (exits 1 — rotation requires live data from candidates) |

### Semantics

When `solo::1`, an account receives live API calls **only if** both conditions hold:

- `is_current == true` — active account on this machine
- `is_owned == true` — `owner` field matches this machine's identity

Every other account: **zero HTTP calls, zero subprocesses**. Cached/historical data is served via a dedicated approximation function (`approximate_quota`) that all non-live code paths must call — direct cache file reads are not permitted.

**`solo::1` controls token consumption, not display.** Display filters (`only_active::`, `count::`, `offset::`, `min_5h::`, etc.) remain fully independent. `solo::1` never removes rows from the table — it only changes whether a row shows live data or approximated cached data.

**Data flow with `solo::1`:**

1. Enumerate all saved accounts
2. Per account: check `is_current && is_owned`
3. Current+owned → live HTTP fetch; all others → `approximate_quota()` (cache fallback)
4. Apply refresh gates (only if solo allows this account)
5. Apply touch gates (only if solo allows this account)
6. Apply display filters: `only_active::`, `count::`, `offset::`, etc.
7. Render table — all surviving rows shown (live or approximated)

**Approximation function:** All code paths needing non-live data for solo-skipped accounts must call `approximate_quota(name, store)` — a single dedicated function that reads the quota cache and returns historical values. This function is the sole source of non-live data; no caller may read cache files directly when `solo::1` is active.

### Gate Placement

`solo::1` inserts a gate at 3 locations, each after the existing ownership gate:

| Gate | Location | Existing gate | Solo addition |
|------|----------|---------------|---------------|
| Fetch | `fetch.rs` (after G1) | `!is_owned → cache` | `solo && !is_current → approximate_quota()` |
| Refresh | `refresh.rs` (after G2) | `!is_owned → skip` | `solo && !aq.is_current → skip` |
| Touch | `touch.rs` (after G4b) | `!is_owned → skip` | `solo && !aq.is_current → skip` |

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| Current account is NOT owned | No account passes both conditions. All accounts display approximated data. Zero HTTP calls. |
| No active marker (no current account) | Same as above. All approximated. Zero HTTP calls. |
| Current+owned but occupied_elsewhere | Solo gate passes (current+owned). G2/G4 still apply — refresh/touch skip for occupied_elsewhere. HTTP fetch still happens. |
| Multiple owned accounts | Only the one that is also current gets live fetch. Others: approximated. |

### Compositions

| Combination | Behavior |
|-------------|----------|
| `solo::1 rotate::1` | **Exit 1.** Conflicting intent — rotation picks a different account but solo prevents live-fetching candidates. |
| `solo::1 live::1` | Allowed. Each loop cycle: HTTP-fetch only current+owned; others show progressively stale approximated data. |
| `solo::1 refresh::1` | Allowed. Refresh subprocess fires only for current+owned account (if error predicate matches). |
| `solo::1 touch::1` | Allowed. Touch subprocess fires only for current+owned account (if idle). |
| `solo::1 only_active::1` | Allowed. Orthogonal: fetch current+owned live, display only active row. |
| `solo::1 assignee::USER@MACHINE` | Allowed. `assignee::` writes local marker file — no API call. (Feature 065: `active::` is REMOVED.) |
| `solo::1 owner::0` | Allowed. `owner::0` writes local JSON — no API call. (Feature 064: `unclaim::1` is REMOVED.) |
| `solo::1 force::1` | Allowed. `force::` bypasses G5-G8 (write-side). Solo is read-side token conservation. Independent domains. |

### Trace Output

When `trace::1`:

```
2026-06-25 · 16:40:04 · fetch    alice@work.pro   live (current+owned)
2026-06-25 · 16:40:05 · fetch    bob@home.pro     solo-skip: approximated (age: 1800s)
2026-06-25 · 16:40:06 · fetch    carol@other.pro  solo-skip: approximated (age: 7200s)
2026-06-25 · 16:40:07 · refresh  bob@home.pro     solo-skip
2026-06-25 · 16:40:08 · touch    bob@home.pro     solo-skip
```

### Examples

```bash
clp .usage solo::1                          # conserve tokens: only probe current+owned
clp .usage solo::1 trace::1                 # solo with diagnostic trace
clp .usage solo::1 live::1 interval::60     # continuous monitor, only current+owned live
clp .usage solo::1 refresh::1               # refresh only current+owned on auth error
clp .usage solo::1 touch::0                 # solo without touch (fetch only)
clp .usage solo::1 only_active::1           # solo + display only active row
```

### Valid Values

| Value | Meaning |
|-------|---------|
| `0` (default) | All owned accounts receive live API calls (existing behavior) |
| `1` | Only the current+owned account receives live API calls; others use `approximate_quota()` |

### Cross-References

| File | Relationship |
|------|--------------|
| [feature/036_account_ownership.md](../../feature/036_account_ownership.md) | Ownership model and G1-G4 gates that solo extends |
| [feature/039_decision_algorithms.md](../../feature/039_decision_algorithms.md) | Decision algorithm reference — solo gate added to fetch/refresh/touch |
| [param/059_rotate.md](059_rotate.md) | `rotate::1` is mutually exclusive with `solo::1` |
| [param/039_only_active.md](039_only_active.md) | `only_active::` is a display filter, orthogonal to solo |
| [param_group/003_fetch_behavior.md](../param_group/003_fetch_behavior.md) | Fetch Behavior group membership |
