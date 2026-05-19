//! Feature tests: FT (Usage — AC coverage for `009_token_usage`).
//!
//! Each FT case maps to one acceptance criterion from
//! `docs/feature/009_token_usage.md`. Command-level tests (IT-N) live in
//! `tests/cli/usage_test.rs`.
//!
//! Live tests (names contain `lim_it`) require network access and are excluded
//! from Docker CI by the nextest filter `!test(lim_it)`.
//!
//! ## Test Matrix
//!
//! | ID   | Test Function                                    | AC    | Live? |
//! |------|--------------------------------------------------|-------|-------|
//! | ft01 | `ft01_missing_access_token_short_error`          | AC-03 | no    |
//! | ft02 | `ft02_lim_it_http_401_shortens_to_auth_expired`  | AC-03 | yes   |
//! | ft03 | `ft03_both_accounts_appear_regardless_of_active` | AC-01 | no    |
//! | ft04 | `ft04_check_mark_follows_live_token_not_active`  | AC-02 | no    |
//! | ft05 | `ft05_unreadable_credential_store_exits_2`       | AC-06 | no    |

use crate::helpers::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token,
  write_live_credentials_with_token,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── FT-01: Error reason shortened — missing accessToken (AC-03) ──────────────

/// FT-01 (AC-03): account whose credential file has no `accessToken` field →
/// the account row appears in the table; the error reason in the last column
/// does NOT begin with the verbose `HTTP transport error:` prefix. Exit 0.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-01`.
#[ test ]
fn ft01_missing_access_token_short_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "noaccess@test.com", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "noaccess@test.com" ),
    "account row must appear in the table, got:\n{text}",
  );
  assert!(
    !text.contains( "HTTP transport error:" ),
    "error must be shortened — must NOT contain verbose 'HTTP transport error:', got:\n{text}",
  );
}

// ── FT-02: HTTP 401 shortens to (auth expired (401)) (AC-03) ─────────────────

/// Write a saved account credential with `PAST_MS` expiry AND an `accessToken`
/// so `read_token()` succeeds but the usage API rejects the token with 401.
fn write_account_with_expired_token( home : &std::path::Path, name : &str, token : &str )
{
  let store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  let json = format!(
    r#"{{"oauthAccount":{{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}},"expiresAt":{PAST_MS},"accessToken":"{token}"}}"#,
  );
  std::fs::write( store.join( format!( "{name}.credentials.json" ) ), json ).unwrap();
}

/// FT-02 (AC-03, `lim_it`): saved account has `PAST_MS` `expiresAt` and a token
/// the usage API rejects with HTTP 401 → rendered row shows `EXPIRED` in the
/// Expires column and `auth expired (401)` in the 7d Reset column, NOT the
/// verbose `HTTP transport error: HTTP 401` string. Exit 0.
///
/// Requires network access — the fake token triggers a real API 401 response.
/// Source: `tests/docs/feature/09_token_usage.md § FT-02`.
#[ test ]
fn ft02_lim_it_http_401_shortens_to_auth_expired()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_expired_token( dir.path(), "expired@test.com", "invalid-token-for-401-test" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "expired@test.com" ),
    "account row must appear in the table, got:\n{text}",
  );
  assert!(
    text.contains( "EXPIRED" ),
    "account with PAST_MS expiresAt must show EXPIRED in Expires column, got:\n{text}",
  );
  assert!(
    text.contains( "auth expired (401)" ),
    "HTTP 401 must shorten to 'auth expired (401)', got:\n{text}",
  );
  assert!(
    !text.contains( "HTTP transport error:" ),
    "verbose HTTP error string must NOT appear in output, got:\n{text}",
  );
}

// ── FT-03: All saved accounts fetched, not only _active (AC-01) ──────────────

/// FT-03 (AC-01): two saved accounts with neither stored as `_active` → both
/// names appear in stdout. Exit 0.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-03`.
#[ test ]
fn ft03_both_accounts_appear_regardless_of_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@a.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@a.com",   "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "alice@a.com" ),
    "alice@a.com must appear in output regardless of _active marker, got:\n{text}",
  );
  assert!(
    text.contains( "bob@a.com" ),
    "bob@a.com must appear in output regardless of _active marker, got:\n{text}",
  );
}

// ── FT-04: ✓ follows live token match, not _active marker (AC-02) ────────────

/// FT-04 (AC-02): `alice@a.com` is stored as `_active`; the live
/// `~/.claude/.credentials.json` has an `accessToken` matching `work@a.com`'s
/// saved token → a line in stdout contains `✓` and `work@a.com`; no line
/// contains `✓` and `alice@a.com`. Exit 0.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-04`.
#[ test ]
fn ft04_check_mark_follows_live_token_not_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@a.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@a.com",  "tok-work",  false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let work_has_check = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "work@a.com" ) );
  assert!(
    work_has_check,
    "work@a.com must have ✓ (live token match), got:\n{text}",
  );
  let alice_has_check = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "alice@a.com" ) );
  assert!(
    !alice_has_check,
    "alice@a.com must NOT have ✓ (only _active, not live token match), got:\n{text}",
  );
}

// ── FT-05: Unreadable credential store exits 2 (AC-06) ───────────────────────

/// FT-05 (AC-06): credential store directory chmod 000 → `account::list()` fails
/// → `.usage` exits 2 with a non-empty error on stderr.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-05`.
#[ cfg( unix ) ]
#[ test ]
fn ft05_unreadable_credential_store_exits_2()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the directory.
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "unreadable store must produce error on stderr" );
}
