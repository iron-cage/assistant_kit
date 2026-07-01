# Feature: Account Inspect

### Scope

- **Purpose**: Unified account diagnostic — show identity, subscription, org, and quota utilization for one account by calling live endpoints 002 (account), 005 (roles), and 001 (usage). Primary diagnostic use case: verifying subscription selection when an account has multiple memberships (see BUG-237) and checking remaining quota.
- **Responsibility**: Documents the `.account.inspect` CLI command, its three-endpoint fetch sequence, multi-membership display, selection-priority indicator, quota utilization section, and graceful fallback to local snapshots when credentials are expired.
- **In Scope**: `.account.inspect` command parameters (`name::`, `refresh::`, `trace::`, `format::`); live fetch from endpoints 002 (account — identity + memberships), 005 (roles — org/workspace), 001 (usage — 5h/7d/Sonnet utilization); display of identity fields (tagged_id, uuid, email, name), ALL memberships with selection-priority indicator, rate-limit utilization with reset times; graceful per-endpoint fallback to local snapshot data when a fetch fails; `format::json` shape; membership selection priority rule (stripe_subscription + claude_max preferred).
- **Out of Scope**: Mutations to credentials or snapshots; endpoints 003, 004, 006–010; automatic token storage or re-save.

### Design

#### Motivation

`parse_oauth_account` in `claude_quota` selects the active membership from endpoint 002 by finding the first `"organization":` key in the JSON body — always reading `memberships[0]`. Accounts with multiple memberships (e.g., a free personal membership at index 0 and a paid Max membership at index 1) silently show the wrong subscription tier in `.usage`. `.account.inspect` surfaces the full membership list and the selection priority so operators can diagnose this class of problem. (See BUG-237.)

#### Endpoint fetch sequence

```
1. Read credentials from {credential_store}/{name}.credentials.json
2. If expiresAt locally expired AND refresh::1 → attempt token refresh via refresh_account_token()
3. GET /api/oauth/account            (endpoint 002) → tagged_id, uuid, email_address, full_name, display_name, memberships[]
4. GET /api/oauth/claude_cli/roles   (endpoint 005) → org UUID/name, workspace UUID/name, role
5. GET /api/oauth/usage              (endpoint 001) → 5h/7d/Sonnet utilization + reset times
```

Steps 3–5 are called independently. A failure on one endpoint is shown inline; the other endpoints still contribute their data. When all three fail (e.g., expired token with `refresh::0`), local snapshot data from `{name}.json` is shown with a `(snapshot)` suffix per field.

**Data source migration (BUG-295):** Identity fields (`tagged_id`, `uuid`, `email_address`) and memberships now come from endpoint 002 (`GET /api/oauth/account`) instead of the fabricated endpoints `/api/oauth/userinfo` and `/api/oauth/claude_cli/subscriptions` which returned HTTP 404. Quota utilization comes from endpoint 001 (`GET /api/oauth/usage`) — a free GET, not the POST `/v1/messages` probe used by `.account.limits`.

#### Membership selection priority

Applied to all memberships returned by endpoint 002, in order:

| Priority | Criteria |
|----------|----------|
| 1 (highest) | `billing_type == "stripe_subscription"` AND capabilities contain `"claude_max"` |
| 2 | `billing_type == "stripe_subscription"` (any capabilities) |
| 3 (fallback) | first membership (`memberships[0]`) |

The membership selected by this logic is marked `← selected` in the multi-membership display. For single-membership accounts no marker is shown. This selection priority matches the BUG-237 fix applied to `parse_oauth_account`.

#### Text output format

