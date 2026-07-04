# tests/cli_args_test/

Module files for argument parsing tests, included by `tests/cli_args_test.rs`
(the test crate entry point). Tests invoke the `claude_version` binary via
`std::process::Command` using the `CARGO_BIN_EXE_claude_version` env macro.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `subprocess_helpers.rs` | Container guard, binary runner, and output extractors |
| `help_test.rs` | `.help` command and anywhere-in-argv routing tests (TC-001/002/026/038-040/489-490, EC-3..EC-8) |
| `parsing_test.rs` | Command recognition, param syntax enforcement, unknown param/command rejection |
| `param_verbosity_test.rs` | `v::` / `verbosity::` parameter: range, type, last-wins, canonical-key parity |
| `param_format_test.rs` | `format::` parameter: empty, wrong-case, last-wins, default-absent |
| `param_bool_test.rs` | `dry::` / `force::` parameters: acceptance, non-0/1 rejection, last-wins |
| `param_numeric_test.rs` | `count::` / `interval::` / `version::` parameters: overflow, semver format |
| `type_surface_test.rs` | Type contract tests for VerbosityLevel, OutputFormat, VersionSpec, SettingsKey, SettingsValue |
