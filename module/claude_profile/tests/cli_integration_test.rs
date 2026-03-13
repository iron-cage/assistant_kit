//! Integration test crate entry point for `claude_profile`.
//!
//! Includes all integration test modules. Tests invoke the compiled binary
//! via subprocess using `CARGO_BIN_EXE_clp`.
//!
//! ## Parallel Execution Note
//!
//! These tests spawn the `claude_profile` binary as a subprocess. Under heavy
//! workspace-wide parallel nextest execution, process spawning resource contention
//! can cause intermittent failures (e.g. `x06_param_order_independence_token`).
//! If a test fails here while passing in isolation, run with
//! `cargo nextest run -p claude_profile --no-fail-fast` to distinguish contention
//! from a genuine logic regression. The tests themselves are deterministic.

#[ path = "integration/helpers.rs" ]
pub mod helpers;

#[ path = "integration/account_list_status_test.rs" ]
mod account_list_status_test;

#[ path = "integration/account_mutations_test.rs" ]
mod account_mutations_test;

#[ path = "integration/token_paths_test.rs" ]
mod token_paths_test;

#[ path = "integration/cross_cutting_test.rs" ]
mod cross_cutting_test;

#[ path = "integration/usage_test.rs" ]
mod usage_test;

#[ path = "integration/persist_test.rs" ]
mod persist_test;

#[ path = "integration/account_status_name_test.rs" ]
mod account_status_name_test;

#[ path = "integration/credentials_test.rs" ]
mod credentials_test;

#[ path = "integration/account_limits_test.rs" ]
mod account_limits_test;
