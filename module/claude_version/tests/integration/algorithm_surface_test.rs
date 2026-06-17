//! Algorithm surface tests (AC- prefix) for `claude_version`.
//!
//! Implements test cases from:
//! - `tests/docs/algorithm/001_settings_type_inference.md`
//! - `tests/docs/algorithm/002_config_resolution.md`
//!
//! # Coverage Map — 001 (Settings Type Inference)
//!
//! | AC-ID | Function |
//! |-------|----------|
//! | AC-1  | covered by `tc322_settings_set_stores_boolean_true` |
//! | AC-2  | covered by `tc323_settings_set_stores_boolean_false` |
//! | AC-3  | covered by `tc324_settings_set_zero_stored_as_number` |
//! | AC-4  | `ac004_float_inference` |
//! | AC-5  | `ac005_nan_stores_string` |
//! | AC-6  | covered by `tc326_settings_set_stores_string` |
//!
//! # Coverage Map — 002 (Config Resolution)
//!
//! | AC-ID | Function |
//! |-------|----------|
//! | AC-1  | `ac01_002_env_overrides_user` |
//! | AC-2  | `ac02_002_user_config_wins_without_env` |
//! | AC-3  | `ac03_002_project_config_key` |
//! | AC-4  | `ac04_002_catalog_default_returned` |
//! | AC-5  | `ac05_002_all_layers_absent` |
//! | AC-6  | `ac06_002_project_overrides_user` |
//! | AC-7  | `ac07_002_home_unset_skips_user_config` |
//! | AC-8  | `ac08_002_ancestor_project_config_found` |

use tempfile::TempDir;
use claude_version_core::config_catalog;
use claude_version_core::config_resolve::{ resolve, Layer };

use crate::helpers::{ assert_exit, run_clm_with_env };

// ─── AC-4: finite float stored as JSON float ──────────────────────────────────

// AC-4: value::3.14 → settings.json contains "pi": 3.14 (unquoted float)
#[ test ]
fn ac004_float_inference()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::pi", "value::3.14" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"pi\": 3.14" ), "3.14 must be stored as bare float: {content}" );
  assert!( !content.contains( "\"pi\": \"3.14\"" ), "3.14 must NOT be quoted: {content}" );
}

// ─── AC-5: NaN / inf strings stored as JSON string ───────────────────────────

// AC-5: value::nan → settings.json contains "bad": "nan" (quoted — not a float)
#[ test ]
fn ac005_nan_stores_string()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::bad", "value::nan" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"bad\": \"nan\"" ), "nan must be stored as quoted string: {content}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// 002: Config Resolution — 4-layer priority tests
// Calls `config_resolve::resolve()` directly against the catalog.
// Each test uses a fresh TempDir for home_dir and cwd to isolate file state.
// Nextest runs every test in its own process, so env var mutations are safe.
// ─────────────────────────────────────────────────────────────────────────────

// AC-1: env var present → source=Env, overrides user config
#[ test ]
fn ac01_002_env_overrides_user()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  crate::helpers::write_settings( home_dir.path(), &[ ( "model", "claude-sonnet-4-6" ) ] );
  std::env::set_var( "CLAUDE_MODEL", "claude-opus-4-6" );
  let r = resolve( "model", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::Env,                             "env must beat user config: got {:?}", r.source );
  assert_eq!( r.value,  Some( "claude-opus-4-6".to_string() ),  "wrong value: {:?}", r.value );
}

// AC-2: env var absent, key in user config → source=User
#[ test ]
fn ac02_002_user_config_wins_without_env()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  crate::helpers::write_settings( home_dir.path(), &[ ( "model", "claude-haiku-4-5-20251001" ) ] );
  std::env::remove_var( "CLAUDE_MODEL" );
  let r = resolve( "model", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::User,                                    "user config must win when env absent: got {:?}", r.source );
  assert_eq!( r.value,  Some( "claude-haiku-4-5-20251001".to_string() ), "wrong value: {:?}", r.value );
}

