# tests/cli/

| File | Responsibility |
|------|----------------|
| `helpers.rs` | Shared binary runner, assertion, and fixture helpers. |
| `account_list_status_test.rs` | Help output, account list, and account status tests. |
| `account_mutations_test.rs` | Account save, switch, and delete tests. |
| `token_paths_test.rs` | Token status classification and paths output tests. |
| `cross_cutting_test.rs` | Cross-cutting and environment behavior tests. |
| `usage_test.rs` | Token usage statistics display tests. |
| `persist_test.rs` | PersistPaths: $PRO/$HOME/$USERPROFILE resolution, is_dir guard, ensure_exists. |
| `account_status_name_test.rs` | FR-16: `.account.status name::` optional param — 14 test scenarios. |
| `credentials_test.rs` | FR-17: `.credentials.status` live cred access without account store — cred01–cred05. |
| `account_limits_test.rs` | FR-18: `.account.limits` error paths — lim01–lim05 (IT-6 through IT-9). |
