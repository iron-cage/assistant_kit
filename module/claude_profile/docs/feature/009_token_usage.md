# Feature: All-Accounts Live Quota Reporting

### Scope

- **Purpose**: Surface live quota utilization for all saved accounts and the currently live session via `GET /api/oauth/usage`, showing 5h, 7d, and Sonnet-specific weekly quota remaining.
- **Responsibility**: Documents the `usage` module and `.usage` CLI command.
- **In Scope**: Per-account quota fetch via `claude_quota::fetch_oauth_usage()` calling `GET /api/oauth/usage`, `OauthUsageData` parsing with `five_hour`/`seven_day`/`seven_day_sonnet` fields, parallel fetch of account billing state via `claude_quota::fetch_oauth_account()` → `OauthAccountData` (`billing_type`, `has_max`, `org_created_at`), token expiry from credential files (`expires_at_ms`), live account detection by matching `accessToken` in `~/.claude/.credentials.json` against saved account tokens, active account divergence marker (`*` in flag column for active-marker-but-not-current accounts), synthetic `(current session)` row when live credentials are unsaved, `Sub` column (subscription label: `max`/`pro`/`—`/`?`, hidden by default — `cols::+sub` to show), `~Renews` column (estimated next Stripe billing date from `org_created_at` day-of-month), table output using `data_fmt`, graceful handling of expired/missing tokens, composite `●` status emoji (AND of 5h and 7d), per-column emoji in `5h Left` and `7d Left` values, three-tier universal display grouping (🟢 → 🟡 → 🔴) with h-exhausted sub-group before weekly-exhausted sub-group within 🟡, `cols::` column visibility modifiers, `next::` recommendation strategy parameter, multi-strategy footer, `7d Son Reset` column (hidden by default), duration format capped to 2 significant units, `format::json` output.
- **Out of Scope**: Historical token counts from stats-cache.json (replaced by live API data); verbosity levels (single fixed output level per command design); relying on per-machine active marker for `✓` determination (live credential matching via `accessToken` comparison determines `✓`; active marker determines `*` only).

### Design

`claude_profile` CLI provides a `.usage` command that fetches live quota utilization for every saved account by calling `claude_quota::fetch_oauth_usage(&token)` which issues `GET /api/oauth/usage` to `api.anthropic.com`. Results are displayed as a table.

**Live account detection:** The per-machine active marker is NOT used to determine which account is currently in use. Instead, the command reads the `accessToken` from `~/.claude/.credentials.json` (the live credentials file used by Claude Code) and compares it against each saved account's stored token. This is correct even when an external actor (Claude Code, `claude auth login`, another process) has changed the credentials without going through `clp`.

**Algorithm:**
1. Read the credential store — enumerate all saved accounts (`{credential_store}/*.credentials.json`) via `account::list()`; each `Account` struct includes `expires_at_ms`.
2. Read `~/.claude/.credentials.json` to obtain the **live** `accessToken` and `expiresAt`. This identifies the credentials currently in use by Claude Code regardless of the per-machine active marker.
3. Detect the **live account** by comparing the live `accessToken` against each saved account's stored token:
   a. If exactly one saved account's token matches, that account is the live account (it will receive `✓`).
   b. If no saved account's token matches (credentials were set by an external actor and not yet saved, or are from a fresh login), construct a **synthetic entry**:
      - Name: email from `~/.claude/.claude.json` `oauthAccount.emailAddress` if readable and non-empty; otherwise `(current session)`.
      - Quota: fetched using the live token (identical path to saved accounts).
      - Expiry: `expiresAt` parsed from `~/.claude/.credentials.json`.
      - The synthetic entry is marked live (`✓`) and prepended at the top of the table (before the alphabetically sorted saved accounts).
4. For each saved account (in alphabetical order):
   a. Compute `expires_in_secs = saturating_sub(expires_at_ms / 1000, now_secs)`.
   b. Read the account's `accessToken` from the credential file.
   c. If token read succeeds:
      1. Spawn `claude_quota::fetch_oauth_account(&token)` on a background thread.
      2. Call `claude_quota::fetch_oauth_usage(&token)` on the current thread → `OauthUsageData` or error reason.
      3. Join the background thread → `Option<OauthAccountData>` (`None` on any fetch or parse error).
   d. On quota success: record `5h Left = 100.0 - five_hour.utilization`, `five_hour.resets_at`, `7d Left = 100.0 - seven_day.utilization`, `seven_day.resets_at`; `7d(Son) = 100.0 - seven_day_sonnet.utilization` when `seven_day_sonnet` is `Some`, else `None`.
   e. On any failure (token read or API): record the error reason.
