# Endpoint: GET /api/web/domain_info

### Scope

- **Purpose**: Check whether Claude Code is permitted to fetch content from a given domain — controls the web fetch feature's allow/deny decision.
- **Responsibility**: Complete wire contract for `GET /api/web/domain_info?domain=<domain>`: URL, query parameter, auth, full response schema.
- **In Scope**: Query parameter format, `can_fetch` semantics, error codes.
- **Out of Scope**: Web fetch implementation in Claude Code; domain allowlist administration.

### Request

```
GET https://api.anthropic.com/api/web/domain_info?domain=<domain>
Authorization: Bearer {access_token}
```

Query parameter `domain` is a bare domain name without scheme or path (e.g., `claude.ai`, not `https://claude.ai/`).

No `anthropic-version` header required. No request body.

### Response

HTTP 200 on success. Body is a JSON object.

| Field | Type | Semantics |
|-------|------|-----------|
| `domain` | string | Echo of the requested domain |
| `can_fetch` | bool | `true` = Claude Code may fetch from this domain; `false` = blocked |

### Example Response

HTTP 200 (`domain=claude.ai`, example response):

```json
{
  "domain": "claude.ai",
  "can_fetch": true
}
```

### `can_fetch` Semantics

`true` — Claude Code may make HTTP requests to the domain on behalf of the user.
`false` — The domain is blocked. Claude Code should not attempt to fetch from it.

The blocklist is maintained server-side and is not configurable per-user. Anthropic controls which domains are allowed/blocked.

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200 | Success |
| 401 | Token invalid or expired |
| 400 | Malformed or missing `domain` parameter |

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [004_oauth_token.md](004_oauth_token.md) | Token refresh |
