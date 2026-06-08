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
- FT-03 and FT-09 are implemented as render integration tests in `claude_profile/src/usage/render.rs` (tests module).
- FT-08 is structural: cached rows are stored as `result: Ok(data)` with `cached: true` — all sort/strategy/next logic operates on `Ok` rows identically regardless of the `cached` flag.
- FT-10 is implemented as a unit test in `src/usage/refresh.rs` `#[cfg(test)]` module (private fn test). MRE for BUG-255.
- FT-11 is a unit test in `src/usage/refresh.rs` `#[cfg(test)]` module. Verifies the retry OK arm clears `aq.cached`/`aq.cache_age_secs` and writes the quota cache file. MRE for BUG-256.
