# Algorithm 005: Next-Account Positive Selection

AC test cases for `docs/algorithm/005_next_account_selection.md`. Tests
`find_next_for_strategy(strategy, accounts, prefer, gate_ownership, now_secs)` in
`src/usage/sort_next.rs`.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Eligible non-current account selected; all-current → None | Nominal | ✅ |
| AC-2 | Occupied-elsewhere accounts skipped (all strategies) | Nominal | ✅ |
| AC-3 | h-exhausted accounts skipped (all strategies) | Nominal | ✅ |
| AC-4 | `five_hour = None` is NOT h-exhausted → eligible | BUG-299 class | ✅ |
| AC-5 | `is_active` (current) accounts skipped | Nominal | ✅ |
| AC-6 | Token-expired accounts skipped | Nominal | ✅ |
| AC-7 | Error result accounts skipped | Nominal | ✅ |
| AC-8 | `gate_ownership=true` rejects non-owned; `=false` allows non-owned | Nominal | ✅ |
| AC-9 | `sort::renew` — subscription renewal sooner than 7d reset ranks first (BUG-229) | Regression | ✅ |
| AC-10 | `sort::renews` — ascending subscription renewal order | Nominal | ✅ |
| AC-11 | `sort::renew` tiebreaker: alphabetical when renewal equal (BUG-260 fix) | Regression | ✅ |
| AC-12 | `sort::renew` skips weekly-exhausted even with soonest renewal (BUG-292) | Regression | ✅ |
| AC-13 | Cancelled subscription (`billing_type = "none"`) not recommended (BUG-317) | Regression | ✅ |
| AC-14 | Gate 7 boundary: `five_hour.utilization = 85.0` exactly → skipped (Gate 4) | Boundary | ✅ |
| AC-15 | BUG-324: divergent `7d/7d_son` — raw `seven_day_left` used for Gate 7 | Regression | ✅ |

---

### AC-1: Eligible non-current account selected; all-current → None

- **Given:** Two accounts — one current (`is_active=true`), one non-current and eligible.
- **When:** `find_next_for_strategy` runs with any strategy.
- **Then:** Returns the non-current eligible account. When all accounts are current, returns `None`.
- **Source fn:** `test_find_next_for_strategy_some_when_eligible_none_when_all_current` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-2: Occupied-elsewhere accounts skipped by all strategies

- **Given:** An account with `is_occupied_elsewhere = true`.
- **When:** `find_next_for_strategy` runs with `sort::name`, `sort::renew`, and `sort::renews`.
- **Then:** The occupied account is skipped for all three strategies.
- **Source fn:** `test_all_strategies_skip_occupied_elsewhere` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-3: h-exhausted accounts skipped by all strategies

- **Given:** An account with `five_hour.utilization ≥ 85.0` (≤ 15% left).
- **When:** `find_next_for_strategy` runs with all strategies.
- **Then:** The h-exhausted account is skipped for all three strategies. Gate 4 fires.
- **Source fn:** `test_all_strategies_skip_h_exhausted` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-4: `five_hour = None` is NOT h-exhausted → eligible

- **Given:** An account with `five_hour = None` (no 5h period data).
- **When:** `find_next_for_strategy` runs.
- **Then:** The account is NOT skipped — absence of 5h data is treated conservatively as 100%
  available. Gate 4 does not fire when `five_hour` is `None`.
- **Source fn:** `test_cc_five_hour_none_not_h_exhausted` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-5: `is_active` (current) accounts skipped

- **Given:** An account with `is_active = true` (current session account).
- **When:** `find_next_for_strategy` runs.
- **Then:** The active account is skipped — the next-account recommendation excludes the
  account already in use.
- **Source fn:** `test_cc_is_active_skips_account` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-6: Token-expired accounts skipped

- **Given:** An account with `Ok` result but token expired (`expires_at_ms < now_ms`).
- **When:** `find_next_for_strategy` runs.
- **Then:** The expired account is skipped regardless of quota levels.
- **Source fn:** `test_cc_expired_ok_account_skipped` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-7: Error result accounts skipped

- **Given:** An account with `result = Err(...)`.
- **When:** `find_next_for_strategy` runs.
- **Then:** The error account is skipped — it cannot be reliably recommended.
- **Source fn:** `test_cc_err_account_skipped_via_find_next_for_strategy` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-8: `gate_ownership` controls non-owned account eligibility

