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
| AC-7 | Pre-fit filter — expired window discards old measurements | Filter | ✅ |
| AC-8 | Non-owned account — history append skipped | Gate | ✅ |

---

### AC-1: `empty` — 0 measurements, approximation returns None

- **Given:** An account with a `_quota_cache` entry in `{name}.json` but no `history` array,
  or a `history` array with 0 entries.
- **When:** `read_cached_quota()` is called for this account.
- **Then:** Returns `None` (cannot approximate with zero measurements). The ring buffer is in
  `empty` state — no historical data available for any fit.
- **Source fn:** `test_read_cached_quota_absent_returns_none` in
  `tests/usage/fetch_tests.rs`
- **Source:** [state_machine/005_quota_measurement_lifecycle.md](../../../docs/state_machine/005_quota_measurement_lifecycle.md)

---

### AC-2: `empty` — raw quota returned when no history array

- **Given:** An account with a `_quota_cache` entry that has no `history` key (raw cache only).
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

### AC-7: Pre-fit filter — expired window discards old measurements

- **Given:** An account with measurements in the history ring buffer, some of which were taken
  BEFORE the most recent `resets_at` timestamp (i.e., from a prior quota window cycle).
- **When:** `read_cached_quota()` applies the pre-fit filter.
- **Then:** Measurements before `window_start = resets_at - window_duration` are discarded.
  If all measurements are discarded, returns zero/reset value. This prevents old-window data
  from contaminating the polynomial fit for the current window.
- **Source fn:** `test_read_cached_quota_expired_window_returns_zero` in
  `tests/usage/fetch_tests.rs`
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
