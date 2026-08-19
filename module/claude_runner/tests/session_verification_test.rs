//! Integration tests for BUG-320 session mismatch detection.
//!
//! Verifies that `run_print_mode()` emits a non-fatal warning on stderr when the
//! `session_id` returned by claude does not match the UUID expected from the
//! pre-launch session storage scan.
//!
//! # Root Cause (BUG-320)
//! `session_exists()` returned `bool`, making the expected UUID inaccessible after
//! `-c` injection.  Without the UUID, post-execution comparison was impossible and
//! silent session drift went undetected.
//!
//! # Fix Applied
//! `session_exists()` returns `Option<SessionId>`; `build_claude_command()` returns
//! `(ClaudeCommand, Option<SessionId>)`.  On the print-mode success path,
//! `run_print_mode()` calls `extract_session_id(raw_stdout)` and compares.
//!
//! # Why Not Caught
//! No prior test exercised the post-execution comparison path because `session_exists()`
//! discarded the UUID before returning.
//!
//! # Prevention
//! sv2 below is the regression guard: it asserts the warning fires when UUIDs differ.
//!
//! # Pitfall
//! Mismatch detection only fires in print mode — interactive sessions output TTY escape
//! codes, not a parseable JSON envelope, so `extract_session_id()` returns `None`.
#![ cfg( unix ) ]
#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, make_session_for, stderr_str };

/// UUID written to the session dir `.jsonl` file — the "expected" session.
const UUID_A : &str = "11111111-1111-1111-1111-111111111111";

/// UUID returned in the claude JSON envelope — simulates a different active session.
const UUID_B : &str = "22222222-2222-2222-2222-222222222222";

/// Build a minimal CLR result JSON envelope with the given `session_id`.
fn clr_envelope( session_id : &str ) -> String
{
  format!(
    r#"{{"type":"result","subtype":"success","session_id":"{session_id}","is_error":false,"result":"ok","usage":{{"input_tokens":1,"output_tokens":1}},"total_cost_usd":0.0}}"#
  )
}

/// Run `clr` with a fake claude that emits `envelope` as stdout.
///
/// Injects `--max-sessions 0` (unlimited gate), `_CLR_DEFAULT_TIMEOUT=10` (short
/// watchdog for fast failure), and `claude_home` as `CLAUDE_HOME`.  Returns raw `Output`.
///
/// Fix(BUG-493): `expected_id` (and therefore the BUG-320 mismatch check) is now fed
/// exclusively by `--from`'s resolved source storage, never by raw `--session-dir` —
/// see `builder.rs`'s `session_exists`/`session_from_dir`.  Callers seed the expected
/// session via `make_session_for()` under an isolated `CLAUDE_HOME`, not a raw
/// `--session-dir` path.
fn run_with_fake_claude(
  envelope    : &str,
  claude_home : &std::path::Path,
  extra_args  : &[ &str ],
  extra_envs  : &[ ( &str, &str ) ],
) -> std::process::Output
{
  let body = format!( "printf '%s' '{envelope}'" );
  let ( _dir, path ) = fake_claude_dir( &body );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .args( extra_args )
    .env( "PATH", &path )
    .env( "CLAUDE_HOME", claude_home )
    .env( "_CLR_DEFAULT_TIMEOUT", "10" )
    .envs( extra_envs.iter().copied() )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_DIR" )
    .output()
    .expect( "spawn clr" )
}

// sv1 — matching UUIDs: no mismatch warning emitted
//
// `--from <src>` storage (under an isolated CLAUDE_HOME) contains UUID_A.jsonl →
// expected = UUID_A. Fake claude returns session_id = UUID_A → match.
// Expected: exit 0, stderr does NOT contain "session mismatch".
#[ test ]
fn sv1_matching_uuid_emits_no_warning()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/sv1-session-verification-src";
  let _ = make_session_for( claude_home.path(), src, UUID_A );
  let envelope = clr_envelope( UUID_A );

  let out = run_with_fake_claude(
    &envelope,
    claude_home.path(),
    &[
      "--from", src,
      "hello",
      "--max-sessions", "0",
      "--output-style", "raw",
    ],
    &[],
  );
  let err = stderr_str( &out );
  assert!(
    out.status.success(),
    "sv1: expected exit 0, got {}; stderr: {err}",
    out.status.code().unwrap_or( -1 )
  );
  assert!(
    !err.contains( "session mismatch" ),
    "sv1: no mismatch warning expected when UUIDs match; stderr:\n{err}"
  );
}

