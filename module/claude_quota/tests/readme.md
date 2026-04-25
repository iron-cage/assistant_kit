# claude_quota — tests

## Responsibility

| File | Responsibility |
|------|---------------|
| readme.md | Test directory organization guide (this file) |
| rate_limit_test.rs | Unit tests T01–T16: parse_headers, QuotaError, RateLimitData, constants |

## Organization

Single file — all tests cover one functional domain (`parse_headers` + supporting types).
No subdirectories. All tests are offline (no network, no ureq in dev-dependencies).

## Domain Map

| Domain | File | What it tests |
|--------|------|---------------|
| Header parsing | `rate_limit_test.rs` | `parse_headers` happy path, missing headers, malformed headers |
| Error types | `rate_limit_test.rs` | `QuotaError` Display for all 3 variants, `std::error::Error` bound |
| Data types | `rate_limit_test.rs` | `RateLimitData` field accessibility |
| Constants | `rate_limit_test.rs` | `ANTHROPIC_BETA` canary (undocumented OAuth beta string) |

## Adding New Tests

- **New header field** added to `parse_headers`? Add missing-header + malformed-header tests, update the matrix.
- **New `QuotaError` variant**? Add Display test and extend T09 to cover the new variant.
- **New constant**? Add a canary test for any security-critical undocumented constant.
- **`fetch_rate_limits` offline logic**? Add here. Live network tests belong in `claude_profile/tests/cli/account_limits_test.rs`.
- **Structural change**: update test matrix in `rate_limit_test.rs` module doc first, then add the test.
