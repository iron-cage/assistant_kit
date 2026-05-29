//! Integration tests for the `clg .show` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/03_show.md`
//!
//! ## Coverage
//!
//! - INT-1: No args shows current project's sessions
//! - INT-2: `session_id::` shows conversation content
//! - INT-3: `project::` selects explicit project
//! - INT-4: `session_id::` + `project::` shows session in named project
//! - INT-5: `show_metadata::1` suppresses content, shows metadata
//! - INT-6: `show_entries::1` shows all session entries
//! - INT-7: Exit code 2 when cwd has no project
//! - INT-8: `project::` with path-encoded ID

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// INT-1: No args shows current project's sessions.
///
/// ## Purpose
/// Verify that `.show` with no arguments uses the cwd to identify the
/// current project and lists its sessions.
///
/// ## Coverage
/// Session IDs for cwd-matched project appear; exit 0.
///
/// ## Validation Strategy
/// Write a path-encoded project whose path is the temp dir itself.
/// Run `.show` with `current_dir` set to that path and `CLAUDE_STORAGE_ROOT`
/// pointing to the fixture. Assert session ID appears in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-1
#[ test ]
fn int_1_no_args_shows_current_project_sessions()
{
  let root  = TempDir::new().unwrap();
  let cwd   = TempDir::new().unwrap();

  common::write_path_project_session(
    root.path(), cwd.path(), "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-1: .show with no args must produce output for cwd project; stderr: {}",
    stderr( &out )
  );
  assert!(
    s.contains( "-default_topic" ) || s.contains( "default_topic" ),
    "INT-1: session '-default_topic' must appear in .show output; got:\n{s}"
  );
}

/// INT-2: `session_id::` shows conversation content.
///
/// ## Purpose
/// Verify that `session_id::-default_topic` shows content or summary for
/// that specific session, including the session ID in output.
///
/// ## Coverage
/// Session ID visible in output; exit 0.
///
/// ## Validation Strategy
/// Write path-encoded project alpha with session `-default_topic` (4 entries).
/// Run `.show ``session_id::``-default_topic ``project::alph``a`. Assert session
/// ID appears in stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-2
#[ test ]
fn int_2_session_id_shows_conversation_content()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "show2-alpha" );
  let encoded = common::write_path_project_session(
    root.path(), &alpha, "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "-default_topic" ) || s.contains( "default_topic" ),
    "INT-2: session_id::-default_topic must appear in .show output; got:\n{s}"
  );
}

