//! Integration tests for `.processes.kill` — E7.
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 310 | no processes → `no active processes`, exit 0 | P | 0 |
//! | 311 | `dry::1` no processes → `no active processes` | P | 0 |
//! | 312 | `dry::1 force::1` no processes → `no active processes` | P | 0 |
//! | 313 | `v::0` → accepted, exit 0 | P | 0 |
//! | 314 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | 315 | `let _ = send_sigterm/sigkill` removed — errors now propagated | verify | — |
//! | 316 | `dry::1 format::json` → JSON output, exit 0 | P | 0 |
//!
//! # Lesson Learned
//!
//! **`/proc` is global state**: `find_claude_processes()` scans the real `/proc`
//! regardless of subprocess environment. Tests for `processes kill` cannot assume
//! zero processes — they must handle both "no processes" and "processes exist" paths.
//! Setting `PATH=""` only hides the `claude` binary from subprocess, not from `/proc`.

use crate::subprocess_helpers::{ assert_exit, run_clv, stdout };

// ─── E7: processes kill ───────────────────────────────────────────────────────

// TC-310: .processes.kill dry::1 exits 0 — shows [dry-run] or "no active processes"
#[ test ]
fn tc310_processes_kill_dry_exits_0()
{
  let out = run_clv( &[ ".processes.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no active processes" ) || text.contains( "[dry-run]" ),
    "must be dry-run preview or no processes: {text}"
  );
}

// TC-311: .processes.kill dry::1 → preview mentions SIGTERM
#[ test ]
fn tc311_processes_kill_dry_mentions_sigterm()
{
  let out = run_clv( &[ ".processes.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  if !text.contains( "no active processes" )
  {
    assert!( text.contains( "SIGTERM" ), "dry-run must mention SIGTERM: {text}" );
  }
}

// TC-312: .processes.kill dry::1 force::1 → dry wins, mentions SIGKILL
#[ test ]
fn tc312_processes_kill_dry_force_mentions_sigkill()
{
  let out = run_clv( &[ ".processes.kill", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  if !text.contains( "no active processes" )
  {
    assert!( text.contains( "SIGKILL" ), "dry+force must mention SIGKILL: {text}" );
  }
}

// TC-315: verify signal errors are no longer silently swallowed
//
// Root Cause: `let _ = send_sigterm(p.pid)` and `let _ = send_sigkill(p.pid)`
//   discarded all signal delivery errors, making exit code 2 unreachable when a
//   signal failed for any reason other than "process survived" (caught by the
//   trailing `remaining > 0` check).
// Why Not Caught: no test exercised the signal-error path — triggering it
//   requires a process that exists in the Claude process list but rejects signals,
//   which is not reproducible in a clean test environment without injection.
// Fix Applied: `let _` replaced with proper Result collection; Err is returned
//   immediately if any signal delivery fails.
// Prevention: AF check below verifies the `let _` pattern is absent at source level.
// Pitfall: `find_claude_processes()` reads real `/proc`; tests cannot inject fake
//   PIDs into the process list, so the new error path is verified via code inspection
//   only. Functional regression is covered by TC-310–312 (happy paths still work).
#[ test ]
fn tc315_processes_kill_no_let_underscore_on_send_sig()
{
  // Verify at source level that `let _ = send_sig` is absent from commands/process.rs.
  // This is an AF (anti-faking) check — the only reliable test for a code path
  // that cannot be triggered through the binary without process injection.
  let src = std::fs::read_to_string( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/commands/process.rs" ) )
    .expect( "could not read commands/process.rs for AF check" );
  assert!(
    !src.contains( "let _ = send_sigterm" ),
    "let _ = send_sigterm must be absent — signal errors must be propagated",
  );
  assert!(
    !src.contains( "let _ = send_sigkill" ),
    "let _ = send_sigkill must be absent — signal errors must be propagated",
  );
}

// TC-313: v::0 → accepted, exit 0
#[ test ]
fn tc313_processes_kill_v0_accepted()
{
  let out = run_clv( &[ ".processes.kill", "v::0" ] );
  assert_exit( &out, 0 );
}

// TC-314: format::JSON (uppercase) → exit 1
#[ test ]
fn tc314_processes_kill_format_uppercase_rejected()
{
  let out = run_clv( &[ ".processes.kill", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-316: dry::1 format::json → JSON output, exit 0
#[ test ]
fn tc316_processes_kill_dry_format_json()
{
  let out = run_clv( &[ ".processes.kill", "dry::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '{' ),
    "format::json must produce JSON object: {text}"
  );
}