```
Account:         alice@acme.com
Name:            Alice (Alice)
Email:           alice@acme.com
Status:          🟢 valid (expires in 3h 52m)
Tagged ID:       user_01abc...def
UUID:            aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee

Memberships:     2
  [0]  billing_type=none              has_max=false  capabilities=[chat]
  [1]  billing_type=stripe_subscription  has_max=true   capabilities=[claude_max, chat]  ← selected

Billing:         stripe_subscription
Has Max:         yes
Capabilities:    [claude_max, chat]
Tier:            default_claude_max_20x

Session (5h):    45% consumed, resets in 12m
Weekly (7d):     33% consumed, resets in 1d 5h
Sonnet (7d):     53% consumed, resets in 1d 5h

Org:             alice@acme.com's Organization
Org UUID:        aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
Org Role:        admin
Workspace UUID:  (none)
Workspace:       (none)
```

- **`Name:` / `Email:`** From endpoint 002. Format: `full_name (display_name)` when both non-empty; omitted when absent.
- **`Status:`** Derived from `expiresAt` in the credential file. Format matches `.credentials.status` expiry display.
- **`Tagged ID:` / `UUID:`** From endpoint 002 (user-level fields). Previously from fabricated `/api/oauth/userinfo` (BUG-295).
- **`Memberships:`** Each line shows `[index]`, `billing_type`, `has_max`, and `capabilities` from the endpoint 002 membership objects. The highest-priority membership is marked `← selected`. When only one membership exists, no `← selected` marker is shown.
- **`Billing:` / `Has Max:`** Taken from the selected membership.
- **`Capabilities:` / `Tier:`** From the selected membership's `capabilities[]` and `rate_limit_tier`.
- **`Session (5h):` / `Weekly (7d):` / `Sonnet (7d):`** From endpoint 001 (`GET /api/oauth/usage`). Shows utilization percentage and reset countdown. Omitted when endpoint 001 is unavailable.
- **`Org:` / `Org UUID:` / `Org Role:` / `Workspace UUID:` / `Workspace:`** From endpoint 005. `(none)` for null/absent workspace fields.
- **Snapshot fallback:** When an endpoint is unavailable, each field sourced from that endpoint shows the local snapshot value with `(snapshot)` appended. Fields with no local fallback show `N/A`. Quota fields (from endpoint 001) show `N/A` with no snapshot fallback.

#### Single-membership account output

```
Account:         bob@example.com
Status:          🟢 valid (expires in 7h 12m)
Tagged ID:       user_02xyz
UUID:            bbbbbbbb-cccc-dddd-eeee-ffffffffffff

Memberships:     1
  [0]  billing_type=stripe_subscription  has_max=true   capabilities=[claude_max, chat]

Billing:         stripe_subscription
Has Max:         yes
Org:             bob@example.com's Organization
Org UUID:        ...
Org Role:        admin
Workspace UUID:  (none)
Workspace:       (none)
```

#### Token-expired output (refresh::0 or refresh fails)

```
Account:         carol@example.com
Status:          🔴 expired (3h 12m ago)
Tagged ID:       user_03qrs (snapshot)
UUID:            cccccccc-dddd... (snapshot)

Memberships:     endpoint unavailable (auth error)

Billing:         stripe_subscription (snapshot)
Has Max:         yes (snapshot)
Org:             carol@example.com's Organization (snapshot)
Org UUID:        ... (snapshot)
Org Role:        admin (snapshot)
Workspace UUID:  (none)
Workspace:       (none)
```

#### `format::json`

JSON output always includes all fields regardless of availability:

```json
{
  "account": "alice@acme.com",
  "status": "valid",
  "expires_in_secs": 14000,
  "tagged_id": "user_01abc...def",
  "uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "email_address": "alice@acme.com",
  "full_name": "Alice",
  "display_name": "Alice",
  "memberships": [
    {
      "index": 0,
      "billing_type": "none",
      "has_max": false,
      "capabilities": ["chat"],
      "selected": false
    },
    {
      "index": 1,
      "billing_type": "stripe_subscription",
      "has_max": true,
      "capabilities": ["claude_max", "chat"],
      "selected": true
    }
  ],
  "billing_type": "stripe_subscription",
  "has_max": true,
  "capabilities": ["claude_max", "chat"],
  "rate_limit_tier": "default_claude_max_20x",
  "session_5h_pct": 45,
  "session_5h_reset_ts": 1750089000,
  "weekly_7d_pct": 33,
  "weekly_7d_reset_ts": 1750180000,
  "sonnet_7d_pct": 53,
  "sonnet_7d_reset_ts": 1750180000,
  "organization_name": "alice@acme.com's Organization",
  "organization_uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "organization_role": "admin",
  "workspace_uuid": "",
  "workspace_name": "",
  "data_source": "live"
}
```

