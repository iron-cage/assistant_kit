# FT — Feature 033: Quota Cache Fallback

### Scope

- **Purpose**: Test cases for quota cache fallback behavior — write-on-success, read-on-failure, staleness display, and side-effect persistence.
- **Source**: `docs/feature/033_quota_cache.md`
- **Covers**: AC-01 through AC-11

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Write quota cache preserves existing `{name}.json` fields | `cache_write_preserves_existing_fields` |
| FT-02 | AC-02 | Cache read returns cached values when fetch errors | `cache_read_returns_entry_when_present` |
| FT-03 | AC-03 | Cached data displays with `~` prefix and age indicator | `ft03_033_render_text_cached_shows_tilde_prefix` |
| FT-04 | AC-04 | No cache = dashes (no regression) | `cache_read_returns_none_when_absent` |
| FT-05 | AC-05 | Model override writes `cache.model_override` field | `cache_field_string_persisted` |
| FT-06 | AC-06 | Touch writes `cache.last_touch_at` + `cache.touch_idle` | `cache_field_bool_persisted` |
| FT-07 | AC-07 | Cache write→read round-trip preserves all quota fields | `cache_write_read_roundtrip` |
| FT-08 | AC-08 | Strategy recommendations operate on cached values | structural (cached rows have `Ok` result) |
| FT-09 | AC-09 | JSON output includes `"cached"` and `"cache_age_secs"` fields | `ft09_033_render_json_cached_includes_fields` |
| FT-10 | AC-10 | Cached+expired account triggers `should_refresh()` | `mre_bug255_cache_defeats_refresh` |
| FT-11 | AC-11 | After retry OK, `cached` flag cleared and cache file written with fresh data | `mre_bug256_retry_ok_stale_cached_metadata` |

### Notes

- FT-01 through FT-07 are implemented as unit tests in `claude_profile_core/tests/account_test.rs`.
- FT-03 and FT-09 are implemented as render integration tests in `src/usage/render_tests.rs`.
- FT-08 is structural: cached rows are stored as `result: Ok(data)` with `cached: true` — all sort/strategy/next logic operates on `Ok` rows identically regardless of the `cached` flag.
- FT-10 is implemented as a unit test in `src/usage/refresh_predicate.rs` `#[cfg(test)]` module. MRE for BUG-255.
- FT-11 is a unit test in `src/usage/refresh_tests.rs`. Verifies the retry OK arm clears `aq.cached`/`aq.cache_age_secs` and writes the quota cache file. MRE for BUG-256.

---

### FT-01: Cache write preserves existing account fields

- **Given:** Account `alice` has `alice.json` containing `{"expires_at_ms": 12345, "token_count": 100}`. A quota update payload is ready to cache.
- **When:** The quota cache write function is called for `alice`.
- **Then:** `alice.json` retains `expires_at_ms: 12345` and `token_count: 100`; a `cache` sub-object containing the new quota fields is present.
- **Exit:** Ok(())
- **Source fn:** `cache_write_preserves_existing_fields`
- **Source:** [033_quota_cache.md AC-01](../../../docs/feature/033_quota_cache.md)

---

### FT-02: Cache read returns cached quota on fetch failure

- **Given:** `alice.json` contains a `cache` object with quota fields from a prior write.
- **When:** The live quota fetch for `alice` fails with a network error.
- **Then:** The returned `AccountQuota` uses the cached utilization values; no error is propagated to the caller.
- **Exit:** Ok(cached_data)
- **Source fn:** `cache_read_returns_entry_when_present`
- **Source:** [033_quota_cache.md AC-02](../../../docs/feature/033_quota_cache.md)

---

### FT-03: Cached display shows tilde prefix and age indicator

- **Given:** An `AccountQuota` with `cached: true` and `cache_age_secs: 300`.
- **When:** The usage row is rendered as text output.
- **Then:** The rendered line includes a `~` prefix on the utilization value and an age indicator (e.g., `~30%  5m`).
- **Exit:** rendered string contains `~`
- **Source fn:** `ft03_033_render_text_cached_shows_tilde_prefix`
- **Source:** [033_quota_cache.md AC-03](../../../docs/feature/033_quota_cache.md)

---

### FT-04: No cache means dashes (no-cache baseline unaffected)

