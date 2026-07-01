# Algorithm: Quota Status Groups

### Scope

- **Purpose**: Define the quota status group partitioning algorithm for `.usage` display ordering.
- **Responsibility**: Documents the 4-group partition table, thresholds, and fixed display order for `status_group_of()`.
- **In Scope**: `status_group_of()` logic; Green/h-exhausted/weekly-exhausted/Dead classification; threshold values; cancelled billing gate.
- **Out of Scope**: Sort strategies within groups (→ algorithm/007); next-account selection (→ algorithm/005).

### Abstract

Partition accounts into 4 fixed display groups for `.usage` output ordering and color coding.

### Algorithm

#### Entry Point

`src/usage/sort.rs:31-48` — `status_group_of(aq)`

#### Group Table

| 5h Left | 7d Left | Condition | Group | Emoji | Display position |
|---|---|---|---|---|---|
| > 15% | > 5% | both available | Green | 🟢 | top |
| ≤ 15% | > 5% | 5h gone, 7d ok | h-exhausted | 🟡 | 2nd |
| — | ≤ 5% | 7d gone (any 5h) | weekly-exhausted | 🟡 | 3rd |
| — | — | `result=Err` OR `billing_type="none"` | Dead | 🔴 | bottom |

#### Thresholds

```
5h threshold:  five_hour_left(aq) > 15.0    (15% of 5h window)
7d threshold:  seven_day_left(aq) > 5.0     (5% of 7d window)
```

`five_hour_left(aq)` = `100.0 - five_hour.utilization`.
`seven_day_left(aq)` = uses raw 7d value (NOT `prefer_weekly`) — this is a global partition, strategy-independent. Fix BUG-299: must use `seven_day_left` (raw), not the strategy-weighted `prefer_weekly`.

#### Group Order is Fixed

`desc::` only reverses row order *within* each group, never between groups. The Green→h-exhausted→weekly-exhausted→Dead order is invariant.

#### Weekly-Exhausted Group (G3)

Any account with `7d Left ≤ 5%` is G3 (weekly-exhausted 🟡), **regardless of 5h Left**. This includes accounts where both quotas are exhausted (`5h Left ≤ 15%` AND `7d Left ≤ 5%`): the 7d constraint is the binding factor. When the 7d resets, the 5h window will have already reset many times over (5h << 7 days), so both-exhausted accounts have identical recovery behavior to weekly-exhausted accounts. Fix(BUG-321): the former code mapped `( false, false )` to `StatusGroup::Red`, incorrectly classifying these recoverable accounts as dead.

#### Dead Accounts (G4)

Accounts with `result = Err(...)` OR `billing_type = "none"` (confirmed cancelled subscription) are classified as Dead (G4, 🔴). These are unrecoverable without external action:
- `result = Err` — token missing, expired, auth error, or fetch failed
- `billing_type = "none"` — subscription permanently cancelled; quota can never reset

The `billing_type` gate fires before quota thresholds. Fix(BUG-317): `status_group_of()` checks `billing_type == "none"` via `account.as_ref().is_some_and()` before evaluating 5h/7d thresholds. `account = None` (API fetch failed) is NOT treated as cancelled — absent data is ambiguous.

### Features

| File | Relationship |
|------|-------------|
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | Sort strategies context |
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 3 (legacy reference) |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/004](004_eligibility_gates.md) | Eligibility gates use same thresholds |
| [algorithm/005](005_next_account_selection.md) | Green-only selection |

### Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/001](../pitfall/001_quota_gate_pitfalls.md) | Pitfall 4 — billing_type check in status/filter functions |
