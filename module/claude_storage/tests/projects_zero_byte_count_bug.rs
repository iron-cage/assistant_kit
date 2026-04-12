//! Bug-reproducer tests for issue-034: zero-byte session count/display mismatch in
//! `.projects` list mode.
//!
//! ## Background
//!
//! Claude Code creates a 0-byte `.jsonl` placeholder on startup before any entries are
//! written (B8 behavior invariant). These placeholder files must never inflate the
//! session count in list-mode headers or appear as the "best session" in summary mode.
//!
//! Three code sites are fixed:
//! 1. `aggregate_projects` — excludes zero-byte from best-session selection and count
//! 2. `projects_routine` `use_families` branch — `root_count` excludes zero-byte roots
//! 3. `projects_routine` flat branch — `group_count` derived from `displayable` (post-filter)

mod common;

use std::fs;
use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr
  );
}

/// Write a 0-byte JSONL placeholder under `<root>/projects/<project_id>/<session_id>.jsonl`.
///
/// Mirrors the Claude Code startup behavior (B8): a 0-byte file is created before
/// any entries are written to it.
///
/// # Panics
///
/// Panics if directory creation or file creation fails.
fn write_zero_byte_session(
  root       : &std::path::Path,
  project_id : &str,
  session_id : &str,
)
{
  let dir = root.join( "projects" ).join( project_id );
  fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  fs::File::create( &path ).expect( "create zero-byte session file" );
}

/// Write a 0-byte JSONL placeholder for a path-encoded project.
///
/// Returns the encoded project ID.
fn write_zero_byte_path_session(
  root         : &std::path::Path,
  project_path : &std::path::Path,
  session_id   : &str,
) -> String
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  write_zero_byte_session( root, &encoded, session_id );
  encoded
}

