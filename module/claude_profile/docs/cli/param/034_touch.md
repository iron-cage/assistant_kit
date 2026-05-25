# Parameter :: 34. `touch::`

Keep active accounts' 5h session windows alive by sending a minimal prompt via `claude_profile_core::account::refresh_account_token()` in an isolated subprocess, then re-fetch quota to obtain the updated reset time. Trigger condition: `five_hour.resets_at` is present (an active 5h countdown exists).

- **Type:** `bool`
- **Default:** `1` (on — accounts with active 5h windows are touched automatically)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` — in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Extends the 5h session quota window for accounts with an active countdown, preventing mid-session expiry and keeping the account available for continued use.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
touch::1   → for each account with valid quota and an active 5h Reset countdown, send minimal prompt via isolated subprocess; re-fetch quota (default)
touch::0   → no subprocesses spawned; accounts appear as-is
```

**Notes:**
- Trigger condition: account's quota fetch succeeded (valid token, no error) AND `five_hour.resets_at` is present. Accounts with errored quota (expired token, auth failure) are never touched. Accounts with `resets_at` absent (idle, no active 5h window) are also not touched.
- Uses the same `account::refresh_account_token()` lifecycle as `refresh::` — `switch_account -> run_isolated(["--print", "."]) -> save`. The subprocess sends a minimal prompt to the Claude API, resetting the 5h countdown to ~5 hours from now.
- After the subprocess completes, quota is re-fetched for the touched account to obtain the updated `five_hour.resets_at` value.
- When both `refresh::1` and `touch::1` are active, refresh runs first (retries auth errors); touch runs second on post-refresh results.
- In `live::1` mode, `touch::1` applies on every cycle. Accounts with `resets_at` present are touched each cycle, continuously extending their 5h window.
- Each touch spawns an isolated subprocess (~35s timeout). With N active accounts, this adds up to N * 35s. On by default; pass `touch::0` to suppress subprocess spawning when explicit control is needed.
- After all touch operations complete, the original active account is restored.
- `touch::` does not affect `format::json` output structure.

**See Also:** [feature/024_session_touch.md](../../feature/024_session_touch.md) for trigger conditions, algorithm, and AC criteria.
