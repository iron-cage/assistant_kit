//! Integration tests: `.model` — unified session + subprocess model/effort command.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! Maps function names to the Test Matrix rows in
//! `task/claude_profile/465_unified_model_command_scope_routing.md`.
//! `.model.select`'s own retirement stub (T23) is covered separately in
//! `model_select_test.rs`.
//!
//! | Function | Row | Condition | P/N |
//! |----------|-----|-----------|-----|
//! | `t01_get_default_scope_is_session`                    | T01 | `.model` (fresh HOME) → `scope: session (<path>)`, both unset | P |
//! | `t02_get_subprocess_scope`                             | T02 | `scope::subprocess` (fresh HOME) → `scope: subprocess (<path>)`, both unset | P |
//! | `t03_get_invalid_scope_exits_1`                        | T03 | `scope::bad` → exit 1, valid values named | N |
//! | `t04_set_model_session_each_shorthand`                 | T04 | `model::opus\|sonnet\|haiku` → full ID written to settings.json | P |
//! | `t05_set_model_session_default_removes_key`            | T05 | `model::default` → removes `model`, preserves other keys | P |
//! | `t06_set_model_session_invalid_exits_1`                | T06 | `model::bad` → exit 1, shorthand list in stderr | N |
//! | `t07_set_model_subprocess_writes_config_toml`          | T07 | `scope::subprocess model::claude-opus-4-8` → written to config.toml | P |
//! | `t08_set_model_subprocess_empty_exits_1`               | T08 | `scope::subprocess model::` → exit 1, non-empty requirement | N |
//! | `t09_set_effort_session_writes_effort_level`           | T09 | `effort_level::high` → `effortLevel` written to settings.json | P |
//! | `t10_set_effort_session_invalid_exits_1`               | T10 | `effort_level::bad` → exit 1, low/normal/high/max listed | N |
//! | `t11_set_effort_subprocess_writes_config_toml`         | T11 | `scope::subprocess effort_level::medium` → `effort` written | P |
//! | `t12_set_effort_subprocess_session_only_value_exits_1` | T12 | `scope::subprocess effort_level::normal` → exit 1, subprocess vocab listed | N |
//! | `t13_reset_model_session_removes_key`                  | T13 | `reset_model::1` → removes `model` key | P |
//! | `t14_reset_effort_session_removes_key`                 | T14 | `reset_effort_level::1` → removes `effortLevel` key | P |
//! | `t15_reset_model_subprocess_idempotent`                | T15 | `scope::subprocess reset_model::1` twice → exit 0 both times | P |
//! | `t16_reset_effort_subprocess_idempotent`                | T16 | `scope::subprocess reset_effort_level::1` twice → exit 0 both times | P |
//! | `t17_mutual_exclusion_model_exits_1`                    | T17 | `model::opus reset_model::1` → exit 1, named conflict | N |
//! | `t18_mutual_exclusion_effort_exits_1`                   | T18 | `effort_level::high reset_effort_level::1` → exit 1, named conflict | N |
//! | `t19_combine_across_concepts`                           | T19 | `model::opus reset_effort_level::1` → both applied, exit 0 | P |
//! | `t20_combine_within_subprocess_scope_preserves_keys`    | T20 | `scope::subprocess model::... effort_level::max` → both written, other keys preserved | P |
//! | `t21_json_format_shape`                                 | T21 | `format::json` (fresh HOME) → JSON shape matches spec | P |
//! | `t22_subprocess_creates_missing_dir_and_file`           | T22 | `scope::subprocess model::VALUE` (fresh HOME) → creates `.clr/` + `config.toml` | P |

use crate::cli_runner::{ run_cs_with_env, stdout, stderr, assert_exit };
use tempfile::TempDir;

/// Read `~/.clr/config.toml` from a temp home directory; `None` if absent.
///
/// Mirrors the identical local helper in `account_provider_test.rs` — kept local
/// per this test suite's existing one-helper-per-file precedent for fixture reads.
fn read_clr_config( home : &std::path::Path ) -> Option< String >
{
  std::fs::read_to_string( home.join( ".clr" ).join( "config.toml" ) ).ok()
}

/// Read and parse `~/.claude/settings.json` from a temp home directory.
///
/// Returns `serde_json::json!({})` when the file is absent.
///
/// # Panics
///
/// Panics if the file exists but is not valid JSON.
fn read_settings_json( home : &std::path::Path ) -> serde_json::Value
{
  std::fs::read_to_string( home.join( ".claude" ).join( "settings.json" ) )
    .ok()
    .map_or_else( || serde_json::json!( {} ), | s | serde_json::from_str( &s ).expect( "settings.json must be valid JSON" ) )
}

