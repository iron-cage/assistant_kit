# Feature: Account Rate-Limit Utilization

### Scope

- **Purpose**: Show plan and rate-limit utilization for a selected account so the user can see how much of their allocation remains before hitting limits.
- **Responsibility**: Documents the `.account.limits` command requirements (FR-18). L3 implemented: happy-path (ureq HTTP client, `anthropic-ratelimit-unified-*` headers) and error-path (lim01–lim05) both passing.
- **In Scope**: Command output structure, verbosity levels, account selection, confirmed data source (HTTP response headers).
- **Out of Scope**: Implementation details (→ implementation task), historical token counts (→ 009_token_usage.md).

### Design

`.account.limits` must display plan/rate-limit utilization for the selected account.

**Output (at default verbosity):** Modelled on the Claude Code settings panel limits view:
- Session usage (5-hour window) — percentage consumed and reset time
- Weekly all-model usage — percentage consumed and reset time
- Weekly Sonnet usage — percentage consumed and reset time

**Account selection:** Uses `name::` parameter (optional) to select which account to inspect. When omitted, shows limits for the active account.

**Parameters:** `name::` (optional), `v::`, `format::` — consistent with other `.account.*` commands.

**Data source (confirmed):** Rate-limit utilization comes exclusively from HTTP response headers returned by the Anthropic API. Claude Code fetches these by making a lightweight `POST /v1/messages` (`max_tokens: 1`, content `"quota"`) and reading the response headers. Headers:

| Header | Value | Meaning |
|--------|-------|---------|
| `anthropic-ratelimit-unified-5h-utilization` | `0.0–1.0` | 5-hour session window consumed |
| `anthropic-ratelimit-unified-5h-reset` | Unix timestamp | Reset time for 5h window |
| `anthropic-ratelimit-unified-7d-utilization` | `0.0–1.0` | Weekly all-model consumed |
| `anthropic-ratelimit-unified-7d-reset` | Unix timestamp | Reset time for 7d window |
| `anthropic-ratelimit-unified-status` | `allowed` / `allowed_warning` / `rejected` | Current rate-limit state |

These headers are never cached locally — no local file contains them. `stats-cache.json` has raw token counts only.

**Current implementation state:** Error paths (lim01–lim05) and happy path (AC-01 through AC-03) are implemented. `ureq` v2 HTTP client is gated under the `enabled` feature. The command makes a lightweight `POST /v1/messages` (`max_tokens: 1`, `content: "quota"`) and reads `anthropic-ratelimit-unified-*` response headers. IT-1 through IT-5 require a live API call and are tracked in `tests/manual/readme.md`.

**Exit codes:**
- 0: success
- 1: invalid `name::` characters (usage error)
- 2: runtime error (account not found, data unavailable, HOME unset)

### Acceptance Criteria

- **AC-01**: `.account.limits` shows current session, weekly all-model, and weekly Sonnet utilization for the active account.
- **AC-02**: `.account.limits name::work` shows limits for the named account.
- **AC-03**: `format::json` returns structured JSON with utilization fields.
- **AC-04**: Missing data source → exits 2 with an actionable error (not a silent zero).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [009_token_usage.md](009_token_usage.md) | Related: historical token counts from stats-cache.json (distinct data) |
| doc | [011_account_status_by_name.md](011_account_status_by_name.md) | Related: account selection pattern via `name::` |
| doc | [cli/commands.md](../cli/commands.md) | CLI commands table (row 12) and command detail section |
| source | `src/commands.rs` | `account_limits_routine()` — fully implemented via `ureq` HTTP client (feature-gated) |
| test | `tests/` — `lim01–lim04` | Error-path coverage: not-found, no credentials, data unavailable, invalid chars |
| doc | [cli/testing/command/account_limits.md](../cli/testing/command/account_limits.md) | Manual integration test specification |
