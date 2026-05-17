//! Shared test helpers for `claude_runner` integration tests.
//!
//! # Test Matrix
//!
//! | Helper | Used By |
//! |--------|---------|
//! | `run_cli` | `cli_args_test`, `dry_run_test`, `ultrathink_args_test`, `effort_args_test` |
//! | `run_cli_with_env` | `env_var_test` |

use std::process::Command;

/// Invoke the `clr` binary with `args`, returning raw `Output` without asserting success.
///
/// Used for both success-path and expected-failure cases — callers check
/// `output.status` or inspect `output.stdout`/`output.stderr` directly.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched (process spawn failure).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_cli( args : &[ &str ] ) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .output()
    .expect( "Failed to invoke clr binary" )
}

/// Invoke the `clr` binary with `args` and extra environment variables, returning raw `Output`.
///
/// Env vars are injected via `Command::envs()` — no process-global `std::env::set_var`.
/// Safe for concurrent test execution; each subprocess sees only the injected env.
///
/// # Panics
///
/// Panics if the `clr` binary cannot be launched (process spawn failure).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn run_cli_with_env
(
  args : &[ &str ],
  env  : &[ ( &str, &str ) ],
) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .envs( env.iter().copied() )
    .output()
    .expect( "failed to execute clr binary" )
}
