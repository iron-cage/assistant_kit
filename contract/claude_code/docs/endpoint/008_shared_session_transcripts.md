# Endpoint: POST /api/claude_code_shared_session_transcripts

### Scope

- **Purpose**: Publish a Claude Code session transcript for sharing — creates a shareable link to a conversation history.
- **Responsibility**: Wire contract for `POST /api/claude_code_shared_session_transcripts`: URL, method, observed behavior with standard OAuth tokens.
- **In Scope**: Method requirement (POST, not GET), 403 behavior, suspected auth scope gap.
- **Out of Scope**: Transcript format and field schema (not observed — 403 before processing); shared transcript viewing UI.

### Request

```
POST https://api.anthropic.com/api/claude_code_shared_session_transcripts
Authorization: Bearer {access_token}
Content-Type:  application/json

{"content": "<transcript_data>"}
```

The `content` field is inferred from the 405 response to GET and observed usage patterns in the Claude Code binary. The exact schema of `content` is not confirmed — HTTP 403 is returned before the body is processed with standard tokens.

### Method Notes

`GET /api/claude_code_shared_session_transcripts` returns HTTP 405 Method Not Allowed. This endpoint is POST-only.

### Observed Behavior

HTTP 403 "Access Forbidden" for standard Max OAuth tokens. The `user:sessions:claude_code` scope in the standard token set is insufficient, or the endpoint requires a separate authentication mechanism.

**Status**: Request body schema and success response format are unknown — no successful response has been observed.

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200/201 | Success (schema: TBD) |
| 403 | `"Access Forbidden"` — standard OAuth tokens insufficient |
| 405 | Method Not Allowed — only POST is accepted |
| 401 | Token invalid or expired |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [004_oauth_token.md](004_oauth_token.md) | Token refresh — scope is fixed at login |
