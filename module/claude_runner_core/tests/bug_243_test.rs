//! BUG-243 reproducer: timeout with partial stdout preserved.
//!
//! # Root Cause (BUG-243)
//!
//! `run_isolated()` spawned a thread that called `cmd.execute()` (which calls
//! `cmd.output()`, blocking until EOF). The main thread used `recv_timeout` to impose
//! a deadline. When the deadline fired, the thread kept the `Child` handle; no kill or
//! partial-output collection was possible — all buffered stdout was irrecoverably dropped.
//!
//! # Why Not Caught (BUG-243)
//!
//! All timeout tests (IT-3, IT-4) used `timeout=0` and asserted on the error type, not
//! on the content of the error message. No test verified that partial stdout was preserved.
//!
//! # Fix Applied (BUG-243)
//!
//! Restructured `run_isolated()` to use `spawn_piped()` (new method on `ClaudeCommand`)
//! + `try_wait` polling. On timeout: `child.kill()` then `child.wait_with_output()`
//!   collects buffered data. Added `RunnerError::TimeoutWithOutput { secs, partial_stdout }`.
//!
//! # Prevention (BUG-243)
//!
//! When you need timeout+kill+output: always keep the `Child` handle in scope through the
//! timeout. Thread-based approaches that move `Child` into the thread lose this ability.
//!
//! # Pitfall (BUG-243)
//!
//! `child.wait_with_output()` waits for stdout/stderr pipes to close (which happens after
//! kill), then returns whatever was buffered. Call it AFTER `child.kill()`, not before.

#![ cfg( unix ) ]

use claude_runner_core::{ run_isolated, RunnerError, IsolatedModel };

// Return a temp dir containing a fake `claude` shell script and the augmented PATH value.
//
// The returned `TempDir` must be kept alive for the duration of the test — dropping it
// removes the directory and makes the fake binary inaccessible.
#[ cfg( unix ) ]
fn fake_claude_dir( body : &str ) -> ( tempfile::TempDir, String )
{
  use std::os::unix::fs::PermissionsExt as _;
  let dir  = tempfile::TempDir::new().expect( "tmpdir" );
  let path = dir.path().join( "claude" );
  let script = format!( "#!/bin/sh\n{body}\n" );
  std::fs::write( &path, script.as_bytes() ).expect( "write fake-claude" );
  std::fs::set_permissions( &path, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake-claude" );
  let path_val = format!(
    "{}:{}",
    dir.path().display(),
    std::env::var( "PATH" ).unwrap_or_default(),
  );
  ( dir, path_val )
}

/// BUG-243 reproducer: timeout fires after partial stdout → `TimeoutWithOutput` has content.
///
/// Before fix: the thread/channel approach left `Child` inside the spawned thread;
/// `recv_timeout` fired and all buffered stdout was irrecoverably discarded — the error
/// variant `Timeout { secs }` carried no output.
/// After fix: `spawn_piped()` + `try_wait` polling keeps `Child` in scope; on timeout
/// `child.kill()` + `child.wait_with_output()` recovers buffered data.
#[ test ]
#[ doc = "bug_reproducer(BUG-243)" ]
#[ allow( unsafe_code ) ]
fn timeout_includes_partial_stdout()
{
  // Fake claude: print a marker then sleep briefly.
  // Fix(BUG-243-slow): use sleep 3 instead of sleep 999 to keep test fast.
  // Root cause: child.kill() only kills the direct shell process; the `sleep` grandchild
  //   inherits the pipe FD and holds it open until it exits, blocking wait_with_output().
  // Pitfall: reducing sleep doesn't change the test assertion — the timeout still fires
  //   after 1s, the marker is still captured. The grandchild just exits sooner (~2s after).
  let ( _dir, path_val ) = fake_claude_dir( "printf 'partial-output-marker'; sleep 3" );

  // Minimal credentials JSON.
  let creds_json = r#"{"accessToken":"fake","refreshToken":"fake","expiresAt":9999999999999}"#;

  // Temporarily extend PATH so run_isolated can find the fake claude binary.
  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: single-threaded test binary; no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let result = run_isolated( creds_json, vec![], 1, IsolatedModel::KeepCurrent );
  // SAFETY: restoring PATH to the original value; single-threaded test binary.
  unsafe { std::env::set_var( "PATH", &orig_path ); }

  match result
  {
    Err( RunnerError::TimeoutWithOutput { secs : _, partial_stdout } ) =>
    {
      assert!(
        partial_stdout.contains( "partial-output-marker" ),
        "BUG-243: TimeoutWithOutput.partial_stdout must contain the marker; got:\n{partial_stdout}"
      );
    }
    Err( RunnerError::Timeout { .. } ) =>
    {
      panic!( "BUG-243: expected TimeoutWithOutput (with content), got Timeout (empty)" );
    }
    other =>
    {
      panic!( "BUG-243: expected TimeoutWithOutput error; got: {other:?}" );
    }
  }
}
