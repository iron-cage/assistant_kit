# Algorithm: Sort Strategies

### Purpose

Define how `.usage` orders accounts within each status group, and how `prefer_weekly` (strategy-weighted 7d capacity) is computed for `sort::renew`.

### Entry Points

- `src/usage/sort.rs` — `sort_indices()`, `relevant_quotas()`
- `sort::` and `prefer::` parameters

### Active Strategies

| Strategy | Primary sort key | Direction | Notes |
|---|---|---|---|
| `sort::name` | Account name | A→Z | Alphabetical |
| `sort::renew` *(default)* | `min(7d_reset_secs, sub_renewal_secs)` | Ascending (soonest renewal first) | Secondary: `prefer_weekly` ascending; tertiary: name asc |
| `sort::renews` | `sub_renewal_secs` | Ascending (soonest billing renewal first) | Secondary: name asc |

**Removed strategies:** `sort::endurance`, `sort::drain`, `sort::next` (alias), `sort::expires` — all removed during simplification. See MEMORY.md.

### `prefer_weekly(aq, prefer)` Computation

`prefer_weekly` is the strategy-weighted 7d remaining capacity used as sort key and eligibility gate 7.

| `prefer::` value | Formula |
|---|---|
| `any` *(default)* | `min(seven_day_left, seven_day_sonnet_left)` when Sonnet tier present; else `seven_day_left` |
| `son` | `seven_day_sonnet_left` when Sonnet tier present; else `0.0` (absent = ineligible) |
| `opus` | `seven_day_left` |

Where `seven_day_left = 100.0 - seven_day.utilization` and `seven_day_sonnet_left = 100.0 - seven_day_sonnet.utilization`.

### Sort Key Definitions

| Key | Source | `u64::MAX` when |
|-----|--------|-----------------|
| `7d_reset_secs` | `seven_day.resets_at` (ISO 8601 → unix secs) | absent |
| `sub_renewal_secs` | `renewal_at` (ISO 8601) or estimated from `org_created_at` | absent |

Source locations: `sort.rs:113-116` (`7d_reset_secs`), `sort.rs:117-121` (`sub_renewal_secs`).

### Group Invariant

Sorting is always applied within status groups — the 4-group partition (Green → h-exhausted → weekly-exhausted → Red) is never reordered by any strategy. `desc::` reverses row order within each group only.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | Full feature spec, `sort::`, `desc::`, `prefer::` parameters |
| [algorithm/003](003_quota_status_groups.md) | Status group partition |
| [algorithm/005](005_next_account_selection.md) | Positive selection uses same sort keys |
