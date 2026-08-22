//! Integration tests for the `clg .count` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/04_count.md`
//!
//! ## Coverage
//!
//! - INT-1: Default count returns project count
//! - INT-2: `target::sessions` with `project::` returns session count
//! - INT-3: `target::entries` with `project::` and `session::` returns entry count
//! - INT-4: Output is a single integer line
//! - INT-5: Exit code 0 on success
//! - INT-6: Exit code 1 on invalid target value
//! - INT-7: `target::sessions` with no `project::` counts all sessions
//! - INT-8: `target::entries` with no `session::` counts all entries in project
//! - T01: `target::projects` default (no `scope::`) regression guard
//! - T02: `target::projects scope::local` narrows below the global count
//! - T03: `target::sessions` default (no `scope::`) regression guard
//! - T04: `target::sessions scope::under` narrows to descendant projects' sessions
//! - T05: `issue-003a` cwd-shortcut is unaffected by `scope::` (even an invalid value)
//! - T06: `target::projects scope::bogus` rejected with the canonical error
//! - T07: `target::entries` ignores `scope::` (already fully scoped via `project::`)
//! - T08: default/global `target::projects` proven to use the fast `count_projects()`
//!   path (not silently rerouted through the resolver) via an unloadable directory
//!   entry only the fast path counts

mod common;

use tempfile::TempDir;




/// INT-1: Default count returns project count.
///
/// ## Purpose
/// Verify that `.count` with no parameters returns the total number of
/// projects as a bare integer.
///
/// ## Coverage
/// Output is integer 3; exit 0.
///
/// ## Validation Strategy
/// Write 3 projects into temp root. Run `clg .count`.
/// Assert output parses as integer and equals 3.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-1
#[ test ]
fn int_1_default_count_returns_project_count()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "cnt1-a" );
  let p2 = root.path().join( "cnt1-b" );
  let p3 = root.path().join( "cnt1-c" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );
  common::write_path_project_session( root.path(), &p3, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-1: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 3, "INT-1: expected project count 3; got {n}" );
}

/// INT-2: `target::sessions` with `project::` returns session count.
///
/// ## Purpose
/// Verify that `target::sessions ``project::alph``a` returns the count of sessions
/// in the specified project.
///
/// ## Coverage
/// Output is integer 4; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (path contains "alpha") with 4 sessions. Run
/// `clg .count ``target::sessions`` ``project::alph``a`. Assert output is 4.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-2
#[ test ]
fn int_2_target_sessions_with_project_returns_session_count()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &alpha, "s002", 2 );
  common::write_path_project_session( root.path(), &alpha, "s003", 2 );
  common::write_path_project_session( root.path(), &alpha, "s004", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-2: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 4, "INT-2: expected 4 sessions in project alpha; got {n}" );
}

/// INT-3: `target::entries` with `project::` and `session::` returns entry count.
///
/// ## Purpose
/// Verify that `target::entries ``project::alpha`` ``session::s``1` returns the
/// entry count for that specific session.
///
/// ## Coverage
/// Output is integer 7; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with session s1 (7 entries). Run `clg .count
/// ``target::entries`` ``project::alpha`` ``session::s``1`. Assert output is 7.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-3
#[ test ]
fn int_3_target_entries_with_project_and_session_returns_entry_count()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s1", 7 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( format!( "project::{enc}" ) )
    .arg( "session::s1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-3: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 7, "INT-3: expected 7 entries in session s1; got {n}" );
}

/// INT-4: Output is a single integer line.
///
/// ## Purpose
/// Verify that `.count` output is exactly `{n}\n` — one integer followed
/// by a newline, with nothing else.
///
/// ## Coverage
/// Trimmed output is exactly "2"; no extra text; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects. Run `clg .count`. Assert trimmed stdout == "2" exactly.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-4
#[ test ]
fn int_4_output_is_single_integer_line()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "cnt4-x" );
  let p2 = root.path().join( "cnt4-y" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let raw = common::stdout( &out );
  let trimmed = raw.trim();
  assert_eq!(
    trimmed,
    "2",
    "INT-4: .count output must be exactly '2\\n'; got: {raw:?}"
  );
}

