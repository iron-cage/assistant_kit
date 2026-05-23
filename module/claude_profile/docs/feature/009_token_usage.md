# Feature: All-Accounts Live Quota Reporting

### Scope

- **Purpose**: Surface live quota utilization for all saved accounts and the currently live session via `GET /api/oauth/usage`, showing 5h, 7d, and Sonnet-specific weekly quota remaining.
- **Responsibility**: Documents the `usage` module and `.usage` CLI command.
- **In Scope**: Per-account quota fetch via `claude_quota::fetch_oauth_usage()` calling `GET /api/oauth/usage`, `OauthUsageData` parsing with `five_hour`/`seven_day`/`seven_day_sonnet` fields, parallel fetch of account billing state via `claude_quota::fetch_oauth_account()` → `OauthAccountData` (`billing_type`, `has_max`, `org_created_at`), token expiry from credential files (`expires_at_ms`), live account detection by matching `accessToken` in `~/.claude/.credentials.json` against saved account tokens, active account divergence marker (`*` in flag column for `_active`-but-not-current accounts), synthetic `(current session)` row when live credentials are unsaved, `Sub` column (subscription label: `max`/`pro`/`—`/`?`), `~Renews` column (estimated next Stripe billing date from `org_created_at` day-of-month), table output using `data_fmt`, graceful handling of expired/missing tokens, recommendation marker for best next account, footer summary line, `format::json` output.
- **Out of Scope**: Historical token counts from stats-cache.json (replaced by live API data); verbosity levels (single fixed output level per command design); relying on `_active` marker for `✓` determination (live credential matching via `accessToken` comparison determines `✓`; `_active` determines `*` only).

### Design

`claude_profile` CLI provides a `.usage` command that fetches live quota utilization for every saved account by calling `claude_quota::fetch_oauth_usage(&token)` which issues `GET /api/oauth/usage` to `api.anthropic.com`. Results are displayed as a table.

**Live account detection:** The `_active` marker is NOT used to determine which account is currently in use. Instead, the command reads the `accessToken` from `~/.claude/.credentials.json` (the live credentials file used by Claude Code) and compares it against each saved account's stored token. This is correct even when an external actor (Claude Code, `claude auth login`, another process) has changed the credentials without going through `clp`.

**Algorithm:**
1. Read the credential store — enumerate all saved accounts (`{credential_store}/*.credentials.json`) via `account::list()`; each `Account` struct includes `expires_at_ms`.
2. Read `~/.claude/.credentials.json` to obtain the **live** `accessToken` and `expiresAt`. This identifies the credentials currently in use by Claude Code regardless of the `_active` marker.
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
   b. Mark the `_active` account with `*` in the flag column when `is_active = true` AND `is_current = false`. No `*` is emitted when the active and current accounts are the same.
   c. From non-live accounts with valid quota data and `expires_in_secs > 0`, select the one with the highest `5h Left`; mark it `→` (recommended next). If no such account exists, no `→` is emitted.
6. Render results as a table using `data_fmt`:
   - Columns: flag (`✓`/`*`/`→`/ , priority: `✓` > `*` > `→` > blank), Account, Expires, Sub, ~Renews, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset
   - `Expires`: "in Xh Ym" when `expires_in_secs > 0`; "EXPIRED" when `expires_in_secs == 0`
   - `Sub`: `"max"` (`billing_type == "stripe_subscription"` + `has_max`), `"pro"` (`billing_type == "stripe_subscription"` + `!has_max`), `"—"` (`billing_type == "none"`), `"?"` (`OauthAccountData` unavailable)
   - `~Renews`: `"Mon DD"` format — day-of-month from `org_created_at` projected to next occurrence after today (e.g. `"Jun  5"`); `"?"` when `OauthAccountData` unavailable; `"—"` when parsing fails
   - `5h Left` / `7d Left`: remaining percentage (0–100, rounded to nearest integer); sourced from `OauthUsageData.five_hour.utilization` / `seven_day.utilization` (0.0–100.0 scale, remaining = `100 - utilization`)
   - `7d(Son)`: remaining Sonnet-only weekly quota percentage; sourced from `OauthUsageData.seven_day_sonnet.utilization`; shows `—` when `seven_day_sonnet` is `None`
   - `5h Reset` / `7d Reset`: countdown formatted via `format_duration_secs`; sourced from `five_hour.resets_at` / `seven_day.resets_at` (ISO-8601 UTC string → Unix seconds via `iso_to_unix_secs`)
   - Unavailable accounts show `—` for all quota columns and a shortened error reason in parentheses in the last visible column
   - `Sub` and `~Renews` are populated from `OauthAccountData` regardless of whether the quota fetch succeeded; both show `"?"` when the account fetch failed