- **Given:** `alice.json` exists but contains no `cache` sub-object.
- **When:** The live quota fetch fails and the cache is consulted.
- **Then:** The returned `AccountQuota` has all quota fields as dash/empty values — same as the live-fetch-absent baseline.
- **Exit:** Ok(empty_data)
- **Source fn:** `cache_read_returns_none_when_absent`
- **Source:** [033_quota_cache.md AC-04](../../../docs/feature/033_quota_cache.md)

---

### FT-05: Model override persists as `cache.model_override`

- **Given:** A quota cache write with `model_override = "opus"`.
- **When:** The cache is written and read back from `alice.json`.
- **Then:** `alice.json` contains `"cache": {"model_override": "opus"}`.
- **Exit:** Ok(())
- **Source fn:** `cache_field_string_persisted`
- **Source:** [033_quota_cache.md AC-05](../../../docs/feature/033_quota_cache.md)

---

### FT-06: Touch fields persist in cache

- **Given:** A quota cache write with `last_touch_at` timestamp and `touch_idle = true`.
- **When:** The cache is written and read back.
- **Then:** `alice.json` contains `"cache": {"last_touch_at": <ts>, "touch_idle": true}`.
- **Exit:** Ok(())
- **Source fn:** `cache_field_bool_persisted`
- **Source:** [033_quota_cache.md AC-06](../../../docs/feature/033_quota_cache.md)

---

### FT-07: Write→read round-trip preserves all quota fields

- **Given:** A full quota cache payload with all known fields set to non-default values.
- **When:** The payload is written via cache write, then read back via cache read.
- **Then:** All fields match the original payload exactly (no data loss or type corruption across the JSON serialization boundary).
- **Exit:** Ok(original_data)
- **Source fn:** `cache_write_read_roundtrip`
- **Source:** [033_quota_cache.md AC-07](../../../docs/feature/033_quota_cache.md)

---

### FT-08: Strategy logic operates on cached rows without special-casing

- **Given:** A batch of usage rows including some with `cached: true` and `result: Ok(data)`.
- **When:** Sort strategies, next-account selection, or row filtering are applied to the batch.
- **Then:** Cached rows participate in all strategy logic identically to live-fetched rows; no strategy short-circuits on the `cached` flag.
- **Exit:** N/A — structural invariant; `Ok` rows are treated uniformly regardless of `cached` flag.
- **Source fn:** structural (cached rows stored as `result: Ok(data)` with `cached: true`)
- **Source:** [033_quota_cache.md AC-08](../../../docs/feature/033_quota_cache.md)

---

### FT-09: JSON output includes `"cached"` and `"cache_age_secs"` fields

- **Given:** An `AccountQuota` with `cached: true` and `cache_age_secs: 120`.
- **When:** The usage row is rendered as JSON output.
- **Then:** The JSON object contains `"cached": true` and `"cache_age_secs": 120`.
- **Exit:** json object with both fields present
- **Source fn:** `ft09_033_render_json_cached_includes_fields`
- **Source:** [033_quota_cache.md AC-09](../../../docs/feature/033_quota_cache.md)

---

### FT-10: Cached and expired account triggers `should_refresh()`

- **Given:** Account `alice` has `cached: true` and `cache_age_secs` exceeding the staleness threshold; quota data originated from the cache (not a live fetch).
- **When:** `should_refresh(alice_quota)` is evaluated.
- **Then:** Returns `true` — the stale cached account is eligible for a fresh quota fetch attempt.
- **Exit:** true
- **Source fn:** `mre_bug255_cache_defeats_refresh`
- **Source:** [033_quota_cache.md AC-10](../../../docs/feature/033_quota_cache.md)

---

### FT-11: After retry OK, cached flag cleared and cache file written with fresh data

- **Given:** Account `alice` has `cached: true`; a retry quota fetch for `alice` returns `Ok(fresh_data)`.
- **When:** The retry OK arm processes the fresh response.
- **Then:** `aq.cached` is cleared to `false`; `aq.cache_age_secs` is cleared; the fresh quota data is written to the `alice.json` `cache` sub-object.
- **Exit:** Ok(fresh_data) with aq.cached = false
- **Source fn:** `mre_bug256_retry_ok_stale_cached_metadata`
- **Source:** [033_quota_cache.md AC-11](../../../docs/feature/033_quota_cache.md)
