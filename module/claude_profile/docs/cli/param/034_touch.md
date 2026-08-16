# Parameter: 34. `touch::`

Activate idle quota windows by sending a minimal prompt via an isolated subprocess. Trigger condition: any of the three quota timers absent — `five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` — meaning no active session for that quota dimension.

- **Default:** `1` (on — idle accounts are activated automatically)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; effective only under `#[cfg(feature = "enabled")]` �� in offline builds the parameter is accepted but has no effect
- **Purpose:** Starts a 5h session for idle accounts (those with no active countdown), making them immediately available for use with concrete session timers.

**Examples:**

```text
touch::1   → idle account detected → send minimal prompt via isolated subprocess (default)
touch::0   → no subprocess spawned; idle accounts remain idle
```

**Notes:**
- Trigger condition: account's quota fetch succeeded (valid token, no error) AND at least one quota timer is absent (`five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` = None). Accounts with errored quota (expired token, auth failure) are never touched. Accounts where all three timers are present (all windows active) are skipped. If the `seven_day` or `seven_day_sonnet` field is absent entirely (no weekly-quota tracking on the plan), that dimension is treated as running and does not trigger touch.
- **Full skip-guard list** (`touch_skip_reason`, 6 guards checked in order): `solo::1` with the account not current (solo-skip) → account not owned (`is_owned = false`) → account occupied on another machine (`is_occupied_elsewhere`) → quota fetch errored (`result = Err`) → cached `touch_idle = false` (a subprocess already activated this account this cycle) → already-active (all three timers running), h-exhausted, or 7d-exhausted. The h/7d-exhaustion floor (Fix(TSK-418)) fires whenever `5h Left <= 0%` or `7d Left <= 0%`, even when a quota timer is absent — a fully-exhausted account is skipped regardless of whether it would otherwise look idle.
- **On `.usage`:** Uses the same `account::refresh_account_token()` lifecycle as `refresh::` — `read credentials -> run_isolated(["--print", "."]) -> write credentials -> save`. After the subprocess completes, quota is re-fetched unconditionally for that account. No account-restore step runs after touch operations complete — the snapshot+restore pattern was removed (Fix(BUG-211): the post-loop restore raced with concurrent `.account.use` switches during the ~35s subprocess window). `touch::` does not affect `format::json` output structure.
- **On `.usage`:** When both `refresh::1` and `touch::1` are active, refresh runs first (retries auth errors); touch runs second on post-refresh results. Accounts whose refresh already started a session are skipped by touch.
- **On `.usage`:** `touch::1` has no effect in `live::1` mode — the live-monitor loop (`execute_live_mode`) only fetches quota; it never calls the touch-apply routine. Idle accounts detected during `live::1` are not activated.
- **On `.usage`:** Each touch spawns an isolated subprocess (~35s timeout). With N idle accounts, touch adds up to N × 35s.
- **On `.account.use`:** Touch applies to the single target account only (the just-switched-to account), via `apply_post_switch_touch`. It routes through the same `refresh_account_token()` lifecycle as `refresh::`/`.usage` touch (AC-34/Invariant 008 — credential read, `run_isolated(["--print", "."])`, write, and save are all internal to that call), then performs a best-effort, non-aborting post-subprocess quota re-fetch to keep the cache consistent (AC-21). No account restore occurs — there is none to restore, since this runs against the account just switched to. The switch always completes regardless of touch outcome.

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
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Auto-activate idle quota windows during quota fetch |
| 2 | [`.account.use`](../command/001_account.md#command-5-accountuse) | Activate idle session on switched-to account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Idle session activation enables immediate account use |
