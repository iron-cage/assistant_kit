//! Shared spawn helper for real-subprocess `ControlSession` integration tests (task 415).
//!
//! Every control method responds over the wire without requiring any prior user turn —
//! confirmed empirically: an `initialize` `control_request` round-trips in ~1-1.5s against a
//! freshly spawned, idle session (subprocess startup dominates; the control channel itself
//! answers in milliseconds once warm). Tests spawn a session and call methods directly, with
//! no `stream_input()` warm-up needed except where a method's own scenario requires it.
//!
//! Not compiled as its own test binary — lives at `tests/control_session_common/mod.rs`
//! (Cargo's special-cased layout for shared integration-test support code).

use claude_runner_core::{ ClaudeCommand, ControlSession, InputFormat, OutputFormat, PermissionMode };

/// Spawn a real `claude` control session in a fresh scratch directory, with the exact flag
/// combination `spawn_control_session()` requires (`--input-format stream-json
/// --output-format stream-json --verbose`), plus `--permission-mode bypassPermissions` so no
/// interactive prompt can ever block the subprocess (matches the flags captured in
/// `tests/fixtures/sdk_control_capture/argv.json`).
///
/// Returns the session alongside the `TempDir` guard — callers must keep both bindings alive
/// for the test's duration (binding the tuple, e.g. `let ( session, _dir ) = spawn_session();`,
/// is sufficient; Rust drops `session` before `_dir` since it's declared first).
///
/// # Panics
///
/// Panics if the scratch directory can't be created or the real `claude` binary can't be
/// spawned (missing from `PATH`, or the flag-precondition check in `spawn_control_session()`
/// fails) — a hard requirement for these tests, not a soft skip, per this crate's "Complete
/// Integration Tests" testing principle.
// BUG-002 task/claude_runner_core/bug/002_ac_claims_unreachable_zero_failures.md — this hard
// requirement is correct-by-design; the shared `runbox` container intentionally never
// provisions `claude`, so tests calling this helper cannot pass Container-Only Testing as-is.
#[ must_use ]
pub fn spawn_session() -> ( ControlSession, tempfile::TempDir )
{
  let dir = tempfile::TempDir::new().expect( "failed to create scratch tempdir" );
  let session = ClaudeCommand::new()
    .with_working_directory( dir.path() )
    .with_input_format( InputFormat::StreamJson )
    .with_output_format( OutputFormat::StreamJson )
    .with_verbose( true )
    .with_permission_mode( PermissionMode::BypassPermissions )
    .spawn_control_session()
    .expect( "failed to spawn real claude control session — is `claude` on PATH and authenticated?" );
  ( session, dir )
}
