# Algorithm 007: Sort Strategies

AC test cases for `docs/algorithm/007_sort_strategies.md`. Tests `sort_indices()` and
`relevant_quotas()` in `src/usage/sort.rs`, and `prefer_weekly()` in `src/usage/format.rs`.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `sort::name` — alphabetical A→Z | Nominal | ✅ |
| AC-2 | `sort::renew` — `min(7d_reset, sub_renewal)` ascending; name tiebreak | Nominal | ✅ |
| AC-3 | `sort::renews` — `sub_renewal_secs` ascending; name tiebreak | Nominal | ✅ |
| AC-4 | `prefer::any` — `min(seven_day_left, seven_day_sonnet_left)` when Sonnet present | Nominal | ✅ |
| AC-5 | `prefer::any` — `seven_day_left` when Sonnet absent | Nominal | ✅ |
| AC-6 | `prefer::son` — `seven_day_sonnet_left` when Sonnet present | Nominal | ✅ |
| AC-7 | `prefer::son` — `0.0` when Sonnet absent (sorts last; NOT ineligible) | Nominal | ✅ |
| AC-8 | `prefer::opus` — always `seven_day_left` (model-agnostic) | Nominal | ✅ |
| AC-9 | `relevant_quotas()` — `Err` result returns `(0.0, 0.0)` | Boundary | ✅ |
| AC-10 | `sort::renew` — `sub_renewal_secs` from `renewal_at` when present | Nominal | ✅ |
| AC-11 | `sort::renew` — `sub_renewal_secs` from `org_created_at` estimate when `renewal_at` absent | Nominal | ✅ |
| AC-12 | BUG-291: `sort::renew` next-account tiebreaker matches `sort_indices` ordering | Regression | ✅ |

---

### AC-1: `sort::name` — alphabetical A→Z

- **Given:** Multiple eligible accounts with different names.
- **When:** `sort_indices(strategy=Name, ...)` is called.
- **Then:** Accounts are sorted in ascending alphabetical order of account name. No secondary
  key needed.
- **Source fn:** `test_find_next_for_strategy_some_when_eligible_none_when_all_current` and
  renew/name tiebreak tests in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-2: `sort::renew` — `min(7d_reset, sub_renewal)` ascending; name tiebreak

- **Given:** Accounts with various combinations of `7d_reset_secs` and `sub_renewal_secs`.
- **When:** `sort_indices(strategy=Renew, ...)` is called.
- **Then:** Primary sort key is `min(7d_reset_secs, sub_renewal_secs)` ascending (soonest
  first). On tie: secondary key `prefer_weekly` ascending; tertiary: name ascending
  (Fix BUG-260).
- **Source fn:** `mre_bug229_sort_renew_subscription_sooner_than_7d_ranks_first`,
  `test_sort_renew_tiebreaker_alphabetical_when_equal_renewal` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-3: `sort::renews` — `sub_renewal_secs` ascending

- **Given:** Accounts with different subscription billing renewal times.
- **When:** `sort_indices(strategy=Renews, ...)` is called.
- **Then:** Primary sort key is `sub_renewal_secs` ascending. Secondary: name ascending.
- **Source fn:** `test_sort_renews_ascending` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-4: `prefer::any` with Sonnet present — `min(seven_day_left, sonnet_left)`

- **Given:** Account with `seven_day = Some(...)` and `seven_day_sonnet = Some(...)`.
  `seven_day_left = 40%`, `seven_day_sonnet_left = 20%`.
- **When:** `prefer_weekly(aq, Any)` is called.
- **Then:** Returns `min(40.0, 20.0) = 20.0` — `any` preference takes the more constrained
  of the two 7d capacities.
- **Source fn:** `test_relevant_quotas_any_no_sonnet`,
  implied by `test_relevant_quotas_son_with_sonnet` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-5: `prefer::any` with Sonnet absent — `seven_day_left`

