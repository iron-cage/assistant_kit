# Algorithm: Next-Account Positive Selection

### Scope

- **Purpose**: Define the positive selection algorithm for next-account footer recommendation and auto-switch.
- **Responsibility**: Documents the 3-step selection process: status group partition, sort strategy application, and first-eligible winner.
- **In Scope**: `find_next_for_strategy()` logic; sort key computation; Green-only selection invariant; `prefer_weekly` computation.
- **Out of Scope**: Eligibility gates (→ algorithm/004); status group computation (→ algorithm/003); sort strategy definitions (→ algorithm/007).

### Abstract

Select the **winner** account for footer recommendation and auto-switch from among eligible accounts.

### Algorithm

#### Entry Point

`src/usage/sort_next.rs:63-103` — `find_next_for_strategy(accounts, strategy, prefer, now_secs, gate_ownership, selected_provider)`

#### Algorithm (3 steps)

#### Step 1 — Partition into status groups (display order)

All accounts are partitioned into 4 status groups in fixed order: Green → h-exhausted → weekly-exhausted → Red. Selection is restricted to the Green group — gates 5 and 7 enforce this (non-Green accounts fail at least one gate).

#### Step 2 — Sort within Green by strategy

Every strategy below is additionally prefixed with a **leading `reserve` key** (non-reserved before reserved) ahead of its own primary key — see [algorithm/007](007_sort_strategies.md). The table shows each strategy's own keys only; the `reserve` prefix applies uniformly to all three and is omitted here to avoid duplicating algorithm/007's table.

| Strategy | Primary key | Direction | Secondary key | Direction | Tertiary |
|---|---|---|---|---|---|
| `sort::name` | account name | ascending (A→Z) | — | — | — |
| `sort::renew` | `min(7d_reset_secs, sub_renewal_secs)` | ascending (soonest first) | `prefer_weekly(aq, prefer)` | ascending (lowest first) | name asc |
| `sort::renews` | `sub_renewal_secs` | ascending (soonest first) | name | ascending | — |

Key definitions:
- `7d_reset_secs`: seconds until 7d quota resets (`seven_day.resets_at`; `u64::MAX` if absent). Source: `sort.rs:113-116`.
- `sub_renewal_secs`: seconds until subscription billing renewal (`renewal_at` or estimated `org_created_at`; `u64::MAX` if absent). Source: `sort.rs:138-152` (`sort::renew`), `sort.rs:179-187` (`sort::renews`).
- `prefer_weekly`: model-aware 7d capacity via `relevant_quotas(aq, prefer).1` (`format.rs`). See [algorithm/007](007_sort_strategies.md).

#### Step 3 — First eligible wins, preferring weekly headroom

Walk the sorted list from position 0. The first account passing all 10 eligibility gates (see [algorithm/004](004_eligibility_gates.md)) is the winner — marked `→` in the table, shown in footer `Next` line. If no account passes, result is `None` (no recommendation; auto-switch returns error). Because `reserve` is a leading sort key (Step 2) rather than a gate, a reserved account is only ever reached by this walk once every non-reserved candidate has already failed a gate — "first eligible wins" needs no change to produce "reserved accounts are picked only when nothing else qualifies."

The walk runs **twice** (`find_preferring_headroom`, BUG-558), identically except for Gate 7's weekly floor:

| Pass | Gate 7 floor | Runs when |
|---|---|---|
| 1 | `seven_day_left > ROTATION_HEADROOM_THRESHOLD` (15%) | always |
| 2 | `seven_day_left > WEEKLY_EXHAUSTION_THRESHOLD` (3%) | only when pass 1 returned `None` |

Both passes walk the *same* sorted slice, so Step 2's strategy ordering is preserved exactly — headroom narrows the candidate set, it never reorders it. Under `sort::renew` this is what stops the soonest-reset key from electing an account with minutes of usable life left merely because its reset is imminent: an account at 5% weekly and an account at 40% weekly are both above the 3% exclusion floor, and only pass 1 distinguishes them.

The second pass is load-bearing, not a nicety: a single raised gate would make `rotate::1` report "no eligible account to rotate to" ([feature/038](../feature/038_usage_strategy_rotate.md) AC-03) the moment every account fell below 15%, converting a degraded-but-working fleet into a hard failure.

#### Why `sort::renew` uses ascending `prefer_weekly` as secondary key

Lower weekly capacity = account benefits most from the upcoming renewal event. An account at 10% weekly capacity benefits more from an imminent renewal than one at 50%. This ensures the recommendation prioritizes accounts whose renewal will have the greatest regenerative impact.

### Features

| File | Relationship |
|------|-------------|
| [feature/039_decision_algorithms.md](../feature/039_decision_algorithms.md) | Table 5 (legacy reference) |
| [feature/020_usage_sort_strategies.md](../feature/020_usage_sort_strategies.md) | `sort::`, `prefer::` parameters |
| [feature/038_usage_strategy_rotate.md](../feature/038_usage_strategy_rotate.md) | Auto-switch uses this winner |
| [feature/070_account_claim_and_reservation_control.md](../feature/070_account_claim_and_reservation_control.md) | Gate 9 (`claim_lock`) added to Step 3; `reserve` leading key added to Step 2 |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/003](003_quota_status_groups.md) | Status groups (Green-only selection) |
| [algorithm/004](004_eligibility_gates.md) | Eligibility gates applied in step 3 |
| [algorithm/007](007_sort_strategies.md) | Sort key computation |
