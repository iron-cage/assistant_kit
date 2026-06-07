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
