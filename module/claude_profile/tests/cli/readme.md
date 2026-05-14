# tests/cli/

| File | Responsibility |
|------|----------------|
| `helpers.rs` | Shared binary runner, assertion, and fixture helpers. |
| `accounts_test.rs` | Help output and `.accounts` command tests (h01–h07, acc01–acc16). |
| `account_mutations_test.rs` | Account save, use, and delete tests. |
| `token_paths_test.rs` | Token status classification and paths output tests. |
| `cross_cutting_test.rs` | Cross-cutting and environment behavior tests. |
| `usage_test.rs` | Live rate-limit quota table display tests. |
| `persist_test.rs` | PersistPaths: $PRO/$HOME/$USERPROFILE resolution, is_dir guard, ensure_exists. |
| `credentials_test.rs` | FR-17: `.credentials.status` live cred access without account store — cred01–cred05. |
| `credentials_status_help_test.rs` | FR-17: `.credentials.status` help descriptions — csh01–csh02. |
| `account_limits_test.rs` | FR-18: `.account.limits` error paths — lim01–lim05 (IT-5 through IT-8). |
| `dot_test.rs` | Help output and `.` / `.help` delegation tests (dot01–dot12). |
