# Feature: Quota Cache Fallback

### Scope

- **Purpose**: Persist last-known quota data in `{name}.json` so that when the usage API is unavailable (429, timeout, network error), the display shows cached values with a staleness indicator instead of dashes.
- **Responsibility**: Documents the cache write-on-success, read-on-failure mechanism, the `"cache"` key structure in `{name}.json`, staleness display, and touch/model state persistence.
- **In Scope**: Cache write after every successful `fetch_oauth_usage` call, cache read when fetch returns any error, staleness indicator in display (`~` prefix on percentages, `(Nm ago)` age suffix), model override and touch state persistence in the same cache object, `{name}.json` read-merge-write (no new files).
- **Out of Scope**: Cache invalidation by time (stale data is always better than no data), separate cache files (all data goes into existing `{name}.json`), cache for `fetch_oauth_account` identity data (already persisted in `oauthAccount` field).

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
2. **On fetch error**: Read `{name}.json`, extract the `"cache"` object if present. If `cache.fetched_at` exists, compute `age_minutes = now - fetched_at`. Use cached quota values for display. Mark the row with a staleness indicator.
3. **On model override**: After `apply_model_override` determines the target model, write `cache.model_override` to `{name}.json`.
4. **On touch completion**: After a successful touch subprocess, write `cache.last_touch_at` and `cache.touch_idle = false` to `{name}.json`.
5. **On successful retry after token refresh**: After `apply_refresh()` performs a token refresh and the quota retry returns `Ok(retried)`, set `aq.cached = false` and `aq.cache_age_secs = None` on the in-memory `AccountQuota`, then call `write_quota_cache()` with the fresh data. This clears the `~` staleness indicators and updates the on-disk cache so the next run starts from fresh data.

**Display with cached data:**

- Quota percentages are prefixed with `~` to indicate stale data: `~86%` instead of `86%`
- The `5h Reset` / `7d Reset` columns show the cached `resets_at` countdown (which may be in the past if stale — display `(stale)` when computed countdown is negative)
- The composite status emoji `●` is computed from cached values (same thresholds as live)
- A row-level age indicator shows time since last successful fetch: `(12m ago)` appended to the error reason column

**Graceful degradation:**

- If `{name}.json` has no `"cache"` key (first-ever fetch for this account, or file predates the feature): display dashes as before (no regression)
- If `cache.fetched_at` is unparseable: treat as no cache
- Cache is best-effort — write failures are silently ignored (quota display is non-critical)

### Acceptance Criteria

- **AC-01**: On successful `fetch_oauth_usage`, the `"cache"` key in `{name}.json` is written with `fetched_at`, `status`, and all quota fields.
- **AC-02**: On fetch error (429, timeout, network), if `{name}.json` contains a valid `"cache"` object, quota columns display cached values with `~` prefix.
- **AC-03**: When cached data is displayed, an age indicator (`(Nm ago)` or `(Nh ago)`) appears in the error reason position.
- **AC-04**: When no cache exists (fresh account, never fetched), display remains `—` (no regression from current behavior).
- **AC-05**: The `model_override` field is written to cache after `apply_model_override` executes.
- **AC-06**: The `last_touch_at` and `touch_idle` fields are written to cache after touch subprocess completion.
- **AC-07**: Cache write uses read-merge-write on `{name}.json` — existing fields (`host`, `model`, `oauthAccount`, `_renewal_at`) are preserved.
- **AC-08**: Strategy recommendations (`next::`, sort) operate on cached quota values when live data is unavailable — recommendations remain functional.
- **AC-09**: `format::json` output includes a `"cached": true` flag and `"cache_age_secs": N` field when displaying cached data.
- **AC-10**: When cache fallback converts a fetch error to `Ok(cached_data)` (AC-02 path), accounts whose local token is expired (`expires_at_ms / 1000 <= now_secs`) are still flagged for token refresh by `should_refresh()` via the `cached + expired` guard — the `Ok` result does not suppress refresh when `cached = true` and the token is locally expired.
- **AC-11**: After `apply_refresh()` executes a successful token refresh and quota retry (`retry OK`), `aq.cached` is reset to `false` and `aq.cache_age_secs` is cleared to `None` on the in-memory `AccountQuota`, and the fresh data is written to `{name}.json` via `write_quota_cache()`. The row no longer shows `~` prefix or `(Xh ago)` label, and the next run reads fresh cache data.

### Bugs

| File | Relationship |
|------|--------------|
| [BUG-255 🟢 Fixed](../../../../../task/claude_profile/bug/255_cache_fallback_defeats_should_refresh.md) | Cache fallback Err→Ok conversion defeats `should_refresh()` — fixed via `cached + expired` guard in `should_refresh()` |
| [BUG-256 🟢 Fixed](../../../../../task/claude_profile/bug/256_retry_ok_does_not_clear_cached_metadata.md) | `retry OK` does not clear `cached` metadata — `~` and `(Xh ago)` persist after successful refresh; fix = AC-11 |

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Live quota reporting — this feature adds fallback when live fetch fails |
| [024_session_touch.md](024_session_touch.md) | Touch lifecycle — cache persists touch state |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | Model override — cache persists override decision |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `{name}.json` structure — cache extends the same file |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/fetch.rs` | Cache write on fetch success (`write_quota_cache`); cache read on fetch error (`read_quota_cache`) |
| `src/usage/refresh.rs` | `apply_refresh()` retry cache write — clears `aq.cached`/`aq.cache_age_secs` and calls `write_quota_cache()` after `retry OK` (AC-11); `should_refresh()` `cached + expired` guard (AC-10) |
| `src/usage/render.rs` | Staleness display — `~` prefix via `prefix_tilde()`, `(Nm ago)` age label, `(stale)` markers, `cache_json_fields()` |
| `src/usage/api.rs` | Side-effect cache — `write_cache_string()` (model_override, AC-05) and `write_cache_bool()` (touch_idle, AC-06) |
| `claude_profile_core/src/account.rs` | Storage layer — `QuotaCacheEntry`, `read_quota_cache()`, `write_quota_cache()`, `write_cache_field()` |
