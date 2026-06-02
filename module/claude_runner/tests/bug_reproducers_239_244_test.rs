//! Bug reproducers for BUG-239 through BUG-244.
//!
//! Each test is a minimal reproducible example (MRE) that demonstrates the
//! failure mode before the fix and passes after. Tests use real subprocess
//! execution with fake-`claude` shell scripts — no mocking.
//!
//! # Test Matrix
//!
//! | Test | BUG | Root Cause | Requires Live Claude |
//! |------|-----|------------|----------------------|
//! | `print_mode_propagates_exit_42`        | BUG-239 | `exit(1)` hardcoded | No |
//! | `print_mode_propagates_exit_0`         | BUG-239 | `exit(1)` hardcoded | No |
//! | `spawn_error_visible_at_verbosity_0`   | BUG-240 | Err gated on verbosity | No |
//! | `binary_not_found_shows_install_hint`  | BUG-241 | Raw OS error emitted | No |
//! | `signal_sigterm_exits_143`             | BUG-242 | `unwrap_or(1)` collapsed | No (Unix) |
//! | `signal_sigkill_exits_137`             | BUG-242 | `unwrap_or(1)` collapsed | No (Unix) |
//! | `timeout_includes_partial_stdout`      | BUG-243 | thread lost Child handle | No |
//! | `storage_subdir_flag_accepted`         | BUG-244 | storage mirror behind | No |
//! | `storage_subdir_env_var_applied`       | BUG-244 | storage mirror behind | No |
//!
//! # Root Cause (BUG-239)
//!
//! `run_print_mode` in `src/cli/mod.rs` called `std::process::exit(1)` unconditionally
//! when `output.exit_code != 0`, collapsing every non-zero subprocess exit code to 1.
//!
//! # Why Not Caught (BUG-239)
//!
//! All pre-existing tests only asserted `exit_code == 0` (success path) or `exit_code != 0`
//! (failure path). No test asserted `exit_code == specific_value` for a non-zero case.
//!
//! # Fix Applied (BUG-239)
//!
//! `std::process::exit(1)` replaced with `std::process::exit(output.exit_code)` so the
//! subprocess exit code is forwarded exactly.
//!
//! # Prevention (BUG-239)
//!
//! Test both a specific non-zero exit code (e.g. 42) and zero when adding print-mode tests.
//!
//! # Pitfall (BUG-239)
//!
//! Never substitute a generic exit code where the subprocess's code is available. Any
//! hardcoded `exit(1)` after a subprocess wait is a silent-failure risk.
//!
//! ---
//!
//! # Root Cause (BUG-240)
//!
//! The `Err(e)` spawn-failure branch in `run_print_mode` and `run_interactive` was guarded
//! by `if verbosity.shows_errors()` — at `CLR_VERBOSITY=0` the error was swallowed.
//!
//! # Why Not Caught (BUG-240)
//!
//! All verbosity tests ran with the default verbosity (3). No test combined
//! `CLR_VERBOSITY=0` with a binary-not-found scenario.
//!
//! # Fix Applied (BUG-240)
//!
//! Removed the `if verbosity.shows_errors()` guard from both `run_print_mode` and
//! `run_interactive`. Fatal errors are always emitted regardless of verbosity.
//!
//! # Prevention (BUG-240)
//!
//! For every code path that calls `eprintln!` on a fatal error, test it with verbosity=0.
//!
//! # Pitfall (BUG-240)
//!
//! Verbosity gates runner diagnostics (progress, trace output), never fatal errors. A
//! fatal error that the user can't see is worse than no verbosity filtering at all.
//!
//! ---
//!
//! # Root Cause (BUG-241)
//!
//! `execute()` and `execute_interactive()` in `command/mod.rs` used a generic
//! `format!("Failed to execute Claude Code: {e}")` for all spawn errors, including
//! `io::ErrorKind::NotFound` — providing no install guidance.
//!
//! # Why Not Caught (BUG-241)
//!
//! Tests always ran with `claude` in PATH. No test asserted on the content of the
//! error message for the binary-not-found case.
//!
//! # Fix Applied (BUG-241)
//!
//! Added a `NotFound` branch in both `execute()` and `execute_interactive()` that emits
//! `"claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code"`.
//!
//! # Prevention (BUG-241)
//!
//! Test binary-not-found with `PATH=/tmp` and assert the error message contains both
//! `"not found"` and `"install"`.
//!
//! # Pitfall (BUG-241)
//!
//! Binary-not-found is the most common user error when deploying to a new environment.
//! Never let it surface as a raw OS error string.
//!
//! ---
//!
//! # Root Cause (BUG-242)
//!
//! Exit-code sites used `status.code().unwrap_or(1)` or `.unwrap_or(-1)`. On Unix,
//! `code()` returns `None` when a signal killed the process; the fallback collapsed
//! SIGTERM (→143) and SIGKILL (→137) to 1 or -1.
//!
//! # Why Not Caught (BUG-242)
//!
//! No test ever verified the exact exit code produced when a subprocess is killed by
//! a specific signal.
//!
//! # Fix Applied (BUG-242)
//!
//! Added `signal_exit_code(&status)` in `claude_runner_core/src/exit_code.rs` that
//! follows the POSIX `128 + signal_number` convention on Unix. Replaced all `unwrap_or`
//! call sites with this helper.
//!
//! # Prevention (BUG-242)
//!
//! Always use `signal_exit_code()` instead of `status.code().unwrap_or(N)` at any
//! site that propagates a subprocess exit code to the caller.
//!
//! # Pitfall (BUG-242)
//!
//! `#[cfg(unix)]` is required for any code that calls `ExitStatusExt::signal()` —
//! that trait is Unix-only. The non-unix fallback returns `status.code().unwrap_or(1)`.
//!
//! ---
//!
//! # Root Cause (BUG-243)
//!
//! `run_isolated()` spawned a thread that called `cmd.execute()` (which calls
//! `cmd.output()`, blocking until EOF). The main thread used `recv_timeout` to impose
//! a deadline. When the deadline fired, the thread kept the `Child` handle; no kill or
//! partial-output collection was possible — all buffered stdout was irrecoverably dropped.
//!
//! # Why Not Caught (BUG-243)
//!
//! All timeout tests (IT-3, IT-4) used `timeout=0` and asserted on the error type, not
//! on the content of the error message. No test verified that partial stdout was preserved.
//!
//! # Fix Applied (BUG-243)
//!
//! Restructured `run_isolated()` to use `spawn_piped()` (new method on `ClaudeCommand`)
//! + `try_wait` polling. On timeout: `child.kill()` then `child.wait_with_output()`
//!   collects buffered data. Added `RunnerError::TimeoutWithOutput { secs, partial_stdout }`.
//!
//! # Prevention (BUG-243)
//!
//! When you need timeout+kill+output: always keep the `Child` handle in scope through the
//! timeout. Thread-based approaches that move `Child` into the thread lose this ability.
//!
//! # Pitfall (BUG-243)
//!
//! `child.wait_with_output()` waits for stdout/stderr pipes to close (which happens after
//! kill), then returns whatever was buffered. Call it AFTER `child.kill()`, not before.

