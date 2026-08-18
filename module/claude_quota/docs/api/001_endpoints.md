# API: Endpoints

### Scope

- **Purpose**: Document the programmatic interface of the claude_quota Anthropic endpoint clients.
- **Responsibility**: Specify the parse/fetch contracts for the five Anthropic HTTP endpoints and their shared error type.
- **In Scope**: All public items of `src/lib.rs`, grouped by endpoint cluster below.
- **Out of Scope**: OAuth token acquisition/refresh (callers supply a valid `token: &str`), quota caching and rendering (consumer crates), retry policy (consumers decide; `QuotaError::HttpStatus` gives them a stable signal).

### Abstract

`claude_quota` is a dependency-light HTTP client for five Anthropic API endpoints. Every endpoint follows the same split: a pure `parse_*` function over a response body (offline-testable, no I/O) and a `fetch_*` function that performs the HTTP call and delegates to the parser. Network functions are gated behind the `enabled` feature and share one hardened agent configuration: `https_only`, a 30s global timeout, and per-phase connect/response/body timeouts — no request can hang indefinitely regardless of which phase stalls (guarded structurally by `tests/bug172_guard_test.rs`: no bare `ureq::*` calls may bypass the configured agent). Endpoint URLs are `pub const`, with one test seam: the `CLAUDE_QUOTA_BASE_URL` env var (name exported as `BASE_URL_ENV`) swaps the origin while preserving each endpoint's path, letting tests target a real local HTTP server. `https_only` is relaxed solely when that override points at plaintext loopback (`127.*`/`localhost`); any other plaintext origin stays rejected. Unset, every fetch targets the live API unchanged (live-token test lane).

### Clusters

#### Shared error type

`QuotaError` — `HttpTransport(String)` (network/TLS failure), `HttpStatus(u16)` (non-success HTTP status ≥ 400; the `Display` form `HTTP NNN` is a stable contract for retry/refresh predicates — match the variant or the anchored `"HTTP "` prefix, never a bare code substring), `MissingHeader(String)`, `MalformedHeader(String)`, `ResponseParse(String)` (names the missing/malformed field).

#### Rate limits (`/v1/messages` headers)

`API_URL`, `ANTHROPIC_BETA`, `ANTHROPIC_VERSION`, `BASE_URL_ENV`, `RateLimitData` (5h/7d utilization fractions 0.0–1.0, Unix reset timestamps, `status` string `allowed`/`allowed_warning`/`rejected`), `parse_headers` (generic over a header-lookup closure — testable without HTTP), `fetch_rate_limits`.

#### OAuth usage (`/api/oauth/usage`)

`OAUTH_USAGE_URL`, `PeriodUsage` (utilization as a percentage 0.0–100.0, optional ISO-8601 `resets_at`), `OauthUsageData` (three optional buckets: `five_hour`, `seven_day`, `seven_day_sonnet` — `None` when the server returns `null`), `iso_to_unix_secs` (dependency-free ISO-8601 → Unix seconds; assumes UTC, `None` on parse failure), `parse_oauth_usage`, `fetch_oauth_usage`.

#### OAuth account (`/api/oauth/account`)

`OAUTH_ACCOUNT_URL`, `OauthAccountData` (identity, billing type, `has_max`, capabilities, rate-limit tier, org creation date, raw memberships), `MembershipRaw`, `select_membership_index` (the single authority for picking the active membership from a multi-entry list), `parse_oauth_account`, `fetch_oauth_account`.

#### CLI roles (`/api/oauth/claude_cli/roles`)

`CLAUDE_CLI_ROLES_URL`, `ClaudeCliRolesData` (organization uuid/name/role, workspace uuid/name), `parse_claude_cli_roles`, `fetch_claude_cli_roles`.

#### Models (`/v1/models`)

`MODELS_URL`, `ModelInfo` (id, display name, creation date, token limits, capabilities — `&'static` fields), `STATIC_MODELS` (compile-time catalog of known Claude models), `fetch_models` (live model list).

### Error Handling

All `fetch_*` functions return `Result<_, QuotaError>`; all `parse_*` functions return `Result<_, QuotaError>` with `ResponseParse`/`MissingHeader`/`MalformedHeader` naming the offending field. `iso_to_unix_secs` and `select_membership_index` are `Option`-returning/infallible. Note the utilization unit difference between clusters: `RateLimitData` carries fractions (0.0–1.0), `PeriodUsage` carries percentages (0.0–100.0).

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | All clusters above (single-file crate) |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/rate_limit_test.rs` | `parse_headers`, `QuotaError`, `RateLimitData`, constants |
| `../../tests/oauth_usage_test.rs` | `parse_oauth_usage`, `iso_to_unix_secs`, `OauthUsageData`, `PeriodUsage` |
| `../../tests/oauth_account_test.rs` | `parse_oauth_account` membership selection and scanner hardening |
| `../../tests/bug172_guard_test.rs` | Structural guard: no bare `ureq` calls bypassing the timeout-configured agent |
| `../../tests/base_url_seam_test.rs` | `BASE_URL_ENV` seam: path grafting, loopback `https_only` carve-out, non-loopback plaintext rejection |