/// Write `~/.claude/settings.json` with an arbitrary JSON object (raw, full control).
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
fn write_settings_json_raw( home : &std::path::Path, value : &serde_json::Value )
{
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), serde_json::to_string_pretty( value ).unwrap() ).unwrap();
}

// ── T01–T03: get mode + scope validation ────────────────────────────────────────

/// T01 (AC-01): `.model` (no `scope::`, fresh HOME) → exit 0, `scope: session (<path>)`
/// with both `model` and `effort_level` shown as `(unset)`.
#[ test ]
fn t01_get_default_scope_is_session()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let expected_path = dir.path().join( ".claude" ).join( "settings.json" ).display().to_string();
  assert_eq!(
    stdout( &out ),
    format!( "scope: session ({expected_path})\nmodel: (unset)\neffort_level: (unset)\n" ),
    "T01: unexpected get-mode output",
  );
}

/// T02 (AC-02): `.model scope::subprocess` (fresh HOME) → exit 0,
/// `scope: subprocess (<path>)` with both fields `(unset)`.
#[ test ]
fn t02_get_subprocess_scope()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "scope::subprocess" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let expected_path = dir.path().join( ".clr" ).join( "config.toml" ).display().to_string();
  assert_eq!(
    stdout( &out ),
    format!( "scope: subprocess ({expected_path})\nmodel: (unset)\neffort_level: (unset)\n" ),
    "T02: unexpected get-mode output",
  );
}

/// T03 (AC-03): `.model scope::bad` → exit 1, stderr names both valid values.
#[ test ]
fn t03_get_invalid_scope_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "scope::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "session" ) && err.contains( "subprocess" ),
    "T03: stderr must name valid scope values, got:\n{err}" );
}

// ── T04–T06: set model, session ─────────────────────────────────────────────────

/// T04 (AC-04/05/06): `.model model::opus|sonnet|haiku` each write the corresponding
/// full model ID to `settings.json`'s `model` key.
#[ test ]
fn t04_set_model_session_each_shorthand()
{
  let cases = [
    ( "opus",   "claude-opus-4-8" ),
    ( "sonnet", "claude-sonnet-5" ),
    ( "haiku",  "claude-haiku-4-5-20251001" ),
  ];
  for ( shorthand, full_id ) in cases
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();

    let out = run_cs_with_env( &[ ".model", &format!( "model::{shorthand}" ) ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let expected_path = dir.path().join( ".claude" ).join( "settings.json" ).display().to_string();
    assert_eq!(
      stdout( &out ),
      format!( "model: {shorthand}  →  {expected_path} (session)\n" ),
      "T04({shorthand}): unexpected confirmation line",
    );

    let settings = read_settings_json( dir.path() );
    assert_eq!( settings[ "model" ], serde_json::json!( full_id ),
      "T04({shorthand}): settings.json model must be {full_id:?}, got:\n{settings}" );
  }
}

/// T05 (AC-07): `.model model::default` removes the `model` key while preserving an
/// unrelated pre-existing key.
#[ test ]
fn t05_set_model_session_default_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings_json_raw( dir.path(), &serde_json::json!( { "model": "claude-sonnet-5", "theme": "dark" } ) );

  let out = run_cs_with_env( &[ ".model", "model::default" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let settings = read_settings_json( dir.path() );
  assert!( settings.get( "model" ).is_none(), "T05: model key must be removed, got:\n{settings}" );
  assert_eq!( settings[ "theme" ], serde_json::json!( "dark" ), "T05: unrelated key must be preserved, got:\n{settings}" );
}

/// T06 (AC-08): `.model model::bad` → exit 1, stderr lists the shorthand vocabulary.
#[ test ]
fn t06_set_model_session_invalid_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "model::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    [ "opus", "sonnet", "haiku", "default" ].iter().all( | v | err.contains( v ) ),
    "T06: stderr must list opus/sonnet/haiku/default, got:\n{err}",
  );
}

// ── T07–T08: set model, subprocess ──────────────────────────────────────────────

/// T07 (AC-09): `.model scope::subprocess model::claude-opus-4-8` writes the full ID
/// to `~/.clr/config.toml`'s user tier.
#[ test ]
fn t07_set_model_subprocess_writes_config_toml()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "scope::subprocess", "model::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let expected_path = dir.path().join( ".clr" ).join( "config.toml" ).display().to_string();
  assert_eq!(
    stdout( &out ),
    format!( "model: claude-opus-4-8  →  {expected_path} (subprocess)\n" ),
    "T07: unexpected confirmation line",
  );

  let config = read_clr_config( dir.path() ).expect( "T07: config.toml must be created" );
  assert!( config.contains( "model" ) && config.contains( "claude-opus-4-8" ),
    "T07: config.toml must persist model = \"claude-opus-4-8\", got:\n{config}" );
}

