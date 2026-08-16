//! `ControlSession` stderr-drain regression test — a stderr-flooding subprocess must not
//! deadlock the session (audit-control-stderr-deadlock).
//!
//! Uses a fake `claude` script (no real binary, no network): floods stderr well past the
//! OS pipe buffer, then emits one stdout marker line. Only a drained stderr pipe lets the
//! script ever reach the stdout line.

#![ cfg( unix ) ]

mod fake_claude_bin;
use fake_claude_bin::fake_claude_dir;

use claude_runner_core::{ ClaudeCommand, InputFormat, OutputFormat };

/// Session survives ~300 KiB of subprocess stderr and still delivers stdout messages.
///
/// # Root Cause (audit-control-stderr-deadlock)
///
/// `spawn_control_session()` spawned the subprocess with `stderr: piped()`, but
/// `ControlSession::from_child()` never took the handle — nothing ever read the pipe.
/// A control session mandates `--verbose`, so a normally chatty subprocess eventually
/// filled the ~64 KiB pipe buffer, blocked in `write()`, and the whole session
/// deadlocked: no stdout progress, every request timing out.
///
/// # Why Not Caught
///
/// All control-session tests ran against the real `claude` binary whose stderr volume
/// stayed under the pipe buffer for the tested scenarios — the deadlock only appears
/// past the 64 KiB threshold, which no test ever crossed.
///
/// # Fix Applied
///
/// `from_child()` now takes stderr and spawns a dedicated drain thread retaining a
/// bounded tail (last 64 lines, ≤1024 bytes each) readable via `stderr_tail()`;
/// `close()` joins the drain thread.
///
/// # Prevention
///
/// Every `Stdio::piped()` stream needs a reader for the child's entire lifetime.
/// When adding a piped stream to any spawn path, add its consumer in the same change.
///
/// # Pitfall
///
/// The flood must be written BEFORE the stdout marker in the fake script — writing the
/// marker first would let the test pass even with a full, unread stderr pipe.
#[ test ]
#[ allow( unsafe_code ) ]
fn stderr_flood_does_not_deadlock_session()
{
  // ~300 KiB of stderr (300 lines x 1001 bytes), far past the ~64 KiB pipe buffer,
  // then a single stdout marker line.
  let script = r#"i=0
while [ $i -lt 300 ]; do printf '%01000d\n' 7 1>&2; i=$((i+1)); done
printf '{"type":"noise","marker":"stderr-drain-ok"}\n'"#;
  let ( _dir, path_val ) = fake_claude_dir( script );

  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: nextest runs one process per test; no other thread reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let session = ClaudeCommand::new()
    .with_input_format( InputFormat::StreamJson )
    .with_output_format( OutputFormat::StreamJson )
    .with_verbose( true )
    .spawn_control_session();
  // SAFETY: restoring PATH to the original value.
  unsafe { std::env::set_var( "PATH", &orig_path ); }
  let mut session = session.expect( "fake claude control session must spawn" );

  // Without the drain, the subprocess blocks mid-flood and this returns None after 15s.
  let msg = session.recv_message( core::time::Duration::from_secs( 15 ) );
  session.close().expect( "close is best-effort and must not error" );

  let marker_seen = msg.as_ref()
    .and_then( | v | v.get( "marker" ) )
    .and_then( serde_json::Value::as_str )
    == Some( "stderr-drain-ok" );
  assert!(
    marker_seen,
    "stdout marker must arrive despite the stderr flood (deadlock if absent); got: {msg:?}"
  );

  // close() joined the drain thread — the tail is final: capped at 64 lines, each <=1024 bytes.
  let tail = session.stderr_tail();
  assert_eq!( tail.len(), 64, "tail must keep exactly the last 64 of 300 flood lines" );
  assert!(
    tail.iter().all( | l | l.len() <= 1024 ),
    "every retained stderr line must be truncated to <=1024 bytes"
  );
}
