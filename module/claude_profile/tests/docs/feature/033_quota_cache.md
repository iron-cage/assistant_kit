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
| FT-15 | AC-15 | Non-live-fetch branch (cache-first, G1-not-owned, or `approximate_quota()`) surfaces a cached `org_created_at` through `AccountQuota.org_created_at`, producing a real `~Renews` Estimate value instead of `"?"`; absent/pre-migration cache gracefully falls back to `None` | `mre_bug327_cache_first_surfaces_org_created_at` (cache-first branch only — see Notes) |

### Notes

- FT-01 through FT-07 are implemented as unit tests in `claude_profile_core/tests/account_test.rs`.
- FT-03 and FT-09 are implemented as render integration tests in `tests/usage/render_tests_a.rs`.
- FT-08 is structural: cached rows are stored as `result: Ok(data)` with `cached: true` — all sort/strategy/next logic operates on `Ok` rows identically regardless of the `cached` flag.
- FT-10 is implemented as an integration test in `tests/usage/refresh_predicate_tests.rs` (via `claude_profile::usage::test_bridge`, since `should_refresh` is `pub(super)`) — not a `#[cfg(test)]` module inside `src/usage/refresh_predicate.rs`, which itself notes "Tests live in tests/usage/refresh_predicate_tests.rs". MRE for BUG-255.
- FT-11 is a unit test in `tests/usage/refresh_tests_b.rs`. Verifies the retry OK arm clears `aq.cached`/`aq.cache_age_secs` and writes the quota cache file. MRE for BUG-256.
- FT-12 is a unit test in `tests/usage/fetch_tests.rs`. Verifies that the cache fallback match guard `Err( ref e ) if !e.contains("401") && !e.contains("403")` is present, and that a catch-all `Err` arm propagates auth errors without cache conversion. MRE for BUG-296.
- FT-14 is implemented as a single integration test in `tests/usage/render_tests_a.rs` exercising `render_text`/`render_tsv`/`render_json` together against one `AccountQuota` with `fallback_reason: Some(...)`. **Correction (found during implementation):** `render_tsv.rs` has no pre-existing cache-age-suffix mechanism (unlike `render.rs`) — the original AC-03/AC-14 wording implying TSV combines the reason with an age label was inaccurate; TSV's shortened reason renders as its own standalone parenthetical, e.g. `alice (rate limited (429))`, vs. text's combined `alice (2h ago, rate limited (429))`. MRE for BUG-335.
- FT-15 is partially implemented. `mre_bug327_cache_first_surfaces_org_created_at` (in `tests/usage/fetch_tests.rs`, lines 794-882) covers the cache-first branch only: it writes a quota cache via the real `write_quota_cache()`/`write_cache_string()` production path, drives `fetch_quota_for_list()` with no live credentials file (forcing cache-first before any HTTP path), and asserts `AccountQuota.org_created_at` surfaces the cached value and that `renews_label()` renders a `"~in "`-prefixed estimate instead of `"?"`. Two coverage gaps remain unimplemented: (1) the G1-not-owned and `approximate_quota()` branches are not exercised by this or any other test — only cache-first is; (2) the absent-cache/pre-migration fallback (`org_created_at` key absent → `None` → `~Renews` renders `"?"` unchanged) and a `claude_profile_core/tests/account_test.rs` unit test round-tripping `write_quota_cache()`/`read_quota_cache()` for the new field are both still missing, tracked as an implementation gap for BUG-327's fix task.

---

### FT-01: Cache write preserves existing account fields

- **Given:** Account `alice@acme.com` has `alice@acme.com.json` containing `{"host":"wbox","role":"dev"}`. A quota update payload is ready to cache.
- **When:** `write_quota_cache()` is called for `alice@acme.com` with a `five_hour`/`seven_day` payload.
- **Then:** `alice@acme.com.json` retains `"host": "wbox"` and `"role": "dev"`; a `"cache"` sub-object is present containing `fetched_at`, `five_hour` (with `left_pct`), and `seven_day`.
- **Exit:** n/a (`write_quota_cache` returns `()`)
- **Note:** Corrected the preserved-field example — `expires_at_ms`/`token_count` do not appear anywhere in the cited test; the actual pre-existing fixture fields are `host`/`role` (Feature 029 profile metadata).
- **Source fn:** `cache_write_preserves_existing_fields`
- **Source:** [033_quota_cache.md AC-01](../../../docs/feature/033_quota_cache.md)