7. Append footer line when ≥2 accounts with valid quota data exist:
   `Valid: X / Y   →  Next: name  (N% session left, token expires in Xh Ym)`
   Omit footer when 0 or 1 valid account.
8. For `format::json`: output a JSON array with one object per account (synthetic first if present, then alphabetical saved), always including `expires_in_secs`.

**Output format (text) — saved account is live:**

```
Quota

  Account          Expires     Sub  ~Renews  5h Left  5h Reset    7d Left  7d(Son)  7d Reset
✓ i12@wbox.pro    in 7h 24m  max  Jun  5   86%      in 3h 19m  65%      35%      in 4d 23h
→ i6@wbox.pro     in 5h 02m  max  Jun  6   100%     in 4h 58m  88%      28%      in 6d 14h
  i7@wbox.pro     EXPIRED    ?    ?        —        —           —        —        (missing accessToken)
  i8@wbox.pro     EXPIRED    ?    ?        —        —           —        —        (missing accessToken)

Valid: 2 / 4   →  Next: i6@wbox.pro  (100% session left, token expires in 5h 02m)
```

(`?` in Sub/~Renews = account fetch failed or skipped due to token read error)

**Output format (text) — divergence (active ≠ current):**

```
Quota

  Account          Expires     Sub  ~Renews  5h Left  5h Reset    7d Left  7d(Son)  7d Reset
✓ i12@wbox.pro    in 7h 24m  max  Jun  5   86%      in 3h 19m  65%      35%      in 4d 23h
* i6@wbox.pro     in 5h 02m  max  Jun  6   100%     in 4h 58m  88%      28%      in 6d 14h
→ i3@wbox.pro     in 6h 11m  max  Jun 11   95%      in 3h 44m  72%      54%      in 5d 01h

Valid: 3 / 3   →  Next: i3@wbox.pro  (95% session left, token expires in 6h 11m)
```

(`*` = `_active` marker points here, but live credentials belong to `i12@wbox.pro`)

**Output format (text) — unsaved account is live (synthetic row):**

```
Quota

  Account              Expires    Sub  ~Renews  5h Left  5h Reset   7d Left  7d(Son)  7d Reset
✓ (current session)   in 4h 39m  max  Jun  5   64%      in 1h 39m  39%      —        in 3d 17h 39m
→ i3@wbox.pro         in 5h 02m  max  Jun 11   100%     in 4h 58m  88%      28%      in 6d 14h
  i7@wbox.pro         EXPIRED    ?    ?        —        —           —        —        (missing accessToken)

Valid: 2 / 3   →  Next: i3@wbox.pro  (100% session left, token expires in 5h 02m)
```

**Output format (JSON):**

```json
[
  {"account":"i12@wbox.pro","is_current":true,"is_active":false,"expires_in_secs":26640,"billing_type":"stripe_subscription","has_max":true,"next_renewal_est":"Jun  5","session_5h_left_pct":86,"session_5h_resets_in_secs":11940,"weekly_7d_left_pct":65,"weekly_7d_sonnet_left_pct":35,"weekly_7d_resets_in_secs":432540},
  {"account":"i6@wbox.pro","is_current":false,"is_active":true,"expires_in_secs":18120,"billing_type":"stripe_subscription","has_max":true,"next_renewal_est":"Jun  6","session_5h_left_pct":100,"session_5h_resets_in_secs":17880,"weekly_7d_left_pct":88,"weekly_7d_sonnet_left_pct":28,"weekly_7d_resets_in_secs":500040},
  {"account":"i7@wbox.pro","is_current":false,"is_active":false,"expires_in_secs":0,"billing_type":null,"has_max":null,"next_renewal_est":null,"error":"missing accessToken"},
  {"account":"i8@wbox.pro","is_current":false,"is_active":false,"expires_in_secs":0,"billing_type":null,"has_max":null,"next_renewal_est":null,"error":"missing accessToken"}
]
```

