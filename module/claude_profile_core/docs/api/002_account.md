# API: Account

### Scope

- **Purpose**: Document the programmatic interface of the claude_profile_core `account` module.
- **Responsibility**: Specify the credential-store account domain contract at cluster level — one section per functional cluster, key signatures named, exhaustive per-item detail left to the source doc comments.
- **In Scope**: All `pub` items of `src/account.rs`, grouped into the clusters below.
- **Out of Scope**: Token expiry classification (→ `001_token.md`), CLI parameter handling and rendering (→ `claude_profile`), the subprocess execution engine behind refresh (→ `claude_runner_core`).

### Abstract

`claude_profile_core::account` is the Layer 1 domain engine behind `clp`'s account commands: named credential storage under a credential-store directory, atomic account switching into `~/.claude/.credentials.json`, token refresh orchestration, multi-machine ownership/claim/reservation markers, a per-account quota cache, and a measurement history log. All credential writes go through `claude_core::file_io::atomic_write_secret` (`0o600`), all non-secret store writes through `atomic_write`. This doc indexes the surface by cluster; the authoritative per-item contract is each item's doc comment in `src/account.rs` (`#![warn(missing_docs)]` is enforced).

### Clusters

#### Account CRUD and identity

`Account` (parsed store entry), `AccountBackend` (`Oauth`/`Redirect`), `list`, `save`, `delete`, `check_delete_preconditions`, `validate_name`, `validate_redirect_name`, `validate_name_for_save`, `credential_stem`, `lock_store` + `StoreLock` (blocking exclusive `flock` on `{store}/-store.lock`, RAII release; taken internally by `save`/`switch_account`/`delete` so their multi-file sequences never interleave across processes). A store entry is `{name}.credentials.json` plus sidecar metadata in `{name}.json`.

#### Switching and session overrides

`check_switch_preconditions`, `switch_account` (atomic copy of the named credentials into the live `~/.claude/.credentials.json`), `override_session_model_to_opus`, `override_session_model_to_sonnet`, `set_session_model`, `get_session_model`, `set_session_effort`, `get_session_effort`, `remove_session_effort`.

#### Token refresh

`refresh_account_token` (refreshes a named account's token by spawning an isolated Claude Code subprocess — `--print .` via `claude_runner_core::run_isolated` — whose startup OAuth refresh rewrites the credentials, then persists the rotation back into the store), `manipulate_expires_at`, `credentials_usable`, `read_access_token_from_file`.

#### Ownership, claim, and reservation (multi-machine)

`resolve_hostname`, `current_identity` (`{hostname}_{user}`), `read_owner`, `write_owner`, `is_owned`, `read_claim_lock`, `write_claim_lock`, `write_reserve`, `read_backend`, `active_marker_filename` (`_active_{hostname}_{user}`), `other_machines_active`.

#### Renewal override

`RenewalOperation`, `account_renewal`, `secs_to_iso8601`, `parse_from_now_delta` (Feature 030 renewal-date overrides).

#### JSON field helpers

`parse_string_field`, `parse_u64_field`, `parse_bool_field`, `parse_string_array_field`, `extract_object_block` — dependency-free needle parsers over credential/sidecar JSON, shared so callers never hand-roll field extraction.

#### Quota cache

`QuotaCacheEntry` (fetched_at, per-period utilization/reset pairs, `model_override`, `last_touch_at`, `touch_idle`, `org_created_at`), `write_quota_cache`, `read_quota_cache`, `write_cache_field`, `write_cache_string`, `write_cache_bool`, `write_cache_string_if_changed`, `parse_iso_utc_secs`. Two-tier layout (TSK-500/502): volatile fetch snapshots (`fetched_at`, periods) live in the tracked per-host tree `cache/{host}_{user}/{name}.json`, merged freshest-`fetched_at`-wins across every host subtree (plus self-cleaning legacy `-cache/` migration); low-churn metadata (`model_override`, `last_touch_at`, `touch_idle`, `org_created_at`) lives as top-level keys of the tracked store-root `{name}.json` via the `write_cache_*` helpers — the same file `read_quota_cache` consults, which is why Fix(BUG-488) routed the touch-flag writes through them.

#### Measurement history

`HistoryEntry`, `read_history`, `write_history_entry` (Feature 040 quota measurement history).

### Error Handling

Fallible operations return `std::io::Error`; validation failures use `InvalidInput` with a message naming the offending value. Read helpers over optional sidecar state (`read_owner`, `read_backend`, `read_quota_cache`, …) return defaults/`Option` rather than erroring on absent files.

### Compatibility Guarantees

- Credential-bearing writes are atomic and `0o600` (`atomic_write_secret`); a crash mid-switch leaves either the old or the new credentials, never a torn file.
- `parse_expires_at`-style field parsers never panic on malformed JSON — absent/malformed fields read as `None`/defaults.
- Quota-cache keys are top-level in the tracked `{name}.json`; readers tolerate entries written before a field existed (each field is `Option`).

### Sources

| File | Relationship |
|------|--------------|
| `../../src/account.rs` | All clusters above |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/account_test.rs` | Deletion, snapshot cleanup, quota-cache storage (TSK-500 two-tier) |
| `../../tests/account_refresh_test.rs` | `refresh_account_token` failure paths |
