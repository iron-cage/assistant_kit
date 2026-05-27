# Parameter :: 34. `touch::`

Activate idle accounts' 5h session windows by sending a minimal prompt via an isolated subprocess. Trigger condition: `five_hour.resets_at` is absent (no active 5h session — account is idle).

- **Type:** `bool`
- **Default:** `1` (on — idle accounts are activated automatically)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` �� in offline builds the parameter is accepted but has no effect
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage), [`.account.use`](../command/001_account.md#command--5-accountuse)
- **Purpose:** Starts a 5h session for idle accounts (those with no active countdown), making them eligible for endurance sort strategy and immediately available for use.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
touch::1   → idle account detected → send minimal prompt via isolated subprocess (default)
touch::0   → no subprocess spawned; idle accounts remain idle
```

**Notes:**
- Trigger condition: account's quota fetch succeeded (valid token, no error) AND `five_hour.resets_at` is absent (idle — no active 5h session). Accounts with errored quota (expired token, auth failure) are never touched. Accounts with `resets_at` present (already active 5h window) are skipped — they already have a running session.
- **On `.usage`:** Uses the same `account::refresh_account_token()` lifecycle as `refresh::` — `switch_account -> read credentials -> run_isolated(["--print", "."]) -> write credentials -> save`. After the subprocess completes, quota is re-fetched unconditionally for that account. After all touch operations complete, the original active account is restored. `touch::` does not affect `format::json` output structure.
- **On `.usage`:** When both `refresh::1` and `touch::1` are active, refresh runs first (retries auth errors); touch runs second on post-refresh results. Accounts whose refresh already started a session are skipped by touch.
- **On `.usage`:** In `live::1` mode, `touch::1` applies on every cycle. Accounts whose sessions expired since the last cycle (becoming idle) are re-activated.
- **On `.usage`:** Each touch spawns an isolated subprocess (~35s timeout). With N idle accounts, touch adds up to N × 35s.
- **On `.account.use`:** Touch applies to the single target account only (the just-switched-to account). Credentials are read directly from `{credential_store}/{name}.credentials.json`; `run_isolated(["--print", "."])` is called directly (no `refresh_account_token` lifecycle, no quota re-fetch, no account restore). The switch always completes regardless of touch outcome.

**See Also:** [feature/024_session_touch.md](../../feature/024_session_touch.md) for trigger conditions, algorithm, and AC criteria.
