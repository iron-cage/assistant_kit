# Feature: Quota Cache Fallback

### Scope

- **Purpose**: Persist last-known quota data so that when the usage API is unavailable (429, timeout, network error), the display shows cached values with a staleness indicator instead of dashes — without dirtying the git-tracked credential files on every fetch (TSK-500), and with fleet-wide visibility: every host's cache rides ordinary commits, so a host that never fetched an account still displays the freshest data any host has (TSK-502).
- **Responsibility**: Documents the two-tier cache storage (volatile quota data in the tracked per-host tree `cache/{host}_{user}/{name}.json`, low-churn metadata as top-level keys of the tracked `{name}.json`), the freshest-wins merge read, the write-on-success/read-on-failure mechanism, the legacy migrations (tracked `cache{}` block and gitignored `-cache/` file), staleness display, and touch/model state persistence.
- **In Scope**: Cache write after every successful `fetch_oauth_usage` call, proactive cache-first read when cache is ≤30 s old (skips live API call entirely — prevents burst-rate flooding), cache read when fetch returns any transient error, freshest-`fetched_at`-wins merge across all host subtrees, staleness indicator in display (`~` prefix on percentages, `(Nm ago)` age suffix), model override and touch state persistence as top-level `{name}.json` keys, zero tracked-credential-file writes on successful fetch (volatile data lands only in the per-host cache file — AC-16), one-time legacy `cache{}` migration (AC-18), self-cleaning migration of the TSK-500-era gitignored `-cache/{name}.json` (AC-17), persistence of `fetch_oauth_account`'s `org_created_at` identity field so non-live-fetch branches can compute a real `~Renews` value instead of `"?"` (see AC-15; fix for BUG-327).
- **Out of Scope**: Cache invalidation by time (stale data is always better than no data), display provenance (showing which host's cache served a row — the age suffix already conveys staleness), retention/pruning of host subtrees (a decommissioned host's entries lose every freshest-wins comparison naturally), cross-host refresh coordination (two hosts double-fetching the same unoccupied stale account between syncs is rare and harmless — each writes its own subtree), persistence of any `fetch_oauth_account` identity field other than `org_created_at` (e.g. `display_name`, `billing_type`, `capabilities` remain live-fetch-only and are never cached — only `org_created_at` has an established need, per AC-15).

### Design

When the usage API (`GET /api/oauth/usage`) returns an error for an account, the `.usage` table currently shows `—` for all quota columns. With this feature, the last successful fetch result is persisted and displayed as fallback when the live fetch fails.

**Storage targets (TSK-500 two-tier split, TSK-502 per-host tree):**

- **Volatile quota data** — changes on every fetch — lives flat in this host's tracked file `{credential_store}/cache/{host}_{user}/{name}.json`. The `{host}_{user}` slug is the same sanitized identity as the `_active_{host}_{user}` markers (single sanitization source: `host_user_slug()`), so exactly one host writes each subtree — churn is merge-trivial and cross-host conflicts are structurally impossible. No path component is hyphen-prefixed, so the tree is tracked and rides ordinary commits, giving every host visibility into every other host's last fetch.
- **Low-churn metadata** (`model_override`, `last_touch_at`, `touch_idle`, `org_created_at`) — must survive across hosts via git — lives as top-level keys of the tracked `{name}.json`, written via the established read-merge-write pattern.
- **Legacy layouts**: (pre-TSK-500) a `"cache"` top-level object in `{name}.json` holding both kinds — still fully readable as a fallback; dissolved by a one-time migration on the first `write_quota_cache` (see Algorithm step 0). (TSK-500 era) a gitignored host-local `-cache/{name}.json` — participates in the merge read as a candidate; deleted after the first successful per-host write (self-cleaning, see Algorithm step 1).

**Per-host cache structure** (`cache/{host}_{user}/{name}.json`, flat — follows [invariant/007](../invariant/007_json_storage_format.md)):

