# Algorithm 010: Renewal Date Computation

AC test cases for `docs/algorithm/010_renewal_date_computation.md`. Tests `renewal_secs(renewal_at_opt, org_created_at_opt, now_secs)` (entry point, exercised indirectly through `renews_label()`) in `claude_profile/src/usage/format.rs`. Implementations reside in `claude_profile/tests/usage/format_tests.rs` as `rl_*`.

### AC Case Index

| AC | Short Name | Category | FT Mapping | Status |
|----|------------|----------|------------|--------|
| AC-1 | Exact branch returns precise delta when `_renewal_at` is future | Nominal | rl_exact_from_renewal_at | ✅ |
| AC-2 | Estimate branch computes next billing-day occurrence from `org_created_at` | Nominal | rl_estimate_from_org_created_at | ✅ |
| AC-3 | Both sources absent returns `None` (rendered as `"?"`) | Boundary | rl_absent_returns_question | ✅ |
| AC-4 | Exact branch auto-advances a past `_renewal_at` into the future | Regression | rl_auto_advance_past_renewal_at | ✅ |
| AC-5 | Exact branch auto-advance preserves original day-of-month | Regression | — | ❌ (BUG-329, open) |

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

### AC-5: Exact branch auto-advance preserves original day-of-month

- **Given:** `renewal_at_opt = Some(<past ISO timestamp with day-of-month D>)`, where advancing by flat 30-day steps crosses at least one 31-day month or February
- **When:** `renewal_secs(renewal_at_opt, None, now_secs)` is called
- **Then (expected, per feature/030 AC-10):** the auto-advanced timestamp's day-of-month equals D.
- **Then (actual, current implementation):** the flat `2_592_000`s step drifts the day-of-month backward by 1-3 days per 31-day/February month crossed.
- **Status:** ❌ **Not covered by any existing test.** No test currently asserts day-of-month equality across the auto-advance loop — `rl_auto_advance_past_renewal_at` (AC-4 above) only asserts a `≤30d` bound, and `it151_past_renewal_at_auto_advances_in_usage` (`tests/docs/feature/030_account_renewal_override.md` FT-10) only asserts the exact-vs-estimate `~` prefix. This gap is why BUG-329 went undetected. Tracked by `task/claude_profile/bug/329_auto_advance_flat_step_drifts_day_of_month.md` (open) and referenced from `task/claude_profile/368_fix_quota_cache_missing_org_created_at.md` (T09).
