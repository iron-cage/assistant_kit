# Feature: Expired Token Refresh via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to silently refresh expired OAuth tokens before fetching quota, so users see current quota data rather than per-account auth error rows.
- **Responsibility**: Documents the `refresh::` parameter, its retry-on-auth-error trigger, the `claude_profile_core::account::refresh_account_token()` call from `usage.rs`, and credential write-back to disk.
- **In Scope**: `refresh::` parameter semantics; HTTP auth error detection from `fetch_oauth_usage`; `account::refresh_account_token()` call from `usage.rs`; credential write-back; one-retry-per-account semantics; non-aborting error handling.
- **Out of Scope**: `run_isolated()` internals (→ `claude_runner_core/docs/feature/004_run_isolated.md`); live monitor mode (→ `018_live_monitor.md`); `fetch_oauth_usage` implementation (→ `claude_quota`); proactive expiry detection before any API call.

### Design

The `refresh::` parameter takes `1` (default, on) or `0` (off). When `0`, `.usage` behaves identically to the baseline — auth errors appear as error rows in the table and no subprocess is spawned.

When `refresh::1`, the command wraps `fetch_oauth_usage` with a retry layer: on an HTTP authentication error (401 or 403), it calls `claude_profile_core::account::refresh_account_token()` for that account, then retries the quota fetch if updated credentials are returned. `refresh_account_token()` encapsulates the full account lifecycle: `read credentials` → `run_isolated` (via `claude_runner_core`) → `write credentials` → `save`; it returns `Some(new_creds_json)` on success or `None` on any failure in the lifecycle.

**Trigger condition:** HTTP auth errors (401, 403) always trigger a refresh attempt. Additionally, HTTP 429 (rate-limit) triggers a refresh when the per-account credential file has a locally-expired `expiresAt` — this handles the case where Claude Code updated `~/.claude/.credentials.json` in the live session but the saved per-account copy was never re-saved, leaving a stale token. Network failures, timeouts, and 429 with a non-expired local token are passed through as-is, preventing unnecessary subprocess launches.

**Algorithm (per-account loop):**

```
results = fetch_all_quota(credential_store, live_creds_file)   // all accounts

if refresh_param == 1:
    now_secs = current_unix_timestamp()
    for each account_quota in results where should_refresh(account_quota, now_secs):
        // should_refresh returns true for:
        //   - result is auth_error("401") or auth_error("403")
        //   - result is rate_limit("429") AND expires_at_ms / 1000 <= now_secs
        new_json = account::refresh_account_token(account_quota.name, credential_store, claude_paths, trace)
        // Encapsulates: read credentials → run_isolated(["--print", "."], 35s) → write credentials → save(update_marker=false)
        // Returns Some(new_creds_json) if lifecycle succeeds; None on any failure

        if new_json is Some(json):
            // Step 1: derive expiry from JWT exp claim (works for JWT-format accessTokens)
            // Step 2: fall back to expiresAt field for opaque sk-ant-oat01-* tokens (BUG-170)
            if let Some(exp_ms) = jwt_exp_ms(json.access_token):
                account_quota.expires_at_ms = exp_ms
            else if let Some(exp_ms) = parse_u064_field(json, "expiresAt"):
                account_quota.expires_at_ms = exp_ms
            // else: unchanged (last-resort safe fallback — both strategies failed)
            account_quota.result = fetch_oauth_usage(new_token)   // retry this account only
            if account_quota.result is Ok:
                // Fix(BUG-171): re-populate account data with new token so ~Renews/Sub show current values
                if let Some(account_data) = fetch_oauth_account(new_token):
                    account_quota.account = Some(account_data)
                // on fetch_oauth_account failure: original account_quota.account preserved (non-aborting)
        else:
            // original error preserved; account row shows pre-refresh error state

render results as table
```

