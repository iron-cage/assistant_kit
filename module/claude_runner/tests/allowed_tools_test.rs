//! `--allowed-tools` Integration Tests
//!
//! Covers EC-1 through EC-7 from `tests/docs/cli/param/063_allowed_tools.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

// ── EC-1: --allowed-tools "Read,Edit" → forwarded to assembled command ────────

/// EC-1: `--allowed-tools Read,Edit` appears in the assembled command; hyphen-form flag.
#[ test ]
fn ec1_allowed_tools_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--allowed-tools", "Read,Edit", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--allowed-tools" ),
    "assembled command must contain --allowed-tools. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Read,Edit" ),
    "assembled command must contain the value Read,Edit (comma-preserved). Got:\n{stdout}"
  );
}

// ── EC-2: --allowed-tools without value → exit 1 ─────────────────────────────

/// EC-2: `--allowed-tools` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_allowed_tools_missing_value()
{
  let out = run_cli( &[ "--allowed-tools" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --allowed-tools has no value: {out:?}"
  );
}

// ── EC-3: --allowed-tools at end of argv → exit 1 ────────────────────────────

/// EC-3: `--allowed-tools` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_allowed_tools_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--allowed-tools" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --allowed-tools appears at end of argv: {out:?}"
  );
}

// ── EC-4: Any tool string accepted ────────────────────────────────────────────

/// EC-4: Complex tool pattern `Bash(git:*),Read` accepted; no rejection.
#[ test ]
fn ec4_allowed_tools_any_string_accepted()
{
  let out = run_cli( &[ "--dry-run", "--allowed-tools", "Bash(git:*),Read", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0 for complex tool pattern: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--allowed-tools" ),
    "assembled command must contain --allowed-tools. Got:\n{stdout}"
  );
}

// ── EC-5: `--help` lists `--allowed-tools` ────────────────────────────────────

/// EC-5: `clr --help` output contains `--allowed-tools`.
#[ test ]
fn ec5_allowed_tools_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--allowed-tools" ),
    "`clr --help` must list --allowed-tools. Got:\n{stdout}"
  );
}

// ── EC-6: Without --allowed-tools → flag absent ───────────────────────────────

/// EC-6: Without `--allowed-tools`, the assembled command does NOT contain `--allowed-tools`.
#[ test ]
fn ec6_allowed_tools_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--allowed-tools" ),
    "assembled command must NOT contain --allowed-tools without explicit flag. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_ALLOWED_TOOLS env var → forwarded ───────────────────────────────

/// EC-7: `CLR_ALLOWED_TOOLS=Read,Edit` env var causes `--allowed-tools Read,Edit` to appear.
#[ test ]
fn ec7_allowed_tools_env_var_forwarded()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_ALLOWED_TOOLS", "Read,Edit" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--allowed-tools" ),
    "assembled command must contain --allowed-tools from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Read,Edit" ),
    "assembled command must contain the value Read,Edit. Got:\n{stdout}"
  );
}
