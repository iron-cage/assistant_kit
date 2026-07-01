# Algorithm 006: Quota Polynomial Approximation

AC test cases for `docs/algorithm/006_quota_approximation.md`. Tests
`approximate_utilization(period, history, now_secs)` in `src/usage/approx.rs` and the
cache integration in `src/usage/fetch.rs` via `read_cached_quota`.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | 0 post-filter measurements → `None` (no data) | Nominal | ✅ |
| AC-2 | 1 measurement → constant (raw value returned unchanged) | Nominal | ✅ |
| AC-3 | 2 measurements → linear extrapolation applied | Nominal | ✅ |
| AC-4 | 3–10 measurements → quadratic LS applied | Nominal | ✅ |
| AC-5 | Window expired (`now > resets_at`) → returns `0.0` | Boundary | ✅ |
| AC-6 | Approximation applied for non-owned accounts (G1 non-owned path) | Nominal | ✅ |
| AC-7 | Independent periods: absent `seven_day_sonnet` unaffected by 5h/7d approx | Isolation | ✅ |
| AC-8 | Cache fallback skipped when quota fetch succeeds (no spurious approximation) | Nominal | ✅ |
| AC-9 | History not appended for non-owned accounts (no ownership contamination) | Nominal | ✅ |

---

### AC-1: 0 post-filter measurements → `None` (no data)

- **Given:** `{name}.json` exists but contains no history entries, OR all entries are outside
  the window (`window_start = resets_at - window_duration`).
- **When:** `read_cached_quota` / `approximate_utilization` is called.
- **Then:** Returns `None` — no data is available for approximation. The raw cached value is
  returned instead when history is absent.
- **Source fn:** `test_read_cached_quota_absent_returns_none` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-2: 1 measurement → constant (last value returned)

- **Given:** Exactly 1 history entry within the window.
- **When:** `approximate_utilization` is called.
- **Then:** The single measurement's utilization value is returned unchanged (degree-0
  polynomial = constant).
- **Source fn:** `test_read_cached_quota_one_history_returns_raw` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-3: 2 measurements → linear extrapolation

- **Given:** Exactly 2 history entries within the window.
- **When:** `approximate_utilization` is called.
- **Then:** Linear least-squares extrapolation is applied (`linear_extrapolate`). The result
  reflects the trend between the two points projected to `now_secs`.
- **Source fn:** `cc08_read_cached_quota_two_history_entries_applies_linear` in
  `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-4: 3–10 measurements → quadratic LS applied

- **Given:** 3 or more history entries within the window.
- **When:** `approximate_utilization` is called.
- **Then:** Quadratic least-squares fit (Cramer 3×3 solver) is applied. When the Cramer
  determinant is near-zero (singular system), falls back to linear extrapolation.
- **Source fn:** `test_read_cached_quota_applies_approximation` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-5: Window expired (`now > resets_at`) → returns `0.0`

- **Given:** `resets_at` is in the past relative to `now_secs`.
- **When:** `approximate_utilization` is called.
- **Then:** Returns `0.0` — the quota window has reset; the historical extrapolation is no
  longer valid. A fresh fetch will reflect the new window.
- **Source fn:** `test_read_cached_quota_expired_window_returns_zero` in
  `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-6: Approximation applied for non-owned accounts (G1b path)

- **Given:** A non-owned account where `is_current = false` and
  `occupied_elsewhere.contains(&name) = true` — live HTTP fetch is skipped (G1b gate).
- **When:** `fetch_all_quota` runs.
- **Then:** The non-owned account's quota is populated via `approximate_quota()` using cached
  history. The result reflects an approximation rather than a live fetch.
- **Source fn:** `ft23_g1_non_owned_applies_approximation` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-7: Independent periods — absent `seven_day_sonnet` unaffected by 5h/7d approximation

- **Given:** Account has `five_hour` and `seven_day` history; `seven_day_sonnet` is absent
  from the cache.
- **When:** Approximation runs for 5h and 7d periods.
- **Then:** `seven_day_sonnet` remains `None` in the approximated quota — approximating 5h/7d
  does not fabricate a Sonnet tier value. Period approximations are independent.
- **Source fn:** `ft05_approx_independent_periods_absent_sn_unaffected` in
  `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-8: Cache fallback skipped when quota fetch succeeds

- **Given:** Live `fetch_oauth_usage()` succeeds and returns a valid quota.
- **When:** `fetch_all_quota` runs.
- **Then:** The successful live result is used directly; the cached/approximated value is NOT
  substituted. Approximation is a fallback, not a primary path.
- **Source fn:** `ft03_history_skips_cached_fallback` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-9: History not appended for non-owned accounts

- **Given:** A non-owned account (quota derived from approximation, not live fetch).
- **When:** `fetch_all_quota` completes.
- **Then:** No new history entry is written to `{name}.json` — only live-fetched results
  contribute to history. Non-owned approximated results do not pollute the history baseline.
- **Source fn:** `ft12_history_non_owned_skips_append` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)