`data_source`: `"live"` when all endpoints succeeded; `"partial_snapshot"` when some endpoints fell back; `"snapshot"` when all endpoints fell back to local data.

#### `name::` resolution

Resolved via [`AccountSelector`](../cli/type/004_account_selector.md): full email, prefix, or positional bare argument. Omit `name::` to inspect the currently active account.

### Acceptance Criteria

- **AC-01**: `clp .account.inspect` with the active account fetches endpoint 002 (`GET /api/oauth/account`) and shows `Account:`, `Status:`, `Tagged ID:`, `UUID:`.
- **AC-02**: All memberships from endpoint 002 are shown with `[index]`, `billing_type`, `has_max`, `capabilities`.
- **AC-03**: For accounts with multiple memberships, the membership matching the highest selection priority is marked `← selected`; others are unmarked.
- **AC-04**: For a single-membership account, no `← selected` marker is shown.
- **AC-05**: Endpoint 005 is called; `Org:`, `Org UUID:`, `Org Role:`, `Workspace UUID:`, `Workspace:` are shown.
- **AC-06**: `Billing:` and `Has Max:` are taken from the selected membership (AC-03 priority rule), not necessarily `memberships[0]`.
- **AC-07**: When endpoint 002 fails (auth error, network error), `Memberships:` shows `endpoint unavailable ({reason})`; `Billing:` and `Has Max:` fall back to the `{name}.json` snapshot with `(snapshot)` suffix.
- **AC-08**: When endpoint 002 fails, `Tagged ID:` and `UUID:` fall back to `{name}.json` snapshot values with `(snapshot)` suffix.
- **AC-09**: When endpoint 005 fails, org fields fall back to `{name}.json` snapshot values with `(snapshot)` suffix.
- **AC-10**: `refresh::1` (default): when `expiresAt` is locally expired, attempts token refresh via `refresh_account_token()` before endpoint calls.
- **AC-11**: `refresh::0`: locally-expired token is NOT refreshed; all three endpoint calls receive the stale token and fail with auth errors; graceful fallback to snapshot for all fields.
- **AC-12**: `name::` resolves via `AccountSelector` (email, prefix, positional); invalid name exits 2 with `account not found: {name}`.
- **AC-13**: `format::json` always includes all fields: `account`, `status`, `expires_in_secs`, `tagged_id`, `uuid`, `email_address`, `full_name`, `display_name`, `memberships[]` (with `index`, `billing_type`, `has_max`, `capabilities`, `selected`), `billing_type`, `has_max`, `capabilities`, `rate_limit_tier`, `session_5h_pct`, `session_5h_reset_ts`, `weekly_7d_pct`, `weekly_7d_reset_ts`, `sonnet_7d_pct`, `sonnet_7d_reset_ts`, `organization_name`, `organization_uuid`, `organization_role`, `workspace_uuid`, `workspace_name`, `data_source`.
- **AC-14**: `trace::1` emits timestamped diagnostic lines to stderr for each endpoint call: URL, HTTP status, and field count extracted.
- **AC-15**: `clp .account.inspect name::user@host` with no credential store directory exits 2 with `credential file not found: {path}`. Absent store is treated identically to absent credential file — the email resolver short-circuits store lookup and the file existence check produces the error.
- **AC-16**: `name::` resolves successfully but the account's credentials file (`{name}.credentials.json`) is absent: exits 2 with `credential file not found: {path}`.
- **AC-17**: Enterprise accounts with non-null `workspace_uuid` and `workspace_name` from endpoint 005: `Workspace UUID:` shows the UUID string; `Workspace:` shows the workspace name. `(none)` is used only when the field is null or absent. In `format::json`, `workspace_uuid` and `workspace_name` contain the raw value (empty string when null/absent).
- **AC-18**: Credentials file exists but is zero bytes (empty file): command runs, `Status:` shows `unknown`, exits 0. Distinct from AC-16 (absent file → exits 2) — an existing-but-empty file is not a missing file, so the resolver finds it; the JSON parse failure produces an unknown status rather than an error exit.
- **AC-19**: Credentials file contains valid JSON but lacks the `oauthAccount` wrapper object (e.g. `{"version":"2","data":{}}`): command runs, `Status:` shows `unknown`, exits 0. The `expiresAt` field cannot be located; graceful degradation applies rather than a hard error.
- **AC-20**: `Name:` and `Email:` fields are shown from endpoint 002's `full_name`, `display_name`, and `email_address` user-level fields. Format: `full_name (display_name)` when both differ; `full_name` alone when identical.
- **AC-21**: `Capabilities:` shows the selected membership's `capabilities[]` array; `Tier:` shows the selected membership's `rate_limit_tier` string.
- **AC-22**: Endpoint 001 (`GET /api/oauth/usage`) is called; `Session (5h):`, `Weekly (7d):`, and `Sonnet (7d):` show utilization percentages and reset countdowns.
- **AC-23**: When endpoint 001 fails, the quota section is omitted (no snapshot fallback for quota data).
- **AC-24**: `format::json` includes new fields: `email_address`, `full_name`, `display_name`, `capabilities`, `rate_limit_tier`, `session_5h_pct`, `session_5h_reset_ts`, `weekly_7d_pct`, `weekly_7d_reset_ts`, `sonnet_7d_pct`, `sonnet_7d_reset_ts`.
- **AC-25**: Identity fields (`tagged_id`, `uuid`, `email_address`) come from endpoint 002 (`GET /api/oauth/account`), NOT from `/api/oauth/userinfo` (removed — BUG-295).

