# State Machine 005: Quota Measurement Lifecycle

AC test cases for `docs/state_machine/005_quota_measurement_lifecycle.md`. Tests the
`empty/single/linear/quadratic/full` history ring buffer states and the pre-fit filter
that governs which measurements contribute to polynomial approximation.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `empty` — 0 measurements, approximation returns None | State | ✅ |
| AC-2 | `empty` — raw quota returned when no history array | State | ✅ |
| AC-3 | `single` — 1 measurement, degree-0 raw value returned | State | ✅ |
| AC-4 | `linear` — 2 measurements, linear extrapolation applied | State | ✅ |
| AC-5 | `quadratic` — 3+ measurements, polynomial fit applied | State | ✅ |
| AC-6 | Fetch failure — measurement NOT appended (only success appends) | Invariant | ✅ |
| AC-7 | Expired-window short-circuit — `now > resets_at` returns zero | Filter | ✅ |
| AC-8 | Non-owned account — history append skipped | Gate | ✅ |

---

### AC-1: `empty` — 0 measurements, approximation returns None

- **Given:** No `{name}.json` cache file exists at all for this account — not merely an empty
  `history` array, but no file on disk (a `cache` entry present with no/empty `history` is
  AC-2's scenario, which returns `Some` with raw values, not `None`).
- **When:** `read_cached_quota(store.path(), name, now_secs)` is called for this account.
- **Then:** Returns `None` (cannot approximate with zero measurements and no cached raw data
  to fall back to). This is the absent-cache base case (AC-11 backward-compat) — the `empty`
  ring-buffer state with a cache entry present is exercised separately by AC-2's test.
- **Source fn:** `test_read_cached_quota_absent_returns_none` in
  `tests/usage/fetch_tests_b.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-2: `empty` — raw quota returned when no history array

- **Given:** An account with a `cache` entry that has no `history` key (raw cache only).
- **When:** `read_cached_quota()` is called.
- **Then:** Returns the raw cached quota value without approximation. `empty` state means
  approximation cannot be applied; raw values from the last successful fetch are used instead.
- **Source fn:** `test_read_cached_quota_no_history_returns_raw` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-3: `single` — 1 measurement, degree-0 (constant) approximation

- **Given:** An account with exactly 1 entry in the history ring buffer.
- **When:** `read_cached_quota()` is called.
- **Then:** Returns the raw quota value from that single measurement (degree-0 fit = constant
  last value). The `single` state does not extrapolate — it returns the observed value as-is.
- **Source fn:** `test_read_cached_quota_one_history_returns_raw` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-4: `linear` — 2 measurements, linear extrapolation applied

- **Given:** An account with exactly 2 entries in the history ring buffer, both within the
  current window period (timestamps after `resets_at - window_duration`).
- **When:** `read_cached_quota()` is called.
- **Then:** Linear extrapolation (degree-1 least squares) is applied to project the current
  utilization based on the two data points. The `linear` state produces a more accurate
  estimate than degree-0.
- **Source fn:** `cc08_read_cached_quota_two_history_entries_applies_linear` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-5: `quadratic` — 3+ measurements, polynomial fit applied

- **Given:** An account with 3 or more entries in the history ring buffer, all within the
  current window period.
- **When:** `read_cached_quota()` is called.
- **Then:** Degree-2 polynomial fit (Cramer's rule 3×3) is applied. The `quadratic` state
  produces the most accurate approximation by capturing acceleration in quota consumption.
- **Source fn:** `test_read_cached_quota_applies_approximation` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-6: Fetch failure — measurement NOT appended to history ring buffer

- **Given:** `fetch_oauth_usage()` returns a cached fallback result (not a fresh API response).
- **When:** The usage fetch completes with a cache hit (not a live fetch success).
- **Then:** No measurement is appended to the history array in `{name}.json`. The ring buffer
  state is unchanged. Only successful live API responses advance the ring buffer lifecycle.
  Caching fallback and error results are filtered out.
- **Source fn:** `ft03_history_skips_cached_fallback` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-7: Expired-window short-circuit — `now > resets_at` returns zero before the pre-fit filter runs

- **Given:** An account with 2 measurements in the history ring buffer, BOTH within the
  current window's `window_start = resets_at - window_duration` (the cited test's own
  comments confirm neither point is old enough to be discarded by age). `now_secs` is set
  well after `resets_at` — the window itself has already reset/expired.
- **When:** `read_cached_quota()` calls `approximate_utilization()`.
- **Then:** The `now_secs > resets_at_secs` guard (`src/usage/approx.rs` lines 56-60) fires
  FIRST and returns `Some(0.0)` immediately — before the age-based `window_start` per-point
  filter (lines 62-67) is ever reached. This test does NOT exercise the age-based discard
  filter (its 2 points are deliberately in-window); it exercises the separate
  "already-expired-window" short-circuit, a distinct guard that runs earlier in the same
  function.
- **Source fn:** `test_read_cached_quota_expired_window_returns_zero` in
  `tests/usage/fetch_tests_b.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-8: Non-owned account — history append skipped

- **Given:** An account with `is_owned=false` (owned by another machine).
- **When:** `fetch_oauth_usage()` succeeds for this account.
- **Then:** No measurement is appended to the account's history ring buffer. Non-owned accounts
  are read-only — their history is not written because the owning machine is responsible for
  maintaining the measurement record. Writing history on a non-owned account would cause
  cross-machine ring buffer corruption.
- **Source fn:** `ft12_history_non_owned_skips_append` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)
