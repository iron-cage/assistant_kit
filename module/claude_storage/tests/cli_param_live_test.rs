//! Edge case tests for the `live::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/44_live.md`
//! - Parameter: `docs/cli/param/44_live.md`
//! - Algorithm: `docs/algorithm/002_session_liveness.md`
//!
//! ## Coverage
//!
//! - EC-1: `live::0` lists every project when none is live
//! - EC-2: `live::0` and an unset `live::` agree when nothing is live
//! - EC-3: `live::1` never lists a project with no attached process
//! - EC-4: Non-boolean value rejected
//! - EC-5: Out-of-range boolean value rejected
//! - EC-6: `STATUS` column absent when no row is live
//! - EC-7: `detail::sessions` carries no state tag when no row is live
//! - EC-8: `live::` composes with `filter::`
//! - EC-9: `ids::1 live::0` passes a non-live project through
//! - EC-10: `ids::1 live::1` withholds a non-live project's ids
//! - EC-11: the tree layout carries the same liveness affordance as the flat one
//!
//! ## Why these assert on the negative
//!
//! Liveness is inferred from the real process table — there is no injection
//! point through the CLI boundary, and a fixture cannot conjure an attached
//! Claude Code process whose cwd is a freshly-created temp directory. What a
//! black-box test *can* pin is the half of the contract that holds regardless
//! of what is running on the host: a fixture project is never live, so every
//! liveness-derived affordance must be absent for it. The positive half — the
//! `/proc` walk, the history join, the working/waiting split — is covered by
//! `src/cli/liveness.rs`'s own unit tests, which build a real `/proc`-shaped
//! tree and can therefore assert on presence.
#![ cfg( unix ) ]

mod common;

use tempfile::TempDir;





/// EC-1: `live::0` lists every project when none is live.
///
/// ## Purpose
/// `live::0` is the inverse filter, not a no-op alias for "hide everything".
/// A fixture project can never carry an attached process, so `live::0` must
/// pass all of them through.
///
/// ## Coverage
/// Exit 0; both fixture projects present.
///
/// ## Validation Strategy
/// Two path-based projects under one temp root. Run `.projects scope::global
/// live::0` with `HOME` redirected so no real history file is consulted.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-1
#[ test ]
fn ec_1_live_0_lists_every_non_live_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for name in [ "live-ec1-alpha", "live-ec1-beta" ]
  {
    let project = root.path().join( name );
    std::fs::create_dir_all( &project ).unwrap();
    common::write_path_project_session( &storage_root, &project, &format!( "session-{name}" ), 2 );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "live::0" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "live-ec1-alpha" ), "EC-1: alpha must survive live::0; got:\n{s}" );
  assert!( s.contains( "live-ec1-beta" ), "EC-1: beta must survive live::0; got:\n{s}" );
}

/// EC-2: `live::0` and an unset `live::` agree when nothing is live.
///
/// ## Purpose
/// Unset is a third state, distinct from `0`: it applies no filter at all. The
/// two only coincide when no project is live — which is exactly the fixture
/// case, making this the strongest available check that unset does not
/// accidentally default to `0` (or to `1`).
///
/// ## Coverage
/// Exit 0 for both; stdout identical once the relative-age column is pinned.
///
/// ## Validation Strategy
/// One fixture, two invocations differing only in the presence of `live::0`.
/// Compare stdout through [`common::normalize_relative_time`] — the two spawns can straddle a
/// second boundary, which changes the age text and the column width derived
/// from it without changing anything this case is about.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-2
#[ test ]
fn ec_2_live_0_matches_unset_when_nothing_is_live()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec2-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ec2", 3 );

  let filtered = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "live::0" )
    .output()
    .unwrap();

  let unset = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &filtered, 0 );
  common::assert_exit( &unset, 0 );
  assert_eq!(
    common::normalize_relative_time( &common::stdout( &filtered ) ),
    common::normalize_relative_time( &common::stdout( &unset ) ),
    "EC-2: live::0 must reproduce the unfiltered output when no project is live"
  );
}

