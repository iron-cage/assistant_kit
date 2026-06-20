//! `--add-dir` Integration Tests
//!
//! Covers EC-1 through EC-7 from `tests/docs/cli/param/066_add_dir.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

// ── EC-1: --add-dir /tmp → forwarded to assembled command ─────────────────────

/// EC-1: `--add-dir /tmp` appears in the assembled command.
#[ test ]
fn ec1_add_dir_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--add-dir", "/tmp", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--add-dir" ),
    "assembled command must contain --add-dir. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/tmp" ),
    "assembled command must contain the value /tmp. Got:\n{stdout}"
  );
}

// ── EC-2: --add-dir without value → exit 1 ───────────────────────────────────

/// EC-2: `--add-dir` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_add_dir_missing_value()
{
  let out = run_cli( &[ "--add-dir" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --add-dir has no value: {out:?}"
  );
}

// ── EC-3: --add-dir at end of argv → exit 1 ──────────────────────────────────

/// EC-3: `--add-dir` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_add_dir_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--add-dir" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --add-dir appears at end of argv: {out:?}"
  );
}

// ── EC-4: Any path string accepted ────────────────────────────────────────────

/// EC-4: Non-existent path `/nonexistent/path` accepted; no existence validation.
#[ test ]
fn ec4_add_dir_any_path_accepted()
{
  let out = run_cli( &[ "--dry-run", "--add-dir", "/nonexistent/path", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0 for non-existent path: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--add-dir" ),
    "assembled command must contain --add-dir. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/nonexistent/path" ),
    "assembled command must contain the value /nonexistent/path. Got:\n{stdout}"
  );
}

// ── EC-5: `--help` lists `--add-dir` ─────────────────────────────────────────

/// EC-5: `clr --help` output contains `--add-dir`.
#[ test ]
fn ec5_add_dir_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--add-dir" ),
    "`clr --help` must list --add-dir. Got:\n{stdout}"
  );
}

// ── EC-6: Without --add-dir → flag absent ────────────────────────────────────

/// EC-6: Without `--add-dir`, the assembled command does NOT contain `--add-dir`.
#[ test ]
fn ec6_add_dir_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--add-dir" ),
    "assembled command must NOT contain --add-dir without explicit flag. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_ADD_DIR=/tmp env var → forwarded ───────────────────────────────

/// EC-7: `CLR_ADD_DIR=/tmp` env var causes `--add-dir /tmp` to appear.
#[ test ]
fn ec7_add_dir_env_var_forwarded()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_ADD_DIR", "/tmp" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--add-dir" ),
    "assembled command must contain --add-dir from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/tmp" ),
    "assembled command must contain the value /tmp. Got:\n{stdout}"
  );
}
