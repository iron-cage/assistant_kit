# tests/

Unit tests for the `claude_profile_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `token_test.rs`   | Token expiry parsing and Valid / ExpiringSoon / Expired classification |
| `account_test.rs`         | Account deletion, snapshot cleanup, and quota cache storage (TSK-500 two-tier) |
| `account_refresh_test.rs` | Failure-path unit tests for `refresh_account_token`   |
