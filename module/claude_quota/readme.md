# claude_quota

Anthropic API quota HTTP transports — Layer `*` standalone primitive.

Provides types and transport functions for two Anthropic quota APIs:
- **`fetch_rate_limits`**: POST `/v1/messages` response headers — per-request rate-limit data (used by `.account.limits`).
- **`fetch_oauth_usage`**: GET `/api/oauth/usage` — per-period usage breakdown including Sonnet-only weekly quota (used by `.usage`).

## Responsibility

Owns the Anthropic API wire protocols for quota data: URLs, OAuth headers, request/response
parsing. No credentials handling, no output formatting — those belong to consumers (`claude_profile`, `dream`).

## Feature Flags

| Feature   | Adds                         | Extra dep |
|-----------|------------------------------|-----------|
| (none)    | `RateLimitData`, `OauthUsageData`, `PeriodUsage`, `QuotaError`, `parse_headers`, `parse_oauth_usage` | — |
| `enabled` | `fetch_rate_limits(token)`, `fetch_oauth_usage(token)` | `ureq ~2` |

## Public API

```rust
// Always available — rate-limit headers (used by .account.limits)
pub struct RateLimitData { utilization_5h, reset_5h, utilization_7d, reset_7d, status }
pub fn     parse_headers<F: Fn(&str) -> Option<String>>(get: F) -> Result<RateLimitData, QuotaError>

// Always available — OAuth usage endpoint (used by .usage)
pub struct OauthUsageData { five_hour: Option<PeriodUsage>, seven_day: Option<PeriodUsage>, seven_day_sonnet: Option<PeriodUsage> }
pub struct PeriodUsage    { utilization: f64, resets_at: Option<String> }
pub fn     parse_oauth_usage(body: &str) -> Result<OauthUsageData, QuotaError>

// Shared error type
pub enum QuotaError { HttpTransport(String), MissingHeader(String), MalformedHeader(String), ResponseParse(String) }

// Only under `enabled` feature
pub fn fetch_rate_limits(token: &str) -> Result<RateLimitData, QuotaError>
pub fn fetch_oauth_usage(token: &str) -> Result<OauthUsageData, QuotaError>
```

## Files

| File | Responsibility |
|------|---------------|
| src/lib.rs | Types, errors, constants, `parse_headers`, `parse_oauth_usage`, `fetch_rate_limits`, `fetch_oauth_usage` |
| tests/readme.md | Test directory organization guide |
| tests/rate_limit_test.rs | Unit tests T01–T16 for `parse_headers` (offline, closure-based) |
| tests/oauth_usage_test.rs | Unit tests T17–T28 for `parse_oauth_usage` (offline, JSON string-based) |
| `verb/` | Shell scripts for each `do` protocol verb. |
