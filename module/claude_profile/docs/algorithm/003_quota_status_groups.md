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
| ≤ 15% | ≤ 5% OR `result=Err` | Red | 🔴 | bottom |

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

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | Sort strategies context |
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 3 (legacy reference) |
| [algorithm/004](004_eligibility_gates.md) | Eligibility gates use same thresholds |
| [algorithm/005](005_next_account_selection.md) | Green-only selection |
