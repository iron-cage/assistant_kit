//! Integration test crate entry point for `claude_manager`.
//!
//! Includes all integration test modules. Tests invoke the compiled binary
//! via subprocess using `CARGO_BIN_EXE_claude_manager`.
//!
//! Add new integration test modules here as phases progress.

#[ path = "integration/helpers.rs" ]
pub mod helpers;

#[ path = "integration/framework_test.rs" ]
mod framework_test;

#[ path = "integration/read_commands_test.rs" ]
mod read_commands_test;

#[ path = "integration/mutation_commands_test.rs" ]
mod mutation_commands_test;

#[ path = "integration/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "integration/error_messages_test.rs" ]
mod error_messages_test;
