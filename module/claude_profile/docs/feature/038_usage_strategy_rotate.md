# Feature: Usage Strategy Rotate

### Scope

- **Purpose**: Enable strategy-driven account rotation directly from `.usage` — the quota table and the switch action are a single command.
- **Responsibility**: Documents the `rotate::1` parameter on `.usage`: eligibility filtering, ownership gate, strategy selection via the active `sort::` algorithm, dry-run preview, and touch reuse from the already-fetched `AccountQuota`.
- **In Scope**: `rotate::` boolean param, G5 ownership gate on rotate path, strategy selection via `find_next_for_strategy()` (same `sort::` that drives the footer recommendation), post-switch touch from pre-fetched quota, `dry::1` preview, `force::1` bypass, mutual exclusion with `live::1`, no-eligible-account error.
- **Out of Scope**: The recommendation display (footer) and sort order — that is Feature 020. The deprecated `auto_rotate()` API and `.account.rotate` command — see [008_auto_rotate.md](008_auto_rotate.md) (deprecated).

### Design

Adding `rotate::1` to `.usage` merges account rotation into the quota-fetch pipeline. The `.usage` command already fetches `AccountQuota` for every account and runs `find_next_for_strategy()` to identify the recommended account. With `rotate::1`, that account is immediately activated after the table is rendered.

**Selection algorithm:**

`rotate::1` reuses `find_next_for_strategy(accounts, next_strategy, prefer, now_secs)` — the same function that selects the footer recommendation. The recommended account and the switched-to account are always the same account.

When `rotate::1` is active, `find_first_eligible` applies an additional ownership filter: only owned accounts (`aq.is_owned == true`) are eligible. This mirrors the G5 gate on `.account.use`. `force::1` bypasses this gate, allowing rotation to non-owned accounts.

**Execution order:**

```
1. Enumerate accounts, fetch quota (unchanged .usage pipeline steps 1-6)
2. find_next_for_strategy() → winner (or None)
3. Render table (unchanged)
4. (when rotate::1)
   a. If winner is None → exit 1: "no eligible account to rotate to"
   b. If dry::1     → append "[dry-run] would switch to '{name}'" → exit 0
   c. Ownership gate (G5): if !winner.is_owned && !force → exit 1
   d. switch_account(winner_name, credential_store, paths)
   d'. apply_model_override(winner.result.ok(), paths) — bidirectional: "opus" when Sonnet left < 15%; "sonnet" when >= 15% or tier absent (Fix BUG-311); writes effort unconditionally: "max" for Opus, "high" for Sonnet/absent-tier (TSK-335). Carry-forward set_session_effort() removed — apply_model_override() owns all effort writes.
   e. apply_touch(winner) — touch subprocess may refresh token (writes to STORE only)
   e'. Re-sync: copy winner's store credentials to live session (BUG-310 fix — AC-11)
   f. append "switched to '{name}'\n" to output
```

**Touch reuse:** When `.account.use` performs a post-switch touch it must first call `pre_switch_touch_ctx()` to fetch quota. With `.usage rotate::1`, the quota is already in `AccountQuota` from the main fetch — touch applies directly from the in-memory data, eliminating one API call.

**Ownership gate on `find_first_eligible`:**
When `rotate::1` (without `force::1`): the `extra` predicate passed to `find_first_eligible` includes `aq.is_owned`. When `rotate::1 force::1`: the ownership check is skipped. For display-only paths (`rotate::0`), no change — `find_first_eligible` does not filter by ownership.

**Behavioral difference from `.account.rotate` (removed):**
The former `.account.rotate` used `max_by_key(expires_at_ms)` — the account with the longest-lived OAuth token. `.usage rotate::1` defaults to `sort::renew` — the account whose quota renewal event fires soonest. This is intentional: the new behavior is operationally superior for quota-management automation.

### Acceptance Criteria

- **AC-01**: `rotate::1` switches to the account selected by the active `sort::` strategy (the recommended account). The footer shows the recommended account; the output ends with `switched to '{name}'`.
- **AC-02**: `rotate::1 dry::1` previews the target account with `[dry-run] would switch to '{name}'`; no credentials are written; exit 0.
- **AC-03**: When no eligible candidate exists for the active `sort::` strategy (all accounts are current, active, occupied, h-exhausted, or non-owned when `force::0`), `rotate::1` exits 1 with `"no eligible account to rotate to"`. The table is still rendered.
- **AC-04**: `rotate::1 live::1` exits 1 before any fetch with `"rotate::1 and live::1 are mutually exclusive"`.
- **AC-05**: `rotate::1` applies the G5 ownership gate to `find_first_eligible`: non-owned accounts (`aq.is_owned == false`) are skipped. A non-owned account is excluded from the recommendation and is never switched to.
- **AC-06**: `rotate::1 force::1` bypasses the G5 ownership gate: non-owned accounts are eligible for rotation (same bypass semantics as `.account.use force::1`).
- **AC-07**: `rotate::1 sort::renews` switches to the account with soonest billing renewal.
- **AC-08**: `rotate::1 format::json` still executes the switch; JSON output is unchanged (no `"switched_to"` field added to JSON).
- **AC-09**: Post-switch touch is applied using the winner's `AccountQuota` already in memory from the main fetch — no additional `GET /api/oauth/usage` call for the winner account.
- **AC-10**: Exit codes: 0 = switch succeeded (or dry-run); 1 = usage error (no eligible account, mutual exclusion, ownership violation); 2 = runtime error (credential store unreadable, switch I/O failure).
- **AC-11**: After `apply_touch(winner)` completes in the rotation dispatch (step 4e), the winner's credentials are re-synced from the persistent store to the live session file (`~/.claude/.credentials.json`). This ensures any token refresh performed by the touch subprocess (writes to STORE only via `refresh_account_token → save(update_marker=false)`) is reflected in the live session. Without re-sync, the live session retains pre-refresh credentials that may be server-invalidated. Fix(BUG-310).

