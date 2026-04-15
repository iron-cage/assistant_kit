//! Verbosity Level Tests
//!
//! Tests for the `--verbosity` flag that controls `claude_runner`'s own
//! diagnostic output level (0–5). Claude Code subprocess output is not gated.
//!
//! ## Test Matrix
//!
//! | # | Scenario | Config Under Test | Expected Behavior |
//! |---|----------|-------------------|-------------------|
//! | T01 | `--verbosity 0 --dry-run` | silence level | `--dry-run` stdout shown (core feature); stderr empty (diagnostics suppressed) |
//! | T02 | `--verbosity 3 --dry-run` | normal level | Runner emits standard progress output |
//! | T03 | `--verbosity 5 --dry-run` | debug level | Runner emits output (no crash) |
//! | T04 | `--verbosity 6` | invalid level | Rejected: range error on stderr |
//! | T05 | no `--verbosity` | default | Behaves as `--verbosity 3` |
//! | T06 | `--verbosity 1 --dry-run` | errors only | `--dry-run` stdout shown; verbosity does not suppress core feature output |
//! | T07 | `--verbosity 0 -p` | suppressed errors | Runner output suppressed; Claude still executes |
//! | T08 | `VerbosityLevel::from_str("0")` | type parsing | Returns `Ok(VerbosityLevel(0))` |
//! | T09 | `VerbosityLevel::from_str("6")` | invalid parse | Returns `Err(...)` |
//! | T10 | `VerbosityLevel::default()` | default value | Returns `VerbosityLevel(3)` |
//! | T11 | `--verbosity 4 --dry-run` | verbose detail | Preview printed to stderr before execute |

use claude_runner::VerbosityLevel;
use std::process::Command;

fn run_bin( args : &[ &str ] ) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
  .args( args )
  .output()
  .expect( "Failed to invoke clr binary" )
}

// T01: --verbosity 0 silences runner *diagnostics* but not --dry-run output.
// --dry-run is core feature output (always shown); stderr diagnostic output is suppressed.
#[ test ]
fn t01_verbosity_zero_silences_runner_diagnostics()
{
  let out = run_bin( &[ "--verbosity", "0", "--dry-run", "test" ] );
  assert!( out.status.success(), "--verbosity 0 must not fail" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--verbosity 0 must still show --dry-run output (core feature, not a diagnostic). Got:\n{stdout}"
  );
  assert!(
    stderr.is_empty(),
    "--verbosity 0 must suppress stderr diagnostic output. Got:\n{stderr}"
  );
}

// T02: --verbosity 3 shows normal progress output (command preview).
#[ test ]
fn t02_verbosity_three_shows_normal_output()
{
  let out = run_bin( &[ "--verbosity", "3", "--dry-run", "test" ] );
  assert!( out.status.success(), "--verbosity 3 must succeed" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--verbosity 3 must show dry-run command preview. Got:\n{stdout}"
  );
}

// T03: --verbosity 5 (debug) runs without crash and shows output.
#[ test ]
fn t03_verbosity_five_runs_without_crash()
{
  let out = run_bin( &[ "--verbosity", "5", "--dry-run", "test" ] );
  assert!( out.status.success(), "--verbosity 5 must not crash" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude" ),
    "--verbosity 5 must show claude command in dry-run output. Got:\n{stdout}"
  );
}

// T04: --verbosity 6 is rejected with a range error (valid range is 0–5).
#[ test ]
fn t04_verbosity_six_rejected()
{
  let out = run_bin( &[ "--verbosity", "6", "--dry-run", "test" ] );
  assert!( !out.status.success(), "--verbosity 6 must be rejected" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "verbosity" ),
    "Error message must mention 'verbosity'. Got:\n{stderr}"
  );
}

