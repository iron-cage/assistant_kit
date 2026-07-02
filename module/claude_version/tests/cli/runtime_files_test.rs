//! Integration tests for `.runtime_files` — path enumeration, format, and error paths.
//!
//! Spec: `tests/docs/cli/command/15_runtime_files.md` (IT-1 through IT-9)
//! Spec: `tests/docs/feature/008_runtime_file_discovery.md` (FT-1 through FT-5)
//!
//! ## IT tests (integration)
//! | IT | Description | Exit |
//! |----|-------------|------|
//! | IT-1 | Basic invocation: `version_history_cache.json` path present in stdout | 0 |
//! | IT-2 | Output lines are raw absolute paths only; no headers or decorations | 0 |
//! | IT-3 | Each non-empty output line begins with `/` (absolute path) | 0 |
//! | IT-4 | Path reflects current HOME value | 0 |
//! | IT-5 | Custom HOME produces correct path prefix | 0 |
//! | IT-6 | Command exits 0 when files do not exist on disk | 0 |
//! | IT-7 | All registered runtime file paths present in output | 0 |
//! | IT-8 | Pipeline composability: at least 1 output line | 0 |
//! | IT-9 | HOME unset → exit 2; no path output | 2 |
//!
//! ## FT tests (feature acceptance)
//! | FT | AC | Description | Exit |
//! |----|----|-------------|------|
//! | FT-1 | AC-1 | `.runtime_files` exits 0; `version_history_cache.json` path present | 0 |
//! | FT-2 | AC-2 | Output format: one absolute path per line, no headers or decorations | 0 |
//! | FT-3 | AC-3 | Command succeeds even when files do not exist on disk | 0 |
//! | FT-4 | AC-4 | Path is absolute and derived from HOME, not hardcoded | 0 |
//! | FT-5 | AC-5 | HOME unset → exit 2 | 2 |

use crate::subprocess_helpers::{ assert_exit, run_clv_with_env, stdout };

// ─── IT tests ─────────────────────────────────────────────────────────────────

// IT-1: basic invocation exits 0; version_history_cache.json path present
#[ test ]
fn it1_runtime_files_exits_0_with_cache_path()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "/tmp/test_home/.claude/.transient/version_history_cache.json" ),
    "must contain version_history_cache.json path: {text}"
  );
}

// IT-2: output lines are raw absolute paths only; no headers or decorations
#[ test ]
fn it2_output_is_raw_paths_only()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines()
  {
    assert!(
      line.starts_with( '/' ),
      "every non-empty output line must be a raw absolute path, got: {line:?}"
    );
  }
}

// IT-3: each non-empty output line begins with '/' (absolute path)
#[ test ]
fn it3_each_line_is_absolute()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let non_empty : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert!( !non_empty.is_empty(), "must have at least one output line" );
  for line in &non_empty
  {
    assert!( line.starts_with( '/' ), "path must be absolute, got: {line:?}" );
  }
}

// IT-4: path reflects current HOME value
#[ test ]
fn it4_path_reflects_home()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "/tmp/test_home" ),
    "path must use HOME value as prefix: {text}"
  );
}

// IT-5: custom HOME produces correct path prefix
#[ test ]
fn it5_custom_home_prefix()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/custom/path" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "/custom/path/.claude/.transient/version_history_cache.json" ),
    "must use custom HOME as path prefix: {text}"
  );
}

// IT-6: command exits 0 when files do not exist on disk
#[ test ]
fn it6_succeeds_when_files_absent()
{
  // /tmp/nonexistent_home_xyz does not have .claude/.transient/ on disk
  let out = run_clv_with_env(
    &[ ".runtime_files" ],
    &[ ( "HOME", "/tmp/nonexistent_home_xyz_runtime_files_test" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "/tmp/nonexistent_home_xyz_runtime_files_test/.claude/.transient/version_history_cache.json" ),
    "must still output path even when files absent on disk: {text}"
  );
}

// IT-7: all registered runtime file paths present in output
#[ test ]
fn it7_all_registered_paths_present()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // version_history_cache.json is the only currently registered runtime file
  assert!(
    text.contains( "/tmp/test_home/.claude/.transient/version_history_cache.json" ),
    "version_history_cache.json path must be present: {text}"
  );
}

// IT-8: pipeline composability — output has at least 1 line
#[ test ]
fn it8_pipeline_composable_line_count()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line_count = text.lines().filter( | l | !l.is_empty() ).count();
  assert!(
    line_count >= 1,
    "must have at least 1 output line for pipeline composability; got {line_count}"
  );
}

// IT-9: HOME unset → exit 2; no path output
#[ test ]
fn it9_home_unset_exits_2()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
  let text = stdout( &out );
  assert!(
    !text.contains( "/.claude/" ),
    "must not emit any path when HOME is unset: {text}"
  );
}

// ─── FT tests (feature acceptance) ───────────────────────────────────────────

// FT-1: exits 0; version_history_cache.json path present in output
#[ test ]
fn ft1_show_all_exits_0_with_cache_path()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "/tmp/test_home/.claude/.transient/version_history_cache.json" ),
    "FT-1 must contain version_history_cache.json path"
  );
}

// FT-2: output is one path per line, no headers or decorations
#[ test ]
fn ft2_output_format_one_path_per_line()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/tmp/test_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines()
  {
    assert!(
      line.starts_with( '/' ),
      "FT-2: every line must be a raw path, no headers: {line:?}"
    );
  }
}

// FT-3: command succeeds even when listed files do not exist on disk
#[ test ]
fn ft3_succeeds_when_files_absent()
{
  let out = run_clv_with_env(
    &[ ".runtime_files" ],
    &[ ( "HOME", "/tmp/nonexistent_ft3_home" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "/tmp/nonexistent_ft3_home/.claude/.transient/version_history_cache.json" ),
    "FT-3 must output path even when file absent on disk"
  );
}

// FT-4: path is absolute and derived from HOME (not relative or hardcoded)
#[ test ]
fn ft4_path_absolute_and_uses_home_expansion()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "/custom/user_home" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "/custom/user_home/.claude/.transient/version_history_cache.json" ),
    "FT-4 must expand HOME into path: {text}"
  );
  for line in text.lines()
  {
    assert!( line.starts_with( '/' ), "FT-4 path must be absolute: {line:?}" );
  }
}

// FT-5: HOME unset → exit 2
#[ test ]
fn ft5_home_unset_exits_2()
{
  let out = run_clv_with_env( &[ ".runtime_files" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}