```json
{
  "fetched_at": "2026-06-07T07:52:00Z",
  "status": "ok",
  "five_hour": { "left_pct": 86.0, "resets_at": "2026-06-07T11:49:00Z" },
  "seven_day": { "left_pct": 16.0, "resets_at": "2026-06-07T16:00:00Z" },
  "seven_day_sonnet": { "left_pct": 0.0, "resets_at": "2026-06-07T16:00:00Z" },
  "history": [ { "t": 1749900000, "h5": [ 14.0, "2026-06-07T11:49:00Z" ], "d7": null, "sn": null } ]
}
```

**Tracked low-churn keys** (top level of `{name}.json`, alongside `host`, `model`, etc.):

```json
{
  "model_override": "opus",
  "last_touch_at": "2026-06-07T06:30:00Z",
  "touch_idle": true,
  "org_created_at": "2026-01-01T00:00:00Z"
}
```

**Merged read** (`read_quota_cache`): the volatile source is the freshest candidate by `fetched_at` across every host subtree `cache/*/{name}.json` plus the legacy gitignored `-cache/{name}.json` (`read_volatile_candidates()`), else the legacy tracked `cache{}`; when nothing exists the entry is `None` (the "no cache" contract is unchanged). A candidate with an unparseable `fetched_at` is skipped entirely — never selected, never aborting the merge. Each low-churn field reads the tracked top-level key first, falling back to the legacy `cache{}`.

**Algorithm:**

0. **Legacy migration (inside `write_quota_cache`, once per account)**: If the tracked `{name}.json` still has a `cache{}` object, relocate the four low-churn keys to top level (an existing top-level value wins over the legacy one), remove the `cache` key entirely, and write the tracked file — a single write. The removed object's `history` seeds the per-host file so no measurements are lost. Already-migrated accounts short-circuit with zero tracked writes.
1. **On successful fetch**: After `fetch_oauth_usage` returns `Ok(usage_data)`, serialize the quota fields flat into this host's `cache/{host}_{user}/{name}.json` (subtree created on demand). The `fetched_at` timestamp is set to `now()` UTC ISO-8601. The `status` field is set to `"ok"`. The history ring is carried from the freshest candidate anywhere — own subtree, another host's, or the legacy gitignored file — so ring continuity survives host handoffs. The tracked `{name}.json` is not written (AC-16). After a successful write, the legacy gitignored `-cache/{name}.json` is deleted (self-cleaning migration — deletion only follows a confirmed write, so a failed write never orphans the only copy).
2. **On fetch error (transient errors only — 429, timeout, network)**: Read the merged cache entry (`read_quota_cache` — freshest candidate across host subtrees and the legacy gitignored file, legacy tracked `cache{}` fallback). If `fetched_at` exists, compute `age_minutes = now - fetched_at`. Use cached quota values for display. Mark the row with a staleness indicator. **Auth errors (HTTP 401, HTTP 403) bypass cache fallback entirely** — they pass through as `Err` so `should_refresh()` can trigger a token refresh. Only transient errors fall back to cache; auth errors must remain `Err` so the refresh pipeline sees them. Fix for BUG-296.
3. **On model override**: After `apply_model_override` determines the target model, write top-level `model_override` to `{name}.json`.
4. **On touch completion**: After a successful touch subprocess, write top-level `last_touch_at` and `touch_idle = false` to `{name}.json`.
5. **On successful retry after token refresh**: After `apply_refresh()` performs a token refresh and the quota retry returns `Ok(retried)`, set `aq.cached = false` and `aq.cache_age_secs = None` on the in-memory `AccountQuota`, then call `write_quota_cache()` with the fresh data. This clears the `~` staleness indicators and updates the on-disk cache so the next run starts from fresh data.
6. **On successful live `fetch_oauth_account`**: persist its `org_created_at` field as a top-level `{name}.json` key via `write_cache_string_if_changed()` — a conditional write that skips when the stored value already matches, so steady-state fetches keep the zero-tracked-write property (AC-16). On any non-live-fetch branch (cache-first, G1-not-owned, `approximate_quota()`) — where no live account fetch occurs and `AccountQuota.account` stays `None` — read `org_created_at` back (top-level first, legacy `cache.org_created_at` fallback) and surface it through a new, independent `AccountQuota.org_created_at` field so `renews_label()`'s Estimate branch can still compute a real `~Renews` countdown. Accounts never live-fetched gracefully fall back to `None` (unchanged `"?"` display). Fix for BUG-327.

