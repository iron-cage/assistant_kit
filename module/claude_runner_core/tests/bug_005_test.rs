//! BUG-005 reproducer: timeout-with-refreshed-credentials branch preserves partial stdout.
//!
//! # Root Cause
//!
//! `Fix(issue-isolated-credentials-on-timeout)` added the `credentials.is_some()` timeout
//! branch to stop a refreshed OAuth token from being discarded on timeout. At the time, the
//! thread/channel implementation made buffered subprocess output unrecoverable on timeout
//! regardless of branch, so `stdout`/`stderr` were filled with accurate `String::new()`
//! placeholders. `Fix(BUG-243)` later replaced that implementation with `spawn_piped()` +
//! `try_wait` polling so output IS recoverable after `kill()`, and wired the recovered data
//! into the sibling `TimeoutWithOutput` branch it was written against — but never revisited
//! this branch, whose placeholders silently went from accurate to lossy.
//!
//! # Why Not Caught
//!
//! `isolated_test.rs` T09 hand-constructs an `IsolatedRunResult` literal and never calls
//! `run_isolated`/`run_isolated_ext`, so the branch had zero production coverage. BUG-243's
//! own reproducer supplies credentials the fake `claude` never rewrites, so it always lands
//! on the sibling `TimeoutWithOutput` branch and never exercises this one.
//!
//! # Fix Applied
//!
//! `isolated.rs`'s `credentials.is_some()` timeout branch now forwards the already-recovered
//! `stdout`/`stderr` locals instead of substituting `String::new()` for both.
//!
//! # Prevention
//!
//! When a fix makes previously-unavailable data available (BUG-243), audit every other
//! construction site of the type that carries it — not only the one site that motivated it.
//!
//! # Pitfall
//!
//! `exit_code: -1` is this crate's only producer of `-1`; every `-1` a caller observes comes
//! from this branch, so whatever these two fields carry is the entirety of that call's
//! diagnostics.

#![ cfg( unix ) ]

use claude_runner_core::{ run_isolated, IsolatedModel };
mod fake_claude_bin;
use fake_claude_bin::fake_claude_dir;

/// BUG-005 reproducer: timeout with refreshed credentials must still return the recovered
/// `stdout` (BUG-243's contract), not the empty-string placeholder this branch substituted.
#[ test ]
#[ doc = "bug_reproducer(BUG-005)" ]
#[ allow( unsafe_code ) ]
fn timeout_with_credentials_preserves_partial_stdout()
{
  // Fake claude: emit a marker, refresh the credentials file, then outlive the deadline.
  // Writing creds is what steers execution into the `credentials.is_some()` branch.
  // Fix(BUG-243-slow) precedent: sleep 3, not longer — keeps the test fast while still
  // outliving the 1s timeout below.
  let ( _dir, path_val ) = fake_claude_dir(
    "printf 'partial-output-marker'; \
     printf '{\"accessToken\":\"refreshed\",\"refreshToken\":\"fake\",\"expiresAt\":9999999999999}' \
       > \"$HOME/.claude/.credentials.json\"; \
     sleep 3"
  );
  let creds_json = r#"{"accessToken":"original","refreshToken":"fake","expiresAt":9999999999999}"#;

  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  // SAFETY: single-threaded test binary; no other test reads PATH concurrently.
  unsafe { std::env::set_var( "PATH", &path_val ); }
  let result = run_isolated( creds_json, vec![], 1, IsolatedModel::KeepCurrent );
  // SAFETY: restoring PATH to the original value; single-threaded test binary.
  unsafe { std::env::set_var( "PATH", &orig_path ); }

  let out = result.expect( "timeout+credentials must return Ok, not Err" );
  assert_eq!( out.exit_code, -1, "timeout-with-credentials sentinel must be -1" );
  assert!( out.credentials.is_some(), "refreshed credentials must be forwarded" );
  assert!(
    out.stdout.contains( "partial-output-marker" ),
    "BUG-005: recovered stdout must survive the credentials branch; got: {:?}", out.stdout
  );
}