5. Post-process:
   a. Mark the live account (detected in step 3) with `✓` in the flag column (`is_current = true`).
   b. Mark the active account with `*` in the flag column when `is_active = true` AND `is_current = false`. No `*` is emitted when the active and current accounts are the same.
   c. Recommendation is controlled by the `next::` parameter (see [023_next_account_strategies.md](023_next_account_strategies.md)). The account selected by the active strategy receives `→` in the table body; the footer always shows one recommendation per strategy. Default strategy is `endurance`.
6. Render results as a table using `data_fmt`:
   - **Default columns:** flag (`✓`/`*`/`→`/ blank, priority `✓` > `*` > `→` > blank), status (`🔴`/`🟡`/`🟢`, header `●`), Account, Expires, ~Renews, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset
   - **Hidden-by-default columns:** Sub, 7d Son Reset — available via `cols::+sub`, `cols::+7d_son_reset`
   - **Column visibility:** The `cols::` parameter accepts comma-separated `+col_id` / `-col_id` modifiers relative to the default column set. The `flag` and `account` columns are structural and always visible. See [param/033_cols.md](../cli/param/033_cols.md).
   - **Composite status emoji column (`●`):** placed between the flag and Account columns; populated on every row; uses AND logic of 5h and 7d:
     - `🔴` — token read failed or API returned an error (no valid quota data; `result` is `Err`)
     - `🟡` — valid token, either `5h Left ≤ 15.0%` or `7d Left ≤ 5.0%` (at least one quota exhausted; `result` is `Ok`)
     - `🟢` — valid token, both `5h Left > 15.0%` and `7d Left > 5.0%` (both quotas healthy; `result` is `Ok`)
     - No JSON equivalent — the status is a display-only column derived from existing fields
   - **Per-column emoji:** `5h Left` and `7d Left` column values embed an individual 🟢/🟡 emoji based on their own ≤5% threshold: `🟢 86%` when > 5%, `🟡 3%` when ≤ 5%. This provides drill-down visibility beyond the composite `●`.
   - `Expires`: "in Xh Ym" when `expires_in_secs > 0`; "EXPIRED" when `expires_in_secs == 0`
   - `Sub` (hidden by default): `"max"` (`billing_type == "stripe_subscription"` + `has_max`), `"pro"` (`billing_type == "stripe_subscription"` + `!has_max`), `"—"` (`billing_type == "none"`), `"?"` (`OauthAccountData` unavailable)
   - `~Renews`: `"Mon DD"` format — day-of-month from `org_created_at` projected to next occurrence after today (e.g. `"Jun  5"`); `"?"` when `OauthAccountData` unavailable; `"—"` when parsing fails
   - `5h Left` / `7d Left`: remaining percentage (0–100, rounded to nearest integer) with per-column emoji prefix; sourced from `OauthUsageData.five_hour.utilization` / `seven_day.utilization` (0.0–100.0 scale, remaining = `100 - utilization`)
   - `7d(Son)`: remaining Sonnet-only weekly quota percentage; sourced from `OauthUsageData.seven_day_sonnet.utilization`; shows `—` when `seven_day_sonnet` is `None`
   - `5h Reset` / `7d Reset`: countdown formatted via `format_duration_secs` (capped to 2 significant units); sourced from `five_hour.resets_at` / `seven_day.resets_at` (ISO-8601 UTC string → Unix seconds via `iso_to_unix_secs`)
   - `7d Son Reset` (hidden by default): countdown to Sonnet-specific weekly reset; shows `—` when `seven_day_sonnet` is `None`
   - Unavailable accounts show `—` for all quota columns and a shortened error reason in parentheses in the last visible column
   - `Sub` and `~Renews` are populated from `OauthAccountData` regardless of whether the quota fetch succeeded; both show `"?"` when the account fetch failed
   - **Three-tier display grouping:** Before applying the sort strategy, accounts are grouped by composite health tier: 🟢 tier (both > 5%) → 🟡 tier (either ≤ 5%) → 🔴 tier (error). Within the 🟡 tier, accounts are further ordered into two sub-groups: **h-exhausted** (`5h Left ≤ 5%`) first, then **weekly-exhausted** (`5h Left > 5%` and `7d Left ≤ 5%`). Accounts where both quotas are ≤ 5% fall in the h-exhausted sub-group. Sort strategy applies within each sub-group. This ensures healthy accounts always appear above exhausted or errored accounts regardless of sort strategy or direction, and session-blocked accounts are visually separated above weekly-blocked accounts within 🟡.
   - **Duration format:** `format_duration_secs` output is capped to 2 significant units (e.g., `1d 2h` not `1d 2h 45m`, `3h 19m` not `3h 19m 5s`).
