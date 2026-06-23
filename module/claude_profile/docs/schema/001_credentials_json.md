# Schema: Credential Snapshot — `{name}.credentials.json`

### Scope

- **Purpose**: Define the on-disk format of the per-account OAuth credential snapshot stored in the credential store.
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
| `accessToken` | string | OAuth access token (JWT or opaque `sk-ant-oat01-*` format). Used for all API calls. Expires per `expiresAt`. |
| `refreshToken` | string | OAuth refresh token. Used by `run_isolated` during token refresh to obtain a new `accessToken`/`refreshToken` pair. Rotated on each refresh. |
| `expiresAt` | number (u64 ms) | UTC epoch milliseconds when `accessToken` expires. Set by the OAuth server at token issuance. NOT updated by `run_isolated` — use JWT `exp` claim instead (see [feature/017](../feature/017_token_refresh.md) BUG-162). |

### Example

```json
{
  "accessToken": "eyJhbG...",
  "refreshToken": "eyJhbG...",
  "expiresAt": 1750000000000
}
```

### Write Callers

| Caller | When |
|--------|------|
| `account::save()` in `claude_profile_core/src/account.rs` | `.account.save`, credential writeback after token refresh (BUG-221 fix: writes to credential store only, never to `~/.claude/.credentials.json`) |

### Read Callers

| Caller | When |
|--------|------|
| `account::list()` | `.accounts`, `.usage` — reads all `{name}.credentials.json` to build `AccountQuota` list |
| `account::refresh_account_token()` | Per-account token refresh — reads credential to pass to `run_isolated` with forced `expiresAt: "1"` (AC-32) |

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/002_account_save.md](../feature/002_account_save.md) | Save algorithm; step 1 writes this file |
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | Refresh lifecycle; BUG-162 (expiresAt not updated by subprocess) |
| [schema/002](002_account_json.md) | Companion supplementary metadata file `{name}.json` |
| [invariant/007](../invariant/007_json_storage_format.md) | 2-space pretty-print + trailing newline requirement |
