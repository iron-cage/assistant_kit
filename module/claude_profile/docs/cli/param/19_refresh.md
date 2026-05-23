# Parameter :: 19. `refresh::`

When an account's quota fetch returns an HTTP auth error (401 or 403), or an HTTP 429 rate-limit error when the per-account credential file has a locally-expired `expiresAt`, silently attempt a token refresh via `claude_profile_core::account::refresh_account_token()` and retry the fetch once before reporting failure.

- **Type:** `bool`
- **Default:** `1` (on — expired tokens silently refreshed before reporting failure)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` — in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](../command/usage.md#command--9-usage)
- **Purpose:** Allows `.usage` to silently recover expired OAuth tokens without requiring a manual `clp .account.use` rotation, so the table shows current quota rather than per-account auth error rows.
- **Group:** [Fetch Behavior](../param_group/03_fetch_behavior.md)

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
- When `claude_paths` is available (normal runtime case), the new credentials are written to the live session file (`~/.claude/.credentials.json`) first, then `account::save()` propagates them to `{credential_store}/{account}.credentials.json` and companion files atomically. (See [BUG-165](../../../../task/claude_profile/bug/165_apply_refresh_skips_account_lifecycle.md).)
