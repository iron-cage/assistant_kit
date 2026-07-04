# Feature: Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to activate idle quota windows by sending a minimal prompt in an isolated subprocess, so accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet session window) get sessions started and are available for immediate use.
- **Responsibility**: Documents the `touch::` parameter, its trigger condition (any of the three quota timers absent — `five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at`), subprocess invocation via the existing `refresh_account_token()` infrastructure, quota re-fetch after any successful subprocess, `touch_idle` quota cache read guard as defense-in-depth skip before timer checks (AC-16), and skip-reason trace lines for non-qualifying accounts.
- **In Scope**: `touch::` parameter semantics; trigger condition (account has valid quota data AND at least one quota timer is absent — no active 5h, 7d, or 7d-Sonnet window); subprocess invocation via `account::refresh_account_token()` with `["--print", "."]`; quota re-fetch after any successful subprocess (unconditional on credentials); `touch_idle=false` quota cache read guard before timer checks (AC-16 — defense-in-depth for API propagation lag); skip-reason trace line for non-qualifying accounts; interaction with `refresh::` and `live::`.
- **Out of Scope**: `run_isolated()` internals (-> `claude_runner_core/docs/feature/004_run_isolated.md`); the refresh trigger logic itself (-> 017_token_refresh.md); sort strategies and footer recommendation (-> 020_usage_sort_strategies.md).

### Design

The `touch::` parameter takes `1`/`true` (on, default) or `0`/`false` (off). When `0`, `.usage` behaves identically to the baseline — idle accounts remain idle.

When `touch::1`, after the initial quota fetch the command identifies accounts where any of the three quota timers lacks an active window — `five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` absent — AND whose quota fetch succeeded (valid token, no error). For each such account, it calls `account::refresh_account_token()` — the same lifecycle used by `refresh::` — which performs `read credentials → run_isolated(["--print", "."])  → write credentials → save`. The subprocess sends a minimal prompt to the Claude API, which starts sessions for any inactive windows (5h, 7d, 7d-Sonnet) and sets their `resets_at` values. After the subprocess completes, the command re-fetches quota for that account to obtain the newly-set timer values.

**Why this works:** The Anthropic API starts session windows on any API call when those windows lack active timers. Sending any prompt — even `"."` — transitions absent timers from `resets_at = None` to active countdown values, making the account immediately available with concrete session tracking across all quota dimensions.

**Model-capability constraint (BUG-289):** The 5h window and 7d general window are model-agnostic — any model call (including Haiku) starts them. The 7d-Sonnet window (`seven_day_sonnet.resets_at`) is model-specific: only Sonnet-family API calls create a `seven_day_sonnet.resets_at` timestamp. When `imodel::auto` selects Haiku for a touch subprocess targeting an account where `son_running=false` (Sonnet window absent, 5h and 7d running), the Haiku subprocess does not start the Sonnet window — `resets_at` remains `None` after the re-fetch, `son_running` stays `false`, and the touch trigger fires again on the next invocation (infinite per-call no-op loop). Fix (BUG-289, BUG-290, TSK-292): `resolve_model(Auto)` selects Sonnet (`claude-sonnet-5`) whenever `son_idle=true` regardless of 5h or 7d timer state — a single Sonnet touch opens all idle dimensions simultaneously and breaks the loop. Implemented in `src/usage/subprocess.rs`.

**Why it matters for account rotation:** Activating idle quota windows ensures accounts have concrete countdown timers for all dimensions (5h, 7d, 7d-Sonnet), making them available for rotation under `sort::renew`. Under `live::1`, touch runs each cycle to activate any windows that lost their timers, keeping the full pool available for rotation.

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

        if account_quota.result is Err:
            // Skip: no valid quota
            if trace:
                emit "... · touch  <name>  skipped (reason: Err)"
            continue

        // Fix(BUG-288-FixB): defense-in-depth — read local touch_idle cache flag before timer checks
        if read_quota_cache(credential_store, name) is Some(cache) AND cache.touch_idle == Some(false):
            // Skip: subprocess already activated this account; local flag not subject to API lag
            if trace:
                emit "... · touch  <name>  skipped (reason: touch_idle=false)"
            continue

        if all_running
           OR five_hour_left(account_quota) <= 15.0%
           OR seven_day_left(account_quota) <= 0.0%:
            // Skip: all timers running; h-exhausted; or 7d-exhausted
            if trace:
                emit "... · touch  <name>  skipped (reason: ...)"
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

