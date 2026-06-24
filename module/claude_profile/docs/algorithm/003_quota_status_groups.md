# Algorithm: Quota Status Groups

### Purpose

Partition accounts into 4 fixed display groups for `.usage` output ordering and color coding.

### Entry Point

`src/usage/sort.rs:31-48` — `status_group_of(aq)`

### Group Table

| 5h Left | 7d Left | Group | Emoji | Display position |
|---|---|---|---|---|
| > 15% | > 5% | Green | 🟢 | top |
| ≤ 15% | > 5% | h-exhausted | 🟡 | 2nd |
| > 15% | ≤ 5% | weekly-exhausted | 🟡 | 3rd |
| ≤ 15% | ≤ 5% OR `result=Err` OR `billing_type="none"` | Red | 🔴 | bottom |

### Thresholds

```
5h threshold:  five_hour_left(aq) > 15.0    (15% of 5h window)
7d threshold:  seven_day_left(aq) > 5.0     (5% of 7d window)
```

`five_hour_left(aq)` = `100.0 - five_hour.utilization`.
`seven_day_left(aq)` = uses raw 7d value (NOT `prefer_weekly`) — this is a global partition, strategy-independent. Fix BUG-299: must use `seven_day_left` (raw), not the strategy-weighted `prefer_weekly`.

### Group Order is Fixed

`desc::` only reverses row order *within* each group, never between groups. The Green→h-exhausted→weekly-exhausted→Red order is invariant.

### Error Accounts

Accounts with `result = Err(...)` are classified as Red regardless of quota percentages.

### Cancelled Accounts

Accounts with `billing_type = "none"` (confirmed cancelled subscription) are classified as Red regardless of quota percentages. The `billing_type` gate fires before quota thresholds — even if the API returns healthy quota values for a cancelled account, the account is permanently unusable and must appear in the Red group. Fix(BUG-317): `status_group_of()` checks `billing_type == "none"` via `account.as_ref().is_some_and()` before evaluating 5h/7d thresholds. `account = None` (API fetch failed) is NOT treated as cancelled — absent data is ambiguous.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | Sort strategies context |
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 3 (legacy reference) |
| [algorithm/004](004_eligibility_gates.md) | Eligibility gates use same thresholds |
| [algorithm/005](005_next_account_selection.md) | Green-only selection |
| [pitfall/001](../pitfall/001_quota_gate_pitfalls.md) | Pitfall 4 — billing_type check in status/filter functions |
