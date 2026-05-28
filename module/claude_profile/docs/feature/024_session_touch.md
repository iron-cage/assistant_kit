# Feature: Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to activate idle 5h session windows by sending a minimal prompt in an isolated subprocess, so accounts with `five_hour.resets_at` absent (no active session) get a 5h session started, enabling them to qualify for endurance sort strategy and be available for immediate use.
- **Responsibility**: Documents the `touch::` parameter, its trigger condition (absent `five_hour.resets_at` — idle accounts with no active 5h window), subprocess invocation via the existing `refresh_account_token()` infrastructure, quota re-fetch after any successful subprocess, and skip-reason trace lines for non-qualifying accounts.
- **In Scope**: `touch::` parameter semantics; trigger condition (account has valid quota data AND `five_hour.resets_at` is absent — idle, no active 5h window); subprocess invocation via `account::refresh_account_token()` with `["--print", "."]`; quota re-fetch after any successful subprocess (unconditional on credentials); skip-reason trace line for non-qualifying accounts; interaction with `refresh::` and `live::`.
- **Out of Scope**: `run_isolated()` internals (-> `claude_runner_core/docs/feature/004_run_isolated.md`); the refresh trigger logic itself (-> 017_token_refresh.md); endurance qualification algorithm (-> 020_usage_sort_strategies.md); recommendation strategies (-> 023_next_account_strategies.md).

### Design

The `touch::` parameter takes `1`/`true` (on, default) or `0`/`false` (off). When `0`, `.usage` behaves identically to the baseline — idle accounts remain idle.

When `touch::1`, after the initial quota fetch the command identifies accounts whose `five_hour.resets_at` is absent (rendered as `—` in the `5h Reset` column — idle, no active session) AND whose quota fetch succeeded (valid token, no error). For each such account, it calls `account::refresh_account_token()` — the same lifecycle used by `refresh::` — which performs `read credentials → run_isolated(["--print", "."])  → write credentials → save`. The subprocess sends a minimal prompt to the Claude API, which starts a new 5h session and sets `resets_at` to ~5 hours from now. After the subprocess completes, the command re-fetches quota for that account to obtain the newly-set `five_hour.resets_at` value.

**Why this works:** The Anthropic API starts a 5h session window on any API call when no active window exists. Sending any prompt — even `"."` — transitions the account from idle (`resets_at = None`) to active (`resets_at` set to ~5h from now), making the account immediately available with a concrete session countdown.

**Why it matters for account rotation:** Activating idle accounts ensures they have a concrete `5h Reset` countdown, making them eligible for the endurance sort strategy (which requires `5h_reset ∈ [15m, 60m]`). Under `live::1`, touch runs each cycle to activate any newly-idle accounts, keeping the full pool available for rotation.

**Algorithm:**

```
results = fetch_all_quota(credential_store, live_creds_file)

if touch_param == 1:
    for each account_quota in results:
        if account_quota.result is Err
           OR account_quota.five_hour_resets_at is Some
           OR five_hour_left(account_quota) <= 15.0%:
            // Skip: no valid quota, already active 5h window, or h-exhausted
            if trace:
                emit "[trace] touch  <name>  skipped (reason: ...)"
            continue
        // Account qualifies: valid quota, idle (no active 5h window)
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

**Interaction with `live::`:** In `live::1` mode, `touch::1` applies on every cycle. If an account's session expires between cycles (its `resets_at` becomes absent), touch will re-activate it on the next cycle. Accounts that remain active (`resets_at` present) are not touched again — only newly-idle accounts are activated.

**Subprocess cost:** Each touch spawns an isolated Claude Code subprocess (~35s timeout). With N idle accounts, this adds up to N * 35s. This is acceptable for the common case (a few idle accounts) but can be slow when many accounts are idle simultaneously. The parameter is on by default (`touch::1`); pass `touch::0` to suppress subprocess spawning when explicit control is needed.

**Feature gate:** Same as refresh — `#[cfg(feature = "enabled")]`. When `enabled` is absent, `touch::1` is accepted but no subprocess is spawned.

### Acceptance Criteria