- **Given:** Account with `seven_day_sonnet = None` (no Sonnet tier).
- **When:** `prefer_weekly(aq, Any)` is called.
- **Then:** Returns `seven_day_left` — when Sonnet is absent, the 7d general quota is used
  directly.
- **Source fn:** `test_relevant_quotas_any_no_sonnet` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-6: `prefer::son` with Sonnet present — `seven_day_sonnet_left`

- **Given:** Account with `seven_day_sonnet = Some(PeriodUsage { utilization: 60.0 })`.
- **When:** `prefer_weekly(aq, Sonnet)` is called.
- **Then:** Returns `40.0` (100 - 60.0) — Sonnet-specific remaining capacity.
- **Source fn:** `test_relevant_quotas_son_with_sonnet` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-7: `prefer::son` with Sonnet absent — `0.0` (sorts last, not ineligible)

- **Given:** Account with `seven_day_sonnet = None`.
- **When:** `prefer_weekly(aq, Sonnet)` is called.
- **Then:** Returns `0.0` — absence of Sonnet tier means zero Sonnet capacity; account sorts
  last for Sonnet preference. Gate 7 uses raw `seven_day_left` (AC-15 of algorithm 005), so
  a zero `prefer_weekly` does NOT make the account ineligible.
- **Source fn:** `test_relevant_quotas_son_no_sonnet`,
  `test_cc_prefer_sonnet_absent_tier_eligible` in `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-8: `prefer::opus` — always `seven_day_left`

- **Given:** Account with any quota state.
- **When:** `prefer_weekly(aq, Opus)` is called.
- **Then:** Returns `seven_day_left` — Opus preference is model-agnostic; it always uses the
  raw 7d general quota as the tiebreak key.
- **Source fn:** `test_relevant_quotas_opus` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-9: `relevant_quotas()` with error result → `(0.0, 0.0)`

- **Given:** `AccountQuota.result = Err(...)`.
- **When:** `relevant_quotas(aq, prefer)` is called.
- **Then:** Returns `(0.0, 0.0)` — error accounts sort last regardless of preference strategy.
- **Source fn:** `test_relevant_quotas_err` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-10: `sort::renew` — `sub_renewal_secs` from `renewal_at` when present

- **Given:** Account with `renewal_at` set in `{name}.json`.
- **When:** `renewal_secs(aq, now)` is called.
- **Then:** Returns seconds until `renewal_at` — exact subscription renewal date used directly.
- **Source fn:** `mre_bug229_strategy_metric_renew_exact_sub_shows_both_timers` in
  `tests/usage/sort_next_tests.rs`; `rl_exact_from_renewal_at` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-11: `sort::renew` — `sub_renewal_secs` estimated from `org_created_at`

- **Given:** Account with `renewal_at = None` but `org_created_at` set.
- **When:** `renewal_secs(aq, now)` is called.
- **Then:** Returns estimated seconds from `org_created_at` monthly cycle — advances past the
  current `now` to find the next estimated renewal date.
- **Source fn:** `mre_bug229_strategy_metric_renew_no_sub_shows_7d_only`,
  `rl_estimate_from_org_created_at`, `rl_auto_advance_past_renewal_at` in
  `tests/usage/format_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)

---

### AC-12: BUG-291 fix — `sort::renew` next-account tiebreaker matches `sort_indices` order

- **Given:** Multiple accounts with different `prefer_weekly` values as tiebreaker.
- **When:** `find_next_for_strategy(Renew)` and `sort_indices(Renew)` are called on the same
  account set.
- **Then:** The account selected by `find_next_for_strategy` matches the first element from
  `sort_indices` — the two sort paths use identical ordering logic.
- **Source fn:** `mre_bug291_renew_next_tiebreaker_matches_sort_indices` in
  `tests/usage/sort_next_tests.rs`
- **Source:** [algorithm/007_sort_strategies.md](../../../docs/algorithm/007_sort_strategies.md)
