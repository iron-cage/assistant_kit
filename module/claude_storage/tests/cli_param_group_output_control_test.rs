//! Cross-command interaction tests for Output Control parameter group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param_group/01_output_control.md`
//!
//! ## Coverage
//!
//! - CC-1: `verbosity::0` gives minimal output in .status
//! - CC-2: `verbosity::0` gives minimal output in .list
//! - CC-3: `verbosity::1` default is consistent across commands
//! - CC-4: `verbosity::2` adds detail in .status
//! - CC-5: `v::` alias works in .list same as `verbosity::`
//! - CC-6: verbosity level does not affect which results are returned

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

/// CC-1: `verbosity::0` gives minimal output in .status.
///
/// ## Purpose
/// Verify that `.status ``verbosity::``0` produces minimal, unlabelled output —
/// no decorative headers, table borders, or section labels.
///
/// ## Coverage
/// Minimal output format; no decorative labels; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-1
#[ test ]
fn cc_1_verbosity_0_minimal_output_in_status()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "cc1-proj" );
  common::write_path_project_session( root.path(), &proj, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "CC-1: verbosity::0 .status must produce some output; got silence; stderr: {}",
    stderr( &out )
  );
  // verbosity::0 must not include decorated table borders or section headers
  assert!(
    !s.contains( "===" ) && !s.contains( "---" ) && !s.contains( "Storage" ),
    "CC-1: verbosity::0 must not include decorative headers; got:\n{s}"
  );
}

/// CC-2: `verbosity::0` gives minimal output in .list.
///
/// ## Purpose
/// Verify that `.list ``verbosity::``0` produces one path per line with no header
/// line and no count footer.
///
/// ## Coverage
/// One-per-line undecorated output; no header or footer; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-2
#[ test ]
fn cc_2_verbosity_0_minimal_output_in_list()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "cc2proj" );
  common::write_path_project_session( root.path(), &proj, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "CC-2: verbosity::0 .list must produce output; stderr: {}",
    stderr( &out )
  );
  // No count line and no section header
  assert!(
    !s.contains( "project" ) || s.lines().count() == 1,
    "CC-2: verbosity::0 .list must not add header/footer lines; got:\n{s}"
  );
  // Output contains the project path segment (no hyphens that get re-encoded)
  assert!(
    s.contains( "cc2proj" ),
    "CC-2: project path must appear in verbosity::0 .list output; got:\n{s}"
  );
}

/// CC-3: `verbosity::1` default is consistent across commands.
///
/// ## Purpose
/// Verify that omitting `verbosity::` (defaulting to `verbosity::1`) produces
/// non-empty styled output for both `.status` and `.list`.
///
/// ## Coverage
/// Default verbosity output; both commands produce output; exit 0 for both.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-3
#[ test ]
fn cc_3_verbosity_1_default_consistent_across_commands()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "cc3-proj" );
  common::write_path_project_session( root.path(), &proj, "s001", 2 );

  let status_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  let list_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &status_out, 0 );
  assert_exit( &list_out, 0 );

  assert!(
    !stdout( &status_out ).is_empty(),
    "CC-3: default .status must produce output; stderr: {}",
    stderr( &status_out )
  );
  assert!(
    !stdout( &list_out ).is_empty(),
    "CC-3: default .list must produce output; stderr: {}",
    stderr( &list_out )
  );
}

/// CC-4: `verbosity::2` adds detail in .status.
///
/// ## Purpose
/// Verify that `.status ``verbosity::``2` produces more detailed output than
/// `verbosity::1`, such as per-project session counts.
///
/// ## Coverage
/// Per-project detail rows present; more detail than `verbosity::1`; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-4
#[ test ]
fn cc_4_verbosity_2_adds_detail_in_status()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "cc4-alpha" );
  let p2 = root.path().join( "cc4-beta" );
  common::write_path_project_session( root.path(), &p1, "s001", 4 );
  common::write_path_project_session( root.path(), &p2, "s002", 4 );
  common::write_path_project_session( root.path(), &p2, "s003", 4 );

  let v1_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  let v2_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &v1_out, 0 );
  assert_exit( &v2_out, 0 );

  let v1 = stdout( &v1_out );
  let v2 = stdout( &v2_out );

  assert!(
    v2.len() > v1.len(),
    "CC-4: verbosity::2 must produce more output than verbosity::1;\n  v1 ({} bytes):\n{v1}\n  v2 ({} bytes):\n{v2}",
    v1.len(),
    v2.len()
  );
}

/// CC-5: `v::` alias works in .list same as `verbosity::`.
///
/// ## Purpose
/// Verify that `v::0` is a valid alias for `verbosity::0` and produces
/// identical output in `.list`.
///
/// ## Coverage
/// Alias equivalence; identical output; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-5
#[ test ]
fn cc_5_v_alias_works_in_list_same_as_verbosity()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "cc5-proj" );
  common::write_path_project_session( root.path(), &proj, "s001", 2 );

  let long_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  let alias_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "v::0" )
    .output()
    .unwrap();

  assert_exit( &long_out, 0 );
  assert_exit( &alias_out, 0 );

  assert_eq!(
    stdout( &long_out ),
    stdout( &alias_out ),
    "CC-5: v::0 must produce identical output to verbosity::0 in .list"
  );
}

/// CC-6: verbosity level does not affect which results are returned.
///
/// ## Purpose
/// Verify that both `verbosity::0` and `verbosity::3` return all 3 projects;
/// only the format of each entry differs, not the result set.
///
/// ## Coverage
/// Result set invariant across verbosity levels; all 3 projects visible; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-6
#[ test ]
fn cc_6_verbosity_level_does_not_affect_which_results_are_returned()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "cc6alpha" );
  let p2 = root.path().join( "cc6beta" );
  let p3 = root.path().join( "cc6gamma" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p3, "s003", 2 );

  let v0_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  let v3_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "verbosity::3" )
    .output()
    .unwrap();

  assert_exit( &v0_out, 0 );
  assert_exit( &v3_out, 0 );

  // Both outputs must contain all 3 project path segments
  for ( label, out ) in &[ ( "verbosity::0", stdout( &v0_out ) ), ( "verbosity::3", stdout( &v3_out ) ) ]
  {
    assert!(
      out.contains( "cc6alpha" ),
      "CC-6: {label} must include cc6alpha; got:\n{out}"
    );
    assert!(
      out.contains( "cc6beta" ),
      "CC-6: {label} must include cc6beta; got:\n{out}"
    );
    assert!(
      out.contains( "cc6gamma" ),
      "CC-6: {label} must include cc6gamma; got:\n{out}"
    );
  }
}