---

### FT-02: Cache read returns cached quota on fetch failure

- **Given:** `carol@acme.com.json` contains a fully-populated `"cache"` object (`fetched_at`, `status`, `five_hour.left_pct`/`resets_at`, `seven_day.left_pct`, `model_override`, `last_touch_at`, `touch_idle`) — written directly, not via a simulated fetch failure.
- **When:** `read_quota_cache(store.path(), name)` is called directly.
- **Then:** Returns `Some(QuotaCacheEntry)` with every field matching the written values; `seven_day_sonnet` is `None` (absent from the fixture).
- **Exit:** `Some(entry)`
- **Note:** The cited test verifies `read_quota_cache()`'s JSON-parsing correctness against a hand-written cache blob — it does not simulate a live fetch failure, construct an `AccountQuota`, or exercise `fetch_all_quota`'s cache-fallback conversion path. No test in this suite drives that full "transient fetch error → `Ok(cached_data)`" path end-to-end; `mre_bug255_cache_defeats_refresh` (FT-10) and `mre_bug296_cached_non_expired_401_no_refresh` (FT-12) cover adjacent pieces (the refresh-eligibility predicate, and a structural source-text guard) but neither drives the fallback conversion itself.
- **Source fn:** `cache_read_returns_entry_when_present`
- **Source:** [033_quota_cache.md AC-02](../../../docs/feature/033_quota_cache.md)

---

### FT-03: Cached display shows tilde prefix and age indicator

- **Given:** An `AccountQuota` with `cached: true`, `cache_age_secs: Some(300)`, and `five_hour` utilization `14.0` (86% left).
- **When:** The usage row is rendered as text output via `render_text`.
- **Then:** The rendered line contains a `~` prefix on the 5h-left cell; the exact cell text is `~🟢 86%` (green, since 86% left is healthy).
- **Exit:** rendered string contains `~🟢 86%`
- **Note:** Corrected the illustrative `~30%  5m` example — the cited test's fixture renders `~🟢 86%` (14% utilization → 86% left) and only asserts the `~` prefix plus that exact cell string. It does not assert any `Xm`/`Xh ago`-style age-suffix text (that mechanism exists elsewhere — e.g. FT-11's `(Xh ago)` label — but is not exercised by this specific test).
- **Source fn:** `ft03_033_render_text_cached_shows_tilde_prefix`
- **Source:** [033_quota_cache.md AC-03](../../../docs/feature/033_quota_cache.md)

---

### FT-04: No cache means dashes (no-cache baseline unaffected)

- **Given:** `bob@acme.com.json` exists (`{"host":"wbox"}`) but contains no `"cache"` sub-object.
- **When:** `read_quota_cache(store.path(), name)` is called directly.
- **Then:** Returns `None`.
- **Exit:** `None`
- **Note:** The cited test verifies `read_quota_cache()` returns `None` for a cache-less file — it does not simulate a live fetch failure or construct an `AccountQuota` to confirm the "dash/empty values" rendering baseline; that rendering behavior is a separate, unverified claim by this citation.
- **Source fn:** `cache_read_returns_none_when_absent`
- **Source:** [033_quota_cache.md AC-04](../../../docs/feature/033_quota_cache.md)

---

### FT-05: Model override persists as `cache.model_override`

- **Given:** A quota cache already exists for the account (written via `write_quota_cache`).
- **When:** `write_cache_string(store.path(), name, "model_override", "opus")` is called.
- **Then:** The `cache` sub-object contains `"model_override": "opus"`; pre-existing quota fields (e.g. `five_hour`) survive the field write.
- **Exit:** n/a (`write_cache_string` returns `()`)
- **Note:** `model_override` is set via a separate `write_cache_string()` call, not as part of the initial `write_quota_cache()` call this FT case's original wording implied.
- **Source fn:** `cache_field_string_persisted`
- **Source:** [033_quota_cache.md AC-05](../../../docs/feature/033_quota_cache.md)

