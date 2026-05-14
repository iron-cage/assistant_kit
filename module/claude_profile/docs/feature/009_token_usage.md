# Feature: All-Accounts Live Quota Reporting

### Scope

- **Purpose**: Surface live rate-limit utilization for all saved accounts from Anthropic API response headers.
- **Responsibility**: Documents the `usage` module and `.usage` CLI command.
- **In Scope**: Per-account quota fetch via `anthropic-ratelimit-unified-*` headers, token expiry from credential files (`expires_at_ms`), table output using `data_fmt`, graceful handling of expired/missing tokens, recommendation marker for best next account, footer summary line, `format::json` output.
- **Out of Scope**: Historical token counts from stats-cache.json (replaced by live API data); verbosity levels (single fixed output level per command design).

### Design

`claude_profile` CLI provides a `.usage` command that fetches live quota utilization for every saved account by making a minimal API call per account and reading rate-limit headers from the response. Results are displayed as a table.

**Algorithm:**
1. Read the credential store — enumerate all saved accounts (`{credential_store}/*.credentials.json`) via `account::list()`; each `Account` struct includes `expires_at_ms`.
2. Identify the active account from the `_active` marker (resolved by `account::list()`).
3. For each saved account (in alphabetical order):
   a. Compute `expires_in_secs = saturating_sub(expires_at_ms / 1000, now_secs)`.
   b. Read the account's `accessToken` from the credential file.
   c. If token read succeeds: call `claude_quota::fetch_rate_limits(&token)` → `RateLimitData` or error reason.
   d. On quota success: record `5h Left = (1.0 - utilization_5h) * 100`, `reset_5h`, `7d Left = (1.0 - utilization_7d) * 100`, `reset_7d`, `status`.
   e. On any failure (token read or API): record the error reason.
4. Post-process:
   a. Mark the active account with `✓` in the flag column.
   b. From non-active accounts with valid quota data and `expires_in_secs > 0`, select the one with the highest `5h Left`; mark it `→` (recommended next). If no such account exists, no `→` is emitted.
5. Render results as a table using `data_fmt`:
   - Columns: flag (`✓`/`→`/ ), Account, Expires, 5h Left, 5h Reset, 7d Left, 7d Reset, Status
   - `Expires`: "in Xh Ym" when `expires_in_secs > 0`; "EXPIRED" when `expires_in_secs == 0`
   - `5h Left` / `7d Left`: remaining percentage (0–100, rounded to nearest integer)
   - `5h Reset` / `7d Reset`: countdown formatted via `format_duration_secs`
   - Unavailable accounts show `—` for quota columns and shortened error reason in Status
6. Append footer line when ≥2 accounts with valid quota data exist:
   `Valid: X / Y   →  Next: name  (N% session left, token expires in Xh Ym)`
   Omit footer when 0 or 1 valid account.
7. For `format::json`: output a JSON array with one object per account, always including `expires_in_secs`.

**Output format (text):**

```
Quota

  Account          Expires     5h Left  5h Reset    7d Left  7d Reset     Status
✓ i12@wbox.pro    in 7h 24m  86%      in 3h 19m  65%      in 4d 23h   allowed
→ i6@wbox.pro     in 5h 02m  100%     in 4h 58m  88%      in 6d 14h   allowed
  i7@wbox.pro     EXPIRED    —        —           —        —            (missing accessToken)
  i8@wbox.pro     EXPIRED    —        —           —        —            (missing accessToken)

Valid: 2 / 4   →  Next: i6@wbox.pro  (100% session left, token expires in 5h 02m)
```

**Output format (JSON):**

```json
[
  {"account":"i12@wbox.pro","active":true,"expires_in_secs":26640,"session_5h_left_pct":86,"session_5h_resets_in_secs":11940,"weekly_7d_left_pct":65,"weekly_7d_resets_in_secs":432540,"status":"allowed"},
  {"account":"i6@wbox.pro","active":false,"expires_in_secs":18120,"session_5h_left_pct":100,"session_5h_resets_in_secs":17880,"weekly_7d_left_pct":88,"weekly_7d_resets_in_secs":500040,"status":"allowed"},
  {"account":"i7@wbox.pro","active":false,"expires_in_secs":0,"error":"missing accessToken"},
  {"account":"i8@wbox.pro","active":false,"expires_in_secs":0,"error":"missing accessToken"}
]
```

**Table rendering:** All table and tree output MUST use the `data_fmt` crate. No hand-rolled string formatting.

**Error handling:**
- `HOME` unset → `InternalError`
- Credential store unreadable → `InternalError`
- Individual account token expired or invalid → inline `error` field in that row (non-fatal; other accounts still processed)
- Empty credential store → empty table with `(no accounts configured)` message

### Acceptance Criteria

- **AC-01**: `.usage` fetches quota for every saved account, not only the active one.
- **AC-02**: The active account row has `✓` in the flag column; all other rows have a blank flag or `→`.
- **AC-03**: Accounts with expired or missing tokens show `—` in quota columns and a shortened error reason in Status.
- **AC-04**: Table output is rendered by `data_fmt`.
- **AC-05**: `format::json` returns a valid JSON array with one object per account; each object includes `expires_in_secs`; successful rows use `session_5h_left_pct` and `weekly_7d_left_pct` (remaining, not consumed).
- **AC-06**: Missing credential store exits 2 with an actionable error message.
- **AC-07**: The `Expires` column shows token TTL ("in Xh Ym") for valid tokens and "EXPIRED" for tokens whose `expiresAt` is in the past; sourced from the credential file without an API call.
- **AC-08**: `5h Left` and `7d Left` show remaining quota percentage (100 − consumed); `5h Reset` and `7d Reset` show independent reset countdowns as separate columns.
- **AC-09**: The `→` flag marks the non-active account with the highest remaining session quota among those with valid quota data and a non-expired token; no `→` is emitted when no such account exists.
- **AC-10**: A footer line "Valid: X / Y   →  Next: name  (...)" is appended when ≥2 accounts have valid quota data; the footer is absent when 0 or 1 valid account.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `usage_routine()` CLI handler, quota fetching, table rendering, JSON output |
| source | `src/commands.rs` | Re-exports `usage_routine()` from `src/usage.rs` |
| dep | `claude_quota` | `fetch_rate_limits()` transport function |
| dep | `data_fmt` | Table rendering for all output |
| test | `tests/cli/usage_test.rs` | All-accounts quota table and JSON output tests |
| doc | [013_account_limits.md](013_account_limits.md) | `.account.limits` command for single-account quota |
| doc | [cli/commands.md](../cli/commands.md#command--9-usage) | CLI command specification |
