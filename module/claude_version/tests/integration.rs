//! Integration test crate entry point for `claude_version`.
//!
//! Includes all integration test modules. Tests invoke the compiled binary
//! via subprocess using `CARGO_BIN_EXE_claude_version`.
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

#[ path = "integration/config_commands_test.rs" ]
mod config_commands_test;

#[ path = "integration/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "integration/error_messages_test.rs" ]
mod error_messages_test;

#[ path = "integration/feature_surface_test.rs" ]
mod feature_surface_test;

#[ path = "integration/algorithm_surface_test.rs" ]
mod algorithm_surface_test;

#[ path = "integration/scope_param_test.rs" ]
mod scope_param_test;

#[ path = "integration/unset_param_test.rs" ]
mod unset_param_test;

#[ path = "integration/config_identity_test.rs" ]
mod config_identity_test;

#[ path = "integration/user_story_test.rs" ]
mod user_story_test;

#[ path = "integration/format_surface_test.rs" ]
mod format_surface_test;

#[ path = "integration/pitfall_surface_test.rs" ]
mod pitfall_surface_test;

#[ path = "integration/collection_surface_test.rs" ]
mod collection_surface_test;
