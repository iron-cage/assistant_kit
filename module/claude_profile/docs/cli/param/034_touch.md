# Parameter :: 34. `touch::`

Activate idle accounts' 5h session windows by sending a minimal prompt via `claude_profile_core::account::refresh_account_token()` in an isolated subprocess, then re-fetch quota to obtain the newly-set reset time. Trigger condition: `five_hour.resets_at` is absent (no active 5h session — account is idle).

- **Type:** `bool`
- **Default:** `1` (on — idle accounts are activated automatically)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` �� in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Starts a 5h session for idle accounts (those with no active countdown), making them eligible for endurance sort strategy and immediately available for use.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
touch::1   → for each idle account with valid quota and no active 5h session, send minimal prompt via isolated subprocess; re-fetch quota (default)
touch::0   → no subprocesses spawned; idle accounts remain idle
```

**Notes:**
- Trigger condition: account's quota fetch succeeded (valid token, no error) AND `five_hour.resets_at` is absent (idle — no active 5h session). Accounts with errored quota (expired token, auth failure) are never touched. Accounts with `resets_at` present (already active 5h window) are skipped — they already have a running session.
- Uses the same `account::refresh_account_token()` lifecycle as `refresh::` — `switch_account -> run_isolated(["--print", "."]) -> save`. The subprocess sends a minimal prompt to the Claude API, starting a new 5h session with `resets_at` set to ~5 hours from now.
- After the subprocess completes, quota is re-fetched for the touched account to obtain the newly-set `five_hour.resets_at` value.
- When both `refresh::1` and `touch::1` are active, refresh runs first (retries auth errors); touch runs second on post-refresh results. Accounts whose refresh already started a session are skipped by touch.
- In `live::1` mode, `touch::1` applies on every cycle. Accounts whose sessions expired since the last cycle (becoming idle) are re-activated.
- Each touch spawns an isolated subprocess (~35s timeout). With N idle accounts, this adds up to N * 35s. On by default; pass `touch::0` to suppress subprocess spawning when explicit control is needed.
- After all touch operations complete, the original active account is restored.
- `touch::` does not affect `format::json` output structure.

**See Also:** [feature/024_session_touch.md](../../feature/024_session_touch.md) for trigger conditions, algorithm, and AC criteria.
