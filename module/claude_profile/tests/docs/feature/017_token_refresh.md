# Test: Feature 017 — Expired Token Refresh via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/017_token_refresh.md`. Each FT case maps to one acceptance criterion. Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

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
| FT-13 | `original_active` account restored to live session after refresh cycle | Algorithm | test_apply_refresh_lifecycle_original_active_restored |
| FT-14 | `None`-paths fallback — credential absent in store → `refresh_account_token` returns `None` | Algorithm | test_apply_refresh_401_no_cred_file |
| FT-15 | `trace::1` propagated to `refresh_account_token`; lifecycle steps logged to stderr; no panic | AC-26 | test_apply_refresh_lifecycle_l010_trace_run_isolated_invoked_no_panic, art_some_paths_run_isolated_invoked_trace_no_panic |
| FT-16 | `expires_at_ms` from `expiresAt` field when JWT decode returns `None` (opaque token) | AC-25 | test_parse_u064_from_str_mre_bug170_extracts_expires_at, test_jwt_exp_ms_mre_bug170_opaque_returns_none |
| FT-17 | `trace::1` emits `restore switch_account` line after refresh cycle; error paths emit unconditionally | AC-28 | test_apply_refresh_mre_bug208_restore_trace_emitted |

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
| FT-13 | `original_active` restored via `switch_account` after refresh cycle | Algorithm | Active Restore |
| FT-14 | `None`-paths — credential absent in store skips account without corrupting result | Algorithm | None-paths Skip |
| FT-15 | `trace::1` propagates to `refresh_account_token`; lifecycle steps logged; no panic | AC-26 | Trace Propagation |
| FT-16 | Post-refresh `expires_at_ms` from `expiresAt` field for opaque `sk-ant-oat01-*` token | AC-25 | JWT Expiry (Opaque) |
| FT-17 | `trace::1` emits `restore switch_account` line after refresh cycle; failure always visible | AC-28 | Restore Trace |

**Total:** 17 FT cases

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
- **Source fn:** `test_jwt_exp_ms_mre_bug162`
- **Note:** Fix for BUG-162; implemented by TSK-163 (`jwt_exp_ms()` in `src/usage.rs`).
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

### FT-13: `original_active` account restored to live session after refresh cycle

- **Given:** Active marker (`_active_{hostname}_{user}`) contains `"alice@example.com"`; `alice@example.com.credentials.json` exists in the persistent store; `{fake_home}/.claude/` directory exists for the live session; one account `"bob@example.com"` has a 401 error but no credential file in the persistent store.
- **When:** `apply_refresh(&mut accounts, store.path(), Some(&paths), false)` is called (unit test context; equivalent to `clp .usage refresh::1` cycling through accounts)
- **Then:** `switch_account("bob@example.com", ...)` fails and bob is skipped; after the loop, `switch_account("alice@example.com", store, paths)` runs (restore); `{store}/_active_{hostname}_{user}` contains `"alice@example.com"`; `{fake_home}/.claude/.credentials.json` contains alice's credential content.
- **Source fn:** `test_apply_refresh_lifecycle_original_active_restored`
- **Note:** BUG-165 regression guard; the restore call at `usage.rs:897-904` had zero unit test coverage before TSK-166.
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
- **Then:** `[trace] refresh {name}  read credentials: OK` and `[trace] refresh {name}  run_isolated: invoking claude  args=["--print", "."]  timeout=35s` are emitted to stderr (in that order); `[trace] refresh {name}  run_isolated: Err(…)` or `OK credentials=None` follows; no panic; account result unchanged.
- **Source fn:** `test_apply_refresh_lifecycle_l010_trace_run_isolated_invoked_no_panic` (L10 in `usage.rs`), `art_some_paths_run_isolated_invoked_trace_no_panic` (in `account_refresh_test.rs`)
- **Note:** Fix for BUG-166 — `refresh_account_token` previously had no `trace` parameter; all failure paths returned `None` silently without any diagnostic output. Testing uses "does not panic" pattern because nextest does not support reliable stderr assertion for `eprintln!` in unit tests.
- **Source:** [017_token_refresh.md AC-26](../../../docs/feature/017_token_refresh.md)

---

### FT-16: Post-refresh `expires_at_ms` from `expiresAt` field for opaque `sk-ant-oat01-*` token

- **Given:** One saved account whose credential is expired; `account::refresh_account_token()` returns `Some(new_creds)` where the `accessToken` is an opaque `sk-ant-oat01-*` value (no `.` separator — `jwt_exp_ms` returns `None`); the `new_creds` JSON contains a future `expiresAt` value written by the OAuth server.
- **When:** `apply_refresh` processes `new_creds` (unit test via `test_apply_refresh_mre_bug170_opaque_token_expires_fallback`)
- **Then:** `account_quota.expires_at_ms` is set to the `expiresAt` value from `new_creds`; the Expires column shows a future time (not `EXPIRED`); expiry is derived from `parse_u064_field(new_creds, "expiresAt")`, not from JWT decode.
- **Exit:** 0
- **Source fn:** `test_parse_u064_from_str_mre_bug170_extracts_expires_at` (primary fix guard), `test_jwt_exp_ms_mre_bug170_opaque_returns_none` (precondition guard) — both in `src/usage.rs` `mod tests`
- **Note:** Fix for BUG-170 — the TSK-163 fix for BUG-162 introduced this gap: `jwt_exp_ms` silently returns `None` for opaque tokens, leaving `expires_at_ms` stale. The `expiresAt` field in the returned credentials JSON is the authoritative post-refresh expiry for opaque tokens.
- **Source:** [017_token_refresh.md AC-25](../../../docs/feature/017_token_refresh.md)

---

### FT-17: `trace::1` emits `restore switch_account` line after refresh cycle; failure always logged

- **Given:** `apply_refresh` is called with `trace=true`; at least one account exists so the restore branch is reached; `original_active` is set (active marker contains a non-empty account name); the original account has a credential file in the store so `switch_account` can succeed.
- **When:** `apply_refresh(&mut accounts, store.path(), Some(&paths), true)` is called (unit test context; equivalent to `clp .usage refresh::1 trace::1` with active marker present)
- **Then:** Stderr contains `[trace] refresh  {original_name}  restore switch_account: OK`; the restore step is not silent under `trace::1`.
- **And:** In a separate scenario where `switch_account` fails at restore time (e.g., credential file deleted between snapshot and restore), stderr contains the failure line unconditionally — without requiring `trace=true`.
- **Source fn:** `test_apply_refresh_mre_bug208_restore_trace_emitted` (in `tests/cli/usage_test.rs` or `src/usage.rs #[cfg(test)]`)
- **Note:** Fix for BUG-208 — both `apply_refresh` and `apply_touch` used `let _ = switch_account(...)` at the restore site, making restore failures silent and restore trace completeness impossible.
- **Source:** [017_token_refresh.md AC-28](../../../docs/feature/017_token_refresh.md)