/// T08 (AC-10): `.model scope::subprocess model::` (empty) → exit 1, stderr names the
/// non-empty requirement.
#[ test ]
fn t08_set_model_subprocess_empty_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "scope::subprocess", "model::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "non-empty" ), "T08: stderr must name the non-empty requirement, got:\n{err}" );
  assert!( read_clr_config( dir.path() ).is_none(), "T08: config.toml must not be created on rejection" );
}

// ── T09–T10: set effort, session ────────────────────────────────────────────────

/// T09 (AC-11): `.model effort_level::high` writes `effortLevel` to `settings.json`.
#[ test ]
fn t09_set_effort_session_writes_effort_level()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "effort_level::high" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let expected_path = dir.path().join( ".claude" ).join( "settings.json" ).display().to_string();
  assert_eq!(
    stdout( &out ),
    format!( "effort_level: high  →  {expected_path} (session)\n" ),
    "T09: unexpected confirmation line",
  );

  let settings = read_settings_json( dir.path() );
  assert_eq!( settings[ "effortLevel" ], serde_json::json!( "high" ), "T09: got:\n{settings}" );
}

/// T10 (AC-12): `.model effort_level::bad` → exit 1, stderr lists low/normal/high/max.
#[ test ]
fn t10_set_effort_session_invalid_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "effort_level::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    [ "low", "normal", "high", "max" ].iter().all( | v | err.contains( v ) ),
    "T10: stderr must list low/normal/high/max, got:\n{err}",
  );
}

// ── T11–T12: set effort, subprocess ─────────────────────────────────────────────

/// T11 (AC-13): `.model scope::subprocess effort_level::medium` writes `effort` to
/// `config.toml`.
#[ test ]
fn t11_set_effort_subprocess_writes_config_toml()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "scope::subprocess", "effort_level::medium" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let config = read_clr_config( dir.path() ).expect( "T11: config.toml must be created" );
  assert!( config.contains( "effort" ) && config.contains( "medium" ),
    "T11: config.toml must persist effort = \"medium\", got:\n{config}" );
}

/// T12 (AC-14): `.model scope::subprocess effort_level::normal` → exit 1 — `normal` is
/// session-only vocabulary, not valid for subprocess (`low`/`medium`/`high`/`max`).
#[ test ]
fn t12_set_effort_subprocess_session_only_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "scope::subprocess", "effort_level::normal" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    [ "low", "medium", "high", "max" ].iter().all( | v | err.contains( v ) ),
    "T12: stderr must list the subprocess vocabulary (low/medium/high/max), got:\n{err}",
  );
}

// ── T13–T14: reset, session ──────────────────────────────────────────────────────

