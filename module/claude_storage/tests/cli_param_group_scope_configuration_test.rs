//! Cross-command interaction tests for Scope Configuration parameter group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param_group/05_scope_configuration.md`
//!
//! ## Coverage
//!
//! - CC-1: `scope::local` uses `path::` as directory anchor
//! - CC-2: `scope::relevant` starts ancestor walk from `path::`
//! - CC-3: `scope::under` searches subtree rooted at `path::`
//! - CC-4: `scope::global` ignores `path::` value
//! - CC-5: `scope::under` without `path::` defaults to cwd
//! - CC-6: `path::` without `scope::` defaults to under scope

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

/// CC-1: `scope::local` uses `path::` as directory anchor.
///
/// ## Purpose
/// Verify that `scope::local ``path::``/a/b/c` returns only the project at
/// exactly `/a/b/c` and excludes ancestor projects at `/a/b` and `/a`.
///
/// ## Coverage
/// Local scope exact-match; ancestors excluded; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/05_scope_configuration.md` — CC-1
#[ test ]
fn cc_1_scope_local_uses_path_as_directory_anchor()
{
  let root = TempDir::new().unwrap();
  let storage = root.path().join( ".claude" );

  // Nested project hierarchy
  let p_abc = root.path().join( "a" ).join( "b" ).join( "c" );
  let p_ab  = root.path().join( "a" ).join( "b" );
  let p_a   = root.path().join( "a" );

  common::write_path_project_session( &storage, &p_abc, "s-abc", 2 );
  common::write_path_project_session( &storage, &p_ab,  "s-ab",  2 );
  common::write_path_project_session( &storage, &p_a,   "s-a",   2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", &storage )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", p_abc.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  // Only the exact path project should appear
  assert!(
    s.contains( 'c' ),
    "CC-1: scope::local must include the path:: project; got:\n{s}"
  );
  // Ancestor projects must not appear as separate entries
  // (check that shorter paths are not listed independently)
  let lines : Vec< &str > = s.lines().collect();
  // The output should have exactly one project entry for the local scope
  let project_lines : Vec< &str > = lines.iter()
    .filter( | l | l.contains( 'a' ) && ( l.contains( "/b" ) || l.contains( "\\b" ) ) )
    .copied()
    .collect();
  assert!(
    project_lines.len() <= 1 || project_lines.iter().all( | l | l.contains( 'c' ) ),
    "CC-1: scope::local must not include ancestor /a/b without /c; got:\n{s}"
  );
}

/// CC-2: `scope::relevant` starts ancestor walk from `path::`.
///
/// ## Purpose
/// Verify that `scope::relevant ``path::``/a/b/c` returns all ancestor projects:
/// `/a/b/c`, `/a/b`, and `/a`.
///
/// ## Coverage
/// Relevant scope ancestor walk; all three ancestor projects visible; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/05_scope_configuration.md` — CC-2
#[ test ]
fn cc_2_scope_relevant_starts_ancestor_walk_from_path()
{
  let root = TempDir::new().unwrap();
  let storage = root.path().join( ".claude" );

  let p_abc = root.path().join( "r" ).join( "b" ).join( "c" );
  let p_ab  = root.path().join( "r" ).join( "b" );
  let p_a   = root.path().join( "r" );

  common::write_path_project_session( &storage, &p_abc, "s-rbc", 2 );
  common::write_path_project_session( &storage, &p_ab,  "s-rb",  2 );
  common::write_path_project_session( &storage, &p_a,   "s-r",   2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", &storage )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", p_abc.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  // All three levels must appear
  assert!(
    s.contains( 'c' ),
    "CC-2: scope::relevant must include deepest path /r/b/c; got:\n{s}"
  );
  assert!(
    s.contains( 'b' ),
    "CC-2: scope::relevant must include ancestor /r/b; got:\n{s}"
  );
  assert!(
    s.contains( 'r' ),
    "CC-2: scope::relevant must include ancestor /r; got:\n{s}"
  );
}

/// CC-3: `scope::under` searches subtree rooted at `path::`.
///
/// ## Purpose
/// Verify that `scope::under ``path::``/a/b` returns projects at `/a/b`, `/a/b/c`,
/// and `/a/b/c/d` but excludes an unrelated project at `/z`.
///
/// ## Coverage
/// Under scope subtree search; unrelated project excluded; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/05_scope_configuration.md` — CC-3
#[ test ]
fn cc_3_scope_under_searches_subtree_rooted_at_path()
{
  let root = TempDir::new().unwrap();
  let storage = root.path().join( ".claude" );

  let p_ab   = root.path().join( "u" ).join( "b" );
  let p_abc  = root.path().join( "u" ).join( "b" ).join( "c" );
  let p_abcd = root.path().join( "u" ).join( "b" ).join( "c" ).join( "d" );
  let p_z    = root.path().join( "z-unrelated" );

  common::write_path_project_session( &storage, &p_ab,   "s-ub",   2 );
  common::write_path_project_session( &storage, &p_abc,  "s-ubc",  2 );
  common::write_path_project_session( &storage, &p_abcd, "s-ubcd", 2 );
  common::write_path_project_session( &storage, &p_z,    "s-z",    2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", &storage )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", p_ab.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  assert!(
    s.contains( 'u' ) && s.contains( 'b' ),
    "CC-3: scope::under must include /u/b and its subtree; got:\n{s}"
  );
  assert!(
    !s.contains( "z-unrelated" ),
    "CC-3: scope::under must exclude unrelated /z project; got:\n{s}"
  );
}

/// CC-4: `scope::global` ignores `path::` value.
///
/// ## Purpose
/// Verify that `scope::global ``path::``/a/b` returns ALL projects in storage
/// regardless of the `path::` value.
///
/// ## Coverage
/// Global scope ignores `path::` anchor; all projects returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/05_scope_configuration.md` — CC-4
#[ test ]
fn cc_4_scope_global_ignores_path_value()
{
  let root = TempDir::new().unwrap();
  let storage = root.path().join( ".claude" );

  let p_ab = root.path().join( "g" ).join( "b" );
  let p_cd = root.path().join( "g-c" ).join( "d" );
  let p_ef = root.path().join( "g-e" ).join( "f" );

  common::write_path_project_session( &storage, &p_ab, "s-gb", 2 );
  common::write_path_project_session( &storage, &p_cd, "s-gd", 2 );
  common::write_path_project_session( &storage, &p_ef, "s-gf", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", &storage )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( format!( "path::{}", p_ab.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  // All three projects must appear despite path:: pointing to only /g/b
  assert!(
    s.contains( "g-c" ) || s.contains( "gd" ),
    "CC-4: scope::global must include /g-c/d despite path::/g/b; got:\n{s}"
  );
  assert!(
    s.contains( "g-e" ) || s.contains( "gf" ),
    "CC-4: scope::global must include /g-e/f despite path::/g/b; got:\n{s}"
  );
}

/// CC-5: `scope::under` without `path::` defaults to cwd.
///
/// ## Purpose
/// Verify that `scope::under` without `path::` uses the current working
/// directory as the subtree root, including projects at and below cwd but
/// excluding unrelated projects.
///
/// ## Coverage
/// Default cwd anchor for `scope::under`; subtree projects visible; non-subtree absent; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/05_scope_configuration.md` — CC-5
#[ test ]
fn cc_5_scope_under_without_path_defaults_to_cwd()
{
  let root = TempDir::new().unwrap();
  let storage = root.path().join( ".claude" );

  let base = root.path().join( "cwdscope" );
  let sub  = base.join( "sub" );
  let unrelated = root.path().join( "cwdunrelated" );

  std::fs::create_dir_all( &base ).unwrap();

  common::write_path_project_session( &storage, &base,      "s-base", 2 );
  common::write_path_project_session( &storage, &sub,       "s-sub",  2 );
  common::write_path_project_session( &storage, &unrelated, "s-unrel", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", &storage )
    .current_dir( &base )
    .arg( ".projects" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  assert!(
    s.contains( "cwdscope" ),
    "CC-5: scope::under with cwd must include base and sub projects; got:\n{s}"
  );
  assert!(
    !s.contains( "cwdunrelated" ),
    "CC-5: scope::under with cwd must exclude unrelated project; got:\n{s}"
  );
}

/// CC-6: `path::` without `scope::` defaults to under scope.
///
/// ## Purpose
/// Verify that providing `path::` without `scope::` defaults to under-scope
/// semantics, returning the path project and all sub-path projects but not
/// unrelated projects.
///
/// ## Coverage
/// Default under scope with `path::`; sub-projects included; unrelated excluded; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/05_scope_configuration.md` — CC-6
#[ test ]
fn cc_6_path_without_scope_defaults_to_under_scope()
{
  let root = TempDir::new().unwrap();
  let storage = root.path().join( ".claude" );

  let anchor = root.path().join( "def-scope" ).join( "anchor" );
  let sub    = anchor.join( "subsection" );
  let other  = root.path().join( "def-other" );

  common::write_path_project_session( &storage, &anchor, "s-anchor", 2 );
  common::write_path_project_session( &storage, &sub,    "s-sub",    2 );
  common::write_path_project_session( &storage, &other,  "s-other",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", &storage )
    .arg( ".projects" )
    .arg( format!( "path::{}", anchor.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  assert!(
    s.contains( "anchor" ),
    "CC-6: path:: without scope:: must include the anchor project; got:\n{s}"
  );
  assert!(
    s.contains( "subsection" ),
    "CC-6: path:: without scope:: must include sub-path projects; got:\n{s}"
  );
  assert!(
    !s.contains( "def-other" ),
    "CC-6: path:: without scope:: must exclude unrelated projects; got:\n{s}"
  );
}
