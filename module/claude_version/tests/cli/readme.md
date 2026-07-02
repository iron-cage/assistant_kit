# tests/cli/

Module files for CLI tests, organised by domain. All files are included by `tests/cli.rs`
(the test crate entry point). Tests invoke the `claude_version` binary via
`std::process::Command` using the `CARGO_BIN_EXE_claude_version` env macro.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `subprocess_helpers.rs` | Subprocess execution and fixture helpers for integration tests |
| `framework_test.rs` | Pipeline smoke tests: `.help`, unknown command, exit codes |
| `read_help_test.rs` | Integration tests for `.help` (E1) |
| `read_status_test.rs` | Integration tests for `.status` and format edge cases (E2) |
| `read_version_test.rs` | Integration tests for `.version.show` and `.version.list` (E3, E4) |
| `read_processes_test.rs` | Integration tests for `.processes` (E6) |
| `read_settings_test.rs` | Integration tests for `.settings.show` and `.settings.get` (E8, E9) |
| `read_version_history_test.rs` | Integration tests for `.version.history` (E15) |
| `mutation_version_install_test.rs` | Integration tests for `.version.install` (E5) |
| `mutation_processes_kill_test.rs` | Integration tests for `.processes.kill` (E7) |
| `mutation_version_guard_test.rs` | Integration tests for `.version.guard` (E14) |
| `mutation_settings_set_test.rs` | Integration tests for `.settings.set` and value type inference (E10) |
| `config_commands_test.rs` | Integration tests for `.config` command (IT + FT) |
| `cross_cutting_test.rs` | Cross-cutting: dry+force interaction, verbosity parity, format parity |
| `error_messages_test.rs` | Error message content and format validation |
| `feature_surface_test.rs` | FT- feature surface tests from tests/docs/feature/ specs |
| `algorithm_surface_test.rs` | AC- algorithm surface tests from tests/docs/algorithm/ specs |
| `scope_param_test.rs` | EC- edge case tests for the `scope::` parameter |
| `unset_param_test.rs` | EC- edge case tests for the `unset::` parameter |
| `config_identity_test.rs` | GI- interaction tests for Parameter Group 4: Config Identity |
| `user_story_test.rs` | Acceptance tests for all user story specifications |
| `format_surface_test.rs` | FM- format surface tests from tests/docs/cli/format/ specs |
| `pitfall_surface_test.rs` | PF- pitfall surface tests from tests/docs/pitfall/ specs |
| `catalog_surface_test.rs` | DD- design decision tests from tests/docs/feature/05_cli_design.md |
| `version_param_test.rs` | EC- edge case tests for the `version::` parameter |
| `dry_param_test.rs` | EC- edge case tests for the `dry::` parameter |
| `force_param_test.rs` | EC- edge case tests for the `force::` parameter |
| `verbosity_param_test.rs` | EC- edge case tests for the `v::`/`verbosity::` parameter |
| `format_param_test.rs` | EC- edge case tests for the `format::` parameter |
| `key_param_test.rs` | EC- edge case tests for the `key::` parameter |
| `value_param_test.rs` | EC- edge case tests for the `value::` parameter |
| `count_param_test.rs` | EC- edge case tests for the `count::` parameter |
| `process_isolation_test.rs` | Kill-isolation regression: guard does not send kill signals to processes |
| `params_command_test.rs` | IT- integration tests for the `.params` command (IT-1 through IT-14) |
| `kind_param_test.rs` | EC- and TC- tests for the `kind::` parameter and `ParamKind` type |
| `runtime_files_test.rs` | Integration tests for `.runtime_files` (IT-1..IT-9, FT-1..FT-5) |
