# Schema: Credential Snapshot — `{name}.credentials.json`

### Scope

- **Purpose**: Define the on-disk format of the per-account OAuth credential snapshot stored in the credential store.
- **Responsibility**: Documents the on-disk format of per-account OAuth credential snapshots in the credential store.
- **In Scope**: Field names, types, semantics, and write/read callers for `{name}.credentials.json`.
- **Out of Scope**: The live session credential file `~/.claude/.credentials.json` (same format, different location and ownership); token refresh mechanics (→ [feature/017](../feature/017_token_refresh.md)); credential store path (→ [schema/004](004_storage_root.md)).

### File Location

```
{credential_store}/{name}.credentials.json
```

Where `{credential_store}` = `{root}/.persistent/claude/credential/` and `{root}` = `$PRO` (if set and exists) or `$HOME`. See [schema/004](004_storage_root.md).

### Format

2-space pretty-printed JSON, trailing newline. See [invariant/007](../invariant/007_json_storage_format.md).

### Fields

| Field | Type | Semantics |
|-------|------|-----------|
| `accessToken` | string | OAuth access token (JWT or opaque `sk-ant-oat01-*` format) for `backend: "anthropic"` accounts; static API key for `backend: "redirect"` accounts (see [feature/071](../feature/071_redirect_backend_accounts.md)). Used for all API calls. Expires per `expiresAt` when present. |
| `refreshToken` | string | **Anthropic backend only.** OAuth refresh token. Used by `run_isolated` during token refresh to obtain a new `accessToken`/`refreshToken` pair. Rotated on each refresh. Absent entirely for `backend: "redirect"` accounts — a static API key has nothing to refresh. |
| `expiresAt` | number (u64 ms) | **Anthropic backend only.** UTC epoch milliseconds when `accessToken` expires. Set by the OAuth server at token issuance. NOT updated by `run_isolated` — use JWT `exp` claim instead (see [feature/017](../feature/017_token_refresh.md) BUG-162). Absent entirely for `backend: "redirect"` accounts — this absence is itself the non-expiring signal consumed by `TokenStatus::Static` (see [state_machine/002](../state_machine/002_oauth_token_lifecycle.md)). |

### Example

```json
{
  "accessToken": "eyJhbG...",
  "refreshToken": "eyJhbG...",
  "expiresAt": 1750000000000
}
```

### Redirect Backend Example

A `backend: "redirect"` account (see [feature/071](../feature/071_redirect_backend_accounts.md)) stores only `accessToken`, holding the static API key supplied via `api_key::` at `.account.save` time — `refreshToken` and `expiresAt` are both omitted, never written as `null` or empty string:

```json
{
  "accessToken": "sk-moonshot-abc123..."
}
```

### Write Callers

| Caller | When |
|--------|------|
| `account::save()` in `claude_profile_core/src/account/store.rs` | Single entry point for both backends — takes a `backend: AccountBackend` param and branches internally (account.rs:325 pre-split). `backend == AccountBackend::Anthropic`: `.account.save`, credential writeback after token refresh (BUG-221 fix: writes to credential store only, never to `~/.claude/.credentials.json`). `backend == AccountBackend::Redirect`: `.account.save backend::redirect api_key::KEY` — writes `accessToken` only, from the caller-supplied `api_key::` bytes; never touches `~/.claude/.credentials.json` (see [feature/071](../feature/071_redirect_backend_accounts.md)) |

### Read Callers

| Caller | When |
|--------|------|
| `account::list()` | `.accounts`, `.usage` — reads all `{name}.credentials.json` to build `AccountQuota` list |
| `account::refresh_account_token()` | Per-account token refresh — reads credential to pass to `run_isolated` with forced `expiresAt: "1"` (AC-32) |

### Features

| File | Relationship |
|------|-------------|
| [feature/002_account_save.md](../feature/002_account_save.md) | Save algorithm; step 1 writes this file |
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | Refresh lifecycle; BUG-162 (expiresAt not updated by subprocess) |
| [feature/071_redirect_backend_accounts.md](../feature/071_redirect_backend_accounts.md) | Redirect-backend accounts — `accessToken`-only shape, no `refreshToken`/`expiresAt` |

### Schema

| File | Relationship |
|------|-------------|
| [002_account_json.md](002_account_json.md) | Companion supplementary metadata file `{name}.json` |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/007](../invariant/007_json_storage_format.md) | 2-space pretty-print + trailing newline requirement |
