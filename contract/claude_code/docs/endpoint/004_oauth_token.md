# Endpoint: POST /v1/oauth/token — Token Refresh

### Scope

- **Purpose**: Exchange a refresh token for a new access token — the only mechanism to obtain a fresh Bearer token without re-authenticating interactively.
- **Responsibility**: Complete wire contract for `POST /v1/oauth/token`: URL, required body fields, response schema, error behavior, and rotation semantics.
- **In Scope**: Request body fields, response fields, HTTP 429 handling, refresh token rotation, `CLIENT_ID` constant.
- **Out of Scope**: Interactive OAuth login flow (→ Claude desktop app); credential file storage (→ `claude_profile` credential management); scope semantics (→ OAuth provider docs).

### Request

```
POST https://platform.claude.com/v1/oauth/token
Content-Type: application/json

{
  "grant_type":    "refresh_token",
  "refresh_token": "<current_refresh_token>",
  "client_id":     "9d1c250a-e61b-44d9-88ed-5944d1962f5e",
  "scope":         "user:file_upload user:inference user:mcp_servers user:profile user:sessions:claude_code"
}
```

No `Authorization` header — this endpoint IS the authentication step. The `client_id` is the public OAuth client ID for the Claude desktop application.

Workspace constants: `claude_auth::TOKEN_URL`, `claude_auth::CLIENT_ID`.

### Scope String

The `scope` field must contain all scopes originally granted at login. The standard Claude Max OAuth scope string (as observed from `*.credentials.json`):

```
user:file_upload user:inference user:mcp_servers user:profile user:sessions:claude_code
```

Sending a subset of the original scopes may return a token with reduced permissions. Sending scopes beyond what was granted returns an error.

### Response

HTTP 200 on success. Body is a JSON object.

| Field | Type | Semantics |
|-------|------|-----------|
| `access_token` | string | New Bearer token for API calls |
| `refresh_token` | string | New refresh token — **replaces** the one sent in the request |
| `expires_in` | u64 | Access token lifetime in seconds (typically 3600) |

**Refresh token rotation**: Each successful refresh invalidates the previous refresh token and issues a new one. The new refresh token must be persisted before the old one is discarded. Failure to persist the new refresh token results in permanent session loss.

**`expires_at_ms` computation**: `now_ms + expires_in * 1000` where `now_ms` is the current Unix time in milliseconds. Stored in credential files to detect expiry without a live API call.

### Example Response

HTTP 200:

```json
{
  "access_token":  "sk-ant-ocp05-...",
  "refresh_token": "ant-oauth-rt-...",
  "expires_in":    3600
}
```

### Error Codes

| HTTP | `AuthError` variant | Meaning |
|------|---------------------|---------|
| 200 | — | Success |
| 429 | `RateLimited` | Too many refresh attempts — back off before retrying |
| 4xx | `HttpTransport(status_text)` | Invalid token, bad client ID, or scope mismatch |
| Transport | `HttpTransport(msg)` | Network or TLS failure |

HTTP 429 is the only status code that maps to a distinct `AuthError` variant. All other non-200 status codes produce `HttpTransport`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../../../module/claude_auth/src/lib.rs` | `refresh_token`, `TOKEN_URL`, `CLIENT_ID`, `AuthError` |
| doc | `../../../../module/claude_profile/docs/feature/017_token_refresh.md` | Refresh trigger policy and credential write-back |
| doc | [001_oauth_usage.md](001_oauth_usage.md) | Endpoint that consumes access tokens |
| doc | [002_oauth_account.md](002_oauth_account.md) | Endpoint that consumes access tokens |