**`expiresAt` not written by subprocess (BUG-162):** The isolated subprocess updates `accessToken` and `refreshToken` in the credentials file but does NOT update `expiresAt` — that field is a server-side claim set at token issuance and is not written back during the OAuth refresh exchange. Reading `expiresAt` from the credentials file after the subprocess returns therefore yields the **original expired timestamp**, causing `compute_expires_cell` to continue showing `EXPIRED` regardless of the successful refresh. The correct source for post-refresh expiry is the JWT `exp` claim embedded in the new `accessToken` (second `.`-separated base64url segment, `"exp"` field, seconds → multiply by 1000 for ms). See [BUG-162](../../../../task/claude_profile/bug/162_expiresAt_not_updated_by_subprocess.md) and TSK-163.

**Expired refresh token (expected limitation):** When an account's OAuth refresh token has itself expired (distinct from access token expiry), `run_isolated` cannot obtain new credentials — Claude Code contacts the OAuth server with the expired refresh token, gets rejected, and does not update the credential file. In this case `credentials` is `None`, the account is skipped, and the original auth error persists in the output. The operational remedy is to re-authenticate the affected account via browser-based OAuth flow and `clp .account.save`.

**Rate-limit handling (conditional refresh):** HTTP 429 responses are handled conditionally via `should_refresh()`. When the per-account credential file has a non-expired `expiresAt` (local token appears valid), 429 is passed through without retry — the token is valid and the rate limit must resolve on its own. When `expiresAt` is past (locally expired), 429 may indicate that the rate-limit check ran before auth, and the per-account file may be stale (Claude Code updated `~/.claude/.credentials.json` but not the saved per-account copy). In this case, a refresh attempt is made. Refreshing ALL 429 responses unconditionally (as an earlier task did) added a pointless 30-second wait for valid-but-rate-limited accounts; refreshing NONE (as the task-150 fix did) broke recovery for accounts with stale per-account files. `shorten_error()` renders `"HTTP transport error: HTTP 429"` as `"rate limited (429)"` in the Status column.

**Retry semantics:** Exactly one retry per account per invocation. If the retried `fetch_oauth_usage` also fails, the final error is shown in the account's row — the table continues processing remaining accounts (non-aborting).

**Credential write-back:** When `run_isolated` returns `credentials: Some(new_json)`, the live session file (`~/.claude/.credentials.json`) is overwritten with `new_json`, then `account::save()` copies it to `{credential_store}/{name}.credentials.json` and updates the per-machine active marker and companion files atomically. This ensures the live session, persistent store, and companion files are all consistent after a successful refresh. See [BUG-165](../../../../task/claude_profile/bug/165_apply_refresh_skips_account_lifecycle.md) — the previous implementation wrote only to the persistent store, leaving the live session stale.

**Subprocess trigger mechanism:** `run_isolated` must be invoked with `["--print", "."]` so Claude Code performs its startup OAuth token refresh before making the API call. At process startup, Claude Code refreshes the OAuth access token if expired — writing updated credentials to `$HOME/.claude/.credentials.json` — then attempts the `--print .` API call. The API call may succeed, fail, or time out, but credentials are written at startup regardless. The `isolated.rs` `issue-isolated-credentials-on-timeout` fix handles timeout exactly: when the credentials file changes before the 35-second timeout fires, `run_isolated` returns `Ok(IsolatedRunResult { credentials: Some(new_json), exit_code: -1 })` — the updated credentials are captured even when the subprocess was terminated by timeout.

Two other arg combinations are broken and must not be used:
- **Empty args `[]`** (TSK-168 regression, [BUG-169](../../../../task/claude_profile/bug/169_refresh_args_interactive_mode_regression.md)): Claude Code in non-TTY mode with no args detects it has nothing to do and exits immediately, without performing startup OAuth token refresh. The subprocess returns exit 0 but never writes to the credentials file — `run_isolated` returns `credentials: None` for every expired account.
- **`["--print", ".", "--max-tokens", "1"]`** (original issue-151 bug): `--max-tokens 1` triggers an API rejection before the OAuth exchange can occur. Credentials are never written. See [TSK-151](../../../../task/claude_profile/151_refresh_failure_message.md).

