# Test: Algorithm 004 — Next-Account Eligibility Gates

Algorithm correctness test cases for `docs/algorithm/004_eligibility_gates.md`. Each AC case verifies one gate behavior from the Gate Table. Focus: Gate 7 (weekly-exhausted) model-agnostic invariant exposed by BUG-324.

### AC Coverage Index

| AC | Gate | Condition | Notes |
|----|------|-----------|-------|
| AC-01 | G7 fires (raw 7d exhausted) | `seven_day_left = 4.0 ≤ 5.0` → skipped | Unit test |
| AC-02 | G7 passes (divergent 7d/7d_son, BUG-324) | `seven_day_left = 31.0 > 5.0`, `7d_son_left = 0%` → eligible | BUG-324 fix |
| AC-03 | G7 boundary | `seven_day_left = 5.0` exactly at threshold → skipped | Boundary |
| AC-04 | G7 model-agnostic invariant | `prefer::any` and `prefer::opus` produce same gate result | Invariant |
| AC-05 | G3b cancelled subscription | `billing_type = "none"` → skipped (BUG-317) | Unit test |
| AC-06 | G8 foreign-owned (`gate_ownership=true`) | `is_owned = false` → skipped | Unit test |
| AC-07 | G8 foreign-owned (`gate_ownership=false`) | `is_owned = false` → eligible | Unit test |

### Test Case Index

| ID | Test Name | Gate | Category |
|----|-----------|------|----------|
| AC-01 | G7 raw 7d exhausted → skipped | G7 | Weekly exhaustion |
| AC-02 | G7 divergent 7d/7d_son passes (BUG-324) | G7 | Model-agnostic |
| AC-03 | G7 boundary at 5.0% → skipped | G7 | Boundary |
| AC-04 | G7 same result under any/opus | G7 | Invariant |
| AC-05 | G3b cancelled subscription skipped | G3b | Cancelled |
| AC-06 | G8 foreign-owned skipped (gate_ownership=true) | G8 | Ownership |
| AC-07 | G8 foreign-owned eligible (gate_ownership=false) | G8 | Ownership |

**Total:** 7 AC cases

---

### AC-01: Gate 7 — raw 7d exhausted, account skipped

- **Given:** An `AccountQuota` with `seven_day_util=96%` → `seven_day_left = 4.0`. `seven_day_sonnet = None`. Non-current, non-active.
- **When:** Gate 7 evaluates `seven_day_left(aq) > WEEKLY_EXHAUSTION_THRESHOLD` in `find_next_for_strategy()`.
- **Then:** `4.0 > 5.0 = false` → gate fires → account skipped. Weekly-exhausted accounts have negligible remaining capacity.
- **Source fn:** `mre_bug292_renew_skips_weekly_exhausted_even_with_soonest_renewal` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [algorithm/004_eligibility_gates.md Gate 7](../../../docs/algorithm/004_eligibility_gates.md)

---

### AC-02: Gate 7 — divergent `7d/7d_son` with `seven_day_sonnet` present passes gate (BUG-324)

- **Given:** An `AccountQuota` with `seven_day_util=69%` → `seven_day_left = 31.0`, `seven_day_sonnet_util=100%` → `7d_son_left = 0%`. `prefer::any` in effect. Non-current, non-active.
- **When:** Gate 7 evaluates `seven_day_left(aq) > WEEKLY_EXHAUSTION_THRESHOLD`.
- **Then:** `31.0 > 5.0 = true` → gate does NOT fire → account eligible. Before Fix(BUG-324): `prefer_weekly(aq, Any) = min(31.0, 0.0) = 0.0 ≤ 5.0` — gate would fire and block this green account.
- **Note:** Same class as BUG-299. Eligibility is model-agnostic; `apply_model_override()` handles model selection post-rotation.
- **Source fn:** `mre_bug324_green_account_eligible_when_7d_son_exhausted` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [algorithm/004_eligibility_gates.md Gate 7](../../../docs/algorithm/004_eligibility_gates.md)

---

