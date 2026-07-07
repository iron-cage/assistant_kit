//! CLI test crate entry point for `claude_version`.
//!
//! Includes all CLI test modules, organised by domain. Tests invoke the compiled binary
//! via subprocess using `CARGO_BIN_EXE_claude_version`.
//!
//! Add new CLI test modules here as new domains or commands are covered.

#[ path = "cli/subprocess_helpers.rs" ]
pub mod subprocess_helpers;

#[ path = "cli/framework_test.rs" ]
mod framework_test;

#[ path = "cli/read_help_test.rs" ]
mod read_help_test;

#[ path = "cli/read_status_test.rs" ]
mod read_status_test;

#[ path = "cli/read_version_test.rs" ]
mod read_version_test;

#[ path = "cli/read_processes_test.rs" ]
mod read_processes_test;

#[ path = "cli/read_settings_test.rs" ]
mod read_settings_test;

#[ path = "cli/read_version_history_test.rs" ]
mod read_version_history_test;

#[ path = "cli/mutation_version_install_test.rs" ]
mod mutation_version_install_test;

#[ path = "cli/mutation_processes_kill_test.rs" ]
mod mutation_processes_kill_test;

#[ path = "cli/mutation_version_guard_test.rs" ]
mod mutation_version_guard_test;

#[ path = "cli/mutation_settings_set_test.rs" ]
mod mutation_settings_set_test;

#[ path = "cli/config_commands_test.rs" ]
mod config_commands_test;

#[ path = "cli/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "cli/error_messages_test.rs" ]
mod error_messages_test;

#[ path = "cli/feature_surface_test.rs" ]
mod feature_surface_test;

#[ path = "cli/algorithm_surface_test.rs" ]
mod algorithm_surface_test;

#[ path = "cli/scope_param_test.rs" ]
mod scope_param_test;

#[ path = "cli/unset_param_test.rs" ]
mod unset_param_test;

#[ path = "cli/config_identity_test.rs" ]
mod config_identity_test;

#[ path = "cli/user_story_test.rs" ]
mod user_story_test;

#[ path = "cli/format_surface_test.rs" ]
mod format_surface_test;

#[ path = "cli/pitfall_surface_test.rs" ]
mod pitfall_surface_test;

#[ path = "cli/catalog_surface_test.rs" ]
mod catalog_surface_test;

#[ path = "cli/version_param_test.rs" ]
mod version_param_test;

#[ path = "cli/dry_param_test.rs" ]
mod dry_param_test;

#[ path = "cli/force_param_test.rs" ]
mod force_param_test;

#[ path = "cli/verbosity_param_test.rs" ]
mod verbosity_param_test;

#[ path = "cli/format_param_test.rs" ]
mod format_param_test;

#[ path = "cli/key_param_test.rs" ]
mod key_param_test;

#[ path = "cli/value_param_test.rs" ]
mod value_param_test;

#[ path = "cli/count_param_test.rs" ]
mod count_param_test;

#[ path = "cli/process_isolation_test.rs" ]
mod process_isolation_test;

#[ path = "cli/params_command_test.rs" ]
mod params_command_test;

#[ path = "cli/kind_param_test.rs" ]
mod kind_param_test;

#[ path = "cli/runtime_files_test.rs" ]
mod runtime_files_test;

#[ path = "cli/paths_test.rs" ]
mod paths_test;

#[ path = "cli/path_key_test.rs" ]
mod path_key_test;
