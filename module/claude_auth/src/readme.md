# src/

Anthropic OAuth token refresh transport — Layer `*` standalone primitive, zero workspace dependencies.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | `TokenRefreshResult`, `AuthError`, `parse_response()`, `refresh_token()` (feature `enabled`) |

### Scope

**In Scope:**
- OAuth token-refresh wire protocol: `TOKEN_URL`, `CLIENT_ID`, request body construction, response parsing
- Dependency-free JSON field extraction (`parse_response` needs no `serde`)
- Blocking HTTP transport for the refresh call (feature `enabled`, via `ureq`)

**Out of Scope:**
- Quota/usage data (→ `claude_quota`)
- Profile/account management, credential storage (→ `claude_profile`, `dream`)
- Redaction of tokens in logs (→ `json_redact`)

See [`docs/api/001_token_refresh_api.md`](../docs/api/001_token_refresh_api.md) for the full behavioral contract.
