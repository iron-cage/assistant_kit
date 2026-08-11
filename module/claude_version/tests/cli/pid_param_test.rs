//! EC- edge-case tests for the `pid::` parameter on `.ps.kill`.
//!
//! Covers EC-1 through EC-8 from `tests/docs/cli/param/17_pid.md`.
//!
//! `find_claude_processes()` scans the real `/proc` (or `CLR_PROC_DIR` when
//! set — see `claude_core::process`). Tests that need a guaranteed-valid
//! claude PID build a synthetic `CLR_PROC_DIR` with a fake `<pid>/cmdline`
//! entry whose basename is `claude`; every such test also passes `dry::1` so
//! no real `kill` signal is ever sent to the fabricated PID.

use crate::subprocess_helpers::{ assert_exit, fake_claude_process, run_clv, run_clv_with_env, stdout };
use tempfile::TempDir;

/// EC-1: `pid::` absent → bulk mode (all processes targeted)
#[ test ]
fn pid_ec1_absent_bulk_mode()
{
  let fake_proc = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".ps.kill", "dry::1" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "no active processes" ) );
}

/// EC-2: valid claude PID + `dry::1` → exit 0; only the targeted PID is referenced
#[ test ]
fn pid_ec2_valid_pid_targeted()
{
  let fake_proc  = TempDir::new().unwrap();
  let target_pid = 424_242_u32;
  let other_pid  = 424_244_u32;
  fake_claude_process( fake_proc.path(), target_pid );
  fake_claude_process( fake_proc.path(), other_pid );

  let out = run_clv_with_env(
    &[ ".ps.kill", &format!( "pid::{target_pid}" ), "dry::1" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( &target_pid.to_string() ), "must reference the targeted PID: {text}" );
  assert!( !text.contains( &other_pid.to_string() ), "must NOT reference the untargeted PID: {text}" );
}

/// EC-3: non-claude PID (PID 1, always exists but is never `claude`) → exit 1
#[ test ]
fn pid_ec3_non_claude_pid_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::1", "dry::1" ] );
  assert_exit( &out, 1 );
}

/// EC-4: PID not present in `/proc` at all → exit 1
#[ test ]
fn pid_ec4_nonexistent_pid_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::99999999" ] );
  assert_exit( &out, 1 );
}

/// EC-5: non-integer `pid::` value → exit 1 (rejected by unilang's type check)
#[ test ]
fn pid_ec5_abc_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::abc" ] );
  assert_exit( &out, 1 );
}

/// EC-6: empty `pid::` value → exit 1
#[ test ]
fn pid_ec6_empty_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::" ] );
  assert_exit( &out, 1 );
}

/// EC-7: `pid::0` → exit 1 (zero is never a valid process id)
#[ test ]
fn pid_ec7_zero_exits_1()
{
  let out = run_clv( &[ ".ps.kill", "pid::0" ] );
  assert_exit( &out, 1 );
}

/// EC-8: `pid::PID dry::1` → dry-run preview, no actual kill
#[ test ]
fn pid_ec8_dry_preview()
{
  let fake_proc = TempDir::new().unwrap();
  let fake_pid  = 424_245_u32;
  fake_claude_process( fake_proc.path(), fake_pid );

  let out = run_clv_with_env(
    &[ ".ps.kill", &format!( "pid::{fake_pid}" ), "dry::1" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "[dry-run]" ), "must show dry-run preview: {}", stdout( &out ) );
}
