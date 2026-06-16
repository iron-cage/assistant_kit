# Feature: Account Rate-Limit Utilization

### Scope

- **Purpose**: Show plan and rate-limit utilization for a selected account so the user can see how much of their allocation remains before hitting limits.
- **Responsibility**: Documents the `.account.limits` command and its rate-limit utilization output (FR-18).
- **In Scope**: Command output structure, account selection, confirmed data source (HTTP response headers).
- **Out of Scope**: Implementation details (→ implementation task), historical token counts (→ 009_token_usage.md).

### Design

`.account.limits` must display plan/rate-limit utilization for the selected account.

**Output:**
- Session usage (5-hour window) — percentage consumed and reset time
- Weekly all-model usage (7-day window) — percentage consumed and reset time
- Rate-limit status — `allowed`, `allowed_warning`, or `rejected`

**Account selection:** Uses `name::` parameter (optional) to select which account to inspect. When omitted, shows limits for the active account.

**Parameters:** `name::` (optional), `format::` — consistent with other `.account.*` commands.

**Data source (confirmed):** Rate-limit utilization comes exclusively from HTTP response headers returned by the Anthropic API. Claude Code fetches these by making a lightweight `POST /v1/messages` (`max_tokens: 1`, content `"quota"`) and reading the response headers. Headers:

| Header | Value | Meaning |
|--------|-------|---------|
| `anthropic-ratelimit-unified-5h-utilization` | `0.0–1.0` | 5-hour session window consumed |
| `anthropic-ratelimit-unified-5h-reset` | Unix timestamp | Reset time for 5h window |
| `anthropic-ratelimit-unified-7d-utilization` | `0.0–1.0` | Weekly all-model consumed |
| `anthropic-ratelimit-unified-7d-reset` | Unix timestamp | Reset time for 7d window |
| `anthropic-ratelimit-unified-status` | `allowed` / `allowed_warning` / `rejected` | Current rate-limit state |

These headers are never cached locally — no local file contains them. `stats-cache.json` has raw token counts only.

**Current implementation state:** Error paths (lim01–lim05) and happy path (AC-01 through AC-03) fully implemented. HTTP transport is delegated to `claude_quota::fetch_rate_limits(token)` (gated under the `enabled` feature via `dep:claude_quota`). The command reads credentials from disk, then calls `claude_quota` which makes a lightweight `POST /v1/messages` (`max_tokens: 1`, `content: "quota"`) and reads `anthropic-ratelimit-unified-*` response headers. IT-1 (default text) and IT-3 (`format::json`) are automated live API tests in `tests/cli/account_limits_test.rs`. IT-4 (named account `name::alice@acme.com`) is manual-only — requires a saved account and is tracked in `tests/manual/readme.md`.

**Exit codes:**
- 0: success
- 1: invalid `name::` characters (usage error)
- 2: runtime error (account not found, data unavailable, HOME unset)

### Acceptance Criteria

- **AC-01**: `.account.limits` shows current session (5h) utilization, weekly all-model (7d) utilization, and rate-limit status for the active account.
- **AC-02**: `.account.limits name::alice@acme.com` shows limits for the named account.
- **AC-03**: `format::json` returns structured JSON with utilization fields.
- **AC-04**: Missing data source → exits 2 with an actionable error (not a silent zero).

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command--11-accountlimits) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [009_token_usage.md](009_token_usage.md) | Related: all-accounts live quota table (distinct from single-account limits) |
| [011_account_status_by_name.md](011_account_status_by_name.md) | Related: account selection pattern via `name::` |
| [031_account_inspect.md](031_account_inspect.md) | Merged: `.account.inspect` now includes 5h/7d/Sonnet utilization via endpoint 001 (`GET /api/oauth/usage`) — superset of `.account.limits` data |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.limits`](../cli/command/001_account.md#command--11-accountlimits) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/limits.rs` | `account_limits_routine()` — delegates HTTP transport to `claude_quota::fetch_rate_limits` (feature-gated) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_limits_test.rs` | Error-path coverage: not-found, no credentials, data unavailable, invalid chars, existing-account data-unavailable |
| `tests/cli/account_limits_test.rs` — `lim_it1`, `lim_it3` | Automated live API tests: default text, JSON format |
| [tests/docs/cli/command/11_account_limits.md](../../tests/docs/cli/command/11_account_limits.md) | Integration test case planning |
