//! Edge case tests for the `scope::` parameter.
//!
//! Implements test cases from:
//! - `tests/docs/cli/param/011_scope.md` (EC-1 through EC-7)
//!
//! # Test Matrix
//!
//! | EC  | Description | Exit |
//! |-----|-------------|------|
//! | EC-1 | `scope::user` writes to `~/.claude/settings.json` | 0 |
//! | EC-2 | `scope::project` writes to `{cwd}/.claude/settings.json` | 0 |
//! | EC-3 | `scope::invalid` → unknown scope value | 1 |
//! | EC-4 | `scope::` (empty value) → exit 1 | 1 |
//! | EC-5 | `scope::user` without `key::` and `value::` → scope applies to writes only | 1 |
//! | EC-6 | `scope::project` with `key::K value::V` creates `.claude/` directory if absent | 0 |
//! | EC-7 | `scope::project` in show-all mode (no `key::`) → exit 1 | 1 |

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm_with_env };

// ─── EC-1: scope::user writes to user settings ────────────────────────────────

/// EC-1: `scope::user` (explicit) writes key to ~/.claude/settings.json; exit 0
#[ test ]
fn scope_ec1_user_writes_to_user_settings()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::user" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"theme\"" ),
    "user settings.json must contain the written key: {content}" );
  assert!( content.contains( "dark" ),
    "user settings.json must contain the written value: {content}" );
}

// ─── EC-2: scope::project writes to project settings ─────────────────────────

/// EC-2: `scope::project` writes to {cwd}/.claude/settings.json; user config untouched; exit 0
#[ test ]
fn scope_ec2_project_writes_to_project_settings()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();
  let bin         = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::theme", "value::dark", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );

  let proj_settings = project_dir.path().join( ".claude/settings.json" );
  assert!( proj_settings.exists(),
    "project settings.json must be created by scope::project write" );
  let content = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( content.contains( "dark" ),
    "project settings.json must contain the written value: {content}" );
  // User settings must NOT be created.
  assert!( !home_dir.path().join( ".claude/settings.json" ).exists(),
    "user settings.json must not be created by scope::project write" );
}

// ─── EC-3: scope::invalid → exit 1 ──────────────────────────────────────────

/// EC-3: `scope::invalid` → unknown scope value rejected; exit 1
#[ test ]
fn scope_ec3_invalid_scope_value_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::invalid" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── EC-4: scope:: (empty value) → exit 1 ────────────────────────────────────

/// EC-4: `scope::` (empty value) → invalid scope; exit 1
#[ test ]
fn scope_ec4_empty_scope_value_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── EC-5: scope::user without key:: and value:: → exit 1 ───────────────────

/// EC-5: `scope::user` without `key::` and `value::` → scope applies to writes only; exit 1
#[ test ]
fn scope_ec5_scope_without_write_op_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "scope::user" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── EC-6: scope::project creates .claude/ directory if absent ───────────────

/// EC-6: `scope::project` creates .claude/ directory when it does not exist; exit 0
#[ test ]
fn scope_ec6_project_creates_directory_when_absent()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();
  let bin         = env!( "CARGO_BIN_EXE_claude_version" );

  // Verify .claude/ does not pre-exist.
  assert!(
    !project_dir.path().join( ".claude" ).exists(),
    ".claude/ must not exist before scope_ec6 runs"
  );

  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::plugin", "value::test", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .unwrap();
  assert_exit( &out, 0 );

  assert!( project_dir.path().join( ".claude" ).exists(),
    ".claude/ directory must be created by scope::project write" );
  assert!( project_dir.path().join( ".claude/settings.json" ).exists(),
    ".claude/settings.json must be created" );
}

// ─── EC-7: scope::project in show-all mode → exit 1 ─────────────────────────

/// EC-7: `scope::project` with no `key::` (show-all mode) → scope applies to writes only; exit 1
#[ test ]
fn scope_ec7_project_in_show_all_mode_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "scope::project" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}
