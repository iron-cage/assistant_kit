# claude_quota

Anthropic API quota HTTP transports — Layer `*` standalone primitive.

Provides types and transport functions for four Anthropic quota and account APIs:
- **`fetch_rate_limits`**: POST `/v1/messages` response headers — per-request rate-limit data (used by `.account.limits`).
- **`fetch_oauth_usage`**: GET `/api/oauth/usage` — per-period usage breakdown including Sonnet-only weekly quota (used by `.usage`, `.account.inspect`).
- **`fetch_oauth_account`**: GET `/api/oauth/account` — account identity (tagged_id, uuid, email, name), billing type, subscription state, and full membership list.
- **`fetch_claude_cli_roles`**: GET `/api/oauth/claude_cli/roles` — org and workspace identity.
- **`fetch_models`**: GET `/v1/models` — live model catalog (id, display name); `STATIC_MODELS` is the embedded compile-time fallback catalog.

## Responsibility

Owns the Anthropic API wire protocols for quota data: URLs, OAuth headers, request/response
parsing. No credentials handling, no output formatting — those belong to consumers (`claude_profile`, `dream`).

## Feature Flags

| Feature   | Adds                                                                                                | Extra dep |
|-----------|-----------------------------------------------------------------------------------------------------|-----------|
| (none)    | All types, all parse functions, all constants, `select_membership_index`, `iso_to_unix_secs`, `STATIC_MODELS` | —         |
| `enabled` | `fetch_rate_limits`, `fetch_oauth_usage`, `fetch_oauth_account`, `fetch_claude_cli_roles`, `fetch_models` | `ureq ~3` |

## Public API

```rust
// ── Constants ──────────────────────────────────────────────────────────────────
pub const API_URL                      : &str;  // POST /v1/messages (rate-limit headers)
pub const ANTHROPIC_BETA               : &str;  // "oauth-2025-04-20"
pub const ANTHROPIC_VERSION            : &str;  // "2023-06-01"
pub const OAUTH_USAGE_URL              : &str;  // GET /api/oauth/usage
pub const OAUTH_ACCOUNT_URL            : &str;  // GET /api/oauth/account
pub const CLAUDE_CLI_ROLES_URL         : &str;  // GET /api/oauth/claude_cli/roles
pub const MODELS_URL                   : &str;  // GET /v1/models
pub const STATIC_MODELS                : &[ModelInfo];  // embedded fallback catalog

// ── Types (always available) ───────────────────────────────────────────────────
pub struct RateLimitData      { utilization_5h, reset_5h, utilization_7d, reset_7d, status }
pub struct PeriodUsage        { utilization: f64, resets_at: Option<String> }
pub struct OauthUsageData     { five_hour, seven_day, seven_day_sonnet: Option<PeriodUsage> }
pub struct OauthAccountData   { tagged_id, uuid, email_address, full_name, display_name, billing_type, has_max, capabilities, rate_limit_tier, org_created_at, memberships }
pub struct MembershipRaw      { index, billing_type, has_max, capabilities, org_created_at, rate_limit_tier }
pub struct ClaudeCliRolesData { organization_uuid, organization_name, organization_role, workspace_uuid, workspace_name }
pub struct ModelInfo          { id, display_name, created_at, max_input_tokens, max_tokens, capabilities }
pub enum   QuotaError         { HttpTransport(String), HttpStatus(u16), MissingHeader(String), MalformedHeader(String), ResponseParse(String) }
// HttpStatus Display is the stable machine-matchable form "HTTP NNN" — consumers
// match the variant (or that anchored prefix), never a bare code substring.

// ── Parse functions (always available, offline-testable) ──────────────────────
pub fn parse_headers<F: Fn(&str) -> Option<String>>(get: F) -> Result<RateLimitData, QuotaError>
pub fn parse_oauth_usage(body: &str)      -> Result<OauthUsageData, QuotaError>
pub fn parse_oauth_account(body: &str)    -> Result<OauthAccountData, QuotaError>
pub fn parse_claude_cli_roles(body: &str) -> Result<ClaudeCliRolesData, QuotaError>
pub fn select_membership_index(memberships: &[MembershipRaw]) -> usize
pub fn iso_to_unix_secs(s: &str) -> Option<u64>

// ── Network functions (feature = "enabled" only) ──────────────────────────────
pub fn fetch_rate_limits(token: &str)      -> Result<RateLimitData, QuotaError>
pub fn fetch_oauth_usage(token: &str)      -> Result<OauthUsageData, QuotaError>
pub fn fetch_oauth_account(token: &str)    -> Result<OauthAccountData, QuotaError>
pub fn fetch_claude_cli_roles(token: &str) -> Result<ClaudeCliRolesData, QuotaError>
pub fn fetch_models(token: &str)           -> Result<Vec<ModelInfo>, QuotaError>
```

All network functions share one agent configuration: `https_only`, a 30s global
timeout, and per-phase connect/response/body timeouts — no request can hang
indefinitely regardless of which phase stalls.

## Files

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/lib.rs` | Types, constants, all parse functions, all network functions (`enabled`) |
| `tests/readme.md` | Test directory organization guide |
| `tests/rate_limit_test.rs` | Unit tests T01–T16, ET-01: `parse_headers`, `QuotaError`, `RateLimitData`, constants |
| `tests/oauth_usage_test.rs` | Unit tests T17–T28, FT/BT/UT series: `parse_oauth_usage`, `iso_to_unix_secs`, `OauthUsageData`, `PeriodUsage` |
| `tests/oauth_account_test.rs` | Unit tests MRE-237, AT series: `parse_oauth_account` membership selection and scanner hardening |
| `tests/bug172_guard_test.rs` | Static-analysis guard: no bare ureq calls without timeout configuration |
| `docs/` | Endpoint reference and API documentation. |
| `verb/` | Shell scripts for each `do` protocol verb. |