/// EC-3: `live::1` never lists a project with no attached process.
///
/// ## Purpose
/// The narrowing half of the filter. Two outcomes are both correct and which
/// one appears depends on the host, not the fixture: with no Claude Code
/// process running anywhere, the command reports that detection found nothing
/// rather than presenting an empty list as an answer; with processes running
/// elsewhere, the fixture projects are simply filtered out. Either way the
/// fixture project must not appear.
///
/// ## Coverage
/// Exit 0; fixture project absent; output is one of the two documented forms.
///
/// ## Validation Strategy
/// One fixture project, `live::1`, `HOME` redirected. Assert absence of the
/// project name, then assert the output matches the unavailable-detection note
/// or a zero-project listing — pinning that no third shape can appear.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-3, "When detection is unavailable"
#[ test ]
fn ec_3_live_1_excludes_projects_without_an_attached_process()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec3-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ec3", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "live::1" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!(
    !s.contains( "live-ec3-proj" ),
    "EC-3: a project with no attached process must not survive live::1; got:\n{s}"
  );
  assert!(
    s.contains( "No attached Claude Code processes found." ) || s.contains( "0 projects" ),
    "EC-3: an empty live::1 result must be either the unavailable-detection note \
     or a zero-project listing; got:\n{s}"
  );
}

/// EC-4: Non-boolean value rejected.
///
/// ## Purpose
/// `live::` accepts only `0` or `1`. A typo must fail as an argument error
/// before any storage access, not silently degrade to one of the two states.
///
/// ## Coverage
/// Exit 1; non-empty stderr; no listing on stdout.
///
/// ## Validation Strategy
/// Run `.projects live::bogus` against an empty temp root from a neutral cwd.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-4
#[ test ]
fn ec_4_live_non_boolean_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "live::bogus" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  let err = common::stderr( &out );
  assert!( !err.is_empty(), "EC-4: live::bogus must produce an error on stderr" );
  assert!(
    common::stdout( &out ).trim().is_empty(),
    "EC-4: a rejected argument must produce no listing on stdout"
  );
}

/// EC-5: Out-of-range boolean value rejected.
///
/// ## Purpose
/// `2` parses as an integer but is not a boolean. Covers the range check
/// separately from EC-4's parse failure — they fail at different points.
///
/// ## Coverage
/// Exit 1; non-empty stderr.
///
/// ## Validation Strategy
/// Run `.projects live::2` from a neutral cwd.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-5
#[ test ]
fn ec_5_live_out_of_range_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "live::2" )
    .output()
    .unwrap();

  common::assert_exit( &out, 1 );
  assert!(
    !common::stderr( &out ).is_empty(),
    "EC-5: live::2 must produce an error on stderr"
  );
}

/// EC-6: `STATUS` column absent when no row is live.
///
/// ## Purpose
/// The column is conditional, like `⚠ gone` before it: reserving width for a
/// column that is empty on every row wastes the terminal line that the terse
/// overview exists to conserve. Absence is also the honest rendering — the
/// detector reports only positives, so a blank `STATUS` cell would read as
/// "not running" when it means "not detected".
///
/// ## Coverage
/// Exit 0; the four unconditional columns present; `STATUS` absent.
///
/// ## Validation Strategy
/// One fixture project, default flat layout, `HOME` redirected.
///
/// ## Related Requirements
/// `docs/cli/command/07_projects.md` § Session Liveness
#[ test ]
fn ec_6_status_column_absent_when_no_row_is_live()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec6-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ec6", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  for column in [ "LAST", "CONV", "AGENTS", "PROJECT" ]
  {
    assert!( s.contains( column ), "EC-6: the {column} column must still render; got:\n{s}" );
  }
  assert!(
    !s.contains( "STATUS" ),
    "EC-6: the STATUS column must not be reserved when no row is live; got:\n{s}"
  );
}