### Bugs

| File | Relationship |
|------|--------------|
| BUG-237 | BUG-237 — motivation for multi-membership display |

### Features

| File | Relationship |
|------|--------------|
| [013_account_limits.md](013_account_limits.md) | Merged: quota utilization data from endpoint 001 now included in inspect output |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md#command--15-accountinspect) | CLI command specification |

### Contracts

| File | Relationship |
|------|--------------|
| `contract/claude_code/docs/endpoint/002_oauth_account.md` | Wire contract for `GET /api/oauth/account` — identity + memberships |
| `contract/claude_code/docs/endpoint/005_claude_cli_roles.md` | Wire contract for `GET /api/oauth/claude_cli/roles` — org identity |
| `contract/claude_code/docs/endpoint/001_oauth_usage.md` | Wire contract for `GET /api/oauth/usage` — quota utilization |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — AccountSelector |
| [cli/param/002_format.md](../cli/param/002_format.md) | `format::` |
| [cli/param/019_refresh.md](../cli/param/019_refresh.md) | `refresh::` — token refresh on expired credentials |
| [cli/param/023_trace.md](../cli/param/023_trace.md) | `trace::` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.inspect`](../cli/command/001_account.md#command--15-accountinspect) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `account::list()` — reads local snapshot data for fallback |
| `src/commands/account_inspect.rs` | `account_inspect_routine()` — three-endpoint fetch (002, 005, 001), selection priority, text/json render |
| `src/registry.rs` | Registration of `.account.inspect` and its four parameters |
| `claude_quota/src/lib.rs` | `fetch_oauth_account()`, `fetch_claude_cli_roles()`, `fetch_oauth_usage()` — endpoint transports; `select_membership_index()` — priority logic |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_inspect_test.rs` | Integration tests for AC-01..AC-25 |
| [tests/docs/feature/031_account_inspect.md](../../tests/docs/feature/031_account_inspect.md) | FT-level behavioral test cases |