**Display with cached data:**

- Quota percentages are prefixed with `~` to indicate stale data: `~86%` instead of `86%`
- The `5h Reset` / `7d Reset` columns show the cached `resets_at` countdown (which may be in the past if stale — display `(stale)` when computed countdown is negative)
- The composite status emoji `●` is computed from cached values (same thresholds as live)
- A row-level age indicator shows time since last successful fetch: `(12m ago)` appended to the account name in the NAME cell (not an error-reason position — see AC-03)
- When the display originates from a cache-fallback conversion (a transient fetch error substituted with cached data — AC-02), the original failure reason is also preserved on the in-memory result and surfaced via `shorten_error()` in every render format (text table, TSV, JSON) — see AC-14. The text table combines it with the existing NAME-cell age suffix in one parenthetical; TSV has no pre-existing age-suffix mechanism, so it appends the shortened reason as its own standalone parenthetical instead. Live successes never carry a failure reason and render unchanged.

**Non-owned accounts (Feature 036 interaction):** When account ownership is enabled, non-owned accounts use the quota cache as their **primary** fetch source (G1 gate in Feature 036), not as a fallback. The cache read path, staleness display, and `~` prefix are identical to the error-fallback path — the distinction is only in how the cache-read was triggered. Under the per-host tree this path now serves real data for occupied-elsewhere accounts: the occupying host's fetches arrive via git sync, so the reader displays them (with the age suffix) instead of "(no cache)".

**Cross-host freshness bound (TSK-502):** cache visibility across hosts is bounded by commit/pull cadence — a reader sees another host's fetch only after that host's watchdog tick commits it and the reader's tree pulls it. Between syncs the reader serves its own freshest candidate, which may be minutes older than the writer's; the `(Nm ago)` age suffix makes that staleness visible. Clock skew between hosts shifts freshest-wins comparisons by at most the skew — bounded to display staleness only, since rotation decisions never consume another host's staggered cache directly (Feature 036's occupancy gates govern that).

**Graceful degradation:**

- If no host subtree, no legacy gitignored `-cache/{name}.json`, and no legacy tracked `cache{}` exists (first-ever fetch for this account): display dashes as before (no regression)
- If a candidate's `fetched_at` is unparseable: that candidate is skipped (when none parse anywhere, treat as no cache)
- Cache is best-effort — write failures are silently ignored (quota display is non-critical)

### Acceptance Criteria

