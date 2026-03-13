# tests

Unit and integration tests for `claude_manager`.

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

## Integration Test Modules

| File | Responsibility |
|------|----------------|
| `integration/helpers.rs` | Shared subprocess execution and fixture helpers |
| `integration/framework_test.rs` | Pipeline smoke tests: help listing, exit codes |
| `integration/read_commands_test.rs` | Read-only command integration tests (E1–E15) |
| `integration/mutation_commands_test.rs` | 4 mutation command integration tests |
| `integration/cross_cutting_test.rs` | Cross-cutting: dry+force, verbosity parity, format parity |
| `integration/error_messages_test.rs` | Error message content and format validation |
