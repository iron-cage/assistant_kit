# Commands :: Usage

Live quota utilization commands.

---

### Command :: 9. `.usage`

Fetches live quota utilization for every saved account via `claude_quota::fetch_oauth_usage()` (`GET /api/oauth/usage`) and account billing state via `claude_quota::fetch_oauth_account()` (`GET /api/oauth/account`, parallel thread). Renders results as a `data_fmt` table with a status emoji column (`●`: 🟢/🟡/🔴), plus 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, and → Next columns, and a footer recommendation line. `~Renews` shows a duration countdown (exact `in Xh Ym` when `_renewal_at` override is set, estimated `~in Xd` from `org_created_at`). `→ Next` shows the soonest strategic quota reset event (`+7d`/`$ren`); token expiry and 5h resets are not included since they are already shown in `Expires` and `5h Reset`. Supports optional token refresh on auth errors (`refresh::1`) and continuous live-monitor mode (`live::1`).

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`format::`](../param/002_format.md), [`dry::`](../param/004_dry.md), [`refresh::`](../param/019_refresh.md), [`live::`](../param/020_live.md), [`interval::`](../param/021_interval.md), [`jitter::`](../param/022_jitter.md), [`trace::`](../param/023_trace.md), [`sort::`](../param/025_sort.md), [`desc::`](../param/026_desc.md), [`prefer::`](../param/027_prefer.md), [`next::`](../param/032_next.md), [`cols::`](../param/033_cols.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`count::`](../param/037_count.md), [`offset::`](../param/038_offset.md), [`only_active::`](../param/039_only_active.md), [`only_next::`](../param/040_only_next.md), [`min_5h::`](../param/041_min_5h.md), [`min_7d::`](../param/042_min_7d.md), [`only_valid::`](../param/043_only_valid.md), [`exclude_exhausted::`](../param/044_exclude_exhausted.md), [`get::`](../param/045_get.md), [`abs::`](../param/046_abs.md), [`no_color::`](../param/047_no_color.md), [`set_model::`](../param/054_set_model.md), [`unclaim::`](../param/056_unclaim.md), [`assign::`](../param/057_assign.md), [`force::`](../param/058_force.md), [`for::`](../param/053_for.md), [`rotate::`](../param/059_rotate.md)
-- **Exit:** 0 (success) | 1 (usage: invalid param combination) | 2 (runtime: credential store unreadable, HOME unset)

**Syntax:**

