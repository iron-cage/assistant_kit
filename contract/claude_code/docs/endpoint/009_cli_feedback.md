# Endpoint: POST /api/claude_cli_feedback

### Scope

- **Purpose**: Submit feedback from the Claude CLI — thumbs up/down ratings or text feedback on Claude Code sessions.
- **Responsibility**: Wire contract for `POST /api/claude_cli_feedback`: URL, method, observed behavior with standard OAuth tokens.
- **In Scope**: Method, auth headers, 403 behavior with standard tokens.
- **Out of Scope**: Feedback form UI; feedback data retention policy.

### Request

```
POST https://api.anthropic.com/api/claude_cli_feedback
Authorization:     Bearer {access_token}
anthropic-version: 2023-06-01
Content-Type:      application/json
```

Request body schema is not confirmed — HTTP 403 is returned before body processing with standard tokens. Inferred fields from Claude Code binary analysis: `content` (feedback text), `rating` (thumbs value), `session_id`.

### Observed Behavior

HTTP 403 "Access Forbidden" for standard Max OAuth tokens. Feedback submission likely requires a feature flag or separate authentication path not available to standard personal accounts.

**Status**: Request body schema and success response format are unknown — no successful response has been observed.

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200/201 | Success (schema: TBD) |
| 403 | `"Access Forbidden"` — standard OAuth tokens insufficient |
| 401 | Token invalid or expired |

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [004_oauth_token.md](004_oauth_token.md) | Token refresh |
| doc | [../param/106_disable_feedback_command.md](../param/106_disable_feedback_command.md) | Env var to hide `/feedback` command |