/// INT-3: `project::` selects explicit project.
///
/// ## Purpose
/// Verify that `project::alpha` shows sessions from alpha regardless of cwd,
/// without mixing in sessions from other projects.
///
/// ## Coverage
/// Alpha's session appears; beta's session absent; cwd is unrelated; exit 0.
///
/// ## Validation Strategy
/// Write projects alpha and beta. Run `.show ``project::alph``a` from a cwd that
/// matches neither. Assert alpha session appears and beta session absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-3
#[ test ]
fn int_3_project_param_selects_explicit_project()
{
  let root      = TempDir::new().unwrap();
  let alpha     = root.path().join( "show3-alpha" );
  let beta      = root.path().join( "show3-beta" );
  let alpha_enc = common::write_path_project_session(
    root.path(), &alpha, "alpha-sess", 2
  );
  common::write_path_project_session( root.path(), &beta, "beta-sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    // current_dir does not match any project
    .current_dir( root.path() )
    .arg( ".show" )
    .arg( format!( "project::{alpha_enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha-sess" ),
    "INT-3: alpha session must appear with project:: selector; got:\n{s}"
  );
  assert!(
    !s.contains( "beta-sess" ),
    "INT-3: beta session must be absent when project::alpha selected; got:\n{s}"
  );
}

/// INT-4: `session_id::` + `project::` shows session in named project.
///
/// ## Purpose
/// Verify that combining `session_id::` and `project::` resolves to the
/// session in the specified project, not a same-named session in another.
///
/// ## Coverage
/// Content from alpha's s1 shown; beta's s1 content absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (session s1, last message "alpha-content") and
/// project beta (session s1, last message "beta-content"). Run `.show
/// ``session_id::s1`` ``project::alph``a`. Assert alpha content present and
/// beta content absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-4
#[ test ]
fn int_4_session_id_and_project_show_session_in_named_project()
{
  let root      = TempDir::new().unwrap();
  let alpha     = root.path().join( "show4-alpha" );
  let beta      = root.path().join( "show4-beta" );
  let alpha_enc = common::write_path_project_session(
    root.path(), &alpha, "s1", 0
  );
  // Re-write alpha s1 with distinct last message
  common::write_test_session_with_last_message(
    root.path(), &alpha_enc, "s1", 0, "alpha-only-content"
  );
  let beta_enc = common::write_path_project_session(
    root.path(), &beta, "s1", 0
  );
  common::write_test_session_with_last_message(
    root.path(), &beta_enc, "s1", 0, "beta-only-content"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::s1" )
    .arg( format!( "project::{alpha_enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "s1" ),
    "INT-4: session s1 header must appear; got:\n{s}"
  );
}

/// INT-5: `show_metadata::1` suppresses content, shows metadata only.
///
/// ## Purpose
/// Verify that `show_metadata::1` shows metadata fields (entry count, type) but
/// omits actual message text from the session.
///
/// ## Coverage
/// Metadata present; message text absent; exit 0.
///
/// ## Validation Strategy
/// Write session `-default_topic` with known messages ("entry 0", "entry 1").
/// Run `.show ``session_id::``-default_topic ``show_metadata::1`` ``project::``...`.
/// Assert entry count info present but "entry 0" absent (suppressed by metadata mode).
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-5
#[ test ]
fn int_5_metadata_1_suppresses_content_shows_metadata()
{
  let root  = TempDir::new().unwrap();
  let p     = root.path().join( "show5-proj" );
  let enc   = common::write_path_project_session(
    root.path(), &p, "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "show_metadata::1" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // show_metadata::1 must produce output (metadata rows)
  assert!(
    !s.is_empty(),
    "INT-5: show_metadata::1 must produce output; stderr: {}",
    stderr( &out )
  );
  // actual entry text must be suppressed
  assert!(
    !s.contains( "entry 0" ),
    "INT-5: message text must be absent with show_metadata::1; got:\n{s}"
  );
}

/// INT-6: `show_entries::1` shows all session entries.
///
/// ## Purpose
/// Verify that `show_entries::1` shows all entries from a session including
/// both user and assistant message content.
///
/// ## Coverage
/// All 4 entries visible (user + assistant); exit 0.
///
/// ## Validation Strategy
/// Write session `-default_topic` with 4 entries (2 user, 2 assistant).
/// Run `.show ``session_id::``-default_topic ``show_entries::1`` ``project::``...`.
/// Assert multiple entries appear in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-6
#[ test ]
fn int_6_entries_1_shows_all_session_entries()
{
  let root  = TempDir::new().unwrap();
  let p     = root.path().join( "show6-proj" );
  let enc   = common::write_path_project_session(
    root.path(), &p, "-default_topic", 4
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "show_entries::1" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-6: show_entries::1 must show entry content; stderr: {}",
    stderr( &out )
  );
  // At least one entry's text must appear
  assert!(
    s.contains( "entry" ),
    "INT-6: entry content must appear with show_entries::1; got:\n{s}"
  );
}

/// INT-7: Exit code 2 when cwd has no project.
///
/// ## Purpose
/// Verify that `.show` with no args exits with code 2 and emits an error
/// when the cwd does not match any project in storage.
///
/// ## Coverage
/// Exit code 2; error on stderr; exit 0.
///
/// ## Validation Strategy
/// Use an empty storage root. Run `.show` from `/tmp` (no matching project).
/// Assert exit 2 and stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-7
#[ test ]
fn int_7_exit_code_2_when_cwd_has_no_project()
{
  let root = TempDir::new().unwrap();
  // Empty storage — no projects written

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-7: .show from unmatched cwd must emit error on stderr; got silence"
  );
}

/// INT-8: `project::` with path-encoded ID.
///
/// ## Purpose
/// Verify that supplying a raw path-encoded project ID to `project::` resolves
/// and lists sessions for that project.
///
/// ## Coverage
/// Session list for path-encoded project visible; exit 0.
///
/// ## Validation Strategy
/// Write a project whose path encodes to a known ID (e.g. a temp dir path).
/// Run `.show ``project::``{encoded_id}`. Assert session appears in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/03_show.md` — INT-8
#[ test ]
fn int_8_project_param_with_path_encoded_id()
{
  let root      = TempDir::new().unwrap();
  let proj_path = root.path().join( "show8-encoded" );
  let encoded   = common::write_path_project_session(
    root.path(), &proj_path, "enc-session", 2
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "enc-session" ) || s.contains( &encoded ),
    "INT-8: session for path-encoded project must appear; got:\n{s}"
  );
}
