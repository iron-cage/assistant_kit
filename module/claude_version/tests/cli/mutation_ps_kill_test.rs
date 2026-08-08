//! Integration tests for `.ps.kill` — E7.
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 310 | `dry::1` → `no active processes` or `[dry-run]`, exit 0 | P | 0 |
//! | 311 | `dry::1` no processes → `no active processes` | P | 0 |
//! | 312 | `dry::1 force::1` no processes → `no active processes` | P | 0 |
//! | 313 | `v::0` → accepted, exit 0 | P | 0 |
//! | 314 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | 315 | `let _ = send_sigterm/sigkill` removed — errors now propagated | verify | — |
//! | 316 | `dry::1 format::json` → JSON output, exit 0 | P | 0 |
//! | 317 | `pid::1 dry::1` → exit 1 (PID 1 is not a claude process) | N | 1 |
//! | 318 | `pid::99999999` → exit 1 (nonexistent PID) | N | 1 |
//! | 319 | `pid::abc` → exit 1 (non-integer `pid::`) | N | 1 |
//! | 320 | bulk kill, no processes (deterministic) → exit 0 | P | 0 |
//! | 321 | `bogus::x` → exit 1 | N | 1 |
//! | 322 | `dry::2` → exit 1, out-of-range boolean | N | 1 |
//! | 323 | `force::2` → exit 1, out-of-range boolean | N | 1 |
//!
//! # Lesson Learned
//!
//! **`/proc` is global state**: `find_claude_processes()` scans the real `/proc`
//! regardless of subprocess environment. Tests for `ps kill` cannot assume
//! zero processes — they must handle both "no processes" and "processes exist" paths.
//! Setting `PATH=""` only hides the `claude` binary from subprocess, not from `/proc`.
//! `CLR_PROC_DIR` (`claude_core` scanner override) is the supported seam: point it at
//! an empty directory to give a live-kill test a deterministic zero-process table.

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout };
use tempfile::TempDir;

// ─── E7: ps kill ───────────────────────────────────────────────────────────

// TC-310: .ps.kill dry::1 exits 0 — shows [dry-run] or "no active processes"
#[ test ]
fn tc310_ps_kill_dry_exits_0()
{
  let out = run_clv( &[ ".ps.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no active processes" ) || text.contains( "[dry-run]" ),
    "must be dry-run preview or no processes: {text}"
  );
}

// TC-311: .ps.kill dry::1 → preview mentions SIGTERM
#[ test ]
fn tc311_ps_kill_dry_mentions_sigterm()
{
  let out = run_clv( &[ ".ps.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  if !text.contains( "no active processes" )
  {
    assert!( text.contains( "SIGTERM" ), "dry-run must mention SIGTERM: {text}" );
  }
}

// TC-312: .ps.kill dry::1 force::1 → dry wins, mentions SIGKILL
#[ test ]
fn tc312_ps_kill_dry_force_mentions_sigkill()
{
  let out = run_clv( &[ ".ps.kill", "dry::1", "force::1" ] );
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
fn tc315_ps_kill_no_let_underscore_on_send_sig()
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
//
// Confined to an empty CLR_PROC_DIR (claude_core process-scanner override) so
// this live (non-dry) kill sweeps a simulated-empty process table: the
// whole-workspace suite runs many claude-spawning tests in one container PID
// namespace, and a real sweep both kills sibling tests' subprocesses and fails
// its own settle-rescan when siblings spawn new ones during the 500ms window.
#[ test ]
fn tc313_ps_kill_v0_accepted()
{
  let fake_proc = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".ps.kill", "v::0" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
}

// TC-314: format::JSON (uppercase) → exit 1
#[ test ]
fn tc314_ps_kill_format_uppercase_rejected()
{
  let out = run_clv( &[ ".ps.kill", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-316: dry::1 format::json → JSON output, exit 0
#[ test ]
fn tc316_ps_kill_dry_format_json()
{
  let out = run_clv( &[ ".ps.kill", "dry::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '{' ),
    "format::json must produce JSON object: {text}"
  );
}

// TC-317 / IT-11: pid::1 dry::1 → exit 1 (PID 1 exists but is not a claude process)
#[ test ]
fn tc317_ps_kill_pid_non_claude_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::1", "dry::1" ] );
  assert_exit( &out, 1 );
}

// TC-318 / IT-12: pid::99999999 → exit 1 (PID not in /proc)
#[ test ]
fn tc318_ps_kill_pid_nonexistent_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::99999999" ] );
  assert_exit( &out, 1 );
}

// TC-319 / IT-13: pid::abc → exit 1 (non-integer, rejected by unilang type check)
#[ test ]
fn tc319_ps_kill_pid_non_integer_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::abc" ] );
  assert_exit( &out, 1 );
}

// TC-320 / IT-1: bulk kill (no pid::, no dry::) with zero processes → exit 0
//
// Uses CLR_PROC_DIR override (see TC-313) for a deterministic empty process
// table — a real (non-dry) bulk kill against actual /proc would risk
// terminating sibling tests' subprocesses in this shared-container suite.
#[ test ]
fn tc320_ps_kill_bulk_no_processes_exits_0()
{
  let fake_proc = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".ps.kill" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "no active processes" ) );
}

// TC-321 / IT-8: bogus::x → exit 1 (unknown parameter)
#[ test ]
fn tc321_ps_kill_bogus_param_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-322 / IT-9: dry::2 → exit 1 (out-of-range boolean)
#[ test ]
fn tc322_ps_kill_dry_out_of_range_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "dry::2" ] );
  assert_exit( &out, 1 );
}

// TC-323 / IT-10: force::2 → exit 1 (out-of-range boolean)
#[ test ]
fn tc323_ps_kill_force_out_of_range_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "force::2" ] );
  assert_exit( &out, 1 );
}
