//! Integration tests for the `.config` command.
//!
//! Implements test cases from:
//! - `tests/docs/cli/command/13_config.md` (IT-1 through IT-17)
//! - `tests/docs/feature/006_config_command.md` (FT-01 through FT-12)
//! - `tests/docs/cli/type/06_config_scope.md` (TC-1 through TC-6)
//! - `tests/docs/cli/type/07_config_key.md` (TC-1 through TC-6)
//!
//! # Test Matrix (IT)
//!
//! | IT | Description | Mode | Exit |
//! |----|-------------|------|------|
//! | IT-1  | No params → show-all with source labels | show-all | 0 |
//! | IT-2  | `key::theme` → get with source annotation | get | 0 |
//! | IT-3  | `key::theme value::dark` → set user, bool inferred | set | 0 |
//! | IT-4  | `key::model value::claude-opus-4-6 scope::project` → project write | set | 0 |
//! | IT-5  | `key::theme unset::1` → key removed from user settings | unset | 0 |
//! | IT-6  | `format::json` → JSON with source fields | show-all | 0 |
//! | IT-7  | `key::model` with `CLAUDE_MODEL` set → shows env value | get | 0 |
//! | IT-8  | `key::unknownArbitraryKey value::v` → accepted, written | set | 0 |
//! | IT-9  | `key::model` no env/config → shows catalog default | get | 0 |
//! | IT-10 | `key::theme value::dark dry::1` → preview, no write | set | 0 |
//! | IT-11 | `value::v` without `key::` → exit 1 | — | 1 |
//! | IT-12 | `unset::1` without `key::` → exit 1 | — | 1 |
//! | IT-13 | `value::v unset::1 key::k` → exit 1 (mutually exclusive) | — | 1 |
//! | IT-14 | `scope::global` → exit 1 (invalid value) | — | 1 |
//! | IT-15 | `format::xml` → exit 1 | — | 1 |
//! | IT-16 | `HOME` unset → exit 2 | — | 2 |
//! | IT-17 | `dry::2` → exit 1, out-of-range | — | 1 |
//!
//! # Test Matrix (FT)
//!
//! | FT | AC | Scenario | Exit |
//! |----|----|----------|------|
//! | FT-01 | AC-01 | show-all prints resolved settings in text format | 0 |
//! | FT-02 | AC-02 | get prints value with source layer annotation | 0 |
//! | FT-03 | AC-03 | set writes to user settings.json with type inference | 0 |
//! | FT-04 | AC-04 | set with scope::project writes to project settings.json | 0 |
//! | FT-05 | AC-05 | unset removes key from user settings | 0 |
//! | FT-06 | AC-06 | format::json returns resolved settings with source fields | 0 |
//! | FT-07 | AC-07 | env var (CLAUDE_MODEL) overrides user config for model key | 0 |
//! | FT-08 | AC-08 | absent key shows default (hasCompletedOnboarding→false) | 0 |
//! | FT-09 | AC-09 | dry::1 previews set, no file change | 0 |
//! | FT-10 | AC-10 | HOME unset → exit 2 | 2 |
//! | FT-11 | AC-11 | non-catalog key accepted and written | 0 |
//! | FT-12 | AC-12 | catalog default for model is claude-sonnet-4-6 | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clm_with_env, stdout, write_settings };

// ─── IT-1: show-all with source labels ───────────────────────────────────────

// IT-1: no params → show-all includes source labels; exit 0
#[ test ]
fn it01_config_show_all_source_labels()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".config" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // show-all must include resolved keys with source annotations
  assert!( text.contains( "(user)" ) || text.contains( "(default)" ),
    "show-all must include source annotations: {text}" );
  assert!( text.contains( "theme" ), "show-all must include theme key: {text}" );
}

// ─── IT-2: get with source annotation ────────────────────────────────────────

// IT-2: key::theme → get mode; output includes value and source annotation; exit 0
#[ test ]
fn it02_config_get_shows_source_annotation()
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
  assert!( text.contains( "dark" ), "get must show value: {text}" );
  assert!( text.contains( "(user)" ), "get must show source annotation: {text}" );
}

// ─── IT-3: set user scope ────────────────────────────────────────────────────

// IT-3: key::theme value::dark → write to user settings.json; exit 0
#[ test ]
fn it03_config_set_user_scope()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"theme\"" ), "settings.json must contain theme key: {content}" );
  assert!( content.contains( "dark" ), "settings.json must contain value dark: {content}" );
}

