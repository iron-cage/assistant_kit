//! Tests for `scope::around` in `.projects` — bidirectional neighborhood semantics.
//!
//! ## Purpose
//!
//! Verify that `scope::around` (the union of `relevant ∪ under`) correctly
//! includes ancestor projects, the cwd project itself, and descendant projects,
//! while excluding sibling projects at the same depth.
//!
//! Also verifies that the default scope for `.projects` is now `around`, not
//! `under` — tested by omitting `scope::` but providing `path::` (which
//! disables `is_default` routing, putting the command in list mode with the
//! default scope).
//!
//! ## Test Matrix
//!
//! | ID    | Scenario | Expected |
//! |-------|----------|----------|
//! | IT-57 | `scope::around` explicit — ancestor+self+descendant visible, sibling excluded | 4 assertions |
//! | IT-58 | Default scope (no `scope::` arg) with explicit `path::` — same result as `scope::around` | 3 assertions |
//! | IT-59 | `scope::around` degenerate — no ancestor or descendant; only self visible | 1 assertion  |

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

fn assert_exit_0( out : &std::process::Output )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "expected exit 0; stderr: {}", stderr( out )
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// IT-57: scope::around includes ancestor+self+descendant; excludes sibling
// ─────────────────────────────────────────────────────────────────────────────

/// IT-57: `scope::around` explicit — bidirectional neighborhood semantics.
///
/// ## Purpose
///
/// Verify that `scope::around` is the union of `relevant ∪ under`: it includes
/// the ancestor project (↑), the cwd project itself, and the descendant project
/// (↓), while excluding a sibling project at the same depth as cwd.
///
/// ## Coverage
///
/// Four-project fixture:
///   - `outer/`         — ancestor project (included via `relevant` arm)
///   - `outer/mid/`     — cwd project (included as exact match)
///   - `outer/mid/inner/` — descendant project (included via `under` arm)
///   - `outer/branch/`  — sibling project (excluded: not ancestor, not descendant)
///
/// ## Validation Strategy
///
/// Assert session IDs present/absent in stdout using distinct session names per
/// project. The assertions directly reflect the four union-semantics cases.
///
/// ## Related Requirements
///
/// `docs/cli/types.md` § `ScopeValue` — AROUND constant, bidirectional semantics
#[test]
fn it57_scope_around_includes_ancestor_self_descendant_excludes_sibling()
{
  let root         = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor   = root.path().join( "outer" );
  let cwd        = ancestor.join( "mid" );
  let descendant = cwd.join( "inner" );
  let sibling    = ancestor.join( "branch" );

  common::write_path_project_session( &storage_root, &ancestor,   "session-57-ancestor",   2 );
  common::write_path_project_session( &storage_root, &cwd,        "session-57-cwd",        2 );
  common::write_path_project_session( &storage_root, &descendant, "session-57-descendant", 2 );
  common::write_path_project_session( &storage_root, &sibling,    "session-57-sibling",    2 );

  let out = common::clg_cmd()
    .env( "HOME",                root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::around" )
    .arg( format!( "path::{}", cwd.display() ) )
    .output()
    .unwrap();

  assert_exit_0( &out );
  let s = stdout( &out );
  assert!(
    s.contains( "session-57-ancestor" ),
    "scope::around must include ancestor project; got:\n{s}"
  );
  assert!(
    s.contains( "session-57-cwd" ),
    "scope::around must include the cwd project itself; got:\n{s}"
  );
  assert!(
    s.contains( "session-57-descendant" ),
    "scope::around must include descendant project; got:\n{s}"
  );
  assert!(
    !s.contains( "session-57-sibling" ),
    "scope::around must exclude sibling project at same depth; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// IT-58: default scope (no scope:: arg) produces scope::around semantics
// ─────────────────────────────────────────────────────────────────────────────

/// IT-58: Default scope with explicit `path::` uses `scope::around` semantics.
///
/// ## Purpose
///
/// Verify that omitting `scope::` and providing only `path::` (which disables
/// the `is_default` gate, dropping into list mode with the default scope)
/// produces the same bidirectional neighborhood output as explicit
/// `scope::around`. After task-018, `unwrap_or("around")` replaces
/// `unwrap_or("under")`.
///
/// ## Coverage
///
/// Three-project fixture (same topology as IT-57 minus sibling):
///   - `outer/`         — ancestor project
///   - `outer/mid/`     — cwd / `path::` target
///   - `outer/mid/inner/` — descendant project
///
/// Command passes `path::outer/mid` but no `scope::` — triggering the default.
///
/// ## Validation Strategy
///
/// Under the old default (`under`): only `session-58-cwd` and
/// `session-58-descendant` would appear (ancestor excluded).
/// Under the new default (`around`): all three appear.
///
/// ## Related Requirements
///
/// `docs/cli/params.md` § `scope::` — default value changed to `around`
#[test]
fn it58_default_scope_equals_around_when_path_is_explicit()
{
  let root         = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor   = root.path().join( "outer" );
  let cwd        = ancestor.join( "mid" );
  let descendant = cwd.join( "inner" );

  common::write_path_project_session( &storage_root, &ancestor,   "session-58-ancestor",   2 );
  common::write_path_project_session( &storage_root, &cwd,        "session-58-cwd",        2 );
  common::write_path_project_session( &storage_root, &descendant, "session-58-descendant", 2 );

  let out = common::clg_cmd()
    .env( "HOME",                root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    // No scope:: arg — exercises unwrap_or default
    .arg( format!( "path::{}", cwd.display() ) )
    .output()
    .unwrap();

  assert_exit_0( &out );
  let s = stdout( &out );
  assert!(
    s.contains( "session-58-ancestor" ),
    "default scope must include ancestor (scope::around); got:\n{s}"
  );
  assert!(
    s.contains( "session-58-cwd" ),
    "default scope must include cwd project; got:\n{s}"
  );
  assert!(
    s.contains( "session-58-descendant" ),
    "default scope must include descendant (scope::around); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// IT-59: scope::around degenerate — only cwd project, no relatives
// ─────────────────────────────────────────────────────────────────────────────

/// IT-59: `scope::around` with no ancestor or descendant projects shows only self.
///
/// ## Purpose
///
/// Verify the degenerate case: when only the cwd project exists in storage
/// (no ancestor projects, no descendant projects, no sibling projects),
/// `scope::around` returns exactly one project — the same result as
/// `scope::local`.
///
/// ## Coverage
///
/// Single-project fixture:
///   - `standalone/` — the only project in storage
///
/// ## Validation Strategy
///
/// stdout contains the single project's session ID; exit 0; "Found 1 project:"
/// singular form confirms exactly one project was found.
///
/// ## Related Requirements
///
/// `docs/cli/types.md` § `ScopeValue` — AROUND semantics in empty neighborhood
#[test]
fn it59_scope_around_degenerate_shows_only_self_when_no_relatives()
{
  let root         = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let standalone = root.path().join( "standalone" );
  common::write_path_project_session( &storage_root, &standalone, "session-59-only", 2 );

  let out = common::clg_cmd()
    .env( "HOME",                root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::around" )
    .arg( format!( "path::{}", standalone.display() ) )
    .output()
    .unwrap();

  assert_exit_0( &out );
  let s = stdout( &out );
  assert!(
    s.contains( "session-59-only" ),
    "scope::around must include the cwd project in the degenerate case; got:\n{s}"
  );
  assert!(
    s.contains( "Found 1 project:" ),
    "degenerate scope::around must show exactly 1 project; got:\n{s}"
  );
}