7. Append footer when ≥2 accounts with valid quota data exist. Footer always shows one recommendation line per strategy (endurance, drain). The `→` table marker appears on the account selected by the `next::` strategy (see [023_next_account_strategies.md](023_next_account_strategies.md)). Omit footer when 0 or 1 valid account.
8. For `format::json`: output a JSON array with one object per account (synthetic first if present, then alphabetical saved), always including `expires_in_secs`.

**Output format (text) — saved account is live, `next::endurance` (default):**

```
Quota

  ●  Account              Expires    ~Renews  5h Left     5h Reset  7d Left     7d(Son)  7d Reset
✓ 🟢 alice@example.com    in 7h 24m  Jun  5   🟢 86%     in 3h 19m 🟢 65%     35%      in 4d 23h
→ 🟢 bob@example.com      in 5h 02m  Jun  6   🟢 100%    in 4h 58m 🟢 88%     28%      in 6d 14h
  🟡 carol@example.com    in 1h 12m  Jun  8   🟡 3%      in 0h 23m 🟢 52%     18%      in 2d 11h
  🔴 dave@example.com     EXPIRED    ?        —          —          —          —        (missing accessToken)
  🔴 eve@example.com      EXPIRED    ?        —          —          —          —        (missing accessToken)

Valid: 3 / 5   ->  Next by strategy:
  endurance  bob@example.com     100% session, 88% 7d left, expires in 5h 02m
  drain      carol@example.com   3% session, resets in 23m
```

(Sub column hidden by default; show with `cols::+sub`. Three-tier grouping: 🟢 tier → 🟡 tier → 🔴 tier. `→` marks the account selected by `next::` strategy.)

**Output format (text) — divergence, `next::endurance` (default):**

```
Quota

  ●  Account              Expires    ~Renews  5h Left     5h Reset  7d Left     7d(Son)  7d Reset
✓ 🟢 alice@example.com    in 7h 24m  Jun  5   🟢 86%     in 3h 19m 🟢 65%     35%      in 4d 23h
* 🟢 bob@example.com      in 5h 02m  Jun  6   🟢 100%    in 4h 58m 🟢 88%     28%      in 6d 14h
→ 🟢 carol@example.com    in 6h 11m  Jun 11   🟢 95%     in 3h 44m 🟢 72%     54%      in 5d 01h

Valid: 3 / 3   ->  Next by strategy:
  endurance  carol@example.com   95% session, 72% 7d left, expires in 6h 11m
  drain      carol@example.com   95% session, resets in 3h 44m
```

(`*` = active marker points here, but live credentials belong to `alice@example.com`. Both strategies agree — carol is the only eligible account.)

**Output format (text) — unsaved account is live (synthetic row):**

```
Quota

  ●  Account              Expires    ~Renews  5h Left     5h Reset  7d Left     7d(Son)  7d Reset
✓ 🟢 (current session)    in 4h 39m  Jun  5   🟢 64%     in 1h 39m 🟢 39%     —        in 3d 17h
→ 🟢 alice@example.com    in 5h 02m  Jun 11   🟢 100%    in 4h 58m 🟢 88%     28%      in 6d 14h
  🔴 bob@example.com      EXPIRED    ?        —          —          —          —        (missing accessToken)

Valid: 2 / 3   ->  Next by strategy:
  endurance  alice@example.com   100% session, 88% 7d left, expires in 5h 02m
  drain      alice@example.com   100% session, resets in 4h 58m
```

**Output format (JSON):**

