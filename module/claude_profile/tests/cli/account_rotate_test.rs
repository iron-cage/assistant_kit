//! Integration tests: ROT (Account Rotate).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### ROT — Account Rotate
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | rot01 | `rot01_empty_store_exits_2` | empty credential store → exit 2 | N |
//! | rot02 | `rot02_single_active_exits_2` | only one account and it is active → exit 2 | N |
//! | rot03 | `rot03_rotates_to_best_inactive` | two accounts; rotates to inactive; `_active` changes | P |
//! | rot04 | `rot04_selects_highest_expires_at` | three accounts; selects highest `expiresAt` inactive | P |
//! | rot05 | `rot05_dry_no_mutation` | `dry::1` shows candidate; `_active` unchanged | P |
//! | rot06 | `rot06_dry_output_prefix` | `dry::1` output contains `[dry-run]` | P |
//! | rot07 | `rot07_output_confirms_name` | output contains `rotated to` + account name | P |
//! | rot08 | `rot08_unknown_param_exits_1` | unknown parameter → exit 1 | N |

use crate::helpers::{
  run_cs_with_env,
  stdout, assert_exit,
  write_credentials, write_account,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── ROT: Account Rotate ───────────────────────────────────────────────────────

#[ test ]
fn rot01_empty_store_exits_2()
{
  // IT-1: no credential files at all → auto_rotate finds no inactive accounts → exit 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn rot02_single_active_exits_2()
{
  // IT-2: only one account; it is active → no inactive candidates → exit 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "solo@example.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn rot03_rotates_to_best_inactive()
{
  // IT-3: two accounts; inactive has higher expiresAt; _active must change to inactive account.
  // write_credentials creates ~/.claude/ so switch_account can write .credentials.json there.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", PAST_MS );
  write_account( dir.path(), "active@work.com",    "pro", "standard", PAST_MS,       true  );
  write_account( dir.path(), "best@candidate.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must exist after rotate" );
  assert_eq!(
    active.trim(),
    "best@candidate.com",
    "_active must change to the best inactive account",
  );
}

#[ test ]
fn rot04_selects_highest_expires_at()
{
  // IT-4: three accounts — one active, two inactive with different expiresAt;
  // the account with the highest expiresAt among inactive accounts wins.
  // write_credentials creates ~/.claude/ so switch_account can write .credentials.json there.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "current@work.com", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "lower@acme.com",   "pro", "standard", PAST_MS,       false );
  write_account( dir.path(), "higher@acme.com",  "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must exist after rotate" );
  assert_eq!(
    active.trim(),
    "higher@acme.com",
    "must select the inactive account with the highest expiresAt",
  );
}

#[ test ]
fn rot05_dry_no_mutation()
{
  // IT-5: dry::1 shows the candidate but does NOT switch _active.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "active@work.com",    "pro", "standard", PAST_MS,       true  );
  write_account( dir.path(), "best@candidate.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let before = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must exist before dry-run" );

  let out = run_cs_with_env( &[ ".account.rotate", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must still exist after dry-run" );
  assert_eq!(
    before.trim(),
    after.trim(),
    "_active must not change during dry-run",
  );
}

#[ test ]
fn rot06_dry_output_prefix()
{
  // IT-6: dry::1 output contains the `[dry-run]` prefix.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "active@work.com",    "pro", "standard", PAST_MS,       true  );
  write_account( dir.path(), "best@candidate.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.rotate", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "dry-run output must contain [dry-run] prefix, got:\n{text}",
  );
}

#[ test ]
fn rot07_output_confirms_name()
{
  // IT-7: live rotation output contains "rotated to" and the selected account name.
  // write_credentials creates ~/.claude/ so switch_account can write .credentials.json there.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", PAST_MS );
  write_account( dir.path(), "active@work.com",    "pro", "standard", PAST_MS,       true  );
  write_account( dir.path(), "best@candidate.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "rotated to" ),
    "output must confirm rotation with 'rotated to', got:\n{text}",
  );
  assert!(
    text.contains( "best@candidate.com" ),
    "output must name the selected account, got:\n{text}",
  );
}

#[ test ]
fn rot08_unknown_param_exits_1()
{
  // IT-8: `.account.rotate` only accepts `dry::` — any other param is rejected with exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "active@work.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.rotate", "unknown::x" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}
