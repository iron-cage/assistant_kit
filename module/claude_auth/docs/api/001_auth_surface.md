# API: Auth Surface

### Scope

- **Purpose**: Pin the signature and error contract of every item `claude_auth` exports, so a consumer can depend on it without reading the source.
- **In Scope**: All public items in `lib.rs` — two constants, two types, two functions.
- **Out of Scope**: The private helpers `parse_string_field` and `parse_u64_field` (their behavior is specified in [feature/002_response_parsing.md](../feature/002_response_parsing.md), but they are not API).

### Availability

Every item below is available with **no feature enabled**, except `refresh_token`, which
requires feature `enabled` (and with it, `ureq`). See
[invariant/002_offline_parse_core.md](../invariant/002_offline_parse_core.md).

### Constants

| Signature | Contract |
|-----------|----------|
| `pub const TOKEN_URL : &str` | `https://platform.claude.com/v1/oauth/token`. Changing it is a wire-protocol change — see [feature/procedure.md](../feature/procedure.md). |
| `pub const CLIENT_ID : &str` | The public OAuth client ID of the Claude desktop application. Public by design; it is an identifier, not a secret. |

### `TokenRefreshResult`

A plain data carrier — all three fields are `pub`, there are no methods, and it derives only
`Debug`.

| Field | Contract |
|-------|----------|
| `access_token : String` | Fresh Bearer token. |
| `refresh_token : String` | The **new** refresh token. Anthropic rotates this on every refresh, so a caller that keeps its old one will fail the next refresh. Persist this value. |
| `expires_at_ms : u64` | Absolute expiry, milliseconds since the Unix epoch — not the relative `expires_in` the server sent. |

`Debug` is derived and will print token material. Do not log a `TokenRefreshResult` whole.

### `AuthError`

Derives `Debug`; implements `Display` and `std::error::Error`. Has no `source()` — the
underlying error is flattened into a `String` at construction so the type carries no lifetime
or dependency on `ureq`.

| Variant | Meaning | Caller's response |
|---------|---------|-------------------|
| `AuthError::HttpTransport( String )` | Connection, TLS, body-read failure, or any HTTP status ≥ 400 other than 429. The `String` is either the underlying error text or `"HTTP {status}"`. | Retry or surface; not distinguishable by status without parsing the string. |
| `AuthError::ResponseParse( String )` | A required field was absent or malformed. The `String` names the field — `"access_token"`, `"refresh_token"`, or `"expires_in"`. | Do not retry; the server contract changed. |
| `AuthError::RateLimited` | The server returned HTTP 429. | Back off before retrying. |

`RateLimited` is a separate variant precisely so back-off is expressible without string
matching — see [feature/001_token_refresh.md](../feature/001_token_refresh.md) FR-5.

### Functions

| Signature | Contract |
|-----------|----------|
| `parse_response( body : &str, now_ms : u64 ) -> Result< TokenRefreshResult, AuthError >` | Pure. No I/O, no clock read. `now_ms` is milliseconds since the Unix epoch, supplied by the caller, and is used only to turn the response's relative `expires_in` into an absolute `expires_at_ms`. Errors: `ResponseParse` only. |
| `refresh_token( refresh_tok : &str, scope : &str ) -> Result< TokenRefreshResult, AuthError >` | Feature `enabled`. **Blocking** — performs a synchronous HTTP POST; do not call from an async executor without `spawn_blocking`. Reads the system clock once, then delegates to `parse_response`. Errors: all three variants. |

Neither function retries. Retry and back-off policy belongs to the caller.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/001_token_refresh.md](../feature/001_token_refresh.md) | Behavior behind `refresh_token` |
| doc | [feature/002_response_parsing.md](../feature/002_response_parsing.md) | Behavior behind `parse_response` |
| doc | [invariant/002_offline_parse_core.md](../invariant/002_offline_parse_core.md) | Why `now_ms` is a parameter rather than a clock read |
| source | `../../src/lib.rs` | The implementation this contract pins |
| test | `../../tests/auth_test.rs` | T01–T06 |
