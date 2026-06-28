# Test: Feature 017 — Expired Token Refresh via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/017_token_refresh.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Command IT |
|----|-----------|-----|------------|
| FT-01 | `refresh::0` — no `refresh_account_token` calls; errors shown as rows | AC-18 | it19, it20 |
| FT-02 | HTTP 401 triggers refresh attempt | AC-19 | — |
| FT-03 | HTTP 403 triggers refresh attempt | AC-19 | — |
| FT-04 | 429 + non-expired local token — NOT retried | AC-19 | — |
| FT-05 | 429 + expired local token — refresh triggered | AC-19 | — |
| FT-06 | Live session updated first; `account::save()` propagates before retry fetch | AC-20 | it32 |
| FT-07 | Refresh failure shown in row; other accounts still rendered | AC-21 | — |
| FT-08 | `format::json` output structure unchanged by refresh | AC-22 | — |
| FT-09 | `refresh::` in `.usage --help` with default `1` | AC-23 | it37 |
| FT-10 | Help text documents conditional 429 case | AC-24 | it33, it38 |
| FT-11 | `expires_at_ms` derived from JWT `exp` after refresh | AC-25 | test_jwt_exp_ms_mre_bug162 |
| FT-12 | `Some(paths)` — credential absent in store → `refresh_account_token` returns `None` → account skipped | Algorithm | test_apply_refresh_lifecycle_switch_fails_result_unchanged |
| FT-13 | `apply_refresh` does not call `switch_account`; `_active` marker unchanged throughout cycle | Algorithm | test_apply_refresh_lifecycle_active_marker_unchanged |
| FT-14 | `None`-paths fallback — credential absent in store → `refresh_account_token` returns `None` | Algorithm | test_apply_refresh_401_no_cred_file |
| FT-15 | `trace::1` propagated to `refresh_account_token`; lifecycle steps logged to stderr; no panic | AC-26 | test_apply_refresh_lifecycle_l010_trace_run_isolated_invoked_no_panic, art_some_paths_run_isolated_invoked_trace_no_panic |
| FT-16 | `expires_at_ms` from `expiresAt` field when JWT decode returns `None` (opaque token) | AC-25 | test_parse_u064_from_str_mre_bug170_extracts_expires_at, test_jwt_exp_ms_mre_bug170_opaque_returns_none |
| FT-17 | No `switch_account` in `apply_refresh`; `_active` unchanged confirms no restore occurred | AC-28 | test_apply_refresh_mre_bug208_restore_trace_emitted |
| FT-18 | After refresh re-fetch succeeds, `aq.account` re-populated via `fetch_oauth_account()` | AC-27 | mre_bug_171_account_populated_after_refresh |
| FT-13+ | `apply_refresh` does not write `~/.claude/.credentials.json`; file unchanged after cycle | AC-29 | (structural — FT-06/AC-20 mechanism + FT-13/FT-17 verification) |
| FT-19 | `refresh_account_token` returns `None` (RT expired) → `aq.result = Err("refresh token expired")` before `continue;` | AC-30 | — |
| FT-20 | `should_refresh()` returns `false` for owned account with `is_occupied_elsewhere == true` | AC-31 | — |
| FT-21 | `apply_refresh` trace emits `reason: cached-expired` (not `reason: ok`) for owned+cached+expired account | Algorithm | — |
| FT-22 | `refresh_account_token` sets `expiresAt=1` before `run_isolated` — RT rotates on every call | AC-32 | — |
| FT-23 | Current account: live creds differ from stored -> sync live->store, no subprocess spawned | AC-33 | — |
| FT-24 | Current account: `run_isolated` returns `None` -> race recovery re-reads live creds | AC-33 | — |
| FT-25 | `claude_profile/src/` contains zero direct `run_isolated` calls (invariant 008 grep test) | AC-34 | — |
| FT-26 | Current account becomes non-current during `run_isolated` window — re-read of active marker prevents stale `is_active` from writing wrong-account credentials to store (BUG-316 MRE) | AC-33 | — |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `refresh::0` produces no retry — auth error shown in row | AC-18 | Disable Refresh |
| FT-02 | HTTP 401 triggers `refresh_account_token` + retry | AC-19 | Auth Error Trigger |
| FT-03 | HTTP 403 triggers `refresh_account_token` + retry | AC-19 | Auth Error Trigger |
| FT-04 | 429 + non-expired local token passes through unchanged | AC-19 | Conditional 429 |
| FT-05 | 429 + expired local token triggers `refresh_account_token` | AC-19 | Conditional 429 |
| FT-06 | Live session updated first; `account::save()` propagates to store + active marker | AC-20 | Write-back |
| FT-07 | Refresh failure in row; remaining accounts processed | AC-21 | Non-aborting |
| FT-08 | `format::json` structure contains refreshed data unchanged | AC-22 | Format Interaction |
| FT-09 | `refresh::` appears in `.usage --help` with default `1` | AC-23 | Help Output |
| FT-10 | Help documents conditional 429 case (not unconditionally excluded) | AC-24 | Help Output |
| FT-11 | Post-refresh `expires_at_ms` from JWT `exp`; not from file `expiresAt` | AC-25 | JWT Expiry |
| FT-12 | `Some(paths)` — credential absent in store skips account without corrupting result | Algorithm | Lifecycle Skip |
| FT-13 | `apply_refresh` does not call `switch_account`; `_active` marker unchanged throughout cycle | Algorithm | Restore Absent |
| FT-14 | `None`-paths — credential absent in store skips account without corrupting result | Algorithm | None-paths Skip |
| FT-15 | `trace::1` propagates to `refresh_account_token`; lifecycle steps logged; no panic | AC-26 | Trace Propagation |
| FT-16 | Post-refresh `expires_at_ms` from `expiresAt` field for opaque `sk-ant-oat01-*` token | AC-25 | JWT Expiry (Opaque) |
| FT-17 | No `switch_account` in `apply_refresh`; `_active` unchanged confirms no restore occurred | AC-28 | Restore Absent |
| FT-18 | After refresh, `aq.account` re-populated via `fetch_oauth_account(new_token)` | AC-27 | BUG-171 MRE |
| FT-19 | `refresh_account_token` returns `None` → `aq.result = Err("refresh token expired")` (BUG-297 MRE) | AC-30 | BUG-297 MRE |
| FT-20 | `should_refresh()` returns `false` for owned account with `is_occupied_elsewhere == true` (BUG-303 MRE) | AC-31 | G2 Occupancy Guard |
| FT-21 | `apply_refresh` trace emits `reason: cached-expired` (not `reason: ok`) for owned+cached+expired account (BUG-298 MRE) | Algorithm | BUG-298 MRE |
| FT-22 | `refresh_account_token` sets `expiresAt=1` before `run_isolated` — RT rotates | AC-32 | RT Rotation |
| FT-23 | Current account: live creds differ from stored -> sync live->store, no subprocess | AC-33 | Live Sync |
| FT-24 | Current account: `run_isolated` returns `None` -> race recovery reads live creds | AC-33 | Race Recovery |
| FT-25 | `claude_profile/src/` contains zero direct `run_isolated` calls (invariant 008) | AC-34 | Grep Invariant |
| FT-26 | Current account becomes non-current during `run_isolated` window — stale `is_active` guard MUST NOT write wrong-account credentials to A's store slot (BUG-316 MRE) | AC-33 | TOCTOU Race Guard |

