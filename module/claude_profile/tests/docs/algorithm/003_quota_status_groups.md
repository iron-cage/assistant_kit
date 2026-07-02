# Algorithm 003: Quota Status Groups

AC test cases for `docs/algorithm/003_quota_status_groups.md`. Tests `status_group_of(aq)` in
`src/usage/sort.rs` and `status_emoji()` in `src/usage/format.rs`.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Both quotas ample → Green | Nominal | ✅ |
| AC-2 | 5h exhausted, 7d ok → h-exhausted (Yellow) | Nominal | ✅ |
| AC-3 | 7d exhausted (any 5h) → weekly-exhausted (Yellow) | Nominal | ✅ |
| AC-4 | Error result OR billing-none → Dead (Red) | Nominal | ✅ |
| AC-5 | `five_hour = None` treated conservatively as 100% left → Green | BUG-299 class | ✅ |
| AC-6 | Boundary: 5h at exactly 15% left → h-exhausted (≤ 15% = exhausted) | Boundary | ✅ |
| AC-7 | Status groups use raw `seven_day_left`, not `prefer_weekly` (BUG-299 fix) | Regression | ✅ |
| AC-8 | Cancelled subscription (`billing_type = "none"`) → Dead (BUG-317) | Regression | ✅ |
| AC-9 | Both-exhausted (5h ≤ 15% AND 7d ≤ 5%) → weekly-exhausted, not Red (BUG-321 fix) | Regression | ✅ |

---

### AC-1: Both quotas ample → Green group

- **Given:** `five_hour.utilization = 50.0` → 50% left (> 15%); `seven_day.utilization = 50.0`
  → 50% left (> 5%).
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `Green`. `status_emoji()` returns `🟢`.
- **Source fn:** `test_status_emoji_and_both_ample_green` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-2: 5h exhausted, 7d ok → h-exhausted group

- **Given:** `five_hour.utilization = 90.0` → 10% left (≤ 15%); `seven_day.utilization = 50.0`
  → 50% left (> 5%).
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `HExhausted`. `status_emoji()` returns `🟡`.
- **Source fn:** `test_status_emoji_and_5h_low_yellow` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-3: 7d exhausted (any 5h value) → weekly-exhausted group

- **Given:** `seven_day.utilization = 97.0` → 3% left (≤ 5%); `five_hour.utilization = 50.0`
  → 50% left (> 15%).
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `WeeklyExhausted`. `status_emoji()` returns `🟡`.
- **Source fn:** `test_status_emoji_and_7d_low_yellow` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-4: Error result OR billing_type="none" → Dead group

- **Given (error):** `AccountQuota.result = Err(...)` — quota fetch failed.
- **Given (cancelled):** `AccountQuota.result = Ok(...)` but `billing_type = "none"`.
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `Dead`. `status_emoji()` returns `🔴`.
- **Source fn:** `test_status_emoji_and_both_at_threshold_red`,
  `mre_bug317_cancelled_status_emoji_is_red` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-5: `five_hour = None` treated as 100% left → Green (conservative)

- **Given:** `five_hour = None` (server did not return a 5h period); `seven_day = None`.
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `Green` — absence of 5h data is treated conservatively as fully available
  (0% utilization), not as exhausted. `five_hour_left(aq)` returns 100.0 via `map_or`.
- **Source fn:** `test_status_emoji_five_hour_none_is_green` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-6: 5h at exactly 15% left → h-exhausted (boundary)

- **Given:** `five_hour.utilization = 85.0` → exactly 15% left; `seven_day` ample.
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `HExhausted` — the threshold is strict `> 15%`; exactly 15% is exhausted.
- **Source fn:** `it151_status_emoji_boundary_precision` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-7: Status groups use raw `seven_day_left`, not `prefer_weekly` (BUG-299 fix)

- **Given:** Account with `seven_day_left = 32%` (> 5%) but `seven_day_sonnet = None` →
  `prefer_weekly(any) = min(32, 0) = 0%` (≤ 5%).
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `HExhausted` (not `WeeklyExhausted`) — group boundary uses raw
  `seven_day_left`, not the model-weighted `prefer_weekly`. Before Fix(BUG-299), this account
  was incorrectly placed in `WeeklyExhausted`.
- **Note:** The `prefer_weekly` value is only used for sort tiebreak, never for group/gate
  boundaries.
- **Source fn:** `mre_bug321_both_exhausted_status_emoji_is_yellow` in
  `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-8: Cancelled subscription → Dead (BUG-317 fix)

- **Given:** `billing_type = "none"` — account has no active subscription.
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `Dead`. Before Fix(BUG-317), cancelled accounts fell through to
  `WeeklyExhausted` or `HExhausted` depending on quota levels — now they are always `Dead`
  regardless of quota values.
- **Source fn:** `mre_bug317_cancelled_status_emoji_is_red` in `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)

---

### AC-9: Both-exhausted → weekly-exhausted, NOT Red (BUG-321 fix)

- **Given:** `five_hour.utilization = 90%` (5h ≤ 15%) AND `seven_day.utilization = 97%`
  (7d ≤ 5%) — both windows are exhausted.
- **When:** `status_group_of(aq)` is called.
- **Then:** Returns `WeeklyExhausted` (🟡), NOT `Dead` (🔴). The `d7_ok` check takes priority
  over the `h5_ok` check — when 7d is exhausted, the account is weekly-exhausted regardless
  of 5h state. Before Fix(BUG-321), evaluation order placed both-exhausted into `Dead`.
- **Source fn:** `mre_bug321_both_exhausted_status_emoji_is_yellow` in
  `tests/usage/format_tests.rs`
- **Source:** [algorithm/003_quota_status_groups.md](../../../docs/algorithm/003_quota_status_groups.md)