// ─── IT-4: set project scope ─────────────────────────────────────────────────

// IT-4: key::model value::claude-opus-4-6 scope::project → writes project settings; exit 0
#[ test ]
fn it04_config_set_project_scope()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();
  let project     = project_dir.path().to_str().unwrap();

  // std::env::current_dir() reads /proc/self/cwd on Linux — must set cwd via Command.
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::model", "value::claude-opus-4-6", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
  let proj_settings = project_dir.path().join( ".claude/settings.json" );
  assert!( proj_settings.exists(), "project settings.json must be created" );
  let content = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( content.contains( "claude-opus-4-6" ), "project settings must contain model value: {content}" );
  // User settings must NOT be changed.
  assert!( !home_dir.path().join( ".claude/settings.json" ).exists()
    || !std::fs::read_to_string( home_dir.path().join( ".claude/settings.json" ) )
         .unwrap_or_default().contains( "claude-opus-4-6" ),
    "user settings must not be modified by project scope write" );
}

// ─── IT-5: unset removes key ─────────────────────────────────────────────────

// IT-5: key::theme unset::1 → theme removed from user settings; other keys preserved; exit 0
#[ test ]
fn it05_config_unset_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ), ( "autoUpdates", "true" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( !content.contains( "\"theme\"" ), "theme key must be removed: {content}" );
  assert!( content.contains( "autoUpdates" ), "other keys must be preserved: {content}" );
}

// ─── IT-6: format::json show-all ─────────────────────────────────────────────

// IT-6: format::json → JSON object with source fields per key; exit 0
#[ test ]
fn it06_config_show_all_json_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "format::json" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( '{' ), "JSON output must start with {{: {text}" );
  assert!( text.contains( "\"source\"" ), "JSON output must contain source field: {text}" );
  assert!( text.contains( "\"theme\"" ), "JSON output must contain theme key: {text}" );
}

// ─── IT-7: env var overrides for model ───────────────────────────────────────

// IT-7: key::model with CLAUDE_MODEL set → shows env value with (env) annotation; exit 0
#[ test ]
fn it07_config_get_env_override()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::model" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "claude-opus-4-6" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-opus-4-6" ), "must show env value: {text}" );
  assert!( text.contains( "(env)" ), "must show (env) source annotation: {text}" );
}

// ─── IT-8: arbitrary key accepted ────────────────────────────────────────────

// IT-8: key::unknownArbitraryKey value::v → written to settings.json without error; exit 0
#[ test ]
fn it08_config_arbitrary_key_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::unknownArbitraryKey", "value::myval" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "unknownArbitraryKey" ), "arbitrary key must be written: {content}" );
  assert!( content.contains( "myval" ), "arbitrary value must be written: {content}" );
}

// ─── IT-9: catalog default for model ─────────────────────────────────────────

// IT-9: key::model with no env/config → shows catalog default; exit 0
//
// Runs the subprocess with current_dir set to the isolated temp dir to prevent
// the project config walk from finding /workspace/.claude/settings.json (the
// ~/.claude mount in the container).
#[ test ]
fn it09_config_catalog_default_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let bin  = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::model" ] )
    .env( "HOME", home )
    .env( "CLAUDE_MODEL", "" )
    .current_dir( dir.path() )
    .output()
    .expect( "failed to execute claude_version binary" );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-sonnet-4-6" ), "must show catalog default: {text}" );
  assert!( text.contains( "(default)" ), "must show (default) source annotation: {text}" );
}

// ─── IT-10: dry-run shows preview ────────────────────────────────────────────

// IT-10: key::theme value::dark dry::1 → shows preview, no file change; exit 0
#[ test ]
fn it10_config_set_dry_run_no_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "light" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry-run must show [dry-run] prefix: {text}" );
  // File must remain unchanged.
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "light" ), "file must remain unchanged: {content}" );
  assert!( !content.contains( "\"theme\": \"dark\"" ) && !content.contains( "\"theme\":\"dark\"" ),
    "file must not contain new value: {content}" );
}

// ─── IT-11: value:: without key:: → exit 1 ───────────────────────────────────

// IT-11: value::v without key:: → invalid combination; exit 1
#[ test ]
fn it11_config_value_without_key_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "value::somevalue" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── IT-12: unset::1 without key:: → exit 1 ──────────────────────────────────

// IT-12: unset::1 without key:: → invalid combination; exit 1
#[ test ]
fn it12_config_unset_without_key_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "unset::1" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── IT-13: value:: and unset::1 together → exit 1 ───────────────────────────

