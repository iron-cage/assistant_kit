//! Unit tests for `claude_version_core::config_resolve`.
//!
//! ## Purpose
//!
//! Verify the 4-layer resolution algorithm: env var → project config → user config → catalog default.
//!
//! ## Coverage
//!
//! - AT-01: Env var present → source=Env, overrides user config
//! - AT-02: Env var absent, key in user config → source=User
//! - AT-03: Key in project config, not in user config → source=Project
//! - AT-04: Key only in catalog defaults → source=Default
//! - AT-05: Key absent everywhere → source=Absent, value=None
//! - AT-06: Project config overrides user config when both have key
//!
//! ## Test Matrix
//!
//! | AT | Scenario | Source fn |
//! |----|----------|-----------|
//! | AT-01 | Env var present → Env source | `at01_002_env_overrides_user` |
//! | AT-02 | User config present, no env → User source | `at02_002_user_config_wins_without_env` |
//! | AT-03 | Project config present → Project source | `at03_002_project_config_key` |
//! | AT-04 | Catalog default → Default source | `at04_002_catalog_default_returned` |
//! | AT-05 | Absent everywhere → Absent source | `at05_002_all_layers_absent` |
//! | AT-06 | Project config overrides user config | `at06_002_project_overrides_user` |
//! | T06 | nested `env.*` catalog keys resolve via User layer, not Absent | `at07_002_lock_version_nested_env_resolves` |

use std::path::Path;
use tempfile::TempDir;
use claude_version_core::config_catalog::catalog;
use claude_version_core::config_resolve::{ resolve, Layer };
use claude_version_core::settings_io::{ set_setting, set_env_var };

// Helper: write a settings.json with a single key→value into {dir}/.claude/settings.json.
fn write_settings( dir : &Path, key : &str, value : &str )
{
  let claude_dir = dir.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  set_setting( &claude_dir.join( "settings.json" ), key, value ).unwrap();
}

// ─── AT-01: env var overrides user config ─────────────────────────────────────

// AT-01: CLAUDE_MODEL set → resolve("model") returns Env source overriding user config
#[ test ]
fn at01_002_env_overrides_user()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path();
  write_settings( home, "model", "claude-sonnet-5" );

  // Set env var in this process for the test duration.
  // Using a non-existent cwd to avoid project config interference.
  let no_project = TempDir::new().unwrap();

  std::env::set_var( "CLAUDE_MODEL", "claude-opus-4-8" );
  let rv = resolve( "model", home, no_project.path(), catalog() );
  std::env::remove_var( "CLAUDE_MODEL" );

  assert_eq!( rv.source, Layer::Env, "source must be Env when env var is set" );
  assert_eq!( rv.value.as_deref(), Some( "claude-opus-4-8" ), "value must come from env var" );
}

// ─── AT-02: user config wins when env absent ──────────────────────────────────

// AT-02: CLAUDE_MODEL unset, model in user config → resolve returns User source
#[ test ]
fn at02_002_user_config_wins_without_env()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path();
  write_settings( home, "model", "claude-haiku-4-5-20251001" );

  let no_project = TempDir::new().unwrap();

  std::env::remove_var( "CLAUDE_MODEL" );
  let rv = resolve( "model", home, no_project.path(), catalog() );

  assert_eq!( rv.source, Layer::User, "source must be User when env absent and user config has key" );
  assert_eq!( rv.value.as_deref(), Some( "claude-haiku-4-5-20251001" ),
    "value must come from user config" );
}

// ─── AT-03: project config key returned ───────────────────────────────────────

