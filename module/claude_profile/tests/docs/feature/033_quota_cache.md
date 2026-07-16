# FT — Feature 033: Quota Cache Fallback

### Scope

- **Purpose**: Test cases for quota cache fallback behavior — write-on-success, read-on-failure, staleness display, and side-effect persistence.
- **Source**: `docs/feature/033_quota_cache.md`
- **Covers**: AC-01 through AC-15

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
| FT-12 | AC-12 | HTTP 401 / 403 auth errors bypass cache fallback — `Err` propagates | `mre_bug296_cached_non_expired_401_no_refresh` |
| FT-14 | AC-14 | Cache-fallback row preserves the original failure reason and surfaces it via `shorten_error()` in text, TSV, and JSON render formats (text combines it with the AC-03 age suffix; TSV has no age suffix to combine with, so it stands alone) | `mre_bug335_cache_fallback_reason_surfaced_on_all_render_surfaces` |
| FT-15 | AC-15 | Non-live-fetch branch (cache-first, G1-not-owned, or `approximate_quota()`) surfaces a cached `org_created_at` through `AccountQuota.org_created_at`, producing a real `~Renews` Estimate value instead of `"?"`; absent/pre-migration cache gracefully falls back to `None` | `mre_bug327_org_created_at_surfaced_from_cache_on_non_live_branches` |

### Notes

- FT-01 through FT-07 are implemented as unit tests in `claude_profile_core/tests/account_test.rs`.
- FT-03 and FT-09 are implemented as render integration tests in `tests/usage/render_tests_a.rs`.
- FT-08 is structural: cached rows are stored as `result: Ok(data)` with `cached: true` — all sort/strategy/next logic operates on `Ok` rows identically regardless of the `cached` flag.
- FT-10 is implemented as a unit test in `src/usage/refresh_predicate.rs` `#[cfg(test)]` module. MRE for BUG-255.
- FT-11 is a unit test in `tests/usage/refresh_tests_b.rs`. Verifies the retry OK arm clears `aq.cached`/`aq.cache_age_secs` and writes the quota cache file. MRE for BUG-256.
- FT-12 is a unit test in `tests/usage/fetch_tests.rs`. Verifies that the cache fallback match guard `Err( ref e ) if !e.contains("401") && !e.contains("403")` is present, and that a catch-all `Err` arm propagates auth errors without cache conversion. MRE for BUG-296.
- FT-14 is implemented as a single integration test in `tests/usage/render_tests_a.rs` exercising `render_text`/`render_tsv`/`render_json` together against one `AccountQuota` with `fallback_reason: Some(...)`. **Correction (found during implementation):** `render_tsv.rs` has no pre-existing cache-age-suffix mechanism (unlike `render.rs`) — the original AC-03/AC-14 wording implying TSV combines the reason with an age label was inaccurate; TSV's shortened reason renders as its own standalone parenthetical, e.g. `alice (rate limited (429))`, vs. text's combined `alice (2h ago, rate limited (429))`. MRE for BUG-335.
- FT-15 is not yet implemented — tracked as an implementation gap for BUG-327's fix task. Expected location: a unit test in `claude_profile_core/tests/account_test.rs` covering `write_quota_cache()`/`read_quota_cache()` round-tripping the new `org_created_at` field, plus an integration test in `tests/usage/fetch_tests.rs` (or nearest existing fetch test file) constructing a cache-first/G1-not-owned/`approximate_quota()` scenario and asserting `AccountQuota.org_created_at` is `Some` when the cache carries the field and `None` when it does not.

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

---

### FT-12: HTTP 401 / 403 auth errors bypass cache fallback