**Total:** 26 FT cases

---

### FT-01: `refresh::0` produces no retry — auth error shown in row

- **Given:** One saved account whose credential is expired; the usage API returns HTTP 401 for that account.
- **When:** `clp .usage refresh::0`
- **Then:** The account's row shows the auth error (e.g., `auth expired (401)`); no retry is attempted; exit 0.
- **Exit:** 0
- **Source fn:** `it019_refresh_disabled_param_accepted`
- **Source:** [017_token_refresh.md AC-18](../../../docs/feature/017_token_refresh.md)

---

### FT-02: HTTP 401 triggers `refresh_account_token` + retry

- **Given:** One saved account whose credential is expired; the usage API returns HTTP 401 for that account.
- **When:** `clp .usage refresh::1`
- **Then:** A `refresh_account_token` call is made for that account; if updated credentials are returned, the quota fetch is retried and the row shows live data; exit 0.
- **Exit:** 0
- **Source fn:** `it032_lim_it_refresh_per_account` [live — requires credentials]
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-03: HTTP 403 triggers `refresh_account_token` + retry

- **Given:** One saved account whose credential returns HTTP 403 from the usage API.
- **When:** `clp .usage refresh::1`
- **Then:** A `refresh_account_token` call is made for that account (403 is treated identically to 401); exit 0.
- **Exit:** 0
- **Source fn:** `test_apply_refresh_lifecycle_ft3_403_no_cred_result_unchanged`
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-04: 429 + non-expired local token passes through unchanged