#![ cfg( feature = "enabled" ) ]
#![ cfg( unix ) ]   // signal tests are Unix-only

use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

mod cli_binary_test_helpers;

// ── helpers ──────────────────────────────────────────────────────────────────

fn clr_bin() -> &'static str { env!( "CARGO_BIN_EXE_clr" ) }

fn exit_code( o : &std::process::Output ) -> i32 { o.status.code().unwrap_or( -1 ) }
fn stderr_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stderr ).to_string()
}

/// Create a temp dir containing a `claude` script with the given body.
/// Returns the `TempDir` (keep alive) and the PATH string to inject.
fn fake_claude_dir( body : &str ) -> ( TempDir, String )
{
  let dir = TempDir::new().expect( "tmpdir" );
  let path = dir.path().join( "claude" );
  let script = format!( "#!/bin/sh\n{body}\n" );
  std::fs::write( &path, script.as_bytes() ).expect( "write fake-claude" );
  std::fs::set_permissions( &path, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake-claude" );
  let path_val = format!( "{}:{}", dir.path().display(), std::env::var( "PATH" ).unwrap_or_default() );
  ( dir, path_val )
}

/// Run `clr` with given args and env overrides, return raw Output.
fn run_clr( args : &[ &str ], env : &[ ( &str, &str ) ] ) -> std::process::Output
{
  std::process::Command::new( clr_bin() )
    .args( args )
    .envs( env.iter().copied() )
    .output()
    .expect( "failed to invoke clr" )
}

// ── BUG-239 ──────────────────────────────────────────────────────────────────

/// BUG-239 reproducer T1: fake-claude exits 42 → clr must exit 42, not 1.
///
/// Before fix: `run_print_mode` called `std::process::exit(1)` unconditionally;
/// exit code from subprocess was discarded.
#[ test ]
#[ doc = "bug_reproducer(BUG-239)" ]
fn print_mode_propagates_exit_42()
{
  let ( _dir, path_val ) = fake_claude_dir( "exit 42" );
  let out = run_clr( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  assert_eq!(
    exit_code( &out ),
    42,
    "BUG-239: clr --print must propagate subprocess exit 42; got {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
}

/// BUG-239 reproducer T2: fake-claude exits 0 → clr must exit 0.
#[ test ]
#[ doc = "bug_reproducer(BUG-239)" ]
fn print_mode_propagates_exit_0()
{
  let ( _dir, path_val ) = fake_claude_dir( "exit 0" );
  let out = run_clr( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  assert_eq!(
    exit_code( &out ),
    0,
    "BUG-239: clr --print must propagate subprocess exit 0; got {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
}

// ── BUG-240 ──────────────────────────────────────────────────────────────────

/// BUG-240 reproducer T3: `CLR_VERBOSITY=0` + binary absent → error must appear on stderr.
///
/// Before fix: `Err(e)` branch was inside `if verbosity.shows_errors()` — at verbosity 0
/// the error was swallowed; stderr was empty while clr still exited 1.
#[ test ]
#[ doc = "bug_reproducer(BUG-240)" ]
fn spawn_error_visible_at_verbosity_0()
{
  let out = run_clr(
    &[ "--print", "test" ],
    &[ ( "PATH", "/tmp" ), ( "CLR_VERBOSITY", "0" ) ],
  );
  assert_ne!( exit_code( &out ), 0, "BUG-240: must exit non-zero when binary absent" );
  let err = stderr_str( &out );
  assert!(
    !err.is_empty(),
    "BUG-240: stderr must contain an error message at CLR_VERBOSITY=0; got empty stderr"
  );
}

// ── BUG-241 ──────────────────────────────────────────────────────────────────

/// BUG-241 reproducer T4: PATH=/tmp → stderr must contain "not found" AND "install".
///
/// Before fix: `execute()` emitted `"Failed to execute Claude Code: {raw_os_error}"` —
/// the OS error string on Linux is "No such file or directory (os error 2)" with no
/// install guidance.
#[ test ]
#[ doc = "bug_reproducer(BUG-241)" ]
fn binary_not_found_shows_install_hint()
{
  let out = run_clr( &[ "--print", "test" ], &[ ( "PATH", "/tmp" ) ] );
  assert_ne!( exit_code( &out ), 0, "BUG-241: must exit non-zero when binary absent" );
  let err = stderr_str( &out );
  assert!(
    err.contains( "not found" ),
    "BUG-241: stderr must contain 'not found'; got:\n{err}"
  );
  assert!(
    err.contains( "install" ),
    "BUG-241: stderr must contain 'install'; got:\n{err}"
  );
}

// ── BUG-242 ──────────────────────────────────────────────────────────────────

/// BUG-242 reproducer T5: subprocess killed by SIGTERM → clr exits 143 (128+15).
///
/// Before fix: `signal_exit_code()` did not exist; `status.code().unwrap_or(1)` or
/// `unwrap_or(-1)` collapsed SIGTERM to 1/-1. After fix: `signal_exit_code(&status)`
/// computes `128 + 15 = 143`.
#[ test ]
#[ doc = "bug_reproducer(BUG-242)" ]
fn signal_sigterm_exits_143()
{
  // `kill -TERM $$` sends SIGTERM to the shell itself; it dies by the signal so the
  // parent sees WIFSIGNALED=true, WTERMSIG=15 → signal_exit_code returns 143.
  let ( _dir, path_val ) = fake_claude_dir( "kill -TERM $$" );
  let out = run_clr( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  assert_eq!(
    exit_code( &out ),
    143,
    "BUG-242: SIGTERM (signal 15) must produce exit code 143 (128+15); got {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
}

/// BUG-242 reproducer T6: subprocess killed by SIGKILL → clr exits 137 (128+9).
#[ test ]
#[ doc = "bug_reproducer(BUG-242)" ]
fn signal_sigkill_exits_137()
{
  let ( _dir, path_val ) = fake_claude_dir( "kill -KILL $$" );
  let out = run_clr( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  assert_eq!(
    exit_code( &out ),
    137,
    "BUG-242: SIGKILL (signal 9) must produce exit code 137 (128+9); got {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
}

// ── BUG-243 ──────────────────────────────────────────────────────────────────

/// BUG-243 reproducer T7: timeout fires after partial stdout → `TimeoutWithOutput` has content.
///
/// Before fix: the thread/channel approach left `Child` inside the spawned thread;
/// `recv_timeout` fired and all buffered stdout was irrecoverably discarded — the error
/// variant `Timeout { secs }` carried no output.
/// After fix: `spawn_piped()` + `try_wait` polling keeps `Child` in scope; on timeout
/// `child.kill()` + `child.wait_with_output()` recovers buffered data.
#[ test ]
#[ doc = "bug_reproducer(BUG-243)" ]
#[ allow( unsafe_code ) ]
fn timeout_includes_partial_stdout()
{
  use claude_runner_core::{ run_isolated, RunnerError, IsolatedModel };

  // Fake claude: print a marker then sleep forever.
  let ( _dir, path_val ) = fake_claude_dir( "printf 'partial-output-marker'; sleep 999" );

  // Minimal credentials JSON.
  let creds_json = r#"{"accessToken":"fake","refreshToken":"fake","expiresAt":9999999999999}"#;

  // Set PATH in the environment for the subprocess spawned by run_isolated.
  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // run_isolated inherits the current process env; temporarily extend PATH.
  // We use a subprocess (clr binary path) rather than run_isolated directly
  // because run_isolated reads PATH from the process environment at spawn time.
  // Restore PATH after the test to avoid interfering with other tests.
  // SAFETY: single-threaded test binary; no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let result = run_isolated( creds_json, vec![], 1, IsolatedModel::KeepCurrent );
  // SAFETY: restoring PATH to the original value; single-threaded test binary.
  unsafe { std::env::set_var( "PATH", &orig_path ); }

  match result
  {
    Err( RunnerError::TimeoutWithOutput { secs : _, partial_stdout } ) =>
    {
      assert!(
        partial_stdout.contains( "partial-output-marker" ),
        "BUG-243: TimeoutWithOutput.partial_stdout must contain the marker; got:\n{partial_stdout}"
      );
    }
    Err( RunnerError::Timeout { .. } ) =>
    {
      panic!( "BUG-243: expected TimeoutWithOutput (with content), got Timeout (empty)" );
    }
    other =>
    {
      panic!( "BUG-243: expected TimeoutWithOutput error; got: {other:?}" );
    }
  }
}

// ── BUG-244 ──────────────────────────────────────────────────────────────────

/// BUG-244 regression T8: `--subdir foo` accepted without error via clr binary.
///
/// Verifies that the subdir feature (added after the storage mirror diverged)
/// is present in the synced copy.
#[ test ]
fn storage_subdir_flag_accepted()
{
  let out = run_clr( &[ "--subdir", "foo", "--dry-run", "test" ], &[] );
  assert_eq!(
    exit_code( &out ),
    0,
    "BUG-244: --subdir foo must be accepted; got exit {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
}

/// BUG-244 regression T9: `CLR_SUBDIR=foo` env var applied correctly.
#[ test ]
fn storage_subdir_env_var_applied()
{
  let out = run_clr( &[ "--dry-run", "test" ], &[ ( "CLR_SUBDIR", "foo" ) ] );
  assert_eq!(
    exit_code( &out ),
    0,
    "BUG-244: CLR_SUBDIR=foo must be applied; got exit {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout ).to_string();
  assert!(
    stdout.contains( "foo" ),
    "BUG-244: CLR_SUBDIR=foo must appear in dry-run output; got:\n{stdout}"
  );
}
