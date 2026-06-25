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
            // Fix(BUG-297): aq.result set to Err("refresh token expired") — original result
            //   may be Ok(cached_data) from cache masking; leaving it Ok causes apply_touch
            //   to fire a redundant subprocess on an unrecoverable account.
            account_quota.result = Err("refresh token expired")

render results as table
```

**`expiresAt` not written by subprocess (BUG-162):** The isolated subprocess updates `accessToken` and `refreshToken` in the credentials file but does NOT update `expiresAt` — that field is a server-side claim set at token issuance and is not written back during the OAuth refresh exchange. Reading `expiresAt` from the credentials file after the subprocess returns therefore yields the **original expired timestamp**, causing `compute_expires_cell` to continue showing `EXPIRED` regardless of the successful refresh. The correct source for post-refresh expiry is the JWT `exp` claim embedded in the new `accessToken` (second `.`-separated base64url segment, `"exp"` field, seconds → multiply by 1000 for ms). See BUG-162 and TSK-163.

**Expired refresh token (expected limitation):** When an account's OAuth refresh token has itself expired (distinct from access token expiry), `run_isolated` cannot obtain new credentials — Claude Code contacts the OAuth server with the expired refresh token, gets rejected, and does not update the credential file. In this case `credentials` is `None`, `apply_refresh` sets `aq.result = Err("refresh token expired")`, and processing continues with the next account. Setting `aq.result` to `Err` is required even when the pre-refresh result was `Ok(cached_data)` (from cache masking) — downstream phases (`apply_touch`) use `aq.result` as their sole recoverability signal and must not fire subprocesses on unrecoverable accounts. Fix for BUG-297. The operational remedy is to re-authenticate the affected account via browser-based OAuth flow and `clp .account.save`.

**Rate-limit handling (conditional refresh):** HTTP 429 responses are handled conditionally via `should_refresh()`. When the per-account credential file has a non-expired `expiresAt` (local token appears valid), 429 is passed through without retry — the token is valid and the rate limit must resolve on its own. When `expiresAt` is past (locally expired), 429 may indicate that the rate-limit check ran before auth, and the per-account file may be stale (Claude Code updated `~/.claude/.credentials.json` but not the saved per-account copy). In this case, a refresh attempt is made. Refreshing ALL 429 responses unconditionally (as an earlier task did) added a pointless 30-second wait for valid-but-rate-limited accounts; refreshing NONE (as the task-150 fix did) broke recovery for accounts with stale per-account files. `shorten_error()` renders `"HTTP transport error: HTTP 429"` as `"rate limited (429)"` in the Status column.

**Retry semantics:** Exactly one retry per account per invocation. If the retried `fetch_oauth_usage` also fails, the final error is shown in the account's row — the table continues processing remaining accounts (non-aborting).

**Credential write-back:** When `run_isolated` returns `credentials: Some(new_json)`, `account::save()` writes `new_json` directly to `{credential_store}/{name}.credentials.json` (via `creds: Some(new_json.as_bytes())`, the 5th param added by BUG-221) and updates the companion `{name}.json` (oauthAccount snapshot, org identity). The live session file (`~/.claude/.credentials.json`) is NOT written during batch refresh — writing to it was the root cause of BUG-221 (fixed in TSK-230): every batch refresh clobbered the user's live Claude session. See also BUG-165 for the original lifecycle extraction history.

**Subprocess trigger mechanism:** `run_isolated` must be invoked with `["--print", "."]` so Claude Code performs its startup OAuth token refresh before making the API call. At process startup, Claude Code refreshes the OAuth access token if expired — writing updated credentials to `$HOME/.claude/.credentials.json` — then attempts the `--print .` API call. The API call may succeed, fail, or time out, but credentials are written at startup regardless. The `isolated.rs` `issue-isolated-credentials-on-timeout` fix handles timeout exactly: when the credentials file changes before the 35-second timeout fires, `run_isolated` returns `Ok(IsolatedRunResult { credentials: Some(new_json), exit_code: -1 })` — the updated credentials are captured even when the subprocess was terminated by timeout.

Two other arg combinations are broken and must not be used:
- **Empty args `[]`** (TSK-168 regression, BUG-169): Claude Code in non-TTY mode with no args detects it has nothing to do and exits immediately, without performing startup OAuth token refresh. The subprocess returns exit 0 but never writes to the credentials file — `run_isolated` returns `credentials: None` for every expired account.
- **`["--print", ".", "--max-tokens", "1"]`** (original issue-151 bug): `--max-tokens 1` triggers an API rejection before the OAuth exchange can occur. Credentials are never written. See TSK-151.

**Ownership gate (G2/G3):** When account ownership is enabled (Feature 036), `should_refresh()` returns `false` for accounts where `aq.is_owned == false` (non-owned) OR `aq.is_occupied_elsewhere == true` (owned by this machine but actively in use on another machine — BUG-303 fix). The `apply_refresh()` loop skips such accounts via this predicate. Refreshing an occupied account calls `refresh_account_token()`, which writes new `accessToken`/`refreshToken` to disk and immediately invalidates the live session running on the other machine. Non-owned accounts remain on the G1 cache path (see Feature 036).

**Feature gate:** The retry logic is compiled only under `#[cfg(feature = "enabled")]`, matching `fetch_oauth_usage`. When `enabled` is absent, `refresh::1` is accepted as a parameter but no refresh attempt is made (offline builds cannot spawn subprocesses).

