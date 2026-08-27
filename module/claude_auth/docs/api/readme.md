# API Doc Entity

### Scope

**Responsibilities:** Public API contracts for the `claude_auth` crate.
**In Scope:** Token-refresh response parsing, OAuth token-refresh HTTP transport.
**Out of Scope:** Internal field-extraction helpers, caller-specific credential storage.

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_token_refresh_api.md` | TokenRefreshResult, AuthError, parse_response, refresh_token contract |
