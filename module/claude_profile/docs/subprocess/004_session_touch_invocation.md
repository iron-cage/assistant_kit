# Subprocess: Session Touch Invocation

### Scope

- **Purpose**: Document when `.usage touch::` triggers `refresh_account_token()` to activate idle quota windows and the predicate (`apply_touch()`) controlling that decision.
- **Responsibility**: Authoritative reference for the session touch trigger predicate, skip-reason trace codes, invocation signature, post-touch actions, and ordering with `refresh::`.
- **In Scope**: `apply_touch()` predicate; skip-reason trace system; invocation via `refresh_account_token()`; post-touch re-fetch and cache write; ordering with `refresh::`.
- **Out of Scope**: Token refresh trigger predicate (→ `subprocess/003`); credential write-back protocol details (→ `subprocess/002`); model selection algorithm internals (→ `algorithm/001`).

### Purpose

Document when `.usage touch::` triggers `refresh_account_token()` to activate idle quota windows, and the exact predicate and skip-reason trace system.

### Trigger Predicate (`apply_touch()`)

An account is submitted for touch when ALL of the following are true:

| Condition | Rationale |
|-----------|-----------|
| `aq.result` is `Ok(data)` | Only valid-quota accounts are touched |
| `aq.is_owned == true` AND `aq.is_occupied_elsewhere == false` | G4 gate (BUG-302 fix) |
| NOT solo gate | `solo::1` skips non-current accounts |
| `touch_idle = false` cache flag absent (or cache absent) | Defense-in-depth: skip if subprocess already activated this account (AC-16; avoids API lag re-triggers) |
| At least one quota timer absent: | |
| `five_hour.resets_at == None` | 5h window not started |
| OR `seven_day` present AND `seven_day.resets_at == None` | 7d window not started |
| OR `seven_day_sonnet` present AND `seven_day_sonnet.resets_at == None` | 7d-Sonnet window not started |
| `five_hour_left(aq) > 15.0%` | Skip h-exhausted accounts |
| `seven_day_left(aq) > 0.0%` | Skip weekly-exhausted accounts |

Source: `src/usage/touch.rs`

### Skip-Reason Trace Codes

| Reason | Condition |
|--------|-----------|
| `reason: Err` | `aq.result` is `Err` |
| `reason: touch_idle=false` | Cache flag indicates already activated |
| `reason: all_running` | All three timers have active windows |
| `reason: h-exhausted` | `five_hour_left ≤ 15.0%` |
| `reason: weekly-exhausted` | `seven_day_left ≤ 0.0%` |
| `reason: not owned` | `is_owned = false` |
| `reason: occupied elsewhere` | `is_occupied_elsewhere = true` |
| `reason: solo-skip` | `solo::1` and not current account |

### Invocation

Same `refresh_account_token()` call as token refresh (with `update_marker=false`). Internally: `run_isolated(["--print", "."], model, timeout_secs=35)`.

Model selection uses [algorithm/001](../algorithm/001_touch_model_selection.md) — Sonnet when `son_idle=true` or `son_available=true` (prevents BUG-289 infinite loop where Haiku cannot open the 7d-Sonnet window).

### Post-Touch Actions

After any `refresh_account_token()` call (regardless of `credentials` result):
- Update `expires_at_ms` if `credentials = Some(new_json)`
- Re-fetch quota unconditionally: `account_quota.result = fetch_oauth_usage(new_token)`
- Write quota cache (Fix BUG-309)

### Ordering with `refresh::`

When both `refresh::1` and `touch::1` are active, `apply_refresh()` runs first (retries auth errors), then `apply_touch()` runs on the post-refresh results.

### Features

| File | Relationship |
|------|-------------|
| [feature/024_session_touch.md](../feature/024_session_touch.md) | Full feature spec |

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/001](../algorithm/001_touch_model_selection.md) | Model selection algorithm |

### Subprocess

| File | Relationship |
|------|-------------|
| [subprocess/001](001_run_isolated_contract.md) | `run_isolated()` contract |
| [subprocess/002](002_credential_writeback.md) | Credential write-back protocol |
| [subprocess/003](003_token_refresh_invocation.md) | Token refresh (same infrastructure) |