// AC-3: key in project config, not in user config → source=Project
#[ test ]
fn ac03_002_project_config_key()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  // Write project settings at cwd/.claude/settings.json (found by ancestor walk).
  crate::helpers::write_settings( cwd.path(), &[ ( "model", "claude-opus-4-6" ) ] );
  std::env::remove_var( "CLAUDE_MODEL" );
  let r = resolve( "model", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::Project,                        "project config must supply value: got {:?}", r.source );
  assert_eq!( r.value,  Some( "claude-opus-4-6".to_string() ), "wrong value: {:?}", r.value );
}

// AC-4: key only in catalog defaults → source=Default
#[ test ]
fn ac04_002_catalog_default_returned()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  // No settings files written; "model" has catalog default "claude-sonnet-4-6".
  std::env::remove_var( "CLAUDE_MODEL" );
  let r = resolve( "model", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::Default,                           "catalog default must be returned: got {:?}", r.source );
  assert_eq!( r.value,  Some( "claude-sonnet-4-6".to_string() ),  "wrong default value: {:?}", r.value );
}

// AC-5: key absent everywhere → source=Absent, value=None
#[ test ]
fn ac05_002_all_layers_absent()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  // "myArbitraryKey" has no env mapping, no project/user config, no catalog entry.
  let r = resolve( "myArbitraryKey", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::Absent, "absent key must return Absent: got {:?}", r.source );
  assert_eq!( r.value,  None,          "absent key must return None value: {:?}", r.value );
}

// AC-6: project config overrides user config when both have same key
#[ test ]
fn ac06_002_project_overrides_user()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  crate::helpers::write_settings( cwd.path(),      &[ ( "theme", "dark"  ) ] );
  crate::helpers::write_settings( home_dir.path(), &[ ( "theme", "light" ) ] );
  let r = resolve( "theme", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::Project,            "project must beat user for same key: got {:?}", r.source );
  assert_eq!( r.value,  Some( "dark".to_string() ), "wrong value: {:?}", r.value );
}

// AC-7: HOME unset → user config layer absent → catalog default returned
// resolve() takes home_dir as a parameter; when home_dir has no .claude/settings.json
// (the state the CLI produces when HOME is unset), Step 3 is skipped and Step 4
// returns the catalog default.
#[ test ]
fn ac07_002_home_unset_skips_user_config()
{
  let home_dir = TempDir::new().unwrap();
  let cwd      = TempDir::new().unwrap();
  // No .claude/settings.json written to home_dir — simulates absent HOME at CLI layer.
  // No project config in cwd.
  std::env::remove_var( "CLAUDE_MODEL" );
  let r = resolve( "theme", home_dir.path(), cwd.path(), config_catalog::catalog() );
  assert_eq!( r.source, Layer::Default,                "absent user config must fall through to Default: got {:?}", r.source );
  assert_eq!( r.value,  Some( "system".to_string() ),  "catalog default for 'theme' must be \"system\": {:?}", r.value );
}

// AC-8: project config found in ancestor directory
// find_project_config_file walks upward from cwd; project settings written to parent.
// cwd is a subdirectory of parent with no local .claude/settings.json.
#[ test ]
fn ac08_002_ancestor_project_config_found()
{
  let parent   = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  crate::helpers::write_settings( parent.path(), &[ ( "preferredVersionSpec", "beta" ) ] );
  // child is a subdirectory of parent — it has no .claude/settings.json of its own.
  let child = parent.path().join( "subdir" );
  std::fs::create_dir_all( &child ).unwrap();
  std::env::remove_var( "CLAUDE_MODEL" );
  let r = resolve( "preferredVersionSpec", home_dir.path(), &child, config_catalog::catalog() );
  assert_eq!( r.source, Layer::Project,              "ancestor project config must supply value: got {:?}", r.source );
  assert_eq!( r.value,  Some( "beta".to_string() ),  "ancestor config value must be returned: {:?}", r.value );
}