```bash
clp .usage
clp .usage format::json
clp .usage refresh::1
clp .usage live::1
clp .usage live::1 interval::60 jitter::10
clp .usage live::1 refresh::1 interval::60
clp .usage refresh::1 trace::1
clp .usage sort::endurance
clp .usage sort::drain prefer::sonnet
clp .usage sort::endurance desc::0
clp .usage sort::renew prefer::opus
clp .usage next::endurance
clp .usage next::renew
clp .usage next::drain
clp .usage sort::next
clp .usage cols::+sub
clp .usage cols::+sub,-7d_son
clp .usage touch::0
clp .usage touch::0 refresh::1 trace::1
clp .usage imodel::sonnet
clp .usage imodel::opus effort::max
clp .usage imodel::keep effort::high
clp .usage set_model::sonnet
clp .usage set_model::default
clp .usage rotate::1
clp .usage rotate::1 next::endurance
clp .usage rotate::1 next::drain
clp .usage rotate::1 dry::1
clp .usage rotate::1 force::1
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (`text` or `json`; `json` incompatible with `live::1`) |
| `refresh::` | `bool` | `1` | On 401/403 auth error or 429 with locally-expired token, refresh via isolated subprocess and retry |
| `live::` | `bool` | `0` | Enable continuous refresh loop (Ctrl-C to exit) |
| `interval::` | `u64` | `30` | Seconds between refresh cycles (≥ 30; only validated when `live::1`) |
| `jitter::` | `u64` | `0` | Max random seconds added to each cycle delay (≤ interval; only validated when `live::1`) |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr: credential reads, API calls, and refresh steps |
| `sort::` | `enum` | `renew` | Row ordering strategy: `renew` (soonest quota refill), `drain` (lowest weekly quota first), `name` (alphabetical), `endurance` (sustained session), `next` (mirrors active `next::` strategy), `expires` (soonest token expiry first), `renews` (soonest billing renewal first) |
| `desc::` | `bool` | context-sensitive | Sort direction; default depends on `sort::` strategy (`name`/`drain`/`renew`/`expires`/`renews`→`0`, `endurance`→`1`) |
| `prefer::` | `enum` | `any` | Weekly quota column for sort heuristics: `any` = `min(7d Left, 7d(Son))`, `opus` = `7d Left`, `sonnet` = `7d(Son)` |
| `next::` | `enum` | `renew` | Strategy placing `→` on recommended account: `renew`, `endurance`, `drain`; footer always shows renew, endurance, and drain recommendations regardless of `next::` value |
| `cols::` | `string` | `""` | Column visibility modifiers: comma-separated `+col_id` / `-col_id` relative to default set |
| `touch::` | `bool` | `1` | Activate accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window) by sending minimal prompt via isolated subprocess; re-fetch quota |
| `imodel::` | `enum` | `auto` | Model for isolated subprocesses: `auto` (haiku — sufficient for keep-alive), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort level for isolated subprocesses: `auto` (`low` for any model; no flag for haiku or keep), `low`, `normal`, `high`, `max` |
| `count::` | `u64` | `0` | Maximum rows to display (0 = all rows) |
| `offset::` | `u64` | `0` | Skip first N rows from display |
| `only_active::` | `bool` | `0` | Show only the active (current/starred) account row |
| `only_next::` | `bool` | `0` | Show only the recommended next account (`→` row) |
| `min_5h::` | `f64` | `0` | Hide accounts with `5h Left` below this percentage (0–100) |
| `min_7d::` | `f64` | `0` | Hide accounts with `7d Left` below this percentage (0–100) |
| `only_valid::` | `bool` | `0` | Hide accounts with invalid/missing tokens (status ≠ 🔴) |
| `exclude_exhausted::` | `bool` | `0` | Hide weekly-exhausted (🟡) and invalid (🔴) accounts |
| `get::` | `string` | `""` | Extract a single column value for the first matching row; implies `format::value`; valid field ids: `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, `next_event_type`, `next_event_secs`, `sub`, `status`, `account`, `host`, `role` |
| `abs::` | `bool` | `0` | Show absolute token counts instead of percentages |
| `no_color::` | `bool` | `0` | Strip emoji and ANSI colors from output |
| `set_model::` | `enum` | *(omit)* | Explicitly write session model to `settings.json` for the current account: `opus`, `sonnet`, `haiku`, `default`; when provided, skips automatic `apply_model_override()` |
| `name::` | `string` | *(omit)* | Restrict mutation (`assign::`, `unclaim::`) to the named account; when absent, mutation applies to all accounts (or the active account for `assign::1`) |
| `dry::` | `bool` | `0` | Preview mutation result without writing to disk; prints what would change |
| `unclaim::` | `bool` | `0` | Release ownership on the target account(s); clears ownership markers; requires `force::1` when the account is owned by another machine (G8 ownership gate) |
| `assign::` | `bool` | `0` | Write per-machine active marker for the target account on this machine (or the machine named by `for::`) |
| `force::` | `bool` | `0` | Bypass G8 ownership gate for `unclaim::1` when the account is owned by a different machine |
| `for::` | `string` | *(omit)* | Target `USER@MACHINE` identity for `assign::1`; defaults to current user@hostname when absent |
| `rotate::` | `bool` | `0` | After quota fetch and table render, switch to the `→` recommended account (active `next::` strategy winner); G5 ownership gate (non-owned accounts skipped; `force::1` bypasses); mutually exclusive with `live::1`; `dry::1` previews without switching |

