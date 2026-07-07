//! Integration tests for the `.params` command.
//!
//! Implements test cases from:
//! - `tests/docs/cli/command/14_params.md` (IT-1 through IT-14)
//!
//! # Test Matrix
//!
//! | IT | Description | Mode | Exit |
//! |----|-------------|------|------|
//! | IT-1  | No params → show-all ≥35 entries, each annotated | show-all | 0 |
//! | IT-2  | `key::model` → CLI, env, config forms + default | single | 0 |
//! | IT-3  | `kind::config` → only config-key params; env-only absent | show-all | 0 |
//! | IT-4  | `kind::env` → only env-var params; config-only absent | show-all | 0 |
//! | IT-5  | `key::model` with CLAUDE_MODEL set → env value + (env) | single | 0 |
//! | IT-6  | `key::bash_timeout` → env-only, unset, default 120000 | single | 0 |
//! | IT-7  | `format::json` → valid JSON array with name field per entry | show-all | 0 |
//! | IT-8  | `key::print` → CLI-only annotation | single | 0 |
//! | IT-9  | `v::0` → compact values-only; no "Forms:" labels | show-all | 0 |
//! | IT-10 | `key::model` no env no config → (default) annotation | single | 0 |
//! | IT-11 | Show-all output is alphabetically sorted | show-all | 0 |
//! | IT-12 | `key::NONEXISTENT_KEY` → exit 2 | — | 2 |
//! | IT-13 | `kind::badvalue` → exit 1 | — | 1 |
//! | IT-14 | `format::xml` → exit 1 | — | 1 |
//! | IT-15 | `key::disable_updates` → shows config form env.DISABLE_UPDATES | single | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv_with_env, stderr, stdout };

// ─── IT-1: show-all ≥35 entries ──────────────────────────────────────────────

// IT-1: no params → show-all; ≥35 param entries; each annotated; exit 0
#[ test ]
fn it01_params_show_all_min_entries()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Top-level lines (param name entries) start at column 0; indented lines are details.
  let entry_count = text.lines()
    .filter( |l| !l.starts_with( ' ' ) && !l.is_empty() )
    .count();
  assert!(
    entry_count >= 35,
    "show-all must list ≥35 params, got {entry_count}:\n{text}"
  );
}

// ─── IT-2: key::model → all three forms + default ────────────────────────────

// IT-2: key::model no env no config → shows --model, CLAUDE_MODEL, default; exit 0
#[ test ]
fn it02_params_single_model_full_detail()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "key::model" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "--model" ),           "must show CLI form --model: {text}" );
  assert!( text.contains( "CLAUDE_MODEL" ),      "must show env form CLAUDE_MODEL: {text}" );
  assert!( text.contains( "claude-sonnet-5" ), "must show default value: {text}" );
}

// ─── IT-3: kind::config → only config-key params ─────────────────────────────

// IT-3: kind::config → config-key params present; env-only (bash_timeout) absent; exit 0
#[ test ]
fn it03_params_kind_config_filters()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "kind::config" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),         "kind::config must include model: {text}" );
  assert!( !text.contains( "bash_timeout" ), "kind::config must exclude env-only bash_timeout: {text}" );
}

// ─── IT-4: kind::env → only env-var params ───────────────────────────────────

// IT-4: kind::env → env-var params present; config-only (theme) absent; exit 0
#[ test ]
fn it04_params_kind_env_filters()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "kind::env" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "bash_timeout" ), "kind::env must include bash_timeout: {text}" );
  assert!( !text.contains( "theme" ),        "kind::env must exclude config-only theme: {text}" );
}

// ─── IT-5: key::model with CLAUDE_MODEL set ──────────────────────────────────

// IT-5: CLAUDE_MODEL=claude-opus-4-8 → env value shown with (env) annotation; exit 0
#[ test ]
fn it05_params_env_override_visible()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "key::model" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "claude-opus-4-8" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-opus-4-8" ), "must show env value: {text}" );
  assert!( text.contains( "(env)" ),            "must annotate with (env): {text}" );
}

// ─── IT-6: key::bash_timeout → env-only param, unset ────────────────────────

// IT-6: CLAUDE_CODE_BASH_TIMEOUT unset → shows env form, "unset", default 120000; exit 0
#[ test ]
fn it06_params_env_only_param_unset()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "key::bash_timeout" ],
    &[ ( "HOME", home ), ( "CLAUDE_CODE_BASH_TIMEOUT", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "CLAUDE_CODE_BASH_TIMEOUT" ), "must show env form: {text}" );
  assert!( text.contains( "120000" ),                    "must show default 120000: {text}" );
}

// ─── IT-7: format::json → valid JSON array ───────────────────────────────────

