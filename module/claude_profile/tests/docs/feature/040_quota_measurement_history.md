# FT — Feature 040: Quota Measurement History and Polynomial Approximation

### Scope

- **Purpose**: Test cases for measurement history ring buffer storage and polynomial approximation behavior.
- **Source**: `docs/feature/040_quota_measurement_history.md`
- **Covers**: AC-01 through AC-13 (FT-14..FT-18 cover read_cached_quota() pipeline — BUG-304 fix)

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | History append stores measurement with correct t/h5/d7/sn fields | ✅ `history_append_stores_correct_fields` |
| FT-02 | AC-02 | Ring buffer evicts oldest entry when 11th measurement appended | ✅ `history_ring_buffer_evicts_oldest` |
| FT-03 | AC-03 | Only real server values stored — cached/error values never appended | ✅ `ft03_history_skips_cached_fallback` |
| FT-04 | AC-04 | Quadratic approximation with 3+ measurements produces ~-prefixed values | ✅ `approx_quadratic_three_points_extrapolates` |
| FT-05 | AC-05 | Independent period approximation — absent period unaffected | ✅ `ft05_approx_independent_periods_absent_sn_unaffected` |
| FT-06 | AC-06 | Pre-window measurements excluded from polynomial fit | ✅ `approx_filters_pre_window_measurements` |
| FT-07 | AC-07 | Expired window (now > resets_at) yields utilization 0.0 | ✅ `approx_expired_window_returns_zero` |
| FT-08 | AC-08 | Value clamping — extrapolation never exceeds [0.0, 100.0] | ✅ `approx_clamps_to_100` |
| FT-09 | AC-09 | Tangent-line continuation beyond 2x measurement span | ✅ `approx_tangent_line_beyond_2x_span` |
| FT-10 | AC-10 | Singular matrix falls back to linear; degenerate falls back to constant | ✅ `approx_singular_matrix_falls_back_to_constant` |
| FT-11 | AC-11 | Backward compatibility — no history key = 0 measurements | ✅ `history_read_absent_key_returns_empty` |
| FT-12 | AC-12 | Non-owned accounts skip history append | ✅ `ft12_history_non_owned_skips_append` |
| FT-13 | AC-13 | Duplicate timestamp overwrites instead of append | ✅ `history_duplicate_timestamp_overwrites` |
| FT-14 | AC-04 | `read_cached_quota` — absent cache returns `None` | ✅ `test_read_cached_quota_absent_returns_none` |
| FT-15 | AC-11 | `read_cached_quota` — cache present, no history → raw values (backward compat) | ✅ `test_read_cached_quota_no_history_returns_raw` |
| FT-16 | AC-04 | `read_cached_quota` — 1 history entry → raw values (< 2 skips approximation) | ✅ `test_read_cached_quota_one_history_returns_raw` |
| FT-17 | AC-04 | `read_cached_quota` — 3+ history entries → approximated values (polynomial applied) | ✅ `test_read_cached_quota_applies_approximation` |
| FT-18 | AC-07 | `read_cached_quota` — resets_at elapsed → utilization 0.0 | ✅ `test_read_cached_quota_expired_window_returns_zero` |

### Notes

- FT-01, FT-02, FT-11, FT-13 are storage-layer unit tests in `claude_profile_core/tests/account_test.rs`.
- FT-04, FT-06, FT-07, FT-08, FT-09, FT-10 are pure-math unit tests in `src/usage/approx.rs` `#[cfg(test)]` module.
- FT-14..FT-18 are unit tests for `read_cached_quota()` in `src/usage/fetch.rs` — verify the centralized cache-read + approximation pipeline (BUG-304 fix, TSK-316).
- FT-03, FT-05, FT-12 are integration tests verifying the fetch pipeline behavior in `src/usage/fetch.rs` test module.
- FT-04 render integration (display with `~` prefix) may be covered by existing FT-03/033 render tests — the display path is shared.

---

### FT-01: History append stores measurement with correct fields

