# Feature: Account Rate-Limit Utilization

### Scope

- **Purpose**: Show plan and rate-limit utilization for a selected account so the user can see how much of their allocation remains before hitting limits.
- **Responsibility**: Documents the `.account.limits` command and its rate-limit utilization output (FR-18).
- **In Scope**: Command output structure, verbosity levels, account selection, confirmed data source (HTTP response headers).
- **Out of Scope**: Implementation details (→ implementation task), historical token counts (→ 009_token_usage.md).

### Design

`.account.limits` must display plan/rate-limit utilization for the selected account.

**Output (at default verbosity):**
- Session usage (5-hour window) — percentage consumed and reset time
- Weekly all-model usage (7-day window) — percentage consumed and reset time
- Rate-limit status — `allowed`, `allowed_warning`, or `rejected`

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

**Current implementation state:** Error paths (lim01–lim05) and happy path (AC-01 through AC-03) fully implemented. HTTP transport is delegated to `claude_quota::fetch_rate_limits(token)` (gated under the `enabled` feature via `dep:claude_quota`). The command reads credentials from disk, then calls `claude_quota` which makes a lightweight `POST /v1/messages` (`max_tokens: 1`, `content: "quota"`) and reads `anthropic-ratelimit-unified-*` response headers. IT-1 (`v::1` default), IT-2 (`v::0` compact), IT-3 (`format::json`), and IT-5 (`v::2` verbose) are automated live API tests in `tests/cli/account_limits_test.rs`. IT-4 (named account `name::alice@acme.com`) is manual-only — requires a saved account and is tracked in `tests/manual/readme.md`.

**Verbosity dispatch:** Output format dispatches on both `opts.format` (outer: `json` vs `text`) and `opts.verbosity` (inner, only when `text`): `0` → compact (bare percentages + status, no labels or reset times), `1` (default) → labelled with reset durations, `2` → verbose with raw floats and Unix timestamps. The inner verbosity match is SEPARATE from the outer format match — omitting the inner match is a silent bug where all verbosity levels produce identical `v::1` output.

**Exit codes:**
- 0: success
- 1: invalid `name::` characters (usage error)
- 2: runtime error (account not found, data unavailable, HOME unset)

### Acceptance Criteria

- **AC-01**: `.account.limits` shows current session (5h) utilization, weekly all-model (7d) utilization, and rate-limit status for the active account.
- **AC-02**: `.account.limits name::alice@acme.com` shows limits for the named account.
- **AC-03**: `format::json` returns structured JSON with utilization fields.
- **AC-04**: Missing data source → exits 2 with an actionable error (not a silent zero).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [009_token_usage.md](009_token_usage.md) | Related: historical token counts from stats-cache.json (distinct data) |
| doc | [011_account_status_by_name.md](011_account_status_by_name.md) | Related: account selection pattern via `name::` |
| doc | [cli/commands.md](../cli/commands.md) | CLI commands table (row 12) and command detail section |
| source | `src/commands.rs` | `account_limits_routine()` — delegates HTTP transport to `claude_quota::fetch_rate_limits` (feature-gated) |
| test | `tests/cli/account_limits_test.rs` | Error-path coverage: not-found, no credentials, data unavailable, invalid chars, existing-account data-unavailable |
| test | `tests/cli/account_limits_test.rs` — `lim_it1`, `lim_it2`, `lim_it3`, `lim_it5` | Automated live API tests: default, compact, JSON, verbose verbosity levels |
| doc | [cli/testing/command/12_account_limits.md](../cli/testing/command/12_account_limits.md) | Manual integration test specification |