/// INT-5: Exit code 0 on success.
///
/// ## Purpose
/// Verify that `.count` exits with code 0 on a valid fixture.
///
/// ## Coverage
/// Integer output on stdout; exit 0.
///
/// ## Validation Strategy
/// Write 1 project. Run `clg .count`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-5
#[ test ]
fn int_5_exit_code_0_on_success()
{
  let root = TempDir::new().unwrap();
  let p = root.path().join( "cnt5-proj" );
  common::write_path_project_session( root.path(), &p, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  assert!(
    s.parse::< usize >().is_ok(),
    "INT-5: .count must produce integer on stdout; got: '{s}'"
  );
}

/// INT-6: Exit code 1 on invalid target value.
///
/// ## Purpose
/// Verify that `target::widgets` (invalid) causes `.count` to fail with
/// exit code 1 and an error on stderr.
///
/// ## Coverage
/// Error on stderr; no count on stdout; exit 1.
///
/// ## Validation Strategy
/// Run `clg .count ``target::widget``s`. Assert exit 1 and stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-6
#[ test ]
fn int_6_exit_code_1_on_invalid_target_value()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::widgets" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  let err = common::stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-6: invalid target must produce error on stderr; got silence"
  );
}

/// INT-7: `target::sessions` with no `project::` counts all sessions.
///
/// ## Purpose
/// Verify that `target::sessions` without a `project::` restriction sums
/// sessions across all projects.
///
/// ## Coverage
/// Output is integer 6 (2 + 3 + 1); exit 0.
///
/// ## Validation Strategy
/// Write 3 projects with 2, 3, and 1 sessions. Run `clg .count ``target::session``s`.
/// Assert output is 6.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-7
#[ test ]
fn int_7_target_sessions_no_project_counts_all_sessions()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "cnt7-a" );
  let p2 = root.path().join( "cnt7-b" );
  let p3 = root.path().join( "cnt7-c" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p1, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );
  common::write_path_project_session( root.path(), &p3, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-7: .count target::sessions output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 6, "INT-7: expected 6 total sessions; got {n}" );
}

/// INT-8: `target::entries` with no `session::` counts all entries in project.
///
/// ## Purpose
/// Verify that `target::entries ``project::alph``a` without a `session::` filter
/// sums entries across all sessions in the project.
///
/// ## Coverage
/// Output is integer 8 (5 + 3); exit 0.
///
/// ## Validation Strategy
/// Write project alpha with 2 sessions: s1 (5 entries), s2 (3 entries).
/// Run `clg .count ``target::entries`` ``project::alph``a`. Assert output is 8.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-8
#[ test ]
fn int_8_target_entries_no_session_counts_all_entries_in_project()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s1", 5 );
  common::write_path_project_session( root.path(), &alpha, "s2", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-8: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 8, "INT-8: expected 8 total entries across s1+s2; got {n}" );
}

/// T01: `target::projects` default (no `scope::`) regression guard.
///
/// ## Purpose
/// Verify that `target::projects` with no `scope::` still counts every
/// project, matching pre-retrofit behavior (the `global` default fast path).
///
/// ## Coverage
/// Count equals the total written projects (3); exit 0.
///
/// ## Validation Strategy
/// Write 3 unrelated projects. Run `.count target::projects` with no
/// `scope::`. Assert the count is 3.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T01
#[ test ]
fn t01_target_projects_default_scope_global_regression_guard()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "t01alpha" );
  let p2 = root.path().join( "t01beta" );
  let p3 = root.path().join( "t01gamma" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );
  common::write_path_project_session( root.path(), &p3, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::projects" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "T01: .count target::projects output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 3, "T01: expected project count 3 at default scope; got {n}" );
}

