# Algorithm 010: Renewal Date Computation

AC test cases for `docs/algorithm/010_renewal_date_computation.md`. Tests `renewal_secs(renewal_at_opt, org_created_at_opt, now_secs)` (entry point, exercised indirectly through `renews_label()`) in `claude_profile/src/usage/format.rs`. Implementations reside in `claude_profile/tests/usage/format_tests.rs` as `rl_*`.

### AC Case Index

| AC | Short Name | Category | FT Mapping | Status |
|----|------------|----------|------------|--------|
| AC-1 | Exact branch returns precise delta when `_renewal_at` is future | Nominal | rl_exact_from_renewal_at | ✅ |
| AC-2 | Estimate branch computes next billing-day occurrence from `org_created_at` | Nominal | rl_estimate_from_org_created_at | ✅ |
| AC-3 | Both sources absent returns `None` (rendered as `"?"`) | Boundary | rl_absent_returns_question | ✅ |
| AC-4 | Exact branch auto-advances a past `_renewal_at` into the future | Regression | rl_auto_advance_past_renewal_at | ✅ |
| AC-5 | Exact branch auto-advance preserves original day-of-month, clamped at month-length boundaries | Regression | rl_auto_advance_single_step_preserves_day_across_31_day_month, rl_auto_advance_multi_year_preserves_day_of_month, rl_auto_advance_clamps_day_31_anchor_at_shorter_month_end, rl_auto_advance_day29_clamps_in_common_february_then_recovers | ✅ (BUG-329 fixed) |
| AC-6 | Estimate branch clamps billing day-of-month at month-length boundaries | Regression | rl_estimate_clamps_day31_billing_anchor_at_shorter_month_end | ✅ (BUG-329 fixed) |

---

### AC-1: Exact branch returns precise delta when `_renewal_at` is future

- **Given:** `renewal_at_opt = Some("2030-01-01T03:47:00Z")`, `now_secs` = Unix seconds for `2030-01-01T00:00:00Z`
- **When:** `renews_label(renewal_at_opt, None, now_secs)` is called (wraps `renewal_secs()`)
- **Then:** Returns `"in 3h 47m"` — the exact branch computes `parse_iso_secs(renewal_at) - now_secs` directly; no auto-advance needed since the timestamp is already in the future
- **Source fn:** `rl_exact_from_renewal_at` (in `tests/usage/format_tests.rs`)

### AC-2: Estimate branch computes next billing-day occurrence

- **Given:** `org_created_at_opt = Some("2025-01-15T00:00:00Z")` (billing day 15), `now_secs` = Unix seconds for `2030-01-01T00:00:00Z` (day 1)
- **When:** `renews_label(None, org_created_at_opt, now_secs)` is called
- **Then:** Returns a string starting `"~in "` (estimate prefix) containing a days unit — billing day 15 is still ahead of day 1 in the current month, so `(renewal_year, renewal_month)` stays at the current month
- **Source fn:** `rl_estimate_from_org_created_at` (in `tests/usage/format_tests.rs`)

### AC-3: Both sources absent returns `None`

- **Given:** `renewal_at_opt = None`, `org_created_at_opt = None`
- **When:** `renews_label(None, None, now_secs)` is called
- **Then:** Returns `"?"` — `renewal_secs()` returns `None` before either branch executes
- **Source fn:** `rl_absent_returns_question` (in `tests/usage/format_tests.rs`)

### AC-4: Exact branch auto-advances a past `_renewal_at` into the future

- **Given:** `renewal_at_opt = Some("2020-01-01T00:00:00Z")`, `now_secs` = Unix seconds for `2030-01-01T00:00:00Z` (~10 years later)
- **When:** `renews_label(renewal_at_opt, None, now_secs)` is called
- **Then:** Returns a string starting `"in "` (no `~`) landing within 30 days of `now_secs` — the `while ts < now_secs` loop in the exact branch advanced the timestamp by 122 flat 30-day steps
- **Source fn:** `rl_auto_advance_past_renewal_at` (in `tests/usage/format_tests.rs`)

### AC-5: Exact branch auto-advance preserves original day-of-month, clamped at month-length boundaries

- **Given:** `renewal_at_opt = Some(<past ISO timestamp with day-of-month D>)`, where advancing month-by-month crosses at least one month shorter than D
- **When:** `renewal_secs(renewal_at_opt, None, now_secs)` is called
- **Then (per feature/030 AC-10):** the auto-advanced timestamp's day-of-month equals D, except in a month shorter than D, where it clamps to `min(D, days_in_month(target_year, target_month))`.
- **Status:** ✅ **Fixed and covered** by 4 tests: a single-step isolation case, a ~10-year/120-step accumulation case, and two direct clamping cases (day-31 anchor through February; day-29 anchor through a common-year February, then recovering to day 29 in March). Previously not covered by any existing test — `rl_auto_advance_past_renewal_at` (AC-4 above) only asserted a `≤30d` bound, and `it151_past_renewal_at_auto_advances_in_usage` (`tests/docs/feature/030_account_renewal_override.md` FT-10) only asserted the exact-vs-estimate `~` prefix — which is why the flat-step drift went undetected. Fixed by BUG-329 (`task/claude_profile/bug/329_auto_advance_flat_step_drifts_day_of_month.md`, closed) — the underlying defect is fixed and verified in code, and the bug-tracking file's own lifecycle closure is complete.
- **Source fn:** `rl_auto_advance_single_step_preserves_day_across_31_day_month`, `rl_auto_advance_multi_year_preserves_day_of_month`, `rl_auto_advance_clamps_day_31_anchor_at_shorter_month_end`, `rl_auto_advance_day29_clamps_in_common_february_then_recovers` (in `tests/usage/format_tests.rs`)

### AC-6: Estimate branch clamps billing day-of-month at month-length boundaries

- **Given:** `org_created_at_opt = Some(<ISO timestamp with day-of-month 31>)`, `now_secs` such that the target renewal month is shorter than 31 days (e.g. April, 30 days)
- **When:** `renewal_secs(None, org_created_at_opt, now_secs)` is called
- **Then:** the computed renewal timestamp clamps to `min(billing_day, days_in_month(renewal_year, renewal_month))` — e.g. billing day 31 projected onto April lands on April 30, not May 1.
- **Status:** ✅ **Fixed and covered.** Previously catalogued in `docs/algorithm/010_renewal_date_computation.md` as a non-blocking "Caveat" rather than a defect, and untested — the only pre-existing estimate-branch test (AC-2 above) uses billing day 15, which never reaches a month-length boundary. Closed as part of the same BUG-329 fix that closed AC-5, since both branches shared the identical missing-clamp root cause.
- **Source fn:** `rl_estimate_clamps_day31_billing_anchor_at_shorter_month_end` (in `tests/usage/format_tests.rs`)
