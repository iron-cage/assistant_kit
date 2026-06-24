# Feature: Solo Token Conservation Mode

### Scope

- **Purpose**: Restrict all credential-consuming operations (HTTP fetch, refresh subprocess, touch subprocess) to the single account that is both `is_current` and `is_owned`. All other accounts display historical approximated data from `approximate_quota()`.
- **Responsibility**: Documents the `solo::1` parameter behavior, the `is_current && is_owned` gate predicate, gate placement at fetch/refresh/touch sites, the `approximate_quota()` function as the sole non-live data source, mutual exclusion with `rotate::1`, and trace output format.
- **In Scope**: `solo::1` gate predicate, `approximate_quota()` function, gate placement (fetch/refresh/touch), solo+rotate mutual exclusion, display filter independence, trace output for skipped accounts.
- **Out of Scope**: Polynomial approximation algorithm (→ Feature 040), cache storage format (→ Feature 033), ownership enforcement gates G1–G8 (→ Feature 036), rotate strategy logic (→ Feature 038).

### Design

**Solo Predicate**

When `solo::1`, each account is evaluated against the predicate `is_current && is_owned`:

- `is_current` — the account's token matches the active credential file; set by the per-machine active marker (Feature 025)
- `is_owned` — `owner` field in `{name}.json` matches this machine's identity (Feature 036 G1 gate)

Only the account where both conditions hold receives live API calls. All other accounts are served via `approximate_quota()`.

**`approximate_quota()` Function**

A dedicated function `approximate_quota(name, store)` is the sole source of non-live quota data for accounts skipped by the solo gate. It:

1. Reads the quota cache from `{name}.json`
2. Reads the measurement history (`cache.history[]`)
3. Applies the Feature 040 polynomial approximation to each period independently
4. Returns a `QuotaData` struct with `cached: true` and approximated values

No caller may read cache files directly when serving solo-skipped accounts. All non-live data paths must call `approximate_quota()`.

**Gate Placement**

`solo::1` inserts a gate at three locations in the fetch pipeline:

| Gate | Source file | Position | Skip condition | Trace emission |
|------|-------------|----------|----------------|----------------|
| Fetch | `src/usage/fetch.rs` | After `is_current` computed, before HTTP request | `solo && !(is_current && is_owned)` | `[trace] fetch {name} solo-skip: approximated (age: Ns)` |
| Refresh | `src/usage/refresh.rs` | First gate in per-account loop | `solo && !aq.is_current` | `[trace] refresh {name} solo-skip` |
| Touch | `src/usage/touch.rs` | First gate in per-account loop | `solo && !aq.is_current` | `[trace] touch {name} solo-skip` |

When the fetch gate fires, `approximate_quota()` is called and its result replaces the live fetch. When the refresh or touch gates fire, the account is skipped entirely — no subprocess is spawned.

**Mutual Exclusion: solo + rotate**

`solo::1` and `rotate::1` are mutually exclusive: rotation requires live quota data from all candidate accounts to make an informed selection, but solo prevents live-fetching candidates. If both are present, the process exits 1 before any fetch begins. The error message references both parameter names.

**Display Filter Independence**

`solo::1` controls token consumption, not display. It never removes rows from the output table. Display filters (`only_active::`, `count::`, `offset::`, `min_5h::`, `min_7d::`, etc.) apply after fetch and remain fully independent of solo.

**Edge Cases**

| Scenario | Behavior |
|----------|----------|
| Current account is not owned | No account passes `is_current && is_owned`. All accounts display approximated data. Zero HTTP calls. |
| No active marker | `is_current` is false for all accounts. All accounts display approximated data. Zero HTTP calls. |
| Multiple owned accounts | Only the one that is also current gets live fetch. Others use `approximate_quota()`. |
| Current+owned but `occupied_elsewhere` | Solo gate passes (current+owned). G2/G4 occupancy gates still apply — refresh/touch may still skip. HTTP fetch still happens. |

**Allowed Compositions**

