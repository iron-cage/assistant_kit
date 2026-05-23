# Commands :: Usage

Live quota utilization commands.

---

### Command :: 9. `.usage`

Fetches live quota utilization for every saved account via `claude_quota::fetch_oauth_usage()` (`GET /api/oauth/usage`). Renders results as a `data_fmt` table with per-account Expires, 5h Left, 5h Reset, 7d Left, 7d(Son), and 7d Reset columns, plus a footer recommendation line. Supports optional token refresh on auth errors (`refresh::1`) and continuous live-monitor mode (`live::1`).

-- **Parameters:** [`format::`](../param/02_format.md), [`refresh::`](../param/19_refresh.md), [`live::`](../param/20_live.md), [`interval::`](../param/21_interval.md), [`jitter::`](../param/22_jitter.md), [`trace::`](../param/23_trace.md)
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
```

| Parameter | Type | Default | Purpose |
|-----------|------|---------|---------|
| `format::` | [`OutputFormat`](../type/02_output_format.md) | `text` | Output format (`text` or `json`; `json` incompatible with `live::1`) |
| `refresh::` | `bool` | `1` | On 401/403 auth error or 429 with locally-expired token, refresh via isolated subprocess and retry |
| `live::` | `bool` | `0` | Enable continuous refresh loop (Ctrl-C to exit) |
| `interval::` | `u64` | `30` | Seconds between refresh cycles (≥ 30; only validated when `live::1`) |
| `jitter::` | `u64` | `0` | Max random seconds added to each cycle delay (≤ interval; only validated when `live::1`) |
| `trace::` | `bool` | `0` | Print `[trace]` lines to stderr: credential reads, API calls, and refresh steps |

**Examples:**

```bash
clp .usage
# Quota
#
#   Account          Expires     5h Left  5h Reset    7d Left  7d(Son)  7d Reset
# ✓ i12@wbox.pro    in 7h 24m  86%      in 3h 19m  65%      35%      in 4d 23h
# → i6@wbox.pro     in 5h 02m  100%     in 4h 58m  88%      28%      in 6d 14h
#   i7@wbox.pro     EXPIRED    —        —           —        —        (missing accessToken)
#
# Valid: 2 / 3   →  Next: i6@wbox.pro  (100% session left, token expires in 5h 02m)

clp .usage live::1 interval::60 jitter::10
# Quota
# ...table...
#
#   Next update in 0:59 (at 14:32:07 UTC)  [Ctrl-C to exit]
# (refreshes every 60–70 seconds; Ctrl-C exits cleanly)
```

**Notes:**
- Accounts are enumerated from `{credential_store}/*.credentials.json` in alphabetical order.
- Flag column priority: `✓` = current account, `*` = `_active`-but-not-current (divergence), `→` = recommended next account. See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).
- `Expires` is sourced from `expiresAt` in the credential file — available even when the API call fails.
- Accounts with expired or missing `accessToken` show `—` for quota columns and a shortened error reason.
- Footer "Valid: X / Y   →  Next: ..." appears when ≥2 accounts have valid quota data.
- Empty credential store exits 0 with `(no accounts configured)`.
- `refresh::1` triggers at most one retry per account per cycle. See [feature/017_token_refresh.md](../../feature/017_token_refresh.md).
- `live::1 format::json` exits 1 before any fetch. See [feature/018_live_monitor.md](../../feature/018_live_monitor.md).
- See [feature/009_token_usage.md](../../feature/009_token_usage.md) for the baseline algorithm and AC criteria.
