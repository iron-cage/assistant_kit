# Changelog

All notable changes to claude_profile will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Per-account credential store consolidated from 5 files to 2**
  - Previous satellite files `{name}.claude.json`, `{name}.settings.json`, `{name}.roles.json`,
    and `{name}.profile.json` merged into a single `{name}.json`
  - `{name}.json` is now the unified per-account metadata file: OAuth identity (`oauthAccount`),
    model preference, org identity (endpoint 005), and host/role labels — all co-located in one
    document; read-merged on every `save()` to preserve existing keys across callers
  - `save()` signature extended with `host: Option<&str>` and `role: Option<&str>` trailing
    params; CLI commands pass explicit values; background token refresh passes `None, None`
    (no metadata capture in the background path)
  - Legacy satellite files removed best-effort by `save()` on every invocation — no migration
    script needed; files are cleaned up on the next `clp .account.save`
  - Test assertions expecting "file must not exist" converted to content-based checks;
    CLI-triggered `save()` always writes `{name}.json`

### Added

- **Account ownership and `.account.unclaim` command** (FR-36)
  - `owner` field in `{name}.json` — auto-stamped by `.account.save` via `current_identity()`
    (`$USER@<hostname>` using the same `resolve_hostname()` fallback chain as active markers)
  - Eight enforcement gates (G1–G8): non-owner machines cannot switch (G5), delete (G6),
    relogin (G7), or unclaim (G8) the account; fetch (G1), refresh (G2/G3), and touch (G4)
    are silently skipped — non-owned accounts use quota cache as primary source
  - `.account.unclaim name::EMAIL` — dedicated command; calls `write_owner(name, store, "")`
    directly; pure metadata operation (no credential touch, no active marker change)
  - G8 ownership gate evaluates BEFORE `dry::1` check — non-owner gets exit 1 even in dry-run
  - `is_owned` field propagated to `format::json` output on `.usage`
  - Backward compatible: accounts without `owner` field behave identically to pre-feature
  - `.account.assign` is ownership-neutral — marker-only write, does NOT call `write_owner()`
  - 15 new tests (ft02, ft15–ft17, it01–it11) covering unclaim, G8 gate, dry-run, idempotency,
    credential isolation, read-merge preservation, and unknown parameter rejection

- **`.model` — dedicated model get/set command** (FR-35)
  - `clp .model` (get): reads `~/.claude/settings.json` `model` field; prints shorthand or `(unset)`
  - `clp .model set::VALUE` (set): writes model to settings via `set_session_model()`;
    accepts `opus`, `sonnet`, `haiku`, `default` (removes key)
  - `format::json` returns `{"model": "opus"}` or `{"model": null}`
  - Shared `map_model_shorthand()` inner function — no duplication with Feature 034 mapping table
  - `get_session_model()` helper added to `claude_profile_core`

- **`set_model::` — explicit session model override** (FR-34)
  - New parameter on `.account.use` and `.usage`; writes `~/.claude/settings.json` model key
  - Precedence: explicit `set_model::` always wins over automatic `apply_model_override()` by ordering
  - `default` removes the `model` key (reverts Claude Code to built-in default)
  - Trace line emitted on `.account.use` when `trace::1`

- **Quota cache fallback** (FR-33, TSK-256)
  - Persists last-known quota data in `{name}.json` under `"cache"` key after every successful fetch
  - On API error (429, timeout, network): reads cached values and displays with `~` prefix and
    `(Nm ago)` age indicator instead of dashes
  - Also persists `model_override` and `last_touch_at` / `touch_idle` state
  - After successful token refresh retry, cached indicators are cleared and fresh data written
  - Non-owned accounts (Feature 036) use cache as primary source (G1 gate), not as fallback

- **`.account.assign` — write per-machine active-account marker** (FR-32, TSK-251)
  - New command (Command 16); writes only `_active_{machine}_{user}` for any `USER@MACHINE` pair
  - No credential rotation; `~/.claude/.credentials.json` and `~/.claude.json` are never touched
  - `name::` absent → emits live usage block (current machine identity, active account, copy-paste examples); exit 0
  - `for::USER@MACHINE` targets a remote machine; both components sanitized (alphanumeric, `-`, `.` kept; others → `_`)
  - Exit 2 when account not found; exit 1 when `for::` format invalid (no `@`, or empty component)
  - 12 integration tests (aa01–aa12) covering marker write, remote target, dry-run, usage block, errors, sanitization, prefix resolution