/// T02: `target::projects scope::local` narrows below the global count.
///
/// ## Purpose
/// Verify `scope::local` limits the projects counted to the cwd's own
/// project, producing a strictly smaller count than the unscoped total in
/// the same storage fixture (AF1).
///
/// ## Coverage
/// Global count is 3; `scope::local` count is 1; `1 < 3`; both exit 0.
///
/// ## Validation Strategy
/// Write the cwd's own project (a real directory, required for
/// `Command::current_dir`) plus two unrelated projects, all in the same
/// storage root. Run `.count target::projects` (no `scope::`) and
/// `.count target::projects scope::local` from the cwd project's own
/// directory. Assert the local count is strictly less than the global count.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T02
#[ test ]
fn t02_target_projects_scope_local_narrows_below_global()
{
  let root       = TempDir::new().unwrap();
  let target_tmp = TempDir::new().unwrap();
  let target     = target_tmp.path().join( "t02targetmarker" );
  std::fs::create_dir_all( &target ).unwrap();

  common::write_path_project_session( root.path(), &target, "s001", 2 );

  let other1 = root.path().join( "t02other1" );
  let other2 = root.path().join( "t02other2" );
  common::write_path_project_session( root.path(), &other1, "s001", 2 );
  common::write_path_project_session( root.path(), &other2, "s001", 2 );

  let global_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::projects" )
    .output()
    .unwrap();

  let local_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &target )
    .arg( ".count" )
    .arg( "target::projects" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  common::assert_exit( &global_out, 0 );
  common::assert_exit( &local_out, 0 );

  let global_n : usize = common::stdout( &global_out ).trim().parse().unwrap_or_else( |_| panic!(
    "T02: global count must be a bare integer; got: '{}'", common::stdout( &global_out )
  ) );
  let local_n : usize = common::stdout( &local_out ).trim().parse().unwrap_or_else( |_| panic!(
    "T02: scope::local count must be a bare integer; got: '{}'", common::stdout( &local_out )
  ) );

  assert_eq!( global_n, 3, "T02: expected global project count 3; got {global_n}" );
  assert_eq!( local_n, 1, "T02: expected scope::local project count 1; got {local_n}" );
  assert!(
    local_n < global_n,
    "T02 (AF1): scope::local count ({local_n}) must be strictly less than the global count ({global_n}) in the same fixture"
  );
}

/// T03: `target::sessions` default (no `scope::`) regression guard.
///
/// ## Purpose
/// Verify that `target::sessions` with no `project::` and no `scope::`
/// still sums sessions across every project, matching pre-retrofit behavior.
///
/// ## Coverage
/// Count equals the total written sessions (2 + 3 = 5); exit 0.
///
/// ## Validation Strategy
/// Write 2 projects with 2 and 3 sessions. Run `.count target::sessions`
/// with no `scope::`. Assert the count is 5.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T03
#[ test ]
fn t03_target_sessions_default_scope_global_regression_guard()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "t03alpha" );
  let p2 = root.path().join( "t03beta" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p1, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "T03: .count target::sessions output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 5, "T03: expected total session count 5 at default scope; got {n}" );
}

/// T04: `target::sessions scope::under` narrows to descendant projects' sessions.
///
/// ## Purpose
/// Verify `scope::under` from an ancestor cwd sums sessions only across
/// descendant projects, excluding an unrelated project elsewhere in storage.
///
/// ## Coverage
/// Global count is 5 (2 + 3); `scope::under` count is 2 (nested project
/// only); `2 < 5`; both exit 0.
///
/// ## Validation Strategy
/// Write a 2-session project nested under an ancestor directory (the
/// ancestor itself is a real directory for `Command::current_dir`) plus a
/// 3-session unrelated project elsewhere. Run `.count target::sessions` (no
/// `scope::`) and `.count target::sessions scope::under` from the ancestor.
/// Assert the under-scoped count is strictly less than the global count.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T04
#[ test ]
fn t04_target_sessions_scope_under_narrows_to_descendants()
{
  let root       = TempDir::new().unwrap();
  let anchor_tmp = TempDir::new().unwrap();
  let anchor     = anchor_tmp.path().join( "t04anchor" );
  let nested     = anchor.join( "t04nestedchild" );
  std::fs::create_dir_all( &anchor ).unwrap();

  common::write_path_project_session( root.path(), &nested, "s001", 2 );
  common::write_path_project_session( root.path(), &nested, "s002", 2 );

  let other = root.path().join( "t04other" );
  common::write_path_project_session( root.path(), &other, "s001", 2 );
  common::write_path_project_session( root.path(), &other, "s002", 2 );
  common::write_path_project_session( root.path(), &other, "s003", 2 );

  let global_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  let under_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &anchor )
    .arg( ".count" )
    .arg( "target::sessions" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  common::assert_exit( &global_out, 0 );
  common::assert_exit( &under_out, 0 );

  let global_n : usize = common::stdout( &global_out ).trim().parse().unwrap_or_else( |_| panic!(
    "T04: global count must be a bare integer; got: '{}'", common::stdout( &global_out )
  ) );
  let under_n : usize = common::stdout( &under_out ).trim().parse().unwrap_or_else( |_| panic!(
    "T04: scope::under count must be a bare integer; got: '{}'", common::stdout( &under_out )
  ) );

  assert_eq!( global_n, 5, "T04: expected global session count 5; got {global_n}" );
  assert_eq!( under_n, 2, "T04: expected scope::under session count 2; got {under_n}" );
  assert!(
    under_n < global_n,
    "T04 (AF1): scope::under count ({under_n}) must be strictly less than the global count ({global_n}) in the same fixture"
  );
}

