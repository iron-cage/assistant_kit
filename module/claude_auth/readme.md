# claude_auth

Anthropic OAuth token refresh transport — Layer `*` standalone primitive.

## Responsibility

Owns the OAuth token-refresh wire protocol: TOKEN_URL, CLIENT_ID, request body, response
parsing. No quota, no profile management, no output formatting — those belong to consumers
(`claude_profile`, `dream`).

## Feature Flags

| Feature   | Adds                          | Extra dep  |
|-----------|-------------------------------|------------|
| (none)    | `TokenRefreshResult`, `AuthError`, `parse_response`, `TOKEN_URL`, `CLIENT_ID` | — |
| `enabled` | `refresh_token(refresh_tok, scope)` | `ureq ~2` |

## Public API

```rust
// Always available
pub const TOKEN_URL : &str;
pub const CLIENT_ID : &str;
pub struct TokenRefreshResult { access_token: String, refresh_token: String, expires_at_ms: u64 }
pub enum   AuthError           { HttpTransport(String), ResponseParse(String), RateLimited }
pub fn     parse_response(body: &str, now_ms: u64) -> Result<TokenRefreshResult, AuthError>

// Only under `enabled` feature
pub fn refresh_token(refresh_tok: &str, scope: &str) -> Result<TokenRefreshResult, AuthError>
```

## Files

| File | Responsibility |
|------|---------------|
| `src/lib.rs` | Types, errors, constants, `parse_response`, `refresh_token` (feature-gated) |
| `tests/readme.md` | Test directory organization guide |
| `tests/auth_test.rs` | Unit tests T01–T06 for `parse_response` and `AuthError` (offline, no ureq) |
| `verb/` | Shell scripts for each `do` protocol verb. |
| `run/` | Shell scripts for container-orchestrated operations. |
