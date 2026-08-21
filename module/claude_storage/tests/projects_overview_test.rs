//! Tests for the terse `.projects` overview — flat recency table and tree.
//!
//! Covers `src/cli/projects_overview.rs`, reached whenever `detail::projects`
//! is in effect — which, since the default flip, is every bare `.projects`
//! invocation. The full session listing (`detail::sessions`) is covered by
//! `cli_cmd_projects_test.rs` and friends; only the guard case OV-9 touches it
//! here, to prove the terse renderer did not leak into that path.
//!
//! | ID    | What it covers                                                    |
//! |-------|-------------------------------------------------------------------|
//! | OV-1  | Bare `.projects` renders the terse overview, not session listings |
//! | OV-2  | Flat layout emits the LAST/CONV/AGENTS/PROJECT header             |
//! | OV-3  | Zero agents render as `·`, non-zero as `N ag`                     |
//! | OV-4  | Summary line uses singular nouns at a count of one                |
//! | OV-5  | A project whose decoded path is absent carries `⚠ gone`           |
//! | OV-6  | The project matching the process cwd carries the `▸` gutter       |
//! | OV-7  | `show_tree::1` nests projects by directory with tree connectors   |
//! | OV-8  | Empty storage renders the summary line alone, no header row       |
//! | OV-9  | `detail::sessions` still renders the full listing unchanged       |
//! | OV-10 | Full project paths are printed, never factored to a shared prefix |
//! | OV-11 | The tree layout marks an absent decoded path `⚠ gone` too         |
//! | OV-12 | A single-child directory run collapses into one tree node         |

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

// ─── OV-1 ─────────────────────────────────────────────────────────────────────

/// OV-1: Bare `.projects` renders the terse overview, not session listings.
///
/// ## Purpose
/// Pin the default flip. Before it, `detail::sessions` was the default purely to
/// preserve the behavior of the deprecated `.list` command through its
/// absorption into `.projects`; the terse overview is now the primary view.
///
/// ## Coverage
/// Summary line present; session ids, agent brackets, and the old
/// `Found N projects:` header all absent.
#[ test ]
fn ov_1_bare_projects_renders_terse_overview()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "ov1-proj" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-ov1", &[ ( "agent-ov1", "general-purpose" ) ], 2
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "1 project" ), "must show the summary line; got:\n{s}" );
  assert!( !s.contains( "Found 1 project:" ), "must not show the sessions-mode header; got:\n{s}" );
  assert!( !s.contains( "root-ov1" ), "must not list root session ids; got:\n{s}" );
  assert!( !s.contains( "agent-ov1" ), "must not list agent session ids; got:\n{s}" );
}

// ─── OV-2 ─────────────────────────────────────────────────────────────────────

/// OV-2: Flat layout emits the LAST/CONV/AGENTS/PROJECT header.
///
/// ## Purpose
/// The column header is what makes a bare count like `2 conv` readable. It must
/// appear exactly once, above the rows, and only when there is at least one row.
///
/// ## Coverage
/// All four column names present; header precedes every project row.
#[ test ]
fn ov_2_flat_layout_emits_column_header()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "ov2-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ov2", 3 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  for column in [ "LAST", "CONV", "AGENTS", "PROJECT" ]
  {
    assert!( s.contains( column ), "must show the {column} column header; got:\n{s}" );
  }

  let header_line = s.lines().position( | l | l.contains( "PROJECT" ) )
    .expect( "header line must exist" );
  let row_line = s.lines().position( | l | l.contains( "ov2-proj" ) )
    .expect( "project row must exist" );
  assert!( header_line < row_line, "header must precede project rows; got:\n{s}" );
}

// ─── OV-3 ─────────────────────────────────────────────────────────────────────