### AC-03: Gate 7 — boundary at exactly `WEEKLY_EXHAUSTION_THRESHOLD`, account skipped

- **Given:** An `AccountQuota` with `seven_day_util=95%` → `seven_day_left = 5.0`. Non-current, non-active.
- **When:** Gate 7 evaluates `seven_day_left(aq) > 5.0`.
- **Then:** `5.0 > 5.0 = false` → gate fires → account skipped. At exactly the threshold the account is considered exhausted.
- **Source fn:** `mre_bug292_renew_skips_weekly_exhausted_even_with_soonest_renewal` (in `tests/usage/sort_next_tests.rs` — uses value below threshold; boundary-exact test TBD)
- **Source:** [algorithm/004_eligibility_gates.md Gate 7](../../../docs/algorithm/004_eligibility_gates.md)

---

### AC-04: Gate 7 — model-agnostic invariant: `prefer::any` and `prefer::opus` produce same result

- **Given:** An `AccountQuota` with `seven_day_util=69%` → `seven_day_left = 31.0`, `seven_day_sonnet_util=100%` → `7d_son_left = 0%`. Non-current, non-active.
- **When:** `find_next_for_strategy` called with `PreferStrategy::Any`, then with `PreferStrategy::Opus`.
- **Then:** Both return `Some(0)` — same result. Gate 7 uses `seven_day_left(aq)` which is independent of `PreferStrategy`. `PreferStrategy` affects sort order (tiebreaker) but not eligibility.
- **Note:** Before Fix(BUG-324), `prefer_weekly(aq, Any) = 0.0` (gate fires) while `prefer_weekly(aq, Opus) = 31.0` (gate passes) — eligibility depended on model preference, violating model-agnostic invariant.
- **Source fn:** `mre_bug324_green_account_eligible_when_7d_son_exhausted` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [algorithm/004_eligibility_gates.md Gate 7](../../../docs/algorithm/004_eligibility_gates.md)

---

### AC-05: Gate 3b — cancelled subscription skipped (BUG-317)

- **Given:** An `AccountQuota` with `account.billing_type = "none"`. Non-current, non-active, all other gates pass.
- **When:** Gate 3b evaluates `aq.account.as_ref().is_some_and(|a| a.billing_type == "none")`.
- **Then:** Account skipped. Cancelled subscriptions are permanently unusable for rotation regardless of remaining quota.
- **Source fn:** `mre_bug317_cancelled_not_recommended_by_find_next` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [algorithm/004_eligibility_gates.md Gate 3b](../../../docs/algorithm/004_eligibility_gates.md)

---

### AC-06: Gate 8 — foreign-owned account skipped when `gate_ownership=true`

- **Given:** An `AccountQuota` with `is_owned = false` (owned by different machine). Non-current, non-active, all other gates pass.
- **When:** `find_next_for_strategy(&accounts, strategy, prefer, now, true)` — `gate_ownership=true`.
- **Then:** Account skipped. Gate 8 `!gate_ownership || aq.is_owned` evaluates to `!true || false = false`.
- **Source fn:** `test_gate_ownership_true_skips_non_owned` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [algorithm/004_eligibility_gates.md Gate 8](../../../docs/algorithm/004_eligibility_gates.md)

---

### AC-07: Gate 8 — foreign-owned account eligible when `gate_ownership=false`

- **Given:** Same `AccountQuota` as AC-06 (`is_owned = false`).
- **When:** `find_next_for_strategy(&accounts, strategy, prefer, now, false)` — `gate_ownership=false`.
- **Then:** Account eligible. Gate 8 `!gate_ownership || aq.is_owned` evaluates to `!false || false = true`.
- **Note:** Footer recommendation uses `gate_ownership=false` — non-owned accounts appear as recommendations (BUG-320 fix).
- **Source fn:** `test_gate_ownership_false_allows_non_owned` (in `tests/usage/sort_next_tests.rs`)
- **Source:** [algorithm/004_eligibility_gates.md Gate 8](../../../docs/algorithm/004_eligibility_gates.md)
