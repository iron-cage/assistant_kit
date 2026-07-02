# tests

Unit and integration tests for `claude_version`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `smoke_test.rs` | Verify binary exists and is reachable via `CARGO_BIN_EXE` |
| `cli_args_test.rs` | Entry point test crate for argument parsing tests |
| `cli_args_test/` | Module files for arg parsing tests, organised by domain (included by `cli_args_test.rs`) |
| `settings_io_test.rs` | Settings JSON read/write and type-inference unit tests |
| `cli.rs` | Entry point test crate that includes all CLI test modules |
| `cli/` | Module files for CLI tests, organised by domain (included by `cli.rs`) |
| `lib_test.rs` | Library API: `register_commands()` callable and registers all commands |
| `manual/` | Manual testing plan for scenarios requiring human verification |
| `docs/` | Test planning documentation mirroring `docs/` structure |
| `runbox/` | Container runner test environment (Dockerfile, config) |

## CLI Test Modules (`cli/`)

| File | Responsibility |
|------|----------------|
| `cli/subprocess_helpers.rs` | Shared subprocess execution and fixture helpers |
| `cli/framework_test.rs` | Pipeline smoke tests: help listing, exit codes |
| `cli/read_help_test.rs` | Integration tests for `.help` (E1) |
| `cli/read_status_test.rs` | Integration tests for `.status` and format edge cases (E2) |
| `cli/read_version_test.rs` | Integration tests for `.version.show` and `.version.list` (E3, E4) |
| `cli/read_processes_test.rs` | Integration tests for `.processes` (E6) |
| `cli/read_settings_test.rs` | Integration tests for `.settings.show` and `.settings.get` (E8, E9) |
| `cli/read_version_history_test.rs` | Integration tests for `.version.history` (E15) |
| `cli/mutation_version_install_test.rs` | Integration tests for `.version.install` (E5) |
| `cli/mutation_processes_kill_test.rs` | Integration tests for `.processes.kill` (E7) |
| `cli/mutation_version_guard_test.rs` | Integration tests for `.version.guard` (E14) |
| `cli/mutation_settings_set_test.rs` | Integration tests for `.settings.set` and value type inference (E10) |
| `cli/cross_cutting_test.rs` | Cross-cutting: dry+force, verbosity parity, format parity |
| `cli/error_messages_test.rs` | Error message content and format validation |
| `cli/algorithm_surface_test.rs` | Algorithm behavior surface tests |
| `cli/config_commands_test.rs` | `.config` command integration tests |
| `cli/feature_surface_test.rs` | Feature-level surface tests |
| `cli/scope_param_test.rs` | EC- edge case tests for the `scope::` parameter |
| `cli/unset_param_test.rs` | EC- edge case tests for the `unset::` parameter |
| `cli/config_identity_test.rs` | GI- interaction tests for Parameter Group 4: Config Identity |
| `cli/user_story_test.rs` | Acceptance tests for all user story specifications |
| `cli/format_surface_test.rs` | FM- format surface tests from tests/docs/cli/format/ specs |
| `cli/pitfall_surface_test.rs` | PF- pitfall surface tests from tests/docs/pitfall/ specs |
| `cli/catalog_surface_test.rs` | DD- design decision tests from tests/docs/feature/05_cli_design.md |
| `cli/version_param_test.rs` | EC- edge case tests for the `version::` parameter |
| `cli/dry_param_test.rs` | EC- edge case tests for the `dry::` parameter |
| `cli/force_param_test.rs` | EC- edge case tests for the `force::` parameter |
| `cli/verbosity_param_test.rs` | EC- edge case tests for the `v::`/`verbosity::` parameter |
| `cli/format_param_test.rs` | EC- edge case tests for the `format::` parameter |
| `cli/key_param_test.rs` | EC- edge case tests for the `key::` parameter |
| `cli/value_param_test.rs` | EC- edge case tests for the `value::` parameter |
| `cli/count_param_test.rs` | EC- edge case tests for the `count::` parameter |
| `cli/process_isolation_test.rs` | Kill-isolation regression: guard does not send kill signals |
| `cli/params_command_test.rs` | `.params` command integration tests |
| `cli/kind_param_test.rs` | EC- edge case tests for the `kind::` parameter |

## Arg Parsing Test Modules (`cli_args_test/`)

| File | Responsibility |
|------|----------------|
| `cli_args_test/subprocess_helpers.rs` | Container guard, binary runner, and output extractors |
| `cli_args_test/help_test.rs` | `.help` anywhere-in-argv routing and EC-3..EC-8 spec edge cases |
| `cli_args_test/parsing_test.rs` | Command recognition, param syntax enforcement, unknown rejection |
| `cli_args_test/param_verbosity_test.rs` | `v::` / `verbosity::` range, type, last-wins, canonical-key parity |
| `cli_args_test/param_format_test.rs` | `format::` empty, wrong-case, last-wins, default-absent |
| `cli_args_test/param_bool_test.rs` | `dry::` / `force::` acceptance, non-0/1 rejection, last-wins |
| `cli_args_test/param_numeric_test.rs` | `count::` / `interval::` / `version::` overflow and semver format |
| `cli_args_test/type_surface_test.rs` | Type contract tests: VerbosityLevel, OutputFormat, VersionSpec, SettingsKey, SettingsValue |