/// OV-3: Zero agents render as `·`, non-zero as `N ag`.
///
/// ## Purpose
/// A column of `0`s is visual noise in a list where most projects never spawn an
/// agent. The middot keeps the column narrow and makes non-zero values pop.
///
/// ## Coverage
/// Agentless project shows `·`; project with one agent shows `1 ag`.
#[ test ]
fn ov_3_zero_agents_render_as_middot()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let plain = root.path().join( "ov3-plain" );
  std::fs::create_dir_all( &plain ).unwrap();
  common::write_path_project_session( &storage_root, &plain, "session-ov3", 2 );

  let withagent = root.path().join( "ov3-agent" );
  std::fs::create_dir_all( &withagent ).unwrap();
  let encoded = claude_storage_core::encode_path( &withagent ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-ov3", &[ ( "agent-ov3", "general-purpose" ) ], 2
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( '·' ), "agentless project must show the middot placeholder; got:\n{s}" );
  assert!( s.contains( "1 ag" ), "project with one agent must show `1 ag`; got:\n{s}" );
  assert!( !s.contains( " 0 ag" ), "zero must never render as `0 ag`; got:\n{s}" );
}

// ─── OV-4 ─────────────────────────────────────────────────────────────────────

/// OV-4: Summary line uses singular nouns at a count of one.
///
/// ## Purpose
/// `1 projects · 1 conversations` is the classic pluralization bug. The agents
/// segment is additionally omitted entirely at zero rather than shown as
/// `0 agents`.
///
/// ## Coverage
/// `1 project` and `1 conversation` singular; no `agents` segment.
#[ test ]
fn ov_4_summary_line_uses_singular_nouns()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "ov4-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ov4", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let summary = s.lines().next().expect( "summary line must exist" );
  assert!( summary.contains( "1 project ·" ), "project noun must be singular; got: {summary}" );
  assert!( summary.contains( "1 conversation" ), "conversation noun must be singular; got: {summary}" );
  assert!( !summary.contains( "1 projects" ), "must not pluralize at one; got: {summary}" );
  assert!( !summary.contains( "1 conversations" ), "must not pluralize at one; got: {summary}" );
  assert!( !summary.contains( "agent" ), "agents segment must be omitted at zero; got: {summary}" );
}

// ─── OV-5 ─────────────────────────────────────────────────────────────────────

/// OV-5: A project whose decoded path is absent carries `⚠ gone`.
///
/// ## Purpose
/// Path encoding is lossy — `/` and `_` and `.` all collapse to `-`, so a
/// decoded path is only trustworthy while the directory it names still exists to
/// disambiguate it. Once deleted, the rendered path is a guess and must be
/// labelled as one rather than presented as fact.
///
/// ## Coverage
/// Storage entry for a never-created directory renders with the marker; the path
/// itself is still shown, since it is the only identifier the storage key holds.
#[ test ]
fn ov_5_absent_decoded_path_marked_gone()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Deliberately NOT created on disk — this is the deleted-scratch-directory case.
  let vanished = root.path().join( "ov5-vanished" );
  common::write_path_project_session( &storage_root, &vanished, "session-ov5", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "⚠ gone" ), "absent decoded path must be marked; got:\n{s}" );
  assert!( s.contains( "ov5" ), "path must still be shown alongside the marker; got:\n{s}" );
}

// ─── OV-6 ─────────────────────────────────────────────────────────────────────

/// OV-6: The project matching the process cwd carries the `▸` gutter.
///
/// ## Purpose
/// "Project" here means cwd bucket, so "which of these is where I am right now"
/// is the single most common question the list has to answer.
///
/// ## Coverage
/// Marker present when cwd is inside a listed project; absent when it is not.
#[ test ]
fn ov_6_cwd_project_carries_gutter_marker()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "ov6-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ov6", 2 );

  let out_inside = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  let out_outside = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( root.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out_inside, 0 );
  assert_exit( &out_outside, 0 );
  assert!(
    stdout( &out_inside ).contains( '▸' ),
    "cwd project must carry the gutter marker; got:\n{}", stdout( &out_inside )
  );
  assert!(
    !stdout( &out_outside ).contains( '▸' ),
    "no marker when cwd matches no listed project; got:\n{}", stdout( &out_outside )
  );
}

// ─── OV-7 ─────────────────────────────────────────────────────────────────────

