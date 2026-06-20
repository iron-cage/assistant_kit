//! `--fallback-model` Integration Tests
//!
//! Covers EC-1 through EC-7 from `tests/docs/cli/param/067_fallback_model.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

// ── EC-1: --fallback-model sonnet → forwarded to assembled command ─────────────

/// EC-1: `--fallback-model sonnet` appears in the assembled command.
#[ test ]
fn ec1_fallback_model_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--fallback-model", "sonnet", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--fallback-model" ),
    "assembled command must contain --fallback-model. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "sonnet" ),
    "assembled command must contain the value sonnet. Got:\n{stdout}"
  );
}

// ── EC-2: --fallback-model without value → exit 1 ────────────────────────────

/// EC-2: `--fallback-model` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_fallback_model_missing_value()
{
  let out = run_cli( &[ "--fallback-model" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --fallback-model has no value: {out:?}"
  );
}

// ── EC-3: --fallback-model at end of argv → exit 1 ───────────────────────────

/// EC-3: `--fallback-model` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_fallback_model_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--fallback-model" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --fallback-model appears at end of argv: {out:?}"
  );
}

// ── EC-4: Any model string accepted ───────────────────────────────────────────

/// EC-4: Custom model string `custom-fallback` accepted; no validation rejection.
#[ test ]
fn ec4_fallback_model_any_string_accepted()
{
  let out = run_cli( &[ "--dry-run", "--fallback-model", "custom-fallback", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0 for custom model string: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--fallback-model" ),
    "assembled command must contain --fallback-model. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "custom-fallback" ),
    "assembled command must contain the value custom-fallback. Got:\n{stdout}"
  );
}

// ── EC-5: `--help` lists `--fallback-model` ──────────────────────────────────

/// EC-5: `clr --help` output contains `--fallback-model`.
#[ test ]
fn ec5_fallback_model_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--fallback-model" ),
    "`clr --help` must list --fallback-model. Got:\n{stdout}"
  );
}

// ── EC-6: Without --fallback-model → flag absent ─────────────────────────────

/// EC-6: Without `--fallback-model`, the assembled command does NOT contain `--fallback-model`.
#[ test ]
fn ec6_fallback_model_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--fallback-model" ),
    "assembled command must NOT contain --fallback-model without explicit flag. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_FALLBACK_MODEL=haiku env var → forwarded ───────────────────────

/// EC-7: `CLR_FALLBACK_MODEL=haiku` env var causes `--fallback-model haiku` to appear.
#[ test ]
fn ec7_fallback_model_env_var_forwarded()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_FALLBACK_MODEL", "haiku" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--fallback-model" ),
    "assembled command must contain --fallback-model from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "haiku" ),
    "assembled command must contain the value haiku. Got:\n{stdout}"
  );
}
