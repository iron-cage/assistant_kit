# Feature: Account Inspect

### Scope

- **Purpose**: Show all identity, subscription, and org fields for one account by calling live endpoints 001, 002, and 005. Primary diagnostic use case: verifying subscription selection when an account has multiple memberships (see BUG-237).
- **Responsibility**: Documents the `.account.inspect` CLI command, its three-endpoint fetch sequence, multi-membership display, selection-priority indicator, and graceful fallback to local snapshots when credentials are expired.
- **In Scope**: `.account.inspect` command parameters (`name::`, `refresh::`, `trace::`, `format::`); live fetch from endpoints 001 (userinfo), 002 (subscriptions/memberships), 005 (roles); display of ALL memberships from endpoint 002 with selection-priority indicator; graceful per-endpoint fallback to local snapshot data when a fetch fails; `format::json` shape; membership selection priority rule (stripe_subscription + claude_max preferred).
- **Out of Scope**: Quota data (token utilization, 5h/7d counters — see `.usage` / feature 009); mutations to credentials or snapshots; endpoints 003, 004, 006–010; automatic token storage or re-save.

### Design

#### Motivation

`parse_oauth_account` in `claude_quota` selects the active membership from endpoint 002 by finding the first `"organization":` key in the JSON body — always reading `memberships[0]`. Accounts with multiple memberships (e.g., a free personal membership at index 0 and a paid Max membership at index 1) silently show the wrong subscription tier in `.usage`. `.account.inspect` surfaces the full membership list and the selection priority so operators can diagnose this class of problem. (See BUG-237.)

#### Endpoint fetch sequence

```
1. Read credentials from {credential_store}/{name}.credentials.json
2. If expiresAt locally expired AND refresh::1 → attempt token refresh via refresh_account_token()
3. GET /api/oauth/userinfo           (endpoint 001) → taggedId, emailAddress, uuid
4. GET /api/oauth/claude_cli/subscriptions  (endpoint 002) → all memberships
5. GET /api/oauth/claude_cli/roles   (endpoint 005) → org identity
```

Steps 3–5 are called independently. A failure on one endpoint is shown inline; the other endpoints still contribute their data. When all three fail (e.g., expired token with `refresh::0`), local snapshot data from `{name}.claude.json` and `{name}.roles.json` is shown with a `(snapshot)` suffix per field.

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
Status:          🟢 valid (expires in 3h 52m)
Tagged ID:       user_01abc...def
UUID:            aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee

Memberships:     2
  [0]  billing_type=none              has_max=false  capabilities=[chat]
  [1]  billing_type=stripe_subscription  has_max=true   capabilities=[claude_max, chat]  ← selected

