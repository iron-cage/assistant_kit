//! Command handlers: one function per `claude_profile` CLI command.
//!
//! Each handler receives a `VerifiedCommand` and `ExecutionContext` and returns
//! `Result<OutputData, ErrorData>`. Handlers are registered via
//! [`register_commands()`](crate::register_commands) in `lib.rs`;
//! the binary-specific `.` handler is registered inline in `build_registry()` in `lib.rs`.
//!
//! # Note on `needless_pass_by_value`
//!
//! Every handler takes `VerifiedCommand` by value because the `CommandRoutine`
//! type alias requires ownership.

pub( crate ) mod shared;
mod credentials;
mod accounts;
mod account_ops;
mod limits;
mod token_paths;
mod dot;

pub use credentials::credentials_status_routine;
pub use accounts::accounts_routine;
pub use account_ops::{
  account_use_routine,
  account_rotate_routine,
  account_save_routine,
  account_delete_routine,
  account_relogin_routine,
  account_renewal_routine,
};
pub use limits::account_limits_routine;
pub use token_paths::{ token_status_routine, paths_routine };
pub use dot::dot_routine;
pub use crate::usage::usage_routine;