// AT-03: key in project config, not in user config → Project source
#[ test ]
fn at03_002_project_config_key()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();

  // Write to project .claude/settings.json (cwd itself).
  let proj_claude = project_dir.path().join( ".claude" );
  std::fs::create_dir_all( &proj_claude ).unwrap();
  set_setting( &proj_claude.join( "settings.json" ), "model", "claude-opus-4-8" ).unwrap();

  // User settings is empty (no model key).

  std::env::remove_var( "CLAUDE_MODEL" );
  let rv = resolve( "model", home_dir.path(), project_dir.path(), catalog() );

  assert_eq!( rv.source, Layer::Project, "source must be Project when key in project config" );
  assert_eq!( rv.value.as_deref(), Some( "claude-opus-4-8" ),
    "value must come from project config" );
}

// ─── AT-04: catalog default returned ──────────────────────────────────────────

// AT-04: env absent, no project config, no user config → catalog default for model
#[ test ]
fn at04_002_catalog_default_returned()
{
  let home_dir    = TempDir::new().unwrap();
  let no_project  = TempDir::new().unwrap();

  std::env::remove_var( "CLAUDE_MODEL" );
  let rv = resolve( "model", home_dir.path(), no_project.path(), catalog() );

  assert_eq!( rv.source, Layer::Default, "source must be Default when all other layers absent" );
  assert_eq!( rv.value.as_deref(), Some( "claude-sonnet-5" ),
    "catalog default for model must be claude-sonnet-5" );
}

// ─── AT-05: absent everywhere ─────────────────────────────────────────────────

// AT-05: non-catalog key with no env mapping → Absent source, None value
#[ test ]
fn at05_002_all_layers_absent()
{
  let home_dir   = TempDir::new().unwrap();
  let no_project = TempDir::new().unwrap();

  let rv = resolve( "myArbitraryKey", home_dir.path(), no_project.path(), catalog() );

  assert_eq!( rv.source, Layer::Absent, "source must be Absent for unknown key with no value" );
  assert!( rv.value.is_none(), "value must be None when absent in all layers" );
}

// ─── AT-06: project config overrides user config ──────────────────────────────

// AT-06: project has theme=dark, user has theme=light → Project source wins
#[ test ]
fn at06_002_project_overrides_user()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();

  // User config: theme=light
  write_settings( home_dir.path(), "theme", "light" );

  // Project config: theme=dark
  let proj_claude = project_dir.path().join( ".claude" );
  std::fs::create_dir_all( &proj_claude ).unwrap();
  set_setting( &proj_claude.join( "settings.json" ), "theme", "dark" ).unwrap();

  let rv = resolve( "theme", home_dir.path(), project_dir.path(), catalog() );

  assert_eq!( rv.source, Layer::Project, "Project must override User source" );
  assert_eq!( rv.value.as_deref(), Some( "dark" ), "project value must win over user value" );
}

// ─── T06: nested env.* keys resolve via User layer ────────────────────────────

// T06: env.DISABLE_AUTOUPDATER and env.DISABLE_UPDATES live inside the nested
// "env" sub-object, not as flat top-level keys — resolve() must look inside it
// rather than falling through to Absent (regression guard for the Step 3 fix).
#[ test ]
fn at07_002_lock_version_nested_env_resolves()
{
  let home_dir    = TempDir::new().unwrap();
  let no_project  = TempDir::new().unwrap();

  let claude_dir = home_dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let settings_file = claude_dir.join( "settings.json" );
  set_env_var( &settings_file, "DISABLE_AUTOUPDATER", "1" ).unwrap();
  set_env_var( &settings_file, "DISABLE_UPDATES", "1" ).unwrap();

  let rv1 = resolve( "env.DISABLE_AUTOUPDATER", home_dir.path(), no_project.path(), catalog() );
  assert_eq!( rv1.source, Layer::User, "env.DISABLE_AUTOUPDATER must resolve via User layer, not Absent" );
  assert_eq!( rv1.value.as_deref(), Some( "1" ) );

  let rv2 = resolve( "env.DISABLE_UPDATES", home_dir.path(), no_project.path(), catalog() );
  assert_eq!( rv2.source, Layer::User, "env.DISABLE_UPDATES must resolve via User layer, not Absent" );
  assert_eq!( rv2.value.as_deref(), Some( "1" ) );
}