// sv2 — mismatched UUIDs: warning emitted, exit 0 (non-fatal)
//
// `--from <src>` storage contains UUID_A.jsonl → expected = UUID_A.
// Fake claude returns session_id = UUID_B → mismatch.
// Expected: exit 0, stderr contains "[Runner] warning: session mismatch".
#[ test ]
fn sv2_mismatched_uuid_emits_warning_but_exits_zero()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/sv2-session-verification-src";
  let _ = make_session_for( claude_home.path(), src, UUID_A );
  let envelope = clr_envelope( UUID_B );

  let out = run_with_fake_claude(
    &envelope,
    claude_home.path(),
    &[
      "--from", src,
      "hello",
      "--max-sessions", "0",
      "--output-style", "raw",
    ],
    &[],
  );
  let err = stderr_str( &out );
  assert!(
    out.status.success(),
    "sv2: mismatch must be non-fatal (exit 0); got {}; stderr: {err}",
    out.status.code().unwrap_or( -1 )
  );
  assert!(
    err.contains( "session mismatch" ),
    "sv2: expected '[Runner] warning: session mismatch' in stderr; got:\n{err}"
  );
  assert!(
    err.contains( UUID_A ),
    "sv2: warning must name the expected UUID ({UUID_A}); stderr:\n{err}"
  );
  assert!(
    err.contains( UUID_B ),
    "sv2: warning must name the actual UUID ({UUID_B}); stderr:\n{err}"
  );
}

// sv3 — `--new-session` set: no mismatch check (expected_id is None)
//
// `--from <src>` storage contains UUID_A.jsonl, but `--new-session` forces
// expected_id = None. Fake claude returns session_id = UUID_B.
// Expected: exit 0, NO mismatch warning (expected_id is None → check skipped).
#[ test ]
fn sv3_new_session_flag_skips_mismatch_check()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/sv3-session-verification-src";
  let _ = make_session_for( claude_home.path(), src, UUID_A );
  let envelope = clr_envelope( UUID_B );

  let out = run_with_fake_claude(
    &envelope,
    claude_home.path(),
    &[
      "--from", src,
      "hello",
      "--max-sessions", "0",
      "--new-session",
      "--output-style", "raw",
    ],
    &[],
  );
  let err = stderr_str( &out );
  assert!(
    out.status.success(),
    "sv3: expected exit 0; got {}; stderr: {err}",
    out.status.code().unwrap_or( -1 )
  );
  assert!(
    !err.contains( "session mismatch" ),
    "sv3: no mismatch warning expected with --new-session; stderr:\n{err}"
  );
}

// sv4 — empty session dir: no mismatch check (expected_id is None)
//
// `--from <src>` storage has no `.jsonl` files (never seeded) → session_exists()
// returns None → expected_id = None. Fake claude returns session_id = UUID_B.
// Expected: exit 0, NO mismatch warning.
#[ test ]
fn sv4_empty_session_dir_skips_mismatch_check()
{
  let claude_home = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/sv4-session-verification-src";
  let envelope = clr_envelope( UUID_B );

  let out = run_with_fake_claude(
    &envelope,
    claude_home.path(),
    &[
      "--from", src,
      "hello",
      "--max-sessions", "0",
      "--output-style", "raw",
    ],
    &[],
  );
  let err = stderr_str( &out );
  assert!(
    out.status.success(),
    "sv4: expected exit 0; got {}; stderr: {err}",
    out.status.code().unwrap_or( -1 )
  );
  assert!(
    !err.contains( "session mismatch" ),
    "sv4: no mismatch warning expected when session dir is empty; stderr:\n{err}"
  );
}