/// EC-7: `detail::sessions` carries no state tag when no row is live.
///
/// ## Purpose
/// The session-level tag is the finer-grained half of the same contract as
/// EC-6, on a different render path — `format_session_line` rather than the
/// terse table. A fixture session is never driven, so neither label may appear.
///
/// ## Coverage
/// Exit 0; session listed; neither `● working` nor `○ waiting` present.
///
/// ## Validation Strategy
/// One fixture project under `detail::sessions`, `HOME` redirected. Assert the
/// session id renders (proving the listing path ran) and both labels are absent.
///
/// ## Related Requirements
/// `docs/cli/command/07_projects.md` § Session Liveness
#[ test ]
fn ec_7_session_lines_carry_no_state_tag_when_not_live()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec7-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-ec7", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "detail::sessions" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "session-ec7" ), "EC-7: the session must be listed; got:\n{s}" );
  assert!( !s.contains( "working" ), "EC-7: no working tag may appear; got:\n{s}" );
  assert!( !s.contains( "waiting" ), "EC-7: no waiting tag may appear; got:\n{s}" );
}

/// EC-8: `live::` composes with `filter::`.
///
/// ## Purpose
/// `live::` is one narrowing among several and must intersect with the others
/// rather than replace them. Composition is where a filter added late tends to
/// break — by being applied before the others, or by short-circuiting them.
///
/// ## Coverage
/// Exit 0; only the substring-matching project survives both filters.
///
/// ## Validation Strategy
/// Two fixture projects with distinguishable names; `live::0 filter::` naming
/// one of them. Assert the other is gone.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-8
#[ test ]
fn ec_8_live_composes_with_filter()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  for name in [ "live-ec8-alpha", "live-ec8-beta" ]
  {
    let project = root.path().join( name );
    std::fs::create_dir_all( &project ).unwrap();
    common::write_path_project_session( &storage_root, &project, &format!( "session-{name}" ), 2 );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "live::0" )
    .arg( "filter::ec8-alpha" )
    .output()
    .unwrap();

  common::assert_exit( &out, 0 );
  let s = common::stdout( &out );
  assert!( s.contains( "live-ec8-alpha" ), "EC-8: the matching project must survive; got:\n{s}" );
  assert!( !s.contains( "live-ec8-beta" ), "EC-8: filter:: must still exclude beta; got:\n{s}" );
}

/// EC-9: `ids::1 live::0` passes a non-live project through.
///
/// ## Purpose
/// The `ids::` branch answers before the listing path ever reaches its filter,
/// so `live::` has to be re-applied there or it is silently discarded — the
/// worst outcome for a scripting mode, whose caller has no rendered output to
/// notice the discrepancy in. The fixture is never live, so `live::0` is the
/// branch that must *keep* its ids.
///
/// ## Coverage
/// Exit 0; both conversation ids listed; `count::1` reports 2.
///
/// ## Validation Strategy
/// One fixture project with two root conversations. Compare `ids::1 live::0`
/// against plain `ids::1` — they must agree, since nothing is live.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-9, § With `ids::1`
#[ test ]
fn ec_9_ids_with_live_0_keeps_a_non_live_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec9-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "root-ec9-a", 2 );
  common::write_path_project_session( &storage_root, &project, "root-ec9-b", 2 );

  let ids = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( format!( "project::{}", project.display() ) )
    .arg( "ids::1" )
    .arg( "live::0" )
    .output()
    .unwrap();

  common::assert_exit( &ids, 0 );
  let s = common::stdout( &ids );
  let lines : Vec< &str > = s.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 2, "EC-9: live::0 must keep both ids; got:\n{s}" );
  assert!( lines.contains( &"root-ec9-a" ), "EC-9: missing root-ec9-a; got:\n{s}" );
  assert!( lines.contains( &"root-ec9-b" ), "EC-9: missing root-ec9-b; got:\n{s}" );

  let counted = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( format!( "project::{}", project.display() ) )
    .arg( "ids::1" )
    .arg( "count::1" )
    .arg( "live::0" )
    .output()
    .unwrap();

  common::assert_exit( &counted, 0 );
  assert_eq!( common::stdout( &counted ).trim(), "2", "EC-9: count::1 must agree with the id lines" );
}