/// OV-7: `show_tree::1` nests projects by directory with tree connectors.
///
/// ## Purpose
/// Sibling projects under one parent are the norm — a repo checked out once but
/// entered from several subdirectories becomes several projects. The tree makes
/// that structure visible; the flat table cannot.
///
/// ## Coverage
/// Connectors drawn; both sibling leaf names present; the shared parent segment
/// appears once as a node rather than once per row.
#[ test ]
fn ov_7_show_tree_nests_projects_by_directory()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let parent = root.path().join( "ov7-parent" );
  let alpha = parent.join( "alpha" );
  let beta = parent.join( "beta" );
  std::fs::create_dir_all( &alpha ).unwrap();
  std::fs::create_dir_all( &beta ).unwrap();
  common::write_path_project_session( &storage_root, &alpha, "session-ov7-a", 2 );
  common::write_path_project_session( &storage_root, &beta, "session-ov7-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( '├' ) || s.contains( '└' ),
    "tree layout must draw connectors; got:\n{s}"
  );
  assert!( s.contains( "alpha" ), "must show the alpha leaf; got:\n{s}" );
  assert!( s.contains( "beta" ), "must show the beta leaf; got:\n{s}" );
  assert_eq!(
    s.matches( "ov7-parent" ).count(), 1,
    "shared parent must appear once as a node, not repeated per row; got:\n{s}"
  );
}

// ─── OV-8 ─────────────────────────────────────────────────────────────────────

/// OV-8: Empty storage renders the summary line alone, no header row.
///
/// ## Purpose
/// A column header over zero rows is a phantom table. The summary line alone is
/// the honest rendering of nothing.
///
/// ## Coverage
/// Zero counts shown; no LAST/PROJECT header; exit 0, not an error.
#[ test ]
fn ov_8_empty_storage_renders_summary_only()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "0 projects" ), "must report zero projects; got:\n{s}" );
  assert!( !s.contains( "PROJECT" ), "must not draw a header over zero rows; got:\n{s}" );
  assert!( !s.contains( "LAST" ), "must not draw a header over zero rows; got:\n{s}" );
}

// ─── OV-9 ─────────────────────────────────────────────────────────────────────

/// OV-9: `detail::sessions` still renders the full listing unchanged.
///
/// ## Purpose
/// Regression guard on the other side of the default flip — the terse renderer
/// must not leak into the session-detail path. This is the only test here that
/// exercises `detail::sessions`.
///
/// ## Coverage
/// Sessions-mode header present; session id listed; no terse summary line or
/// column header.
#[ test ]
fn ov_9_detail_sessions_renders_full_listing()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "ov9-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ov9", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "detail::sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found 1 project:" ), "sessions mode keeps its header; got:\n{s}" );
  assert!( s.contains( "session-ov9" ), "sessions mode lists session ids; got:\n{s}" );
  assert!( !s.contains( "PROJECT" ), "terse column header must not leak in; got:\n{s}" );
  assert!( !s.contains( " · " ), "terse summary line must not leak in; got:\n{s}" );
}

// ─── OV-10 ────────────────────────────────────────────────────────────────────

/// OV-10: Full project paths are printed, never factored to a shared prefix.
///
/// ## Purpose
/// A project path is the command's primary output: it gets copied into a `cd`,
/// piped into `grep`, pasted into a `project::` argument. Factoring rows against
/// a common base to save horizontal space would break all three. Prefix
/// factoring is `show_tree::1`'s job, where nesting carries the shared segment
/// without truncating any row.
///
/// ## Coverage
/// Every row in the flat layout contains the full ancestor path, even when all
/// rows share it.
#[ test ]
fn ov_10_flat_layout_prints_full_paths()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let shared = root.path().join( "ov10-shared" );
  let alpha = shared.join( "alpha" );
  let beta = shared.join( "beta" );
  std::fs::create_dir_all( &alpha ).unwrap();
  std::fs::create_dir_all( &beta ).unwrap();
  common::write_path_project_session( &storage_root, &alpha, "session-ov10-a", 2 );
  common::write_path_project_session( &storage_root, &beta, "session-ov10-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert_eq!(
    s.matches( "ov10-shared" ).count(), 2,
    "both rows must carry the shared ancestor in full; got:\n{s}"
  );
  assert!( s.contains( "ov10-shared/alpha" ), "alpha row must show its full path; got:\n{s}" );
  assert!( s.contains( "ov10-shared/beta" ), "beta row must show its full path; got:\n{s}" );
}

