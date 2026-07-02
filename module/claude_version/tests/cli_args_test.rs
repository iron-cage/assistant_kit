//! Argument parsing tests via the `claude_version` binary.
//!
//! Tests verify dot-prefixed command parsing, `key::value` parameter parsing,
//! value validation, and all rejection paths through the binary. Test modules
//! are organised by domain (help, parsing, per-parameter).
//!
//! All tests invoke the compiled binary via subprocess using
//! `CARGO_BIN_EXE_claude_version`. Container guard is enforced via
//! `subprocess_helpers::assert_container`.

#[ path = "cli_args_test/subprocess_helpers.rs" ]
pub mod subprocess_helpers;

#[ path = "cli_args_test/help_test.rs" ]
mod help_test;

#[ path = "cli_args_test/parsing_test.rs" ]
mod parsing_test;

#[ path = "cli_args_test/param_verbosity_test.rs" ]
mod param_verbosity_test;

#[ path = "cli_args_test/param_format_test.rs" ]
mod param_format_test;

#[ path = "cli_args_test/param_bool_test.rs" ]
mod param_bool_test;

#[ path = "cli_args_test/param_numeric_test.rs" ]
mod param_numeric_test;

#[ path = "cli_args_test/type_surface_test.rs" ]
mod type_surface_test;