- **AC-01**: On successful `fetch_oauth_usage`, this host's `cache/{host}_{user}/{name}.json` is written with `fetched_at`, `status`, and all quota fields — and the tracked `{name}.json` is not modified (AC-16).
- **AC-02**: On transient fetch error (429, timeout, network), if a cache entry exists (any host subtree `cache/*/{name}.json`, the legacy gitignored `-cache/{name}.json`, or legacy tracked `cache{}` pre-migration), quota columns display cached values with `~` prefix. HTTP 401 and HTTP 403 errors are excluded from cache fallback.
- **AC-03**: When cached data is displayed, an age indicator (`(Nm ago)` or `(Nh ago)`) is appended to the account name in the NAME cell (not an error-reason position — see AC-14 for the separate fallback-reason indicator).
- **AC-04**: When no cache exists (fresh account, never fetched), display remains `—` (no regression from current behavior).
- **AC-05**: The `model_override` field is written as a top-level `{name}.json` key after `apply_model_override` executes (low-churn — must survive across hosts via git).
- **AC-06**: The `last_touch_at` and `touch_idle` fields are written as top-level `{name}.json` keys after touch subprocess completion.
- **AC-07**: Low-churn writes (AC-05/AC-06) and the one-time migration (AC-18) use read-merge-write on `{name}.json` — existing fields (`host`, `model`, `oauthAccount`, `_renewal_at`) are preserved. Volatile writes (AC-01) never touch `{name}.json` at all.
- **AC-08**: Strategy recommendations (`sort::`) operate on cached quota values when live data is unavailable — recommendations remain functional.
- **AC-09**: `format::json` output includes a `"cached": true` flag and `"cache_age_secs": N` field when displaying cached data.
- **AC-10**: When cache fallback converts a fetch error to `Ok(cached_data)` (AC-02 path), accounts whose local token is expired (`expires_at_ms / 1000 <= now_secs`) are still flagged for token refresh by `should_refresh()` via the `cached + expired` guard — the `Ok` result does not suppress refresh when `cached = true` and the token is locally expired.
- **AC-11**: After `apply_refresh()` executes a successful token refresh and quota retry (`retry OK`), `aq.cached` is reset to `false` and `aq.cache_age_secs` is cleared to `None` on the in-memory `AccountQuota`, and the fresh data is written to `{name}.json` via `write_quota_cache()`. The row no longer shows `~` prefix or `(Xh ago)` label, and the next run reads fresh cache data.
- **AC-12**: HTTP 401 and HTTP 403 auth errors from `fetch_oauth_usage` bypass cache fallback — `fetch_all_quota` returns `Err` (not `Ok(cached_data)`) for these error types. The `Err` propagates to `should_refresh()`, which triggers a token refresh attempt. Auth errors must not be masked by cache. Fix for BUG-296.
- **AC-13**: When `fetch_quota_for_list()` checks an owned, non-solo, non-occupied-elsewhere account and finds a cache entry ≤30 seconds old, the live API call (`GET /api/oauth/usage`) is skipped entirely; the cached data is served directly (`cached: true`, `cache_age_secs: N`). This cache-first guard fires after the G1/G1b/solo gates and after `is_current` is resolved, but before the local token-expiry check. Prevents API burst flooding from rapid-succession `.usage` invocations (test suites, polling scripts). The 30 s window is a constant `CACHE_FRESH_SECS` in `fetch.rs`.
- **AC-14**: When cache fallback converts a fetch error to `Ok(cached_data)` (AC-02 path), the original failure reason is preserved on the in-memory account result (`fallback_reason: Option<String>` field, populated only on this arm) and surfaced via `shorten_error()` in every render format: the text table appends the shortened reason alongside the existing NAME-cell age suffix (AC-03) in one parenthetical; the TSV format has no pre-existing age-suffix mechanism, so it appends the shortened reason as its own standalone NAME-cell parenthetical instead; JSON output emits a `"fallback_reason":"<shortened_reason>"` field alongside `"cached"`/`"cache_age_secs"` (AC-09). Live successes (`cached=false`) never populate `fallback_reason` and render unchanged. Auth errors (401/403) never reach the cache-fallback arm (AC-12), so `fallback_reason` is never populated from an auth rejection. Fix for BUG-335.
- **AC-15**: When a live `fetch_oauth_account` call succeeds, its `org_created_at` field is persisted as a top-level `org_created_at` key in `{name}.json` via `write_cache_string_if_changed()` — a conditional write skipped when the stored value already matches, preserving AC-16 on steady-state fetches. On any non-live-fetch branch (cache-first AC-13, G1-not-owned, `approximate_quota()`) that would otherwise leave `AccountQuota.account` as `None`, the persisted `org_created_at` is read back (top-level first, legacy `cache.org_created_at` fallback) and surfaces through a new `AccountQuota.org_created_at: Option<String>` field — independent of `account: Option<OauthAccountData>`, which stays `None` on these branches. `renews_label()` (called with `aq.org_created_at.as_deref()` in place of the previous `aq.account.as_ref().map(|a| a.org_created_at.as_str())` in every render format) can then compute a real `~Renews` Estimate value instead of `"?"` for actively-subscribed accounts that have had at least one prior live account fetch. Accounts with no cache entry, or caches predating this field, gracefully fall back to `None` (unchanged `"?"` display — no regression). The existing `is_no_subscription()` guard (BUG-232 fix) is unaffected — it gates on `result.is_err()` and `account.billing_type`, independent of which field carries `org_created_at`. Fix for BUG-327.
- **AC-16**: A successful fetch-and-persist sequence (`write_quota_cache` + `write_history_entry`) performs zero writes to the tracked `{name}.json` once the account is migrated — the file is byte-identical before/after (verified by content hash, not merely git-status silence). Steady-state quota sweeps leave the credential store clean for git. TSK-500.
- **AC-17**: Volatile fields (`fetched_at`, `status`, `five_hour`, `seven_day`, `seven_day_sonnet`, `history`) live flat in this host's `{credential_store}/cache/{host}_{user}/{name}.json` — no path component is hyphen-prefixed, so the global `-*` gitignore rule cannot match it and the tree is tracked by construction. The `{host}_{user}` slug is single-sourced with the `_active_` marker sanitization (`host_user_slug()`); each subtree has exactly one writer. Reads merge freshest-`fetched_at`-wins across all subtrees; a TSK-500-era gitignored `-cache/{name}.json` participates as a candidate and is deleted after the first successful per-host write (self-cleaning — its values and history survive in the per-host file). The file follows [invariant/007](../invariant/007_json_storage_format.md) (2-space pretty JSON + trailing newline); the subtree is created on demand. TSK-502.
- **AC-18**: The first `write_quota_cache` against a legacy account (tracked `cache{}` present) migrates in a single tracked write: low-churn keys (`model_override`, `last_touch_at`, `touch_idle`, `org_created_at`) are relocated to top level (existing top-level values win), the `cache` key is removed entirely, and the legacy `history` seeds the per-host file. Before migration, the legacy `cache{}` remains fully readable through the merged read path; after migration, the tracked JSON contains no `cache` key. TSK-500.