- **Given:** Account `alice` has `alice.json` with an empty `cache.history[]` array. A successful quota fetch returned `five_hour = (42.0, "2026-06-21T14:00:00+00:00")`, `seven_day = (35.0, "2026-06-25T00:00:00+00:00")`, `seven_day_sonnet = (20.0, "2026-06-25T00:00:00+00:00")`.
- **When:** `write_history_entry()` is called for `alice`.
- **Then:** `alice.json` `cache.history[0]` contains `t` (Unix seconds within 2s of now), `h5: [42.0, "..."]`, `d7: [35.0, "..."]`, `sn: [20.0, "..."]`.
- **Exit:** Ok(())
- **Source fn:** `history_append_stores_correct_fields`
- **Source:** [040_quota_measurement_history.md AC-01](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-02: Ring buffer evicts oldest at capacity

- **Given:** `alice.json` `cache.history[]` already contains 10 entries with `t` values 1000..1009.
- **When:** An 11th measurement is appended with `t = 1010`.
- **Then:** `cache.history[]` has exactly 10 entries; `history[0].t == 1001` (oldest evicted); `history[9].t == 1010` (newest appended).
- **Exit:** Ok(())
- **Source fn:** `history_ring_buffer_evicts_oldest`
- **Source:** [040_quota_measurement_history.md AC-02](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-03: Only real server values stored in history

- **Given:** A fetch pipeline iteration where account `alice` has a transient error (429) and falls back to cache. The fallback produces `Ok(cached_data)` with `cached: true`.
- **When:** The cache write path runs after the fallback.
- **Then:** No entry is appended to `cache.history[]` — only the existing single-point `cache` fields are updated (or not, since it's a fallback). History remains unchanged from the prior fetch.
- **Exit:** history length unchanged
- **Source fn:** `ft03_history_skips_cached_fallback`
- **Source:** [040_quota_measurement_history.md AC-03](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-04: Quadratic approximation with 3+ measurements

- **Given:** Three measurements: `[(t=0, y=10.0), (t=60, y=20.0), (t=120, y=35.0)]` — accelerating usage.
- **When:** `approximate_utilization()` is called with `t_now = 180`.
- **Then:** Returns `Some(value)` where value reflects the quadratic extrapolation (positive a2 coefficient — acceleration). The value is > 35.0 (beyond last measurement) and <= 100.0 (clamped).
- **Exit:** Some(value) in (35.0, 100.0]
- **Source fn:** `approx_quadratic_three_points_extrapolates`
- **Source:** [040_quota_measurement_history.md AC-04](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-05: Independent period approximation

- **Given:** History with 4 measurements where `h5` is present in all 4, `d7` is present in all 4, `sn` is `null` in all 4.
- **When:** Approximation runs for all three periods.
- **Then:** `h5` and `d7` produce `Some(value)` via quadratic fit; `sn` produces `None` (no data points). The absent `sn` does not affect `h5` or `d7` results.
- **Exit:** (Some, Some, None)
- **Source fn:** `ft05_approx_independent_periods_absent_sn_unaffected`
- **Source:** [040_quota_measurement_history.md AC-05](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-06: Pre-window measurements excluded

- **Given:** 5h window with `latest_resets_at = T+18000` (window started at T+0). History has 5 measurements: 2 at `t < T` (previous window) and 3 at `t >= T` (current window).
- **When:** `approximate_utilization()` runs with the reset boundary filter.
- **Then:** Only the 3 current-window measurements are used for the quadratic fit. The 2 pre-window measurements are discarded.
- **Exit:** polynomial fit uses 3 points
- **Source fn:** `approx_filters_pre_window_measurements`
- **Source:** [040_quota_measurement_history.md AC-06](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-07: Expired window yields zero utilization

- **Given:** `resets_at` is in the past (`resets_at_secs < now_secs`). History has 5 measurements all from the expired window.
- **When:** `approximate_utilization()` is called.
- **Then:** Returns `Some(0.0)` — window has reset; new window hasn't accumulated usage.
- **Exit:** Some(0.0)
- **Source fn:** `approx_expired_window_returns_zero`
- **Source:** [040_quota_measurement_history.md AC-07](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-08: Value clamping

- **Given:** Measurements with steeply rising utilization: `[(0, 80.0), (60, 90.0), (120, 97.0)]`. Extrapolation at `t=300` would produce a raw polynomial value > 100.0.
- **When:** `approximate_utilization()` is called with `t_now = 300`.
- **Then:** Returns `Some(100.0)` — clamped to maximum.
- **Exit:** Some(100.0)
- **Source fn:** `approx_clamps_to_100`
- **Source:** [040_quota_measurement_history.md AC-08](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-09: Tangent-line continuation beyond 2x span

- **Given:** Measurements span 120 seconds: `[(0, 10.0), (60, 20.0), (120, 35.0)]`. `t_now = 500` (380s beyond last measurement, > 2x span of 120s).
- **When:** `approximate_utilization()` is called.
- **Then:** Uses tangent line at `t=120` instead of raw quadratic. The result is a linear extrapolation from the derivative at `t=120`, not the accelerating polynomial curve.
- **Exit:** Some(value) < raw_quadratic_at_500
- **Source fn:** `approx_tangent_line_beyond_2x_span`
- **Source:** [040_quota_measurement_history.md AC-09](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-10: Singular matrix fallback

- **Given:** Three measurements with identical timestamps: `[(100, 50.0), (100, 51.0), (100, 52.0)]`. The Vandermonde matrix is singular.
- **When:** `approximate_utilization()` attempts quadratic fit.
- **Then:** Falls back to constant (last measurement value = 52.0) since both quadratic and linear are degenerate.
- **Exit:** Some(52.0)
- **Source fn:** `approx_singular_matrix_falls_back_to_constant`
- **Source:** [040_quota_measurement_history.md AC-10](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-11: Backward compatibility — no history key

- **Given:** `alice.json` has `cache` object with `fetched_at` and quota fields but no `"history"` key (old format).
- **When:** `read_history()` is called for `alice`.
- **Then:** Returns empty `Vec` (0 measurements). The existing single-point fallback behavior from Feature 033 is preserved.
- **Exit:** Ok(vec![])
- **Source fn:** `history_read_absent_key_returns_empty`
- **Source:** [040_quota_measurement_history.md AC-11](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-12: Non-owned accounts skip history append

- **Given:** Account `bob` has `is_owned: false` (non-owned, Feature 036 G1 gate). A cache read for `bob` returns quota data.
- **When:** The fetch pipeline processes `bob`'s result.
- **Then:** No entry is appended to `bob.json` `cache.history[]` — non-owned accounts never append to history since they don't perform real HTTP fetches.
- **Exit:** history unchanged
- **Source fn:** `ft12_history_non_owned_skips_append`
- **Source:** [040_quota_measurement_history.md AC-12](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-13: Duplicate timestamp overwrites

- **Given:** `alice.json` `cache.history[]` has 3 entries. The last entry has `t = 1750521800`.
- **When:** A new measurement is appended with `t = 1750521800` (same second).
- **Then:** `cache.history[]` still has 3 entries (not 4). The last entry's `h5`/`d7`/`sn` are updated to the new values.
- **Exit:** history length unchanged; last entry updated
- **Source fn:** `history_duplicate_timestamp_overwrites`
- **Source:** [040_quota_measurement_history.md AC-13](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-14: `read_cached_quota` — absent cache returns `None`

- **Given:** Account `alice` has no `alice.json` file in the credential store (or `alice.json` exists but has no `"cache"` key).
- **When:** `read_cached_quota(credential_store, "alice", now_secs)` is called.
- **Then:** Returns `None`. No panic, no error propagated.
- **Exit:** None
- **Source fn:** `test_read_cached_quota_absent_returns_none`
- **Note:** `read_cached_quota()` is the centralized cache-read + approximation function (BUG-304 fix). Absent cache → `None` mirrors the graceful-degradation behavior of raw `read_quota_cache()` for this case.
- **Source:** [040_quota_measurement_history.md AC-04](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-15: `read_cached_quota` — no history → raw cached values returned

- **Given:** Account `alice` has `alice.json` with a valid `"cache"` object (known `five_hour.utilization = 55.0`) but no `"history"` key.
- **When:** `read_cached_quota(credential_store, "alice", now_secs)` is called.
- **Then:** Returns `Some((data, age_secs))` where `data.five_hour.utilization == 55.0` (raw cached value — no approximation applied). `age_secs` equals `now_secs - fetched_at_secs`.
- **Exit:** Some((raw_data, age))
- **Note:** Backward-compatible path (AC-11). When `history.len() < 2`, `read_cached_quota` skips approximation and returns the raw single-point cache value.
- **Source fn:** `test_read_cached_quota_no_history_returns_raw`
- **Source:** [040_quota_measurement_history.md AC-11](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-16: `read_cached_quota` — 1 history entry → raw values (threshold not met)

- **Given:** Account `alice` has `alice.json` with `"cache"` (known `five_hour.utilization = 55.0`) and `"history": [{ one entry }]`.
- **When:** `read_cached_quota(credential_store, "alice", now_secs)` is called.
- **Then:** Returns `Some((data, age_secs))` where `data.five_hour.utilization == 55.0` (raw — `len() == 1 < 2` threshold).
- **Exit:** Some((raw_data, age))
- **Note:** The `>= 2` threshold for approximation is the boundary; exactly 1 entry is explicitly below it.
- **Source fn:** `test_read_cached_quota_one_history_returns_raw`
- **Source:** [040_quota_measurement_history.md AC-04](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-17: `read_cached_quota` — 3+ history entries → polynomial-approximated values

- **Given:** Account `alice` has `alice.json` with `"cache"` (`five_hour.utilization = 40.0`, `resets_at` set 4h in the future). `"history"` contains 3 entries in the current 5h window with `h5` utilization values `10.0`, `25.0`, `40.0` at timestamps `t0 < t1 < t2 < now_secs` (monotonically increasing trend).
- **When:** `read_cached_quota(credential_store, "alice", now_secs)` is called.
- **Then:** Returns `Some((data, age_secs))` where `data.five_hour.utilization ≠ 40.0` (quadratic LS polynomial applied — result > 40.0 due to increasing trend). `age_secs` computed from `fetched_at`. `d7` and `sn` unaffected (independent periods, 5h window only has data here).
- **Exit:** Some((approximated_data, age)) where five_hour.utilization > 40.0
- **Note:** This is the core BUG-304 fix verification at the function-unit level. FT-04 tests `approximate_utilization()` in isolation; this test verifies the pipeline: `read_cached_quota()` reads cache + history and applies the algorithm. Complements Feature 036 FT-23 which tests the G1 integration path end-to-end.
- **Source fn:** `test_read_cached_quota_applies_approximation`
- **Source:** [040_quota_measurement_history.md AC-04](../../../docs/feature/040_quota_measurement_history.md)

---

### FT-18: `read_cached_quota` — elapsed `resets_at` → utilization 0.0

- **Given:** Account `alice` has `alice.json` with `"cache"` (`five_hour.utilization = 70.0`, `resets_at` set 2 hours in the PAST). `"history"` contains 3 entries in the now-elapsed window.
- **When:** `read_cached_quota(credential_store, "alice", now_secs)` is called with `now_secs > resets_at_secs`.
- **Then:** Returns `Some((data, age_secs))` where `data.five_hour.utilization == 0.0` — the window has reset; `approximate_utilization()` returns `0.0` when `now_secs > resets_at_secs` (AC-07). The stale raw value of `70.0` is NOT returned.
- **Exit:** Some((data, age)) where five_hour.utilization == 0.0
- **Source fn:** `test_read_cached_quota_expired_window_returns_zero`
- **Source:** [040_quota_measurement_history.md AC-07](../../../docs/feature/040_quota_measurement_history.md)
