//! `PathKey` type compliance and validation tests for `claude_version`.
//!
//! Spec: `tests/docs/cli/type/09_path_key.md` (TC-1 through TC-9)
//!
//! # Coverage Map
//!
//! | Spec | ID | Function |
//! |------|----|----------|
//! | cli/type/09_path_key.md | TC-1 | `path_key_tc1_settings_resolves` |
//! | cli/type/09_path_key.md | TC-2 | `path_key_tc2_versions_dir_resolves` |
//! | cli/type/09_path_key.md | TC-3 | `path_key_tc3_binary_symlink_resolves` |
//! | cli/type/09_path_key.md | TC-4 | `path_key_tc4_version_history_cache_resolves` |
//! | cli/type/09_path_key.md | TC-5 | `path_key_tc5_project_settings_resolves_or_placeholder` |
//! | cli/type/09_path_key.md | TC-6 | `path_key_tc6_absent_shows_all_keys` |
//! | cli/type/09_path_key.md | TC-7 | `path_key_tc7_mixed_case_exits_1` |
//! | cli/type/09_path_key.md | TC-8 | `path_key_tc8_unknown_variant_exits_1` |
//! | cli/type/09_path_key.md | TC-9 | `path_key_tc9_empty_exits_1` |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv_with_env, stderr, stdout };

// TC-1: key::settings → settings.json path
#[ test ]
fn path_key_tc1_settings_resolves()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::settings" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), format!( "{home}/.claude/settings.json\n" ) );
}

// TC-2: key::versions_dir → versions directory path
#[ test ]
fn path_key_tc2_versions_dir_resolves()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::versions_dir" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), format!( "{home}/.local/share/claude/versions\n" ) );
}

// TC-3: key::binary_symlink → symlink path
#[ test ]
fn path_key_tc3_binary_symlink_resolves()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::binary_symlink" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), format!( "{home}/.local/bin/claude\n" ) );
}

// TC-4: key::version_history_cache → cache path
#[ test ]
fn path_key_tc4_version_history_cache_resolves()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::version_history_cache" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), format!( "{home}/.claude/.transient/version_history_cache.json\n" ) );
}

// TC-5: key::project_settings → resolved path when an ancestor .claude/settings.json exists
//
// Isolated project_dir with a real .claude/settings.json — the "resolves" half of the
// Behavioral Divergence Pair with TC-2; the "(none found)" half is already covered by
// paths_test.rs's IT-6/IT-7/FT-4/FT-5.
#[ test ]
fn path_key_tc5_project_settings_resolves_or_placeholder()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();

  let proj_claude = project_dir.path().join( ".claude" );
  std::fs::create_dir_all( &proj_claude ).unwrap();
  std::fs::write( proj_claude.join( "settings.json" ), "{}" ).unwrap();

  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( bin )
    .args( [ ".paths", "key::project_settings" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), format!( "{}\n", proj_claude.join( "settings.json" ).display() ) );
}

// TC-6: absent key:: → all 5 paths shown
#[ test ]
fn path_key_tc6_absent_shows_all_keys()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for label in [ "settings", "project_settings", "versions_dir", "binary_symlink", "version_history_cache" ]
  {
    assert!( text.contains( label ), "show-all must contain key {label}: {text}" );
  }
}

// TC-7: key::Settings → exit 1 (case-sensitive)
#[ test ]
fn path_key_tc7_mixed_case_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::Settings" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "key::Settings rejection must produce an error message: {err}" );
}

// TC-8: key::bogus → exit 1 (unknown variant)
#[ test ]
fn path_key_tc8_unknown_variant_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "bogus" ), "stderr must name the invalid key: {err}" );
  assert!( err.contains( "settings" ), "stderr must list the valid key set: {err}" );
}

// TC-9: key:: (empty) → exit 1
#[ test ]
fn path_key_tc9_empty_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env( &[ ".paths", "key::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key" ), "stderr must reference key:: or empty value: {err}" );
}
