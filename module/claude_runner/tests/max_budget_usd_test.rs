//! `--max-budget-usd` Integration Tests
//!
//! Covers EC-1 through EC-7 from `tests/docs/cli/param/065_max_budget_usd.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

// ── EC-1: --max-budget-usd 5.00 → forwarded with exact string preserved ───────

/// EC-1: `--max-budget-usd 5.00` appears in the assembled command; decimal preserved.
#[ test ]
fn ec1_max_budget_usd_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--max-budget-usd", "5.00", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-budget-usd" ),
    "assembled command must contain --max-budget-usd. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "5.00" ),
    "assembled command must contain the value 5.00 (decimal preserved). Got:\n{stdout}"
  );
}

// ── EC-2: --max-budget-usd without value → exit 1 ────────────────────────────

/// EC-2: `--max-budget-usd` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_max_budget_usd_missing_value()
{
  let out = run_cli( &[ "--max-budget-usd" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --max-budget-usd has no value: {out:?}"
  );
}

// ── EC-3: --max-budget-usd at end of argv → exit 1 ───────────────────────────

/// EC-3: `--max-budget-usd` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_max_budget_usd_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--max-budget-usd" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --max-budget-usd appears at end of argv: {out:?}"
  );
}

// ── EC-4: Any numeric string accepted ────────────────────────────────────────

/// EC-4: Small value `0.01` accepted; no validation rejection.
#[ test ]
fn ec4_max_budget_usd_any_numeric_accepted()
{
  let out = run_cli( &[ "--dry-run", "--max-budget-usd", "0.01", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0 for small value: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-budget-usd" ),
    "assembled command must contain --max-budget-usd. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "0.01" ),
    "assembled command must contain the value 0.01. Got:\n{stdout}"
  );
}

// ── EC-5: `--help` lists `--max-budget-usd` ──────────────────────────────────

/// EC-5: `clr --help` output contains `--max-budget-usd`.
#[ test ]
fn ec5_max_budget_usd_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-budget-usd" ),
    "`clr --help` must list --max-budget-usd. Got:\n{stdout}"
  );
}

// ── EC-6: Without --max-budget-usd → flag absent ─────────────────────────────

/// EC-6: Without `--max-budget-usd`, the assembled command does NOT contain `--max-budget-usd`.
#[ test ]
fn ec6_max_budget_usd_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--max-budget-usd" ),
    "assembled command must NOT contain --max-budget-usd without explicit flag. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_MAX_BUDGET_USD=1.50 env var → forwarded ────────────────────────

/// EC-7: `CLR_MAX_BUDGET_USD=1.50` env var causes `--max-budget-usd 1.50` to appear.
#[ test ]
fn ec7_max_budget_usd_env_var_forwarded()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_MAX_BUDGET_USD", "1.50" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--max-budget-usd" ),
    "assembled command must contain --max-budget-usd from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "1.50" ),
    "assembled command must contain the value 1.50. Got:\n{stdout}"
  );
}
