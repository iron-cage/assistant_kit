# Token Refresh API

**Status**: Implemented | **Since**: 0.1.0

### Scope

- **Purpose**: Exchange an Anthropic OAuth refresh token for a fresh access token, and parse the wire response.
- **Responsibility**: Documents `TokenRefreshResult`, `AuthError`, `parse_response()`, and `refresh_token()` — the crate's entire public surface — and their behavioral contract.
- **In Scope**: OAuth token-refresh JSON response parsing (dependency-free), the blocking HTTP transport that performs the refresh, the `TOKEN_URL`/`CLIENT_ID` constants.
- **Out of Scope**: Credential storage, refresh scheduling/caching (-> `claude_profile`, `dream`), quota/usage data (-> `claude_quota`), redacting tokens in logs (-> `json_redact`).

## Description

`parse_response(body, now_ms)` extracts `access_token`, `refresh_token`, and `expires_in` from a flat OAuth token-refresh JSON body using raw string needles (`"\"key\":"`) — no `serde` dependency, so parsing is always available regardless of feature flags. `expires_at_ms` is computed as `now_ms + expires_in * 1000`, so the caller supplies the current time rather than the function reading the clock itself. `refresh_token(refresh_tok, scope)` (feature `enabled`) performs the actual network call: it POSTs a `grant_type=refresh_token` body to `TOKEN_URL` with `CLIENT_ID`, using a `ureq` agent configured with `http_status_as_error(false)` so non-2xx responses are inspected rather than turned into a generic transport error, then delegates to `parse_response` for the body.

## Interface

```rust
pub const TOKEN_URL : &str;   // "https://platform.claude.com/v1/oauth/token"
pub const CLIENT_ID : &str;   // Public OAuth client ID for the Claude desktop application

pub struct TokenRefreshResult
{
  pub access_token  : String,
  pub refresh_token : String,  // rotated on every refresh
  pub expires_at_ms : u64,     // now_ms + expires_in_secs * 1000
}

pub enum AuthError
{
  HttpTransport( String ),   // connection refused, TLS error, non-429 HTTP error status
  ResponseParse( String ),   // names the missing/malformed field
  RateLimited,                // server returned HTTP 429
}

pub fn parse_response( body : &str, now_ms : u64 ) -> Result< TokenRefreshResult, AuthError >;

#[ cfg( feature = "enabled" ) ]
pub fn refresh_token( refresh_tok : &str, scope : &str ) -> Result< TokenRefreshResult, AuthError >;
```

## Behavioral Contract

- `parse_response` locates each field via the literal needle `"\"key\":"` (colon included), which avoids prefix-collision between keys like `"token"` and `"access_token"`
- `expires_in` must be a bare JSON integer — a quoted string value (`"expires_in":"3600"`) is rejected as `ResponseParse("expires_in")`, not coerced
- Any of the three required fields (`access_token`, `refresh_token`, `expires_in`) absent or malformed returns `Err(ResponseParse(field_name))` naming that specific field
- `refresh_token()` returns `Err(AuthError::RateLimited)` specifically for HTTP 429 (distinct from other 4xx/5xx, which fall into `HttpTransport`), so callers can distinguish "back off" from other transport failures
- `refresh_token()` refreshes are one-shot: no retry, no backoff sleep, no caching — callers own that policy
- The crate has zero workspace dependencies — a Layer `*` standalone primitive, reusable by any caller without pulling in the rest of the workspace

## Sources

- `src/lib.rs` — implementation
- `tests/auth_test.rs` — Test Matrix T01–T06 coverage (offline, body-string interface only — no live network)
