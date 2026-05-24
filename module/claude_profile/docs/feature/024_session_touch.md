# Feature: Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to activate idle 5h session windows by sending a minimal prompt in an isolated subprocess, so accounts with `5h Reset = —` gain a concrete reset time and become eligible for endurance-strategy recommendation.
- **Responsibility**: Documents the `touch::` parameter, its trigger condition (missing `five_hour.resets_at`), subprocess invocation via the existing `refresh_account_token()` infrastructure, and quota re-fetch after activation.
- **In Scope**: `touch::` parameter semantics; trigger condition (account has valid quota data but `five_hour.resets_at` is absent); subprocess invocation via `account::refresh_account_token()` with `["--print", "."]`; quota re-fetch for touched accounts; active-account restore; interaction with `refresh::` and `live::`.
- **Out of Scope**: `run_isolated()` internals (-> `claude_runner_core/docs/feature/004_run_isolated.md`); the refresh trigger logic itself (-> 017_token_refresh.md); endurance qualification algorithm (-> 020_usage_sort_strategies.md); recommendation strategies (-> 023_next_account_strategies.md).

### Design

The `touch::` parameter takes `1`/`true` (on) or `0`/`false` (default, off). When `0`, `.usage` behaves identically to the baseline — accounts with `5h Reset = —` appear as-is.

When `touch::1`, after the initial quota fetch the command identifies accounts whose `five_hour.resets_at` is absent (rendered as `—` in the `5h Reset` column) AND whose quota fetch succeeded (valid token, no error). For each such account, it calls `account::refresh_account_token()` — the same lifecycle used by `refresh::` — which performs `switch_account → run_isolated(["--print", "."])  → save`. The subprocess sends a minimal prompt to the Claude API, which activates the account's 5h session window. After the subprocess completes, the command re-fetches quota for that account to obtain the now-active `five_hour.resets_at` value.

**Why this works:** The Anthropic API starts the 5h session quota window on the first API call in a period. Before any API call, `five_hour.resets_at` is absent from the usage response. Sending any prompt — even `"."` — constitutes an API call that activates the window and sets `resets_at` to ~5 hours from now.

**Why it matters for endurance:** The endurance strategy qualifies accounts whose `5h_reset ∈ [15m, 60m]` AND `weekly ≥ 30%`. Accounts with `5h Reset = —` can never satisfy the reset-range criterion and are permanently treated as unqualified. Touching them starts the timer, making them eventually eligible as the window ages into the [15m, 60m] qualification range.

**Algorithm:**

```
results = fetch_all_quota(credential_store, live_creds_file)

if touch_param == 1:
    original_active = read_file(credential_store / "_active")
    for each account_quota in results:
        if account_quota.result is Ok
           AND account_quota.five_hour_resets_at is None:
            // Account has valid quota but no active 5h window
            new_json = account::refresh_account_token(
                account_quota.name, credential_store, claude_paths, trace
            )
            if new_json is Some(json):
                // Update expiry same as refresh (JWT exp → expiresAt fallback)
                update_expiry(account_quota, json)
                account_quota.result = fetch_oauth_usage(new_token)
            // else: touch failed; account row unchanged

    if original_active is Some(name):
        account::switch_account(name, credential_store, claude_paths)

render results as table
```

**Ordering with `refresh::`:** When both `refresh::1` and `touch::1` are active, refresh runs first (it retries auth errors from the initial fetch). Touch runs second on the post-refresh results — an account that was refreshed and now shows valid quota with `resets_at = None` will be touched. An account that failed refresh (still errored) will not be touched (trigger requires successful quota data).

**Interaction with `live::`:** In `live::1` mode, `touch::1` applies on every cycle. However, once an account is touched, its subsequent fetches will return a concrete `resets_at` value, so the touch trigger will not fire again for that account until the 5h window fully resets and the account goes idle again.

**Subprocess cost:** Each touch spawns an isolated Claude Code subprocess (~35s timeout). With N idle accounts, this adds up to N * 35s. This is acceptable for the common case (1-3 idle accounts) but can be slow with many idle accounts. The parameter is off by default (`touch::0`) to avoid unexpected subprocess spawning.

**Feature gate:** Same as refresh — `#[cfg(feature = "enabled")]`. When `enabled` is absent, `touch::1` is accepted but no subprocess is spawned.

### Acceptance Criteria

- **AC-01**: `touch::0` (default) produces no subprocess spawning for session activation; accounts with `5h Reset = —` appear as-is.
- **AC-02**: `touch::1` invokes `account::refresh_account_token()` for each account whose quota fetch succeeded but `five_hour.resets_at` is absent.
- **AC-03**: After a successful touch, the account's quota is re-fetched and the table shows a concrete `5h Reset` value instead of `—`.
- **AC-04**: Accounts with errored quota fetch (expired token, auth error, etc.) are never touched — the trigger requires a successful quota result with missing `resets_at`.
- **AC-05**: When both `refresh::1` and `touch::1` are active, refresh runs first; touch runs on post-refresh results.
- **AC-06**: After all touch operations complete, the original `_active` account is restored.
- **AC-07**: If the touch subprocess fails, the account's row shows its original quota data unchanged (touch failure is non-aborting).
- **AC-08**: `touch::` does not affect `format::json` output structure — touched accounts appear as normal data objects with their re-fetched quota.
- **AC-09**: When `trace=true`, touch operations emit `[trace]` lines showing the subprocess lifecycle (same format as refresh trace output).
- **AC-10**: `touch::` parameter appears in `.usage --help` output with its default value (`0`).
- **AC-11**: In `live::1` mode with `touch::1` active, touch runs on every cycle. After a successful touch activates an account's 5h window, subsequent fetches return a concrete `resets_at` value; the idle trigger does not fire again for that account until the window fully resets and the account goes idle again.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `touch::` param read; idle-account detection; subprocess call; re-fetch |
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
