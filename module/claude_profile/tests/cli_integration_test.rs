//! Integration test crate entry point for `claude_profile`.
//!
//! Includes all integration test modules. Tests invoke the compiled binary
//! via subprocess using `CARGO_BIN_EXE_clp`.
//!
//! ## Coverage Summary
//!
//! This file wires together all CLI integration submodules. Each module covers
//! one functional domain of the `clp` binary:
//!
//! | Module | Domain | Test Series |
//! |--------|--------|-------------|
//! | `accounts_test` | help, `.accounts` command | H, ACC |
//! | `account_mutations_test` | account save, use, delete, relogin | AS, AW, AD, AR |
//! | `token_paths_test` | token status, paths | TS, P |
//! | `cross_cutting_test` | idempotency, param order, exit codes, env | X, E |
//! | `usage_test` | .usage live quota (all accounts) | IT |
//! | `usage_feature_test` | .usage feature AC coverage | FT |
//! | `persist_test` | `PersistPaths` resolution | P |
//! | `credentials_test` | .credentials.status | cred |
//! | `credentials_status_help_test` | .credentials.status.help descriptions | csh |
//! | `account_limits_test` | .account.limits error paths | lim |
//! | `account_rotate_test` | .account.rotate auto-rotation | ROT |
//! | `dot_test` | `.` / `.help` help output | dot |
//! | `param_help_test` | convenience closure param descriptions + optionality | phd, pho |
//! | `account_inspect_test` | .account.inspect diagnostic command | AI |
//! | `account_assign_test` | .account.assign marker-only write | AA |
//! | `set_model_test` | `set_model::` explicit session model override | FT, EC |
//! | `model_test` | `.model` get/set command (Feature 035) | FT |
//! | `type_test` | CLI type boundary contracts (`AccountName`, `OutputFormat`, `WarningThreshold`, `AccountSelector`) | TC |
//! | `invariant_test` | Architectural invariant assertions (zero deps, cross-platform, clear errors, atomic, etc.) | IN |
//! | `command_verb_test` | Command-verb behavioral contracts (save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status) | BV |
//! | `command_noun_test` | Command-noun contracts (account, token, credentials) | NC |
//! | `user_story_test` | User acceptance tests — account rotation, onboarding, quota monitoring, automation, diagnostics | UA |
//!
//! ## Parallel Execution Note
//!
//! These tests spawn the `claude_profile` binary as a subprocess. Under heavy
//! workspace-wide parallel nextest execution, process spawning resource contention
//! can cause intermittent failures (e.g. `x06_param_order_independence_token`).
//! If a test fails here while passing in isolation, run with
//! `cargo nextest run -p claude_profile --no-fail-fast` to distinguish contention
//! from a genuine logic regression. The tests themselves are deterministic.

#[ path = "cli/cli_runner.rs" ]
pub mod cli_runner;

#[ path = "cli/accounts_test.rs" ]
mod accounts_test;

#[ path = "cli/account_mutations_test.rs" ]
mod account_mutations_test;

#[ path = "cli/token_paths_test.rs" ]
mod token_paths_test;

#[ path = "cli/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "cli/usage_test.rs" ]
mod usage_test;

#[ path = "cli/usage_feature_test.rs" ]
mod usage_feature_test;

#[ path = "cli/persist_test.rs" ]
mod persist_test;

#[ path = "cli/credentials_test.rs" ]
mod credentials_test;

#[ path = "cli/credentials_status_help_test.rs" ]
mod credentials_status_help_test;

#[ path = "cli/account_limits_test.rs" ]
mod account_limits_test;

#[ path = "cli/account_rotate_test.rs" ]
mod account_rotate_test;

#[ path = "cli/dot_test.rs" ]
mod dot_test;

#[ path = "cli/param_help_test.rs" ]
mod param_help_test;

#[ path = "cli/account_inspect_test.rs" ]
mod account_inspect_test;

#[ path = "cli/account_assign_test.rs" ]
mod account_assign_test;

#[ path = "cli/set_model_test.rs" ]
mod set_model_test;

#[ path = "cli/model_test.rs" ]
mod model_test;

#[ path = "cli/type_test.rs" ]
mod type_test;

#[ path = "cli/invariant_test.rs" ]
mod invariant_test;

#[ path = "cli/command_verb_test.rs" ]
mod command_verb_test;

#[ path = "cli/command_noun_test.rs" ]
mod command_noun_test;

#[ path = "cli/user_story_test.rs" ]
mod user_story_test;
