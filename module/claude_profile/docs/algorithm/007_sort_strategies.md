# Algorithm: Sort Strategies

### Scope

- **Purpose**: Define the sort strategy algorithms for ordering accounts within status groups in `.usage` output.
- **Responsibility**: Documents the three active sort strategies, `prefer_weekly` computation, sort key definitions for `sort_indices()`, and the `reserve` leading key applied ahead of every strategy.
- **In Scope**: `sort_indices()` and `relevant_quotas()` logic; `sort::name/renew/renews` strategies; `prefer::` parameter computation; group invariant; `reserve` leading sort key (soft deprioritization, all strategies).
- **Out of Scope**: Status group computation (→ algorithm/003); next-account selection (→ algorithm/005); `reserve` field write path and `claim_lock` (→ feature/070).

### Abstract

Define how `.usage` orders accounts within each status group, and how `prefer_weekly` (strategy-weighted 7d capacity) is computed for `sort::renew`.

### Algorithm

#### Entry Points

- `src/usage/sort.rs` — `sort_indices()`, `relevant_quotas()`
- `sort::` and `prefer::` parameters

#### Active Strategies

All three strategies below are prefixed with the `reserve` leading key (see next subsection) — the "Primary sort key" column shows each strategy's own key, applied after that prefix.

| Strategy | Primary sort key | Direction | Notes |
|---|---|---|---|
| `sort::name` | Account name | A→Z | Alphabetical; prefixed by `reserve` leading key |
| `sort::renew` *(default)* | `min(7d_reset_secs, sub_renewal_secs)` | Ascending (soonest renewal first) | Secondary: `prefer_weekly` ascending; tertiary: name asc; prefixed by `reserve` leading key |
| `sort::renews` | `sub_renewal_secs` | Ascending (soonest billing renewal first) | Secondary: name asc; prefixed by `reserve` leading key |

**Removed strategies:** `sort::endurance`, `sort::drain`, `sort::next` (alias), `sort::expires` — all removed during simplification. See MEMORY.md.

#### `reserve` Leading Key

Every strategy's sort key is prepended with `reserve` as the outermost (leading) key: effective sort key = `(reserve, <strategy's own key>)`. `reserve = false` sorts before `reserve = true` — non-reserved accounts always precede reserved ones, and ordering within each of the two `reserve` groups follows the strategy's own key unchanged. This is a **soft deprioritization**: no eligibility gate is added, so a reserved account still passes Gates 1–9 (see [algorithm/004](004_eligibility_gates.md)) like any other candidate. Combined with the unchanged "first eligible wins, else `None`" walk in [algorithm/005](005_next_account_selection.md) Step 3, this reordering alone is sufficient to produce "reserved accounts are selected only when no non-reserved eligible account remains" — no branching logic beyond the sort key is needed. `reserve` is set via `.accounts reserve::`; see [feature/070](../feature/070_account_claim_and_reservation_control.md) for the full design and properties table.

#### `prefer_weekly(aq, prefer)` Computation

`prefer_weekly` is the strategy-weighted 7d remaining capacity used as sort key (tiebreak within groups). Eligibility gate 7 uses model-agnostic `seven_day_left` (Fix BUG-324).

| `prefer::` value | Formula |
|---|---|
| `any` *(default)* | `min(seven_day_left, seven_day_sonnet_left)` when Sonnet tier present; else `seven_day_left` |
| `son` | `seven_day_sonnet_left` when Sonnet tier present; else `0.0` (absent = sorts last; eligibility model-agnostic) |
| `opus` | `seven_day_left` |

Where `seven_day_left = 100.0 - seven_day.utilization` and `seven_day_sonnet_left = 100.0 - seven_day_sonnet.utilization`.

#### Sort Key Definitions

| Key | Source | `u64::MAX` when |
|-----|--------|-----------------|
| `7d_reset_secs` | `seven_day.resets_at` (ISO 8601 → unix secs) | absent |
| `sub_renewal_secs` | `renewal_at` (ISO 8601) or estimated from `org_created_at` | absent |

Source locations: `sort.rs:113-116` (`7d_reset_secs`), `sort.rs:117-121` (`sub_renewal_secs`).

#### Group Invariant

Sorting is always applied within status groups — the 4-group partition (Green → h-exhausted → weekly-exhausted → Dead) is never reordered by any strategy. Both-exhausted accounts (5h ≤ 15% AND 7d ≤ 5%) merge into G3 weekly-exhausted — the 7d constraint is binding in both cases. `desc::` reverses row order within each group only.

### Features

| File | Relationship |
|------|-------------|
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | Full feature spec, `sort::`, `desc::`, `prefer::` parameters |
| [feature/070_account_claim_and_reservation_control.md](../feature/070_account_claim_and_reservation_control.md) | `reserve` leading sort key — full design and properties table |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/003](003_quota_status_groups.md) | Status group partition |
| [algorithm/005](005_next_account_selection.md) | Positive selection uses same sort keys |