- **Given:** An account with `is_owned = false`.
- **When (gate_ownership=true):** `find_next_for_strategy` with `gate_ownership=true`.
- **Then:** Non-owned account is skipped (Gate 8 fires).
- **When (gate_ownership=false):** `find_next_for_strategy` with `gate_ownership=false`.
- **Then:** Non-owned account is eligible (Gate 8 does not fire).
- **Source fn:** `test_cc_gate_ownership_rejects_non_owned`,
  `test_cc_expired_ok_with_gate_ownership_true`,
  `test_cc_five_hour_none_with_gate_ownership_true` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-9: `sort::renew` — subscription renewal sooner than 7d reset ranks first (BUG-229)

- **Given:** Account A has `sub_renewal_secs = 3d` (3 days to billing renewal) and
  `7d_reset_secs = 7d`. Account B has `sub_renewal_secs = 10d` and `7d_reset_secs = 2d`.
  `min(3d, 7d) = 3d` for A; `min(10d, 2d) = 2d` for B.
- **When:** `sort::renew` strategy runs.
- **Then:** B ranks first (min=2d < min=3d) — `sort::renew` uses `min(7d_reset, sub_renewal)`
  as the primary key.
- **Source fn:** `mre_bug229_sort_renew_subscription_sooner_than_7d_ranks_first`,
  `mre_bug229_find_next_renew_picks_account_with_sooner_subscription` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-10: `sort::renews` — ascending subscription renewal order

- **Given:** Three accounts with different `sub_renewal_secs` values.
- **When:** `sort::renews` strategy runs.
- **Then:** Accounts are sorted in ascending order of subscription billing renewal time
  (soonest renewal first).
- **Source fn:** `test_sort_renews_ascending` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-11: `sort::renew` tiebreaker: alphabetical when renewal equal (BUG-260 fix)

- **Given:** Two accounts with identical `min(7d_reset, sub_renewal)` values — fully tied on
  the primary key.
- **When:** `sort::renew` strategy runs.
- **Then:** Tiebreaker resolves to alphabetical account name order (secondary key: name asc).
  Before Fix(BUG-260), the sort was nondeterministic on tied inputs.
- **Source fn:** `test_sort_renew_tiebreaker_alphabetical_when_equal_renewal`,
  `mre_bug260_renew_nondeterministic_when_fully_tied` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-12: `sort::renew` skips weekly-exhausted even with soonest renewal (BUG-292)

- **Given:** An account with `seven_day_left ≤ 5%` (weekly-exhausted) has the soonest
  `min(7d_reset, sub_renewal)` value.
- **When:** `sort::renew` strategy runs.
- **Then:** The weekly-exhausted account is skipped regardless of having the soonest renewal.
  Gate 7 fires before sort-based selection.
- **Source fn:** `mre_bug292_renew_skips_weekly_exhausted_even_with_soonest_renewal` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-13: Cancelled subscription not recommended (BUG-317)

- **Given:** An account with `billing_type = "none"` (cancelled subscription) and otherwise
  healthy quota.
- **When:** `find_next_for_strategy` runs with any strategy.
- **Then:** The cancelled account is never returned as the winner. Gate 3b (billing-none check)
  fires before selection.
- **Source fn:** `mre_bug317_cancelled_not_recommended_by_find_next` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-14: Gate 4 boundary — `five_hour.utilization = 85.0` exactly → skipped

- **Given:** An account with `five_hour.utilization = 85.0` (exactly at the threshold).
- **When:** Gate 4 evaluates `five_hour.utilization >= 85.0`.
- **Then:** Account is skipped — at exactly the threshold, `>= 85.0 = true`, gate fires.
  Account just below threshold (`84.9`) is eligible (AC-3 coverage in sort_next_tests.rs).
- **Source fn:** `mre_bug_gap8_find_first_eligible_at_exactly_85_utilization`,
  `test_cc_h_exhausted_boundary_below_threshold` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)

---

### AC-15: BUG-324 fix — Gate 7 uses raw `seven_day_left`, not `prefer_weekly`

- **Given:** Account with `seven_day_left = 31%` (> 5%) but `seven_day_sonnet = 0%` →
  `prefer_weekly(any) = min(31, 0) = 0%` (≤ 5%).
- **When:** Gate 7 evaluates eligibility.
- **Then:** Account passes Gate 7 — `seven_day_left(31%) > 5.0 = true`. Before Fix(BUG-324),
  `prefer_weekly = 0%` blocked this green account.
- **Source fn:** `mre_bug324_green_account_eligible_when_7d_son_exhausted`,
  `test_cc_bug324_divergent_at_boundary_eligible`,
  `test_cc_gate7_boundary_exactly_5pct_skipped_in_eligibility` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/005_next_account_selection.md](../../../docs/algorithm/005_next_account_selection.md)
