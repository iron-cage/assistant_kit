//! Tests for `.projects` path encoding/decoding — `decode_project_display` behavior.
//!
//! ## Coverage
//!
//! Bug reproducers for path display correctness when project paths contain
//! underscores or hyphen-prefixed directory components:
//!
//! | ID    | Issue | What it covers                                                |
//! |-------|-------|---------------------------------------------------------------|
//! | IT-24 | 030   | Hyphen-prefixed topic dirs preserved in display path          |
//! | IT-23 | 029   | Underscore-named dirs decoded correctly (not split on `/`)    |
//! | IT-25 | 031   | `scope::under` excludes underscore-suffix sibling modules     |
//! | IT-26 | 032   | `scope::relevant` excludes underscore-prefix sibling modules  |
//! | IT-60 | 035   | Topic path shown even when topic dir absent from disk (T01)   |
//! | IT-61 | 035   | Topic path shown when topic dir present on disk (T02)         |
//! | IT-62 | 035   | Default-topic path shown when absent from disk (T03)          |
//! | IT-63 | 035   | Base path shown correctly with no topic suffix (T04)          |
//! | IT-64 | 035   | Double-topic storage key shows both topic components (T05)    |
//!
//! Note: IT-60..IT-64 follow IT-59 (`scope::around` tests in `projects_scope_around_test.rs`).
//! IT-27..IT-30 were already allocated in `tests/doc/cli/testing/command/07_projects.md`
//! for unrelated tests, so the next available block was used here.

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

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Hyphen-Prefixed Topic Directory (issue-030)
//
// Root Cause: decode_project_display stripped the `--topic` suffix before
// decoding, so `-...-src--default-topic` displayed as `src` even when
// `-default_topic` is a real filesystem directory (the actual working directory).
//
// Why Not Caught: All prior tests used simple session paths with no
// hyphen-prefixed directory components. No test path ended in `/-default_topic`
// or any other `-name` component that the topic strip discarded.
//
// Fix Applied: decode_project_display now tries to extend the decoded base path
// by each `--topic` component as a real filesystem directory. The display uses
// the longest existing path prefix. So `-...-src--default-topic` displays as
// `src/-default_topic` when that directory exists on disk.
//
// Prevention: Test that sessions created from a hyphen-prefixed working
// directory (e.g. `src/-default_topic`) display the full path in the header.
//
// Pitfall: After Fix(issue-035) the existence check in the topic-extension loop
// was removed — topics are now unconditionally joined. The IT-60..IT-62 tests
// verify the absent-dir case. The only remaining existence check is on the base
// path decode (used for underscore/slash ambiguity resolution), which is correct.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-030)
fn it_24_decode_display_includes_hyphen_prefixed_topic_dir()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Project path ending in a hyphen-prefixed directory (the real CWD pattern)
  let project = root.path().join( "src" ).join( "-default_topic" );
  // Create the actual directories so the existence check passes
  std::fs::create_dir_all( &project ).expect( "create src/-default_topic dir" );
  common::write_path_project_session( &storage_root, &project, "session-topic-dir-test", 2 );

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
    s.contains( "-default_topic" ),
    "display path must include hyphen-prefixed topic dir '-default_topic'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.trim_end().ends_with( "src:" ) ),
    "display path must NOT be truncated to 'src' when '-default_topic' exists; got:\n{s}"
  );
  assert!( s.contains( "session-topic-dir-test" ), "session ID must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Underscore Directory Names (issue-029)
//
// Root Cause: encode_path converts `_` → `-` (lossy). The heuristic decoder
// defaults to path separator (`/`) for all unrecognized `-` boundaries, so
// underscore-named directories like `wip_core` decode to `wip/core` in the
// display path.
//
// Why Not Caught: All prior tests used simple single-word project dir names
// (e.g., "proj", "agent_filter_proj"). No test path had underscore-named
// intermediate components like `wip_core/project`.
//
// Fix Applied: decode_project_display now checks whether the heuristic-decoded
// path exists on the filesystem. If not, it falls back to decode_path_via_fs
// which walks the real directory tree, choosing `/` vs `_` at each `-` boundary
// by calling is_dir() on the candidate path prefix.
//
// Prevention: Test project paths that contain underscore-named intermediate
// directories. The test must also create those directories on disk so the
// filesystem walk can verify existence.
//
// Pitfall: decode_path_via_fs requires the project directory to exist at display
// time. Deleted or remote projects fall back to the raw encoded storage dir name.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-029)
fn it_23_decode_display_preserves_underscore_named_dirs()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Project path with underscore-named directory component
  let project = root.path().join( "wip_core" ).join( "myproject" );
  // Create the actual directories so filesystem-guided decode can verify existence
  std::fs::create_dir_all( &project ).expect( "create project dir with underscore component" );
  common::write_path_project_session( &storage_root, &project, "session-underscore-test", 2 );

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
    s.contains( "wip_core" ),
    "display path must preserve underscore: 'wip_core' not 'wip/core'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.contains( "wip/core" ) ),
    "display path must NOT split wip_core into wip/core; got:\n{s}"
  );
  assert!( s.contains( "session-underscore-test" ), "session ID must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Sibling Module Exclusion (issue-031)
//
// Root Cause: encode_path maps both `_` and `/` to `-`. The `under` predicate
// used string starts_with on encoded forms, so a sibling `base_extra/` passed
// the same prefix check as a child `base/sub/`: both encoded forms start with
// the `base-` prefix. String comparison cannot distinguish path-separator `/`
// from underscore `_` in encoded form.
//
// Why Not Caught: All prior scope::under tests used simple single-word base dirs
// (e.g., "workspace"). No test had a sibling whose name was the base name plus
// an underscore suffix, simulating real module naming like `claude_storage_core`
// next to `claude_storage`.
//
// Fix Applied: Two-stage predicate. String prefix is fast-reject only. Candidates
// passing string check (not exact) are verified via decode_path_via_fs +
// Path::starts_with. Path::starts_with is component-wise: Path("/x/base_extra")
// does NOT start_with Path("/x/base") even though string "/x/base_extra"
// starts_with "/x/base".
//
// Prevention: Always test scope::under with a sibling whose encoded form shares the
// base encoded prefix (underscore-suffix sibling). Create all directories on disk
// so decode_path_via_fs can resolve them correctly.
//
// Pitfall: decode_path_via_fs returns None for deleted/remote paths. The fixed
// predicate uses unwrap_or(true) (conservative include) to avoid silently dropping
// sessions from projects that existed when the session was created.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-031)
fn it_25_scope_under_excludes_underscore_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: base = module/claude_storage
  //           child = module/claude_storage/sub  (under base → must appear)
  //           sibling = module/base_extra         (NOT under base → must not appear)
  let base    = root.path().join( "base" );
  let child   = base.join( "sub" );
  let sibling = root.path().join( "base_extra" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → unwrap_or(true) includes all.
  std::fs::create_dir_all( &child ).expect( "create child dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  common::write_path_project_session( &storage_root, &child,   "session-it25-child",   2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it25-sibling", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it25-child" ),
    "must contain session-it25-child (child base/sub is under base); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it25-sibling" ),
    "must NOT contain session-it25-sibling (sibling base_extra is NOT under base); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::relevant — Sibling Module Exclusion (issue-032)
//
// Root Cause: encode_path maps both `_` and `/` to `-`. The `relevant` scope
// predicate (is_relevant_encoded) uses string starts_with: encoded_base
// starts_with(dir_name + "-"). A sibling `base/` passed the same prefix check
// as a real ancestor: if base_path is `/tmp/base_extra`, the project at `/tmp/base`
// (encoded `-tmp-base`) matched because `-tmp-base-extra` starts with `-tmp-base-`.
// String comparison cannot distinguish `/` from `_` in encoded form.
//
// Why Not Caught: All prior scope::relevant tests used simple ancestor chains
// (e.g., /a, /a/b, /a/b/c). No test had a sibling whose encoded name was a
// prefix of the current path's encoded form — the `base` vs `base_extra` pattern.
//
// Fix Applied: Two-stage predicate in the `"relevant"` arm of project_matches.
// is_relevant_encoded is fast-reject only. Exact encoded match returns true.
// Prefix-match candidates are verified via decode_path_via_fs +
// base_path.starts_with(decoded_path). Path::starts_with is component-wise:
// Path("/x/base_extra").starts_with(Path("/x/base")) → false.
//
// Prevention: Always test scope::relevant with a project whose name is a
// string prefix of the current path's name (underscore-suffix sibling).
// Create all directories on disk so decode_path_via_fs can resolve them.
//
// Pitfall: Same as issue-031 fix for scope::under — decode_path_via_fs returns
// None for deleted/remote paths; is_none_or provides conservative include.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-032)
fn it_26_scope_relevant_excludes_underscore_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: sibling = base      (NOT an ancestor of base_extra despite prefix match)
  //           target  = base_extra (current path; encoded -...-base-extra)
  // /base encoded to `-...-base`; `/base_extra` encoded to `-...-base-extra`.
  // Without fix: is_relevant_encoded returns true because encoded_base starts
  // with (dir_name + "-"), making scope::relevant include /base as a false ancestor.
  let sibling = root.path().join( "base" );
  let target  = root.path().join( "base_extra" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → is_none_or(true) includes all.
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );
  std::fs::create_dir_all( &target ).expect( "create target dir" );

  common::write_path_project_session( &storage_root, &sibling, "session-it26-sibling", 2 );
  common::write_path_project_session( &storage_root, &target,  "session-it26-target",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it26-target" ),
    "must contain session-it26-target (current project at base_extra); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it26-sibling" ),
    "must NOT contain session-it26-sibling (/base is NOT an ancestor of /base_extra); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Absent Topic Directory (issue-035)
//
// Root Cause: `decode_project_display` checks `candidate.exists()` before
// extending the decoded base path with a topic component. When the topic
// directory (`-commit`) is absent from disk, the extension is skipped and
// the function returns only the base path. Violates the display-path
// invariant: the storage key records the CWD at session start; current
// filesystem state is irrelevant to session attribution.
//
// Why Not Caught: The issue-030 fix was tested only with extant topic
// directories (`create_dir_all` before running). No test exercised the case
// where the topic directory had been deleted, so the guard was never
// challenged.
//
// Fix Applied: Remove the `candidate.exists()` guard in the topic-extension
// loop — always join unconditionally. The storage key is the authoritative
// CWD record; disk state at query time must not affect session attribution.
//
// Prevention: Every bug_reproducer for `decode_project_display` must include
// both an extant-dir variant and an absent-dir variant to exercise both
// branches.
//
// Pitfall: Do NOT remove the `h.exists()` check on the base path decode —
// that guard enables the filesystem-guided fallback for underscore/slash
// ambiguity and is correct. Only the topic-loop guard (`if candidate.exists()`
// inside `for &topic in &parts[1..]`) is the bug.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-035)
fn projects_shows_topic_path_when_topic_dir_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Base project dir must exist so decode_project_display base-path decode succeeds.
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project base dir" );
  // Build storage key for the topic project (base + --commit suffix).
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  let topic_project_id = format!( "{encoded_base}--commit" );
  // Write session into the topic project dir. Do NOT create -commit dir on disk.
  common::write_test_session( &storage_root, &topic_project_id, "session-t01-absent-commit", 2 );

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
    s.contains( "/-commit" ),
    "display path must include topic '/-commit' even when dir is absent from disk; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.trim_end().ends_with( "myproject:" ) ),
    "display path must NOT be truncated to 'myproject:' when topic dir is absent; got:\n{s}"
  );
  assert!( s.contains( "session-t01-absent-commit" ), "session must appear in output; got:\n{s}" );
}