**Feature gate:** The retry logic is compiled only under `#[cfg(feature = "enabled")]`, matching `fetch_oauth_usage`. When `enabled` is absent, `refresh::1` is accepted as a parameter but no refresh attempt is made (offline builds cannot spawn subprocesses).

**Default is on:** `refresh::1` is the default — every `clp .usage` call automatically retries on 401/403. Use `refresh::0` to explicitly disable. `refresh::0` introduces no subprocess spawn and no credential file writes.

**Output format:** When refresh succeeds, the account's row shows normal quota data — the refresh is invisible to the user. When refresh fails (subprocess error or second fetch also fails), the error reason appears in the account's row exactly as it would without `refresh::`.

### Acceptance Criteria

- **AC-18**: `refresh::0` produces no calls to `run_isolated`; `.usage` behavior is unchanged from the baseline. Use `refresh::0` to explicitly disable the default refresh behavior.
- **AC-19**: `refresh::1` (default) invokes `claude_profile_core::account::refresh_account_token()` (which internally calls `claude_runner_core::run_isolated()`) for any account whose `fetch_oauth_usage` returns an HTTP authentication error (401 or 403), or an HTTP 429 rate-limit error when the per-account credential file has a locally-expired `expiresAt`. HTTP 429 with a non-expired local token is passed through unchanged.
- **AC-24**: The `refresh::` parameter description in `.usage --help` documents the conditional 429 case ("429 when token is locally expired") and does NOT describe 429 as unconditionally excluded from refresh.
- **AC-20**: When `run_isolated` returns `credentials: Some(new_json)`, the live session file is updated first, then `account::save()` propagates the new credentials to the persistent store, per-machine active marker, and companion files before the retry fetch.
- **AC-21**: If the refresh attempt fails (subprocess error, or retried fetch still fails), the account's row shows the final error; the remaining accounts are still processed and the table is still rendered.
- **AC-22**: `refresh::` does not affect `format::json` output structure — refreshed accounts appear as normal data objects, failed-refresh accounts appear as error objects.
- **AC-23**: The `refresh::` parameter appears in `.usage --help` output with its default value (`1`).
- **AC-25**: After `run_isolated` returns `credentials: Some(new_json)`, `account_quota.expires_at_ms` is updated using a two-step fallback: (1) decode the JWT `exp` claim from the new `accessToken` via `jwt_exp_ms(new_json)` — preferred for JWT-format tokens; (2) if JWT decoding returns `None` (e.g., opaque `sk-ant-oat01-*` tokens with no `.` separator), read `expiresAt` directly from the credentials JSON via `parse_u064_field(new_json, "expiresAt")`. If both strategies fail, `expires_at_ms` is left unchanged as a last-resort safe fallback. Fix for [BUG-170](../../../../task/claude_profile/bug/170_expires_column_stale_after_refresh_opaque_token.md).
- **AC-26**: When `trace=true`, `refresh_account_token` emits `[trace] refresh {name}  {step}: {outcome}` lines to stderr for each lifecycle step — `read credentials`, `run_isolated` (with `"invoking claude  args=["--print", "."]  timeout=35s"` before the call), `write credentials`, and `save`. Each outcome is either `OK` (or `OK credentials={Some|None}` for `run_isolated`) or `Err({error})`. The `trace` parameter is forwarded by `apply_refresh` into `refresh_account_token` so the full lifecycle is observable from `clp .usage refresh::1 trace::1`. Fix for [BUG-166](../../../../task/claude_profile/bug/166_refresh_account_token_no_trace.md).
- **AC-27**: After `apply_refresh()` successfully re-fetches quota (i.e., `account_quota.result` transitions to `Ok`), `account_quota.account` is re-populated by calling `fetch_oauth_account()` with the new token. Consequence: `~Renews` and `Sub` columns show current data for successfully-refreshed accounts rather than the stale `?` they would show if `aq.account` were left as `None`. If the `fetch_oauth_account()` call fails, the original `account_quota.account` value is preserved unchanged (non-aborting). Fix for BUG-171.
- **AC-28**: After the per-account refresh loop, `apply_refresh` does NOT call `switch_account` to restore the active account. Instead, `refresh_account_token` passes `update_marker=false` to `save()` — the `_active` marker is never written during per-account cycling, so no restore is needed. Fix for BUG-211.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `refresh::` param read; retry trigger; calls `account::refresh_account_token()`; expiry derivation; retry fetch |
| source | `src/lib.rs` | `refresh::` parameter registration via `register_commands()` |
| source | `claude_profile_core/src/account.rs` | `refresh_account_token()` — `read credentials → run_isolated → write credentials → save` lifecycle |
| dep | `claude_runner_core` | `run_isolated()` — called by `refresh_account_token()` in `_core`; `IsolatedRunResult`, `RunnerError` types |
| dep | `claude_quota` | `fetch_oauth_usage()` — quota HTTP transport; `QuotaError::HttpTransport` |
| task | `task/claude_runner_core/136_run_isolated_subprocess.md` | Prerequisite: implement `run_isolated()` |
| task | `task/claude_profile/137_usage_refresh_param.md` | Implementation task for this feature |
| task | `task/claude_profile/142_fix_refresh_per_account.md` | Per-account loop fix (replaced batch refresh) |
| task | `task/claude_profile/150_fix_apply_refresh_429_trigger.md` | Removed 429 from unconditional retry guard |
| bug | `task/claude_profile/bug/156_refresh_429_expired_not_refreshed.md` | BUG-156: conditional 429+expired refresh fix |
| bug | `task/claude_profile/bug/162_expiresAt_not_updated_by_subprocess.md` | BUG-162: subprocess never writes `expiresAt`; use JWT `exp` instead |
| bug | `task/claude_profile/bug/170_expires_column_stale_after_refresh_opaque_token.md` | BUG-170: `jwt_exp_ms` returns None for opaque tokens; add `expiresAt` fallback |
| bug | `task/claude_profile/bug/165_apply_refresh_skips_account_lifecycle.md` | BUG-165: live session not updated after refresh; fixed by account lifecycle |
| bug | `task/claude_profile/bug/175_switch_account_before_run_isolated_unnecessary_global_write.md` | BUG-175: `Some(paths)` branch called `switch_account` before reading creds — unnecessary global write; removed |
| bug | `task/claude_profile/bug/208_restore_switch_account_silent_result_discard.md` | BUG-208: restore `switch_account` calls wrapped in `let _ = ...` — silent error discard, no `[trace]` line under `trace::1` |
| bug | `task/claude_profile/bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md` | BUG-211 (Fixed): snapshot+restore removed from `apply_refresh`; `save(update_marker=false)` suppresses `_active` writes during per-account cycling |
| bug | `task/claude_profile/bug/166_refresh_account_token_no_trace.md` | BUG-166: `refresh_account_token` had no trace param; all failure paths silently returned `None` |
| bug | `task/claude_profile/bug/169_refresh_args_interactive_mode_regression.md` | BUG-169: TSK-168 regression — empty args `[]` broken; `["--print", "."]` is the only correct invocation |
| task | `task/claude_profile/163_fix_expiresAt_jwt_decode.md` | TSK-163: implement `jwt_exp_ms()` and fix `apply_refresh` expiry derivation |
| task | `task/claude_profile/151_refresh_failure_message.md` | Fixed empty args in `run_isolated` call |
| task | `task/claude_profile/168_fix_refresh_account_token_args.md` | TSK-168: fix broken `--print . --max-tokens 1` args — use `["--print", "."]` (introduced BUG-169 regression) |
| doc | [009_token_usage.md](009_token_usage.md) | Baseline `.usage` algorithm that this extends |
| doc | `claude_runner_core/docs/feature/004_run_isolated.md` | `run_isolated()` API contract |
| doc | [command/006_usage.md](../cli/command/006_usage.md#command--9-usage) | `.usage` CLI command specification |
| doc | [cli/param/019_refresh.md](../cli/param/019_refresh.md) | `refresh::` parameter specification |
