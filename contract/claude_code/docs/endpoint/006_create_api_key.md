# Endpoint: POST /api/oauth/claude_cli/create_api_key

### Scope

- **Purpose**: Create a new Anthropic API key scoped to the authenticated user's organization — used by Claude CLI to obtain a persistent API key from an OAuth session.
- **Responsibility**: Wire contract for `POST /api/oauth/claude_cli/create_api_key`: URL, required scope, request body, error behavior.
- **In Scope**: Scope requirement (`org:create_api_key`), 403 error semantics, standard OAuth token limitation.
- **Out of Scope**: Anthropic Console API key management UI; API key usage for inference (→ Anthropic docs).

### Request

```
POST https://api.anthropic.com/api/oauth/claude_cli/create_api_key
Authorization:     Bearer {access_token}
anthropic-version: 2023-06-01
Content-Type:      application/json
```

Request body schema is not fully known — the endpoint requires `org:create_api_key` scope which is absent from standard OAuth tokens, so a successful request has not been observed.

### Scope Requirement

This endpoint requires the `org:create_api_key` OAuth scope. The standard Claude Max OAuth token contains:

```
user:file_upload user:inference user:mcp_servers user:profile user:sessions:claude_code
```

`org:create_api_key` is **not** in the standard scope set. Standard OAuth tokens always receive HTTP 403 from this endpoint.

**Obtaining the required scope**: The `org:create_api_key` scope must be requested during the initial OAuth login flow in the Claude desktop app (or equivalent). It is not grantable after the fact via token refresh.

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200 | Success (response body schema: TBD — not observed) |
| 403 | `"OAuth token does not meet scope requirement org:create_api_key"` |
| 401 | Token invalid or expired |

### Observed Error Response

HTTP 403 with standard Max OAuth token:

```json
{"error": "OAuth token does not meet scope requirement org:create_api_key"}
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [005_claude_cli_roles.md](005_claude_cli_roles.md) | Org/workspace role lookup (prerequisite context) |
| doc | [004_oauth_token.md](004_oauth_token.md) | Token refresh — scope is fixed at login, not in refresh |