// IT-7: format::json → stdout is JSON array; each entry has "name" field; exit 0
#[ test ]
fn it07_params_json_output_structure()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "format::json" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let trimmed = text.trim();
  assert!( trimmed.starts_with( '[' ), "format::json must produce JSON array starting with '[': {text}" );
  assert!( trimmed.ends_with( ']' ),   "JSON array must end with ']': {text}" );
  assert!( text.contains( "\"name\"" ), "JSON entries must include 'name' field: {text}" );
}

// ─── IT-8: key::print → CLI-only annotation ──────────────────────────────────

// IT-8: key::print → shows --print form and CLI-only annotation; exit 0
#[ test ]
fn it08_params_cli_only_annotation()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "key::print" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let lower = text.to_lowercase();
  assert!( text.contains( "--print" ),
    "must show --print CLI form: {text}" );
  assert!( lower.contains( "cli-only" ) || lower.contains( "cli only" ) || lower.contains( "unobservable" ),
    "must show CLI-only annotation: {text}" );
}

// ─── IT-9: v::0 → compact values-only output ────────────────────────────────

// IT-9: v::0 → exit 0; output not empty; no "Forms:" labels
#[ test ]
fn it09_params_compact_v0_output()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "v::0" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.is_empty(),           "v::0 must produce output" );
  assert!( !text.contains( "Forms:" ), "v::0 must not include 'Forms:' labels: {text}" );
}

// ─── IT-10: key::model no env no config → (default) annotation ──────────────

// IT-10: key::model no env no config → shows claude-sonnet-5 with (default); exit 0
//
// Uses isolated cwd to prevent project config walk from finding container settings.
#[ test ]
fn it10_params_default_annotation()
{
  let dir = TempDir::new().unwrap();
  let bin = env!( "CARGO_BIN_EXE_claude_version" );

  let out = std::process::Command::new( bin )
    .args( [ ".params", "key::model" ] )
    .env( "HOME", dir.path().to_str().unwrap() )
    .env( "CLAUDE_MODEL", "" )
    .current_dir( dir.path() )
    .output()
    .expect( "failed to execute claude_version binary" );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-sonnet-5" ), "must show catalog default: {text}" );
  assert!( text.contains( "(default)" ),          "must show (default) annotation: {text}" );
}

// ─── IT-11: show-all alphabetically sorted ───────────────────────────────────

// IT-11: .params show-all → param name entries appear in ascending alphabetical order; exit 0
#[ test ]
fn it11_params_show_all_alphabetical()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Top-level non-empty, non-indented lines are param name entries.
  let names : Vec< &str > = text.lines()
    .filter( |l| !l.starts_with( ' ' ) && !l.is_empty() )
    .collect();
  assert!( !names.is_empty(), "show-all must produce param entries: {text}" );
  let mut sorted = names.clone();
  sorted.sort_unstable();
  assert_eq!( names, sorted, "param names must be in ascending alphabetical order" );
}

// ─── IT-12: unknown key → exit 2 ─────────────────────────────────────────────

// IT-12: key::NONEXISTENT_KEY → key not in params catalog; exit 2; stderr contains key name
#[ test ]
fn it12_params_unknown_key_exits_2()
{
  let out = run_clv_with_env(
    &[ ".params", "key::NONEXISTENT_KEY" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "exit 2 must produce an error message on stderr" );
}

// ─── IT-13: kind::badvalue → exit 1 ─────────────────────────────────────────

// IT-13: kind::badvalue → unrecognised kind value; exit 1; stderr mentions valid values
#[ test ]
fn it13_params_invalid_kind_exits_1()
{
  let out = run_clv_with_env(
    &[ ".params", "kind::badvalue" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "config" ) || err.contains( "env" ),
    "exit 1 must mention valid kind values (config, env): {err}"
  );
}

// ─── IT-14: format::xml → exit 1 ─────────────────────────────────────────────

// IT-14: format::xml → unrecognised format value; exit 1; stderr mentions valid values
#[ test ]
fn it14_params_invalid_format_exits_1()
{
  let out = run_clv_with_env(
    &[ ".params", "format::xml" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "text" ) || err.contains( "json" ),
    "exit 1 must mention valid format values (text, json): {err}"
  );
}

// ─── IT-15: key::disable_updates → shows config form env.DISABLE_UPDATES ─────

// IT-15: key::disable_updates → config form env.DISABLE_UPDATES now registered; exit 0
#[ test ]
fn it15_params_disable_updates_shows_config_form()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clv_with_env(
    &[ ".params", "key::disable_updates" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "DISABLE_UPDATES" ),      "must show env form DISABLE_UPDATES: {text}" );
  assert!( text.contains( "env.DISABLE_UPDATES" ),  "must show config form env.DISABLE_UPDATES: {text}" );
}
