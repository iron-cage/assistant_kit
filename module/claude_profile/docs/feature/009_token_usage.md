# Feature: All-Accounts Live Quota Reporting

### Scope

- **Purpose**: Surface live rate-limit utilization for all saved accounts from Anthropic API response headers.
- **Responsibility**: Documents the `usage` module and `.usage` CLI command.
- **In Scope**: Per-account quota fetch via `anthropic-ratelimit-unified-*` headers, table output using `data_fmt`, graceful handling of expired/missing tokens, `format::json` output.
- **Out of Scope**: Historical token counts from stats-cache.json (replaced by live API data); verbosity levels (single fixed output level per command design).

### Design

`claude_profile` CLI provides a `.usage` command that fetches live quota utilization for every saved account by making a minimal API call per account and reading rate-limit headers from the response. Results are displayed as a table.

**Algorithm:**
1. Read the credential store — enumerate all saved accounts (`{credential_store}/*.credentials.json`).
2. Identify the active account from the `_active` marker.
3. For each saved account (in alphabetical order):
   a. Load the account's credential token.
   b. Call `claude_quota::fetch_rate_limits(&token)` → `RateLimitData` or error.
   c. On success: record `utilization_5h`, `reset_5h`, `utilization_7d`, `reset_7d`, `status`.
   d. On error: record the error reason (e.g., "expired token").
4. Render results as a table using `data_fmt`:
   - Columns: Account, Session (5h), Weekly (7d), Status
   - Active account marked with `(✓)` inline in the Account column
   - Unavailable accounts show `—` for quota columns and error reason in Status column
5. For `format::json`: output a JSON array with one object per account.

**Output format (text):**

```
Quota

  Account              Session (5h)    Weekly (7d)     Status
  i12@wbox.pro (✓)    0% / 1h 50m    0% / 6d 14h    allowed
  i6@wbox.pro          0% / 1h 58m    0% / 6d 14h    allowed
  i7@wbox.pro          —               —               (expired token)
  i8@wbox.pro          —               —               (expired token)
```

**Output format (JSON):**

```json
[
  {"account":"i12@wbox.pro","active":true,"session_5h_pct":0,"session_5h_resets_in_secs":6600,"weekly_7d_pct":0,"weekly_7d_resets_in_secs":570240,"status":"allowed"},
  {"account":"i6@wbox.pro","active":false,"session_5h_pct":0,"session_5h_resets_in_secs":7080,"weekly_7d_pct":0,"weekly_7d_resets_in_secs":570240,"status":"allowed"},
  {"account":"i7@wbox.pro","active":false,"error":"expired token"},
  {"account":"i8@wbox.pro","active":false,"error":"expired token"}
]
```

**Table rendering:** All table and tree output MUST use the `data_fmt` crate. No hand-rolled string formatting.

**Current implementation state:** Blocked on `data_fmt` crate. Interim implementation reads `stats-cache.json` and reports per-model historical token totals for the 7-day window. `claude_quota` is already a workspace dependency in `Cargo.toml` (gated under `enabled`). Full implementation unblocks when `data_fmt` is added to the workspace. The interim implementation MUST warn when `lastComputedDate` is more than 14 days in the past by prepending `⚠ Data last updated {date} ({N} days ago) — run Claude Code to refresh` to the text output.

**Error handling:**
- `HOME` unset → `InternalError`
- Credential store unreadable → `InternalError`
- Individual account token expired or invalid → inline `error` field in that row (non-fatal; other accounts still processed)
- Empty credential store → empty table with `(no accounts configured)` message

### Acceptance Criteria

- **AC-01**: `.usage` fetches quota for every saved account, not only the active one.
- **AC-02**: The active account is marked `(✓)` in the Account column.
- **AC-03**: Accounts with expired or missing tokens show `—` in quota columns and an error reason in Status.
- **AC-04**: Table output is rendered by `data_fmt`.
- **AC-05**: `format::json` returns a valid JSON array with one object per account.
- **AC-06**: Missing credential store exits 2 with an actionable error message.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `usage_routine()` CLI handler (interim), quota fetching, table rendering, JSON output |
| source | `src/commands.rs` | Re-exports `usage_routine()` from `src/usage.rs` |
| dep | `claude_quota` | `fetch_rate_limits()` transport function |
| dep | `data_fmt` | Table rendering for all output |
| test | `tests/cli/usage_test.rs` | All-accounts quota table and JSON output tests |
| doc | [013_account_limits.md](013_account_limits.md) | `.account.limits` command for single-account quota |
| doc | [cli/commands.md](../cli/commands.md#command--9-usage) | CLI command specification |
