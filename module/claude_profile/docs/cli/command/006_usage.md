# Commands: Usage

Live quota utilization commands.

---

### Command: 9. `.usage`

Fetches live quota utilization for every saved account via `claude_quota::fetch_oauth_usage()` (`GET /api/oauth/usage`) and account billing state via `claude_quota::fetch_oauth_account()` (`GET /api/oauth/account`, parallel thread). Renders results as a `data_fmt` table with a status emoji column (`●`: 🟢/🟡/🔴), plus 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, and → Next columns, and a footer recommendation line. `~Renews` shows a duration countdown (exact `in Xh Ym` when `_renewal_at` override is set, estimated `~in Xd` from `org_created_at`). `→ Next` shows the soonest strategic quota reset event (`+7d`/`$ren`); token expiry and 5h resets are not included since they are already shown in `Expires` and `5h Reset`. Supports optional token refresh on auth errors (`refresh::1`) and continuous live-monitor mode (`live::1`).

-- **Parameters:** [`name::`](../param/001_name.md) *(optional)*, [`format::`](../param/002_format.md), [`dry::`](../param/004_dry.md), [`refresh::`](../param/019_refresh.md), [`live::`](../param/020_live.md), [`interval::`](../param/021_interval.md), [`jitter::`](../param/022_jitter.md), [`trace::`](../param/023_trace.md), [`sort::`](../param/025_sort.md), [`desc::`](../param/026_desc.md), [`prefer::`](../param/027_prefer.md), [`cols::`](../param/033_cols.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`count::`](../param/037_count.md), [`offset::`](../param/038_offset.md), [`only_active::`](../param/039_only_active.md), [`only_next::`](../param/040_only_next.md), [`min_5h::`](../param/041_min_5h.md), [`min_7d::`](../param/042_min_7d.md), [`only_valid::`](../param/043_only_valid.md), [`exclude_exhausted::`](../param/044_exclude_exhausted.md), [`get::`](../param/045_get.md), [`abs::`](../param/046_abs.md), [`no_color::`](../param/047_no_color.md), [`set_model::`](../param/054_set_model.md), [`assignee::`](../param/063_assignee.md), [`owner::`](../param/062_owner.md), [`force::`](../param/058_force.md), [`rotate::`](../param/059_rotate.md), [`solo::`](../param/060_solo.md), [`who::`](../param/061_who.md)
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
clp .usage sort::renew prefer::opus
clp .usage sort::renews
clp .usage sort::name desc::1
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
clp .usage rotate::1 sort::renews
clp .usage rotate::1 dry::1
clp .usage rotate::1 force::1
clp .usage solo::1
clp .usage solo::1 trace::1
clp .usage solo::1 live::1 interval::60
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/002_output_format.md) | `text` | Output format (`text` or `json`; `json` incompatible with `live::1`) |
| `refresh::` | `bool` | `1` | On 401/403 auth error or 429 with locally-expired token, refresh via isolated subprocess and retry |
| `live::` | `bool` | `0` | Enable continuous refresh loop (Ctrl-C to exit) |
| `interval::` | `u64` | `30` | Seconds between refresh cycles (≥ 30; only validated when `live::1`) |
| `jitter::` | `u64` | `0` | Max random seconds added to each cycle delay (≤ interval; only validated when `live::1`) |
| `trace::` | `bool` | `0` | Print timestamped diagnostic lines to stderr: credential reads, API calls, and refresh steps |
| `sort::` | `enum` | `renew` | Row ordering strategy AND footer recommendation: `name` (alphabetical), `renew` (soonest quota refill), `renews` (soonest billing renewal). Recommended account shown in footer `Next (strategy):` line |
| `desc::` | `bool` | context-sensitive | Sort direction; default depends on `sort::` strategy (`name`/`renew`/`renews`→`0`) |
| `prefer::` | `enum` | `any` | Weekly quota column for sort heuristics: `any` = `min(7d Left, 7d(Son))`, `opus` = `7d Left`, `sonnet` = `7d(Son)` |
| `cols::` | `string` | `""` | Column visibility modifiers: comma-separated `+col_id` / `-col_id` relative to default set |
| `touch::` | `bool` | `1` | Activate accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window) by sending minimal prompt via isolated subprocess; re-fetch quota |
| `imodel::` | `enum` | `auto` | Model for isolated subprocesses: `auto` (haiku — sufficient for keep-alive), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort level for isolated subprocesses: `auto` (`low` for any model; no flag for haiku or keep), `low`, `normal`, `high`, `max` |
| `count::` | `u64` | `0` | Maximum rows to display (0 = all rows) |
| `offset::` | `u64` | `0` | Skip first N rows from display |
| `only_active::` | `bool` | `0` | Show only the active (current/starred) account row |
| `only_next::` | `bool` | `0` | Show only the recommended next account row |
| `min_5h::` | `f64` | `0` | Hide accounts with `5h Left` below this percentage (0–100) |
| `min_7d::` | `f64` | `0` | Hide accounts with `7d Left` below this percentage (0–100) |
| `only_valid::` | `bool` | `0` | Hide accounts with invalid/missing tokens (status ≠ 🔴) |
| `exclude_exhausted::` | `bool` | `0` | Hide weekly-exhausted (🟡) and invalid (🔴) accounts |
| `get::` | `string` | `""` | Extract a single column value for the first matching row; implies `format::value`; valid field ids: `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `expires`, `renews`, `next_event_type`, `next_event_secs`, `sub`, `status`, `account`, `host`, `role` |
| `abs::` | `bool` | `0` | Show absolute token counts instead of percentages |
| `no_color::` | `bool` | `0` | Strip emoji and ANSI colors from output |
| `set_model::` | `enum` | *(omit)* | Explicitly write session model to `settings.json` for the current account: `opus`, `sonnet`, `haiku`, `default`; when provided, skips automatic `apply_model_override()` |
| `name::` | `string` | *(omit)* | Restrict mutation (`assignee::`, `owner::`) to the named account; comma-list `X,Y,Z` supported for `owner::` batch operations; when absent, `assignee::` unassigns marker, `owner::0` batch-clears all owned accounts |
| `dry::` | `bool` | `0` | Preview mutation result without writing to disk; G8 gate still runs on `owner::` mutations |
| `assignee::` | `string` (`USER@MACHINE` or `0`) | *(omit)* | When `name::` present: write per-machine marker `_active_{machine}_{user}` = `{name}`. When `name::` absent: clear marker for the given identity. Value `"0"` expands to `$USER@$HOSTNAME`. Value sanitized per `active_marker_filename()` rules (Feature 065; renamed from `active::`) |
| `owner::` | `string` | *(omit)* | `owner::0`: release ownership via `write_owner(name, store, "")`; G8 gate; batch-clear when `name::` absent. `owner::USER@MACHINE`: set ownership; G8 gate; `name::` required (comma-list supported). See Feature 063/064 |
| `force::` | `bool` | `0` | Bypass G8 ownership gate for `owner::0` and `owner::USER@MACHINE` when the account is owned by a different machine |
| `rotate::` | `bool` | `0` | After quota fetch and table render, switch to the recommended account (active `sort::` strategy winner); G5 ownership gate (non-owned accounts skipped; `force::1` bypasses); mutually exclusive with `live::1`; `dry::1` previews without switching |
| `solo::` | `bool` | `0` | Token conservation: restrict HTTP fetch, refresh, and touch to the current+owned account only; all others use `approximate_quota()` for historical data; mutually exclusive with `rotate::1` |
| `who::` | `bool` | auto | Sessions table visibility: auto (shown when >1 `_active_*` marker), `1` (force on), `0` (force off) |

**Algorithm (11 steps):**
1. Enumerate `{credential_store}/*.credentials.json` alphabetically; build account list
2. `(when assignee:: or owner:: present)` Mutation dispatch: `(when assignee::USER@MACHINE + name::X)` write per-machine marker (`assignee::0` expands to `$USER@$HOSTNAME`); `(when assignee::, no name::)` clear per-machine marker; `(when owner::0)` release ownership per-account with G8 gate (batch-clear when `name::` absent); `(when owner::USER@MACHINE)` set ownership per-account with G8 gate; comma-list `name::` supported for `owner::` operations; `(when dry::1)` print planned changes without writing
3. `(when only_active::1)` Pre-filter: retain only the `is_active` account (filesystem `_active_{hostname}_{user}` marker; no HTTP required)
4. `fetch_quota_for_list()`: call `GET /api/oauth/usage` per account; call `GET /api/oauth/account` in parallel thread. `(when solo::1)` only the current+owned account gets live HTTP fetch; all others get `approximate_quota()` (historical cached data via dedicated function — no direct cache file reads)
5. `(when refresh::1)` `apply_refresh()`: for 401/403 errors or 429 + locally-expired token, refresh via isolated subprocess and re-fetch. `(when solo::1)` refresh fires only for the current+owned account
6. `(when touch::1)` `apply_touch()`: for each account with any quota timer absent, spawn isolated subprocess to activate idle window; re-fetch quota (runs after refresh so refreshed accounts are touched). `(when solo::1)` touch fires only for the current+owned account
7. Session-model override: `(when set_model:: provided)` write requested model via `set_session_model()`; `(otherwise, when current account has valid quota)` write resolved model via `apply_model_override()`
8. Post-filter: apply `only_next::`, `only_valid::`, `exclude_exhausted::`, `min_5h::`, `min_7d::`, `count::`, `offset::` predicates
9. Compute derived fields: status emoji, `→ Next` column, `~Renews`, flag column priority (`✓`/`*`/`@`)
10. Four-group status partition (`🟢`→`🟡 h-exhausted`→`🟡 weekly-exhausted`→`🔴 Dead`); apply `sort::` strategy + `desc::` direction within each group
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
#   🟡 frank@example.com    🟡 3%      in 0h 23m  🟢 52%   12%      in 2d 11h  in 1h 12m   ~in 8d       in 2d 11h +7d
#   🔴 dave@example.com     —          —           —        —        —          EXPIRED      ?            —
#
# Current      · alice@example.com · sonnet/high · 4/5
# Next (renew) · frank@example.com · opus/max    · in 2d 11h +7d

clp .usage live::1 interval::60 jitter::10
# Quota
# ...table...
#
#   Next update in 0:59 (at 14:32:07 UTC)  [Ctrl-C to exit]
# (refreshes every 60–70 seconds; Ctrl-C exits cleanly)
```

**Notes:**
- Accounts are enumerated from `{credential_store}/*.credentials.json` in alphabetical order.
- Flag column priority: `✓` = current account, `*` = active-but-not-current (divergence), `@` = occupied on another machine (another machine's `_active_*` marker names this account). Priority: `✓` > `*` > `@` > blank. The recommended next account appears in the footer's `Next (strategy):` line, not in the flag column. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md) and [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md).
- Status emoji column (`●`): composite of 5h and 7d status — `🟢` = both available (`5h Left > 15%` and `7d Left > 5%`); `🟡` = h-exhausted (`5h Left ≤ 15%`, 7d available) or weekly-exhausted (`7d Left ≤ 5%`, any 5h — including both-exhausted; 7d is binding, per BUG-321 fix); `🔴` = error or cancelled subscription (`billing_type = "none"`). Per-column emoji also embedded in `5h Left` (🟢/🟡 at ≤15% threshold) and `7d Left` (🟢/🟡 at ≤5% threshold). No JSON equivalent.
- `Expires` is sourced from `expiresAt` in the credential file — available even when the API call fails.
- `Sub` is sourced from `GET /api/oauth/account` (parallel fetch); shows `?` when that fetch fails.
- `~Renews` shows an exact duration (`in Xh Ym`, no `~`) when `_renewal_at` is set in `{name}.json` (via `.account.renewal`); shows an estimated `~in Xd` from `org_created_at` day-of-month when not set; shows `?` when neither source is available.
- `→ Next` shows the soonest upcoming strategic event among 7d quota reset (`+7d`) and billing renewal (`$ren`). Token expiry (`!tok`) and 5h session resets are not candidates — already shown in `Expires` and `5h Reset`. Shows `—` when all candidates are absent or in the past.
- Accounts with failed quota fetch (expired/missing `accessToken`, 429 rate-limit, or other API error) show `—` for all quota columns (`5h Left` through `7d Reset`) with a shortened error reason replacing the **last visible quota column**. `Expires`, `Sub`, and `~Renews` are sourced independently and retain their values regardless of quota fetch failure.
- Footer: two `·`-delimited, column-aligned lines when ≥2 accounts have valid quota data: `Current · <name> · <model>/<effort> · N/N` (the `✓` account) and `Next (<strategy>) · <name> · <model>/<effort> · <metric>` (recommendation; effort is model-derived: `"max"` for Opus, `"high"` for Sonnet — TSK-335). The flag column shows `✓`, `*`, `@`, or blank.
- Sessions table: appended after the footer when >1 `_active_*` marker exists in the credential store. Shows `Session` (`{user}@{host}`) and `Account` columns. `who::0` suppresses; `who::1` forces on. See [feature/009_token_usage.md AC-33, AC-34](../../feature/009_token_usage.md).
- Empty credential store exits 0 with `(no accounts configured)`.
- `refresh::1` triggers at most one retry per account per cycle. See [feature/017_token_refresh.md](../../feature/017_token_refresh.md).
- `live::1 format::json` exits 1 before any fetch. See [feature/018_live_monitor.md](../../feature/018_live_monitor.md).
- Four-group status partition (🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Dead) applied before sort strategy. Both-exhausted accounts (5h ≤ 15% AND 7d ≤ 5%) merge into G3 weekly-exhausted. Sort applies within each group only; `desc::1` reverses within groups but never changes group order. See [dictionary](../../cli/002_dictionary.md#status-groups).
- `Sub` column hidden by default; show via `cols::+sub`. `7d Son Reset` column also hidden by default; show via `cols::+7d_son_reset`.
- Duration format (`format_duration_secs`) capped to 2 significant units (e.g., `1d 2h` not `1d 2h 45m`).
- See [feature/009_token_usage.md](../../feature/009_token_usage.md) for the baseline algorithm and AC criteria.
- See [feature/020_usage_sort_strategies.md](../../feature/020_usage_sort_strategies.md) for sort strategies and footer recommendation.
- `rotate::1` executes account switch to the footer-recommended account after rendering; mutually exclusive with `live::1` (exits 1 before fetch). G5 ownership gate applies — non-owned accounts are ineligible unless `force::1`. Post-switch touch reuses already-fetched `AccountQuota` (no extra API call). See [feature/038_usage_strategy_rotate.md](../../feature/038_usage_strategy_rotate.md).
- `touch::` (default `1`) activates accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window) by sending a minimal prompt; pass `touch::0` to suppress. Runs after `refresh::` when both active. See [feature/024_session_touch.md](../../feature/024_session_touch.md) for full trigger conditions including skip guards (h-exhausted, 7d-exhausted).
- `imodel::` controls the Claude model injected into `touch::` and `refresh::` subprocesses. `auto` (default) selects Haiku by default; Sonnet when `son_idle=true` (7d-Sonnet window present but not yet started — activates idle window). See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).
- `effort::` controls the effort level (`--effort` flag) for those subprocesses. `auto` (default) uses `low` for any model; no flag for `imodel::haiku` or `imodel::keep`. Low effort prevents extended thinking in keep-alive subprocesses, avoiding timeouts. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [name::](../param/001_name.md) | Restrict mutation to named account |
| 2 | [format::](../param/002_format.md) | Output format |
| 3 | [dry::](../param/004_dry.md) | Preview mutation without writing |
| 4 | [refresh::](../param/019_refresh.md) | Auto-refresh on auth error |
| 5 | [live::](../param/020_live.md) | Continuous refresh loop |
| 6 | [interval::](../param/021_interval.md) | Seconds between refresh cycles |
| 7 | [jitter::](../param/022_jitter.md) | Random delay added to interval |
| 8 | [trace::](../param/023_trace.md) | Diagnostic trace output |
| 9 | [sort::](../param/025_sort.md) | Row ordering strategy |
| 10 | [desc::](../param/026_desc.md) | Reverse sort direction |
| 11 | [prefer::](../param/027_prefer.md) | Weekly column for sort heuristics |
| 12 | [cols::](../param/033_cols.md) | Column visibility modifiers |
| 13 | [touch::](../param/034_touch.md) | Activate idle accounts via subprocess |
| 14 | [imodel::](../param/035_imodel.md) | Model for touch/refresh subprocesses |
| 15 | [effort::](../param/036_effort.md) | Effort level for subprocesses |
| 16 | [count::](../param/037_count.md) | Maximum rows to display |
| 17 | [offset::](../param/038_offset.md) | Skip first N rows |
| 18 | [only_active::](../param/039_only_active.md) | Show only active account row |
| 19 | [only_next::](../param/040_only_next.md) | Show only recommended next row |
| 20 | [min_5h::](../param/041_min_5h.md) | Hide rows below 5h percentage |
| 21 | [min_7d::](../param/042_min_7d.md) | Hide rows below 7d percentage |
| 22 | [only_valid::](../param/043_only_valid.md) | Hide invalid token rows |
| 23 | [exclude_exhausted::](../param/044_exclude_exhausted.md) | Hide exhausted rows |
| 24 | [get::](../param/045_get.md) | Extract single column value |
| 25 | [abs::](../param/046_abs.md) | Show absolute token counts |
| 26 | [no_color::](../param/047_no_color.md) | Strip emoji and ANSI colors |
| 27 | [set_model::](../param/054_set_model.md) | Explicitly write session model |
| 28 | [force::](../param/058_force.md) | Bypass G8 ownership gate |
| 29 | [rotate::](../param/059_rotate.md) | Execute account rotation after fetch |
| 30 | [solo::](../param/060_solo.md) | Token conservation mode |
| 31 | [who::](../param/061_who.md) | Sessions table visibility |
| 32 | [owner::](../param/062_owner.md) | Set or release account ownership |
| 33 | [assignee::](../param/063_assignee.md) | Write per-machine active marker |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Token Usage Reporting](../../feature/009_token_usage.md) | Baseline quota fetch algorithm and AC criteria |
| 2 | [Current Account Awareness](../../feature/016_current_account_awareness.md) | Flag column (`✓`/`*`) and active account detection |
| 3 | [Token Refresh](../../feature/017_token_refresh.md) | Auth error recovery on 401/403/429 |
| 4 | [Live Monitor](../../feature/018_live_monitor.md) | Continuous refresh loop behavior (`live::1`) |
| 5 | [Sort Strategies](../../feature/020_usage_sort_strategies.md) | Row ordering strategies and footer recommendation (`sort::`, `desc::`, `prefer::`) |
| 6 | [Session Touch](../../feature/024_session_touch.md) | Idle account activation trigger conditions |
| 7 | [Per-Machine Active Marker](../../feature/025_per_machine_active_marker.md) | Machine-local active marker (`@` flag column) |
| 8 | [Subprocess Model/Effort](../../feature/026_subprocess_model_effort.md) | Model and effort selection for subprocesses |
| 9 | [Row Filtering](../../feature/028_usage_row_filtering.md) | Filter predicates (`only_active::`, `min_5h::`, etc.) |
| 10 | [Account Renewal Override](../../feature/030_account_renewal_override.md) | `~Renews` exact duration when `_renewal_at` is set |
| 11 | [Usage Strategy Rotate](../../feature/038_usage_strategy_rotate.md) | `rotate::1` — strategy-driven account rotation via `.usage` |
| 12 | [Account Ownership](../../feature/036_account_ownership.md) | `solo::1` extends G1/G2/G4 ownership gates with current-account check |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Primary command for live quota monitoring across accounts |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Machine-readable quota data for automation scripts |

### Referenced Parameter Groups

| # | Group | Parameters Used |
|---|-------|-----------------|
| 1 | [Output Control](../param_group/001_output_control.md) | `format::`, `get::` |
| 2 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `refresh::`, `live::`, `interval::`, `jitter::`, `trace::`, `touch::`, `imodel::`, `effort::`, `solo::` |
| 3 | [Sort Control](../param_group/004_sort_control.md) | `sort::`, `desc::`, `prefer::` |
| 4 | [Display Control](../param_group/005_display_control.md) | `cols::`, `count::`, `offset::`, `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`, `exclude_exhausted::`, `abs::`, `no_color::` |

### Referenced Formats

| # | Format | Trigger | Note |
|---|--------|---------|------|
| 1 | [text](../format/001_text.md) | `format::text` (default) | — |
| 2 | [json](../format/002_json.md) | `format::json` | Incompatible with `live::1` |
| 3 | — value | `format::value` / `get::` | `.usage` only; implied by any `get::` field |
| 4 | — tsv | `format::tsv` | `.usage` only |
| 5 | — plain | `format::plain` | `.usage` only; equivalent to `no_color::1` |