### Bugs

| File | Relationship |
|------|--------------|
| BUG-255 🟢 Fixed | Cache fallback Err→Ok conversion defeats `should_refresh()` — fixed via `cached + expired` guard in `should_refresh()` |
| BUG-256 🟢 Fixed | `retry OK` does not clear `cached` metadata — `~` and `(Xh ago)` persist after successful refresh; fix = AC-11 |
| BUG-288 🟢 Fixed (Fix A) | Fix A complete: `apply_post_switch_touch` now calls `write_quota_cache` with post-subprocess quota data; subsequent `apply_touch` reads updated quota (`resets_at = Some`) and skips the redundant subprocess. Fix B (`touch_idle` read site in `apply_touch` as defense-in-depth for server-side propagation lag) deferred; `touch_idle=false` write (AC-06) remains dead code pending follow-on task. |
| BUG-296 🟢 Fixed (TSK-306) | Auth-error guard added: `fetch.rs:235` changes fallback arm to `Err( ref e ) if !e.contains("401") && !e.contains("403")` — auth errors propagate as `Err`; transient errors still fall back to cache; fix = AC-12 |
| BUG-304 🟢 Fixed (TSK-316) | Three independent cache-read paths reconstructed `OauthUsageData` for utilization; G1 (non-owned) applied no approximation, HTTP-error fallback and `approximate_quota()` each inlined 40–55 lines of duplicated approximation. Fixed: centralized `read_cached_quota()` function |
| BUG-327 🟢 Fixed (TSK-368) | `QuotaCacheEntry` (`claude_profile_core/src/account/quota_cache.rs`; account.rs:1506-1522 pre-split) had no `org_created_at` field — every non-live-fetch branch in `fetch.rs` hardcoded `account: None`, so `~Renews` showed `?` for 15/18 accounts. Fixed per AC-15: `org_created_at` now persisted to `cache.org_created_at` and surfaced via a new independent `AccountQuota.org_created_at` field on all 3 non-live branches. Two accounts still separately escape via the fully-decoupled `_renewal_at` manual override (unaffected by this fix). |
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
| `src/usage/api_switch.rs` | Side-effect metadata — `write_cache_string()` (model_override, AC-05) and `write_cache_bool()` (touch_idle, AC-06), both top-level tracked keys |
| `claude_profile_core/src/account/` | Storage layer — `QuotaCacheEntry`, merged `read_quota_cache()` (freshest-wins via `read_volatile_candidates()`), `write_quota_cache()` (per-host volatile + migrations + legacy self-clean), `write_cache_field()` (top-level tracked), `write_cache_string_if_changed()` (AC-15/AC-16), `migrate_legacy_cache()` (AC-18), `host_user_slug()`/`local_cache_path()`/`legacy_local_cache_path()` (AC-17) |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | Unified `{name}.json` field table — `cache` subtree owned by this feature |
