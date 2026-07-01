//! Integration tests: ROT (Account Rotate — deprecated redirector).
//!
//! `.account.rotate` is deprecated (Feature 038). All invocations exit 1 with
//! a message directing users to `.usage rotate::1`.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### ROT — Account Rotate redirector
//!
//! | ID | Test Function | Condition | P/N | IT-N |
//! |----|---------------|-----------|-----|------|
//! | rot01 | `rot01_always_exits_1` | any invocation → exit 1 | N | IT-1 |
//! | rot02 | `rot02_message_contains_usage_rotate` | stderr/stdout contains `.usage rotate` | N | IT-2 |
//! | rot03 | `rot03_no_mutation_on_exit_1` | _active file unchanged after deprecated call | N | IT-3 |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── ROT: Account Rotate redirector ────────────────────────────────────────────

/// Spec: [tests/docs/cli/command/13_account_rotate.md IT-1]
#[ test ]
fn rot01_always_exits_1()
{
  // IT-1: `.account.rotate` is a deprecated redirector — always exits 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// Spec: [tests/docs/cli/command/13_account_rotate.md IT-2]
#[ test ]
fn rot02_message_contains_usage_rotate()
{
  // IT-2: redirector error message directs users to `.usage rotate::1`.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );

  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!(
    combined.contains( ".usage rotate" ),
    "error output must reference '.usage rotate', got:\n{combined}",
  );
}

/// Spec: [tests/docs/cli/command/13_account_rotate.md IT-3]
#[ test ]
fn rot03_no_mutation_on_exit_1()
{
  // IT-3: deprecated call exits 1 and leaves _active unchanged.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "active@work.com",    "pro", "standard", PAST_MS,       true  );
  write_account( dir.path(), "best@candidate.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let before = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must exist before deprecated call" );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );

  let after = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must still exist after deprecated call" );
  assert_eq!(
    before.trim(),
    after.trim(),
    "_active must not change when deprecated command exits 1",
  );
}