### Migration from `.account.rotate`

| Old | New | Notes |
|-----|-----|-------|
| `clp .account.rotate` | `clp .usage rotate::1` | Default `sort::renew` (soonest renewal). Former default was `max_by_key(expires_at_ms)`. |
| `clp .account.rotate dry::1` | `clp .usage rotate::1 dry::1` | Same semantics. |
| `clp .account.rotate trace::1` | `clp .usage rotate::1 trace::1` | Same semantics. |
| *(no equivalent)* | `clp .usage rotate::1 sort::renews` | New: rotate to account with soonest billing renewal. |

### Bugs

| File | Relationship |
|------|--------------|
| BUG-310 🟢 Fixed | `api.rs:824` copies pre-touch store credentials to live via `switch_account(winner)`; `api.rs:838` `apply_touch(winner)` may refresh token writing to STORE only; live session retains stale pre-refresh credentials; fix = AC-11 — `fs::copy` at `api.rs:847` re-syncs store→live after `apply_touch` (TSK-318) |
| BUG-311 🟢 Fixed | `apply_model_override()` only wrote `"opus"`; no else-branch restored `"sonnet"` when winner had sufficient Sonnet quota. Rotation step d' retained stale `"opus"` after switching to a Sonnet-available account. Fix: bidirectional override (else-branch + tier-absent case) in `api.rs`. |
| BUG-312 🟢 Fixed | `effortLevel` was never written on fresh install or after `.account.use` — only `rotate::1` carry-forward wrote it. Footer omitted effort for all non-carry-forward paths. Fix: `apply_model_override()` (called in step d') now writes effort unconditionally in every branch: `"max"` for Opus, `"high"` for Sonnet/absent-tier (TSK-335). BUG-312 fallback guard retained as unreachable safety net. |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` command — carries the `rotate::` parameter |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/059_rotate.md](../cli/param/059_rotate.md) | `rotate::` parameter specification |
| [cli/param/025_sort.md](../cli/param/025_sort.md) | `sort::` controls which strategy selects the rotation target |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::1` bypasses G5 ownership gate on rotation |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::1` previews rotation target without switching |

### Features

| File | Relationship |
|------|--------------|
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies and footer recommendation — `find_next_for_strategy` reused for rotation target |
| [004_account_use.md](004_account_use.md) | `switch_account()` primitive called after strategy selection |
| [024_session_touch.md](024_session_touch.md) | Post-switch touch applied from in-memory `AccountQuota` |
| [036_account_ownership.md](036_account_ownership.md) | G5 ownership gate enforced on rotation eligibility |
| [008_auto_rotate.md](008_auto_rotate.md) | **DEPRECATED** predecessor; this feature replaces it |
| [061_solo_token_conservation.md](061_solo_token_conservation.md) | `rotate::1` is mutually exclusive with `solo::1` — both present exits 1 before fetch |
| [062_unified_session_config.md](062_unified_session_config.md) | Rotation dispatcher updated: calls `apply_model_override()` for the winner after switch — owns all effort writes (carry-forward removed TSK-335) |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/api.rs` | `usage_routine()` — rotation execution block (post-render) |
| `src/usage/sort_next.rs` | `find_next_for_strategy()` — strategy winner selection |
| `src/usage/params.rs` | `parse_usage_params()` — `rotate::` param parsing and `live::` mutual exclusion guard |
| `src/usage/types.rs` | `UsageParams.rotate` field |
| `src/usage/touch.rs` | Post-switch touch from pre-fetched quota |
| `claude_profile_core/src/account.rs` | `switch_account()` — credentials write + active marker update |

### Tests

| File | Relationship |
|------|--------------|
| [tests/docs/feature/038_usage_strategy_rotate.md](../../tests/docs/feature/038_usage_strategy_rotate.md) | Feature behavioral requirement test surface |
| [tests/docs/cli/command/09_usage.md](../../tests/docs/cli/command/09_usage.md) | Integration test cases for `rotate::` parameter group |

### Subprocess Docs

| File | Relationship |
|------|-------------|
| [subprocess/004_session_touch_invocation.md](../subprocess/004_session_touch_invocation.md) | `apply_touch()` trigger conditions — post-rotation touch from in-memory `AccountQuota` |