**Default is on:** `refresh::1` is the default — every `clp .usage` call automatically retries on 401/403. Use `refresh::0` to explicitly disable. `refresh::0` introduces no subprocess spawn and no credential file writes.

**Output format:** When refresh succeeds, the account's row shows normal quota data — the refresh is invisible to the user. When refresh fails (subprocess error or second fetch also fails), the error reason appears in the account's row exactly as it would without `refresh::`.

### Acceptance Criteria

- **AC-18**: `refresh::0` produces no calls to `run_isolated`; `.usage` behavior is unchanged from the baseline. Use `refresh::0` to explicitly disable the default refresh behavior.
- **AC-19**: `refresh::1` (default) invokes `claude_profile_core::account::refresh_account_token()` (which internally calls `claude_runner_core::run_isolated()`) for any account whose `fetch_oauth_usage` returns an HTTP authentication error (401 or 403), or an HTTP 429 rate-limit error when the per-account credential file has a locally-expired `expiresAt`. HTTP 429 with a non-expired local token is passed through unchanged.
- **AC-24**: The `refresh::` parameter description in `.usage --help` documents the conditional 429 case ("429 when token is locally expired") and does NOT describe 429 as unconditionally excluded from refresh.
- **AC-20**: When `run_isolated` returns `credentials: Some(new_json)`, `account::save()` writes `new_creds` directly to `{credential_store}/{name}.credentials.json` and updates the companion `{name}.json` (oauthAccount snapshot, org identity) before the retry fetch. `~/.claude/.credentials.json` is NOT written. Fix for BUG-221.
- **AC-21**: If the refresh attempt fails (subprocess error, or retried fetch still fails), the account's row shows the final error; the remaining accounts are still processed and the table is still rendered.
- **AC-22**: `refresh::` does not affect `format::json` output structure — refreshed accounts appear as normal data objects, failed-refresh accounts appear as error objects.
- **AC-23**: The `refresh::` parameter appears in `.usage --help` output with its default value (`1`).
- **AC-25**: After `run_isolated` returns `credentials: Some(new_json)`, `account_quota.expires_at_ms` is updated using a two-step fallback: (1) decode the JWT `exp` claim from the new `accessToken` via `jwt_exp_ms(new_json)` — preferred for JWT-format tokens; (2) if JWT decoding returns `None` (e.g., opaque `sk-ant-oat01-*` tokens with no `.` separator), read `expiresAt` directly from the credentials JSON via `parse_u064_field(new_json, "expiresAt")`. If both strategies fail, `expires_at_ms` is left unchanged as a last-resort safe fallback. Fix for BUG-170.
- **AC-26**: When `trace=true`, `refresh_account_token` emits timestamped diagnostic lines to stderr for each lifecycle step: `... · refresh {name}  {step}: {outcome}` — covering `read credentials`, `run_isolated` (with `"invoking claude  args=["--print", "."]  timeout=35s"` before the call), `write credentials`, and `save`. Each outcome is either `OK` (or `OK credentials={Some|None}` for `run_isolated`) or `Err({error})`. The `trace` parameter is forwarded by `apply_refresh` into `refresh_account_token` so the full lifecycle is observable from `clp .usage refresh::1 trace::1`. Fix for BUG-166.
- **AC-27**: After `apply_refresh()` successfully re-fetches quota (i.e., `account_quota.result` transitions to `Ok`), `account_quota.account` is re-populated by calling `fetch_oauth_account()` with the new token. Consequence: `~Renews` and `Sub` columns show current data for successfully-refreshed accounts rather than the stale `?` they would show if `aq.account` were left as `None`. If the `fetch_oauth_account()` call fails, the original `account_quota.account` value is preserved unchanged (non-aborting). Fix for BUG-171.
- **AC-28**: After the per-account refresh loop, `apply_refresh` does NOT call `switch_account` to restore the active account. Instead, `refresh_account_token` passes `update_marker=false` to `save()` — the `_active` marker is never written during per-account cycling, so no restore is needed. Fix for BUG-211.
- **AC-29**: After `apply_refresh()` completes (regardless of how many accounts were refreshed), `~/.claude/.credentials.json` is byte-identical to its state at `apply_refresh()` entry. Batch refresh must not disrupt the user's live Claude session. Fix for BUG-221.
- **AC-30**: When `refresh_account_token` returns `None` (OAuth refresh token expired — `run_isolated` exits without writing credentials), `apply_refresh` sets `aq.result = Err("refresh token expired")` before continuing to the next account. This ensures downstream phases (`apply_touch`) see `Err` and skip the account. Setting `Err` is required regardless of the pre-refresh result value — when cache masking produced `Ok(cached_data)`, leaving `aq.result` as `Ok` would allow `apply_touch` to fire a redundant subprocess. Fix for BUG-297.
- **AC-31**: `should_refresh()` returns `false` for accounts where `aq.is_occupied_elsewhere == true` — another machine is actively using this account (its name appears in that machine's `_active_*` marker file). This guard prevents calling `refresh_account_token()` on an occupied account: a successful refresh writes new `accessToken`/`refreshToken` to disk, immediately invalidating the live session on the other machine. The occupancy guard is checked alongside the ownership guard (`!aq.is_owned`) at the G2 gate in `refresh_predicate.rs`. Fix for BUG-303.
- **AC-32**: `refresh_account_token()` sets `expiresAt` to `"1"` in the credential JSON passed to `run_isolated`, forcing Claude CLI to treat the access token as expired. The CLI uses the refresh token to obtain a fresh AT+RT pair, rotating the RT on every call. The original stored credential file is NOT modified — only the in-memory copy passed to the subprocess is manipulated. This prevents RT decay: without forced expiry, `run_isolated` with a valid AT returns `credentials=None` (CLI uses AT as-is), and the RT ages silently until it expires server-side.
- **AC-33**: When `paths` is `Some` and the account being refreshed is the current account, `refresh_account_token()` independently reads the active marker (`_active_{host}_{user}`) to confirm the account is still the current session, then checks live credentials (`~/.claude/.credentials.json`) against stored credentials before spawning a subprocess. If different (the live Claude Code session already refreshed and rotated the RT), syncs live->store and returns `Some(live_creds)` without spawning `run_isolated`. After `run_isolated` returns `credentials=None`, re-reads the active marker **independently** — not reusing the pre-subprocess result, because a concurrent `switch_account` can update the marker during the 35-second subprocess window — to confirm the account is still the current session before race recovery. Only if the fresh re-read confirms the account is still active does race recovery sync live->store and return `Some(live_creds)`. Fix for BUG-316.
- **AC-34**: All token refresh operations across `claude_profile` MUST go through `refresh_account_token()`. Direct `run_isolated()` calls for credential refresh are forbidden. `apply_post_switch_touch` (previously a direct `run_isolated` call — fire-and-forget, no credential write-back) is routed through `refresh_account_token()`. See [invariant 008](../invariant/008_single_token_refresh_entry.md).

### Bugs

| File | Relationship |
|------|--------------|
| BUG-156 | BUG-156: conditional 429+expired refresh fix |
| BUG-162 | BUG-162: subprocess never writes `expiresAt`; use JWT `exp` instead |
| BUG-165 | BUG-165: live session not updated after refresh; fixed by account lifecycle |
| BUG-166 | BUG-166: `refresh_account_token` had no trace param; all failure paths silently returned `None` |
| BUG-169 | BUG-169: TSK-168 regression — empty args `[]` broken; `["--print", "."]` is the only correct invocation |
| BUG-170 | BUG-170: `jwt_exp_ms` returns None for opaque tokens; add `expiresAt` fallback |
| BUG-175 | BUG-175: `Some(paths)` branch called `switch_account` before reading creds — unnecessary global write; removed |
| BUG-208 | BUG-208 (Closed): restore `switch_account` calls wrapped in `let _ = ...` — silent error discard; `match` arms added for trace; superseded by BUG-211 |
| BUG-211 | BUG-211 (Fixed): snapshot+restore removed from `apply_refresh`; `save(update_marker=false)` suppresses `_active` writes during per-account cycling |
| BUG-221 | BUG-221 (Fixed 2026-05-30, TSK-230): `Some(paths)` branch writes refreshed creds directly to credential store; `save()` gained `creds: Option<&[u8]>` — live session file no longer written during batch refresh |
| BUG-297 🟢 Fixed (TSK-307) | `apply_refresh` else-continue branch sets `aq.result = Err("refresh token expired")` before `continue;` — prevents redundant `apply_touch` subprocess for RT-expired accounts; fix = AC-30 |
| BUG-298 🟢 Fixed | `apply_refresh` trace emits constant `reason: ok` for cached+owned+expired accounts — `fetch.rs` cache fallback converts Err→Ok destroying original error; fix: `else if aq.cached` branch added before `aq.result.err()` at trace reason computation in `refresh.rs` — emits `"cached-expired"` when token is locally expired, `"cached"` when token is still valid (rate-limited) |
| BUG-303 🟢 Fixed (TSK-310) | `should_refresh()` G2 gate at `refresh_predicate.rs:32` checked `!aq.is_owned` only — no `is_occupied_elsewhere` guard; fix: `!aq.is_owned || aq.is_occupied_elsewhere` guard added; fix = AC-31 |
| BUG-316 🟢 Fixed (TSK-321) | `refresh_token_with_live_path` stale `is_active` guard — boolean computed once before `run_isolated`, reused in race-recovery block 35s later; concurrent `switch_account("B")` during subprocess window causes B's live credentials to be written into A's store slot; fix = AC-33 re-read at race-recovery site |

### Commands

| File | Relationship |
|------|--------------|
| [command/006_usage.md](../cli/command/006_usage.md#command--9-usage) | `.usage` CLI command specification |

### Dependencies

| File | Relationship |
|------|--------------|
| `claude_runner_core` | `run_isolated()` — called by `refresh_account_token()` in `_core`; `IsolatedRunResult`, `RunnerError` types |
| `claude_quota` | `fetch_oauth_usage()` — quota HTTP transport; `QuotaError::HttpTransport` |

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Baseline `.usage` algorithm that this extends |
| [018_live_monitor.md](018_live_monitor.md) | Live monitor mode that composites with `refresh::` |
| [019_account_relogin.md](019_account_relogin.md) | Browser re-authentication fallback when `refresh_account_token` returns `credentials=None` |
| [036_account_ownership.md](036_account_ownership.md) | G2/G3: non-owned accounts return `false` from `should_refresh()` — no refresh subprocess spawned |
| [024_session_touch.md](024_session_touch.md) | Session touch — reuses `refresh_account_token()` subprocess infrastructure |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::` and `effort::` apply to refresh subprocesses |
| `claude_runner_core/docs/feature/004_run_isolated.md` | `run_isolated()` API contract |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/019_refresh.md](../cli/param/019_refresh.md) | `refresh::` parameter specification |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../cli/command/006_usage.md#command--9-usage) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/refresh.rs` | `refresh::` param read; retry trigger; calls `account::refresh_account_token()`; expiry derivation; retry fetch |
| `src/lib.rs` | `refresh::` parameter registration via `register_commands()` |
| `src/usage/api.rs` | `apply_post_switch_touch()` — calls `refresh_account_token()` per AC-34 / invariant 008 |
| `claude_profile_core/src/account.rs` | `refresh_account_token()` — `read credentials → run_isolated → write credentials → save` lifecycle; sole authorized caller of `run_isolated()` (invariant 008) |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/008_single_token_refresh_entry.md](../invariant/008_single_token_refresh_entry.md) | Invariant 008: all token refresh through `refresh_account_token()`; AC-32/AC-33/AC-34 implement this invariant |

### Subprocess Docs

| File | Relationship |
|------|-------------|
| [subprocess/001_run_isolated_contract.md](../subprocess/001_run_isolated_contract.md) | `run_isolated()` API — signature, isolation mechanism, `IsolatedRunResult`, `RunnerError` |
| [subprocess/002_credential_writeback.md](../subprocess/002_credential_writeback.md) | Credential write-back protocol — RT rotation, live-file safety, expiry derivation |
| [subprocess/003_token_refresh_invocation.md](../subprocess/003_token_refresh_invocation.md) | `should_refresh()` predicate and post-refresh actions |
| [state_machine/002_oauth_token_lifecycle.md](../state_machine/002_oauth_token_lifecycle.md) | Token validity states and transitions |
| [pitfall/003_credential_sync_pitfalls.md](../pitfall/003_credential_sync_pitfalls.md) | BUG-162/170/208/211/221/310/316 (credential sync pitfalls) |
| [pitfall/002_subprocess_integration_pitfalls.md](../pitfall/002_subprocess_integration_pitfalls.md) | BUG-169 (`[]` args broken), BUG-243 (timeout output discard) |