---

### FT-06: Touch fields persist in cache

- **Given:** A quota cache already exists for the account (written via `write_quota_cache`).
- **When:** `write_cache_bool(store.path(), name, "touch_idle", false)` is called.
- **Then:** The `cache` sub-object contains `"touch_idle": false`; pre-existing quota fields (e.g. `five_hour`) survive the field write.
- **Exit:** n/a (`write_cache_bool` returns `()`)
- **Note:** Corrected `touch_idle` from `true` to `false` — matching the cited test's actual fixture. `last_touch_at` persistence is not exercised by this test at all (it only writes/reads `touch_idle`); it is separately verified by `cache_read_returns_entry_when_present` (FT-02) parsing a hand-written fixture that includes `last_touch_at`.
- **Source fn:** `cache_field_bool_persisted`
- **Source:** [033_quota_cache.md AC-06](../../../docs/feature/033_quota_cache.md)

---

### FT-07: Write→read round-trip preserves all quota fields

- **Given:** A quota cache payload with `five_hour` and `seven_day_sonnet` set to non-default values; `seven_day` deliberately left `None` (absent).
- **When:** The payload is written via `write_quota_cache`, then read back via `read_quota_cache`.
- **Then:** `five_hour` and `seven_day_sonnet` match the original values exactly (utilization + reset timestamp, no data loss or type corruption across the JSON boundary); `seven_day` reads back as `None`, matching the write.
- **Exit:** `Some(entry)` with fields as above
- **Note:** Corrected "all known fields set to non-default values" — the cited test exercises only `write_quota_cache`'s three quota-tuple parameters (`five_hour`/`seven_day`/`seven_day_sonnet`), one of which (`seven_day`) is intentionally `None`; `model_override`/`last_touch_at`/`touch_idle` (written via the separate `write_cache_string`/`write_cache_bool` functions) are not exercised by this test.
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

- **Given:** An `AccountQuota` with `cached: true` and `cache_age_secs: Some(720)`.
- **When:** The usage row is rendered as JSON output via `render_json`.
- **Then:** The JSON object contains `"cached":true` and `"cache_age_secs":720`.
- **Exit:** json object with both fields present
- **Note:** Corrected `cache_age_secs` from `120` to `720` — matching the cited test's actual fixture.
- **Source fn:** `ft09_033_render_json_cached_includes_fields`
- **Source:** [033_quota_cache.md AC-09](../../../docs/feature/033_quota_cache.md)

---

### FT-10: Cached and expired account triggers `should_refresh()`

- **Given:** Account `alice` has `cached: true`, `result: Ok(quota)` (cache fallback converted an earlier `Err` to `Ok`), and a locally expired token (`expires_at_ms: 0`, i.e. `expires_at_ms / 1000 <= now_secs`).
- **When:** `should_refresh(&alice_quota, now_secs)` is evaluated.
- **Then:** Returns `true` — an expired token behind a cache hit still needs refreshing.
- **Exit:** true
- **Note:** Corrected the trigger condition from "`cache_age_secs` exceeding the staleness threshold" — `should_refresh`'s cached-account branch (`aq.cached && (aq.expires_at_ms / 1000) <= now_secs`, `src/usage/refresh_predicate.rs`) never reads `cache_age_secs`; the trigger is the token's own `expires_at_ms`, independent of cache age. The cited test does set `cache_age_secs: Some(3600)` in its fixture, but that field plays no role in the assertion.
- **Source fn:** `mre_bug255_cache_defeats_refresh`
- **Source:** [033_quota_cache.md AC-10](../../../docs/feature/033_quota_cache.md)

---

### FT-11: After retry OK, cached flag cleared and cache file written with fresh data

