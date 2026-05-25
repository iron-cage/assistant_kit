# Feature: Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to keep active 5h session windows alive by sending a minimal prompt in an isolated subprocess, so accounts with `five_hour.resets_at` present get their 5h countdown extended and remain available for sustained use.
- **Responsibility**: Documents the `touch::` parameter, its trigger condition (present `five_hour.resets_at` AND `five_hour_left > 15%`), subprocess invocation via the existing `refresh_account_token()` infrastructure, quota re-fetch after any successful subprocess, and skip-reason trace lines for non-qualifying accounts.
- **In Scope**: `touch::` parameter semantics; trigger condition (account has valid quota data, `five_hour.resets_at` is present, and `five_hour_left > 15%`); subprocess invocation via `account::refresh_account_token()` with `["--print", "."]`; quota re-fetch after any successful subprocess (unconditional on credentials); skip-reason trace line for non-qualifying accounts; active-account restore; interaction with `refresh::` and `live::`.
- **Out of Scope**: `run_isolated()` internals (-> `claude_runner_core/docs/feature/004_run_isolated.md`); the refresh trigger logic itself (-> 017_token_refresh.md); endurance qualification algorithm (-> 020_usage_sort_strategies.md); recommendation strategies (-> 023_next_account_strategies.md).

### Design

The `touch::` parameter takes `1`/`true` (on, default) or `0`/`false` (off). When `0`, `.usage` behaves identically to the baseline — accounts with `5h Reset = —` appear as-is.

When `touch::1`, after the initial quota fetch the command identifies accounts whose `five_hour.resets_at` is present (rendered as an active countdown in the `5h Reset` column) AND whose quota fetch succeeded (valid token, no error). For each such account, it calls `account::refresh_account_token()` — the same lifecycle used by `refresh::` — which performs `switch_account → run_isolated(["--print", "."])  → save`. The subprocess sends a minimal prompt to the Claude API, which resets the account's 5h session countdown to ~5 hours from now. After the subprocess completes, the command re-fetches quota for that account to obtain the updated `five_hour.resets_at` value.

**Why this works:** The Anthropic API resets the 5h session quota window on any API call made while the window is active. Sending any prompt — even `"."` — pushes `resets_at` forward to ~5 hours from the current time, extending the available session window.

**Why it matters for sustained sessions:** Touching an account with an active 5h window prevents the quota from expiring mid-session. Under `live::1`, touch runs each cycle for all accounts with `resets_at` present, continuously extending their windows and ensuring accounts remain available for long-running agent sessions.

**Algorithm:**

```
results = fetch_all_quota(credential_store, live_creds_file)

if touch_param == 1:
    original_active = read_file(credential_store / active_marker_filename())
    for each account_quota in results:
        if account_quota.result is Err
           OR account_quota.five_hour_resets_at is None
           OR five_hour_left(account_quota) <= 15.0%:
            // Skip: no valid quota, no active 5h window, or h-exhausted
            if trace:
                emit "[trace] touch  <name>  skipped (reason: ...)"
            continue
        // Account qualifies: valid quota, active 5h window, not h-exhausted
        new_json = account::refresh_account_token(
            account_quota.name, credential_store, claude_paths, trace
        )
        if new_json is Some(json):
            // Update expiry same as refresh (JWT exp → expiresAt fallback)
            update_expiry(account_quota, json)
        // Re-fetch quota unconditionally after any successful subprocess
        account_quota.result = fetch_oauth_usage(new_token)
        // (if re-fetch fails, account row shows pre-touch data)

    if original_active is Some(name):
        account::switch_account(name, credential_store, claude_paths)

render results as table
```

**Ordering with `refresh::`:** When both `refresh::1` and `touch::1` are active, refresh runs first (it retries auth errors from the initial fetch). Touch runs second on the post-refresh results — an account that was refreshed and now shows valid quota with `resets_at` present will be touched. An account that failed refresh (still errored) will not be touched (trigger requires successful quota data with `resets_at` present).

**Interaction with `live::`:** In `live::1` mode, `touch::1` applies on every cycle. Once an account is touched, its subsequent fetches will return a refreshed `resets_at` value extended ~5h forward; the touch trigger will continue to fire on subsequent cycles as long as `resets_at` remains present.

**Subprocess cost:** Each touch spawns an isolated Claude Code subprocess (~35s timeout). With N accounts that have active 5h windows, this adds up to N * 35s. This is acceptable for the common case (1-3 accounts) but can be slow with many simultaneously active accounts. The parameter is on by default (`touch::1`); pass `touch::0` to suppress subprocess spawning when explicit control is needed.

**Feature gate:** Same as refresh — `#[cfg(feature = "enabled")]`. When `enabled` is absent, `touch::1` is accepted but no subprocess is spawned.

### Acceptance Criteria

- **AC-01**: `touch::1` (default) invokes subprocess activation for accounts with `five_hour.resets_at` present (active 5h window). `touch::0` produces no subprocess spawning; all accounts appear as-is.
- **AC-02**: `touch::1` invokes `account::refresh_account_token()` for each account whose quota fetch succeeded (`result is Ok`), `five_hour.resets_at` is present (active 5h window), AND `five_hour_left > 15%` (not h-exhausted). Accounts failing any of these three conditions are skipped.
- **AC-03**: After a successful touch subprocess, quota is re-fetched unconditionally (regardless of whether the subprocess returned credentials). The table shows an updated `5h Reset` countdown value extended ~5h forward from the current time.
- **AC-04**: Accounts with errored quota fetch (expired token, auth error, etc.) are never touched — the trigger requires a successful quota result with `five_hour.resets_at` present.
- **AC-05**: When both `refresh::1` and `touch::1` are active, refresh runs first; touch runs on post-refresh results.
- **AC-06**: After all touch operations complete, the original active account is restored.
- **AC-07**: If the touch subprocess fails, the account's row shows its original quota data unchanged (touch failure is non-aborting).
- **AC-08**: `touch::` does not affect `format::json` output structure — touched accounts appear as normal data objects with their re-fetched quota.
- **AC-09**: When `trace=true`, every account processed by `apply_touch` emits a `[trace] touch` line: touched accounts show subprocess lifecycle with per-step elapsed time (switch_account duration, run_isolated duration); accounts skipped because they do not qualify (inactive, h-exhausted, or errored) emit a `[trace] touch  <name>  skipped (reason: ...)` line.
- **AC-12**: When `trace=true`, accounts skipped by the touch trigger emit a skip-reason trace line covering all skip cases: `resets_at = None` (no active 5h window) AND `five_hour_left ≤ 15%` (h-exhausted with active reset timer). Both skip cases are diagnostically distinct and produce a trace line.
- **AC-10**: `touch::` parameter appears in `.usage --help` output with its default value (`1`).
- **AC-11**: In `live::1` mode with `touch::1` active, touch runs on every cycle. For each cycle where an account's `five_hour.resets_at` is present, the touch trigger fires and the subprocess extends the 5h window. The trigger does not fire for accounts with `resets_at` absent.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `touch::` param read; active-account detection (`resets_at` present); subprocess call; re-fetch |
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