// IT-13: key::k value::v unset::1 → mutually exclusive; exit 1
#[ test ]
fn it13_config_value_and_unset_together_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::k", "value::v", "unset::1" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── IT-14: scope::global → exit 1 ──────────────────────────────────────────

// IT-14: scope::global → unrecognised scope value; exit 1
#[ test ]
fn it14_config_invalid_scope_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "scope::global" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── IT-15: format::xml → exit 1 ─────────────────────────────────────────────

// IT-15: format::xml → unrecognised format; exit 1
#[ test ]
fn it15_config_invalid_format_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "format::xml" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── IT-16: HOME unset → exit 2 ──────────────────────────────────────────────

// IT-16: HOME unset (or empty) → cannot resolve paths; exit 2
#[ test ]
fn it16_config_home_unset_exits_2()
{
  let out = run_clm_with_env(
    &[ ".config", "key::model" ],
    &[ ( "HOME", "" ) ],
  );
  assert_exit( &out, 2 );
}

// ─── IT-17: dry::2 → exit 1 (out-of-range boolean) ──────────────────────────

// IT-17: dry::2 → boolean param out of range; rejected by unilang; exit 1
#[ test ]
fn it17_config_dry_out_of_range_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "dry::2" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ═══════════════════════════════════════════════════════════════════════════════
// FT tests (feature/006_config_command.md)
// ═══════════════════════════════════════════════════════════════════════════════

// ─── FT-01: AC-01 show-all text format ───────────────────────────────────────

// FT-1: .config (no params) prints resolved settings in text format; exit 0
//
// Uses a separate cwd_dir (not HOME) so the project config walk does not find
// HOME/.claude/settings.json (which would mis-classify user config as project config).
// Also avoids the container-mounted /workspace/.claude/settings.json.
#[ test ]
fn ft1_006_config_show_all_text()
{
  let dir     = TempDir::new().unwrap();
  let cwd_dir = TempDir::new().unwrap();
  let home    = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );
  let bin = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".config" ] )
    .env( "HOME", home )
    .env( "CLAUDE_MODEL", "" )
    .current_dir( cwd_dir.path() )
    .output()
    .expect( "failed to execute claude_version binary" );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Catalog default for model must appear with (default) annotation.
  assert!( text.contains( "claude-sonnet-4-6" ), "must include catalog default for model: {text}" );
  assert!( text.contains( "(default)" ), "must include (default) annotation: {text}" );
  // User setting for theme must appear with (user) annotation.
  assert!( text.contains( "theme" ), "must include theme: {text}" );
  assert!( text.contains( "(user)" ), "must include (user) annotation: {text}" );
}

// ─── FT-02: AC-02 get shows source layer ─────────────────────────────────────

// FT-2: .config key::theme with user config → shows value and (user) source; exit 0
#[ test ]
fn ft2_006_config_get_shows_source()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "light" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "light" ), "must show value: {text}" );
  assert!( text.contains( "(user)" ), "must show (user) source: {text}" );
}

// ─── FT-03: AC-03 set writes with type inference ─────────────────────────────

// FT-3: .config key::autoUpdates value::false → settings.json contains false (JSON bool); exit 0
#[ test ]
fn ft3_006_config_set_user_scope()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::autoUpdates", "value::false" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  // Type inference must store "false" as bare JSON boolean, not quoted string.
  assert!( content.contains( "\"autoUpdates\": false" ),
    "autoUpdates must be stored as JSON bool false: {content}" );
  assert!( !content.contains( "\"autoUpdates\": \"false\"" ),
    "autoUpdates must NOT be stored as quoted string: {content}" );
}

// ─── FT-04: AC-04 set with scope::project ────────────────────────────────────

// FT-4: .config key::model value::claude-haiku scope::project → project settings written; exit 0
#[ test ]
fn ft4_006_config_set_project_scope()
{
  let home_dir    = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let home        = home_dir.path().to_str().unwrap();
  let project     = project_dir.path().to_str().unwrap();

  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::model", "value::claude-haiku-4-5-20251001", "scope::project" ] )
    .env( "HOME", home )
    .current_dir( project )
    .output()
    .unwrap();
  assert_exit( &out, 0 );

  let proj_settings = project_dir.path().join( ".claude/settings.json" );
  assert!( proj_settings.exists(), "project settings.json must be created" );
  let content = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( content.contains( "claude-haiku-4-5-20251001" ),
    "project settings must contain model value: {content}" );

  // User settings must be untouched.
  assert!( !home_dir.path().join( ".claude/settings.json" ).exists(),
    "user settings.json must not be created by project scope write" );
}

