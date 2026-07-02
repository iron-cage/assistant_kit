# Pitfall Tests: Quota Gate Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/001_quota_gate_pitfalls.md`
is in place and prevents the described quota misclassification failure mode.

**Source:** [docs/pitfall/001_quota_gate_pitfalls.md](../../../docs/pitfall/001_quota_gate_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | Status groups must use raw `seven_day_left`, not `prefer_weekly` | BUG-299 | `mre_bug321_both_exhausted_status_emoji_is_yellow`, `test_cc_prefer_sonnet_absent_tier_eligible` |
| PP-2 | Absent Sonnet tier (`None`) is NOT exhaustion — `map_or(0.0)` conflation | BUG-300 | `mre_bug300_model_override_absent_sonnet_no_override`, `ac1_absent_tier_with_opus_session_restores_sonnet` |
| PP-3 | `son_available` must check utilization, not just window state | BUG-301 | `mre_bug285_idle_check_uses_resets_at_as_wrong_oracle` |
| PP-4 | Status group and eligibility thresholds are model-agnostic | BUG-324 | `mre_bug324_green_account_eligible_when_7d_son_exhausted`, `test_cc_gate7_boundary_exactly_5pct_skipped_in_eligibility` |
| PP-5 | Cancelled subscription is `Dead` regardless of quota | BUG-317 | `mre_bug317_cancelled_status_emoji_is_red`, `mre_bug317_cancelled_not_recommended_by_find_next` |

---

### PP-1: Status groups use raw `seven_day_left` for partition boundaries

- **Given:** An account with `seven_day_left = 32%` (> 5%) but `seven_day_sonnet = None` →
  `prefer_weekly(any) = min(32, 0) = 0%` (≤ 5%).
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `HExhausted` (not `WeeklyExhausted`) — the partition uses raw
  `seven_day_left`, not the model-weighted `prefer_weekly`. Fix BUG-299.
- **Rule:** Status group partition and eligibility gate boundaries are always model-agnostic.
  `prefer_weekly` is only a sort tiebreak. Never use strategy-weighted values for group or
  gate boundary decisions.
- **Note:** Same rule applies to Gate 7 in `find_first_eligible` — BUG-324 was the same
  class of error in `sort_next.rs`.
- **Source fn:** `mre_bug321_both_exhausted_status_emoji_is_yellow` in
  `tests/usage/format_tests.rs`; `test_cc_prefer_sonnet_absent_tier_eligible` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [pitfall/001_quota_gate_pitfalls.md §P1](../../../docs/pitfall/001_quota_gate_pitfalls.md)

---

### PP-2: Absent Sonnet tier (`None`) is NOT Sonnet exhaustion

- **Given:** `OauthUsageData { seven_day_sonnet: None }` — account has no Sonnet tier.
- **When:** `apply_model_override()` or `recommended_model()` evaluates Sonnet remaining
  capacity.
- **Then:** The `None` case restores/keeps Sonnet session model (conservative) — it does NOT
  treat `None` as `0.0` remaining and trigger Opus override. Fix BUG-300 / BUG-311.
- **Rule:** Always use `if let Some(ref sonnet)` guard before comparing quota values. `None`
  and `Some { utilization ≥ 85% }` are opposite operational states.
- **Source fn:** `mre_bug300_model_override_absent_sonnet_no_override` in
  `tests/usage/api_tests_a.rs`; `ac1_absent_tier_with_opus_session_restores_sonnet` in
  `tests/usage/api_tests_b.rs`
- **Source:** [pitfall/001_quota_gate_pitfalls.md §P2](../../../docs/pitfall/001_quota_gate_pitfalls.md)

---

### PP-3: `son_available` checks utilization, not just window state

- **Given:** An account where `seven_day_sonnet.resets_at = Some(...)` (window is active)
  but `utilization = 60%` (40% remaining — well above threshold).
- **When:** `resolve_model(Auto, aq)` evaluates Sonnet availability.
- **Then:** Sonnet is selected — available capacity exists. Before Fix(BUG-301), only
  `resets_at.is_none()` was checked; active window with available quota was treated as
  Haiku-only, wasting quota capacity as the window timer ran down.
- **Rule:** Sonnet availability requires two checks: (1) window active (`resets_at = Some`)
  AND (2) utilization below threshold.
- **Source fn:** `mre_bug285_idle_check_uses_resets_at_as_wrong_oracle` in
  `tests/usage/api_tests_b.rs`
- **Source:** [pitfall/001_quota_gate_pitfalls.md §P3](../../../docs/pitfall/001_quota_gate_pitfalls.md)

---

### PP-4: Eligibility gates are model-agnostic — raw `seven_day_left` only

- **Given:** Account with `seven_day_left = 31%` (> 5%) and `seven_day_sonnet_left = 0%`.
  `prefer_weekly(any) = min(31, 0) = 0%`.
- **When:** Gate 7 in `find_first_eligible` evaluates the account.
- **Then:** Account passes Gate 7 — `seven_day_left(31%) > 5.0 = true`. Before Fix(BUG-324),
  `prefer_weekly` was used, blocking this green account.
- **Rule:** `find_first_eligible` Gate 7 (and all group/gate boundaries) must use raw
  `seven_day_left` — never `prefer_weekly` or any model-weighted metric.
- **Source fn:** `mre_bug324_green_account_eligible_when_7d_son_exhausted`,
  `test_cc_gate7_boundary_exactly_5pct_skipped_in_eligibility` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [pitfall/001_quota_gate_pitfalls.md §P4](../../../docs/pitfall/001_quota_gate_pitfalls.md)

---

### PP-5: Cancelled subscription is always `Dead` regardless of quota values

- **Given:** Account with `billing_type = "none"` (no active subscription) and
  otherwise healthy quota numbers.
- **When:** `status_group_of(aq)` or `find_next_for_strategy` is called.
- **Then:** `status_group_of` returns `Dead`; account is NOT recommended by
  `find_next_for_strategy`. Fix BUG-317.
- **Rule:** `billing_type = "none"` must be checked before quota-based classification.
  A cancelled account with 50% quota remaining is still operationally dead.
- **Source fn:** `mre_bug317_cancelled_status_emoji_is_red` in `tests/usage/format_tests.rs`;
  `mre_bug317_cancelled_not_recommended_by_find_next` in `tests/usage/sort_next_tests.rs`
- **Source:** [pitfall/001_quota_gate_pitfalls.md §P5](../../../docs/pitfall/001_quota_gate_pitfalls.md)
