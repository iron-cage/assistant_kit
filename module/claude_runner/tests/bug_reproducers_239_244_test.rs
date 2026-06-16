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
//! | (none — add spawn format test)         | BUG-298 | [Runner] prefix absent from spawn error output | No |
//! | `signal_sigterm_exits_143`             | BUG-242 | `unwrap_or(1)` collapsed | No (Unix) |
//! | `signal_sigkill_exits_137`             | BUG-242 | `unwrap_or(1)` collapsed | No (Unix) |
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
//! # Root Cause (BUG-244)
//!
//! `--subdir` and `CLR_SUBDIR` were added to `claude_runner` CLI parsing and env-var
//! application but the `claude_storage` mirror was not updated in the same session.
//! The storage mirror diverged: it accepted the old arg surface, missing `--subdir` entirely.
//!
//! # Why Not Caught (BUG-244)
//!
//! No cross-crate parity test existed to verify that every arg present in `claude_runner`
//! is also handled in `claude_storage`. The feature gap was only discovered when a
//! dedicated `--subdir` regression test was added directly to the binary test suite.
//!
//! # Fix Applied (BUG-244)
//!
//! Synced `--subdir` flag parsing and `CLR_SUBDIR` env var application to the
//! `claude_storage` mirror, restoring full parity with the `claude_runner` CLI surface.
//!
//! # Prevention (BUG-244)
//!
//! When adding a new CLI arg to `claude_runner`, always update the `claude_storage`
//! mirror in the same session. Run `storage_subdir_flag_accepted` and
//! `storage_subdir_env_var_applied` to confirm parity before closing the task.
//!
//! # Pitfall (BUG-244)
//!
//! `claude_storage` mirrors the `claude_runner` CLI surface — any new arg must be added
//! to both in the same session. A lagging mirror compiles and passes existing tests
//! while silently missing new features, making the gap hard to detect.

#![ cfg( feature = "enabled" ) ]
#![ cfg( unix ) ]   // signal tests are Unix-only

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, fake_claude_dir, run_cli_with_env, stderr_str };

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
  let out = run_cli_with_env( &[ "--print", "--retry-override", "0", "test" ], &[ ( "PATH", &path_val ) ] );
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
  let out = run_cli_with_env( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
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
  let out = run_cli_with_env(
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
  let out = run_cli_with_env( &[ "--print", "test" ], &[ ( "PATH", "/tmp" ) ] );
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
  let out = run_cli_with_env( &[ "--print", "--retry-override", "0", "test" ], &[ ( "PATH", &path_val ) ] );
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
  let out = run_cli_with_env( &[ "--print", "--retry-override", "0", "test" ], &[ ( "PATH", &path_val ) ] );
  assert_eq!(
    exit_code( &out ),
    137,
    "BUG-242: SIGKILL (signal 9) must produce exit code 137 (128+9); got {}\nstderr: {}",
    exit_code( &out ),
    stderr_str( &out ),
  );
}

// ── BUG-244 ──────────────────────────────────────────────────────────────────

/// BUG-244 regression T8: `--subdir foo` accepted without error via clr binary.
///
/// Verifies that the subdir feature (added after the storage mirror diverged)
/// is present in the synced copy.
#[ test ]
fn storage_subdir_flag_accepted()
{
  let out = run_cli_with_env( &[ "--subdir", "foo", "--dry-run", "test" ], &[] );
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
  let out = run_cli_with_env( &[ "--dry-run", "test" ], &[ ( "CLR_SUBDIR", "foo" ) ] );
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
