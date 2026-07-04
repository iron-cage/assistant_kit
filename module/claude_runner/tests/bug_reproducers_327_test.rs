//! Bug reproducer for BUG-327: resumed session with a stale deferred-tool marker fails forever.
//!
//! # Root Cause (BUG-327)
//!
//! When a session is resumed via `-c` after the tool invocation that its resume marker
//! referred to has already run (or the marker falls outside the tail-scan window), Claude
//! Code prints `"No deferred tool marker found in the resumed session. Either the session
//! was not deferred, the marker is stale (tool already ran), or it exceeds the tail-scan
//! window. Provide a prompt to continue the conversation."` and exits non-zero.
//! `run_print_mode()` had no special handling for this diagnostic — it fell through to
//! `classify_error()`, which has no pattern for it, so it exhausted the generic `Unknown`
//! retry class (retrying with the *same* message every time) and never recovered, since
//! resending the original message reproduces the same stale-marker state.
//!
//! # Why Not Caught
//!
//! All prior retry-class tests (`retry_unknown_test.rs`, `retry_runner_test.rs`, etc.)
//! validate that retries resend the *same* builder/message. No test exercised a failure
//! mode whose only recovery is substituting a *different* message on retry.
//!
//! # Fix Applied
//!
//! `run_print_mode()` gains a one-shot `fallback_builder: Option<ClaudeCommand>` flag.
//! Before `classify_error()` runs, output is checked for the marker text; on first match
//! (`fallback_builder.is_none()`), the runner logs a diagnostic, journals a retry event,
//! and sets `fallback_builder = Some(active.clone().with_message("Continue."))`, then
//! retries. `active` (the fallback builder once set, else the original) replaces `builder`
//! at both the subprocess-invocation and `--expect` validation call sites, and the
//! journaled message reflects the substitution.
//!
//! # Prevention
//!
//! Pair every marker-triggered fallback with a "disabled path" test confirming ordinary
//! (non-marker) failures are unaffected and still exhaust through the standard
//! `classify_error()`/retry-count path.
//!
//! # Pitfall
//!
//! The fallback is a one-shot special case checked *before* `classify_error()` — it must
//! never fire twice in the same run (checked via `fallback_builder.is_none()`), or a
//! subprocess that keeps emitting the marker text would retry forever.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `bug327_deferred_tool_marker_fallback_fires` | fake claude emits marker + exits 1 once, then succeeds | exit 0; fallback diagnostic on stderr; second invocation sent "Continue." not the original message |
//! | `bug327_non_marker_error_does_not_trigger_fallback` | fake claude emits unrelated error + exits 5 always | exit 5 relayed; no fallback diagnostic; standard retry path unaffected |

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::stderr_str;
use std::os::unix::fs::PermissionsExt;

/// Exact diagnostic text Claude Code prints when a resumed session's deferred-tool
/// marker is stale or missing. Must match the constant added to `execution.rs` verbatim.
const DEFERRED_TOOL_MARKER : &str = "No deferred tool marker found in the resumed session. Either the session was not deferred, the marker is stale (tool already ran), or it exceeds the tail-scan window. Provide a prompt to continue the conversation.";

/// Original message used across both tests — deliberately distinctive so its absence
/// from the fallback invocation's argv can be asserted unambiguously.
const ORIGINAL_MESSAGE : &str = "original-test-message-xyz";

// ── bug327_deferred_tool_marker_fallback_fires ────────────────────────────────

/// BUG-327: fake claude emits the deferred-tool marker and exits 1 on the first
/// invocation, then succeeds on the second. The runner must substitute `"Continue."`
/// for the message on the fallback attempt (verified via a logged-argv file written by
/// the fake script) rather than resending the original message, and must exit 0.
#[ test ]
#[ doc = "bug_reproducer(BUG-327)" ]
fn bug327_deferred_tool_marker_fallback_fires()
{
  let tmp        = tempfile::tempdir().expect( "create temp dir" );
  let fake       = tmp.path().join( "claude" );
  let count_path = tmp.path().join( "count" );
  let args_path  = tmp.path().join( "args_log" );

  let count_str = count_path.to_str().expect( "count path utf-8" );
  let args_str  = args_path.to_str().expect( "args path utf-8" );

  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_str}\" ]; then\n\
       printf '%s\\n' \"$@\" > \"{args_str}\"\n\
       printf 'ok\\n'\n\
       exit 0\n\
     fi\n\
     touch \"{count_str}\"\n\
     printf '%s\\n' \"{DEFERRED_TOOL_MARKER}\" >&2\n\
     exit 1\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = std::process::Command::new( bin )
    .args( [
      "--print", "--max-sessions", "0", "--output-style", "raw",
      "--retry-on-unknown", "0", "--retry-override", "0",
      ORIGINAL_MESSAGE,
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  let err = stderr_str( &out );
  assert!(
    out.status.success(),
    "BUG-327: fallback attempt must succeed after marker-triggered retry. exit={:?} stderr={err}",
    out.status.code()
  );
  assert!(
    err.contains( "retrying with fallback prompt" ),
    "BUG-327: stderr must report the fallback retry. Got:\n{err}"
  );

  let logged_args = std::fs::read_to_string( &args_path )
    .expect( "BUG-327: fallback invocation must have logged its args" );
  assert!(
    logged_args.lines().any( | l | l == "Continue." ),
    "BUG-327: fallback invocation must send 'Continue.' as the message. Got args:\n{logged_args}"
  );
  assert!(
    !logged_args.contains( ORIGINAL_MESSAGE ),
    "BUG-327: fallback invocation must NOT resend the original message. Got args:\n{logged_args}"
  );
}

// ── bug327_non_marker_error_does_not_trigger_fallback ─────────────────────────

/// BUG-327 (paired disabled-path test): a non-marker failure must NOT trigger the
/// fallback substitution. With all other retry classes explicitly disabled, the
/// subprocess's exit code must relay unchanged and no fallback diagnostic may appear.
#[ test ]
#[ doc = "bug_reproducer(BUG-327)" ]
fn bug327_non_marker_error_does_not_trigger_fallback()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf 'something went wrong\\n' >&2\nexit 5\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = std::process::Command::new( bin )
    .args( [
      "--print", "--max-sessions", "0", "--output-style", "raw",
      "--retry-on-unknown", "0", "--retry-override", "0",
      ORIGINAL_MESSAGE,
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  let err = stderr_str( &out );
  assert_eq!(
    out.status.code(), Some( 5 ),
    "BUG-327: non-marker failure must exhaust via the standard retry path and relay exit 5. Got: {:?} stderr={err}",
    out.status.code()
  );
  assert!(
    !err.contains( "deferred tool marker" ),
    "BUG-327: fallback diagnostic must NOT appear for a non-marker error. Got:\n{err}"
  );
}