/// EC-10: `ids::1 live::1` withholds a non-live project's ids.
///
/// ## Purpose
/// The narrowing half of EC-9, and the one case in this branch whose output is
/// host-dependent. Suppression must be *visible* to a caller: an empty stdout
/// with exit 0 is a real answer ("nothing attached, and detection could see
/// that"), while an empty stdout on a host that cannot see any process at all
/// would be a fabricated one — so that case exits non-zero with the reason on
/// stderr instead. Which of the two occurs depends on what runs on the test
/// host, so the assertion covers both and forbids everything else.
///
/// ## Coverage
/// Either (exit 0, empty stdout, `count::1` = 0) or (exit 1, non-empty stderr).
/// The fixture's ids never appear on stdout under either outcome.
///
/// ## Validation Strategy
/// Same fixture as EC-9, `live::1` instead of `live::0`.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-10, § With `ids::1`
#[ test ]
fn ec_10_ids_with_live_1_withholds_a_non_live_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec10-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "root-ec10-a", 2 );

  let ids = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( format!( "project::{}", project.display() ) )
    .arg( "ids::1" )
    .arg( "live::1" )
    .output()
    .unwrap();

  let s = common::stdout( &ids );
  assert!( !s.contains( "root-ec10-a" ),
    "EC-10: a project with no attached process must never have its ids listed under live::1; got:\n{s}" );

  if ids.status.success()
  {
    assert!( s.trim().is_empty(), "EC-10: detection available and nothing attached means no ids at all; got:\n{s}" );

    let counted = common::clg_cmd()
      .env( "HOME", root.path() )
      .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
      .arg( ".projects" )
      .arg( format!( "project::{}", project.display() ) )
      .arg( "ids::1" )
      .arg( "count::1" )
      .arg( "live::1" )
      .output()
      .unwrap();

    common::assert_exit( &counted, 0 );
    assert_eq!( common::stdout( &counted ).trim(), "0", "EC-10: count::1 must agree with the suppressed listing" );
  }
  else
  {
    assert!( !common::stderr( &ids ).trim().is_empty(),
      "EC-10: a failure here must say why detection could not answer, not fail silently" );
  }
}

/// EC-11: the tree layout carries the same liveness affordance as the flat one.
///
/// ## Purpose
/// `detail::sessions` has two renderers — flat families and `show_tree::1` —
/// and only one of them was originally passed the liveness map, so choosing a
/// layout silently decided whether "is this one running" got answered at all.
/// The layout is a presentation choice; it must not gate a fact.
///
/// ## Coverage
/// Exit 0 for both layouts; the session id renders under each (proving both
/// listing paths ran); neither carries a state tag, since a fixture is never
/// live.
///
/// ## Validation Strategy
/// One fixture rendered twice, once per layout, asserting the affordance is
/// absent from both. This pins the two paths to each other — the positive
/// direction is unreachable from a fixture and is covered by
/// `src/cli/liveness.rs`'s unit tests.
///
/// ## Related Requirements
/// `docs/cli/param/44_live.md` — EC-11
#[ test ]
fn ec_11_tree_and_flat_layouts_agree_on_the_state_tag()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "live-ec11-proj" );
  std::fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "root-ec11", 4 );

  for tree in [ false, true ]
  {
    let mut cmd = common::clg_cmd();
    cmd
      .env( "HOME", root.path() )
      .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
      .arg( ".projects" )
      .arg( "scope::global" )
      .arg( "detail::sessions" );
    if tree { cmd.arg( "show_tree::1" ); }
    let out = cmd.output().unwrap();

    common::assert_exit( &out, 0 );
    let s = common::stdout( &out );
    assert!( s.contains( "root-ec11" ), "EC-11: the listing must render (tree={tree}); got:\n{s}" );
    assert!( !s.contains( "working" ), "EC-11: no working tag for a fixture (tree={tree}); got:\n{s}" );
    assert!( !s.contains( "waiting" ), "EC-11: no waiting tag for a fixture (tree={tree}); got:\n{s}" );
  }
}