(`weekly_7d_sonnet_left_pct` is `null` when `seven_day_sonnet` is absent from the API response. `billing_type`, `has_max`, and `next_renewal_est` are `null` when the account fetch failed or the token could not be read.)

**Table rendering:** All table and tree output MUST use the `data_fmt` crate. No hand-rolled string formatting.

**Error handling:**
- `HOME` unset → `InternalError`
- Credential store unreadable → `InternalError`
- `~/.claude/.credentials.json` unreadable → live detection skipped; no `✓` is emitted on any row; `*` is still emitted for the `_active` account; saved accounts still rendered
- Individual account token expired or invalid → inline `error` field in that row (non-fatal; other accounts still processed)
- Empty credential store (and no synthetic row) → empty table with `(no accounts configured)` message

### Acceptance Criteria

- **AC-01**: `.usage` fetches quota for every saved account, not only the active one.
- **AC-02**: The **live account** — the saved account whose `accessToken` matches the live `~/.claude/.credentials.json` token — has `✓` in the flag column. The `_active` marker is NOT used for `✓` determination.
- **AC-03**: Accounts with expired or missing tokens show `—` in quota columns and a shortened error reason in the final column.
- **AC-04**: Table output is rendered by `data_fmt`.
- **AC-05**: `format::json` returns a valid JSON array with one object per account; each object includes `expires_in_secs`, `is_current` (bool), `is_active` (bool), `billing_type` (string or `null`), `has_max` (bool or `null`), and `next_renewal_est` (string or `null`); successful rows also include `session_5h_left_pct`, `weekly_7d_left_pct`, and `weekly_7d_sonnet_left_pct` (all remaining, not consumed); `weekly_7d_sonnet_left_pct` is `null` when Sonnet quota data is absent from the API response; `billing_type`, `has_max`, and `next_renewal_est` are `null` when the account fetch failed.
- **AC-06**: Missing credential store exits 2 with an actionable error message.
- **AC-07**: The `Expires` column shows token TTL ("in Xh Ym") for valid tokens and "EXPIRED" for tokens whose `expiresAt` is in the past; sourced from the credential file without an API call.
- **AC-08**: `5h Left` and `7d Left` show remaining quota percentage (100 − consumed); `7d(Son)` shows remaining Sonnet-only weekly quota (100 − consumed) or `—` when absent; `5h Reset` and `7d Reset` show independent reset countdowns as separate columns; all quota data sourced from `claude_quota::fetch_oauth_usage()` → `OauthUsageData`.
- **AC-17**: `7d(Son)` column is populated when `OauthUsageData.seven_day_sonnet` is `Some`; shows `—` when `None`. JSON field `weekly_7d_sonnet_left_pct` is an integer when present and `null` when absent.
- **AC-09**: The `→` flag marks the non-live account with the highest remaining session quota among those with valid quota data and a non-expired token; no `→` is emitted when no such account exists.
- **AC-10**: A footer line "Valid: X / Y   →  Next: name  (...)" is appended when ≥2 accounts have valid quota data; the footer is absent when 0 or 1 valid account.
- **AC-11**: When the live `~/.claude/.credentials.json` token does not match any saved account's token, a synthetic row is prepended at the top of the table with `✓`, quota fetched via the live token, and the name set to the email from `~/.claude/.claude.json` (or `(current session)` when that file is unavailable or the field is empty).
- **AC-12**: When `~/.claude/.credentials.json` is unreadable, no `✓` is emitted on any row; all saved accounts are still shown.
- **AC-13**: `*` in the flag column marks the account with the `_active` marker when it differs from the current (live) account; no `*` appears when active and current are the same account.
- **AC-14**: When current = active (normal case), only `✓` appears on the current row; no `*` is emitted on any row.
- **AC-15**: When `~/.claude/.credentials.json` is unreadable, no `✓` is emitted; `*` is still emitted for the `_active` account. See [016_current_account_awareness.md](016_current_account_awareness.md).
- **AC-16**: `format::json` output uses `is_current` (replacing the former `active` field) and includes a new `is_active` boolean field per object.

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
