# Parameter :: 19. `refresh::`

Attempt a token refresh via `claude_profile_core::account::refresh_account_token()` and retry once before reporting failure; trigger and semantics vary by command — HTTP auth error (`.usage`, `.account.use`) or locally-expired `expiresAt` before endpoint calls (`.account.inspect`). See Purpose below for per-command details.

- **Type:** `bool`
- **Default:** `1` (on — expired tokens silently refreshed before reporting failure)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` — in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage), [`.account.use`](../command/001_account.md#command--5-accountuse), [`.account.inspect`](../command/001_account.md#command--15-accountinspect)
- **Purpose:** For `.usage`: silently recovers expired OAuth tokens on 401/403/429-with-expired-local-token errors, retrying the fetch once so the table shows current quota rather than error rows. For `.account.use` (`touch::1` path): when quota fetch fails and `expiresAt` is locally expired, attempts token refresh before refusing with exit 3; `refresh::0` refuses immediately without a retry. For `.account.inspect`: when `expiresAt` is locally expired, attempts token refresh before making endpoint 001/002/005 calls; `refresh::0` skips the refresh and all endpoints fall back to local snapshot data.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
refresh::1   → on 401/403 auth error, attempt token refresh via isolated subprocess, then retry once (default)
refresh::0   → auth errors appear as error rows in the table (explicit disable)
```

**Notes:**
- HTTP 401 and 403 always trigger a refresh attempt. HTTP 429 triggers a refresh only when the per-account credential file has a locally-expired `expiresAt` (`expiresAt / 1000 ≤ now`) — this recovers accounts where Claude Code updated the live session file but the saved per-account copy was never re-saved, leaving a stale token. HTTP 429 with a non-expired local token is passed through as-is (the token is valid; no refresh needed).
- The refresh may silently have no effect when: (a) the token is not actually server-expired (claude detects no need to refresh), (b) `run_isolated` times out before credentials are updated, or (c) the refreshToken itself is also expired. Use `trace::1` to see exactly which step stopped the refresh for each account.
- Network timeouts and other non-auth/non-ratelimit errors are not retried — they pass through as error rows in the table.
- Exactly one retry per account per invocation. If the retried fetch also fails, the final error is shown in the account's row.
- When `claude_paths` is available (normal runtime case), `account::save()` writes the new credentials directly to `{credential_store}/{account}.credentials.json` and updates companion files. `~/.claude/.credentials.json` is NOT written — the user's live session is preserved throughout the batch. (See [BUG-165](../../../../task/claude_profile/bug/165_apply_refresh_skips_account_lifecycle.md), [BUG-221](../../../../task/claude_profile/bug/221_refresh_account_token_some_branch_clobbers_live_credentials.md).)
- **`.account.use` context:** `refresh::` operates on the `touch::1` path only. When quota fetch fails AND `expiresAt` in the per-account credential file is locally expired (current time > `expiresAt / 1000`): `refresh::1` (default) calls `attempt_expired_token_refresh()` — on success re-probes touch context and continues the switch; on failure exits 3 with `account credentials expired and refresh failed: {name} (expired {N}h {M}m ago)`. `refresh::0` exits 3 immediately with `account credentials expired: {name} (expired {N}h {M}m ago)` without attempting a refresh. When `expiresAt` is absent or still in the future, the switch completes with touch skipped, regardless of `refresh::`. (Fix for BUG-213 extended by BUG-230; see AC-17 and AC-20 in [027_account_use_post_switch_touch.md](../../feature/027_account_use_post_switch_touch.md).)
