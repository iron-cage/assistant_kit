# Algorithm: Next-Account Positive Selection

### Purpose

Select the **winner** account for footer recommendation and auto-switch from among eligible accounts.

### Entry Point

`src/usage/sort_next.rs:46-83` — `find_next_for_strategy(strategy, accounts, prefer, gate_ownership, now_secs)`

### Algorithm (3 steps)

#### Step 1 — Partition into status groups (display order)

All accounts are partitioned into 4 status groups in fixed order: Green → h-exhausted → weekly-exhausted → Red. Selection is restricted to the Green group — gates 5 and 7 enforce this (non-Green accounts fail at least one gate).

#### Step 2 — Sort within Green by strategy

| Strategy | Primary key | Direction | Secondary key | Direction | Tertiary |
|---|---|---|---|---|---|
| `sort::name` | account name | ascending (A→Z) | — | — | — |
| `sort::renew` | `min(7d_reset_secs, sub_renewal_secs)` | ascending (soonest first) | `prefer_weekly(aq, prefer)` | ascending (lowest first) | name asc |
| `sort::renews` | `sub_renewal_secs` | ascending (soonest first) | name | ascending | — |

Key definitions:
- `7d_reset_secs`: seconds until 7d quota resets (`seven_day.resets_at`; `u64::MAX` if absent). Source: `sort.rs:113-116`.
- `sub_renewal_secs`: seconds until subscription billing renewal (`renewal_at` or estimated `org_created_at`; `u64::MAX` if absent). Source: `sort.rs:117-121`.
- `prefer_weekly`: model-aware 7d capacity via `relevant_quotas(aq, prefer).1` (`format.rs`). See [algorithm/007](007_sort_strategies.md).

#### Step 3 — First eligible wins

Walk the sorted list from position 0. The first account passing all 8 eligibility gates (see [algorithm/004](004_eligibility_gates.md)) is the winner — marked `→` in the table, shown in footer `Next` line. If no account passes, result is `None` (no recommendation; auto-switch returns error).

### Why `sort::renew` uses ascending `prefer_weekly` as secondary key

Lower weekly capacity = account benefits most from the upcoming renewal event. An account at 10% weekly capacity benefits more from an imminent renewal than one at 50%. This ensures the recommendation prioritizes accounts whose renewal will have the greatest regenerative impact.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 5 (legacy reference) |
| [algorithm/003](003_quota_status_groups.md) | Status groups (Green-only selection) |
| [algorithm/004](004_eligibility_gates.md) | Eligibility gates applied in step 3 |
| [algorithm/007](007_sort_strategies.md) | Sort key computation |
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | `sort::`, `prefer::` parameters |
| [feature/038_usage_strategy_rotate.md](../feature/038_usage_strategy_rotate.md) | Auto-switch uses this winner |
