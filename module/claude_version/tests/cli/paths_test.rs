//! Integration tests for `.paths` — path enumeration, single-key lookup, format/verbosity rendering, error paths.
//!
//! Spec: `tests/docs/cli/command/16_paths.md` (IT-1 through IT-11)
//! Spec: `tests/docs/feature/009_path_discovery.md` (FT-1 through FT-7)
//!
//! ## IT tests (integration)
//! | IT | Description | Exit |
//! |----|-------------|------|
//! | IT-1 | No `key::` → show-all with all 5 keys labeled | 0 |
//! | IT-2 | `key::versions_dir` → single resolved path | 0 |
//! | IT-3 | `key::settings` → single resolved path | 0 |
//! | IT-4 | `format::json` → valid JSON object with 5 keys | 0 |
//! | IT-5 | `v::0` → plain unlabeled paths | 0 |
//! | IT-6 | `v::0` with `project_settings` unresolved → key omitted | 0 |
//! | IT-7 | `v::1` with `project_settings` unresolved → placeholder shown | 0 |
//! | IT-8 | `v::2` → labeled output with one-line description | 0 |
//! | IT-9 | HOME unset → exit 2 | 2 |
//! | IT-10 | `key::bogus` → exit 1 | 1 |
//! | IT-11 | `key::` (empty) → exit 1 | 1 |
//!
//! ## FT tests (feature acceptance)
//! | FT | Description | Exit |
//! |----|-------------|------|
//! | FT-1 | show-all exits 0; all 5 keys present with labels | 0 |
//! | FT-2 | `v::0` output is plain, unlabeled, one per line | 0 |
//! | FT-3 | single-key mode returns exactly one path | 0 |
//! | FT-4 | unresolvable `project_settings` shown as "(none found)" at v::1 | 0 |
//! | FT-5 | unresolvable `project_settings` omitted entirely at v::0 | 0 |
//! | FT-6 | HOME unset → exit 2 | 2 |
//! | FT-7 | `key::bogus` → exit 1 | 1 |

use crate::subprocess_helpers::{ assert_exit, run_clv_with_env, stdout, stderr };

/// All 5 known path labels, as they appear (with trailing colon) in labeled output.
const ALL_LABELS : [ &str; 5 ] =
  [ "settings:", "project_settings:", "versions_dir:", "binary_symlink:", "version_history_cache:" ];

// ─── IT tests ─────────────────────────────────────────────────────────────────

