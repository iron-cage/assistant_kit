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

#### `billing_type == "none"` Scope Boundary (BUG-332)

`status_group_of()`'s use of `billing_type == "none"` above is a deliberately `result`-independent classification (BUG-317's domain: is this account permanently dead for status-group purposes) and is correctly scoped as-is — it does not need pairing with `result`.

This is distinct from a different, narrower question some display logic answers: "does this account have no active subscription?" `billing_type == "none"` alone does NOT answer that question — it also occurs for live non-stripe (team/enterprise) accounts where `result = Ok(...)` with valid quota data (BUG-236). Evaluating `billing_type == "none"` in isolation for that narrower question, without pairing it with the `result` condition BUG-236 identified, produces wrong displayed values for those accounts (BUG-332: `render.rs`'s `~Renews` gate showed `"—"` for an account with a real, valid renewal date). See [invariant/011](../invariant/011_shared_predicate_consistency.md) for the full predicate definition and the non-compliant call sites.

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

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/011](../invariant/011_shared_predicate_consistency.md) | `billing_type=="none"` must pair with `result` when answering "no active subscription" (BUG-332) — this file's Dead-group gate is a correctly-scoped exception |