- **Given:** One saved account with a valid (non-expired) `expiresAt` in its per-account credential file; the usage API returns HTTP 429 for that account.
- **When:** `clp .usage refresh::1`
- **Then:** The account's row shows `rate limited (429)`; no refresh is attempted (local token is valid — the 429 is a genuine rate limit); exit 0.
- **Exit:** 0
- **Source fn:** `test_apply_refresh_ft4_429_valid_token_not_retried`
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-05: 429 + expired local token triggers `refresh_account_token`

- **Given:** One saved account with an expired `expiresAt` in its per-account credential file (`expiresAt / 1000 <= now`); the usage API returns HTTP 429.
- **When:** `clp .usage refresh::1`
- **Then:** `refresh_account_token` is called for that account; the 429 is treated as a stale-credential condition; exit 0.
- **Exit:** 0
- **Source fn:** `test_apply_refresh_ft5_429_expired_refresh_path_entered_no_cred`
- **Source:** [017_token_refresh.md AC-19](../../../docs/feature/017_token_refresh.md)

---

### FT-06: Live session updated first; `account::save()` propagates to store and active marker

- **Given:** One saved account whose credential is expired; `account::refresh_account_token()` returns `Some(new_creds)` (updated credentials from subprocess).
- **When:** `clp .usage refresh::1`
- **Then:** The live session file (`~/.claude/.credentials.json`) is overwritten with `new_json` first; then `account::save()` propagates to `{credential_store}/{name}.credentials.json`, the active marker (`_active_{hostname}_{user}`), and companion files; all writes complete before the retry `fetch_oauth_usage` call; subsequent reads of the per-account credential file yield the refreshed token.
- **Exit:** 0
- **Source fn:** `it032_lim_it_refresh_per_account` [live — requires credentials]
- **Note:** BUG-165 fix; before the fix, only the persistent store was updated, leaving the live session stale.
- **Source:** [017_token_refresh.md AC-20](../../../docs/feature/017_token_refresh.md)

---

### FT-07: Refresh failure shown in row; remaining accounts processed

- **Given:** Two saved accounts: one whose refresh fails (e.g., `refresh_account_token` returns `None`), one whose fetch succeeds normally.
- **When:** `clp .usage refresh::1`
- **Then:** The failing account's row shows the final error reason; the succeeding account's row shows normal quota data; both rows are present; the table is rendered; exit 0.
- **Exit:** 0
- **Source fn:** `test_apply_refresh_mixed_accounts` (C4 — covers multi-account isolation)
- **Source:** [017_token_refresh.md AC-21](../../../docs/feature/017_token_refresh.md)

---

### FT-08: `format::json` structure unchanged by refresh

- **Given:** One saved account whose token is refreshed; one whose token is valid without refresh.
- **When:** `clp .usage format::json refresh::1`
- **Then:** JSON output is a valid array; refreshed accounts appear as normal data objects with quota fields; failed-refresh accounts appear as error objects; output structure matches the baseline `.usage format::json` schema; exit 0.
- **Exit:** 0
- **Source fn:** `test_render_json_ft8_mixed_ok_and_err_both_present`
- **Source:** [017_token_refresh.md AC-22](../../../docs/feature/017_token_refresh.md)

---

### FT-09: `refresh::` appears in `.usage --help` with default `1`