#[test]
fn projects_shows_topic_path_when_topic_dir_present()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Create base AND topic dir on disk — non-regression: behavior must match T01.
  let project   = root.path().join( "myproject" );
  let topic_dir = project.join( "-commit" );
  std::fs::create_dir_all( &topic_dir ).expect( "create myproject/-commit dir" );
  // Build storage key.
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  let topic_project_id = format!( "{encoded_base}--commit" );
  common::write_test_session( &storage_root, &topic_project_id, "session-t02-present-commit", 2 );

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
    s.contains( "/-commit" ),
    "display path must include topic '/-commit' when dir is present on disk; got:\n{s}"
  );
  assert!( s.contains( "session-t02-present-commit" ), "session must appear; got:\n{s}" );
}

#[test]
// bug_reproducer(issue-035)
fn projects_shows_default_topic_path_when_topic_dir_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project base dir" );
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  // "--default-topic" suffix: topic component "default-topic" → dir "-default_topic".
  let topic_project_id = format!( "{encoded_base}--default-topic" );
  // Write session. Do NOT create -default_topic dir on disk.
  common::write_test_session( &storage_root, &topic_project_id, "session-t03-absent-default-topic", 2 );

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
    s.contains( "/-default_topic" ),
    "display path must include '/-default_topic' even when dir is absent; got:\n{s}"
  );
  assert!(
    s.contains( "session-t03-absent-default-topic" ),
    "session must appear in output; got:\n{s}"
  );
}