```json
[
  {"account":"alice@example.com","is_current":true,"is_active":false,"expires_in_secs":26640,"billing_type":"stripe_subscription","has_max":true,"next_renewal_est":"Jun  5","session_5h_left_pct":86,"session_5h_resets_in_secs":11940,"weekly_7d_left_pct":65,"weekly_7d_sonnet_left_pct":35,"weekly_7d_resets_in_secs":432540},
  {"account":"bob@example.com","is_current":false,"is_active":true,"expires_in_secs":18120,"billing_type":"stripe_subscription","has_max":true,"next_renewal_est":"Jun  6","session_5h_left_pct":100,"session_5h_resets_in_secs":17880,"weekly_7d_left_pct":88,"weekly_7d_sonnet_left_pct":28,"weekly_7d_resets_in_secs":500040},
  {"account":"carol@example.com","is_current":false,"is_active":false,"expires_in_secs":0,"billing_type":null,"has_max":null,"next_renewal_est":null,"error":"missing accessToken"},
  {"account":"dave@example.com","is_current":false,"is_active":false,"expires_in_secs":0,"billing_type":null,"has_max":null,"next_renewal_est":null,"error":"missing accessToken"}
]
```

(`weekly_7d_sonnet_left_pct` is `null` when `seven_day_sonnet` is absent from the API response. `billing_type`, `has_max`, and `next_renewal_est` are `null` when the account fetch failed or the token could not be read.)

**Table rendering:** All table and tree output MUST use the `data_fmt` crate. No hand-rolled string formatting.

**Error handling:**
- `HOME` unset → `InternalError`
- Credential store unreadable → `InternalError`
- `~/.claude/.credentials.json` unreadable → live detection skipped; no `✓` is emitted on any row; `*` is still emitted for the active account; saved accounts still rendered
- Individual account token expired or invalid → inline `error` field in that row (non-fatal; other accounts still processed)
- Empty credential store (and no synthetic row) → empty table with `(no accounts configured)` message

### Acceptance Criteria