- **Given:** clean environment.
- **When:** `clp .usage --help`
- **Then:** stdout or stderr contains `refresh::` with a default value of `1`; the parameter is documented in the help output.
- **Exit:** 0
- **Source fn:** `it037_mre_bug155_refresh_defaults_to_1`
- **Source:** [017_token_refresh.md AC-23](../../../docs/feature/017_token_refresh.md)

---

### FT-10: Help documents conditional 429 case (not unconditionally excluded)

- **Given:** clean environment.
- **When:** `clp .usage --help`
- **Then:** The `refresh::` help text references the conditional 429 case (e.g., `429 when token is locally expired` or similar); it does NOT describe 429 as unconditionally excluded from refresh.
- **Exit:** 0
- **Source fn:** `it038_mre_bug156_refresh_help_mentions_429_expired`
- **Source:** [017_token_refresh.md AC-24](../../../docs/feature/017_token_refresh.md)

---

### FT-11: Post-refresh `expires_at_ms` derived from JWT `exp`; not from file `expiresAt`

- **Given:** One saved account whose credential is expired; `account::refresh_account_token()` returns `Some(new_creds)` containing a new `accessToken` with a future JWT `exp` claim; the credential file's `expiresAt` field is NOT updated by the subprocess.
- **When:** `clp .usage refresh::1`
- **Then:** After refresh, the account's Expires column shows a future time (not `EXPIRED`); the expiry is derived from the JWT `exp` claim of the new `accessToken`, not from the stale `expiresAt` field in the credential file.
- **Exit:** 0
- **Source fn:** `mre_bug_162_jwt_exp_ms` (in `tests/cli/usage_feature_test.rs`)
- **Note:** Fix for BUG-162; implemented by TSK-163 (`jwt_exp_ms()` in `src/output.rs`).
- **Source:** [017_token_refresh.md AC-25](../../../docs/feature/017_token_refresh.md)

---

### FT-12: `Some(paths)` — credential absent in store skips account without corrupting result

- **Given:** `claude_paths = Some(paths)` (lifecycle mode); one saved account with a 401 error result; no per-account credential file exists in the persistent store for that account.
- **When:** `apply_refresh(&mut accounts, store.path(), Some(&paths), false)` is called (unit test context; equivalent to `clp .usage refresh::1` when the lifecycle path is active)
- **Then:** `refresh_account_token(name, store, Some(&paths))` returns `None` (no per-account credential file in store); the account is skipped via `continue`; the 401 error result is unchanged after `apply_refresh` returns.
- **Source fn:** `test_apply_refresh_lifecycle_switch_fails_result_unchanged`
- **Note:** BUG-165 regression guard; covers the `Some(paths)` early-exit path not testable at CLI level without spawning live subprocess.
- **Source:** [017_token_refresh.md Algorithm](../../../docs/feature/017_token_refresh.md)

---

### FT-13: `apply_refresh` does not call `switch_account`; `_active` marker unchanged throughout cycle

- **Given:** Active marker (`_active_{hostname}_{user}`) contains `"alice@example.com"`; `alice@example.com.credentials.json` exists in the persistent store; `{fake_home}/.claude/` directory exists; one account `"bob@example.com"` has a 401 error but no credential file in the persistent store.
- **When:** `apply_refresh(&mut accounts, store.path(), Some(&paths), false)` is called (unit test context; equivalent to `clp .usage refresh::1` cycling through accounts)
- **Then:** `refresh_account_token("bob@example.com", ...)` returns `None` (no credential file) and bob is skipped; `apply_refresh` returns without calling `switch_account`; `{store}/_active_{hostname}_{user}` still contains `"alice@example.com"` (unchanged); `{fake_home}/.claude/.credentials.json` does NOT exist (no `switch_account` was called).
- **Source fn:** `test_apply_refresh_lifecycle_active_marker_unchanged`
- **Note:** Fix for BUG-211 — snapshot+restore removed from `apply_refresh`; `refresh_account_token` passes `update_marker=false` to `save()` so `_active` is never written during per-account cycling.
- **Source:** [017_token_refresh.md Algorithm](../../../docs/feature/017_token_refresh.md)

---

### FT-14: `None`-paths — credential absent in store skips account without corrupting result