#[test]
fn projects_shows_base_path_with_no_topic()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project dir" );
  common::write_path_project_session( &storage_root, &project, "session-t04-no-topic", 2 );

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
  assert!( s.contains( "session-t04-no-topic" ), "session must appear; got:\n{s}" );
  // No topic suffix in storage key — path must not include any topic component.
  assert!(
    !s.contains( "/-commit" ),
    "no topic in storage key — must not show /-commit; got:\n{s}"
  );
  assert!(
    !s.contains( "/-default_topic" ),
    "no topic in storage key — must not show /-default_topic; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Double-Topic Storage Key (issue-035, T05)
//
// Root Cause: decode_project_display now unconditionally joins ALL topic
// components (Fix issue-035). For a storage key `{base}--default-topic--commit`,
// split_storage_key yields topics ["default-topic", "commit"], and the loop
// extends: base → base/-default_topic → base/-default_topic/-commit.
//
// Why Not Caught: All issue-035 tests used single-topic suffixes only
// (`--commit` or `--default-topic`). Multiple `--` separators in one key
// were not exercised.
//
// Fix Applied: No code change needed — Fix(issue-035) already handles this
// correctly. This test guards against regression.
//
// Prevention: Whenever adding topic-extension tests, include a multi-topic
// variant to verify split_storage_key and the unconditional join loop together.
//
// Pitfall: Claude Code could in principle create `{base}--default-topic--commit`
// for a session from `base/-default_topic/-commit`. Both topic components must
// appear in the display path.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn projects_shows_both_topic_components_for_double_topic_key()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  std::fs::create_dir_all( &project ).expect( "create project base dir" );
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode project path" );
  // Storage key with two topic components: "--default-topic--commit".
  let topic_project_id = format!( "{encoded_base}--default-topic--commit" );
  // Write session. Do NOT create topic dirs on disk — absence must not drop either topic.
  common::write_test_session( &storage_root, &topic_project_id, "session-t05-double-topic", 2 );

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
    s.contains( "/-default_topic" ),
    "display path must include first topic '/-default_topic'; got:\n{s}"
  );
  assert!(
    s.contains( "/-commit" ),
    "display path must include second topic '/-commit'; got:\n{s}"
  );
  assert!( s.contains( "session-t05-double-topic" ), "session must appear in output; got:\n{s}" );
}
