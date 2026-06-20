//! `--disallowed-tools` Integration Tests
//!
//! Covers EC-1 through EC-7 from `tests/docs/cli/param/064_disallowed_tools.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

// ── EC-1: --disallowed-tools "Bash" → forwarded to assembled command ──────────

/// EC-1: `--disallowed-tools Bash` appears in the assembled command.
#[ test ]
fn ec1_disallowed_tools_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--disallowed-tools", "Bash", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--disallowed-tools" ),
    "assembled command must contain --disallowed-tools. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Bash" ),
    "assembled command must contain the value Bash. Got:\n{stdout}"
  );
}

// ── EC-2: --disallowed-tools without value → exit 1 ──────────────────────────

/// EC-2: `--disallowed-tools` without a value → exit 1 with a missing-value error.
#[ test ]
fn ec2_disallowed_tools_missing_value()
{
  let out = run_cli( &[ "--disallowed-tools" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --disallowed-tools has no value: {out:?}"
  );
}

// ── EC-3: --disallowed-tools at end of argv → exit 1 ─────────────────────────

/// EC-3: `--disallowed-tools` at end of argv → exit 1 (missing value).
#[ test ]
fn ec3_disallowed_tools_at_end_of_argv()
{
  let out = run_cli( &[ "Fix bug", "--disallowed-tools" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 when --disallowed-tools appears at end of argv: {out:?}"
  );
}

// ── EC-4: Any tool string accepted ────────────────────────────────────────────

/// EC-4: Multiple tool list `Write,Edit` accepted; no rejection.
#[ test ]
fn ec4_disallowed_tools_any_string_accepted()
{
  let out = run_cli( &[ "--dry-run", "--disallowed-tools", "Write,Edit", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0 for multi-tool value: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--disallowed-tools" ),
    "assembled command must contain --disallowed-tools. Got:\n{stdout}"
  );
}

// ── EC-5: `--help` lists `--disallowed-tools` ─────────────────────────────────

/// EC-5: `clr --help` output contains `--disallowed-tools`.
#[ test ]
fn ec5_disallowed_tools_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--disallowed-tools" ),
    "`clr --help` must list --disallowed-tools. Got:\n{stdout}"
  );
}

// ── EC-6: Without --disallowed-tools → flag absent ────────────────────────────

/// EC-6: Without `--disallowed-tools`, the assembled command does NOT contain `--disallowed-tools`.
#[ test ]
fn ec6_disallowed_tools_absent_by_default()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--disallowed-tools" ),
    "assembled command must NOT contain --disallowed-tools without explicit flag. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_DISALLOWED_TOOLS env var → forwarded ────────────────────────────

/// EC-7: `CLR_DISALLOWED_TOOLS=Bash` env var causes `--disallowed-tools Bash` to appear.
#[ test ]
fn ec7_disallowed_tools_env_var_forwarded()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_DISALLOWED_TOOLS", "Bash" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--disallowed-tools" ),
    "assembled command must contain --disallowed-tools from env var. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Bash" ),
    "assembled command must contain the value Bash. Got:\n{stdout}"
  );
}
