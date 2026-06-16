//! Interaction tests for Parameter Group 4: Config Identity.
//!
//! Implements test cases from:
//! - `tests/docs/cli/param_group/004_config_identity.md` (GI-1 through GI-10)
//!
//! Config Identity covers cross-parameter interactions for `key::`, `value::`,
//! `scope::`, and `unset::` within the `.config` command.
//!
//! # Test Matrix
//!
//! | GI   | Description | Exit |
//! |------|-------------|------|
//! | GI-1  | `key::K value::V` → set mode writes to user config | 0 |
//! | GI-2  | `key::K value::V scope::project` → set mode writes to project config | 0 |
//! | GI-3  | `key::K unset::1` → unset mode removes key from user config | 0 |
//! | GI-4  | `key::K unset::1 scope::project` → unset mode removes key from project config | 0 |
//! | GI-5  | `key::K value::V unset::1` → mutual exclusion; exit 1 | 1 |
//! | GI-6  | `value::V` without `key::K` → key required; exit 1 | 1 |
//! | GI-7  | `unset::1` without `key::K` → key required; exit 1 | 1 |
//! | GI-8  | `scope::project` without write operation → scope applies to writes only; exit 1 | 1 |
//! | GI-9  | `key::K` alone → get mode; value printed; exit 0 | 0 |
//! | GI-10 | `key::K value::V dry::1` → preview; no file modification; exit 0 | 0 |

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm_with_env, stdout, write_settings };

// ─── GI-1: key::K value::V → set mode writes to user config ──────────────────

/// GI-1: `key::K` `value::V` → set mode; key written to user config; exit 0
#[ test ]
fn config_identity_gi1_set_mode_writes_user_config()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::plugin", "value::enabled" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"plugin\"" ),
    "user settings.json must contain the written key: {content}" );
  assert!( content.contains( "enabled" ),
    "user settings.json must contain the written value: {content}" );
}

// ─── GI-2: key::K value::V scope::project → set mode writes to project config ─

/// GI-2: `key::K` `value::V` `scope::project` → set mode writes to project config; user config untouched; exit 0
#[ test ]
fn config_identity_gi2_set_mode_project_scope()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();
  let bin         = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::plugin", "value::enabled", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );

  let proj_settings = project_dir.path().join( ".claude/settings.json" );
  assert!( proj_settings.exists(), "project settings.json must be created" );
  let content = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( content.contains( "enabled" ),
    "project settings.json must contain the written value: {content}" );
  // User config must remain untouched.
  assert!( !home_dir.path().join( ".claude/settings.json" ).exists(),
    "user settings.json must not be created by scope::project write" );
}

// ─── GI-3: key::K unset::1 → unset mode removes key from user config ─────────

/// GI-3: `key::K` `unset::1` → unset mode removes key from user config; other keys preserved; exit 0
#[ test ]
fn config_identity_gi3_unset_mode_removes_from_user_config()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "plugin", "enabled" ), ( "model", "claude-sonnet-4-6" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::plugin", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( !content.contains( "\"plugin\"" ),
    "plugin key must be absent from user settings after unset: {content}" );
  assert!( content.contains( "model" ),
    "other keys must be preserved after unset: {content}" );
}

// ─── GI-4: key::K unset::1 scope::project → unset from project config ─────────

/// GI-4: `key::K` `unset::1` `scope::project` → key removed from project config; exit 0
#[ test ]
fn config_identity_gi4_unset_mode_project_scope()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();
  let bin         = env!( "CARGO_BIN_EXE_claude_version" );

  // Write a key to project config first.
  std::process::Command::new( bin )
    .args( [ ".config", "key::plugin", "value::enabled", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();

  let proj_settings = project_dir.path().join( ".claude/settings.json" );
  let before = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( before.contains( "\"plugin\"" ),
    "plugin must be present in project settings before GI-4 unset: {before}" );

  // Unset from project scope.
  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::plugin", "unset::1", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( !after.contains( "\"plugin\"" ),
    "plugin key must be absent from project settings after GI-4 unset: {after}" );
}

// ─── GI-5: key::K value::V unset::1 → mutual exclusion; exit 1 ──────────────

/// GI-5: `key::K` `value::V` `unset::1` → `value::` and `unset::` are mutually exclusive; exit 1
#[ test ]
fn config_identity_gi5_value_unset_mutual_exclusion_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "unset::1" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── GI-6: value::V without key::K → key required; exit 1 ────────────────────

/// GI-6: `value::V` without `key::K` → `key::` is required when `value::` is provided; exit 1
#[ test ]
fn config_identity_gi6_value_without_key_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "value::somevalue" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── GI-7: unset::1 without key::K → key required; exit 1 ───────────────────

/// GI-7: `unset::1` without `key::K` → `key::` is required when `unset::1`; exit 1
#[ test ]
fn config_identity_gi7_unset_without_key_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "unset::1" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── GI-8: scope::project without write operation → exit 1 ───────────────────

/// GI-8: `scope::project` without write operation → scope applies to writes only; exit 1
#[ test ]
fn config_identity_gi8_scope_without_write_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "scope::project" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── GI-9: key::K alone → get mode; value printed ───────────────────────────

/// GI-9: `key::K` alone → get mode; resolved value printed with source annotation; exit 0
#[ test ]
fn config_identity_gi9_key_alone_get_mode()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "dark" ),
    "get mode must print the resolved value: {text}" );
}

// ─── GI-10: key::K value::V dry::1 → preview; no file modification ────────────

/// GI-10: `key::K` `value::V` `dry::1` → cross-group: preview output; settings.json unchanged; exit 0
#[ test ]
fn config_identity_gi10_dry_run_no_file_modification()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "light" ) ] );

  let settings_path = dir.path().join( ".claude/settings.json" );
  let before        = std::fs::read_to_string( &settings_path ).unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( &settings_path ).unwrap();
  assert_eq!( before, after,
    "settings.json content must be unchanged after dry::1" );
  assert!(
    !after.contains( "\"theme\": \"dark\"" ) && !after.contains( "\"theme\":\"dark\"" ),
    "settings.json must not contain the new value after dry::1: {after}"
  );
}