| Combination | Behavior |
|-------------|----------|
| `solo::1 live::1` | Allowed. Each monitor cycle: live fetch only current+owned; others show progressively stale approximated data. |
| `solo::1 refresh::1` | Allowed. Refresh subprocess fires only for current+owned account (if auth error). |
| `solo::1 touch::1` | Allowed. Touch subprocess fires only for current+owned account (if idle). |
| `solo::1 only_active::1` | Allowed. Orthogonal — solo controls fetch, `only_active::` controls display. |
| `solo::1 active::USER@MACHINE` | Allowed. `active::` writes local marker file — no API call. (Feature 064: `assign::1` is REMOVED; use `active::USER@MACHINE` instead.) |
| `solo::1 owner::0` | Allowed. `owner::0` writes local JSON — no API call. (Feature 064: `unclaim::1` is REMOVED; use `owner::0` instead.) |
| `solo::1 force::1` | Allowed. `force::` bypasses G5–G8 (write-side). Solo is read-side token conservation. Independent domains. |

### Acceptance Criteria

- **AC-01**: When `solo::1`, the HTTP quota fetch is issued only for the account where `is_current && is_owned`; all other accounts are served via `approximate_quota()` with no HTTP call.
- **AC-02**: When `solo::1`, the refresh subprocess is skipped for all accounts where `!is_current`; refresh fires only for the current+owned account (if auth error predicate matches).
- **AC-03**: When `solo::1`, the touch subprocess is skipped for all accounts where `!is_current`; touch fires only for the current+owned account (if idle predicate matches).
- **AC-04**: When `solo::0` (default), behavior is identical to omitting `solo::` — all owned accounts receive live API calls.
- **AC-05**: When the current account is not owned (`is_current && !is_owned`), no account passes the solo gate. All accounts display approximated data. Zero HTTP calls are made.
- **AC-06**: When no active marker exists (`is_current == false` for all accounts), all accounts display approximated data. Zero HTTP calls are made.
- **AC-07**: `solo::1 rotate::1` together exits 1 before any fetch; error message references both `"solo"` and `"rotate"` by name.
- **AC-08**: `solo::1 live::1` is allowed; the live monitor loop runs with only the current+owned account fetched live each cycle.
- **AC-09**: `solo::1 refresh::1` is allowed; refresh subprocess fires only for current+owned on auth error.
- **AC-10**: `solo::1 touch::1` is allowed; touch subprocess fires only for current+owned when idle.
- **AC-11**: Display filters (`only_active::`, `count::`, `offset::`, `min_5h::`, `min_7d::`, etc.) apply after solo fetch gating and remain fully independent; solo never removes rows from the output.
- **AC-12**: With `trace::1` and `solo::1`: each skipped account's fetch trace line ends with `solo-skip: approximated (age: Ns)`; refresh and touch traces for skipped accounts emit `solo-skip`; the current+owned account trace shows normal live fetch behavior.

### Features

| File | Relationship |
|------|-------------|
| [040_quota_measurement_history.md](040_quota_measurement_history.md) | `approximate_quota()` calls the Feature 040 polynomial approximation |
| [033_quota_cache.md](033_quota_cache.md) | `approximate_quota()` reads the single-point cache as fallback when history is absent |
| [036_account_ownership.md](036_account_ownership.md) | `is_owned` predicate from Feature 036 G1 gate; solo extends ownership-based routing |
| [038_usage_strategy_rotate.md](038_usage_strategy_rotate.md) | `rotate::1` is mutually exclusive with `solo::1` |
| [039_decision_algorithms.md](039_decision_algorithms.md) | Solo gates documented in fetch/refresh/touch decision tables |
| [009_token_usage.md](009_token_usage.md) | `.usage` is the only command exposing `solo::` |

### Parameters

| File | Relationship |
|------|-------------|
| [cli/param/060_solo.md](../cli/param/060_solo.md) | Full specification for `solo::` — type, default, valid values, compositions, gate placement |
| [cli/param/059_rotate.md](../cli/param/059_rotate.md) | `rotate::` is mutually exclusive with `solo::` |
| [cli/param/023_trace.md](../cli/param/023_trace.md) | `trace::` enables per-account solo-skip trace output |

### Sources

| File | Relationship |
|------|-------------|
| `src/usage/fetch.rs` | Solo gate at fetch site; `approximate_quota()` function; solo+rotate mutual exclusion check |
| `src/usage/refresh.rs` | Solo gate at refresh site — first gate in per-account loop |
| `src/usage/touch.rs` | Solo gate at touch site — first gate in per-account loop |
| `src/usage/params.rs` | `solo` field parsing — `parse_int_flag(cmd, "solo", 0) != 0`; mutual exclusion validation |
| `claude_profile_core/src/account.rs` | Storage layer — `read_history()` called by `approximate_quota()` |
