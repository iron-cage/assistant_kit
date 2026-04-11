# tests/

| File | Responsibility |
|------|----------------|
| `account_tests.rs` | Account CRUD: save, list, switch, delete, active-guard. |
| `token_tests.rs` | TokenStatus classification: Valid, ExpiringSoon, Expired. |
| `paths_tests.rs` | ClaudePaths: all canonical path methods, HOME-not-set guard. |
| `responsibility_no_process_execution_test.rs` | Guard: no std::process import anywhere in crate source. |
| `lib_test.rs` | Library exports: COMMANDS_YAML, register_commands(), command presence. |
| `cli_adapter_test.rs` | Adapter and output module: argv conversion, aliases, bool normalization, validation, json_escape, format_duration_secs. |
| `cli_integration_test.rs` | CLI binary integration: entry point for integration/ modules. |
| `integration/` | Split integration test modules (help, accounts, token, paths, env, persist). |
| `manual/` | Manual testing plan: live Claude Code account switching. |
