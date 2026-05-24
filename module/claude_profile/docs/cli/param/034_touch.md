# Parameter :: 34. `touch::`

Activate idle accounts whose 5h session window has not started (`five_hour.resets_at` absent, rendered as `5h Reset = —`) by sending a minimal prompt via `claude_profile_core::account::refresh_account_token()` in an isolated subprocess, then re-fetch quota to obtain a concrete reset time.

- **Type:** `bool`
- **Default:** `0` (off — idle accounts appear as-is with `5h Reset = —`)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` — in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Starts the 5h session quota timer on idle accounts so they gain a concrete `5h Reset` value and become eligible for endurance-strategy recommendation (which requires `5h_reset in [15m, 60m]`).
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
touch::1   → for each account with valid quota but 5h Reset = —, send minimal prompt via isolated subprocess; re-fetch quota
touch::0   → idle accounts appear as-is (default)
```

**Notes:**
- Trigger condition: account's quota fetch succeeded (valid token, no error) AND `five_hour.resets_at` is absent. Accounts with errored quota (expired token, auth failure) are never touched.
- Uses the same `account::refresh_account_token()` lifecycle as `refresh::` — `switch_account -> run_isolated(["--print", "."]) -> save`. The subprocess sends a minimal prompt to the Claude API, activating the 5h window.
- After the subprocess completes, quota is re-fetched for the touched account to obtain the now-active `five_hour.resets_at` value.
- When both `refresh::1` and `touch::1` are active, refresh runs first (retries auth errors); touch runs second on post-refresh results.
- In `live::1` mode, `touch::1` applies on every cycle. Once an account is touched, subsequent fetches return a concrete `resets_at`, so the trigger does not fire again until the 5h window fully resets and the account goes idle.
- Each touch spawns an isolated subprocess (~35s timeout). With N idle accounts, this adds up to N * 35s. Off by default to avoid unexpected subprocess spawning.
- After all touch operations complete, the original `_active` account is restored.
- `touch::` does not affect `format::json` output structure.

**See Also:** [feature/024_session_touch.md](../../feature/024_session_touch.md) for trigger conditions, algorithm, and AC criteria.
