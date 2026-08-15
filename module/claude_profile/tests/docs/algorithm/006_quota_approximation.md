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
| AC-8 | Cache-fallback path does not append a new history entry (only live values recorded) | Nominal | ✅ |
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

- **Given:** Cache has 2 `five_hour` history entries (both `utilization = 70.0`) but zero
  `seven_day_sonnet` history entries (`"sn": null` in both); the raw cached values are
  `five_hour = 50.0`, `seven_day_sonnet = 50.0`.
- **When:** Approximation runs during cache-fallback (`fetch_quota_for_list`).
- **Then:** `five_hour.utilization` becomes `70.0` (approximated from its 2 history points),
  while `seven_day_sonnet.utilization` stays at its raw cached value `50.0` — with 0
  post-filter `sn` measurements, approximation returns `None` for that period (AC-1) and the
  raw cached value is used instead, unaffected by the 5h approximation. Period approximations
  are independent.
- **Source fn:** `ft05_approx_independent_periods_absent_sn_unaffected` in
  `tests/usage/fetch_tests_b.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-8: Cache-fallback path does not append a new history entry

- **Given:** Cache is pre-seeded with 1 existing history entry; the account's token is
  expired (`expiresAt = 1`), triggering the "token expired (local)" error — not 401/403 —
  which routes to the cache-fallback arm rather than a hard failure.
- **When:** `fetch_all_quota` runs.
- **Then:** The result is served from cache (`cached = true`); the history ring buffer still
  has exactly 1 entry afterward — the cache-fallback arm does NOT append a new entry. Only
  live-fetched (real server) values are ever appended to history; cached/error values never
  are.
- **Source fn:** `ft03_history_skips_cached_fallback` in `tests/usage/fetch_tests_b.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)

---

### AC-9: History not appended for non-owned accounts

- **Given:** A non-owned account (quota derived from approximation, not live fetch).
- **When:** `fetch_all_quota` completes.
- **Then:** No new history entry is written to `{name}.json` — only live-fetched results
  contribute to history. Non-owned approximated results do not pollute the history baseline.
- **Source fn:** `ft12_history_non_owned_skips_append` in `tests/usage/fetch_tests.rs`
- **Source:** [algorithm/006_quota_approximation.md](../../../docs/algorithm/006_quota_approximation.md)