- **`.account.inspect` — live identity, subscription, and org details** (TSK-246)
  - New command; calls Anthropic identity endpoints using the named (or active) account's token
  - Shows: taggedId, UUID, billing type, max flag, org name/UUID, workspace UUID/name, capabilities
  - `format::json` returns structured JSON with all fields
  - `refresh::` (default 1) attempts OAuth refresh when stored credentials are locally expired

- **`.account.renewal` — set or clear billing renewal timestamp override** (TSK-248)
  - New command; writes a `renewal_override` field into `{name}.json` for one account, all, or a comma-separated list
  - `at::` — exact ISO-8601 UTC timestamp; `from_now::` — relative offset (e.g. `+1h30m`, `-30m`); `clear::1` — remove override
  - Affects `.usage` renews column: overrides display with `~`-prefixed estimate when no server data available

- **`.account.relogin` — force browser re-authentication** (TSK-249)
  - New command; launches `claude auth login` in browser mode for a named account with a dead `refreshToken`
  - Saves the freshly captured credentials under the account name; supports `dry::` preview
  - Required when OAuth refresh fails and the stored `refreshToken` is no longer accepted

- **`.usage` redesigned — live quota from Anthropic API** (FR-14, task 127)
  - Replaced `stats-cache.json` file reading with live `claude_quota::fetch_rate_limits()` calls
  - 8-column table: flag (`✓`/`→`/ ), Account, Expires, 5h Left, 5h Reset, 7d Left, 7d Reset, Status
  - Active account marked `✓`; recommended next account (highest remaining 5h session quota) marked `→`
  - Footer line: `Valid: X / Y   →  Next: name  (N% session left, token expires in Xh Ym)` when ≥2 valid
  - `format::json` uses `session_5h_left_pct` / `weekly_7d_left_pct` naming (remaining, not consumed)
  - `serde_json` dep removed; `claude_quota` and `data_fmt` added under `enabled` feature

- **`.credentials.status` — live credential metadata** (FR-17)
  - New command; reads `~/.claude/.credentials.json` without requiring account store setup
  - Default-on fields: `account::`, `sub::`, `tier::`, `token::`, `expires::`, `email::`
  - Opt-in fields: `file::`, `saved::`, `display_name::`, `role::`, `billing::`, `model::`
  - `format::json` always includes all fields regardless of field-presence params
  - Exit 2 when credential file absent or HOME unset

- **`.account.limits` — single-account rate-limit utilization** (FR-18)
  - New command; fetches live quota for the active or named account via `claude_quota`
  - Shows session (5h) and weekly (7d) usage with consumed percentage and reset times
  - Optional `name::` to query a non-active account by email; exit 2 for credential/network errors

- **Rich OAuth metadata fields on `.credentials.status` and `.accounts`** (FR-20)
  - `display_name::`, `role::`, `billing::` read from `~/.claude.json` `oauthAccount`
  - `model::` reads from `~/.claude/settings.json`
  - All four opt-in (default off); show `N/A` when source file absent or field missing

### Fixed

- **Account switching and credential integrity** (BUG-174, 209, 211–213, 217, 219, 254, 277, 282, 285)
  - `switch_account()` wholesale overwrite of `~/.claude.json` replaced with surgical JSON-merge
    preserving global config keys (BUG-174); stale `emailAddress` and org fields no longer
    written from snapshot (BUG-217, 219, 254); `~/.claude.json` now restored on switch (BUG-277)
  - `.account.save` active-marker corruption fixed: live email from `~/.claude/.credentials.json`
    used instead of stale `_active` marker (BUG-209, 212); `save()` now writes `_active` marker (BUG-282)
  - `apply_refresh`/`apply_touch` restore no longer unconditionally overwrites `_active` — TOCTOU
    race with concurrent `.account.use` eliminated (BUG-211, 208)
  - `.account.use` now checks `expiresAt` before switch on fetch error (BUG-213); idle check
    replaced stale server-side `resets_at` proxy with explicit subprocess tracking (BUG-285)

