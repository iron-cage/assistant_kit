# claude_quota — tests

## Responsibility

| File | Responsibility |
|------|---------------|
| readme.md | Test directory organization guide (this file) |
| rate_limit_test.rs | Unit tests T01–T16: parse_headers, QuotaError, RateLimitData, constants |
| oauth_usage_test.rs | Unit tests T17–T28: parse_oauth_usage, iso_to_unix_secs, OauthUsageData, PeriodUsage |

## Organization

Two files — each covers a distinct functional domain. All tests are offline (no network, no ureq in dev-dependencies).

## Domain Map

| Domain | File | What it tests |
|--------|------|---------------|
| Header parsing | `rate_limit_test.rs` | `parse_headers` happy path, missing headers, malformed headers |
| Error types | `rate_limit_test.rs` | `QuotaError` Display for all 4 variants, `std::error::Error` bound |
| OAuth usage parsing | `oauth_usage_test.rs` | `parse_oauth_usage` happy path, null periods, missing fields |
| Date conversion | `oauth_usage_test.rs` | `iso_to_unix_secs` known-date validation, invalid input |
| Usage data types | `oauth_usage_test.rs` | `OauthUsageData` and `PeriodUsage` field accessibility |
| Data types | `rate_limit_test.rs` | `RateLimitData` field accessibility |
| Constants | `rate_limit_test.rs` | `ANTHROPIC_BETA` canary (undocumented OAuth beta string) |

## Adding New Tests

- **New header field** added to `parse_headers`? Add missing-header + malformed-header tests, update the matrix.
- **New `QuotaError` variant**? Add Display test and extend T09 to cover the new variant.
- **New constant**? Add a canary test for any security-critical undocumented constant.
- **`fetch_rate_limits` offline logic**? Add here. Live network tests belong in `claude_profile/tests/cli/account_limits_test.rs`.
- **Structural change**: update test matrix in `rate_limit_test.rs` module doc first, then add the test.
