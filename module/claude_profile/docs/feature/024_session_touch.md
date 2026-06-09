# Feature: Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to activate idle quota windows by sending a minimal prompt in an isolated subprocess, so accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet session window) get sessions started, enabling them to qualify for endurance sort strategy and be available for immediate use.
- **Responsibility**: Documents the `touch::` parameter, its trigger condition (any of the three quota timers absent — `five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at`), subprocess invocation via the existing `refresh_account_token()` infrastructure, quota re-fetch after any successful subprocess, and skip-reason trace lines for non-qualifying accounts.
- **In Scope**: `touch::` parameter semantics; trigger condition (account has valid quota data AND at least one quota timer is absent — no active 5h, 7d, or 7d-Sonnet window); subprocess invocation via `account::refresh_account_token()` with `["--print", "."]`; quota re-fetch after any successful subprocess (unconditional on credentials); skip-reason trace line for non-qualifying accounts; interaction with `refresh::` and `live::`.
- **Out of Scope**: `run_isolated()` internals (-> `claude_runner_core/docs/feature/004_run_isolated.md`); the refresh trigger logic itself (-> 017_token_refresh.md); endurance qualification algorithm (-> 020_usage_sort_strategies.md); recommendation strategies (-> 023_next_account_strategies.md).

### Design

The `touch::` parameter takes `1`/`true` (on, default) or `0`/`false` (off). When `0`, `.usage` behaves identically to the baseline — idle accounts remain idle.

When `touch::1`, after the initial quota fetch the command identifies accounts where any of the three quota timers lacks an active window — `five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` absent — AND whose quota fetch succeeded (valid token, no error). For each such account, it calls `account::refresh_account_token()` — the same lifecycle used by `refresh::` — which performs `read credentials → run_isolated(["--print", "."])  → write credentials → save`. The subprocess sends a minimal prompt to the Claude API, which starts sessions for any inactive windows (5h, 7d, 7d-Sonnet) and sets their `resets_at` values. After the subprocess completes, the command re-fetches quota for that account to obtain the newly-set timer values.

**Why this works:** The Anthropic API starts session windows on any API call when those windows lack active timers. Sending any prompt — even `"."` — transitions absent timers from `resets_at = None` to active countdown values, making the account immediately available with concrete session tracking across all quota dimensions.

**Why it matters for account rotation:** Activating idle quota windows ensures accounts have concrete countdown timers for all dimensions (5h, 7d, 7d-Sonnet), making them eligible for the endurance sort strategy (which requires `5h_reset ∈ [15m, 60m]`). Under `live::1`, touch runs each cycle to activate any windows that lost their timers, keeping the full pool available for rotation.

**Algorithm:**

```
results = fetch_all_quota(credential_store, live_creds_file)

if touch_param == 1:
    for each account_quota in results:
        // Timer running checks:
        //   - five_hour field absent or its resets_at=None  → 5h timer not running
        //   - seven_day field present AND its resets_at=None → 7d timer not running
        //   - seven_day_sonnet field present AND its resets_at=None → 7d-Son timer not running
        //   - If seven_day/seven_day_sonnet field is absent entirely → treat as running (no timer to start)
        five_h_running   = five_hour.resets_at is Some
        seven_d_running  = seven_day field absent OR seven_day.resets_at is Some
        seven_ds_running = seven_day_sonnet field absent OR seven_day_sonnet.resets_at is Some
        all_running      = five_h_running AND seven_d_running AND seven_ds_running

        if account_quota.result is Err
           OR all_running
           OR five_hour_left(account_quota) <= 15.0%
           OR seven_day_left(account_quota) <= 0.0%:
            // Skip: no valid quota; all timers running; h-exhausted; or 7d-exhausted
            if trace:
                emit "[trace] touch  <name>  skipped (reason: ...)"
            continue
        // Account qualifies: valid quota, at least one quota timer absent (not fully active)
        // refresh_account_token internally calls save(update_marker=false) — _active marker never written
        new_json = account::refresh_account_token(
            account_quota.name, credential_store, claude_paths, trace
        )
        if new_json is Some(json):
            // Update expiry same as refresh (JWT exp → expiresAt fallback)
            update_expiry(account_quota, json)
        // Re-fetch quota unconditionally after any successful subprocess
        account_quota.result = fetch_oauth_usage(new_token)
        // (if re-fetch fails, account row shows pre-touch data)

render results as table
```

