//! Shared test helpers for `claude_runner` integration tests.
//!
//! # Test Matrix
//!
//! | Helper | Used By |
//! |--------|---------|
//! | `run_cli` | `cli_args_test`, `cli_args_ext_test`, `dry_run_test`, `ultrathink_args_test`, `effort_args_test`, `param_edge_cases_test`, `param_extended_flags_test`, `param_group_test`, `execution_mode_test`, `verbosity_test`, `ask_command_test`, `user_story_test`, `user_story_creds_isolated_test`, `user_story_output_test` |
//! | `run_cli_with_env` | `env_var_test`, `env_var_ext_test`, `invariant_trace_universality_test`, `param_trace_edge_cases_test`, `param_group_test`, `isolated_test`, `user_story_creds_isolated_test`, `user_story_output_test` |
//! | `make_session_dir` | `cli_args_test`, `ultrathink_args_test`, `user_story_test` |
//! | `exit_code` | `refresh_test`, `bug_reproducers_239_244_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_test` |
//! | `stderr_str` | `refresh_test`, `bug_reproducers_239_244_test`, `invariant_trace_universality_test`, `error_classification_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_correctness_test`, `isolated_test` |
//! | `stdout_str` | `refresh_test`, `isolated_correctness_test`, `isolated_test` |
//! | `make_creds_file` | `refresh_test`, `param_trace_edge_cases_test`, `invariant_trace_universality_test`, `user_story_test`, `user_story_creds_isolated_test`, `isolated_correctness_test`, `isolated_test` |
//! | `fake_claude_dir` (unix) | `bug_reproducers_239_244_test`, `error_classification_test` |
//!
//! # Testing Techniques
//!
//! - **`--dry-run`**: Inspect assembled command without spawning Claude subprocess.
//! - **`--trace` (for `isolated`/`refresh`)**: These commands lack `--dry-run`;
//!   use `--trace` to verify the assembled command on stderr.
//! - **`PATH=/nonexistent`**: Force binary-not-found for deterministic failure
//!   testing — trace output fires before subprocess invocation attempt.
//! - **`make_session_dir`**: Create a non-empty temp session dir so `session_exists()`
//!   returns `true` regardless of the ambient host environment.  Tests that assert
//!   `-c` injection must use `--session-dir <path>` with this helper; otherwise they
//!   are fragile and fail in clean container environments with no prior Claude sessions.

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

/// Create a temp session directory with one dummy file; returns `(dir, path_string)`.
///
/// The caller must keep the returned `TempDir` alive for the duration of the test —
/// the directory and its contents are deleted when the `TempDir` is dropped.
/// Pass the returned `path_string` as the value of `--session-dir` to force
/// `session_exists()` to return `true`, making `-c` injection deterministic
/// regardless of the ambient host session state.
///
/// Pitfall: if the caller drops `TempDir` before passing the path to the subprocess,
/// the directory is deleted and `session_exists()` returns `false`.
///
/// # Panics
///
/// Panics if the temp directory or the dummy file cannot be created.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn make_session_dir() -> ( tempfile::TempDir, String )
{
  let dir = tempfile::TempDir::new().expect( "failed to create temp session dir" );
  std::fs::write( dir.path().join( "session.json" ), b"{}" )
    .expect( "failed to write dummy session file" );
  let path = dir.path().to_str().expect( "session dir path must be valid UTF-8" ).to_owned();
  ( dir, path )
}

/// Extract the process exit code from a subprocess `Output`.
///
/// Returns `-1` when the process was terminated by a signal (no numeric exit code).
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn exit_code( o : &std::process::Output ) -> i32 { o.status.code().unwrap_or( -1 ) }

/// Extract `stderr` as a UTF-8 string from a subprocess `Output`.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn stderr_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stderr ).to_string()
}

/// Extract `stdout` as a UTF-8 string from a subprocess `Output`.
#[must_use]
#[inline]
#[allow(dead_code)]
pub fn stdout_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stdout ).to_string()
}

/// Write `content` to a new `NamedTempFile` and return it.
///
/// The caller must keep the returned file alive for the duration of the test;
/// dropping it deletes the file on disk.
///
/// # Panics
///
/// Panics if the temp file cannot be created or written.
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn make_creds_file( content : &str ) -> tempfile::NamedTempFile
{
  use std::io::Write as _;
  let mut f = tempfile::NamedTempFile::new().expect( "failed to create temp creds file" );
  f.write_all( content.as_bytes() ).expect( "failed to write creds content" );
  f
}

/// Create a temp dir containing a `claude` shell script with the given body.
///
/// Returns `(TempDir, path_val)` where `path_val` prepends the dir to `$PATH`
/// for injection into subprocess env.  The caller must keep the `TempDir` alive
/// for the duration of the test; dropping it deletes the script.
///
/// # Panics
///
/// Panics if the temp directory, script file, or permissions cannot be set.
#[cfg(unix)]
#[inline]
#[must_use]
#[allow(dead_code)]
pub fn fake_claude_dir( body : &str ) -> ( tempfile::TempDir, String )
{
  use std::os::unix::fs::PermissionsExt as _;
  let dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let path = dir.path().join( "claude" );
  let script = format!( "#!/bin/sh\n{body}\n" );
  std::fs::write( &path, script.as_bytes() ).expect( "write fake-claude" );
  std::fs::set_permissions( &path, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake-claude" );
  let path_val = format!(
    "{}:{}",
    dir.path().display(),
    std::env::var( "PATH" ).unwrap_or_default(),
  );
  ( dir, path_val )
}
