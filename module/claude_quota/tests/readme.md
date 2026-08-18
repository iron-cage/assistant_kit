# claude_quota — tests

## Responsibility

| File | Responsibility |
|------|---------------|
| readme.md | Test directory organization guide (this file) |
| rate_limit_test.rs | Unit tests T01–T16, ET-01: parse_headers, QuotaError, RateLimitData, constants |
| oauth_usage_test.rs | Unit tests T17–T28, FT/BT/UT series: parse_oauth_usage, iso_to_unix_secs, OauthUsageData, PeriodUsage |
| oauth_account_test.rs | Unit tests MRE-237, AT series: parse_oauth_account membership selection and scanner hardening |
| bug172_guard_test.rs | Static-analysis guard: no bare ureq::get()/post() without timeout |
| base_url_seam_test.rs | SM series: `CLAUDE_QUOTA_BASE_URL` override against a real loopback HTTP server |

## Organization

One file per functional domain. No test touches an external network: parse-layer
tests are pure string/closure tests, and the seam tests (`base_url_seam_test.rs`,
feature `enabled` only) talk to a std `TcpListener` on loopback.

## Domain Map

| Domain | File | What it tests |
|--------|------|---------------|
| Header parsing | `rate_limit_test.rs` | `parse_headers` happy path, missing headers, malformed headers |
| Error types | `rate_limit_test.rs` | `QuotaError` Display for all 5 variants (incl. `HttpStatus` "HTTP NNN" contract), `std::error::Error` bound |
| OAuth usage parsing | `oauth_usage_test.rs` | `parse_oauth_usage` happy path, null periods, missing fields, spaced colons |
| OAuth account parsing | `oauth_account_test.rs` | `parse_oauth_account` membership selection (BUG-237), identity scoping, capability anchoring |
| Date conversion | `oauth_usage_test.rs` | `iso_to_unix_secs` known-date validation, invalid input |
| Usage data types | `oauth_usage_test.rs` | `OauthUsageData` and `PeriodUsage` field accessibility |
| Data types | `rate_limit_test.rs` | `RateLimitData` field accessibility |
| Constants | `rate_limit_test.rs` | `ANTHROPIC_BETA` canary (undocumented OAuth beta string) |
| Base-URL seam | `base_url_seam_test.rs` | `fetch_oauth_usage`/`fetch_rate_limits` against a loopback server: path grafting, header passage, non-loopback plaintext rejection |

## Adding New Tests

- **New header field** added to `parse_headers`? Add missing-header + malformed-header tests, update the matrix.
- **New `QuotaError` variant**? Add Display test and extend T09 to cover the new variant.
- **New constant**? Add a canary test for any security-critical undocumented constant.
- **`fetch_rate_limits` offline logic**? Add here. Live network tests belong in `claude_profile/tests/cli/account_limits_test.rs`.
- **New `fetch_*` transport**? Route its URL through `resolved_url` in `src/lib.rs` and add an SM case in `base_url_seam_test.rs` proving path grafting against the loopback server.
- **Structural change**: update test matrix in `rate_limit_test.rs` module doc first, then add the test.
