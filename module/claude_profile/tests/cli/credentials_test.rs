//! Integration tests: cred (Credentials Status).
//!
//! Verifies that `.credentials.status` reads live credentials directly with no
//! dependency on the account store (`_active` marker or credential store).
//!
//! ## Root Cause Context
//!
//! Account-inspection commands require an `_active` marker — even on machines with valid
//! credentials but no account management initialized. These tests confirm that
//! `.credentials.status` has no such dependency.
//!
//! ## Why Tests Use `TempDir` with No Credential Store
//!
//! Each test that verifies account-store independence explicitly omits the credential
//! store from the temp HOME. This is the anti-fake check: the command succeeds without it.
//!
//! ## Fix Applied
//!
//! Introduced `.credentials.status` command that reads `~/.claude/.credentials.json`
//! and `~/.claude/.claude.json` directly — no `_active`, no credential store scan.
//!
//! ## Prevention
//!
//! Whenever a new "diagnostics" or "inspect" command is added, it must not silently
//! depend on account management state. Use these tests as the template.
//!
//! ## Pitfall
//!
//! Do NOT call `account::list()` in `credentials_status_routine`.
//! Reading `_active` is permitted for the `account::` line only; it is read
//! opportunistically and the command succeeds with `Account: N/A` when absent.
//! Fresh installations (no credential store) must still succeed.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | cred01 | `cred01_no_credential_store_succeeds` | no credential store → exit 0, Account: N/A | P |
//! | cred02 | `cred02_default_with_claude_json` | default → all 7 default-on fields shown | P |
//! | cred03 | `cred03_format_json` | format::json → JSON object with all 9 fields | P |
//! | cred04 | `cred04_missing_credentials_file_exits_nonzero` | no .credentials.json → non-zero | N |
//! | cred05 | `cred05_no_claude_json_shows_na` | no .claude.json → N/A for email/org/account | P |
//! | cred06 | `cred06_suppress_all_default_on` | all default-on suppressed → Token line only | P |
//! | cred07 | `cred07_opt_in_file_and_saved` | file::1 saved::1 → File and Saved lines | P |

use crate::helpers::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_claude_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── cred01 ────────────────────────────────────────────────────────────────────

/// cred01: temp HOME with `.credentials.json` only — no credential store.
///
/// Confirms account-store independence: command exits 0 and shows sub + token.
#[ test ]
fn cred01_no_credential_store_succeeds()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Intentionally do NOT create credential store or _active
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
  assert!( text.contains( "Account:" ), "Account: line must appear, got:\n{text}" );
  assert!( text.contains( "N/A" ),      "Account: N/A must appear (no _active marker), got:\n{text}" );
}

// ── cred02 ────────────────────────────────────────────────────────────────────

/// cred02: default output with both `.credentials.json` and `.claude.json`.
///
/// Confirms all 7 default-on fields shown: account, sub, tier, token, expires, email, org.
#[ test ]
fn cred02_default_with_claude_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "user@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Account:" ),         "Account: line must appear, got:\n{text}" );
  assert!( text.contains( "pro" ),              "sub must appear, got:\n{text}" );
  assert!( text.contains( "standard" ),         "tier must appear, got:\n{text}" );
  assert!( text.contains( "user@example.com" ), "email must appear, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ),        "org must appear, got:\n{text}" );
  assert!(
    text.contains( "Expires" ) || text.contains( "expires" ),
    "Expires: line must appear in default output, got:\n{text}",
  );
}

// ── cred03 ────────────────────────────────────────────────────────────────────

/// cred03: `format::json` — output must be parseable JSON with all 9 required fields.
#[ test ]
fn cred03_format_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "user@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert!( text.starts_with( '{' ) && text.ends_with( '}' ), "output must be JSON object, got:\n{text}" );
  assert!( text.contains( "\"subscription\"" ),  "JSON must have subscription field, got:\n{text}" );
  assert!( text.contains( "\"tier\"" ),          "JSON must have tier field, got:\n{text}" );
  assert!( text.contains( "\"token\"" ),         "JSON must have token field, got:\n{text}" );
  assert!( text.contains( "\"expires_in_secs\"" ), "JSON must have expires_in_secs field, got:\n{text}" );
  assert!( text.contains( "\"email\"" ),         "JSON must have email field, got:\n{text}" );
  assert!( text.contains( "\"org\"" ),           "JSON must have org field, got:\n{text}" );
  assert!( text.contains( "\"account\"" ),       "JSON must have account field, got:\n{text}" );
  assert!( text.contains( "\"file\"" ),          "JSON must have file field, got:\n{text}" );
  assert!( text.contains( "\"saved\"" ),         "JSON must have saved field, got:\n{text}" );
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

/// cred05: default output with no `.claude.json` and no `_active` — email, org, account show N/A.
#[ test ]
fn cred05_no_claude_json_shows_na()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only write credentials — intentionally omit .claude.json and credential store
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // N/A must appear at least 3 times: Account:, Email:, Org:
  let na_count = text.matches( "N/A" ).count();
  assert!(
    na_count >= 3,
    "default output without .claude.json and no _active must show N/A for account, email, org \
     (found {na_count} N/A), got:\n{text}",
  );
}

// ── cred06 ────────────────────────────────────────────────────────────────────

/// cred06: suppress all default-on fields except token — only Token: line in output.
///
/// Confirms per-field boolean control: setting account/sub/tier/expires/email/org to 0
/// leaves only the Token: line in stdout.
#[ test ]
fn cred06_suppress_all_default_on()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".credentials.status", "account::0", "sub::0", "tier::0", "expires::0", "email::0", "org::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.to_lowercase().contains( "valid" ) || text.to_lowercase().contains( "expir" ),
    "token state must be present, got:\n{text}",
  );
  assert!( !text.contains( "Sub:" ),     "Sub: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Tier:" ),    "Tier: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Expires:" ), "Expires: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Email:" ),   "Email: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Org:" ),     "Org: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Account:" ), "Account: must be suppressed, got:\n{text}" );
}

// ── cred07 ────────────────────────────────────────────────────────────────────

/// cred07: opt-in `file::1 saved::1` — File: and Saved: lines appended after default-on fields.
///
/// Confirms that opt-in field-presence flags add the File: and Saved: lines to output.
#[ test ]
fn cred07_opt_in_file_and_saved()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".credentials.status", "file::1", "saved::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "File:" ),  "File: line must appear, got:\n{text}" );
  assert!( text.contains( "Saved:" ), "Saved: line must appear, got:\n{text}" );
  assert!(
    text.contains( ".credentials.json" ),
    "File: line must contain credentials path, got:\n{text}",
  );
}
