# Commands :: Usage

Live quota utilization commands.

---

### Command :: 9. `.usage`

Fetches live quota utilization for every saved account via `claude_quota::fetch_oauth_usage()` (`GET /api/oauth/usage`) and account billing state via `claude_quota::fetch_oauth_account()` (`GET /api/oauth/account`, parallel thread). Renders results as a `data_fmt` table with a status emoji column (`●`: 🟢/🟡/🔴), plus 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, Sub, and ~Renews columns, and a footer recommendation line. Supports optional token refresh on auth errors (`refresh::1`) and continuous live-monitor mode (`live::1`).

-- **Parameters:** [`format::`](../param/002_format.md), [`refresh::`](../param/019_refresh.md), [`live::`](../param/020_live.md), [`interval::`](../param/021_interval.md), [`jitter::`](../param/022_jitter.md), [`trace::`](../param/023_trace.md), [`sort::`](../param/025_sort.md), [`desc::`](../param/026_desc.md), [`prefer::`](../param/027_prefer.md), [`next::`](../param/032_next.md), [`cols::`](../param/033_cols.md), [`touch::`](../param/034_touch.md), [`imodel::`](../param/035_imodel.md), [`effort::`](../param/036_effort.md)
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
clp .usage sort::reset prefer::opus
clp .usage next::endurance
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
| `sort::` | `enum` | `drain` | Row ordering strategy: `drain` (lowest weekly quota first), `name` (alphabetical), `endurance` (sustained session), `reset` (soonest quota refill), `next` (mirrors active `next::` strategy) |
| `desc::` | `bool` | context-sensitive | Sort direction; default depends on `sort::` strategy (`name`/`drain`/`reset`→`0`, `endurance`→`1`) |
| `prefer::` | `enum` | `any` | Weekly quota column for sort heuristics: `any` = `min(7d Left, 7d(Son))`, `opus` = `7d Left`, `sonnet` = `7d(Son)` |
| `next::` | `enum` | `drain` | Strategy placing `→` on recommended account: `drain`, `endurance`; footer always shows both |
| `cols::` | `string` | `""` | Column visibility modifiers: comma-separated `+col_id` / `-col_id` relative to default set |
| `touch::` | `bool` | `1` | Keep accounts with active 5h countdown alive by sending minimal prompt via isolated subprocess; re-fetch quota |
| `imodel::` | `enum` | `auto` | Model for isolated subprocesses: `auto` (sonnet if `7d(Son)≥30%`, else opus), `sonnet`, `opus`, `keep` |
| `effort::` | `enum` | `auto` | Effort level for isolated subprocesses: `auto` (max for model: `high`/sonnet, `max`/opus), `high`, `max` |

**Examples:**

```bash
clp .usage
# Quota
#
#   ●  Account          5h Left  5h Reset    7d Left  7d(Son)  7d Reset  Expires     Sub  ~Renews
# ✓ 🟢 alice@example.com    86%      in 3h 19m  65%      35%      in 4d 23h  in 7h 24m  max  Jun  5
#   🟢 bob@example.com      100%     in 4h 58m  88%      28%      in 6d 14h  in 5h 02m  max  Jun  6
# → 🟡 frank@example.com    3%       in 0h 23m  52%      18%      in 2d 11h  in 1h 12m  max  Jun  8
#   🔴 dave@example.com     —        —           —        —        —          EXPIRED    ?    (missing accessToken)
#
# Valid: 3 / 4   ->  Next by strategy:
#   endurance  bob@example.com     100% session, 88% 7d left, expires in 5h 02m
#   drain      frank@example.com   3% session, resets in 23m

clp .usage live::1 interval::60 jitter::10
# Quota
# ...table...
#
#   Next update in 0:59 (at 14:32:07 UTC)  [Ctrl-C to exit]
# (refreshes every 60–70 seconds; Ctrl-C exits cleanly)
```

**Notes:**
- Accounts are enumerated from `{credential_store}/*.credentials.json` in alphabetical order.
- Flag column priority: `✓` = current account, `*` = active-but-not-current (divergence), `→` = recommended next account. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).
- Status emoji column (`●`): composite AND of 5h and 7d — `🟢` = valid token + `5h Left > 15%` and `7d Left > 5%`; `🟡` = valid token + either `5h Left ≤ 15%` or `7d Left ≤ 5%`; `🔴` = invalid/missing token. Per-column emoji also embedded in `5h Left` (🟢/🟡 at ≤15% threshold) and `7d Left` (🟢/🟡 at ≤5% threshold). No JSON equivalent.
- `Expires` is sourced from `expiresAt` in the credential file — available even when the API call fails.
- `Sub` and `~Renews` are sourced from `GET /api/oauth/account` (parallel fetch); show `?` when that fetch fails.
- Accounts with expired or missing `accessToken` show `—` for quota columns and a shortened error reason.
- Footer: always shows one recommendation per strategy (endurance, drain) when ≥2 accounts have valid quota data; `next::` controls only which account receives `→` in the table body.
- Empty credential store exits 0 with `(no accounts configured)`.
- `refresh::1` triggers at most one retry per account per cycle. See [feature/017_token_refresh.md](../../feature/017_token_refresh.md).
- `live::1 format::json` exits 1 before any fetch. See [feature/018_live_monitor.md](../../feature/018_live_monitor.md).
- Three-tier display grouping (🟢 → 🟡 → 🔴) applied before sort strategy within each tier. Within 🟡, h-exhausted accounts (`5h Left ≤ 15%`) appear before weekly-exhausted accounts (`5h Left > 15%`, `7d Left ≤ 5%`).
- `Sub` column hidden by default; show via `cols::+sub`. `7d Son Reset` column also hidden by default; show via `cols::+7d_son_reset`.
- Duration format (`format_duration_secs`) capped to 2 significant units (e.g., `1d 2h` not `1d 2h 45m`).
- See [feature/009_token_usage.md](../../feature/009_token_usage.md) for the baseline algorithm and AC criteria.
- See [feature/023_next_account_strategies.md](../../feature/023_next_account_strategies.md) for recommendation strategies.
- `touch::` (default `1`) keeps accounts with an active 5h window alive by sending a minimal prompt that resets the 5h countdown; pass `touch::0` to suppress. Runs after `refresh::` when both active. See [feature/024_session_touch.md](../../feature/024_session_touch.md).
- `imodel::` controls the Claude model injected into `touch::` and `refresh::` subprocesses. `auto` (default) selects Sonnet when an account's `7d(Son) ≥ 30%` and Opus otherwise. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).
- `effort::` controls the effort level (`--effort` flag) for those subprocesses. `auto` (default) uses `high` for Sonnet and `max` for Opus; no flag when `imodel::keep`. See [feature/026_subprocess_model_effort.md](../../feature/026_subprocess_model_effort.md).
