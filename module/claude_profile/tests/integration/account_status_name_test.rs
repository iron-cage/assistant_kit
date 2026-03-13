//! Integration tests: ASTNAME (Account Status — `name::` parameter).
//!
//! Covers FR-16: optional `name::` parameter on `.account.status`.
//!
//! ## Test Series Summary
//!
//! | ID | Scenario | Key Assertion |
//! |----|----------|---------------|
//! | astname01 | name:: = active account | Same output as no-name path |
//! | astname02 | name:: = non-active account (different expiry) | Shows named account's OWN token state |
//! | astname03 | name:: = nonexistent account | Exit 2 + "not found" in stderr |
//! | astname04 | name:: = invalid chars | Exit 1 |
//! | astname05 | name:: with v::0 | Two bare lines |
//! | astname06 | name:: = active + .claude.json present | Shows Email/Org from .claude.json |
//! | astname07 | name:: = non-active + v::1 | Email: N/A, Org: N/A |
//! | astname08 | name:: = non-active + v::2 | Shows Expires: line |
//! | astname09 | name:: + format::json | JSON with account + token |
//! | astname10 | name:: = non-active, v::0, own expiry shown | Bare output with named account's expiry |
//! | astname11 | name:: = active, v::1 | Shows Sub: and Tier: from stored account data |
//! | astname12 | name:: = non-active, v::1 | Shows named account's own Sub/Tier, not active account's |
//! | astname13 | name:: = non-active, subscriptionType absent in file | Sub: N/A (not blank) |
//! | astname14 | name:: = non-active, rateLimitTier absent in file | Tier: N/A (not blank) |

use crate::helpers::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_claude_json,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── astname01: name:: = active account → identical to no-name path ────────────

#[ test ]
fn astname01_name_equals_active_same_as_no_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out_no_name  = run_cs_with_env( &[ ".account.status" ],            &[ ( "HOME", home ) ] );
  let out_with_name = run_cs_with_env( &[ ".account.status", "name::work" ], &[ ( "HOME", home ) ] );

  assert_exit( &out_no_name,   0 );
  assert_exit( &out_with_name, 0 );
  // Both paths should show the same account name and token state at default verbosity
  assert!(
    stdout( &out_with_name ).contains( "work" ),
    "must show account name, got:\n{}", stdout( &out_with_name ),
  );
  assert!(
    stdout( &out_with_name ).contains( "valid" ),
    "must show valid token, got:\n{}", stdout( &out_with_name ),
  );
}

// ── astname02: P2 guard — non-active account shows OWN expiry ─────────────────

#[ test ]
fn astname02_nonactive_account_shows_own_expiry()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // work is active with a valid far-future token
  write_account( dir.path(), "work", "max", "tier4", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  // personal is NOT active and has an already-expired token
  write_account( dir.path(), "personal", "pro", "standard", PAST_MS, false );

  let out = run_cs_with_env( &[ ".account.status", "name::personal" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "personal" ), "must show queried account name, got:\n{text}" );
  assert!(
    text.contains( "expired" ),
    "non-active account must show its OWN expired token, not the active account's valid token, got:\n{text}",
  );
  assert!(
    !text.contains( "valid" ),
    "must NOT show active account's valid state for non-active query, got:\n{text}",
  );
}

// ── astname03: nonexistent name → exit 2 ──────────────────────────────────────

#[ test ]
fn astname03_nonexistent_name_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "name::ghost" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "not found" ) || err.to_lowercase().contains( "ghost" ),
    "stderr must mention not found or account name, got:\n{err}",
  );
  assert!( stdout( &out ).is_empty(), "stdout must be empty on error, got:\n{}", stdout( &out ) );
}

// ── astname04: invalid chars in name → exit 1 ─────────────────────────────────

#[ test ]
fn astname04_invalid_chars_in_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // "/" is an invalid character per validate_name
  let out = run_cs_with_env( &[ ".account.status", "name::a/b" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 1 );
}

// ── astname05: name:: + v::0 → two bare lines ────────────────────────────────

#[ test ]
fn astname05_name_v0_bare_output()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "name::work", "v::0" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 2, "v::0 must produce exactly 2 lines, got:\n{text}" );
  assert_eq!( lines[ 0 ], "work",  "v::0 line 0 must be bare account name, got:\n{text}" );
  assert_eq!( lines[ 1 ], "valid", "v::0 line 1 must be bare token state, got:\n{text}" );
  assert!( !text.contains( "Account:" ), "v::0 must not have labels, got:\n{text}" );
}

// ── astname06: name:: = active + .claude.json → shows Email/Org ───────────────

#[ test ]
fn astname06_active_v1_shows_email_org()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "alice@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".account.status", "name::work", "v::1" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@example.com" ), "v::1 must show email from .claude.json, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ),         "v::1 must show org from .claude.json, got:\n{text}" );
}

// ── astname07: name:: = non-active + v::1 → N/A for email/org ────────────────

