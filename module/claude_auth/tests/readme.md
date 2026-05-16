# claude_auth — tests

## Responsibility

| File | Responsibility |
|------|---------------|
| `readme.md` | Test directory organization guide (this file) |
| `auth_test.rs` | Unit tests T01–T06: `parse_response`, `AuthError`, `TokenRefreshResult`, constants |

## Organization

Single file — all tests cover one functional domain (`parse_response` + supporting types).
No subdirectories. All tests are offline (no network, no ureq in dev-dependencies).

## Domain Map

| Domain | File | What it tests |
|--------|------|---------------|
| Response parsing | `auth_test.rs` | `parse_response` happy path, missing fields, malformed values |
| Error types | `auth_test.rs` | `AuthError` Display for all 3 variants, `std::error::Error` bound |
| Data types | `auth_test.rs` | `TokenRefreshResult` field accessibility and `expires_at_ms` formula |

## Adding New Tests

- **New required field** added to `parse_response`? Add missing-field test, update T01, update matrix.
- **New `AuthError` variant**? Add Display test and extend T06 to cover the new variant.
- **`refresh_token` offline logic**? Not applicable — uses live network; live tests belong in `claude_profile/tests/`.
- **Structural change**: update test matrix in `auth_test.rs` module doc first, then add the test.
