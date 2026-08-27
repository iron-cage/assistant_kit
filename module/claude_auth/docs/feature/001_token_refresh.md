# Feature: Token Refresh

### Scope

- **Purpose**: Specify the network exchange that trades a refresh token for a fresh access token against Anthropic's OAuth endpoint.
- **Responsibility**: Define the endpoint, client identity, request shape, HTTP status classification, and expiry computation.
- **In Scope**: Endpoint and client constants (FR-1, FR-2), request construction (FR-3, FR-4), status classification (FR-5–FR-7), expiry computation (FR-8), feature gating (FR-9).
- **Out of Scope**: Body parsing mechanics (→ `002_response_parsing.md`), signature contracts (→ `api/001_auth_surface.md`), what a caller does with the returned tokens (→ `claude_profile`).

### Design

**Endpoint and identity:**

| ID | Requirement |
|----|-------------|
| FR-1 | `TOKEN_URL` is `https://platform.claude.com/v1/oauth/token` |
| FR-2 | `CLIENT_ID` is the public OAuth client ID of the Claude desktop application, `9d1c250a-e61b-44d9-88ed-5944d1962f5e` |

Both are `pub const` and available with no feature enabled, so a consumer can inspect or
report the endpoint without linking an HTTP stack.

**Request construction:**

| ID | Requirement |
|----|-------------|
| FR-3 | The request is an HTTP POST to `TOKEN_URL` carrying header `Content-Type: application/json` |
| FR-4 | The body is a flat JSON object with exactly four keys: `grant_type` (constant `"refresh_token"`), `refresh_token`, `client_id`, and `scope` |

```json
{ "grant_type": "refresh_token", "refresh_token": "…", "client_id": "…", "scope": "…" }
```

**Status classification** — the agent is built with `http_status_as_error( false )` so that
every status reaches the classification below rather than being pre-converted into a transport
error by `ureq`:

| ID | Requirement |
|----|-------------|
| FR-5 | HTTP 429 maps to `AuthError::RateLimited`, distinctly from any other failure, so a caller can back off rather than retry immediately |
| FR-6 | Any other status ≥ 400 maps to `AuthError::HttpTransport( "HTTP {status}" )` |
| FR-7 | A connection, TLS, or body-read failure maps to `AuthError::HttpTransport` carrying the underlying error's string |

Classifying 429 separately is the reason `http_status_as_error` is disabled. Re-enabling it
would collapse 429 into a generic transport error and silently destroy the back-off signal.

**Expiry computation:**

| ID | Requirement |
|----|-------------|
| FR-8 | `expires_at_ms` is absolute, computed as `now_ms + expires_in * 1000` — the response's relative `expires_in` (seconds) is never stored as-is |

Inside `refresh_token`, `now_ms` is read from the system clock; if the clock is before the Unix
epoch the read falls back to `0` rather than panicking, which yields an already-expired token
and forces a refresh on next use — the safe direction to fail.

**Feature gating:**

| ID | Requirement |
|----|-------------|
| FR-9 | `refresh_token` is compiled only under feature `enabled`; the parsing surface it delegates to is always present regardless of features |

### Acceptance Criteria

FR-8's arithmetic is verified offline through the body-string interface, since
`parse_response` takes `now_ms` as a parameter — `tests/auth_test.rs` T01 asserts the computed
`expires_at_ms` for a known body and a known `now_ms`.

FR-1 and FR-2 are verified by inspection against the constants:

```bash
grep -nE 'pub const (TOKEN_URL|CLIENT_ID)' module/claude_auth/src/lib.rs
```

FR-5–FR-7 are **not** covered by an automated test. A live call is unreliable in CI precisely
because of FR-5 — the endpoint rate-limits — so the status classification is verified by
review, and `tests/auth_test.rs` records the omission explicitly as `N/A`. T06 does cover that
all three `AuthError` variants exist and satisfy `Display` + `std::error::Error`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [002_response_parsing.md](002_response_parsing.md) | How the response body is turned into a `TokenRefreshResult` |
| doc | [invariant/002_offline_parse_core.md](../invariant/002_offline_parse_core.md) | Why parsing stays reachable without the network feature |
| doc | [api/001_auth_surface.md](../api/001_auth_surface.md) | Signature and error contract for `refresh_token` |
| source | `../../src/lib.rs` | Implementation of `refresh_token` |
| test | `../../tests/auth_test.rs` | T01 (expiry arithmetic), T06 (error variants) |