// ─── OV-11 ────────────────────────────────────────────────────────────────────

/// OV-11: The tree layout marks an absent decoded path `⚠ gone` too.
///
/// ## Purpose
/// `render_tree` computes the marker independently of `render_flat` — it resolves
/// each node back to its row rather than iterating rows directly. OV-5 pins the
/// flat path only, so the tree branch could regress without failing any test.
/// The guess-versus-fact distinction matters identically in both layouts.
///
/// ## Coverage
/// Two siblings under one parent, one of them never created on disk: the tree
/// draws connectors, marks only the absent sibling, and leaves the live one clean.
///
/// The fixture root is built with an explicit dot-free prefix rather than
/// `TempDir::new()`. `encode_path` collapses `.` to `-` exactly as it does `/`,
/// so under the default `.tmpXXXX` root the absent sibling has no directory left
/// on disk to disambiguate against and decodes to a mangled flat name — which
/// then sorts nowhere near its live sibling and the tree never branches. That is
/// real decoder behavior, not a rendering defect, so the fixture avoids the
/// ambiguity instead of asserting around it.
#[ test ]
fn ov_11_tree_layout_marks_absent_decoded_path_gone()
{
  let root = tempfile::Builder::new().prefix( "ov11" ).tempdir().unwrap();
  let storage_root = root.path().join( ".claude" );
  let parent = root.path().join( "ov11-parent" );
  let live = parent.join( "live" );
  // Deliberately NOT created on disk — the sibling that must carry the marker.
  let vanished = parent.join( "vanished" );
  std::fs::create_dir_all( &live ).unwrap();
  common::write_path_project_session( &storage_root, &live, "session-ov11-a", 2 );
  common::write_path_project_session( &storage_root, &vanished, "session-ov11-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( '├' ) || s.contains( '└' ),
    "siblings must render as a tree, else this asserts nothing about render_tree; got:\n{s}"
  );
  let marked : Vec< &str > = s.lines().filter( | l | l.contains( "⚠ gone" ) ).collect();
  assert_eq!( marked.len(), 1, "exactly the absent sibling must be marked; got:\n{s}" );
  assert!( marked[ 0 ].contains( "vanished" ), "the marked row must be the absent one; got:\n{s}" );
}

// ─── OV-12 ────────────────────────────────────────────────────────────────────

/// OV-12: A single-child directory run collapses into one tree node.
///
/// ## Purpose
/// Without `collapse`, a project buried N directories deep draws N nested levels
/// carrying no information — the tree becomes taller than the flat table it was
/// meant to compress. OV-7 proves nesting happens at a branch point but never
/// exercises a run with nothing to branch on.
///
/// ## Coverage
/// One project under a three-deep single-child chain: the chain's segments appear
/// on a single line, and no connector is drawn for the intermediate directories.
#[ test ]
fn ov_12_single_child_chain_collapses_to_one_node()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let deep = root.path().join( "ov12-a" ).join( "b" ).join( "c" ).join( "leaf" );
  std::fs::create_dir_all( &deep ).unwrap();
  common::write_path_project_session( &storage_root, &deep, "session-ov12", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let chain : Vec< &str > = s.lines().filter( | l | l.contains( "ov12-a" ) ).collect();
  assert_eq!( chain.len(), 1, "the whole single-child run must occupy one line; got:\n{s}" );
  assert!(
    chain[ 0 ].contains( "ov12-a/b/c/leaf" ),
    "collapsed label must carry every segment of the run; got:\n{s}"
  );
  assert!(
    !s.contains( '├' ) && !s.contains( '└' ),
    "a collapsed chain has no branch point, so no connector may be drawn; got:\n{s}"
  );
}