/// T05: `issue-003a` cwd-shortcut is unaffected by `scope::`, even an invalid value.
///
/// ## Purpose
/// Verify the cwd-shortcut (no `target::`, no `project::`) never reads
/// `scope::` at all — not even to validate it — confirming the shortcut is
/// fully exempt per the task's Out of Scope.
///
/// ## Coverage
/// Output identical with and without `scope::bogus`; both exit 0 (not 1 —
/// proving the invalid value was never validated).
///
/// ## Validation Strategy
/// Write the cwd's own project with 2 sessions. Run `.count` and
/// `.count scope::bogus` from that project's directory. Assert both exit 0
/// and produce byte-identical output.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T05
#[ test ]
fn t05_issue_003a_shortcut_unaffected_by_scope()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "t05project" );
  std::fs::create_dir_all( &proj ).unwrap();

  common::write_path_project_session( root.path(), &proj, "s001", 3 );
  common::write_path_project_session( root.path(), &proj, "s002", 2 );

  let no_scope_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".count" )
    .output()
    .unwrap();

  let with_bogus_scope_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".count" )
    .arg( "scope::bogus" )
    .output()
    .unwrap();

  common::assert_exit( &no_scope_out, 0 );
  common::assert_exit(
    &with_bogus_scope_out, 0
  );
  assert_eq!(
    no_scope_out.stdout, with_bogus_scope_out.stdout,
    "T05: issue-003a cwd-shortcut must ignore scope:: entirely, even an invalid value \
    (proves the shortcut never calls validate_scope())"
  );
}

/// T06: `target::projects scope::bogus` rejected with the canonical error.
///
/// ## Purpose
/// Verify invalid `scope::` values are rejected for `.count` the same way
/// as for `.projects`/`.show`/`.export`/`.search`/`.list` — one shared
/// validator, one canonical error.
///
/// ## Coverage
/// Exit 1; stderr contains the exact `validate_scope()` wording.
///
/// ## Validation Strategy
/// Run `.count target::projects scope::bogus`. Assert exit 1 and the
/// canonical error text.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T06
#[ test ]
fn t06_target_projects_scope_bogus_rejected_with_canonical_error()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::projects" )
    .arg( "scope::bogus" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  let err = common::stderr( &out );
  assert!(
    err.contains( "scope must be relevant|local|under|global|around, got bogus" ),
    "T06: scope::bogus must produce the canonical validate_scope() error; got: {err}"
  );
}