// ─── FT-05: AC-05 unset removes key ──────────────────────────────────────────

// FT-5: .config key::theme unset::1 → theme key removed; other keys preserved; exit 0
#[ test ]
fn ft5_006_config_unset_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ), ( "model", "claude-sonnet-4-6" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "unset::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( !content.contains( "\"theme\"" ), "theme key must be removed: {content}" );
  assert!( content.contains( "model" ), "other keys must be preserved: {content}" );
}

// ─── FT-06: AC-06 format::json with source fields ────────────────────────────

// FT-6: .config format::json → JSON object with source field per key; exit 0
#[ test ]
fn ft6_006_config_show_all_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "format::json" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // JSON must include model with source: default and theme with source: user.
  assert!( text.contains( "\"source\"" ), "JSON must contain source field: {text}" );
  assert!( text.contains( "\"default\"" ) || text.contains( "default" ),
    "JSON must include a default source somewhere: {text}" );
  assert!( text.contains( "\"user\"" ) || text.contains( "user" ),
    "JSON must include a user source for theme: {text}" );
}

// ─── FT-07: AC-07 env var overrides user config ──────────────────────────────

// FT-7: CLAUDE_MODEL=claude-opus-4-6 overrides user settings model → shows (env); exit 0
#[ test ]
fn ft7_006_config_env_overrides_user()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "model", "claude-sonnet-4-6" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::model" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "claude-opus-4-6" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-opus-4-6" ), "env value must override user config: {text}" );
  assert!( text.contains( "(env)" ), "must show (env) source annotation: {text}" );
}

// ─── FT-08: AC-08 catalog default shown when no user config ──────────────────

// FT-8: hasCompletedOnboarding not in user config → shows false (default); exit 0
#[ test ]
fn ft8_006_config_get_absent_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::hasCompletedOnboarding" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "false" ), "must show catalog default value false: {text}" );
  assert!( text.contains( "(default)" ), "must show (default) source annotation: {text}" );
}

// ─── FT-09: AC-09 dry::1 previews set ────────────────────────────────────────

// FT-9: .config key::theme value::dark dry::1 → preview shown, file unchanged; exit 0
#[ test ]
fn ft9_006_config_set_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "light" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry-run output must contain [dry-run]: {text}" );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "light" ), "file must remain unchanged: {content}" );
}

// ─── FT-10: AC-10 HOME unset → exit 2 ────────────────────────────────────────

// FT-10: HOME unset → any filesystem operation exits 2
#[ test ]
fn ft10_006_config_home_unset_exits_2()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme" ],
    &[ ( "HOME", "" ) ],
  );
  assert_exit( &out, 2 );
}

// ─── FT-11: AC-11 non-catalog key accepted ───────────────────────────────────

// FT-11: non-catalog key written without error; exit 0
#[ test ]
fn ft11_006_config_arbitrary_key_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::myCustomKey", "value::customValue" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "myCustomKey" ), "non-catalog key must be written: {content}" );
  assert!( content.contains( "customValue" ), "non-catalog value must be written: {content}" );
}

// ─── FT-12: AC-12 catalog default for model ──────────────────────────────────

// FT-12: catalog default for model is claude-sonnet-4-6 when no env or config; exit 0
//
// Uses an isolated cwd to prevent the project config walk from finding the
// container-mounted /workspace/.claude/settings.json.
#[ test ]
fn ft12_006_config_catalog_default_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let bin  = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::model" ] )
    .env( "HOME", home )
    .env( "CLAUDE_MODEL", "" )
    .current_dir( dir.path() )
    .output()
    .expect( "failed to execute claude_version binary" );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-sonnet-4-6" ), "must show catalog default: {text}" );
  assert!( text.contains( "(default)" ), "must show (default) source annotation: {text}" );
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigScope type tests (tests/docs/cli/type/06_config_scope.md)
// ═══════════════════════════════════════════════════════════════════════════════

// ─── TC-1: scope::user accepted ──────────────────────────────────────────────

// TC-1: scope::user → exit 0; value written to ~/.claude/settings.json
#[ test ]
fn tc01_006_scope_user_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::user" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"theme\"" ), "user settings.json must contain theme: {content}" );
  assert!( content.contains( "dark" ), "user settings.json must contain value: {content}" );
}