// ─────────────────────────────────────────────────────────────────────────────
// IT-54: use_families branch — zero-byte sessions excluded from header count
//
// Root Cause: `root_count` was computed as `families.iter().filter(|f| f.root.is_some()).count()`,
// which counted ALL families including those with a zero-byte root session. The
// render layer (`render_families_v1`) excluded zero-byte sessions from display,
// so a project with 1 zero-byte + 1 real session showed "(2 sessions)" in the header
// but only 1 line below it.
//
// Why Not Caught: All prior `.projects` tests used real (non-empty) sessions.
// No test combined a zero-byte placeholder with a real session in the same project
// and checked that the header count matched the displayed line count.
//
// Fix Applied: `root_count` now uses
// `f.root.as_ref().is_some_and(|s| !is_zero_byte_session(s))` to match the
// render layer's exclusion criteria exactly.
//
// Prevention: Whenever a count and a render filter co-exist, derive the count
// FROM the filtered source — never from the raw collection. Add a test combining
// zero-byte + real sessions whenever a new session-list display is introduced.
//
// Pitfall: The render layer and the count expression must stay in sync. If the
// render layer changes its zero-byte handling, the count expression must change too.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-034)
fn it54_use_families_zero_byte_excluded_from_header_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project_path = root.path().join( "proj54" );
  std::fs::create_dir_all( &project_path ).expect( "create project dir" );

  // One real session (has content) and one zero-byte placeholder.
  common::write_path_project_session( &storage_root, &project_path, "session-it54-real", 4 );
  write_zero_byte_path_session( &storage_root, &project_path, "session-it54-zero" );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".projects" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  // Header must reflect only the 1 displayable (non-zero-byte) session.
  assert!(
    s.contains( "(1 session)" ),
    "header must say '(1 session)' — zero-byte must not be counted; got:\n{s}"
  );
  assert!(
    !s.contains( "(2 session" ),
    "header must NOT say '(2 session...)' — that would include zero-byte; got:\n{s}"
  );

  // The real session must appear in the listing.
  assert!(
    s.contains( "session-it54-real" ),
    "real session must appear in output; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// IT-55: flat branch — zero-byte sessions excluded from header count
//
// Root Cause: In the flat display branch (`agent::` filter active), `group_count`
// was set to `sessions.len()` — the size of the unfiltered session list — before
// `displayable` was computed. `displayable` then excluded zero-byte non-agent
// sessions. The header used the pre-filter count while the rendered lines came
// from the post-filter list, creating a "(N sessions) / 0 lines" mismatch.
//
// Why Not Caught: All prior flat-branch tests used non-empty sessions. The
// agent:: filter path was never tested with zero-byte sessions in the project.
//
// Fix Applied: `displayable` is now computed before `group_count`, and
// `group_count = displayable.len()` so both the header and the render loop
// draw from the same filtered collection.
//
// Prevention: Always compute the count from the same slice used for rendering.
// Order matters: filter → count → render, never count → filter → render.
//
// Pitfall: `sessions.len()` looks correct but silently includes items the
// render loop will skip. Derive the count from the rendered collection, not
// the raw input.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-034)
fn it55_flat_branch_zero_byte_excluded_from_header_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project_path = root.path().join( "proj55" );
  std::fs::create_dir_all( &project_path ).expect( "create project dir" );

  // One real session and one zero-byte placeholder.
  common::write_path_project_session( &storage_root, &project_path, "session-it55-real", 4 );
  write_zero_byte_path_session( &storage_root, &project_path, "session-it55-zero" );

  // agent::0 activates flat display branch (agent_filter is Some — use_families = false).
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( "agent::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  // Header must reflect only the 1 non-agent, non-zero-byte session.
  assert!(
    s.contains( "(1 session)" ),
    "flat-branch header must say '(1 session)' — zero-byte must not be counted; got:\n{s}"
  );
  assert!(
    !s.contains( "(2 session" ),
    "flat-branch header must NOT say '(2 session...)'; got:\n{s}"
  );

  // The real session must appear.
  assert!(
    s.contains( "session-it55-real" ),
    "real session must appear in flat output; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// IT-56: zero-byte-only project excluded from list-mode output
//
// Root Cause: `aggregate_projects` iterated all sessions including zero-byte
// placeholders when selecting the "best" session. When all sessions in a project
// are zero-byte, the best-selection iterator now returns None → `continue` →
// no ProjectSummary is created. This is the correct behaviour: a project with
// only empty placeholder files has no content to show.
//
// Why Not Caught: No test created a project whose ONLY sessions are zero-byte
// and verified that it does not appear in list-mode output.
//
// Fix Applied: `.filter(|(_, s)| !is_zero_byte_session(s))` added to the best-
// selection iterator in `aggregate_projects`. The `continue` branch already
// existed for no-mtime sessions; zero-byte-only projects now hit the same path.
//
// Prevention: Test the boundary case where a project's entire session set
// consists of placeholder files. Verify the project does not appear in output.
//
// Pitfall: A project with ONLY zero-byte sessions should be fully invisible in
// list mode. Do not add a "(0 sessions)" fallback row — that would leak
// placeholder state into the UI.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-034)
fn it56_zero_byte_only_project_excluded_from_list_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Project A: only zero-byte sessions (should not appear)
  let proj_zero = root.path().join( "proj56_zero" );
  std::fs::create_dir_all( &proj_zero ).expect( "create proj56_zero dir" );
  write_zero_byte_path_session( &storage_root, &proj_zero, "session-it56-z1" );
  write_zero_byte_path_session( &storage_root, &proj_zero, "session-it56-z2" );

  // Project B: one real session (must appear)
  let proj_real = root.path().join( "proj56_real" );
  std::fs::create_dir_all( &proj_real ).expect( "create proj56_real dir" );
  common::write_path_project_session( &storage_root, &proj_real, "session-it56-real", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  // proj56_real must appear with "Found 1 project" (only real project visible)
  assert!(
    s.contains( "Found 1 project" ),
    "only the real project must be listed; got:\n{s}"
  );
  assert!(
    s.contains( "session-it56-real" ),
    "real session must appear in output; got:\n{s}"
  );

  // proj56_zero must not produce any output row
  assert!(
    !s.contains( "proj56_zero" ),
    "zero-byte-only project must NOT appear in list output; got:\n{s}"
  );
}
