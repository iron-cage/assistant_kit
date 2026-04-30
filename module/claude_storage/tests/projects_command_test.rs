//! Tests for `.projects` command — filter, validation, and output formatting.
//!
//! ## Coverage
//!
//! Parameter validation, filter behavior, and output formatting:
//! - Verbosity level output (`v::0` no header, `v::2` project path header)
//! - Session, agent, and `min_entries` filters
//! - Invalid parameter rejection (verbosity, `min_entries`, agent out of range)
//! - Singular/plural noun formatting in "Found N projects:" header (IT-14..IT-16)
//! - Header uses "conversations" not "sessions" (IT-50)
//!
//! ## Related Files
//!
//! - `projects_edge_case_test.rs` — EC-1..EC-9 scope parameter acceptance/rejection
//! - `projects_scope_test.rs` — scope behavioral semantics (which sessions are returned)
//! - `projects_path_encoding_test.rs` — path decode/display bug reproducers (IT-23..IT-26)
//! - `projects_family_display_test.rs` — family and agent session display (IT-1, IT-33, IT-36..IT-48)
//! - `projects_output_format_test.rs` — output format: path headers, agent collapse (IT-17..IT-29)
//! - `projects_scope_around_test.rs` — `scope::around` neighborhood semantics (IT-57..IT-59)
//! - `projects_zero_byte_count_bug.rs` — zero-byte session exclusion (IT-54..IT-56)

mod common;

use tempfile::TempDir;

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: verbosity::0 → no header, just project paths
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn verbosity_zero_no_header()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-v0-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( !s.contains( "Found" ),  "verbosity::0 must not emit 'Found N projects' header; got:\n{s}" );
  assert!( s.contains( "proj" ),    "verbosity::0 must output project path; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: session:: filter narrows results
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn session_filter_narrows_results()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // two sessions in the same project
  common::write_path_project_session( &storage_root, &project, "session-keep-001", 2 );
  common::write_path_project_session( &storage_root, &project, "session-drop-002", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "session::keep" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-keep-001" ),  "must contain matching session; got:\n{s}" );
  assert!( !s.contains( "session-drop-002" ), "must exclude non-matching session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Validation: verbosity out of range → exit 1
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn invalid_verbosity_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" ).arg( "verbosity::99" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "verbosity" ),
    "error must mention verbosity; got: {}",
    stderr( &out )
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Validation: min_entries negative → exit 1
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn invalid_min_entries_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" ).arg( "min_entries::-1" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "min_entries" ),
    "error must mention min_entries; got: {}",
    stderr( &out )
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: agent::1 returns only agent sessions, agent::0 excludes them
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn agent_filter_includes_only_agent_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  common::write_path_project_session( &storage_root, &project, "session-main", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-001", 2 );

  // agent::1 → only agent session
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "agent::1" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "agent-task-001" ), "agent::1 must include agent session; got:\n{s}" );
  assert!( !s.contains( "session-main" ), "agent::1 must exclude main session; got:\n{s}" );
}

#[test]
fn agent_filter_excludes_agent_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  common::write_path_project_session( &storage_root, &project, "session-main", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-002", 2 );

  // agent::0 → only main sessions
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "agent::0" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-main" ), "agent::0 must include main session; got:\n{s}" );
  assert!( !s.contains( "agent-task-002" ), "agent::0 must exclude agent session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: min_entries:: filters by actual entry count
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn min_entries_filters_by_entry_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // 2-entry session and 6-entry session
  common::write_path_project_session( &storage_root, &project, "session-short", 2 );
  common::write_path_project_session( &storage_root, &project, "session-long", 6 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "min_entries::3" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-long" ), "min_entries::3 must include 6-entry session; got:\n{s}" );
  assert!( !s.contains( "session-short" ), "min_entries::3 must exclude 2-entry session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: verbosity::2 shows project path header (grouped format)
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn verbosity_two_includes_project_label()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-v2-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ), "verbosity::2 must emit 'Found N sessions' header; got:\n{s}" );
  assert!(
    s.lines().any( | l | l.contains( ':' ) && ( l.contains( '/' ) || l.contains( '~' ) ) ),
    "verbosity::2 must show project path header; got:\n{s}"
  );
  assert!( s.contains( "session-v2-test" ), "must list session ID; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// issue-025 regression: "Found 1 sessions:" uses wrong plural — must be
// "Found 1 session:" (singular).
//
// Root Cause: projects_routine always formats the count noun as "sessions"
// regardless of count. English grammar requires singular ("session") when
// count == 1.
//
// Why Not Caught: No existing test asserted the exact singular/plural form of
// the "Found N sessions:" header — only that the word "Found" was present.
//
// Fix Applied: Derive the noun ("session" vs "sessions") based on `rows.len()`
// before formatting the header, and use the derived noun in the format string.
//
// Prevention: Always add an exact-string assertion for count-bearing output
// when writing tests, not just a contains("Found") check.
//
// Pitfall: "Found 0 sessions:" should remain plural ("sessions"), consistent
// with English grammar where zero takes plural form.
// ─────────────────────────────────────────────────────────────────────────────

// IT-14: singular noun when exactly 1 session found
//
// bug_reproducer(issue-025)
#[test]
fn output_uses_singular_noun_when_exactly_one_session_found()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // Exactly one session
  common::write_path_project_session( &storage_root, &project, "session-singular-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 1 project:" ),
    "with 1 result, header must use singular 'project' (not 'projects'); got:\n{s}"
  );
  assert!(
    !s.contains( "Found 1 projects:" ),
    "with 1 result, header must NOT use plural 'projects'; got:\n{s}"
  );
  assert!( s.contains( "session-singular-test" ), "must list the session ID; got:\n{s}" );
}

// IT-15: plural noun when 2 or more sessions found
//
// bug_reproducer(issue-025)
#[test]
fn output_uses_plural_noun_when_multiple_sessions_found()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project_a = root.path().join( "proj-a" );
  let project_b = root.path().join( "proj-b" );

  // Two sessions in two distinct project directories = two projects
  common::write_path_project_session( &storage_root, &project_a, "session-plural-a", 2 );
  common::write_path_project_session( &storage_root, &project_b, "session-plural-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 2 projects:" ),
    "with 2 distinct projects, header must use plural 'projects'; got:\n{s}"
  );
}

