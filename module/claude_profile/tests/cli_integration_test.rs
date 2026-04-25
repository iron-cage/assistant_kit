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
//! | `account_list_status_test` | help, account list, account status | H, AL, ASTAT |
//! | `account_mutations_test` | account save, switch, delete | AS, AW, AD |
//! | `account_status_name_test` | account status by name | ASTNAME |
//! | `token_paths_test` | token status, paths | TS, P |
//! | `cross_cutting_test` | idempotency, param order, exit codes, env | X, E |
//! | `usage_test` | .usage command | U |
//! | `persist_test` | `PersistPaths` resolution | P |
//! | `credentials_test` | .credentials.status | cred |
//! | `account_limits_test` | .account.limits error paths | lim |
//!
//! ## Parallel Execution Note
//!
//! These tests spawn the `claude_profile` binary as a subprocess. Under heavy
//! workspace-wide parallel nextest execution, process spawning resource contention
//! can cause intermittent failures (e.g. `x06_param_order_independence_token`).
//! If a test fails here while passing in isolation, run with
//! `cargo nextest run -p claude_profile --no-fail-fast` to distinguish contention
//! from a genuine logic regression. The tests themselves are deterministic.

#[ path = "cli/helpers.rs" ]
pub mod helpers;

#[ path = "cli/account_list_status_test.rs" ]
mod account_list_status_test;

#[ path = "cli/account_mutations_test.rs" ]
mod account_mutations_test;

#[ path = "cli/token_paths_test.rs" ]
mod token_paths_test;

#[ path = "cli/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "cli/usage_test.rs" ]
mod usage_test;

#[ path = "cli/persist_test.rs" ]
mod persist_test;

#[ path = "cli/account_status_name_test.rs" ]
mod account_status_name_test;

#[ path = "cli/credentials_test.rs" ]
mod credentials_test;

#[ path = "cli/account_limits_test.rs" ]
mod account_limits_test;