// ─── TC-2: scope::project accepted ───────────────────────────────────────────

// TC-2: scope::project → exit 0; {cwd}/.claude/settings.json created; user config unchanged
#[ test ]
fn tc02_006_scope_project_accepted()
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
  assert!( proj_settings.exists(), "project settings.json must be created" );
  let content = std::fs::read_to_string( &proj_settings ).unwrap();
  assert!( content.contains( "dark" ), "project settings must contain value: {content}" );
  assert!( !home_dir.path().join( ".claude/settings.json" ).exists(),
    "user settings.json must not be created by project scope write" );
}

// ─── TC-3: absent scope:: defaults to user ───────────────────────────────────

// TC-3: no scope:: → defaults to user scope; writes to ~/.claude/settings.json; exit 0
#[ test ]
fn tc03_006_scope_absent_defaults_to_user()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let user_settings = dir.path().join( ".claude/settings.json" );
  assert!( user_settings.exists(), "user settings.json must be created by default scope" );
  let content = std::fs::read_to_string( &user_settings ).unwrap();
  assert!( content.contains( "dark" ), "user settings must contain written value: {content}" );
}

// ─── TC-4: scope::global → exit 1, unknown variant ──────────────────────────

// TC-4: scope::global → unrecognised scope value; exit 1
#[ test ]
fn tc04_006_scope_global_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::global" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── TC-5: scope::USER → exit 1, case-sensitive ──────────────────────────────

// TC-5: scope::USER → wrong case; rejected; exit 1
#[ test ]
fn tc05_006_scope_wrong_case_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::USER" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── TC-6: scope:: (empty) → exit 1 ─────────────────────────────────────────

// TC-6: scope:: (empty value) → exit 1
#[ test ]
fn tc06_006_scope_empty_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::theme", "value::dark", "scope::" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigKey type tests (tests/docs/cli/type/07_config_key.md)
// ═══════════════════════════════════════════════════════════════════════════════

// ─── TC-1: key::model → catalog key resolves default ─────────────────────────

// TC-1: key::model with no env/config → catalog default claude-sonnet-4-6; exit 0
//
// Uses isolated cwd to avoid project config walk finding container-mounted settings.
#[ test ]
fn tc01_007_config_key_catalog_default()
{
  let dir = TempDir::new().unwrap();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".config", "key::model" ] )
    .env( "HOME", dir.path().to_str().unwrap() )
    .env( "CLAUDE_MODEL", "" )
    .current_dir( dir.path() )
    .output()
    .expect( "failed to execute claude_version binary" );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-sonnet-4-6" ), "must show catalog default: {text}" );
  assert!( text.contains( "(default)" ), "must show (default) source annotation: {text}" );
}

// ─── TC-2: key::myCustomSetting → arbitrary key, absent ──────────────────────

// TC-2: key::myCustomSetting with no config → arbitrary key accepted; exit 0
#[ test ]
fn tc02_007_config_key_arbitrary_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".config", "key::myCustomSetting" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// ─── TC-3: key::theme → catalog key resolves user config ─────────────────────

// TC-3: key::theme with user config {theme:dark} → shows dark with (user); exit 0
#[ test ]
fn tc03_007_config_key_catalog_user_config()
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
  assert!( text.contains( "dark" ), "must show user config value: {text}" );
  assert!( text.contains( "(user)" ), "must show (user) source annotation: {text}" );
}

// ─── TC-4: key::a.b.c → dot treated as literal ───────────────────────────────

// TC-4: key::a.b.c with user config {a.b.c:test} → dot is literal; exit 0
#[ test ]
fn tc04_007_config_key_dot_literal()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "a.b.c", "test" ) ] );

  let out = run_clm_with_env(
    &[ ".config", "key::a.b.c" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "test" ), "must show value for dot-literal key: {text}" );
}

// ─── TC-5: absent key:: → show-all mode ──────────────────────────────────────

// TC-5: .config with no key:: → show-all mode lists resolved settings; exit 0
#[ test ]
fn tc05_007_config_key_absent_show_all()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".config" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "theme" ), "show-all must include user keys: {text}" );
}

// ─── TC-6: key:: (empty) → exit 1 ───────────────────────────────────────────

// TC-6: key:: (empty value) → exit 1
#[ test ]
fn tc06_007_config_key_empty_exits_1()
{
  let out = run_clm_with_env(
    &[ ".config", "key::" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}
