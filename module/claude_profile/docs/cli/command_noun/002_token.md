# Token

The OAuth access token embedded in `~/.claude/.credentials.json` for the active Claude Code session. Represents the credential used to authenticate API requests; classified as `Valid`, `ExpiringSoon`, or `Expired` relative to a configurable warning threshold.

### Commands

| # | Command | Verb | Purpose | Idempotent | Requires Session |
|---|---------|------|---------|-----------|-----------------|
| 1 | `.token.status` | status | Classify the active token as Valid, ExpiringSoon, or Expired | Yes | Yes |

### Parameter Matrix

| Parameter | `.token.status` |
|-----------|----------------|
| `format::` | optional |
| `threshold::` | optional |
| `trace::` | optional |

### Lifecycle

Stateless — token operations are pure reads with no persistent state.

The token lifecycle is managed externally by the Claude Code OAuth flow. `clp` reads but never writes the token:
- Token is created by browser OAuth login (via `claude` binary or `.account.relogin`)
- Token transitions from Valid → ExpiringSoon → Expired as `expiresAt` approaches
- Token refresh is handled by `.account.use refresh::1` and `.usage refresh::1` (isolated subprocess)

### Provider Contract

| Operation | Implementation |
|-----------|---------------|
| `.token.status` | `token::read_expiry()` — reads `expiresAt` from `~/.claude/.credentials.json`; classifies against threshold |

### Output Schema

**`.token.status format::json`:**

```json
{
  "status": "valid",
  "expires_in_secs": 2820
}
```

Status values: `"valid"`, `"expiring_soon"`, `"expired"`.

### Error Codes

| Code | Trigger | Recovery |
|------|---------|---------|
| exit 2 | `~/.claude/.credentials.json` absent or unreadable | Authenticate via `claude` binary; check `$HOME` env |
| exit 2 | `expiresAt` field absent or unparseable | Re-authenticate; credentials file may be corrupted |

### Relationships

| Related Entity | Relationship | Direction |
|----------------|-------------|---------|
| `account` | Token is embedded in the active account's credential file; account lifecycle governs token state | token → account |

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/006_token_status.md](../../feature/006_token_status.md) | Token expiry classification algorithm and threshold semantics |
| [feature/008_auto_rotate.md](../../feature/008_auto_rotate.md) | Token status drives auto-rotation trigger |
| [feature/017_token_refresh.md](../../feature/017_token_refresh.md) | Expired token refresh via isolated subprocess |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Classify active OAuth token expiry state |