- **Token refresh lifecycle** (BUG-155, 156, 162, 165, 166, 169–171, 175, 205, 221, 230, 235, 271)
  - `refresh::` default corrected from 0 to 1 (BUG-155); 429+locally-expired accounts now
    refreshed (BUG-156, 235); `refresh_account_token()` subprocess writes `expiresAt` (BUG-162)
  - `apply_refresh` now performs full account lifecycle update after token refresh (BUG-165);
    `fetch_oauth_account` retried post-refresh to recover `~Renews` column (BUG-171)
  - `refresh_account_token()` `Some(paths)` branch no longer clobbers live session credentials
    (BUG-221); `switch_account` before `run_isolated` removed (BUG-175)
  - Opaque `sk-ant-oat01-*` tokens: `jwt_exp_ms` returns None handled gracefully (BUG-170);
    HTTP 429 removed from unconditional refresh retry guard (BUG-271)
  - `.account.use` now attempts OAuth refresh before exit 3 on expired token (BUG-230);
    `trace::` param and `read credentials: OK` trace step added (BUG-166, 205)

- **Session model override and restoration** (BUG-222, 225, 226, 238, 244, 286, 290)
  - `switch_account()` now restores per-account `model` preference from snapshot (BUG-222);
    quota-aware Sonnet→Opus upgrade applied post-restore (BUG-225)
  - Model override no longer skipped when account is already active (BUG-238); `.usage` now
    calls `apply_model_override` (BUG-244); full model ID normalized to shorthand (BUG-286)
  - `resolve_model(Auto)` gate simplified — cold accounts no longer require two-touch warm-up (BUG-290)

- **Touch / subprocess lifecycle** (BUG-176–179, 181, 202, 207, 210, 214, 215, 246, 288, 289)
  - Touch trigger inverted to fire on idle accounts, not active sessions (BUG-181); h-exhausted
    (BUG-178) and 7d-exhausted (BUG-214) accounts now skip-guarded
  - Idle detection expanded to include 7d and 7d-Sonnet timers (BUG-215); post-touch quota
    re-fetch ungated from `credentials=None` early return (BUG-179)
  - `only_active::1` now gates touch loop — non-active accounts no longer subprocess-spawned (BUG-246)
  - Post-switch touch result no longer discarded; redundant `.usage touch` subprocess eliminated
    (BUG-288); `son_running=false` infinite Haiku loop fixed (BUG-289)
  - Trace diagnostics: separator added (BUG-176), skip-trace emitted (BUG-177, 202),
    `trace::` param added to `.account.use` (BUG-207), model/effort trace in skip path (BUG-210)

- **Sort strategies and next-account recommendation** (BUG-173, 206, 223–224, 227–229, 240–243, 287, 291–292)
  - `renew` strategy criterion corrected to `min(7d_reset, sub_renewal)` (BUG-229); 5h tiebreaker
    added (BUG-243); `sort::renew` / `~Renews` naming collision resolved (BUG-223)
  - `sort::expires` and `sort::renews` options added (BUG-224); `sort::renew` / `next::renew`
    asymmetry eliminated (BUG-228); `next::renew` unified to `sort_indices` (BUG-291, 292)
  - All `next::` strategies now skip h-exhausted (BUG-240) and occupied-elsewhere (BUG-241) accounts
  - Endurance: weekly tiebreaker added (BUG-173); weekly-floor gate added (BUG-287);
    footer metric corrected (BUG-242); drain: weekly-exhausted filter (BUG-206)
  - `→ Next` column now includes token expiry as a candidate event (BUG-227)

