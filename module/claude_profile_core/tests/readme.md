# tests/

Unit tests for the `claude_profile_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `token_test.rs`   | Token expiry parsing and Valid / ExpiringSoon / Expired classification |
| `account_test.rs`         | Account lifecycle: save, delete, switch, store lock, credential perms |
| `account_session_settings_test.rs` | Session model/effort mutations in live settings.json |
| `account_ownership_test.rs` | Owner/claim fields and per-machine active markers |
| `account_tags_test.rs` | Tag normalization, set ops, lazy role migration (Feature 075) |
| `account_filter_test.rs` | Identity tag filter IO and eligibility predicate (Feature 076) |
| `account_quota_cache_test.rs` | Per-host quota cache tree and history ring (TSK-500/502) |
| `account_backend_test.rs` | Backend/redirect accounts, Kimi tier env vars, JSON field parsing |
| `account_refresh_test.rs` | Failure-path unit tests for `refresh_account_token`   |
| `account_fixture/` | Shared credential-store fixture builders for account test binaries |
