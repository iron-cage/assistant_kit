# Endpoint: GET /api/claude_code/organizations/metrics_enabled

### Scope

- **Purpose**: Check whether usage metrics collection is enabled for the authenticated user's organization — a feature flag for enterprise/team accounts.
- **Responsibility**: Wire contract for `GET /api/claude_code/organizations/metrics_enabled`: URL, auth, observed behavior with personal OAuth tokens.
- **In Scope**: Auth requirements, 400 behavior with personal tokens, suspected enterprise-only gate.
- **Out of Scope**: Metrics collection implementation; organization admin settings UI.

### Request

```
GET https://api.anthropic.com/api/claude_code/organizations/metrics_enabled
Authorization:     Bearer {access_token}
anthropic-version: 2023-06-01
```

No request body.

### Observed Behavior

This endpoint returns HTTP 400 "Access Forbidden" for personal Claude Max OAuth tokens. It is not usable with the standard `user:*` scope set.

Hypothesis: The endpoint requires either:
- An organizational admin role in a team/enterprise account (not personal Max), or
- An additional OAuth scope not granted during standard login.

**Status**: Response body schema and success response format are unknown — no successful response has been observed.

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200 | Success (schema: TBD) |
| 400 | `"Access Forbidden"` — personal OAuth tokens; enterprise-only |
| 401 | Token invalid or expired |

### Observed Error Response

HTTP 400 with standard Max OAuth token:

```json
{"error": "Access Forbidden"}
```

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [005_claude_cli_roles.md](005_claude_cli_roles.md) | Org/workspace role lookup |
| doc | [../params/118_disable_telemetry.md](../params/118_disable_telemetry.md) | Disable telemetry (related feature) |