- **Subscription and billing display** (BUG-152, 216, 220, 231–234, 236–237)
  - Expired subscriptions now show "no subscription" instead of "rate limited (429)" (BUG-231, 233);
    `~Renews` suppressed for cancelled subscriptions (BUG-232)
  - `billing_type == "none"` override no longer fires when usage API returns OK (BUG-236);
    `[trace] result:` emitted after billing_type override (BUG-234)
  - `parse_oauth_account` now scans all memberships, not just `[0]` (BUG-237);
    drain footer label corrected for Sonnet-binding scenarios (BUG-216)
  - HTTP 401 shortening added to `shorten_error` (BUG-152); `~Renews` no longer
    overwritten by error reason for 429 accounts (BUG-220)

- **Quota cache and fetch pipeline** (BUG-218, 245, 255–256)
  - Cache fallback no longer converts Err to Ok — `should_refresh()` preserved (BUG-255);
    `retry OK` now clears `cached` metadata (BUG-256)
  - `only_active::1` now gates `fetch_all_quota` — single-account query no longer fetches all N (BUG-245)
  - `fetch_all_quota()` synthetic row collision check added (BUG-218)

- **CLI parameter handling and routing** (BUG-261–267, 272–274, 278, 294)
  - `fmt::` alias expanded to `format::` (BUG-261); bare positional account name accepted (BUG-262);
    exact local-part match no longer reported as ambiguous (BUG-264)
  - Dry-run existence checks added to `.account.use` (BUG-265) and `.account.delete` (BUG-266);
    `.account.renewal` comma-list names now prefix-resolved (BUG-267)
  - Boolean params reject out-of-range integers (BUG-272); `.help` token pre-scanned in second
    position (BUG-273); verbosity range validated per-command, not globally (BUG-274)
  - Path-unsafe characters (`/`, `\`) in account local-part rejected at validation (BUG-278);
    positional name rewrite scans `argv[1..]` for first bare token (BUG-294)

- **Infrastructure and environment** (BUG-172, 263, 268, 270, 275, 280–281, 283–284)
  - 30-second read timeout added to all `ureq::get()` call sites (BUG-172); stale
    `anthropic-beta: oauth-2023-09-22` header removed (BUG-283)
  - `$PRO` validation uses `is_dir()` instead of `exists()` (BUG-263); `~/.claude.json` read
    from correct path (BUG-270); `$HOME` unset handling improved (BUG-268, 280)
  - Active-account deletion no longer blocked by stale `_active` marker (BUG-275);
    test runner `$PRO` isolation added (BUG-281); inferred-name save path tested (BUG-284)

- **Help and documentation** (BUG-203, 204, 279)
  - Per-command `.help` now shows parameter descriptions (BUG-203); required params shown
    as required, not optional (BUG-204); refresh trigger list corrected (BUG-279)

### Changed

- **`.account.switch` renamed to `.account.use`**
  - Aligns with the ubiquitous-language term "use" (the account you switch *to* is the one in use)
  - CLI alias `.account.switch` removed; callers must use `.account.use`

### Added

- **`PersistPaths` — persistent user storage path resolution** (FR-15)
  - New `persist.rs` module exposing `PersistPaths` struct (zero new dependencies)
  - Resolves `$PRO/persistent/claude_profile/` from env vars (stdlib only),
    falling back to `$HOME/persistent/claude_profile/` when `$PRO` is unset
  - `PersistPaths::new()` — tries `$PRO` (if set and is an existing directory), then
    `$HOME`/`$USERPROFILE`; returns `Err(NotFound)` if none resolve
  - `PersistPaths::base()` — returns the resolved `PathBuf`
  - `PersistPaths::ensure_exists()` — creates base directory via `create_dir_all` on first use
  - **Bug fix (issue-001):** `$PRO` pointing to a file (not a directory) now correctly falls
    back to `$HOME` instead of producing an unusable `<file>/persistent/claude_profile` path;
    fixed by using `path.is_dir()` instead of `path.exists()`
  - 15 integration tests in `persist_test.rs`: p01–p15 covering PRO-set, HOME fallback,
    USERPROFILE fallback, both-unset error, HOME-over-USERPROFILE priority, path shape
    (PRO and HOME roots), ensure_exists idempotency, actionable error message, empty-string
    PRO, file-as-PRO fallback, and Debug formatting

- **`.usage` command — 7-day token usage from `stats-cache.json`** (FR-14)
  - Reads `~/.claude/stats-cache.json` and reports per-model token totals for the
    7-day window ending at `lastComputedDate`
  - `v::0`: compact single line (`17.3M total · sonnet-4-6: 12.2M · …`)
  - `v::1` (default): labelled table with comma-formatted counts and percentages
  - `v::2`: table + per-day breakdown newest-first
  - `format::json`: machine-readable JSON with `period_start`, `period_end`,
    `total_tokens`, `by_model[]`
  - `serde_json` added as optional dependency under `enabled` feature
  - 24 integration tests covering error paths, date boundaries (month/year/leap year),
    token formatting, window filtering, multi-day aggregation, and JSON validity
  - Tests: `usage_test.rs::u01` — `u24`

### Fixed

- **`fmt_tokens_compact`: rounding boundary at K→M tier**
  - `{:.1}` formatting caused `999_999` to display as `"1000.0K"` instead of `"1.0M"`
  - Root cause: boundary was `n < 1_000_000` but `{:.1}` rounds 999.999 up to 1000.0
  - Fix: promote to M at `n >= 999_950` (where `999_950 / 1000 = 999.95` rounds up)
  - Test: `usage_test.rs::u18_usage_token_format_boundaries`

- **`load_usage`: missing `lastComputedDate` silently produced empty output**
  - Root cause: `json["lastComputedDate"].as_str().unwrap_or("unknown")` caused all
    ISO dates to be filtered out — ISO dates sort before "unknown" lexicographically
  - Fix: changed to `ok_or_else(|| ErrorData)` so missing field returns explicit error
  - Test: `usage_test.rs::u04_usage_missing_last_computed_date_exits_2`

### Changed

- **Crate renamed `claude_session` → `claude_profile`** (task 041)
  - Previous name `claude_session` was a misnomer — the crate manages credential
    profiles, not session files (those live in `claude_storage_core`)
  - All import paths, binary names, and documentation updated (79 files)

- **Account management consolidated here** (task 038)
  - `claude_profile` is now the single owner of all account CRUD logic
  - `claude_version` account commands delegate to this library
  - Eliminates duplicate account handling that previously existed in both crates

### Added

- **`clp` binary alias** — short name alongside `claude_profile`, consistent with workspace pattern (`cm`, `clr`)

- **`auto_rotate()` — one-call best-account rotation** (task 042, FR-13)
  - Selects the best inactive account based on token expiry ordering
  - Atomically switches the active account in a single call
  - Tests: `account_tests.rs::auto_rotate_*`

### Added

- **`.account.status name::` — query any account by name** (FR-16, TSK-065)
  - Optional `name::` parameter; omitting falls back to the active-account path
  - Token state computed from the named account's own stored `expiresAt` (never the live
    credentials file) — prevents active account's token state leaking to non-active accounts
  - Email and org shown from `~/.claude.json` for the active account only; `N/A`
    for non-active accounts
  - Returns `NotFound` (exit 2) for unknown names; exit 1 for invalid characters in name
  - 10 integration tests: `account_status_name_test.rs::astname01–10`

- **`.account.status v::1` — Sub, Tier, Email, Org fields** (spec FR-16 line 283, TSK-067)
  - `Sub:` (subscriptionType) and `Tier:` (rateLimitTier) now shown at `v::1` for all
    accounts: active path reads from live `.credentials.json`; named path uses account struct
  - `Email:` and `Org:` now also shown at `v::1` for the active-account path (were missing
    despite being shown on the named-account path)
  - Output field order at `v::1`: Account, Token, Sub, Tier, Email, Org
  - Output field order at `v::2`: Account, Token, Sub, Tier, Expires, Email, Org
  - 3 new integration tests: `astat11`, `astname11`, `astname12`

### Documentation

- **spec.md updated to v0.5** (task 040 / task 042)
  - FR-13 (`auto_rotate`) marked ✅ implemented
  - 9-command CLI inventory, `ClaudePaths` authority documented
  - `docs/cli/testing/command/` testing docs added for all commands
