# tests

Unit and integration tests for `claude_version`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `smoke_test.rs` | Verify binary exists and is reachable via `CARGO_BIN_EXE` |
| `cli_args_test.rs` | Binary arg parsing: flags, aliases, subcommands, validation |
| `settings_io_test.rs` | Settings JSON read/write and type-inference unit tests |
| `integration.rs` | Entry point test crate that includes all integration test modules |
| `integration/` | Module files for integration tests (included by `integration.rs`) |
| `lib_test.rs` | Library API: `register_commands()` callable and registers all commands |
| `manual/` | Manual testing plan for scenarios requiring human verification |
| `docs/` | Test planning documentation mirroring `docs/` structure |

## Integration Test Modules

| File | Responsibility |
|------|----------------|
| `integration/subprocess_helpers.rs` | Shared subprocess execution and fixture helpers |
| `integration/framework_test.rs` | Pipeline smoke tests: help listing, exit codes |
| `integration/read_commands_test.rs` | Read-only command integration tests (E1–E15) |
| `integration/mutation_commands_test.rs` | 4 mutation command integration tests |
| `integration/cross_cutting_test.rs` | Cross-cutting: dry+force, verbosity parity, format parity |
| `integration/error_messages_test.rs` | Error message content and format validation |
| `integration/algorithm_surface_test.rs` | Algorithm behavior surface tests |
| `integration/config_commands_test.rs` | `.config` command integration tests |
| `integration/feature_surface_test.rs` | Feature-level surface tests |
| `integration/scope_param_test.rs` | EC- edge case tests for the `scope::` parameter |
| `integration/unset_param_test.rs` | EC- edge case tests for the `unset::` parameter |
| `integration/config_identity_test.rs` | GI- interaction tests for Parameter Group 4: Config Identity |
| `integration/user_story_test.rs` | Acceptance tests for all user story specifications |
| `integration/format_surface_test.rs` | FM- format surface tests from tests/docs/cli/format/ specs |
| `integration/pitfall_surface_test.rs` | PF- pitfall surface tests from tests/docs/pitfall/ specs |
| `integration/catalog_surface_test.rs` | DD- catalog surface tests from tests/docs/catalog/ specs |
| `integration/version_param_test.rs` | EC- edge case tests for the `version::` parameter |
| `integration/dry_param_test.rs` | EC- edge case tests for the `dry::` parameter |
| `integration/force_param_test.rs` | EC- edge case tests for the `force::` parameter |
| `integration/verbosity_param_test.rs` | EC- edge case tests for the `v::`/`verbosity::` parameter |
| `integration/format_param_test.rs` | EC- edge case tests for the `format::` parameter |
| `integration/key_param_test.rs` | EC- edge case tests for the `key::` parameter |
| `integration/value_param_test.rs` | EC- edge case tests for the `value::` parameter |
| `integration/count_param_test.rs` | EC- edge case tests for the `count::` parameter |
| `integration/process_isolation_test.rs` | Kill-isolation regression: guard does not send kill signals |