**Algorithm (11 steps):**
1. Enumerate `{credential_store}/*.credentials.json` alphabetically; build account list
2. `(when assign::1 or unclaim::1)` Mutation dispatch: `(when assign::1)` write per-machine active marker for the named account (or `for::` target); `(when unclaim::1)` release ownership markers, subject to G8 gate (requires `force::1` when another machine owns the account); `(when dry::1)` print planned changes without writing
3. `(when only_active::1)` Pre-filter: retain only the `is_active` account (filesystem `_active_{hostname}_{user}` marker; no HTTP required)
4. `fetch_quota_for_list()`: call `GET /api/oauth/usage` per account; call `GET /api/oauth/account` in parallel thread
5. `(when refresh::1)` `apply_refresh()`: for 401/403 errors or 429 + locally-expired token, refresh via isolated subprocess and re-fetch
6. `(when touch::1)` `apply_touch()`: for each account with any quota timer absent, spawn isolated subprocess to activate idle window; re-fetch quota (runs after refresh so refreshed accounts are touched)
7. Session-model override: `(when set_model:: provided)` write requested model via `set_session_model()`; `(otherwise, when current account has valid quota)` write resolved model via `apply_model_override()`
8. Post-filter: apply `only_next::`, `only_valid::`, `exclude_exhausted::`, `min_5h::`, `min_7d::`, `count::`, `offset::` predicates
9. Compute derived fields: status emoji, `→ Next` column, `~Renews`, flag column priority (`✓`/`*`/`@`/`→`)
10. Three-tier sort (`🟢`→`🟡`→`🔴`); apply `sort::` strategy + `desc::` direction within each tier
11. `(when format::text)` Render table + footer; `(when get:: provided)` extract single field from first match; `(when live::1)` loop with `interval::` + `jitter::` delay
12. `(when rotate::1)` Rotation dispatch: call `find_next_for_strategy()` winner; if no winner → exit 1 (`"no eligible account to rotate to"`); if `dry::1` → append `"[dry-run] would switch to '{name}'"` and exit 0; apply G5 ownership gate (non-owned accounts exit 1 unless `force::1`); call `switch_account(winner)`; apply post-switch touch from in-memory `AccountQuota` (no re-fetch); append `"switched to '{name}'"` to output

**Examples:**

```bash
clp .usage
# Quota
#
#   ●  Account              5h Left     5h Reset    7d Left  7d(Son)  7d Reset   Expires     ~Renews      → Next
#   🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%   28%      in 6d 14h  in 5h 02m   ~in 30d      in 6d 14h +7d
# ✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%   35%      in 4d 23h  in 7h 24m   ~in 6d       in 4d 23h +7d
# @ 🟢 carol@example.com    🟢 91%     in 4h 12m  🟢 73%   41%      in 5d 8h   in 2h 30m   ~in 14d      in 5d 8h +7d
# → 🟡 frank@example.com    🟡 3%      in 0h 23m  🟢 52%   18%      in 2d 11h  in 1h 12m   ~in 8d       in 2d 11h +7d
#   🔴 dave@example.com     —          —           —        —        —          EXPIRED      ?            —
#
# Valid: 4 / 5   ->  Next by strategy:
#   renew      frank@example.com   7d resets in 2d 11h, ~renews in 8d
#   endurance  bob@example.com     100% session, 5h resets in 4h 58m
#   drain      bob@example.com     28% 7d left, 7d resets in 6d 14h

clp .usage live::1 interval::60 jitter::10
# Quota
# ...table...
#
#   Next update in 0:59 (at 14:32:07 UTC)  [Ctrl-C to exit]
# (refreshes every 60–70 seconds; Ctrl-C exits cleanly)
```

