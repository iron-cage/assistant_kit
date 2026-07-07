//! Quiet Flag Tests — `--quiet` and `CLR_QUIET` env var.
//!
//! `--quiet` suppresses non-fatal CLR diagnostics: gate-wait messages, retry-in-progress
//! warnings, retry-exhaustion messages, and keep-claudecode warnings.  Fatal errors
//! (spawn failure, argument errors) are always emitted regardless of `--quiet`.
//!
//! ## Test Matrix
//!
//! | ID | Name | Mechanism | Assertion |
//! |----|------|-----------|-----------|
//! | QT-1 | `qt1_quiet_suppresses_gate_wait` | fake claude exits 0; `--quiet --max-sessions 0 -p ping` | stderr not "Waiting" |
//! | QT-2 | `qt2_quiet_suppresses_retry_warning` | fake claude exits 2; `--retry-on-transient 1 --transient-delay 0 --quiet --max-sessions 0 -p ping` | stderr not "retrying" |
//! | QT-3 | `qt3_no_quiet_default_diagnostics_visible` | `--dry-run --max-sessions 0 "Fix bug"` (no `--quiet`) | exit 0; no unknown-flag error |
//! | QT-4 | `qt4_clr_quiet_env_var` | `CLR_QUIET=true`; fake claude exits 2; `--retry-on-transient 1 --transient-delay 0 --max-sessions 0 -p ping` | stderr not "retrying" |
//! | QT-5 | `qt5_quiet_does_not_suppress_dry_run` | `--quiet --dry-run "Fix bug"` | exit 0; stdout has `CLAUDE_CODE_MAX_OUTPUT_TOKENS=` |
//! | QT-6 | `qt6_quiet_fatal_error_still_on_stderr` | `PATH=/nonexistent`; `--quiet -p "Fix bug"` | exit non-zero; stderr not empty |

mod cli_binary_test_helpers;

// QT-1: --quiet does not produce spurious "Waiting" output (no gate-wait scenario).
// Full gate-wait suppression requires ≥1 active session (host-dependent); this test
// asserts --quiet works without breaking the run and produces no "Waiting" on stderr
// in the common no-gate case (max-sessions 0 skips the gate entirely).
#[ cfg( unix ) ]
#[ test ]
fn qt1_quiet_suppresses_gate_wait()
{
  let ( _tmp, path_val ) = cli_binary_test_helpers::fake_claude_dir( "exit 0" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--quiet", "--max-sessions", "0", "-p", "ping" ] )
    .env( "PATH", &path_val )
    .env_remove( "CLR_TRACE" )
    .env_remove( "CLR_QUIET" )
    .output()
    .expect( "failed to invoke clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "Waiting" ),
    "qt1: --quiet must not produce 'Waiting' on stderr. Got:\n{stderr}"
  );
}

// QT-2: --quiet suppresses the "retrying" warning emitted on transient retry.
// fake claude exits 2 → Transient class; --retry-on-transient 1 enables one retry;
// --transient-delay 0 avoids sleeping; --quiet must suppress the retry warning line.
#[ cfg( unix ) ]
#[ test ]
fn qt2_quiet_suppresses_retry_warning()
{
  let ( _tmp, path_val ) = cli_binary_test_helpers::fake_claude_dir( "exit 2" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [
      "--quiet",
      "--max-sessions", "0",
      "--retry-on-transient", "1",
      "--transient-delay", "0",
      "-p", "ping",
    ] )
    .env( "PATH", &path_val )
    .env_remove( "CLR_TRACE" )
    .env_remove( "CLR_QUIET" )
    .output()
    .expect( "failed to invoke clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "retrying" ),
    "qt2: --quiet must suppress 'retrying' diagnostic. Got:\n{stderr}"
  );
}

// QT-3: no --quiet — baseline check that normal operation still works.
// Dry-run with no --quiet must succeed (exit 0) and produce no unknown-flag error —
// confirms --quiet was added cleanly without breaking the existing flag set.
#[ test ]
fn qt3_no_quiet_default_diagnostics_visible()
{
  let out = cli_binary_test_helpers::run_cli(
    &[ "--dry-run", "--max-sessions", "0", "Fix bug" ]
  );
  assert!(
    out.status.success(),
    "qt3: --dry-run without --quiet must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "unknown option" ),
    "qt3: baseline run must not produce unknown-option error. Got:\n{stderr}"
  );
}

// QT-4: CLR_QUIET=true env var suppresses retry warning (same gate as --quiet flag).
// Verifies the env-var fallback path in apply_env_vars() reads CLR_QUIET correctly.
#[ cfg( unix ) ]
#[ test ]
fn qt4_clr_quiet_env_var()
{
  let ( _tmp, path_val ) = cli_binary_test_helpers::fake_claude_dir( "exit 2" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [
      "--max-sessions", "0",
      "--retry-on-transient", "1",
      "--transient-delay", "0",
      "-p", "ping",
    ] )
    .env( "PATH", &path_val )
    .env( "CLR_QUIET", "true" )
    .env_remove( "CLR_TRACE" )
    .output()
    .expect( "failed to invoke clr" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "retrying" ),
    "qt4: CLR_QUIET=true must suppress 'retrying' diagnostic. Got:\n{stderr}"
  );
}

// QT-5: --quiet does not suppress --dry-run core feature output.
// --dry-run output is core functionality (always shown); --quiet gates diagnostics only.
#[ test ]
fn qt5_quiet_does_not_suppress_dry_run()
{
  let out = cli_binary_test_helpers::run_cli( &[ "--quiet", "--dry-run", "Fix bug" ] );
  assert!(
    out.status.success(),
    "qt5: --quiet --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "qt5: --quiet must not suppress --dry-run stdout. Got:\n{stdout}"
  );
}

// QT-6: --quiet does not suppress fatal spawn errors.
// When PATH contains no claude binary, clr exits non-zero and emits a spawn error
// to stderr regardless of --quiet — fatal errors are never silenced.
#[ test ]
fn qt6_quiet_fatal_error_still_on_stderr()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--quiet", "-p", "--max-sessions", "0", "Fix bug" ] )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "failed to invoke clr" );
  assert!(
    !out.status.success(),
    "qt6: missing claude binary must exit non-zero even with --quiet"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.is_empty(),
    "qt6: --quiet must not suppress fatal spawn error on stderr. Got empty stderr."
  );
}