**Ownership gate (G4):** When account ownership is enabled (Feature 036), `apply_touch()` skips accounts where `aq.is_owned == false` (owned by a different machine) OR `aq.is_occupied_elsewhere == true` (owned by this machine but another machine is actively using this account — BUG-302 fix). When `trace::1`, emits `... · touch  <name>  skipped (reason: not owned)` or `... · touch  <name>  skipped (reason: occupied elsewhere)` respectively. Spawning a touch subprocess on an account actively used by another machine creates a competing subprocess that contends with the remote session.

**Subprocess cost:** Each touch spawns an isolated Claude Code subprocess (~35s timeout). With N idle accounts, this adds up to N * 35s. This is acceptable for the common case (a few idle accounts) but can be slow when many accounts are idle simultaneously. The parameter is on by default (`touch::1`); pass `touch::0` to suppress subprocess spawning when explicit control is needed.

**Feature gate:** Same as refresh — `#[cfg(feature = "enabled")]`. When `enabled` is absent, `touch::1` is accepted but no subprocess is spawned.

### Acceptance Criteria

- **AC-01**: `touch::1` (default) invokes subprocess activation for accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window). `touch::0` produces no subprocess spawning; all accounts appear as-is.
- **AC-02**: `touch::1` invokes `account::refresh_account_token()` for each account whose quota fetch succeeded (`result is Ok`) AND at least one quota timer is absent (`five_hour.resets_at`, `seven_day.resets_at`, or `seven_day_sonnet.resets_at` = None — absent timer means no active session window for that quota dimension). Accounts where all three timers are present (all windows active) or with errored quota are skipped.
- **AC-03**: After a successful touch subprocess, quota is re-fetched unconditionally (regardless of whether the subprocess returned credentials). This re-fetch requirement applies to ALL touch code paths — both `apply_touch` (this feature, `.usage`) and `apply_post_switch_touch` (Feature 027, `.account.use`). The table shows an updated `5h Reset` countdown value set to ~5h from the current time (account transitioned from idle `—` to active countdown). Fix(BUG-288): `apply_post_switch_touch` previously omitted this re-fetch, causing `.usage touch` to see stale quota and spawn a redundant second subprocess.
- **AC-04**: Accounts with errored quota fetch (expired token, auth error, etc.) are never touched — the trigger requires a successful quota result with `five_hour.resets_at` absent.
- **AC-05**: When both `refresh::1` and `touch::1` are active, refresh runs first; touch runs on post-refresh results. An account whose refresh already started a session (`resets_at` now present) is skipped by touch.
- **AC-06**: After all touch operations complete, `apply_touch` does NOT call `switch_account`. The `_active` marker is not written during touch cycling (via `update_marker=false` in `save()` inside `refresh_account_token`), so no restore is needed. Fix for BUG-211.
- **AC-07**: If the touch subprocess fails, the account's row shows its original quota data unchanged (touch failure is non-aborting).
- **AC-08**: `touch::` does not affect `format::json` output structure — touched accounts appear as normal data objects with their re-fetched quota.
- **AC-09**: When `trace=true`, every account processed by `apply_touch` emits a timestamped `touch` diagnostic line: touched accounts show subprocess lifecycle steps (`read credentials`, `run_isolated` with elapsed time, `write credentials`, `save`); accounts skipped because they do not qualify (already active, or errored) emit a `... · touch  <name>  skipped (reason: ...)` line.
- **AC-12**: When `trace=true`, accounts skipped by the touch trigger emit a timestamped skip-reason diagnostic line covering all skip cases: all three timers running (already active — skip), h-exhausted (skip), 7d-exhausted (skip), `touch_idle=false` (subprocess already activated — Fix B, BUG-288, see AC-16), Err result (no valid quota), `not owned` (G4 ownership gate — Feature 036), and `occupied elsewhere` (G4 occupancy guard — BUG-302 fix). All skip cases are diagnostically distinct and produce a trace line.
- **AC-17**: `apply_touch()` skips accounts where `aq.is_owned == false` (G4 ownership gate, Feature 036 AC-07) OR `aq.is_occupied_elsewhere == true` (G4 occupancy guard — BUG-302 fix). When `trace::1`, emits `... · touch  <name>  skipped (reason: not owned)` or `... · touch  <name>  skipped (reason: occupied elsewhere)` respectively. No subprocess is spawned for non-owned or currently-occupied accounts.
- **AC-10**: `touch::` parameter appears in `.usage --help` output with its default value (`1`).
- **AC-11**: In `live::1` mode with `touch::1` active, touch runs on every cycle. For each cycle where any quota timer is absent (any session window lapsed since last cycle), the touch trigger fires and a new session is started. The trigger does not fire for accounts where all three timers are present (all windows still active).
- **AC-15**: Touch trigger checks all three quota window timers — `five_hour.resets_at`, `seven_day.resets_at`, and `seven_day_sonnet.resets_at`. An account qualifies for touch if any of these is absent (None), subject to the h-exhausted and 7d-exhausted skip guards. When the `seven_day` or `seven_day_sonnet` field itself is absent (account plan has no weekly-quota tracking), that dimension is treated as "running" and does not trigger touch — only a field-present timer with `resets_at = None` triggers.
- **AC-13**: No `switch_account` call occurs in `apply_touch`. The previous AC-13 restore trace is superseded — trace output for touch lifecycle steps remains unchanged (per AC-09), but no `restore switch_account` line is emitted. Fix for BUG-211.
- **AC-14**: Accounts with 7d weekly quota fully exhausted (`seven_day_left <= 0%`) are skipped by `apply_touch` even when idle (`five_hour.resets_at` absent) and 5h budget is non-zero. The Anthropic API does not open a new 5h session when the 7d budget is exhausted — spawning a subprocess for such accounts wastes wall time (~2.3s per account for a 429 rejection) and produces misleading `run_isolated: OK credentials=None` trace without establishing a session. Fixed in TSK-217 (Docker L3 ✅).
- **AC-16**: `apply_touch` reads the quota cache (via `read_quota_cache`) after the error-account skip guard and before the `all_running` timer check. If the cache entry has `touch_idle = Some(false)` — the flag written by `apply_post_switch_touch` at `api.rs:330-332` after its subprocess activated the account — `apply_touch` skips that account without spawning a subprocess and emits `... · touch  <name>  skipped (reason: touch_idle=false)` when trace is enabled. Defense-in-depth for server-side quota propagation lag: even when Fix A's post-subprocess re-fetch returns `resets_at = None` (API hasn't propagated the new session yet), the local `touch_idle=false` flag prevents a redundant subprocess spawn. The `api.rs:330-332` write is no longer a dead write (zero read sites). Fix(BUG-288-FixB), TSK-291.
- **AC-18**: After `fetch_oauth_usage` succeeds in the `apply_touch` re-fetch block, `write_quota_cache()` is called to persist fresh quota data to `{name}.json`, `aq.cached` is set to `false`, and `aq.cache_age_secs` is set to `None`. These three mutations must appear before `aq.result = Ok(new_data)` — h5/d7/sn are extracted from `new_data` by reference before it is moved. Fix(BUG-309). Mirrors AC-03 of Feature 033 and the equivalent Fix(BUG-256) in `apply_refresh`.