**Notes:**
- Accounts are enumerated from `{credential_store}/*.credentials.json` in alphabetical order.
- Flag column priority: `✓` = current account, `*` = active-but-not-current (divergence), `@` = occupied on another machine (another machine's `_active_*` marker names this account), `→` = recommended next account. Priority: `✓` > `*` > `@` > `→` > blank. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md) and [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md).
- Status emoji column (`●`): composite AND of 5h and 7d — `🟢` = valid token + `5h Left > 15%` and `7d Left > 5%`; `🟡` = valid token + either `5h Left ≤ 15%` or `7d Left ≤ 5%`; `🔴` = invalid/missing token. Per-column emoji also embedded in `5h Left` (🟢/🟡 at ≤15% threshold) and `7d Left` (🟢/🟡 at ≤5% threshold). No JSON equivalent.
- `Expires` is sourced from `expiresAt` in the credential file — available even when the API call fails.
- `Sub` is sourced from `GET /api/oauth/account` (parallel fetch); shows `?` when that fetch fails.
- `~Renews` shows an exact duration (`in Xh Ym`, no `~`) when `_renewal_at` is set in `{name}.json` (via `.account.renewal`); shows an estimated `~in Xd` from `org_created_at` day-of-month when not set; shows `?` when neither source is available.
- `→ Next` shows the soonest upcoming strategic event among 7d quota reset (`+7d`) and billing renewal (`$ren`). Token expiry (`!tok`) and 5h session resets are not candidates — already shown in `Expires` and `5h Reset`. Shows `—` when all candidates are absent or in the past.
- Accounts with failed quota fetch (expired/missing `accessToken`, 429 rate-limit, or other API error) show `—` for all quota columns (`5h Left` through `7d Reset`) with a shortened error reason replacing the **last visible quota column**. `Expires`, `Sub`, and `~Renews` are sourced independently and retain their values regardless of quota fetch failure.
- Footer: always shows one recommendation per strategy (renew, endurance, drain) when ≥2 accounts have valid quota data; `next::` controls only which account receives `→` in the table body.
- Empty credential store exits 0 with `(no accounts configured)`.
- `refresh::1` triggers at most one retry per account per cycle. See [feature/017_token_refresh.md](../../feature/017_token_refresh.md).
- `live::1 format::json` exits 1 before any fetch. See [feature/018_live_monitor.md](../../feature/018_live_monitor.md).
- Three-tier display grouping (🟢 → 🟡 → 🔴) applied before sort strategy within each tier. Within 🟡, h-exhausted accounts (`5h Left ≤ 15%`) appear before weekly-exhausted accounts (`5h Left > 15%`, `7d Left ≤ 5%`).
- `Sub` column hidden by default; show via `cols::+sub`. `7d Son Reset` column also hidden by default; show via `cols::+7d_son_reset`.
- Duration format (`format_duration_secs`) capped to 2 significant units (e.g., `1d 2h` not `1d 2h 45m`).
- See [feature/009_token_usage.md](../../feature/009_token_usage.md) for the baseline algorithm and AC criteria.
- See [feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md) for recommendation strategies.
- `rotate::1` executes account switch to the `→` winner after rendering; mutually exclusive with `live::1` (exits 1 before fetch). G5 ownership gate applies — non-owned accounts are ineligible unless `force::1`. Post-switch touch reuses already-fetched `AccountQuota` (no extra API call). See [feature/038_usage_strategy_rotate.md](../../feature/038_usage_strategy_rotate.md).
- `touch::` (default `1`) activates accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window) by sending a minimal prompt; pass `touch::0` to suppress. Runs after `refresh::` when both active. See [feature/024_session_touch.md](../../feature/024_session_touch.md) for full trigger conditions including skip guards (h-exhausted, 7d-exhausted).
- `imodel::` controls the Claude model injected into `touch::` and `refresh::` subprocesses. `auto` (default) selects Sonnet when an account's `7d(Son) ≥ 20%` and Opus otherwise. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).
- `effort::` controls the effort level (`--effort` flag) for those subprocesses. `auto` (default) uses `low` for any model; no flag for `imodel::haiku` or `imodel::keep`. Low effort prevents extended thinking in keep-alive subprocesses, avoiding timeouts. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Token Usage Reporting](../../feature/009_token_usage.md) | Baseline quota fetch algorithm and AC criteria |
| 2 | [Current Account Awareness](../../feature/016_current_account_awareness.md) | Flag column (`✓`/`*`) and active account detection |
| 3 | [Token Refresh](../../feature/017_token_refresh.md) | Auth error recovery on 401/403/429 |
| 4 | [Live Monitor](../../feature/018_live_monitor.md) | Continuous refresh loop behavior (`live::1`) |
| 5 | [Sort Strategies](../../feature/020_usage_sort_strategies.md) | Row ordering strategies (`sort::`, `desc::`, `prefer::`) |
| 6 | [Next Account Strategies](../../feature/023_next_account_strategies.md) | Recommendation strategy (`next::`) and footer display |
| 7 | [Session Touch](../../feature/024_session_touch.md) | Idle account activation trigger conditions |
| 8 | [Per-Machine Active Marker](../../feature/025_per_machine_active_marker.md) | Machine-local active marker (`@` flag column) |
| 9 | [Subprocess Model/Effort](../../feature/026_subprocess_model_effort.md) | Model and effort selection for subprocesses |
| 10 | [Row Filtering](../../feature/028_usage_row_filtering.md) | Filter predicates (`only_active::`, `min_5h::`, etc.) |
| 11 | [Account Renewal Override](../../feature/030_account_renewal_override.md) | `~Renews` exact duration when `_renewal_at` is set |
| 12 | [Usage Strategy Rotate](../../feature/038_usage_strategy_rotate.md) | `rotate::1` — strategy-driven account rotation via `.usage` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Primary command for live quota monitoring across accounts |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Machine-readable quota data for automation scripts |
