# Commands :: Usage

Live quota utilization commands.

---

### Command :: 9. `.usage`

Fetches live quota utilization for every saved account via `claude_quota::fetch_oauth_usage()` (`GET /api/oauth/usage`) and account billing state via `claude_quota::fetch_oauth_account()` (`GET /api/oauth/account`, parallel thread). Renders results as a `data_fmt` table with a status emoji column (`●`: 🟢/🟡/🔴), plus 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, and → Next columns, and a footer recommendation line. `~Renews` shows a duration countdown (exact `in Xh Ym` when `_renewal_at` override is set, estimated `~in Xd` from `org_created_at`). `→ Next` shows the soonest strategic quota reset event (`+7d`/`$ren`); token expiry and 5h resets are not included since they are already shown in `Expires` and `5h Reset`. Supports optional token refresh on auth errors (`refresh::1`) and continuous live-monitor mode (`live::1`).

-- **Parameters:** [`format::`](../param/002_format.md), [`refresh::`](../param/019_refresh.md), [`live::`](../param/020_live.md), [`interval::`](../param/021_interval.md), [`jitter::`](../param/022_jitter.md), [`trace::`](../param/023_trace.md), [`sort::`](../param/025_sort.md), [`desc::`](../param/026_desc.md), [`prefer::`](../param/027_prefer.md), [`next::`](../param/032_next.md), [`cols::`](../param/033_cols.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md), [`count::`](../param/037_count.md), [`offset::`](../param/038_offset.md), [`only_active::`](../param/039_only_active.md), [`only_next::`](../param/040_only_next.md), [`min_5h::`](../param/041_min_5h.md), [`min_7d::`](../param/042_min_7d.md), [`only_valid::`](../param/043_only_valid.md), [`exclude_exhausted::`](../param/044_exclude_exhausted.md), [`get::`](../param/045_get.md), [`abs::`](../param/046_abs.md), [`no_color::`](../param/047_no_color.md)
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
| `imodel::` | `enum` | `auto` | Model for isolated subprocesses: `auto` (sonnet if `7d(Son)≥30%`, else opus), `sonnet`, `opus`, `haiku`, `keep` |
| `effort::` | `enum` | `auto` | Effort level for isolated subprocesses: `auto` (max for model: `high`/sonnet, `max`/opus, none/haiku), `low`, `normal`, `high`, `max` |
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

**Examples:**

```bash
clp .usage
# Quota
#
#   ●  Account              5h Left     5h Reset    7d Left  7d(Son)  7d Reset   Expires     ~Renews      → Next
#   🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%   28%      in 6d 14h  in 5h 02m   ~in 30d      +7d in 6d 14h
# ✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%   35%      in 4d 23h  in 7h 24m   ~in 6d       +7d in 4d 23h
# @ 🟢 carol@example.com    🟢 91%     in 4h 12m  🟢 73%   41%      in 5d 8h   in 2h 30m   ~in 14d      +7d in 5d 8h
# → 🟡 frank@example.com    🟡 3%      in 0h 23m  🟢 52%   18%      in 2d 11h  in 1h 12m   ~in 8d       +7d in 2d 11h
#   🔴 dave@example.com     —          —           —        —        —          EXPIRED      ?            —
#
# Valid: 4 / 5   ->  Next by strategy:
#   renew      frank@example.com   7d resets in 2d 11h, ~renews in 8d
#   endurance  bob@example.com     100% session, 88% 7d left, expires in 5h 02m
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
- `~Renews` shows an exact duration (`in Xh Ym`, no `~`) when `_renewal_at` is set in `{name}.claude.json` (via `.account.renewal`); shows an estimated `~in Xd` from `org_created_at` day-of-month when not set; shows `?` when neither source is available.
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
- `touch::` (default `1`) activates accounts with any quota timer absent (no active 5h, 7d, or 7d-Sonnet window) by sending a minimal prompt; pass `touch::0` to suppress. Runs after `refresh::` when both active. See [feature/024_session_touch.md](../../feature/024_session_touch.md) for full trigger conditions including skip guards (h-exhausted, 7d-exhausted).
- `imodel::` controls the Claude model injected into `touch::` and `refresh::` subprocesses. `auto` (default) selects Sonnet when an account's `7d(Son) ≥ 20%` and Opus otherwise. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).
- `effort::` controls the effort level (`--effort` flag) for those subprocesses. `auto` (default) uses `high` for Sonnet and `max` for Opus; no flag for `imodel::haiku` or `imodel::keep`. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).
