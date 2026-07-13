# Feature: Quota Cache Fallback

### Scope

- **Purpose**: Persist last-known quota data in `{name}.json` so that when the usage API is unavailable (429, timeout, network error), the display shows cached values with a staleness indicator instead of dashes.
- **Responsibility**: Documents the cache write-on-success, read-on-failure mechanism, the `"cache"` key structure in `{name}.json`, staleness display, and touch/model state persistence.
- **In Scope**: Cache write after every successful `fetch_oauth_usage` call, proactive cache-first read when cache is ≤30 s old (skips live API call entirely — prevents burst-rate flooding), cache read when fetch returns any transient error, staleness indicator in display (`~` prefix on percentages, `(Nm ago)` age suffix), model override and touch state persistence in the same cache object, `{name}.json` read-merge-write (no new files).
- **Out of Scope**: Cache invalidation by time (stale data is always better than no data), separate cache files (all data goes into existing `{name}.json`), cache for `fetch_oauth_account` identity data (`org_created_at` has no persisted slot anywhere in `{name}.json` — tracked as BUG-327, not yet implemented).

### Design

When the usage API (`GET /api/oauth/usage`) returns an error for an account, the `.usage` table currently shows `—` for all quota columns. With this feature, the last successful fetch result is persisted in `{name}.json` under a `"cache"` top-level key, and displayed as fallback when the live fetch fails.

**Storage target**: `{name}.json` — the existing per-account metadata file in the credential store. Uses the established read-merge-write pattern (introduced by the 5-to-2 file consolidation). No new files are created.

**Cache structure** (new `"cache"` key in `{name}.json`):

```json
{
  "cache": {
    "fetched_at": "2026-06-07T07:52:00Z",
    "status": "ok",
    "five_hour": { "left_pct": 86.0, "resets_at": "2026-06-07T11:49:00Z" },
    "seven_day": { "left_pct": 16.0, "resets_at": "2026-06-07T16:00:00Z" },
    "seven_day_sonnet": { "left_pct": 0.0, "resets_at": "2026-06-07T16:00:00Z" },
    "model_override": "opus",
    "last_touch_at": "2026-06-07T06:30:00Z",
    "touch_idle": true
  }
}
```

**Algorithm:**

1. **On successful fetch**: After `fetch_oauth_usage` returns `Ok(usage_data)`, serialize the quota fields into the `"cache"` object and write to `{name}.json` via read-merge-write. The `fetched_at` timestamp is set to `now()` UTC ISO-8601. The `status` field is set to `"ok"`.
2. **On fetch error (transient errors only — 429, timeout, network)**: Read `{name}.json`, extract the `"cache"` object if present. If `cache.fetched_at` exists, compute `age_minutes = now - fetched_at`. Use cached quota values for display. Mark the row with a staleness indicator. **Auth errors (HTTP 401, HTTP 403) bypass cache fallback entirely** — they pass through as `Err` so `should_refresh()` can trigger a token refresh. Only transient errors fall back to cache; auth errors must remain `Err` so the refresh pipeline sees them. Fix for BUG-296.
3. **On model override**: After `apply_model_override` determines the target model, write `cache.model_override` to `{name}.json`.
4. **On touch completion**: After a successful touch subprocess, write `cache.last_touch_at` and `cache.touch_idle = false` to `{name}.json`.
5. **On successful retry after token refresh**: After `apply_refresh()` performs a token refresh and the quota retry returns `Ok(retried)`, set `aq.cached = false` and `aq.cache_age_secs = None` on the in-memory `AccountQuota`, then call `write_quota_cache()` with the fresh data. This clears the `~` staleness indicators and updates the on-disk cache so the next run starts from fresh data.

**Display with cached data:**

- Quota percentages are prefixed with `~` to indicate stale data: `~86%` instead of `86%`
- The `5h Reset` / `7d Reset` columns show the cached `resets_at` countdown (which may be in the past if stale — display `(stale)` when computed countdown is negative)
- The composite status emoji `●` is computed from cached values (same thresholds as live)
- A row-level age indicator shows time since last successful fetch: `(12m ago)` appended to the account name in the NAME cell (not an error-reason position — see AC-03)
- When the display originates from a cache-fallback conversion (a transient fetch error substituted with cached data — AC-02), the original failure reason is also preserved on the in-memory result and surfaced via `shorten_error()` in every render format (text table, TSV, JSON) — see AC-14. The text table combines it with the existing NAME-cell age suffix in one parenthetical; TSV has no pre-existing age-suffix mechanism, so it appends the shortened reason as its own standalone parenthetical instead. Live successes never carry a failure reason and render unchanged.