- **Given:** `src/usage/refresh.rs`'s `apply_refresh` function, specifically its `Ok( retried ) =>` retry-success arm.
- **When:** The test reads `refresh.rs`'s own source text (`include_str!`) and searches within the located `Ok( retried ) =>` arm — this is a structural/source-text assertion, not a constructed `AccountQuota`/simulated-retry scenario.
- **Then:** The arm's source text contains `aq.cached         = false`, `aq.cache_age_secs = None`, and a `write_quota_cache(` call; `write_quota_cache(` appears textually BEFORE `aq.result         = Ok( retried )` (confirming the cache write reads `retried`'s fields before they're moved into `aq.result`, avoiding a use-after-move).
- **Exit:** N/A — structural/source-text assertion, not a runtime `Ok`/`Err` result
- **Note:** Fix for BUG-256. This is a structural test — it greps `refresh.rs`'s own source text for the three AC-11 mutations and their relative order, rather than invoking `apply_refresh` with a constructed `AccountQuota` and a mocked retry response.
- **Source fn:** `mre_bug256_retry_ok_stale_cached_metadata` (in `tests/usage/refresh_tests_b.rs`)
- **Source:** [033_quota_cache.md AC-11](../../../docs/feature/033_quota_cache.md)

---

### FT-12: HTTP 401 / 403 auth errors bypass cache fallback

- **Given:** `src/usage/fetch.rs`'s cache-fallback `match` arm inside `fetch_quota_for_list`.
- **When:** The test reads `fetch.rs`'s own source text (`include_str!`) and searches for the auth-error match guard and catch-all arm — this is a structural/source-text assertion, not a constructed `AccountQuota`/simulated-fetch scenario.
- **Then:** The guard string `!e.contains( "401" ) && !e.contains( "403" )` is present; `read_cached_quota( credential_store` appears textually AFTER that guard (confirming the cache read is gated behind the auth-error exclusion); the catch-all arm `Err( _ ) => ( result, false, None, None, None )` is present (confirming 401/403 propagate as `Err` with `cached=false`, never converted to `Ok(cached_data)`). HTTP 403 is covered by the same guard string.
- **Exit:** N/A — structural/source-text assertion, not a runtime `Err`/`Ok` result
- **Source fn:** `mre_bug296_cached_non_expired_401_no_refresh` (in `tests/usage/fetch_tests.rs`)
- **Note:** Fix for BUG-296. This is a structural test — it greps `fetch.rs`'s own source text for the guard/catch-all pattern rather than constructing an `AccountQuota` and invoking `fetch_quota_for_list` with a mocked 401 response. Auth-error guard: `Err( ref e ) if !e.contains("401") && !e.contains("403") =>` on the cache fallback arm; a catch-all `Err( _ ) =>` arm propagates auth errors unchanged. Only transient errors (429, network, timeout) trigger cache fallback in the real match arm this test inspects.
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

- **Given:** Account `alice` has a `cache` sub-object in `alice.json` containing `org_created_at: "2024-01-01T00:00:00Z"` from a prior live `fetch_oauth_account` call. A `.usage` invocation takes a non-live-fetch branch for `alice` (cache-first: cache age ≤30s; or G1-not-owned; or `approximate_quota()` under `solo::1`) with no `_renewal_at` override set.
- **When:** The branch constructs `alice`'s `AccountQuota` and the row is rendered.
- **Then:** `AccountQuota.org_created_at` is `Some("2024-01-01T00:00:00Z")` (read back from `cache.org_created_at`, independent of `AccountQuota.account` which remains `None` on these branches); `renews_label()` computes a real `~Renews` Estimate value (`"~in "`-prefixed) from it instead of returning `"?"`.
- **Exit:** `aq.org_created_at == Some("2024-01-01T00:00:00Z")`; rendered `~Renews` cell is not `"?"`
- **Source fn:** `mre_bug327_cache_first_surfaces_org_created_at` (in `tests/usage/fetch_tests.rs`) — covers the cache-first branch only; the G1-not-owned and `approximate_quota()` branches named in this case's Given are NOT exercised by this or any other test
- **Note:** Fix for BUG-327. A second scenario in the same test (or a sibling test) must cover the absent-cache / pre-migration-cache case: no `org_created_at` key present → `AccountQuota.org_created_at` is `None` → `~Renews` renders `"?"` unchanged (no regression, AC-15's graceful-fallback clause). A `claude_profile_core/tests/account_test.rs` unit test must separately cover `write_quota_cache()`/`read_quota_cache()` round-tripping the new field. Both remain unimplemented coverage gaps.
- **Source:** [033_quota_cache.md AC-15](../../../docs/feature/033_quota_cache.md)