- **Given:** `claude_paths = None` (persistent-store mode); one saved account with a 401 error result; no per-account credential file (`{name}.credentials.json`) exists in the persistent store.
- **When:** `apply_refresh(&mut accounts, store.path(), None, false)` is called (unit test context; equivalent to `clp .usage refresh::1` with no live session)
- **Then:** `refresh_account_token(name, store, None)` returns `None` (credential file absent in persistent store); the account is skipped via `continue`; the 401 error result is unchanged after `apply_refresh` returns.
- **Source fn:** `test_apply_refresh_401_no_cred_file` (C2 — covers None-paths + no credential file)
- **Note:** Symmetric to FT-12 for the `None`-paths branch; verifies the persistent-store fallback path exits cleanly when the per-account credential file is absent.
- **Source:** [017_token_refresh.md Algorithm](../../../docs/feature/017_token_refresh.md)

---

### FT-15: `trace::1` propagates to `refresh_account_token`; lifecycle steps logged to stderr

- **Given:** `refresh_account_token` is called via `apply_refresh` with `trace=true`; the credential file exists in the persistent store (so `read credentials` succeeds) AND `{fake_home}/.claude/` directory exists (so the write-credentials path has a valid parent if reached); `run_isolated` fails fast (no valid claude binary or fake token).
- **When:** `apply_refresh(&mut accounts, store.path(), Some(&paths), true)` is called (unit test; equivalent to `clp .usage refresh::1 trace::1`)
- **Then:** A timestamped line `... · refresh {name}  read credentials: OK` and `... · refresh {name}  run_isolated: invoking claude  args=["--print", "."]  timeout=35s` are emitted to stderr (in that order); `... · refresh {name}  run_isolated: Err(…)` or `OK credentials=None` follows; no panic; account result unchanged.
- **Source fn:** `test_apply_refresh_lifecycle_l010_trace_run_isolated_invoked_no_panic` (L10 in `usage.rs`), `art_some_paths_run_isolated_invoked_trace_no_panic` (in `account_refresh_test.rs`)
- **Note:** Fix for BUG-166 — `refresh_account_token` previously had no `trace` parameter; all failure paths returned `None` silently without any diagnostic output. Testing uses "does not panic" pattern because nextest does not support reliable stderr assertion for `eprintln!` in unit tests.
- **Source:** [017_token_refresh.md AC-26](../../../docs/feature/017_token_refresh.md)

---

### FT-16: Post-refresh `expires_at_ms` from `expiresAt` field for opaque `sk-ant-oat01-*` token

- **Given:** One saved account whose credential is expired; `account::refresh_account_token()` returns `Some(new_creds)` where the `accessToken` is an opaque `sk-ant-oat01-*` value (no `.` separator — `jwt_exp_ms` returns `None`); the `new_creds` JSON contains a future `expiresAt` value written by the OAuth server.
- **When:** `apply_refresh` processes `new_creds` (unit test via `test_apply_refresh_mre_bug170_opaque_token_expires_fallback`)
- **Then:** `account_quota.expires_at_ms` is set to the `expiresAt` value from `new_creds`; the Expires column shows a future time (not `EXPIRED`); expiry is derived from `parse_u064_field(new_creds, "expiresAt")`, not from JWT decode.
- **Exit:** 0
- **Source fn:** `test_jwt_exp_ms_mre_bug170_opaque_returns_none` (in `src/usage/refresh_tests.rs`)
- **Note:** Fix for BUG-170 — the TSK-163 fix for BUG-162 introduced this gap: `jwt_exp_ms` silently returns `None` for opaque tokens, leaving `expires_at_ms` stale. The `expiresAt` field in the returned credentials JSON is the authoritative post-refresh expiry for opaque tokens.
- **Source:** [017_token_refresh.md AC-25](../../../docs/feature/017_token_refresh.md)

---

### FT-17: No `switch_account` in `apply_refresh`; `_active` unchanged confirms no restore occurred