- **AC-01**: `.usage` fetches quota for every saved account, not only the active one.
- **AC-02**: The **live account** — the saved account whose `accessToken` matches the live `~/.claude/.credentials.json` token — has `✓` in the flag column. The per-machine active marker is NOT used for `✓` determination.
- **AC-03**: Accounts with expired or missing tokens show `—` in quota columns and a shortened error reason in the final column.
- **AC-04**: Table output is rendered by `data_fmt`.
- **AC-05**: `format::json` returns a valid JSON array with one object per account; each object includes `expires_in_secs`, `is_current` (bool), `is_active` (bool), `billing_type` (string or `null`), `has_max` (bool or `null`), and `next_renewal_est` (string or `null`); successful rows also include `session_5h_left_pct`, `weekly_7d_left_pct`, and `weekly_7d_sonnet_left_pct` (all remaining, not consumed); `weekly_7d_sonnet_left_pct` is `null` when Sonnet quota data is absent from the API response; `billing_type`, `has_max`, and `next_renewal_est` are `null` when the account fetch failed.
- **AC-06**: Missing credential store exits 2 with an actionable error message.
- **AC-07**: The `Expires` column shows token TTL ("in Xh Ym") for valid tokens and "EXPIRED" for tokens whose `expiresAt` is in the past; sourced from the credential file without an API call.
- **AC-08**: `5h Left` and `7d Left` show remaining quota percentage (100 − consumed); `7d(Son)` shows remaining Sonnet-only weekly quota (100 − consumed) or `—` when absent; `5h Reset` and `7d Reset` show independent reset countdowns as separate columns; all quota data sourced from `claude_quota::fetch_oauth_usage()` → `OauthUsageData`.
- **AC-17**: `7d(Son)` column is populated when `OauthUsageData.seven_day_sonnet` is `Some`; shows `—` when `None`. JSON field `weekly_7d_sonnet_left_pct` is an integer when present and `null` when absent.
- **AC-09**: The `→` flag in the table body is controlled by the `next::` parameter (see [023_next_account_strategies.md](023_next_account_strategies.md)). The footer always shows one recommendation per strategy (endurance, drain); `next::` controls only which account receives the `→` marker. Default is `next::endurance`.
- **AC-10**: A footer is appended when ≥2 accounts have valid quota data; the footer is absent when 0 or 1 valid account. The footer always shows both strategy recommendations (endurance, drain) regardless of `next::` value.
- **AC-11**: When the live `~/.claude/.credentials.json` token does not match any saved account's token, a synthetic row is prepended at the top of the table with `✓`, quota fetched via the live token, and the name set to the email from `~/.claude/.claude.json` (or `(current session)` when that file is unavailable or the field is empty).
- **AC-12**: When `~/.claude/.credentials.json` is unreadable, no `✓` is emitted on any row; all saved accounts are still shown.
- **AC-13**: `*` in the flag column marks the account with the per-machine active marker when it differs from the current (live) account; no `*` appears when active and current are the same account.
- **AC-14**: When current = active (normal case), only `✓` appears on the current row; no `*` is emitted on any row.
- **AC-15**: When `~/.claude/.credentials.json` is unreadable, no `✓` is emitted; `*` is still emitted for the active account. See [016_current_account_awareness.md](016_current_account_awareness.md).
- **AC-16**: `format::json` output uses `is_current` (replacing the former `active` field) and includes a new `is_active` boolean field per object.
- **AC-18**: Every table row has a composite status emoji in the `●` column (second column, after flag) using AND logic: `🟢` when `result` is `Ok` and both `5h Left > 5%` and `7d Left > 5%`, `🟡` when `result` is `Ok` and either `5h Left ≤ 5%` or `7d Left ≤ 5%`, `🔴` when `result` is `Err`. The emoji appears on every row including the synthetic current-session row.
- **AC-19**: The 5% boundary for composite `●` is exclusive for `🟢` and inclusive for `🟡` on both dimensions: an account with exactly `5h Left = 5%` OR `7d Left = 5%` shows `🟡`; both must be `> 5%` for `🟢`.
- **AC-20**: The `●` status emoji column has no JSON equivalent — `format::json` output is unchanged; pipeline consumers derive status from `session_5h_left_pct`, `weekly_7d_left_pct`, and the `error` field.
- **AC-21**: `5h Left` and `7d Left` column values each embed a per-column emoji prefix: `🟢` when that column's value `> 5%`, `🟡` when `≤ 5%`. This provides individual-dimension visibility beyond the composite `●`.
- **AC-22**: `Sub` column is hidden by default; shown via `cols::+sub`. `7d Son Reset` column is hidden by default; shown via `cols::+7d_son_reset`.
- **AC-23**: `cols::` parameter accepts comma-separated `+col_id` / `-col_id` modifiers. `flag` and `account` columns are structural and always visible. Invalid column IDs exit 1 with an error naming valid column IDs.
- **AC-24**: Three-tier display grouping: accounts are grouped 🟢 → 🟡 → 🔴 by composite health before any sort strategy is applied. Sort strategy applies within each tier. The grouping is never reversed by `desc::`.
- **AC-25**: `format_duration_secs` output is capped to 2 significant units: shows at most 2 time components (e.g., `1d 2h`, `3h 19m`, `23m`), never 3.
- **AC-26**: Within the 🟡 tier, h-exhausted accounts (`5h Left ≤ 5%`) appear before weekly-exhausted accounts (`5h Left > 5%` and `7d Left ≤ 5%`). Accounts where both `5h Left ≤ 5%` and `7d Left ≤ 5%` fall in the h-exhausted sub-group. Sort strategy applies within each sub-group. The sub-grouping is never reversed by `desc::`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `usage_routine()` CLI handler, quota fetching, table rendering, JSON output |
| source | `src/commands.rs` | Re-exports `usage_routine()` from `src/usage.rs` |
| dep | `claude_quota` | `fetch_oauth_usage()`, `fetch_oauth_account()` — transport functions; `OauthUsageData`, `OauthAccountData`, `PeriodUsage` types |
| dep | `data_fmt` | Table rendering for all output |
| test | `tests/cli/usage_test.rs` | All-accounts quota table and JSON output tests |
| doc | [013_account_limits.md](013_account_limits.md) | `.account.limits` command for single-account quota |
| doc | [command/006_usage.md](../cli/command/006_usage.md#command--9-usage) | CLI command specification |
| doc | [016_current_account_awareness.md](016_current_account_awareness.md) | Shared current-account detection algorithm; `*` flag semantics; JSON field renaming |
| doc | [017_token_refresh.md](017_token_refresh.md) | Token refresh extension; `apply_refresh()` and `refresh::` parameter |
| doc | [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies; three-tier grouping; `sort::`, `desc::`, `prefer::` parameters |
| doc | [023_next_account_strategies.md](023_next_account_strategies.md) | Recommendation strategies; `next::` parameter; multi-strategy footer |
| doc | [024_session_touch.md](024_session_touch.md) | Session touch extension; idle-account activation; `touch::` parameter |
| param | [cli/param/032_next.md](../cli/param/032_next.md) | `next::` parameter specification |
| param | [cli/param/033_cols.md](../cli/param/033_cols.md) | `cols::` parameter specification |
| param | [cli/param/034_touch.md](../cli/param/034_touch.md) | `touch::` parameter specification |