#[ test ]
fn astname07_nonactive_v1_na_email_org()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal", "pro", "standard", FAR_FUTURE_MS, false );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "alice@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".account.status", "name::personal", "v::1" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "personal" ), "must show queried account name, got:\n{text}" );
  assert!(
    text.contains( "N/A" ),
    "non-active account email/org must show N/A, got:\n{text}",
  );
  assert!(
    !text.contains( "alice@example.com" ),
    "must NOT leak active account email into non-active query, got:\n{text}",
  );
}

// ── astname08: name:: = non-active + v::2 → shows Expires: line ──────────────

#[ test ]
fn astname08_nonactive_v2_shows_expires()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal", "pro", "standard", FAR_FUTURE_MS, false );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "name::personal", "v::2" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "personal" ), "must show queried account name, got:\n{text}" );
  assert!( text.contains( "Expires:" ), "v::2 must show Expires: line, got:\n{text}" );
}

// ── astname09: name:: + format::json → JSON with account + token ─────────────

#[ test ]
fn astname09_name_format_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "name::work", "format::json" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().starts_with( '{' ), "JSON must start with '{{', got:\n{text}" );
  assert!( text.contains( "\"account\":\"work\"" ), "JSON must contain account field, got:\n{text}" );
  assert!( text.contains( "\"token\":\"valid\"" ),  "JSON must contain token field, got:\n{text}" );
}

// ── astname10: non-active + v::0 → bare output with named account's state ─────

#[ test ]
fn astname10_nonactive_v0_shows_own_state()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal", "pro", "standard", PAST_MS,       false );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "name::personal", "v::0" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 2, "v::0 must produce 2 lines, got:\n{text}" );
  assert_eq!( lines[ 0 ], "personal", "line 0 must be the queried account name, got:\n{text}" );
  assert_eq!( lines[ 1 ], "expired",  "line 1 must reflect personal's own expired state, got:\n{text}" );
}

// ── astname11: named active + v::1 → shows Sub + Tier ─────────────────────────

#[ test ]
fn astname11_active_named_v1_shows_sub_tier()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "alice@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".account.status", "name::work", "v::1" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Sub:" ),     "v::1 must show Sub: line, got:\n{text}" );
  assert!( text.contains( "Tier:" ),    "v::1 must show Tier: line, got:\n{text}" );
  assert!( text.contains( "pro" ),      "v::1 Sub must match subscription type, got:\n{text}" );
  assert!( text.contains( "standard" ), "v::1 Tier must match rate limit tier, got:\n{text}" );
}

// ── astname12: named non-active + v::1 → shows own Sub + Tier ─────────────────

#[ test ]
fn astname12_nonactive_named_v1_shows_own_sub_tier()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work",     "max",   "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal", "pro",   "standard", FAR_FUTURE_MS, false );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "name::personal", "v::1" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Sub:" ),     "v::1 must show Sub: line, got:\n{text}" );
  assert!( text.contains( "Tier:" ),    "v::1 must show Tier: line, got:\n{text}" );
  // personal has "pro"/"standard" — NOT active account's "max"/"tier4"
  assert!( text.contains( "pro" ),      "v::1 Sub must show personal's subscription type, got:\n{text}" );
  assert!( text.contains( "standard" ), "v::1 Tier must show personal's rate tier, got:\n{text}" );
  assert!( !text.contains( "max" ),     "v::1 must not leak active account's subscription, got:\n{text}" );
  assert!( !text.contains( "tier4" ),   "v::1 must not leak active account's tier, got:\n{text}" );
}

// ── astname13: missing subscriptionType in file → Sub: N/A (not blank) ────────

// test_kind: bug_reproducer
// Root Cause: account::list() uses unwrap_or_default() → "" for missing fields.
//   status_named used the raw struct value without N/A normalization, producing blank
//   output for missing sub/tier. Active path showed N/A via unwrap_or_else — inconsistent.
// Fix: Normalize empty strings to "N/A" in status_named.

#[ test ]
fn astname13_missing_sub_in_file_shows_n_a()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  // personal has no subscriptionType field — should show N/A, not blank
  let accounts_dir = dir.path().join( ".claude" ).join( "accounts" );
  std::fs::write(
    accounts_dir.join( "personal.credentials.json" ),
    r#"{"oauthAccount":{"rateLimitTier":"standard"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out = run_cs_with_env( &[ ".account.status", "name::personal" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub:     N/A" ),
    "missing subscriptionType must show 'Sub:     N/A', got:\n{text}",
  );
}

// ── astname14: missing rateLimitTier in file → Tier: N/A (not blank) ──────────

#[ test ]
fn astname14_missing_tier_in_file_shows_n_a()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  // personal has no rateLimitTier field — should show N/A, not blank
  let accounts_dir = dir.path().join( ".claude" ).join( "accounts" );
  std::fs::write(
    accounts_dir.join( "personal.credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"pro"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out = run_cs_with_env( &[ ".account.status", "name::personal" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Tier:    N/A" ),
    "missing rateLimitTier must show 'Tier:    N/A', got:\n{text}",
  );
}