- **Given:** Active marker contains `"alice@example.com"`; `alice@example.com.credentials.json` exists in the persistent store; `{fake_home}/.claude/` directory exists; one account `"bob@example.com"` has a 401 error but no credential file in the persistent store.
- **When:** `apply_refresh(&mut accounts, store.path(), Some(&paths), true)` is called with `trace=true` (unit test context; equivalent to `clp .usage refresh::1 trace::1`)
- **Then:** `apply_refresh` returns without calling `switch_account`; `{fake_home}/.claude/.credentials.json` does NOT exist (no restore occurred); `{store}/_active_{hostname}_{user}` is unchanged (`"alice@example.com"`); no timestamped `... · refresh  {name}  restore switch_account:` line is emitted (restore step no longer exists).
- **Source fn:** `test_apply_refresh_mre_bug208_restore_trace_emitted` (in `src/usage/refresh_tests.rs`)
- **Note:** Fix for BUG-211 — snapshot+restore removed from `apply_refresh`. Previous BUG-208 fix (restore trace instrumentation) is superseded: the entire restore block is gone, so there is no restore line to emit.
- **Source:** [017_token_refresh.md AC-28](../../../docs/feature/017_token_refresh.md)

---

### FT-18: After refresh, `aq.account` re-populated via `fetch_oauth_account(new_token)` (BUG-171 MRE)

- **Given:** `apply_refresh()` has successfully re-fetched quota for one account (i.e., `account_quota.result` transitioned from `Err(auth_error)` to `Ok(quota_data)`).
- **When:** The `Fix(BUG-171)` code path runs: `if let Ok( acct ) = claude_quota::fetch_oauth_account( &token ) { aq.account = Some( acct ); }`.
- **Then:** `account_quota.account` is `Some(...)` — `~Renews` and `Sub` columns show current data for the refreshed account, not stale `?`. If `fetch_oauth_account` fails, the original `aq.account` value is preserved (non-aborting).
- **Exit:** n/a (structural — verifies `Fix(BUG-171)` presence in production code)
- **Source fn:** `mre_bug_171_account_populated_after_refresh` (in `tests/cli/usage_test.rs`)
- **Note:** BUG-171 fix — before fix, `aq.account` remained `None` after refresh because the initial fetch used the expired token and the retry path never re-populated account data.
- **Source:** [017_token_refresh.md AC-27](../../../docs/feature/017_token_refresh.md)

---

### FT-19: `refresh_account_token` returns `None` → `aq.result = Err("refresh token expired")` (BUG-297 MRE)

- **Given:** One `AccountQuota` with `cached: true` and `result: Ok(cached_data)` (cache fallback masked the original auth error); `refresh_account_token` returns `None` — the OAuth refresh token has expired and `run_isolated` exits without writing new credentials.
- **When:** `apply_refresh(&mut accounts, store.path(), None, false)` processes the account and the `None` branch executes.
- **Then:** `account_quota.result` is set to `Err("refresh token expired")` before `continue;` — it is NOT left as `Ok(cached_data)`. Downstream phases (`apply_touch`) see `Err` and skip the account, preventing a redundant subprocess on an unrecoverable account.
- **Source fn:** `mre_bug297_refresh_none_sets_aq_result_err` (in `src/usage/refresh_tests.rs`)
- **Note:** Fix for BUG-297. Pre-fix: `apply_refresh` left `aq.result=Ok(cached_data)` when refresh returned `None`, causing `apply_touch` to fire a subprocess on an account that cannot recover without manual browser re-authentication.
- **Source:** [017_token_refresh.md AC-30](../../../docs/feature/017_token_refresh.md)

---

### FT-20: `should_refresh()` returns `false` for owned account with `is_occupied_elsewhere == true` (BUG-303 MRE)