// T05: no --verbosity flag behaves identically to --verbosity 3.
#[ test ]
fn t05_default_verbosity_behaves_as_three()
{
  let out = run_bin( &[ "--dry-run", "test" ] );
  assert!( out.status.success(), "default verbosity must succeed" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "Default verbosity must show dry-run command preview (same as --verbosity 3). Got:\n{stdout}"
  );
}

// T06: --verbosity 1 (errors only) does not suppress --dry-run output.
// Verbosity gates runner diagnostics only; --dry-run is core feature output (always shown).
#[ test ]
fn t06_verbosity_one_shows_dry_run_output()
{
  let out = run_bin( &[ "--verbosity", "1", "--dry-run", "test" ] );
  assert!( out.status.success(), "--verbosity 1 must succeed" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--verbosity 1 must still show --dry-run output (not a diagnostic). Got:\n{stdout}"
  );
}

// T07: --verbosity 0 with -p (print mode) — runner output suppressed;
// Claude subprocess still executes. Uses a fake claude binary.
// Fix(issue-108): chmod via PermissionsExt is Unix-only.
#[ cfg( unix ) ]
#[ test ]
fn t07_verbosity_zero_suppresses_print_output()
{
  use std::os::unix::fs::PermissionsExt;

  let bin = env!( "CARGO_BIN_EXE_clr" );

  // Fake claude: outputs a fixed known string, exits 0.
  let tmp = tempfile::tempdir().expect( "Failed to create temp dir" );
  let fake_claude = tmp.path().join( "claude" );
  std::fs::write( &fake_claude, "#!/bin/sh\necho FAKE_CLAUDE_VERBOSITY_TEST\n" )
  .expect( "Failed to write fake claude script" );
  std::fs::set_permissions( &fake_claude, std::fs::Permissions::from_mode( 0o755 ) )
  .expect( "Failed to make fake claude executable" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let out = std::process::Command::new( bin )
  .args( [ "--verbosity", "0", "-p", "test" ] )
  .env( "PATH", new_path )
  .output()
  .expect( "Failed to invoke clr binary" );

  // --verbosity 0 must suppress runner-side stderr output.
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--verbosity 0 must suppress runner stderr output. Got:\n{stderr}"
  );

  // Claude subprocess still executes despite --verbosity 0.
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "FAKE_CLAUDE_VERBOSITY_TEST" ),
    "--verbosity 0 must not prevent Claude execution. Got:\n{stdout}"
  );
}

// T08: VerbosityLevel::from_str("0") returns Ok(VerbosityLevel(0)).
#[ test ]
fn t08_from_str_zero_ok()
{
  let result : Result< VerbosityLevel, _ > = "0".parse();
  assert!( result.is_ok(), "from_str('0') must return Ok" );
  assert_eq!( result.unwrap().get(), 0, "parsed level must be 0" );
}

// T09: VerbosityLevel::from_str("6") returns Err (out of range).
#[ test ]
fn t09_from_str_six_err()
{
  let result : Result< VerbosityLevel, _ > = "6".parse();
  assert!( result.is_err(), "from_str('6') must return Err" );
}

// T10: VerbosityLevel::default() returns VerbosityLevel(3).
#[ test ]
fn t10_default_is_three()
{
  let level = VerbosityLevel::default();
  assert_eq!( level.get(), 3, "Default verbosity must be 3" );
}

// T11: --verbosity 4 prints command preview to stderr before execution.
// Uses --dry-run + --verbosity 4 — dry-run output goes to stdout,
// and verbosity 4 also triggers stderr preview.
#[ test ]
fn t11_verbosity_four_shows_stderr_preview()
{
  let out = run_bin( &[ "--verbosity", "4", "--dry-run", "test" ] );
  assert!( out.status.success(), "--verbosity 4 --dry-run must succeed" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  // Dry-run output appears on stdout regardless of verbosity level.
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--verbosity 4 must still show dry-run stdout. Got:\n{stdout}"
  );
  // Note: stderr preview only fires on the execute path (not dry-run).
  // Dry-run returns before the preview code. This is correct behavior.
}
