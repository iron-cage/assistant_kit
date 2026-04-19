//! Integration tests: cred (Credentials Status).
//!
//! Verifies that `.credentials.status` reads live credentials directly with no
//! dependency on the account store (`_active` marker or `accounts/` directory).
//!
//! ## Root Cause Context
//!
//! `.account.status` errors when `_active` is absent — even on machines with valid
//! credentials but no account management initialized. These tests confirm that
//! `.credentials.status` has no such dependency.
//!
//! ## Why Tests Use `TempDir` with No `accounts/`
//!
//! Each test that verifies account-store independence explicitly omits `accounts/`
//! from the temp HOME. This is the anti-fake check: the command succeeds without it.
//!
//! ## Fix Applied
//!
//! Introduced `.credentials.status` command that reads `~/.claude/.credentials.json`
//! and `~/.claude/.claude.json` directly — no `_active`, no `accounts/` scan.
//!
//! ## Prevention
//!
//! Whenever a new "diagnostics" or "inspect" command is added, it must not silently
//! depend on account management state. Use these tests as the template.
//!
//! ## Pitfall
//!
//! Do NOT call `account::list()` or read `_active` in `credentials_status_routine`.
//! Those operations fail on fresh installations. Keep the two domains separated.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | cred01 | `cred01_no_accounts_dir_succeeds` | no accounts/ dir → exit 0 | P |
//! | cred02 | `cred02_v2_all_fields` | v::2 → all credential fields shown | P |
//! | cred03 | `cred03_format_json` | format::json → JSON object | P |
//! | cred04 | `cred04_missing_credentials_file_exits_nonzero` | no .credentials.json → non-zero | N |
//! | cred05 | `cred05_v1_no_claude_json_shows_na` | no .claude.json → N/A for email/org | P |

use crate::helpers::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_claude_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── cred01 ────────────────────────────────────────────────────────────────────

/// cred01: temp HOME with `.credentials.json` only — no `accounts/` dir.
///
/// Confirms account-store independence: command exits 0 and shows sub + token.
#[ test ]
fn cred01_no_accounts_dir_succeeds()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Intentionally do NOT create accounts/ or _active
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "pro" ),
    "output must contain subscription type, got:\n{text}",
  );
  assert!(
    text.to_lowercase().contains( "valid" ) || text.to_lowercase().contains( "expir" ),
    "output must contain token state, got:\n{text}",
  );
}

// ── cred02 ────────────────────────────────────────────────────────────────────

/// cred02: `v::2` with both `.credentials.json` and `.claude.json`.
///
/// Confirms all fields are shown: sub, tier, token, expiry, email, org.
#[ test ]
fn cred02_v2_all_fields()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "user@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".credentials.status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "pro" ),          "sub must appear, got:\n{text}" );
  assert!( text.contains( "standard" ),     "tier must appear, got:\n{text}" );
  assert!( text.contains( "user@example.com" ), "email must appear, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ),    "org must appear, got:\n{text}" );
  assert!(
    text.contains( "Expires" ) || text.contains( "expires" ),
    "expiry line must appear at v::2, got:\n{text}",
  );
}

// ── cred03 ────────────────────────────────────────────────────────────────────

/// cred03: `format::json` — output must be parseable JSON with required fields.
#[ test ]
fn cred03_format_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert!( text.starts_with( '{' ) && text.ends_with( '}' ), "output must be JSON object, got:\n{text}" );
  assert!( text.contains( "\"subscription\"" ), "JSON must have subscription field, got:\n{text}" );
  assert!( text.contains( "\"tier\"" ),         "JSON must have tier field, got:\n{text}" );
  assert!( text.contains( "\"token\"" ),         "JSON must have token field, got:\n{text}" );
}

// ── cred04 ────────────────────────────────────────────────────────────────────

/// cred04: no `.credentials.json` — must exit non-zero with actionable error.
#[ test ]
fn cred04_missing_credentials_file_exits_nonzero()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create .claude dir but NO .credentials.json
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code != 0, "must exit non-zero when .credentials.json absent, got code {code}" );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "credential" ),
    "error must reference the credential file, got:\n{err}",
  );
}

// ── cred05 ────────────────────────────────────────────────────────────────────

/// cred05: `v::1` with no `.claude.json` — email and org must show N/A.
#[ test ]
fn cred05_v1_no_claude_json_shows_na()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only write credentials — intentionally omit .claude.json
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "v::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // N/A must appear twice: once for Email, once for Org
  let na_count = text.matches( "N/A" ).count();
  assert!(
    na_count >= 2,
    "v::1 without .claude.json must show N/A for email and org (found {na_count} N/A), got:\n{text}",
  );
}