- **Given:** `should_refresh()` is called with one `AccountQuota` where `is_owned = true` (this machine owns the credentials) AND `is_occupied_elsewhere = true` (another machine's `_active_*` marker file names this account as its active account). The account has a 401 error result that would normally trigger refresh.
- **When:** `should_refresh(&aq)` evaluates the G2 gate.
- **Then:** `should_refresh` returns `false` — the occupancy guard fires and blocks credential mutation. No `refresh_account_token` call is made. The owned-but-occupied account is skipped as if it were non-owned.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `mre_bug303_should_refresh_false_for_occupied_elsewhere` (in `src/usage/refresh_predicate.rs` `#[cfg(test)]` module)
- **Note:** BUG-303 MRE (Critical). Before the fix, G2 at `refresh_predicate.rs:32` only checked `!aq.is_owned`, allowing `should_refresh` to return `true` for owned+occupied accounts. Refreshing an occupied account writes new `accessToken`/`refreshToken` to disk while the other machine is actively using those credentials, invalidating its live session. Fix: `if !aq.is_owned || aq.is_occupied_elsewhere { return false; }`. Mirrors `ft06_should_refresh_false_when_not_owned` — same file, same `#[cfg(test)]` block, occupancy variant. This tests the predicate gate; `apply_refresh` never reaches the refresh body when `should_refresh` returns `false`.
- **Source:** [017_token_refresh.md AC-31](../../../docs/feature/017_token_refresh.md)

---

### FT-21: `apply_refresh` trace emits `reason: cached-expired` (not `reason: ok`) for owned+cached+expired account (BUG-298 MRE)

- **Given:** One `AccountQuota` with `is_owned = true`, `cached = true`, `result = Ok(cached_data)` (cache fallback converted Err→Ok), and `expires_at_ms = 0` (expired — BUG-255 guard fires). `apply_refresh` is called with `trace = true`.
- **When:** The trace reason expression in `refresh.rs` evaluates for this account. The BUG-255 guard (`aq.cached && expired`) causes `should_retry = true`.
- **Then:** Stderr contains `reason: cached-expired`. Stderr does NOT contain `reason: ok`. The `else if aq.cached` branch in `reason_label(aq, now_secs)` fires before `aq.result.as_ref().err()` — which would return `None` for `Ok(cached_data)` and produce the misleading constant `"ok"`. Within the cached branch, `(expires_at_ms / 1000) <= now_secs` is true (token expired) → `"cached-expired"`.
- **Source fn:** `mre_bug298_apply_refresh_trace_reason_cached_expired` (in `src/usage/refresh_tests.rs`)
- **Note:** Fix for BUG-298. Root cause: `fetch.rs:229-240` cache fallback converts Err→Ok and sets `aq.cached=true`, making `aq.result.err()` always `None`. The original reason expression `map_or("ok", ...)` on that `None` produced the constant `"ok"` for all cached+owned accounts regardless of triggering cause. The fix adds an explicit branch ordered before the `err()`-based path: not-owned → `"not owned"`, owned+cached+expired → `"cached-expired"`, owned+cached+valid → `"cached"`, owned+live → error string or `"ok"`.
- **Source:** [017_token_refresh.md Algorithm](../../../docs/feature/017_token_refresh.md)

---

### FT-22: `refresh_account_token` sets `expiresAt=1` before `run_isolated` — RT rotates on every call

- **Given:** `refresh_account_token` is called with stored credentials containing a valid (far-future) `expiresAt` value. The access token is not expired.
- **When:** The function prepares credentials for `run_isolated`.
- **Then:** The credential JSON passed to `run_isolated` has `expiresAt` set to `"1"` (past timestamp), forcing Claude CLI to treat AT as expired and use RT to obtain a fresh AT+RT pair. The original stored credential file is NOT modified. `run_isolated` returns `credentials=Some(new_creds)` with a rotated RT.
- **Source fn:** `ft22_manipulate_expires_at_replaces_numeric_value`, `ft22_manipulate_expires_at_replaces_quoted_value`, `ft22_manipulate_expires_at_noop_when_key_absent`, `ft22_manipulate_expires_at_called_before_run_isolated_structural` (in `claude_profile_core/tests/account_refresh_test.rs`)
- **Source:** [017_token_refresh.md AC-32](../../../docs/feature/017_token_refresh.md)

---

### FT-23: Current account — live creds differ from stored -> sync live->store, no subprocess spawned

- **Given:** `refresh_account_token` is called for the current account with `paths = Some(...)`. Live credentials at `~/.claude/.credentials.json` differ from stored credentials at `{store}/{name}.credentials.json` (the live Claude Code session already refreshed and rotated the RT).
- **When:** The function compares live credentials with stored credentials.
- **Then:** Live credentials are written to the store via `std::fs::write` + `save()`; `Some(live_creds)` is returned; no `run_isolated` subprocess is spawned. The live session's fresh RT is preserved in the store.
- **Source fn:** `ft23_live_sync_returns_live_creds_without_subprocess` (in `claude_profile_core/tests/account_refresh_test.rs`)
- **Source:** [017_token_refresh.md AC-33](../../../docs/feature/017_token_refresh.md)

---

### FT-24: Current account — `run_isolated` returns `None` -> race recovery reads live creds

- **Given:** `refresh_account_token` is called for the current account with `paths = Some(...)`. Live credentials initially match stored credentials. After `run_isolated` is invoked, the live Claude Code session refreshes concurrently — live credentials now differ from stored.
- **When:** `run_isolated` returns `credentials=None` (the RT passed to the subprocess was already consumed by the live session's concurrent refresh).
- **Then:** The function re-reads live credentials. They differ from stored (race detected). Live credentials are written to the store; `Some(live_creds)` is returned. The function does NOT return `None`.
- **Source fn:** `ft24_some_paths_branch_reads_credentials_file_twice_structural` (structural — verifies race-recovery code path exists via `credentials_file()` call count in `refresh_token_with_live_path`; behavioral test not feasible without mocking `run_isolated`, which is prohibited by testing principles)
- **Source:** [017_token_refresh.md AC-33](../../../docs/feature/017_token_refresh.md)

---

### FT-25: `claude_profile/src/` contains zero direct `run_isolated` calls (invariant 008)

- **Given:** The `src/` directory of `claude_profile` crate at the current HEAD.
- **When:** `grep -rn "run_isolated(" src/` is run from `module/claude_profile/`.
- **Then:** Zero matches. All token refresh operations go through `refresh_account_token()` in `claude_profile_core/src/account.rs`. The grep-based invariant test in `tests/` enforces this at CI.
- **Source fn:** `single_token_refresh_entry_in1_src_contains_zero_run_isolated_calls` (in `module/claude_profile/tests/cli/invariant_test.rs`)
- **Source:** [017_token_refresh.md AC-34](../../../docs/feature/017_token_refresh.md), [invariant/008_single_token_refresh_entry.md](../../../docs/invariant/008_single_token_refresh_entry.md)

---

### FT-26: Current account becomes non-current during `run_isolated` window — stale `is_active` must not write wrong-account credentials (BUG-316 MRE)

- **Given:** `refresh_token_with_live_path("A", paths, ...)` is entered. The active marker `_active_{host}_{user}` initially contains `"A"` — `is_active` would be `true` if computed here. A per-account credential file `A.credentials.json` exists in the store. Between the initial active-marker read and the race-recovery block (simulating 35 seconds of `run_isolated` blocking), the active marker is overwritten with `"B"` and the live credentials file is overwritten with B's JSON. `run_isolated` returns `credentials=None`.
- **When:** `refresh_token_with_live_path` executes the race-recovery block (the `credentials=None` arm after `run_isolated` returns). The fix re-reads the active marker before the guard.
- **Then:** The fresh re-read finds the marker contains `"B"`, not `"A"` — `is_active_now == false`. Race recovery does NOT fire. The function returns `None`. `A.credentials.json` in the store is NOT overwritten with B's credentials. A's credential store slot is unchanged.
- **Failure case (without fix):** The stale `is_active=true` (computed before `run_isolated`) causes race recovery to read the live file (containing B's credentials) and write them into `A.credentials.json` — credential cross-contamination with no error surfaced.
- **Exit:** N/A (unit test — verifies that `A.credentials.json` is unchanged after `refresh_token_with_live_path` returns `None` when active marker changes mid-function)
- **Source fn:** `mre_bug316_stale_is_active_race_recovery_copies_wrong_account_creds` (in `claude_profile_core/tests/account_refresh_test.rs`)
- **Note:** Fix for BUG-316. The race requires two OS processes (watchdog running `refresh::1` + user running `rotate::1`). The ~35-second `run_isolated` window makes this a practically exploitable race. Fix: re-read `_active_{host}_{user}` marker immediately before race-recovery guard at `account.rs:877`.
- **Source:** [017_token_refresh.md AC-33](../../../docs/feature/017_token_refresh.md)