/// T07: `target::entries` ignores `scope::` (already fully scoped via `project::`).
///
/// ## Purpose
/// Verify `target::entries` (which requires an explicit `project::`) treats
/// a present, valid `scope::` value as a no-op — identical output whether
/// `scope::` is given or omitted.
///
/// ## Coverage
/// Output identical with and without `scope::under`; both exit 0.
///
/// ## Validation Strategy
/// Write a project with one 6-entry session. Run
/// `.count target::entries project::<id> session::s1` with and without
/// `scope::under`. Assert both produce the same count.
///
/// ## Related Requirements
/// `task/claude_storage/verified/517_count_scope_retrofit.md` — T07
#[ test ]
fn t07_target_entries_ignores_scope()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "t07alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s1", 6 );

  let no_scope_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( format!( "project::{enc}" ) )
    .arg( "session::s1" )
    .output()
    .unwrap();

  let with_scope_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( format!( "project::{enc}" ) )
    .arg( "session::s1" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  common::assert_exit( &no_scope_out, 0 );
  common::assert_exit( &with_scope_out, 0 );
  assert_eq!(
    no_scope_out.stdout, with_scope_out.stdout,
    "T07: target::entries must ignore scope:: entirely — identical output with and without it"
  );

  let s = common::stdout( &no_scope_out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "T07: .count target::entries output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 6, "T07: expected 6 entries in session s1; got {n}" );
}

/// T08: default (global) `target::projects` count is produced by the fast
/// `storage.count_projects()` path, not a silent reroute through the resolver.
///
/// ## Purpose
/// AF2's performance contract (the fast path is preserved at the default/
/// global scope) previously had no automated regression guard — the task's
/// own M2 measurement was a grep for `storage.count_projects()`'s call-site
/// text, which cannot tell whether that text is on the branch actually
/// executed at runtime. This test proves the contract by observable
/// behavior instead: `count_projects()` is a raw `fs::read_dir` + `is_dir()`
/// count with no `Project::load()` call, while the resolver path
/// (`resolve_scoped_projects()` → `list_projects()`) calls `Project::load()`
/// per entry and silently skips (via `eprintln!`, not an error) any
/// directory that fails to load.
///
/// ## Coverage
/// A directory literally named `-` fails `decode_path()`'s "encoded path is
/// empty after removing prefix" check, so `Project::load()` returns `Err`
/// for it. The fast path counts it anyway (no load attempted); the resolver
/// path silently drops it.
///
/// ## Validation Strategy
/// Write one real, loadable project plus a raw `-`-named directory directly
/// under `<root>/projects/`. Run `.count target::projects` (no `scope::`)
/// and assert count == 2 — this can only be true if the fast path is what
/// actually executes; a silent reroute to the resolver would drop the
/// unloadable directory and yield 1. Then run the same command with
/// `scope::local` and assert count == 1, confirming the contrast is real.
///
/// ## Related Requirements
/// `task/claude_storage/completed/517_count_scope_retrofit.md` — AF2, M2
#[ test ]
fn t08_default_scope_fast_path_counts_unloadable_dir_resolver_path_skips_it()
{
  let root       = TempDir::new().unwrap();
  let target_tmp = TempDir::new().unwrap();
  let target     = target_tmp.path().join( "t08target" );
  std::fs::create_dir_all( &target ).unwrap();

  common::write_path_project_session( root.path(), &target, "s001", 2 );

  // `is_dir()` finds this, but `Project::load()` cannot: `decode_path("-")`
  // fails its "empty after removing prefix" check.
  std::fs::create_dir_all( root.path().join( "projects" ).join( "-" ) ).unwrap();

  let global_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &target )
    .arg( ".count" )
    .arg( "target::projects" )
    .output()
    .unwrap();

  let local_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &target )
    .arg( ".count" )
    .arg( "target::projects" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  common::assert_exit( &global_out, 0 );
  common::assert_exit( &local_out, 0 );

  let global_n : usize = common::stdout( &global_out ).trim().parse().unwrap_or_else( |_| panic!(
    "T08: global count must be a bare integer; got: '{}'", common::stdout( &global_out )
  ) );
  let local_n : usize = common::stdout( &local_out ).trim().parse().unwrap_or_else( |_| panic!(
    "T08: scope::local count must be a bare integer; got: '{}'", common::stdout( &local_out )
  ) );

  assert_eq!(
    global_n, 2,
    "T08: default/global target::projects must use the fast count_projects() path, which \
     counts the unloadable '-' directory alongside the real project (raw fs::read_dir + \
     is_dir(), no Project::load()) — got {global_n}. A count of 1 here would mean the default \
     scope was silently rerouted through the resolver path, which drops directories \
     Project::load() can't parse."
  );
  assert_eq!(
    local_n, 1,
    "T08: scope::local must use the resolver path (list_projects()), which silently skips \
     the unloadable '-' directory and returns only the real project — got {local_n}."
  );
}