// IT-1: no key:: → show-all with all 5 keys labeled
#[ test ]
fn it01_paths_show_all_keys()
{
  let out = run_clv_with_env( &[ ".paths" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for label in ALL_LABELS
  {
    assert!(
      text.lines().any( | l | l.trim_start().starts_with( label ) ),
      "must contain a line labeled {label}: {text}"
    );
  }
}

// IT-2: key::versions_dir → single resolved path
#[ test ]
fn it02_paths_single_versions_dir()
{
  let out = run_clv_with_env( &[ ".paths", "key::versions_dir" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "/tmp/test_home/.local/share/claude/versions\n" );
}

// IT-3: key::settings → single resolved path
#[ test ]
fn it03_paths_single_settings()
{
  let out = run_clv_with_env( &[ ".paths", "key::settings" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "/tmp/test_home/.claude/settings.json\n" );
}

// IT-4: format::json → valid JSON object with 5 keys
#[ test ]
fn it04_paths_json_object_structure()
{
  let out = run_clv_with_env( &[ ".paths", "format::json" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "must be a JSON object: {text}" );
  for key in [ "\"settings\"", "\"project_settings\"", "\"versions_dir\"", "\"binary_symlink\"", "\"version_history_cache\"" ]
  {
    assert!( text.contains( key ), "JSON object must contain key {key}: {text}" );
  }
}

// IT-5: v::0 → plain unlabeled paths
#[ test ]
fn it05_paths_v0_unlabeled()
{
  let out = run_clv_with_env( &[ ".paths", "v::0" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    assert!( !line.contains( ':' ), "v::0 line must carry no label prefix: {line:?}" );
  }
}

// IT-6: v::0 with project_settings unresolved → key omitted from output
//
// Runs from an isolated `TempDir` (not `run_clv_with_env`'s inherited cwd) because
// the full-suite container mounts the real `~/.claude` at the `/workspace` git
// boundary (runbox.yml `plugin_mount`), which would otherwise make `project_settings`
// resolve to a real path instead of staying unresolved.
#[ test ]
fn it06_paths_v0_unresolved_omitted()
{
  let project_dir = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
    .args( [ ".paths", "v::0" ] )
    .env( "HOME", "/tmp/test_home" )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "project_settings" ), "v::0 must omit unresolved project_settings entirely: {text}" );
  let line_count = text.lines().filter( | l | !l.is_empty() ).count();
  assert_eq!( line_count, 4, "v::0 must show only the 4 resolvable keys when project_settings is unresolved: {text}" );
}

// IT-7: v::1 (default) with project_settings unresolved → "(none found)" placeholder shown
//
// Isolated TempDir cwd — see IT-6 above for why.
#[ test ]
fn it07_paths_v1_unresolved_placeholder()
{
  let project_dir = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
    .args( [ ".paths" ] )
    .env( "HOME", "/tmp/test_home" )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.lines().any( | l | l.trim_start().starts_with( "project_settings:" ) && l.contains( "(none found)" ) ),
    "v::1 (default) must show project_settings placeholder: {text}"
  );
}

// IT-8: v::2 → labeled output with one-line description
#[ test ]
fn it08_paths_v2_description()
{
  let out = run_clv_with_env( &[ ".paths", "key::binary_symlink", "v::2" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "binary_symlink:" ), "must contain label: {text}" );
  assert!( text.contains( "/tmp/test_home/.local/bin/claude" ), "must contain path: {text}" );
  let line_count = text.lines().filter( | l | !l.is_empty() ).count();
  assert!( line_count >= 2, "v::2 must include a description line beyond the path line: {text}" );
}

// IT-9: HOME unset → exit 2
#[ test ]
fn it09_paths_home_unset_exits_2()
{
  let out = run_clv_with_env( &[ ".paths" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
  assert!( stdout( &out ).is_empty(), "must not emit any path when HOME is unset: {}", stdout( &out ) );
}

// IT-10: key::bogus → exit 1
#[ test ]
fn it10_paths_invalid_key_exits_1()
{
  let out = run_clv_with_env( &[ ".paths", "key::bogus" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "bogus" ), "stderr must name the invalid key: {}", stderr( &out ) );
}

// IT-11: key:: (empty) → exit 1
#[ test ]
fn it11_paths_empty_key_exits_1()
{
  let out = run_clv_with_env( &[ ".paths", "key::" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key" ), "stderr must reference key:: or empty value: {err}" );
}

// ─── FT tests (feature acceptance) ───────────────────────────────────────────

// FT-1: show-all exits 0; all 5 keys present with labels
#[ test ]
fn ft1_show_all_exits_0_with_all_keys()
{
  let out = run_clv_with_env( &[ ".paths" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for label in ALL_LABELS
  {
    assert!(
      text.lines().any( | l | l.trim_start().starts_with( label ) ),
      "FT-1 must contain a line labeled {label}: {text}"
    );
  }
}

// FT-2: v::0 output is plain, unlabeled, one per line
#[ test ]
fn ft2_v0_output_is_unlabeled()
{
  let out = run_clv_with_env( &[ ".paths", "v::0" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    assert!( line.starts_with( '/' ) && !line.contains( ':' ), "FT-2 line must be a raw path, no labels: {line:?}" );
  }
}

// FT-3: single-key mode returns exactly one path
#[ test ]
fn ft3_single_key_returns_one_path()
{
  let out = run_clv_with_env( &[ ".paths", "key::versions_dir", "v::0" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "/tmp/test_home/.local/share/claude/versions\n" );
}

// FT-4: unresolvable project_settings shown as "(none found)" at v::1
//
// Isolated TempDir cwd — see IT-6 above for why.
#[ test ]
fn ft4_unresolvable_shown_as_none_found_v1()
{
  let project_dir = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
    .args( [ ".paths" ] )
    .env( "HOME", "/tmp/test_home" )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.lines().any( | l | l.trim_start().starts_with( "project_settings:" ) && l.contains( "(none found)" ) ),
    "FT-4 must show placeholder: {text}"
  );
}

// FT-5: unresolvable project_settings omitted entirely at v::0
//
// Isolated TempDir cwd — see IT-6 above for why. Without this, the assertion is
// trivially true regardless of resolution outcome (v::0 never emits labels either
// way), so this fix also makes the test actually test what it claims.
#[ test ]
fn ft5_unresolvable_omitted_at_v0()
{
  let project_dir = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_claude_version" ) )
    .args( [ ".paths", "v::0" ] )
    .env( "HOME", "/tmp/test_home" )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "project_settings" ),
    "FT-5 must omit project_settings entirely: {}", stdout( &out )
  );
}

// FT-6: HOME unset → exit 2
#[ test ]
fn ft6_home_unset_exits_2()
{
  let out = run_clv_with_env( &[ ".paths" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

// FT-7: key::bogus → exit 1
#[ test ]
fn ft7_invalid_key_exits_1()
{
  let out = run_clv_with_env( &[ ".paths", "key::bogus" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "bogus" ), "FT-7 must name the unknown key: {err}" );
  assert!( err.contains( "settings" ), "FT-7 must mention the valid key set: {err}" );
}
