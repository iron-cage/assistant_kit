# tests/integration/

Module files for integration tests. All files are included by `tests/integration.rs`
(the test crate entry point). Tests invoke the `claude_version` binary via
`std::process::Command` using the `CARGO_BIN_EXE_claude_version` env macro.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `helpers.rs` | Shared subprocess helpers: `run_cm`, `stdout`, `stderr`, `assert_exit` |
| `framework_test.rs` | Pipeline smoke tests: `.help`, unknown command, exit codes |
| `read_commands_test.rs` | Integration tests for 8 read-only commands |
| `mutation_commands_test.rs` | Integration tests for 4 mutation commands |
| `cross_cutting_test.rs` | Cross-cutting: dry+force interaction, verbosity parity, format parity |
| `error_messages_test.rs` | Error message content and format validation |