### Bugs

| File | Relationship |
|------|--------------|
| BUG-208 | BUG-208 (Closed): restore `switch_account` calls wrapped in `let _ = ...` — silent error discard; superseded by BUG-211 (restore removed from `apply_touch`) |
| BUG-211 | BUG-211 (Fixed): snapshot+restore removed from `apply_touch`; `save(update_marker=false)` suppresses `_active` writes during per-account cycling |
| BUG-214 | BUG-214 (TSK-217, ✅ Fixed): `apply_touch` fired subprocess for 7d-exhausted accounts — fixed via `seven_day_left()` guard |
| BUG-215 | BUG-215 (TSK-218, ✅ Fixed): `apply_touch` trigger checked only `five_hour.resets_at`; 7d and 7d-Sonnet timer absence was ignored — fixed via 3-timer `all_running` check |
| BUG-288 | BUG-288 🟢 Fixed (Fix A + Fix B): Fix A — `apply_post_switch_touch` re-fetches quota post-subprocess via `write_quota_cache` (AC-03, Feature 027 AC-21). Fix B — `apply_touch` reads `touch_idle=false` cache flag before `all_running` check as defense-in-depth for API propagation lag; dead write at `api.rs:330-332` now has a read site at `touch.rs:59-66` (TSK-291, AC-16) |
| BUG-289 | BUG-289 🟢 Fixed (TSK-292): `resolve_model(Auto)` selects Sonnet when `son_idle=true`; `son_idle` gate in `src/usage/subprocess.rs` breaks the infinite per-call loop. |
| BUG-290 | BUG-290 🟢 Fixed: over-constrained BUG-289 gate (`five_h_running AND d7_running AND son_idle`) forced two-touch warm-up for cold accounts. Gate simplified to `son_idle` alone — single Sonnet touch opens 5h, 7d, and Son simultaneously. |
| BUG-302 🟢 Fixed (TSK-309) | `apply_touch` G4 gate checked `!aq.is_owned` only — no `is_occupied_elsewhere` guard; subprocess fired for owned+`@` accounts; fix: `is_occupied_elsewhere` guard added alongside G4 at `touch.rs:51-57`; fix = AC-17 |
| BUG-309 🟢 Fixed | `apply_touch` re-fetch block only set `aq.result = Ok(new_data)` — missing `write_quota_cache()`, `aq.cached = false`, `aq.cache_age_secs = None`; `{name}.json` retained stale pre-touch quota; cache-fallback accounts kept `~` markers after successful live re-fetch; fix = AC-18 |
| BUG-310 🟢 Fixed | Cross-feature: `api.rs:824` copies pre-touch store credentials to live; `api.rs:838` `apply_touch` may refresh token to STORE only; live session retains stale pre-refresh credentials. Primary feature: [038_usage_strategy_rotate.md AC-11](038_usage_strategy_rotate.md) — fixed by `fs::copy` at `api.rs:847` (TSK-318) |

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
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies reference quota timers populated by touch |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::` and `effort::` subprocess parameters apply to touch subprocesses |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch touch on `.account.use` — extends the touch concept to account switching |
| [033_quota_cache.md](033_quota_cache.md) | Quota cache — persists touch state (`last_touch_at`, `touch_idle`) |
| [036_account_ownership.md](036_account_ownership.md) | G4: non-owned accounts skipped by `apply_touch()` — ownership gate |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/004_no_process_execution.md](../invariant/004_no_process_execution.md) | `claude_profile` delegates all process execution to `claude_runner_core` |
| [invariant/008_single_token_refresh_entry.md](../invariant/008_single_token_refresh_entry.md) | Invariant 008: all token refresh through `refresh_account_token()` — touch reuses this entry point |

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

### Subprocess Docs

| File | Relationship |
|------|-------------|
| [subprocess/001_run_isolated_contract.md](../subprocess/001_run_isolated_contract.md) | `run_isolated()` API contract |
| [subprocess/002_credential_writeback.md](../subprocess/002_credential_writeback.md) | Credential write-back protocol |
| [subprocess/004_session_touch_invocation.md](../subprocess/004_session_touch_invocation.md) | Full trigger predicate table and skip-reason trace codes |
| [algorithm/001_touch_model_selection.md](../algorithm/001_touch_model_selection.md) | `resolve_model()` — Sonnet selection for `son_idle` accounts |
| [state_machine/003_session_window_lifecycle.md](../state_machine/003_session_window_lifecycle.md) | Session window idle/active/exhausted lifecycle |
| [pitfall/002_subprocess_integration_pitfalls.md](../pitfall/002_subprocess_integration_pitfalls.md) | BUG-289/290 (Haiku cannot open 7d-Sonnet window) |