**Ordering with `refresh::`:** When both `refresh::1` and `touch::1` are active, refresh runs first (it retries auth errors from the initial fetch). Touch runs second on the post-refresh results — an account that was refreshed and now shows valid quota with no `resets_at` (idle) will be touched. An account that failed refresh (still errored) will not be touched. An account whose refresh subprocess already started a session (making `resets_at` present) is skipped by touch — refresh already activated it.

**Interaction with `live::`:** In `live::1` mode, `touch::1` applies on every cycle. If any quota timer lapses between cycles (its `resets_at` becomes absent), touch will re-activate it on the next cycle. Accounts where all three timers remain active are not touched again — only accounts with at least one newly-absent timer are re-activated.

**Subprocess cost:** Each touch spawns an isolated Claude Code subprocess (~35s timeout). With N idle accounts, this adds up to N * 35s. This is acceptable for the common case (a few idle accounts) but can be slow when many accounts are idle simultaneously. The parameter is on by default (`touch::1`); pass `touch::0` to suppress subprocess spawning when explicit control is needed.

**Feature gate:** Same as refresh — `#[cfg(feature = "enabled")]`. When `enabled` is absent, `touch::1` is accepted but no subprocess is spawned.

### Acceptance Criteria

- **AC-01**: `touch::1` (default) invokes subprocess activation for accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window). `touch::0` produces no subprocess spawning; all accounts appear as-is.
- **AC-02**: `touch::1` invokes `account::refresh_account_token()` for each account whose quota fetch succeeded (`result is Ok`) AND at least one quota timer is absent (`five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` = None — absent timer means no active session window for that quota dimension). Accounts where all three timers are present (all windows active) or with errored quota are skipped.
- **AC-03**: After a successful touch subprocess, quota is re-fetched unconditionally (regardless of whether the subprocess returned credentials). The table shows an updated `5h Reset` countdown value set to ~5h from the current time (account transitioned from idle `—` to active countdown).
- **AC-04**: Accounts with errored quota fetch (expired token, auth error, etc.) are never touched — the trigger requires a successful quota result with `five_hour.resets_at` absent.
- **AC-05**: When both `refresh::1` and `touch::1` are active, refresh runs first; touch runs on post-refresh results. An account whose refresh already started a session (`resets_at` now present) is skipped by touch.
- **AC-06**: After all touch operations complete, `apply_touch` does NOT call `switch_account`. The `_active` marker is not written during touch cycling (via `update_marker=false` in `save()` inside `refresh_account_token`), so no restore is needed. Fix for BUG-211.
- **AC-07**: If the touch subprocess fails, the account's row shows its original quota data unchanged (touch failure is non-aborting).
- **AC-08**: `touch::` does not affect `format::json` output structure — touched accounts appear as normal data objects with their re-fetched quota.
- **AC-09**: When `trace=true`, every account processed by `apply_touch` emits a `[trace] touch` line: touched accounts show subprocess lifecycle steps (`read credentials`, `run_isolated` with elapsed time, `write credentials`, `save`); accounts skipped because they do not qualify (already active, or errored) emit a `[trace] touch  <name>  skipped (reason: ...)` line.
- **AC-12**: When `trace=true`, accounts skipped by the touch trigger emit a skip-reason trace line covering all skip cases: all three timers running (already active — skip), h-exhausted (skip), 7d-exhausted (skip), and Err result (no valid quota). All skip cases are diagnostically distinct and produce a trace line.
- **AC-10**: `touch::` parameter appears in `.usage --help` output with its default value (`1`).
- **AC-11**: In `live::1` mode with `touch::1` active, touch runs on every cycle. For each cycle where any quota timer is absent (any session window lapsed since last cycle), the touch trigger fires and a new session is started. The trigger does not fire for accounts where all three timers are present (all windows still active).
- **AC-15**: Touch trigger checks all three quota window timers — `five_hour.resets_at`, `seven_day.resets_at`, and `seven_day_sonnet.resets_at`. An account qualifies for touch if any of these is absent (None), subject to the h-exhausted and 7d-exhausted skip guards. When the `seven_day` or `seven_day_sonnet` field itself is absent (account plan has no weekly-quota tracking), that dimension is treated as "running" and does not trigger touch — only a field-present timer with `resets_at = None` triggers.
- **AC-13**: No `switch_account` call occurs in `apply_touch`. The previous AC-13 restore trace is superseded — trace output for touch lifecycle steps remains unchanged (per AC-09), but no `restore switch_account` line is emitted. Fix for BUG-211.
- **AC-14**: Accounts with 7d weekly quota fully exhausted (`seven_day_left <= 0%`) are skipped by `apply_touch` even when idle (`five_hour.resets_at` absent) and 5h budget is non-zero. The Anthropic API does not open a new 5h session when the 7d budget is exhausted — spawning a subprocess for such accounts wastes wall time (~2.3s per account for a 429 rejection) and produces misleading `run_isolated: OK credentials=None` trace without establishing a session. Fixed in TSK-217 (Docker L3 ✅).

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/208_restore_switch_account_silent_result_discard.md` | BUG-208 (Closed): restore `switch_account` calls wrapped in `let _ = ...` — silent error discard; superseded by BUG-211 (restore removed from `apply_touch`) |
| `task/claude_profile/bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md` | BUG-211 (Fixed): snapshot+restore removed from `apply_touch`; `save(update_marker=false)` suppresses `_active` writes during per-account cycling |
| `task/claude_profile/bug/214_touch_fires_for_7d_exhausted_accounts.md` | BUG-214 (TSK-217, ✅ Fixed): `apply_touch` fired subprocess for 7d-exhausted accounts — fixed via `seven_day_left()` guard |
| `task/claude_profile/bug/215_touch_idle_detection_ignores_7d_and_7d_sonnet_timers.md` | BUG-215 (TSK-218, ✅ Fixed): `apply_touch` trigger checked only `five_hour.resets_at`; 7d and 7d-Sonnet timer absence was ignored — fixed via 3-timer `all_running` check |

### Dependencies

| File | Relationship |
|------|--------------|
| `claude_runner_core` | `run_isolated()` — subprocess mechanism |
| `claude_quota` | `fetch_oauth_usage()` — re-fetch after touch |

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm that this extends |
| [017_token_refresh.md](017_token_refresh.md) | Shared subprocess infrastructure and `refresh_account_token()` design |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Endurance qualification: `5h_reset ∈ [15m, 60m]` + `weekly ≥ 30%` |
| [023_next_account_strategies.md](023_next_account_strategies.md) | Endurance strategy requires concrete `5h_reset` — motivation for touch |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::` and `effort::` subprocess parameters apply to touch subprocesses |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch touch on `.account.use` — extends the touch concept to account switching |
| [033_quota_cache.md](033_quota_cache.md) | Quota cache — persists touch state (`last_touch_at`, `touch_idle`) |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/004_no_process_execution.md](../invariant/004_no_process_execution.md) | `claude_profile` delegates all process execution to `claude_runner_core` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/019_refresh.md](../cli/param/019_refresh.md) | `refresh::` — runs before touch when both active |
| [cli/param/034_touch.md](../cli/param/034_touch.md) | `touch::` parameter specification |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/touch.rs`, `src/usage/params.rs` | `touch::` param read; idle-account detection (`resets_at` absent); subprocess call; re-fetch |
| `src/lib.rs` | `touch::` parameter registration via `register_commands()` |
| `claude_profile_core/src/account.rs` | `refresh_account_token()` — reused for touch |
