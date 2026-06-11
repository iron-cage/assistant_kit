# Parameter :: 34. `touch::`

Activate idle quota windows by sending a minimal prompt via an isolated subprocess. Trigger condition: any of the three quota timers absent — `five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` — meaning no active session for that quota dimension.

- **Default:** `1` (on — idle accounts are activated automatically)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` �� in offline builds the parameter is accepted but has no effect
- **Purpose:** Starts a 5h session for idle accounts (those with no active countdown), making them eligible for endurance sort strategy and immediately available for use.

**Examples:**

```text
touch::1   → idle account detected → send minimal prompt via isolated subprocess (default)
touch::0   → no subprocess spawned; idle accounts remain idle
```

**Notes:**
- Trigger condition: account's quota fetch succeeded (valid token, no error) AND at least one quota timer is absent (`five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` = None). Accounts with errored quota (expired token, auth failure) are never touched. Accounts where all three timers are present (all windows active) are skipped. If the `seven_day` or `seven_day_sonnet` field is absent entirely (no weekly-quota tracking on the plan), that dimension is treated as running and does not trigger touch.
- **On `.usage`:** Uses the same `account::refresh_account_token()` lifecycle as `refresh::` — `read credentials -> run_isolated(["--print", "."]) -> write credentials -> save`. After the subprocess completes, quota is re-fetched unconditionally for that account. After all touch operations complete, the original active account is restored. `touch::` does not affect `format::json` output structure.
- **On `.usage`:** When both `refresh::1` and `touch::1` are active, refresh runs first (retries auth errors); touch runs second on post-refresh results. Accounts whose refresh already started a session are skipped by touch.
- **On `.usage`:** In `live::1` mode, `touch::1` applies on every cycle. Accounts whose sessions expired since the last cycle (becoming idle) are re-activated.
- **On `.usage`:** Each touch spawns an isolated subprocess (~35s timeout). With N idle accounts, touch adds up to N × 35s.
- **On `.account.use`:** Touch applies to the single target account only (the just-switched-to account). Credentials are read directly from `{credential_store}/{name}.credentials.json`; `run_isolated(["--print", "."])` is called directly (no `refresh_account_token` lifecycle, no quota re-fetch, no account restore). The switch always completes regardless of touch outcome.

**See Also:** [feature/024_session_touch.md](../../feature/024_session_touch.md) for trigger conditions, algorithm, and AC criteria.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Auto-activate idle quota windows during quota fetch |
| 2 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Activate idle session on switched-to account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Idle session activation enables immediate account use |