// IT-16: zero sessions header still uses plural ("Found 0 sessions:")
//
// bug_reproducer(issue-025)
#[test]
fn output_uses_plural_noun_when_zero_sessions_found()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // No sessions at all (empty storage)
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 0 projects:" ),
    "with 0 results, header must use plural 'projects' (zero takes plural in English); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation: verbosity::-1 (negative) → exit 1
//
// ## Purpose
// Validates that the verbosity parameter lower bound is enforced. The valid
// range is 0–5; negative values must be rejected with a clear error message.
//
// ## Coverage
// Boundary: verbosity below minimum (< 0). Complements the existing
// `invalid_verbosity_rejected` test which only checks the upper bound (99).
//
// ## Validation Strategy
// Assert exit code 1 and that stderr mentions "verbosity" so the user knows
// which parameter caused the error.
//
// ## Related Requirements
// Same validation contract as `status_routine`, `search_routine`, etc.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn verbosity_negative_one_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" ).arg( "verbosity::-1" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "verbosity" ),
    "error must mention verbosity; got: {}",
    stderr( &out )
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation: agent::2 (out of range) → exit 1
//
// ## Purpose
// Validates that the agent parameter only accepts boolean values (0 or 1).
// Values outside that range must be rejected with a descriptive error.
//
// ## Coverage
// Boolean validation: value > 1. Complements EC-6 (scope validation) and
// `invalid_min_entries_rejected` (numeric validation).
//
// ## Validation Strategy
// Assert exit code 1. The error is produced by the unilang boolean parser
// before projects_routine is entered.
//
// ## Related Requirements
// `agent::` is documented as accepting 0 or 1 only.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn agent_value_out_of_range_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" ).arg( "agent::2" )
    .output().unwrap();
  assert_exit( &out, 1 );
}

// ─────────────────────────────────────────────────────────────────────────────
// Behavioral: min_entries::0 includes all sessions (no lower bound)
//
// ## Purpose
// Confirms that min_entries::0 is treated as "no minimum" and returns all
// sessions regardless of entry count. This is the zero-value boundary case.
//
// ## Coverage
// Boundary: min_entries == 0 includes sessions with any entry count, including
// 1-entry sessions. Complements `min_entries_filters_by_entry_count` which
// tests min_entries::3 with sessions of 2 and 6 entries.
//
// ## Validation Strategy
// Create two sessions (1-entry and 4-entry). Assert both appear in output
// when min_entries::0 is used, since 1 >= 0 and 4 >= 0.
//
// ## Related Requirements
// Consistent with standard "minimum N means N or more" semantics.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn min_entries_zero_includes_all_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  common::write_path_project_session( &storage_root, &project, "session-one-entry",  1 );
  common::write_path_project_session( &storage_root, &project, "session-four-entry", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "min_entries::0" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-one-entry" ),  "min_entries::0 must include 1-entry session; got:\n{s}" );
  assert!( s.contains( "session-four-entry" ), "min_entries::0 must include 4-entry session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-50: Project header always says "conversations", never "sessions"
// ────────────────────────────────────────────────────────────────────────────
/// IT-50: Project headers must always use "conversations" as the user-facing noun.
///
/// "sessions" is a storage-layer implementation detail invisible to users.
/// This test creates a project with sessions but no agents and verifies the
/// header reads "(N conversations)" not "(N sessions)".
#[ test ]
fn it_header_uses_conversations_not_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.contains( "sessions)" ),
    "Project header must not contain 'sessions)' — must say 'conversations)'\nOutput: {s}",
  );
  assert!(
    s.contains( "conversations)" ),
    "Project header must contain 'conversations)'\nOutput: {s}",
  );
}
