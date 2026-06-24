# Feature: All-Accounts Live Quota Reporting

### Scope

- **Purpose**: Surface live quota utilization for all saved accounts and the currently live session via `GET /api/oauth/usage`, showing 5h, 7d, and Sonnet-specific weekly quota remaining.
- **Responsibility**: Documents the `usage` module and `.usage` CLI command.
- **In Scope**: Per-account quota fetch via `claude_quota::fetch_oauth_usage()` calling `GET /api/oauth/usage`, `OauthUsageData` parsing with `five_hour`/`seven_day`/`seven_day_sonnet` fields, parallel fetch of account billing state via `claude_quota::fetch_oauth_account()` → `OauthAccountData` (`billing_type`, `has_max`, `org_created_at`), token expiry from credential files (`expires_at_ms`), live account detection by matching `accessToken` in `~/.claude/.credentials.json` against saved account tokens, active account divergence marker (`*` in flag column for active-marker-but-not-current accounts), `@` occupied-elsewhere flag (accounts named by any `_active_*` marker in the credential store other than the current machine's own marker receive `@` in the flag column when no higher-priority flag applies; `other_machines_active()` reads all non-own `_active_*` files and returns the set of account names), synthetic `(current session)` row when live credentials are unsaved, `Sub` column (subscription label: `max`/`pro`/`—`/`?`, hidden by default — `cols::+sub` to show), `~Renews` column (duration countdown to billing renewal: exact when `_renewal_at` ISO-8601 UTC override present in `{name}.json`, estimated with `~` prefix when derived from `org_created_at` day-of-month), `→ Next` column (soonest upcoming strategic event among `+7d` 7d quota reset and `$ren` billing renewal — token expiry (`!tok`) and 5h resets (`+5h`) are excluded; token expiry already surfaced in `Expires` column, 5h resets already surfaced in `5h Reset` column), `_renewal_at` optional ISO-8601 UTC field in `{name}.json` (written by `.account.renewal`, preserved by `save()` read-merge), table output using `data_fmt`, graceful handling of expired/missing tokens, composite `●` status emoji (AND of 5h and 7d), per-column emoji in `5h Left` and `7d Left` values, four-group status partition (🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red), `cols::` column visibility modifiers, `sort::`-driven 2-line footer (`Current` line identifying the `✓` account + `Next` recommendation line, both using `·` delimiter and aligned columns), sessions table showing all `_active_*` markers as `{user}@{host}` → account (controlled by `who::` parameter, default shown when >1 marker exists), `7d Son Reset` column (hidden by default), duration format capped to 2 significant units, `format::json` output.
- **Out of Scope**: Historical token counts from stats-cache.json (replaced by live API data); verbosity levels (single fixed output level per command design); relying on per-machine active marker for `✓` determination (live credential matching via `accessToken` comparison determines `✓`; active marker determines `*` only).

### Design

`claude_profile` CLI provides a `.usage` command that fetches live quota utilization for every saved account by calling `claude_quota::fetch_oauth_usage(&token)` which issues `GET /api/oauth/usage` to `api.anthropic.com`. Results are displayed as a table.

**Live account detection:** The per-machine active marker is NOT used to determine which account is currently in use. Instead, the command reads the `accessToken` from `~/.claude/.credentials.json` (the live credentials file used by Claude Code) and compares it against each saved account's stored token. This is correct even when an external actor (Claude Code, `claude auth login`, another process) has changed the credentials without going through `clp`.

**Account ownership and non-owned fetch (G1):** When account ownership is enabled (Feature 036), `fetch_quota_for_list()` checks each account's `is_owned` predicate before reading credentials. For non-owned accounts (`is_owned == false`), the credential file is NOT read and NO HTTP call is made — the account's quota is read directly from the local cache (`read_quota_cache()`). Non-owned rows display with `~` prefix and age indicator, identical to the cache-fallback path (Feature 033). If no cache exists, columns show `—`. `aq.is_owned = false` is set and propagated to `format::json` as `"is_owned": false`.

**Algorithm:**
1. Read the credential store — enumerate all saved accounts (`{credential_store}/*.credentials.json`) via `account::list()`; each `Account` struct includes `expires_at_ms`.
2. Read `~/.claude/.credentials.json` to obtain the **live** `accessToken` and `expiresAt`. This identifies the credentials currently in use by Claude Code regardless of the per-machine active marker.
3. Detect the **live account** by comparing the live `accessToken` against each saved account's stored token:
   a. If exactly one saved account's token matches, that account is the live account (it will receive `✓`).
   b. If no saved account's token matches (credentials were set by an external actor and not yet saved, or are from a fresh login), construct a **synthetic entry**:
      - Name: email from `~/.claude.json` `oauthAccount.emailAddress` if readable and non-empty; otherwise `(current session)`.
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
      4. If `OauthAccountData.billing_type == "none"` (confirmed cancelled subscription): override result to `Err("no subscription")` — the usage fetch result is discarded (see AC-31).
   d. On quota success: record `5h Left = 100.0 - five_hour.utilization`, `five_hour.resets_at`, `7d Left = 100.0 - seven_day.utilization`, `seven_day.resets_at`; `7d(Son) = 100.0 - seven_day_sonnet.utilization` when `seven_day_sonnet` is `Some`, else `None`.
   e. On any failure (token read or API): record the error reason.
5. Post-process:
   a. Mark the live account (detected in step 3) with `✓` in the flag column (`is_current = true`).
   b. Mark the active account with `*` in the flag column when `is_active = true` AND `is_current = false`. No `*` is emitted when the active and current accounts are the same.
   c. Mark accounts occupied on other machines with `@` in the flag column when `is_occupied_elsewhere = true` AND `is_active = false` AND `is_current = false`. `other_machines_active(store)` reads all `_active_*` files in the credential store, skips the current machine's own marker (from `active_marker_filename()`), and returns the set of account names referenced by the remaining markers. `is_occupied_elsewhere` is set by looking up each account's name in that returned set.
   d. Recommendation is controlled by the `sort::` parameter (see [020_usage_sort_strategies.md](020_usage_sort_strategies.md)). The top eligible account in the active sort order is shown in the footer's `Next (strategy):` line. The flag column shows only `✓`, `*`, `@`, or blank — no `→` marker. Default strategy is `renew`.
6. Render results as a table using `data_fmt`:
   - **Default columns:** flag (`✓`/`*`/`@`/ blank, priority `✓` > `*` > `@` > blank), status (`🔴`/`🟡`/`🟢`, header `●`), Account, 5h Left, 5h Reset, 7d Left, 7d(Son), 7d Reset, Expires, ~Renews, → Next
   - **Hidden-by-default columns:** Sub, 7d Son Reset — available via `cols::+sub`, `cols::+7d_son_reset`
   - **Column visibility:** The `cols::` parameter accepts comma-separated `+col_id` / `-col_id` modifiers relative to the default column set. The `flag` and `account` columns are structural and always visible. See [param/033_cols.md](../cli/param/033_cols.md).
   - **Composite status emoji column (`●`):** placed between the flag and Account columns; populated on every row; uses AND logic of 5h and 7d:
     - `🔴` — token read failed or API returned an error (`result` is `Err`), OR subscription cancelled (`billing_type = "none"` confirmed via `OauthAccountData`)
     - `🟡` — valid token, active subscription, either `5h Left ≤ 15.0%` or `7d Left ≤ 5.0%` (at least one quota exhausted; `result` is `Ok`)
     - `🟢` — valid token, active subscription, both `5h Left > 15.0%` and `7d Left > 5.0%` (both quotas healthy; `result` is `Ok`)
     - No JSON equivalent — the status is a display-only column derived from existing fields
   - **Per-column emoji:** `5h Left` and `7d Left` column values embed an individual 🟢/🟡 emoji based on their own threshold: `5h Left` uses ≤15% (`🟢 86%` when > 15%, `🟡 12%` when ≤ 15%); `7d Left` uses ≤5% (`🟢 65%` when > 5%, `🟡 3%` when ≤ 5%). This provides drill-down visibility beyond the composite `●`.
   - `Expires`: "in Xh Ym" when `expires_in_secs > 0`; "EXPIRED" when `expires_in_secs == 0`
   - `Sub` (hidden by default): `"max"` (`billing_type == "stripe_subscription"` + `has_max`), `"pro"` (`billing_type == "stripe_subscription"` + `!has_max`), `"—"` (`billing_type == "none"`), `"?"` (`OauthAccountData` unavailable)
   - `~Renews`: Duration countdown to next billing renewal. `"—"` when `billing_type == "none"` (subscription cancelled — no active renewal to track). **Exact** (`in Xh Ym`, no `~` prefix) when `_renewal_at` ISO-8601 UTC override is present in `{name}.json` (see [030_account_renewal_override.md](030_account_renewal_override.md)); auto-advanced by monthly increments when the override timestamp is in the past. **Estimated** (`~in Xd`, `~` prefix) when derived from `org_created_at` day-of-month projection. `"?"` when neither `_renewal_at` nor `OauthAccountData` is available; `"—"` when timestamp parsing fails.
   - `5h Left` / `7d Left`: remaining percentage (0–100, rounded to nearest integer) with per-column emoji prefix; sourced from `OauthUsageData.five_hour.utilization` / `seven_day.utilization` (0.0–100.0 scale, remaining = `100 - utilization`)
   - `7d(Son)`: remaining Sonnet-only weekly quota percentage; sourced from `OauthUsageData.seven_day_sonnet.utilization`; shows `—` when `seven_day_sonnet` is `None`
   - `5h Reset` / `7d Reset`: countdown formatted via `format_duration_secs` (capped to 2 significant units); sourced from `five_hour.resets_at` / `seven_day.resets_at` (ISO-8601 UTC string → Unix seconds via `iso_to_unix_secs`)
   - `7d Son Reset` (hidden by default): countdown to Sonnet-specific weekly reset; shows `—` when `seven_day_sonnet` is `None`
   - `→ Next`: Soonest upcoming strategic event among: `+7d` (7d quota resets, from `seven_day.resets_at`) and `$ren` (billing renewal, from `_renewal_at` override or `org_created_at` estimate). Token expiry (`!tok`) and 5h resets (`+5h`) are not included — token expiry already surfaced in `Expires` column, 5h resets already surfaced in `5h Reset` column. Format: `"in Xh Ym EVENT"` for exact timestamps; `"~in Xd $ren"` when billing source is an estimate. Shows `—` when no event has a known future timestamp.
   - Unavailable accounts show `—` for all quota columns and a shortened error reason in parentheses in the last visible quota data column (the `5h Left`–`7d Reset` range); metadata columns `Expires`, `Sub`, and `~Renews` are populated from their respective non-quota sources and are not overwritten by the error reason
   - `Sub` and `~Renews` are populated from `OauthAccountData` regardless of whether the quota fetch succeeded; `Sub` shows `"?"` when the account fetch failed; `~Renews` shows `"?"` when neither `_renewal_at` nor `OauthAccountData` is available
   - `→ Next` selects the minimum-timestamp event among `+7d` and `$ren`; events are excluded when the corresponding timestamp is absent or in the past
   - **Four-group status partition:** Before applying the sort strategy, accounts are partitioned into four status groups (see [dictionary](../cli/002_dictionary.md#status-groups)): 🟢 Green (both available) → 🟡 h-exhausted (5h exhausted, 7d available) → 🟡 weekly-exhausted (5h available, 7d exhausted) → 🔴 Red (both exhausted or error). Group order is fixed — sort strategy applies within each group only. `desc::1` reverses within groups but never changes group order.
   - **Duration format:** `format_duration_secs` output is capped to 2 significant units (e.g., `1d 2h` not `1d 2h 45m`, `3h 19m` not `3h 19m 5s`).
7. Append footer when ≥2 accounts with valid quota data exist. Footer has two `·`-delimited, column-aligned lines: (1) `Current · <name> · <model>/<effort> · N/N` identifying the `✓` account, its session model and effort from `~/.claude/settings.json`, and the valid/total count; (2) `Next (<strategy>) · <name> · <model> · <metric>` showing the recommendation for the active `sort::` strategy. When no `✓` account is detected (credentials unreadable), fall back to `Valid: N / M   session: <model>  effort: <level>` on line 1. Omit footer when 0 or 1 valid account.
7a. Append sessions table after footer when >1 `_active_*` marker file exists in the credential store. The table shows all markers (including the current machine's own) with columns `Session` (`{user}@{host}`, derived from the `_active_{host}_{user}` filename) and `Account` (file content). The current machine's own session receives `✓` in the Account column. Rendered via `data_fmt`. The `who::0` parameter suppresses the sessions table. When only 1 marker exists, the sessions table is omitted by default (no new information).
8. For `format::json`: output a JSON array with one object per account (synthetic first if present, then alphabetical saved), always including `expires_in_secs`.

**Output format (text) — saved account is live, `sort::renew` (default):**

```
Quota

  ●  Account              5h Left     5h Reset    7d Left     7d(Son)  7d Reset   Expires     ~Renews        → Next
✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%     35%      in 4d 23h  in 7h 24m   in 3h 47m      in 3h 47m $ren
  🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%     28%      in 6d 14h  in 5h 02m   ~in 6d         in 6d 14h +7d
  🟡 carol@example.com    🟡 3%      in 0h 23m  🟢 52%     12%      in 2d 11h  in 1h 12m   ~in 8d         in 2d 11h +7d
  🔴 dave@example.com     —          —           —          —        —          EXPIRED      ?              —
  🔴 eve@example.com      —          —           —          —        —          EXPIRED      ?              —

Current      · alice@example.com · sonnet/low · 3/5
Next (renew) · carol@example.com · opus       · in 2d 11h +7d
```

(Sub column hidden by default; show with `cols::+sub`. Four-group status partition: 🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red. Footer `Current` line identifies the `✓` account with session model/effort from `settings.json` and valid/total count. `Next` line shows the recommended account with metric from `→ Next` column and `opus` because carol's 7d(Son)=12% < 15% — session model override to Opus would fire on switch. Both lines use `·` delimiter with column-aligned padding.)

**Output format (text) — divergence, `sort::renew` (default):**

```
Quota

  ●  Account              5h Left     5h Reset    7d Left     7d(Son)  7d Reset   Expires     ~Renews        → Next
✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%     35%      in 4d 23h  in 7h 24m   in 3h 47m      in 3h 47m $ren
* 🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%     28%      in 6d 14h  in 5h 02m   ~in 6d         in 6d 14h +7d
  🟢 carol@example.com    🟢 95%     in 3h 44m  🟢 72%     54%      in 5d 01h  in 6h 11m   ~in 11d        in 5d 1h +7d

Current      · alice@example.com · sonnet/low · 3/3
Next (renew) · carol@example.com · sonnet     · in 5d 1h +7d
```

(`*` = active marker points here, but live credentials belong to `alice@example.com`. carol is the only eligible account. `sonnet` because carol's 7d(Son)=54% ≥ 15% — no override.)

**Output format (text) — occupied elsewhere:**

```
Quota

  ●  Account              5h Left     5h Reset    7d Left     7d(Son)  7d Reset   Expires     ~Renews        → Next
✓ 🟢 alice@example.com    🟢 86%     in 3h 19m  🟢 65%     35%      in 4d 23h  in 7h 24m   in 3h 47m      in 3h 47m $ren
@ 🟢 bob@example.com      🟢 100%    in 4h 58m  🟢 88%     28%      in 6d 14h  in 5h 02m   ~in 6d         in 6d 14h +7d
  🟢 carol@example.com    🟢 95%     in 3h 44m  🟢 72%     54%      in 5d 01h  in 6h 11m   ~in 11d        in 5d 1h +7d

Current      · alice@example.com · sonnet/low · 3/3
Next (renew) · carol@example.com · sonnet     · in 5d 1h +7d
```

(`@` = bob is the active account on another machine: some other machine's `_active_*` marker in the credential store names bob, while this machine's own marker names alice, and alice is also the live session. `is_occupied_elsewhere = true` for bob; `is_active = false` and `is_current = false` for bob on this machine. No higher-priority flag applies, so bob receives `@`.)

**Output format (text) — unsaved account is live (synthetic row):**

```
Quota

  ●  Account              5h Left     5h Reset    7d Left     7d(Son)  7d Reset   Expires     ~Renews        → Next
✓ 🟢 (current session)    🟢 64%     in 1h 39m  🟢 39%     —        in 3d 17h  in 4h 39m   ?              in 3d 17h +7d
  🟢 alice@example.com    🟢 100%    in 4h 58m  🟢 88%     28%      in 6d 14h  in 5h 02m   in 3h 47m      in 3h 47m $ren
  🔴 bob@example.com      —          —           —          —        —          EXPIRED      ?              —

Current          · (current session)   · sonnet/low · 2/3
Next (renew)     · alice@example.com   · sonnet     · in 3h 47m $ren
```

**Output format (JSON):**

```json
[
  {"account":"alice@example.com","is_current":true,"is_active":false,"is_occupied_elsewhere":false,"expires_in_secs":26640,"billing_type":"stripe_subscription","has_max":true,"renewal_secs":13620,"renewal_is_estimate":false,"next_event_type":"ren","next_event_secs":13620,"session_5h_left_pct":86,"session_5h_resets_in_secs":11940,"weekly_7d_left_pct":65,"weekly_7d_sonnet_left_pct":35,"weekly_7d_resets_in_secs":432540},
  {"account":"bob@example.com","is_current":false,"is_active":true,"is_occupied_elsewhere":false,"expires_in_secs":18120,"billing_type":"stripe_subscription","has_max":true,"renewal_secs":518400,"renewal_is_estimate":true,"next_event_type":"7d","next_event_secs":500040,"session_5h_left_pct":100,"session_5h_resets_in_secs":17880,"weekly_7d_left_pct":88,"weekly_7d_sonnet_left_pct":28,"weekly_7d_resets_in_secs":500040},
  {"account":"carol@example.com","is_current":false,"is_active":false,"is_occupied_elsewhere":false,"expires_in_secs":0,"billing_type":null,"has_max":null,"renewal_secs":null,"renewal_is_estimate":null,"next_event_type":null,"next_event_secs":null,"error":"missing accessToken"},
  {"account":"dave@example.com","is_current":false,"is_active":false,"is_occupied_elsewhere":false,"expires_in_secs":0,"billing_type":null,"has_max":null,"renewal_secs":null,"renewal_is_estimate":null,"next_event_type":null,"next_event_secs":null,"error":"missing accessToken"}
]
```

(`weekly_7d_sonnet_left_pct` is `null` when `seven_day_sonnet` is absent from the API response. `billing_type`, `has_max`, `renewal_secs`, `renewal_is_estimate`, `next_event_type`, and `next_event_secs` are `null` when the account fetch failed or the token could not be read. `is_occupied_elsewhere` is always a bool — `true` when any other machine's `_active_*` marker names this account.)

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
- **AC-03**: Accounts with expired or missing tokens show `—` in quota columns and a shortened error reason in the final visible quota data column (`5h Left` through `7d Reset` range). Metadata columns `Expires`, `Sub`, and `~Renews` are populated from their respective non-quota sources (`expires_at_ms`, `OauthAccountData`) and are not overwritten by the error reason. The error label is context-aware: HTTP 429 + `billing_type == "none"` (confirmed cancelled subscription) → `"no subscription"`; HTTP 429 with active subscription or unknown billing state → `"rate limited (429)"`. (**BUG-220 ✅ Closed** 2026-05-30; **BUG-231 ✅ Fixed** 2026-06-03)
- **AC-04**: Table output is rendered by `data_fmt`.
- **AC-05**: `format::json` returns a valid JSON array with one object per account; each object includes `expires_in_secs`, `is_current` (bool), `is_active` (bool), `is_occupied_elsewhere` (bool), `is_owned` (bool — Feature 036), `billing_type` (string or `null`), `has_max` (bool or `null`), `renewal_secs` (u64 or `null`), `renewal_is_estimate` (bool or `null`), `next_event_type` (string or `null`), and `next_event_secs` (u64 or `null`); successful rows also include `session_5h_left_pct`, `weekly_7d_left_pct`, and `weekly_7d_sonnet_left_pct` (all remaining, not consumed); `weekly_7d_sonnet_left_pct` is `null` when Sonnet quota data is absent from the API response; `billing_type`, `has_max`, `renewal_secs`, `renewal_is_estimate`, `next_event_type`, and `next_event_secs` are `null` when the account fetch failed.
- **AC-06**: Missing credential store exits 2 with an actionable error message.
- **AC-07**: The `Expires` column shows token TTL ("in Xh Ym") for valid tokens and "EXPIRED" for tokens whose `expiresAt` is in the past; sourced from the credential file without an API call.
- **AC-08**: `5h Left` and `7d Left` show remaining quota percentage (100 − consumed); `7d(Son)` shows remaining Sonnet-only weekly quota (100 − consumed) or `—` when absent; `5h Reset` and `7d Reset` show independent reset countdowns as separate columns; all quota data sourced from `claude_quota::fetch_oauth_usage()` → `OauthUsageData`.
- **AC-17**: `7d(Son)` column is populated when `OauthUsageData.seven_day_sonnet` is `Some`; shows `—` when `None`. JSON field `weekly_7d_sonnet_left_pct` is an integer when present and `null` when absent.
- **AC-09**: The recommended next account is determined by the `sort::` parameter (see [020_usage_sort_strategies.md](020_usage_sort_strategies.md)). The top eligible account in the active sort order is shown in the footer's `Next (strategy):` line. Default is `sort::renew`.
- **AC-10**: A footer is appended when ≥2 accounts have valid quota data; the footer is absent when 0 or 1 valid account. The footer has two `·`-delimited, column-aligned lines: (1) `Current · <name> · <model>/<effort> · N/N` — identifies the `✓` account by name; `<model>` is the session model from `~/.claude/settings.json` (`model` field, shortened to `sonnet`/`opus`/`haiku`); `<effort>` is the effort level from `settings.json` (`effortLevel` field); when effort is absent, shows `<model>` only (no slash); `N/N` = valid count / total count. (2) `Next (<strategy>) · <name> · <model> · <metric>` — `<model>` is `opus` when `seven_day_sonnet` exists and `sonnet_left < 15%`, `sonnet` otherwise; `<metric>` uses `→ Next` format (soonest strategic event). When no `✓` account is detected (credentials file unreadable), line 1 falls back to `Valid: N / M   session: <model>  effort: <level>` (legacy format). Column padding aligns the `·` delimiters vertically across both lines.
- **AC-11**: When the live `~/.claude/.credentials.json` token does not match any saved account's token, a synthetic row is prepended at the top of the table with `✓`, quota fetched via the live token, and the name set to the email from `~/.claude.json` (or `(current session)` when that file is unavailable or the field is empty).
- **AC-12**: When `~/.claude/.credentials.json` is unreadable, no `✓` is emitted on any row; all saved accounts are still shown.
- **AC-13**: `*` in the flag column marks the account with the per-machine active marker when it differs from the current (live) account; no `*` appears when active and current are the same account.
- **AC-14**: When current = active (normal case), only `✓` appears on the current row; no `*` is emitted on any row.
- **AC-15**: When `~/.claude/.credentials.json` is unreadable, no `✓` is emitted; `*` is still emitted for the active account. See [016_current_account_awareness.md](016_current_account_awareness.md).
- **AC-16**: `format::json` output uses `is_current` (replacing the former `active` field) and includes a new `is_active` boolean field per object.
- **AC-18**: Every table row has a composite status emoji in the `●` column (second column, after flag) using AND logic: `🟢` when `result` is `Ok` and subscription active and `5h Left > 15%` and `7d Left > 5%`, `🟡` when `result` is `Ok` and subscription active and either `5h Left ≤ 15%` or `7d Left ≤ 5%`, `🔴` when `result` is `Err` OR when `billing_type = "none"` (cancelled subscription — permanently unusable regardless of quota values; Fix BUG-317). `account = None` (API fetch failed) is NOT classified as cancelled — absent data is ambiguous. The emoji appears on every row including the synthetic current-session row.
- **AC-19**: The exhaustion boundary for composite `●` is exclusive for `🟢` and inclusive for `🟡`: 5h dimension uses 15% (`5h Left = 15%` → `🟡`; `> 15%` needed for `🟢`), 7d dimension uses 5% (`7d Left = 5%` → `🟡`; `> 5%` needed for `🟢`).
- **AC-20**: The `●` status emoji column has no JSON equivalent — `format::json` output is unchanged; pipeline consumers derive status from `session_5h_left_pct`, `weekly_7d_left_pct`, and the `error` field.
- **AC-21**: `5h Left` and `7d Left` column values each embed a per-column emoji prefix using their respective thresholds: `5h Left` shows `🟢` when `> 15%`, `🟡` when `≤ 15%`; `7d Left` shows `🟢` when `> 5%`, `🟡` when `≤ 5%`. This provides individual-dimension visibility beyond the composite `●`.
- **AC-22**: `Sub` column is hidden by default; shown via `cols::+sub`. `7d Son Reset` column is hidden by default; shown via `cols::+7d_son_reset`.
- **AC-23**: `cols::` parameter accepts comma-separated `+col_id` / `-col_id` modifiers. `flag` and `account` columns are structural and always visible. Invalid column IDs exit 1 with an error naming valid column IDs.
- **AC-24**: Four-group status partition: accounts are partitioned 🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red before any sort strategy is applied. Sort strategy applies within each status group. Group order is never reversed by `desc::`. See [dictionary](../cli/002_dictionary.md#status-groups).
- **AC-25**: `format_duration_secs` output is capped to 2 significant units: shows at most 2 time components (e.g., `1d 2h`, `3h 19m`, `23m`), never 3.
- **AC-26**: h-exhausted accounts (status group 2: `5h Left ≤ 15%`, `7d Left > 5%`) rank above weekly-exhausted accounts (status group 3: `5h Left > 15%`, `7d Left ≤ 5%`). Accounts with both `5h Left ≤ 15%` and `7d Left ≤ 5%` fall in status group 4 (🔴 Red). Sort strategy applies within each status group. Group order is never reversed by `desc::`.
- **AC-27**: `~Renews` column shows `"—"` when `billing_type == "none"` (subscription cancelled — no active renewal to track); shows `in Xh Ym` (exact duration, no `~` prefix) when `_renewal_at` is present in `{name}.json` and auto-advances monthly when past; shows `~in Xd` (with `~` prefix, 2 significant units max) when only `org_created_at` is available; shows `"?"` when neither source is available; shows `"—"` when timestamp parsing fails. (**BUG-232 ✅ Fixed** 2026-06-03)
- **AC-28**: `→ Next` column shows the chronologically soonest strategic event among `+7d` (7d quota reset from `seven_day.resets_at`) and `$ren` (billing renewal from `_renewal_at` override or `org_created_at` estimate). Token expiry (`!tok`) and 5h resets (`+5h`) are not candidates — token expiry is already surfaced in the `Expires` column, and 5h resets are already surfaced in the `5h Reset` column. Format: `"in Xh Ym EVENT"` for exact sources; `"~in Xd $ren"` when billing source is an estimate. Shows `—` when no event has a known future timestamp. Events with absent or past timestamps are excluded. Selection: minimum-seconds candidate wins; ties broken by iteration order `+7d` → `$ren`.

  **Next Event Type Registry:**

  | Prefix | Event | Source field | Estimated form | Excluded when |
  |--------|-------|-------------|----------------|---------------|
  | `+7d`  | 7d weekly quota reset | `seven_day.resets_at` from API | — (always exact) | `resets_at` absent or past |
  | `$ren` | Billing renewal | `_renewal_at` override or `org_created_at` estimate | `~in Xd $ren` (tilde prefix) | absent or past |
  | `—`    | No event | — | — | both sources absent or past |
- **AC-29**: `format::json` output includes `renewal_secs` (u64 seconds to next billing renewal, or `null`), `renewal_is_estimate` (`true` when sourced from `org_created_at`, `false` when from `_renewal_at`, or `null`), `next_event_type` (string event label `"7d"` or `"ren"` — sigil characters `+` and `$` are stripped in JSON output — or `null` when no event has a future timestamp), and `next_event_secs` (u64 seconds to next event, or `null`). Note: `get::next_event_type` preserves the display sigil and outputs `+7d` or `$ren` (see [feature/028_usage_row_filtering.md](028_usage_row_filtering.md)).
- **AC-30**: Accounts with `is_occupied_elsewhere = true` — their name appears in any `_active_*` marker file in the credential store other than the current machine's own marker (as returned by `other_machines_active(store)`) — receive `@` in the flag column when `is_active = false` AND `is_current = false`. Flag priority chain: `✓` > `*` > `@` > blank; an account receives at most one flag character per row. `format::json` output includes `is_occupied_elsewhere` (bool) per object. `format::json` never emits `@` — the field is a bool, not the single-character flag.

- **AC-32**: After the touch loop, `.usage` applies `apply_model_override()` for the **current** account (`is_current == true`): when the current account has valid quota data (`result` is `Ok`) and `seven_day_sonnet` remaining is below 15%, and the session model in `~/.claude/settings.json` is `"claude-sonnet-4-6"` (or empty), overwrites the session model with `"claude-opus-4-6"`. This ensures the interactive session switches to Opus when Sonnet quota is nearly exhausted, even without an `.account.use` switch event. When `trace::1`, emits `[trace] usage  {name}  model override: sonnet→opus (7d(Son) left={N}%)` to stderr when the override fires. Bidirectional (Fix BUG-311): when `seven_day_sonnet` is `Some` and `≥ 15%`, or when `seven_day_sonnet` is `None` (absent tier treated as unknown, not exhausted — Fix BUG-300), writes `"sonnet"` conservatively via `override_session_model_to_sonnet()`. Trace emits `opus→sonnet` when model changes. No-op when the current account has no quota data or when the session model already matches the target value. (Fix for BUG-244, BUG-311.)

- **AC-33**: A sessions table is appended after the footer when >1 `_active_*` marker file exists in the credential store. The table has two columns: `Session` (`{user}@{host}`, derived by stripping the `_active_` prefix from the marker filename and splitting at the boundary between host and user) and `Account` (file content, trimmed). The current machine's own session receives `✓` appended to the Account value. Rendered via `data_fmt`. When only 1 marker exists, the sessions table is omitted by default.
- **AC-34**: `who::0` suppresses the sessions table unconditionally (even when >1 marker exists). `who::1` forces the sessions table on (even when ≤1 marker). Default behavior (omit parameter) is auto: shown when >1 marker, hidden when ≤1.

- **AC-31**: When `OauthAccountData.billing_type == "none"` (confirmed cancelled subscription via a successful account fetch), the account's per-fetch result is overridden to `Err("no subscription")` — the `GET /api/oauth/usage` HTTP response is discarded regardless of its status code. This makes the result semantically correct at the data layer and removes the need for context-aware display logic in `render.rs`. `~Renews` shows `"—"` for these accounts (AC-27). When `billing_type` is unknown (`OauthAccountData` fetch failed), the raw usage fetch result and standard error mapping apply. Trace behavior: when `trace::1`, `[trace] result:` is emitted AFTER the Class A override, so the trace correctly reflects the final stored result (not the raw API response). (**BUG-233 ✅ Fixed** 2026-06-03; **BUG-234 ✅ Fixed** 2026-06-03 — trace ordering)

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/244_usage_command_never_applies_model_override.md` | BUG-244 ✅ Fixed: `apply_model_override()` call added to `usage_routine()` before row-filter pipeline; `label: &str` param added to distinguish `.usage` from `.account.use` trace prefix (TSK-249) |
| `task/claude_profile/bug/300_model_override_none_sonnet_triggers_opus.md` | BUG-300 ✅ Fixed (TSK-302): `apply_model_override()` used `map_or(0.0, ...)` on `quota.seven_day_sonnet`; when `None`, returned 0.0 < 20.0 → Opus override fired unconditionally for accounts without a Sonnet tier. Fix: `if let Some(ref sonnet) = quota.seven_day_sonnet` guard at `api.rs:267`; `mre_bug300_model_override_absent_sonnet_no_override` added to `api_tests.rs` |
| `task/claude_profile/bug/317_cancelled_subscription_misclassified_weekly_exhausted.md` | BUG-317 ✅ Fixed: Cancelled accounts (`billing_type="none"`) misclassified as 🟡/🟢. Fix A: `status_group_of()` gates on `billing_type` before quota thresholds (`sort.rs`). Fix B: `find_first_eligible()` skips cancelled accounts (`sort_next.rs`). Fix C: `status_emoji()` signature changed to `&AccountQuota`; gates on `billing_type` before quota thresholds (`format.rs`). Fix D: `only_valid` filter excludes cancelled; `exclude_exhausted` auto-fixed via Fix C (`api.rs`). MREs: `mre_bug317_cancelled_status_emoji_is_red`, `mre_bug317_cancelled_not_recommended_by_find_next`, `mre_bug317_cancelled_account_status_group_is_red`, `mre_bug317_cancelled_excluded_by_only_valid` |

### Commands

| File | Relationship |
|------|--------------|
| [command/006_usage.md](../cli/command/006_usage.md#command--9-usage) | CLI command specification |

### Dependencies

| File | Relationship |
|------|--------------|
| `claude_quota` | `fetch_oauth_usage()`, `fetch_oauth_account()` — transport functions; `OauthUsageData`, `OauthAccountData`, `PeriodUsage` types |
| `data_fmt` | Table rendering for all output |

### Features

| File | Relationship |
|------|--------------|
| [013_account_limits.md](013_account_limits.md) | `.account.limits` command for single-account quota |
| [016_current_account_awareness.md](016_current_account_awareness.md) | Shared current-account detection algorithm; `*` flag semantics; JSON field renaming |
| [017_token_refresh.md](017_token_refresh.md) | Token refresh extension; `apply_refresh()` and `refresh::` parameter |
| [020_usage_sort_strategies.md](020_usage_sort_strategies.md) | Sort strategies; four-group status partition; footer recommendation (driven by `sort::`, `desc::`, `prefer::` parameters) |
| [024_session_touch.md](024_session_touch.md) | Session touch; idle 5h window activation; `touch::` parameter |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | `_active_*` naming convention; `other_machines_active()` — reads non-own markers; `@` occupied-elsewhere flag source |
| [030_account_renewal_override.md](030_account_renewal_override.md) | `.account.renewal` command; `_renewal_at` field lifecycle; `~Renews` exact vs. estimated rendering |
| [033_quota_cache.md](033_quota_cache.md) | Quota cache fallback — persist last-known quota in `{name}.json`; display cached values with `~` prefix when live fetch fails |
| [036_account_ownership.md](036_account_ownership.md) | G1 gate: non-owned accounts bypass token read and HTTP; use cache as primary source; `is_owned` JSON field |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/025_sort.md](../cli/param/025_sort.md) | `sort::` parameter specification (drives row ordering and footer recommendation) |
| [cli/param/033_cols.md](../cli/param/033_cols.md) | `cols::` parameter specification |
| [cli/param/034_touch.md](../cli/param/034_touch.md) | `touch::` parameter specification |
| [cli/param/049_at.md](../cli/param/049_at.md) | `at::` — absolute renewal timestamp for `.account.renewal` |
| [cli/param/050_from_now.md](../cli/param/050_from_now.md) | `from_now::` — relative renewal delta for `.account.renewal` |
| [cli/param/051_clear.md](../cli/param/051_clear.md) | `clear::` — remove `_renewal_at` override; restores `~`-prefixed estimate in `~Renews` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../cli/command/006_usage.md#command--9-usage) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/usage/api.rs`, `src/usage/fetch.rs`, `src/usage/render.rs` | `usage_routine()` CLI handler (incl. `apply_model_override` for current account — AC-32), quota fetching, table rendering, JSON output |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/usage_test.rs` | All-accounts quota table and JSON output tests |

### Algorithm Docs

| File | Relationship |
|------|-------------|
| [algorithm/003_quota_status_groups.md](../algorithm/003_quota_status_groups.md) | 4-group status partition — `status_group_of()` drives row ordering and composite `●` emoji |
| [algorithm/004_eligibility_gates.md](../algorithm/004_eligibility_gates.md) | 8 eligibility gates — filter candidates for footer recommendation and auto-switch |
| [algorithm/005_next_account_selection.md](../algorithm/005_next_account_selection.md) | Positive selection — 3-step algorithm producing the `Next` footer recommendation |
| [algorithm/007_sort_strategies.md](../algorithm/007_sort_strategies.md) | `sort::` strategies and `prefer_weekly` computation |
