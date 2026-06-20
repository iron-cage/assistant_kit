//! `--max-turns` Integration Tests
//!
//! Covers EC-1 through EC-7 from `tests/docs/cli/param/062_max_turns.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

// ── EC-1: --max-turns 5 → forwarded to assembled command ─────────────────────

/// EC-1: `--max-turns 5` appears in the assembled command.
#[ test ]
fn ec1_max_turns_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--max-turns", "5", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-turns" ),
    "assembled command must contain --max-turns. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( '5' ),
    "assembled command must contain the value 5. Got:\n{stdout}"
  );
}

// ── EC-2: --max-turns without value → exit 1 ─────────────────────────────────

/// EC-2: `--max-turns` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_max_turns_missing_value()
{
  let out = run_cli( &[ "--max-turns" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --max-turns has no value: {out:?}"
  );
}

// ── EC-3: --max-turns at end of argv → exit 1 ────────────────────────────────

/// EC-3: `--max-turns` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_max_turns_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--max-turns" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --max-turns appears at end of argv: {out:?}"
  );
}

// ── EC-4: Any numeric string accepted ────────────────────────────────────────

/// EC-4: Large value 999 accepted; no rejection.
#[ test ]
fn ec4_max_turns_any_numeric_accepted()
{
  let out = run_cli( &[ "--dry-run", "--max-turns", "999", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0 for large value: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-turns" ),
    "assembled command must contain --max-turns. Got:\n{stdout}"
  );
}

// ── EC-5: `--help` lists `--max-turns` ───────────────────────────────────────

/// EC-5: `clr --help` output contains `--max-turns`.
#[ test ]
fn ec5_max_turns_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-turns" ),
    "`clr --help` must list --max-turns. Got:\n{stdout}"
  );
}

// ── EC-6: Without --max-turns → flag absent ──────────────────────────────────

/// EC-6: Without `--max-turns`, the assembled command does NOT contain `--max-turns`.
#[ test ]
fn ec6_max_turns_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--max-turns" ),
    "assembled command must NOT contain --max-turns without explicit flag. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_MAX_TURNS=10 env var → forwarded ───────────────────────────────

/// EC-7: `CLR_MAX_TURNS=10` env var causes `--max-turns 10` to appear in the assembled command.
#[ test ]
fn ec7_max_turns_env_var_forwarded()
{
  let out = run_cli_with_env( &[ "--dry-run", "Fix bug" ], &[ ( "CLR_MAX_TURNS", "10" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-turns" ),
    "assembled command must contain --max-turns from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "10" ),
    "assembled command must contain the value 10. Got:\n{stdout}"
  );
}