**Non-owned accounts (Feature 036 interaction):** When account ownership is enabled, non-owned accounts use the quota cache as their **primary** fetch source (G1 gate in Feature 036), not as a fallback. The cache read path, staleness display, and `~` prefix are identical to the error-fallback path — the distinction is only in how the cache-read was triggered. This means `write_quota_cache()` calls by the owning machine populate the cache that non-owner machines then read. Non-owned accounts where no cache exists show `—` for quota columns (same as no-cache graceful degradation).

**Graceful degradation:**

- If `{name}.json` has no `"cache"` key (first-ever fetch for this account, or file predates the feature): display dashes as before (no regression)
- If `cache.fetched_at` is unparseable: treat as no cache
- Cache is best-effort — write failures are silently ignored (quota display is non-critical)

### Acceptance Criteria

- **AC-01**: On successful `fetch_oauth_usage`, the `"cache"` key in `{name}.json` is written with `fetched_at`, `status`, and all quota fields.
- **AC-02**: On transient fetch error (429, timeout, network), if `{name}.json` contains a valid `"cache"` object, quota columns display cached values with `~` prefix. HTTP 401 and HTTP 403 errors are excluded from cache fallback.
- **AC-03**: When cached data is displayed, an age indicator (`(Nm ago)` or `(Nh ago)`) is appended to the account name in the NAME cell (not an error-reason position — see AC-14 for the separate fallback-reason indicator).
- **AC-04**: When no cache exists (fresh account, never fetched), display remains `—` (no regression from current behavior).
- **AC-05**: The `model_override` field is written to cache after `apply_model_override` executes.
- **AC-06**: The `last_touch_at` and `touch_idle` fields are written to cache after touch subprocess completion.
- **AC-07**: Cache write uses read-merge-write on `{name}.json` — existing fields (`host`, `model`, `oauthAccount`, `_renewal_at`) are preserved.
- **AC-08**: Strategy recommendations (`sort::`) operate on cached quota values when live data is unavailable — recommendations remain functional.
- **AC-09**: `format::json` output includes a `"cached": true` flag and `"cache_age_secs": N` field when displaying cached data.
- **AC-10**: When cache fallback converts a fetch error to `Ok(cached_data)` (AC-02 path), accounts whose local token is expired (`expires_at_ms / 1000 <= now_secs`) are still flagged for token refresh by `should_refresh()` via the `cached + expired` guard — the `Ok` result does not suppress refresh when `cached = true` and the token is locally expired.
- **AC-11**: After `apply_refresh()` executes a successful token refresh and quota retry (`retry OK`), `aq.cached` is reset to `false` and `aq.cache_age_secs` is cleared to `None` on the in-memory `AccountQuota`, and the fresh data is written to `{name}.json` via `write_quota_cache()`. The row no longer shows `~` prefix or `(Xh ago)` label, and the next run reads fresh cache data.
- **AC-12**: HTTP 401 and HTTP 403 auth errors from `fetch_oauth_usage` bypass cache fallback — `fetch_all_quota` returns `Err` (not `Ok(cached_data)`) for these error types. The `Err` propagates to `should_refresh()`, which triggers a token refresh attempt. Auth errors must not be masked by cache. Fix for BUG-296.
- **AC-13**: When `fetch_quota_for_list()` checks an owned, non-solo, non-occupied-elsewhere account and finds a cache entry ≤30 seconds old, the live API call (`GET /api/oauth/usage`) is skipped entirely; the cached data is served directly (`cached: true`, `cache_age_secs: N`). This cache-first guard fires after the G1/G1b/solo gates and after `is_current` is resolved, but before the local token-expiry check. Prevents API burst flooding from rapid-succession `.usage` invocations (test suites, polling scripts). The 30 s window is a constant `CACHE_FRESH_SECS` in `fetch.rs`.
- **AC-14**: When cache fallback converts a fetch error to `Ok(cached_data)` (AC-02 path), the original failure reason is preserved on the in-memory account result (`fallback_reason: Option<String>` field, populated only on this arm) and surfaced via `shorten_error()` in every render format: the text table appends the shortened reason alongside the existing NAME-cell age suffix (AC-03) in one parenthetical; the TSV format has no pre-existing age-suffix mechanism, so it appends the shortened reason as its own standalone NAME-cell parenthetical instead; JSON output emits a `"fallback_reason":"<shortened_reason>"` field alongside `"cached"`/`"cache_age_secs"` (AC-09). Live successes (`cached=false`) never populate `fallback_reason` and render unchanged. Auth errors (401/403) never reach the cache-fallback arm (AC-12), so `fallback_reason` is never populated from an auth rejection. Fix for BUG-335.

### Bugs