- **Given:** Account `alice` has a valid `cache` sub-object in `alice.json` with quota data from a prior successful fetch; the live quota fetch returns an HTTP 401 auth error for `alice`.
- **When:** `fetch_all_quota` processes the auth-error result for `alice`.
- **Then:** The 401 error propagates as `Err` — the cache fallback is NOT triggered; `Ok(cached_data)` is NOT returned. The auth error reaches `should_refresh()` unchanged, which can trigger a token refresh attempt. HTTP 403 is treated identically.
- **Exit:** `Err("...401...")` (auth error propagates unchanged)
- **Source fn:** `mre_bug296_cached_non_expired_401_no_refresh` (in `tests/usage/fetch_tests.rs`)
- **Note:** Fix for BUG-296. Auth-error guard: `Err( ref e ) if !e.contains("401") && !e.contains("403") =>` on the cache fallback arm; a catch-all `Err( _ ) =>` arm propagates auth errors unchanged. Only transient errors (429, network, timeout) trigger cache fallback.
- **Source:** [033_quota_cache.md AC-12](../../../docs/feature/033_quota_cache.md)

---

### FT-14: Cache-fallback row surfaces the original failure reason on all 3 render formats

- **Given:** An `AccountQuota` with `cached: true`, `cache_age_secs: 7200`, and `fallback_reason: Some("HTTP transport error: HTTP 429 Too Many Requests")` — the reason a cache-fallback `Err→Ok` conversion carried forward.
- **When:** The row is rendered as text, TSV, and JSON output.
- **Then:** Text combines the shortened reason with the existing age suffix in one NAME-cell parenthetical: `alice (2h ago, rate limited (429))`. TSV — which has no pre-existing age-suffix mechanism — appends the shortened reason as its own standalone parenthetical: `alice (rate limited (429))`. JSON emits a new field: `"fallback_reason":"rate limited (429)"`.
- **Exit:** all 3 rendered outputs contain the shortened reason `rate limited (429)`
- **Source fn:** `mre_bug335_cache_fallback_reason_surfaced_on_all_render_surfaces` (in `tests/usage/render_tests_a.rs`)
- **Note:** Fix for BUG-335. `shorten_error()` shortens raw reasons starting with `"HTTP transport error: HTTP 429"` to `"rate limited (429)"` (see `src/usage/format.rs`). Text and TSV diverge in combination strategy solely because TSV never had an age-suffix mechanism to begin with — this is not an inconsistency to reconcile, it reflects each format's actual pre-existing capability.
- **Source:** [033_quota_cache.md AC-14](../../../docs/feature/033_quota_cache.md)

---

### FT-15: Cached `org_created_at` surfaces through non-live-fetch branches

- **Given:** Account `alice` has a `cache` sub-object in `alice.json` containing `org_created_at: "2026-01-01T00:00:00Z"` from a prior live `fetch_oauth_account` call. A `.usage` invocation takes a non-live-fetch branch for `alice` (cache-first: cache age ≤30s; or G1-not-owned; or `approximate_quota()` under `solo::1`) with no `_renewal_at` override set.
- **When:** The branch constructs `alice`'s `AccountQuota` and the row is rendered.
- **Then:** `AccountQuota.org_created_at` is `Some("2026-01-01T00:00:00Z")` (read back from `cache.org_created_at`, independent of `AccountQuota.account` which remains `None` on these branches); `renews_label()` computes a real `~Renews` Estimate value from it instead of returning `"?"`.
- **Exit:** `aq.org_created_at == Some("2026-01-01T00:00:00Z")`; rendered `~Renews` cell is not `"?"`
- **Source fn:** `mre_bug327_org_created_at_surfaced_from_cache_on_non_live_branches` (expected location: `tests/usage/fetch_tests.rs`)
- **Note:** Fix for BUG-327. A second scenario in the same test (or a sibling test) must cover the absent-cache / pre-migration-cache case: no `org_created_at` key present → `AccountQuota.org_created_at` is `None` → `~Renews` renders `"?"` unchanged (no regression, AC-15's graceful-fallback clause). A `claude_profile_core/tests/account_test.rs` unit test must separately cover `write_quota_cache()`/`read_quota_cache()` round-tripping the new field.
- **Source:** [033_quota_cache.md AC-15](../../../docs/feature/033_quota_cache.md)
