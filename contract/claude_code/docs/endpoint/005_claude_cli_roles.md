# Endpoint: GET /api/oauth/claude_cli/roles

### Scope

- **Purpose**: Retrieve the authenticated user's organization and workspace membership context for the Claude CLI — provides org UUID, org name, and role assignments.
- **Responsibility**: Complete wire contract for `GET /api/oauth/claude_cli/roles`: URL, auth, full response schema, and role semantics.
- **In Scope**: Request headers, response JSON schema, workspace field behavior (null for personal accounts), error codes.
- **Out of Scope**: API key creation (→ `006_create_api_key.md`); org-level quota (→ `007_metrics_enabled.md`).

### Request

```
GET https://api.anthropic.com/api/oauth/claude_cli/roles
Authorization:     Bearer {access_token}
anthropic-version: 2023-06-01
```

No request body.

### Response

HTTP 200 on success. Body is a JSON object.

| Field | Type | Observed | Semantics |
|-------|------|----------|-----------|
| `organization_uuid` | string | UUID | The org the user belongs to |
| `organization_name` | string | `"i11@wbox.pro's Organization"` | Display name of the org |
| `organization_role` | string | `"admin"` | User's role in the org |
| `workspace_uuid` | string\|null | `null` | Workspace UUID (enterprise feature) |
| `workspace_name` | string\|null | `null` | Workspace display name |
| `workspace_role` | string\|null | `null` | User's role in the workspace |

**Personal accounts**: `workspace_uuid`, `workspace_name`, and `workspace_role` are all `null`. Workspaces are an enterprise/team feature.

### Example Response

HTTP 200 (personal Max account, i11@wbox.pro, sampled 2026-05-23):

```json
{
  "organization_uuid": "05122eb7-d5c1-4e65-9245-7f05cb2dc8db",
  "organization_name": "i11@wbox.pro's Organization",
  "organization_role": "admin",
  "workspace_uuid": null,
  "workspace_name": null,
  "workspace_role": null
}
```

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200 | Success |
| 401 | Token invalid or expired |
| 403 | Token lacks required scope |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [006_create_api_key.md](006_create_api_key.md) | API key creation — uses `org:create_api_key` scope |
| doc | [002_oauth_account.md](002_oauth_account.md) | Full account identity and org membership |