- **AC-01**: `touch::1` (default) invokes subprocess activation for accounts with `five_hour.resets_at` absent (idle, no active 5h window). `touch::0` produces no subprocess spawning; all accounts appear as-is.
- **AC-02**: `touch::1` invokes `account::refresh_account_token()` for each account whose quota fetch succeeded (`result is Ok`) AND `five_hour.resets_at` is absent (idle — no active 5h window). Accounts with `resets_at` present (already active) or with errored quota are skipped.
- **AC-03**: After a successful touch subprocess, quota is re-fetched unconditionally (regardless of whether the subprocess returned credentials). The table shows an updated `5h Reset` countdown value set to ~5h from the current time (account transitioned from idle `—` to active countdown).
- **AC-04**: Accounts with errored quota fetch (expired token, auth error, etc.) are never touched — the trigger requires a successful quota result with `five_hour.resets_at` absent.
- **AC-05**: When both `refresh::1` and `touch::1` are active, refresh runs first; touch runs on post-refresh results. An account whose refresh already started a session (`resets_at` now present) is skipped by touch.
- **AC-06**: After all touch operations complete, `apply_touch` does NOT call `switch_account`. The `_active` marker is not written during touch cycling (via `update_marker=false` in `save()` inside `refresh_account_token`), so no restore is needed. Fix for BUG-211.
- **AC-07**: If the touch subprocess fails, the account's row shows its original quota data unchanged (touch failure is non-aborting).
- **AC-08**: `touch::` does not affect `format::json` output structure — touched accounts appear as normal data objects with their re-fetched quota.
- **AC-09**: When `trace=true`, every account processed by `apply_touch` emits a `[trace] touch` line: touched accounts show subprocess lifecycle steps (`read credentials`, `run_isolated` with elapsed time, `write credentials`, `save`); accounts skipped because they do not qualify (already active, or errored) emit a `[trace] touch  <name>  skipped (reason: ...)` line.
- **AC-12**: When `trace=true`, accounts skipped by the touch trigger emit a skip-reason trace line covering all skip cases: `resets_at` present (already active 5h window — skip) and Err result (no valid quota). Both skip cases are diagnostically distinct and produce a trace line.
- **AC-10**: `touch::` parameter appears in `.usage --help` output with its default value (`1`).
- **AC-11**: In `live::1` mode with `touch::1` active, touch runs on every cycle. For each cycle where an account's `five_hour.resets_at` is absent (session expired since last cycle), the touch trigger fires and a new session is started. The trigger does not fire for accounts with `resets_at` present (still active).
- **AC-13**: No `switch_account` call occurs in `apply_touch`. The previous AC-13 restore trace is superseded — trace output for touch lifecycle steps remains unchanged (per AC-09), but no `restore switch_account` line is emitted. Fix for BUG-211.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `touch::` param read; idle-account detection (`resets_at` absent); subprocess call; re-fetch |
| source | `src/lib.rs` | `touch::` parameter registration via `register_commands()` |
| source | `claude_profile_core/src/account.rs` | `refresh_account_token()` — reused for touch |
| dep | `claude_runner_core` | `run_isolated()` — subprocess mechanism |
| dep | `claude_quota` | `fetch_oauth_usage()` — re-fetch after touch |
| doc | [009_token_usage.md](009_token_usage.md) | Base `.usage` algorithm that this extends |
| doc | [017_token_refresh.md](017_token_refresh.md) | Shared subprocess infrastructure and `refresh_account_token()` design |
| doc | [023_next_account_strategies.md](023_next_account_strategies.md) | Endurance strategy requires concrete `5h_reset` — motivation for touch |
| doc | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Endurance qualification: `5h_reset ∈ [15m, 60m]` + `weekly ≥ 30%` |
| invariant | [invariant/004_no_process_execution.md](../invariant/004_no_process_execution.md) | `claude_profile` delegates all process execution to `claude_runner_core` |
| param | [cli/param/034_touch.md](../cli/param/034_touch.md) | `touch::` parameter specification |
| param | [cli/param/019_refresh.md](../cli/param/019_refresh.md) | `refresh::` — runs before touch when both active |
| doc | [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch touch on `.account.use` — extends the touch concept to account switching |
| bug | `task/claude_profile/bug/208_restore_switch_account_silent_result_discard.md` | BUG-208: restore `switch_account` calls wrapped in `let _ = ...` — silent error discard, no `[trace]` line under `trace::1` |
| bug | `task/claude_profile/bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md` | BUG-211 (Fixed): snapshot+restore removed from `apply_touch`; `save(update_marker=false)` suppresses `_active` writes during per-account cycling |