Billing:         stripe_subscription
Has Max:         yes
Org:             alice@acme.com's Organization
Org UUID:        aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee
Org Role:        admin
Workspace UUID:  (none)
Workspace:       (none)
```

- **`Status:`** Derived from `expiresAt` in the credential file. Format matches `.credentials.status` expiry display.
- **`Memberships:`** Each line shows `[index]`, `billing_type`, `has_max`, and `capabilities` from the endpoint 002 membership object. The highest-priority membership is marked `← selected`. When only one membership exists, no `← selected` marker is shown.
- **`Billing:` / `Has Max:`** Taken from the selected membership.
- **`Org:` / `Org UUID:` / `Org Role:` / `Workspace UUID:` / `Workspace:`** From endpoint 005. `(none)` for null/absent workspace fields.
- **Snapshot fallback:** When an endpoint is unavailable, each field sourced from that endpoint shows the local snapshot value with `(snapshot)` appended. Fields with no local fallback show `N/A`.

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

- **AC-01**: `clp .account.inspect` with the active account fetches endpoint 001 and shows `Account:`, `Status:`, `Tagged ID:`, `UUID:`.
- **AC-02**: Endpoint 002 is called; all memberships are shown with `[index]`, `billing_type`, `has_max`, `capabilities`.
- **AC-03**: For accounts with multiple memberships, the membership matching the highest selection priority is marked `← selected`; others are unmarked.
- **AC-04**: For a single-membership account, no `← selected` marker is shown.
- **AC-05**: Endpoint 005 is called; `Org:`, `Org UUID:`, `Org Role:`, `Workspace UUID:`, `Workspace:` are shown.
- **AC-06**: `Billing:` and `Has Max:` are taken from the selected membership (AC-03 priority rule), not necessarily `memberships[0]`.
- **AC-07**: When endpoint 002 fails (auth error, network error), `Memberships:` shows `endpoint unavailable ({reason})`; `Billing:` and `Has Max:` fall back to the `{name}.claude.json` snapshot with `(snapshot)` suffix.
- **AC-08**: When endpoint 001 fails, `Tagged ID:` and `UUID:` fall back to `{name}.claude.json` snapshot values with `(snapshot)` suffix.
- **AC-09**: When endpoint 005 fails, org fields fall back to `{name}.roles.json` snapshot values with `(snapshot)` suffix.
- **AC-10**: `refresh::1` (default): when `expiresAt` is locally expired, attempts token refresh via `refresh_account_token()` before endpoint calls.
- **AC-11**: `refresh::0`: locally-expired token is NOT refreshed; all three endpoint calls receive the stale token and fail with auth errors; graceful fallback to snapshot for all fields.
- **AC-12**: `name::` resolves via `AccountSelector` (email, prefix, positional); invalid name exits 2 with `account not found: {name}`.
- **AC-13**: `format::json` always includes all fields: `account`, `status`, `expires_in_secs`, `tagged_id`, `uuid`, `memberships[]` (with `index`, `billing_type`, `has_max`, `capabilities`, `selected`), `billing_type`, `has_max`, `organization_name`, `organization_uuid`, `organization_role`, `workspace_uuid`, `workspace_name`, `data_source`.
- **AC-14**: `trace::1` emits `[trace]` lines to stderr for each endpoint call: URL, HTTP status, and field count extracted.
- **AC-15**: `clp .account.inspect name::user@host` with no credential store directory exits 2 with `credential file not found: {path}`. Absent store is treated identically to absent credential file — the email resolver short-circuits store lookup and the file existence check produces the error.
- **AC-16**: `name::` resolves successfully but the account's credentials file (`{name}.credentials.json`) is absent: exits 2 with `credential file not found: {path}`.
- **AC-17**: Enterprise accounts with non-null `workspace_uuid` and `workspace_name` from endpoint 005: `Workspace UUID:` shows the UUID string; `Workspace:` shows the workspace name. `(none)` is used only when the field is null or absent. In `format::json`, `workspace_uuid` and `workspace_name` contain the raw value (empty string when null/absent).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `account::list()` — reads local snapshot data for fallback |
| source | `src/commands/account_ops.rs` | `account_inspect_routine()` — three-endpoint fetch, selection priority, text/json render |
| source | `src/registry.rs` | Registration of `.account.inspect` and its four parameters |
| source | `claude_quota/src/lib.rs` | `fetch_userinfo()`, `fetch_subscriptions()`, `fetch_claude_cli_roles()` — endpoint transports; `select_membership_index()` — priority logic |
| test | `tests/cli/account_inspect_test.rs` | Integration tests for AC-01..AC-17 |
| doc | [031_account_inspect.md](../tests/docs/feature/031_account_inspect.md) | FT-level behavioral test cases |
| doc | [cli/command/001_account.md](../cli/command/001_account.md#command--15-accountinspect) | CLI command specification |
| doc | [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — AccountSelector |
| doc | [cli/param/019_refresh.md](../cli/param/019_refresh.md) | `refresh::` — token refresh on expired credentials |
| doc | [cli/param/023_trace.md](../cli/param/023_trace.md) | `trace::` |
| doc | [cli/param/002_format.md](../cli/param/002_format.md) | `format::` |
| bug | `task/claude_profile/bug/237_parse_oauth_account_multi_membership_reads_first_only.md` | BUG-237 — motivation for multi-membership display |
| contract | `contract/claude_code/docs/endpoint/011_oauth_userinfo.md` (to create) | Wire contract for `GET /api/oauth/userinfo` — identity fields |
| contract | `contract/claude_code/docs/endpoint/012_claude_cli_subscriptions.md` (to create) | Wire contract for `GET /api/oauth/claude_cli/subscriptions` — all memberships |
| contract | `contract/claude_code/docs/endpoint/005_claude_cli_roles.md` | Wire contract for `GET /api/oauth/claude_cli/roles` — org identity |