| File | Relationship |
|------|--------------|
| BUG-255 🟢 Fixed | Cache fallback Err→Ok conversion defeats `should_refresh()` — fixed via `cached + expired` guard in `should_refresh()` |
| BUG-256 🟢 Fixed | `retry OK` does not clear `cached` metadata — `~` and `(Xh ago)` persist after successful refresh; fix = AC-11 |
| BUG-288 🟢 Fixed (Fix A) | Fix A complete: `apply_post_switch_touch` now calls `write_quota_cache` with post-subprocess quota data; subsequent `apply_touch` reads updated quota (`resets_at = Some`) and skips the redundant subprocess. Fix B (`touch_idle` read site in `apply_touch` as defense-in-depth for server-side propagation lag) deferred; `touch_idle=false` write (AC-06) remains dead code pending follow-on task. |
| BUG-296 🟢 Fixed (TSK-306) | Auth-error guard added: `fetch.rs:235` changes fallback arm to `Err( ref e ) if !e.contains("401") && !e.contains("403")` — auth errors propagate as `Err`; transient errors still fall back to cache; fix = AC-12 |
| BUG-304 🟢 Fixed (TSK-316) | Three independent cache-read paths reconstructed `OauthUsageData` for utilization; G1 (non-owned) applied no approximation, HTTP-error fallback and `approximate_quota()` each inlined 40–55 lines of duplicated approximation. Fixed: centralized `read_cached_quota()` function |
| BUG-327 🔴 Open | `QuotaCacheEntry` (`claude_profile_core/src/account.rs:1506-1522`) has no `org_created_at` field — every non-live-fetch branch in `fetch.rs` hardcodes `account: None`, so `~Renews` shows `?` for 15/18 accounts. Root cause: false premise at this doc's own Out of Scope line (identity data was never persisted anywhere, despite this file previously claiming otherwise). Two accounts escape via the fully-decoupled `_renewal_at` manual override. |
| BUG-335 🟢 Fixed (TSK-416) | Cache-fallback `Ok(data)` render row never called `shorten_error()` — the original fetch-failure reason (e.g. HTTP 429) was discarded once the fallback arm converted `Err` to `Ok(cached_data)`, so text/TSV/JSON render paths showed only the `~` prefix and age suffix with zero trace of why the row was stale. Fixed via new `AccountQuota.fallback_reason` field, populated only in `fetch.rs`'s cache-fallback arm; fix = AC-14 |

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Live quota reporting — this feature adds fallback when live fetch fails |
| [024_session_touch.md](024_session_touch.md) | Touch lifecycle — cache persists touch state |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | Model override — cache persists override decision |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `{name}.json` structure — cache extends the same file |
| [036_account_ownership.md](036_account_ownership.md) | G1: non-owned accounts use cache as primary source; same display path as cache-fallback |
| [040_quota_measurement_history.md](040_quota_measurement_history.md) | Extends single-point cache with 10-entry measurement history ring buffer and polynomial approximation |
| [061_solo_token_conservation.md](061_solo_token_conservation.md) | `approximate_quota()` reads the single-point cache as fallback when history is absent |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/fetch.rs` | Cache write on fetch success (`write_quota_cache`); cache read on fetch error (`read_quota_cache`) |
| `src/usage/refresh.rs` | `apply_refresh()` retry cache write — clears `aq.cached`/`aq.cache_age_secs` and calls `write_quota_cache()` after `retry OK` (AC-11); `should_refresh()` `cached + expired` guard (AC-10) |
| `src/usage/render.rs` | Staleness display (text table) — `~` prefix via `prefix_tilde()`, `(Nm ago)` age label, `(stale)` markers; NAME-cell fallback-reason suffix (AC-14) |
| `src/usage/render_tsv.rs` | Staleness display (TSV format) — same `~` prefix surfacing as `render.rs`, TSV-encoded; NAME-cell fallback-reason suffix (AC-14) is a standalone parenthetical — this format has no age-suffix mechanism to combine it with |
| `src/usage/render_json.rs` | Staleness display (JSON format) — `cache_json_fields()` emits `"cached"`/`"cache_age_secs"` (AC-09); `"fallback_reason"` field (AC-14) |
| `src/usage/format.rs` | `shorten_error()` — failure-reason shortening shared by all three render formats (AC-03/AC-14); `cache_age_label()` — age-suffix formatting (AC-03); `status_emoji()` — threshold-based status coloring, cache-blind by design |
| `src/usage/api.rs` | Side-effect cache — `write_cache_string()` (model_override, AC-05) and `write_cache_bool()` (touch_idle, AC-06) |
| `claude_profile_core/src/account.rs` | Storage layer — `QuotaCacheEntry`, `read_quota_cache()`, `write_quota_cache()`, `write_cache_field()` |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | Unified `{name}.json` field table — `cache` subtree owned by this feature |