/// T13 (AC-15): `.model reset_model::1` removes the `model` key from `settings.json`.
#[ test ]
fn t13_reset_model_session_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings_json_raw( dir.path(), &serde_json::json!( { "model": "claude-opus-4-8" } ) );

  let out = run_cs_with_env( &[ ".model", "reset_model::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let expected_path = dir.path().join( ".claude" ).join( "settings.json" ).display().to_string();
  assert_eq!(
    stdout( &out ),
    format!( "model: (reset)  →  {expected_path} (session)\n" ),
    "T13: unexpected confirmation line",
  );

  let settings = read_settings_json( dir.path() );
  assert!( settings.get( "model" ).is_none(), "T13: model key must be removed, got:\n{settings}" );
}

/// T14 (AC-16): `.model reset_effort_level::1` removes the `effortLevel` key —
/// exercises task 464's `remove_session_effort()`.
#[ test ]
fn t14_reset_effort_session_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings_json_raw( dir.path(), &serde_json::json!( { "effortLevel": "high" } ) );

  let out = run_cs_with_env( &[ ".model", "reset_effort_level::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let expected_path = dir.path().join( ".claude" ).join( "settings.json" ).display().to_string();
  assert_eq!(
    stdout( &out ),
    format!( "effort_level: (reset)  →  {expected_path} (session)\n" ),
    "T14: unexpected confirmation line",
  );

  let settings = read_settings_json( dir.path() );
  assert!( settings.get( "effortLevel" ).is_none(), "T14: effortLevel key must be removed, got:\n{settings}" );
}

// ── T15–T16: reset, subprocess, idempotent ──────────────────────────────────────

/// T15 (AC-17): `.model scope::subprocess reset_model::1` exits 0 whether or not the
/// key/file already exists — run twice against a fresh HOME with no `.clr/` at all.
#[ test ]
fn t15_reset_model_subprocess_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out1 = run_cs_with_env( &[ ".model", "scope::subprocess", "reset_model::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );
  let out2 = run_cs_with_env( &[ ".model", "scope::subprocess", "reset_model::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
}

/// T16 (AC-18): `.model scope::subprocess reset_effort_level::1` exits 0 whether or
/// not the key/file already exists — run twice against a fresh HOME.
#[ test ]
fn t16_reset_effort_subprocess_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out1 = run_cs_with_env( &[ ".model", "scope::subprocess", "reset_effort_level::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );
  let out2 = run_cs_with_env( &[ ".model", "scope::subprocess", "reset_effort_level::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
}

// ── T17–T18: mutual exclusion ────────────────────────────────────────────────────

/// T17 (AC-19): `.model model::opus reset_model::1` together → exit 1, stderr names
/// the conflict.
#[ test ]
fn t17_mutual_exclusion_model_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "model::opus", "reset_model::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "model::" ) && err.contains( "reset_model::1" ) && err.contains( "mutually exclusive" ),
    "T17: stderr must name the model::/reset_model::1 conflict, got:\n{err}" );
}

/// T18 (AC-20): `.model effort_level::high reset_effort_level::1` together → exit 1,
/// stderr names the conflict.
#[ test ]
fn t18_mutual_exclusion_effort_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "effort_level::high", "reset_effort_level::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "effort_level::" ) && err.contains( "reset_effort_level::1" ) && err.contains( "mutually exclusive" ),
    "T18: stderr must name the effort_level::/reset_effort_level::1 conflict, got:\n{err}" );
}

// ── T19–T20: combining actions ───────────────────────────────────────────────────

/// T19 (AC-21): `.model model::opus reset_effort_level::1` applies both actions
/// (across concepts) in one call.
#[ test ]
fn t19_combine_across_concepts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings_json_raw( dir.path(), &serde_json::json!( { "effortLevel": "max" } ) );

  let out = run_cs_with_env( &[ ".model", "model::opus", "reset_effort_level::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let settings = read_settings_json( dir.path() );
  assert_eq!( settings[ "model" ], serde_json::json!( "claude-opus-4-8" ), "T19: model must be set, got:\n{settings}" );
  assert!( settings.get( "effortLevel" ).is_none(), "T19: effortLevel must be reset, got:\n{settings}" );
}

/// T20 (AC-22): `.model scope::subprocess model::claude-opus-4-8 effort_level::max`
/// writes both keys in one call and preserves an unrelated pre-existing key.
#[ test ]
fn t20_combine_within_subprocess_scope_preserves_keys()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Pre-populate an unrelated key via `.provider.select` (separate command, separate key).
  let setup = run_cs_with_env( &[ ".provider.select", "id::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &setup, 0 );

  let out = run_cs_with_env(
    &[ ".model", "scope::subprocess", "model::claude-opus-4-8", "effort_level::max" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let config = read_clr_config( dir.path() ).expect( "T20: config.toml must exist" );
  assert!( config.contains( "claude-opus-4-8" ), "T20: model key must be written, got:\n{config}" );
  assert!( config.contains( "effort" ) && config.contains( "max" ), "T20: effort key must be written, got:\n{config}" );
  assert!( config.contains( "kimi" ), "T20: unrelated provider key must be preserved, got:\n{config}" );
}

// ── T21: JSON format ──────────────────────────────────────────────────────────────

/// T21 (AC-23): `.model format::json` (fresh HOME, get mode) matches the documented
/// JSON shape: `{"scope":...,"path":...,"model":null,"effort_level":null}`.
#[ test ]
fn t21_json_format_shape()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".model", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let parsed : serde_json::Value = serde_json::from_str( stdout( &out ).trim() )
    .unwrap_or_else( | e | panic!( "T21: stdout must be valid JSON: {e}\ngot:\n{}", stdout( &out ) ) );
  let expected_path = dir.path().join( ".claude" ).join( "settings.json" ).display().to_string();
  assert_eq!(
    parsed,
    serde_json::json!( { "scope": "session", "path": expected_path, "model": null, "effort_level": null } ),
    "T21: unexpected JSON shape",
  );
}

// ── T22: subprocess store creation ────────────────────────────────────────────────

/// T22 (AC-24): `.model scope::subprocess model::VALUE` against a fresh HOME with no
/// `.clr/` directory at all creates both the directory and `config.toml`.
#[ test ]
fn t22_subprocess_creates_missing_dir_and_file()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  assert!( !dir.path().join( ".clr" ).exists(), "T22 precondition: .clr/ must not pre-exist" );

  let out = run_cs_with_env( &[ ".model", "scope::subprocess", "model::claude-haiku-4-5-20251001" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  assert!( dir.path().join( ".clr" ).is_dir(), "T22: .clr/ directory must be created" );
  assert!( dir.path().join( ".clr" ).join( "config.toml" ).is_file(), "T22: config.toml must be created" );
}
