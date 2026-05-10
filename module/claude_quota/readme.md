# claude_quota

Anthropic API rate-limit HTTP transport — Layer `*` standalone primitive.

Provides types for representing rate-limit utilization data returned by the Anthropic API,
and an HTTP transport function (feature-gated) that fetches them via `POST /v1/messages`.

## Responsibility

Owns the Anthropic API wire protocol for rate-limit data: the URL, OAuth headers, HTTP body,
and response header parsing. No credentials handling, no output formatting — those belong
to consumers (`claude_profile`, `dream`).

## Feature Flags

| Feature   | Adds                         | Extra dep |
|-----------|------------------------------|-----------|
| (none)    | `RateLimitData`, `QuotaError`, `parse_headers` | — |
| `enabled` | `fetch_rate_limits(token: &str)` | `ureq ~2` |

## Public API

```rust
// Always available
pub struct RateLimitData { utilization_5h, reset_5h, utilization_7d, reset_7d, status }
pub enum   QuotaError    { HttpTransport(String), MissingHeader(String), MalformedHeader(String) }
pub fn     parse_headers<F: Fn(&str) -> Option<String>>(get: F) -> Result<RateLimitData, QuotaError>

// Only under `enabled` feature
pub fn fetch_rate_limits(token: &str) -> Result<RateLimitData, QuotaError>
```

## Files

| File | Responsibility |
|------|---------------|
| src/lib.rs | Types, error, constants, parse_headers, fetch_rate_limits |
| tests/readme.md | Test directory organization guide |
| tests/rate_limit_test.rs | Unit tests T01–T16 (offline, closure-based) |
| `verb/` | Shell scripts for each `do` protocol verb. |
