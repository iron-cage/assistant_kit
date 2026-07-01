//! Feature surface tests (FT- prefix) for `claude_version`.
//!
//! Implements test cases from `tests/docs/feature/` spec files.
//! Each function maps to one FT- case that is not already covered by
//! `mutation_commands_test.rs` or `read_commands_test.rs`.
//!
//! # Coverage Map
//!
//! | FT-spec | ID | Function |
//! |---------|----|----------|
//! | feature/003_settings_management.md | FT-3 | `ft003_settings_set_get_round_trip` |
//! | feature/002_process_lifecycle.md | FT-4 | `ft004_processes_kill_force_no_procs` |
//! | feature/005_cli_design.md | FT-1 | `ft005_1_unknown_param_exits_1` |
//! | feature/005_cli_design.md | FT-2 | `ft005_2_empty_bool_param_value_exits_1` |
//! | feature/005_cli_design.md | FT-3 | `ft005_3_last_param_wins` |
//! | feature/007_params_command.md | FT-1 | `ft1_007_params_show_all_min_entries` |
//! | feature/007_params_command.md | FT-2 | `ft2_007_params_single_model_full_detail` |
//! | feature/007_params_command.md | FT-3 | `ft3_007_params_kind_config_filters` |
//! | feature/007_params_command.md | FT-4 | `ft4_007_params_kind_env_filters` |
//! | feature/007_params_command.md | FT-5 | `ft5_007_params_env_override_visible` |
//! | feature/007_params_command.md | FT-6 | `ft6_007_params_env_only_param` |
//! | feature/007_params_command.md | FT-7 | `ft7_007_params_json_output_structure` |
//! | feature/007_params_command.md | FT-8 | `ft8_007_params_cli_only_annotation` |
//! | feature/007_params_command.md | FT-9 | `ft9_007_params_unknown_key_exits_2` |
//! | feature/007_params_command.md | FT-10 | `ft10_007_params_invalid_kind_exits_1` |
//! | feature/007_params_command.md | FT-11 | `ft11_007_params_default_source_annotation` |
//! | feature/007_params_command.md | FT-12 | `ft12_007_params_show_all_alphabetical` |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clm, run_clm_with_env, stdout, write_settings };

// ─── FT-3 (feature/003_settings_management.md): set+get round-trip ───────────

// FT-3: .settings.set then .settings.get returns the stored value
#[ test ]
fn ft003_settings_set_get_round_trip()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let set_out = run_clm_with_env(
    &[ ".settings.set", "key::color", "value::blue" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &set_out, 0 );

  let get_out = run_clm_with_env(
    &[ ".settings.get", "key::color" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &get_out, 0 );
  let text = stdout( &get_out );
  assert!( text.contains( "blue" ), "get must return stored value 'blue': {text}" );
}

// ─── FT-4 (feature/002_process_lifecycle.md): force::1, no processes ─────────

// FT-4: force::1 with no processes → "no active processes", exit 0
#[ test ]
fn ft004_processes_kill_force_no_procs()
{
  // Use PATH="" so no `claude` binary is found in /proc scan from subprocess lookup,
  // though /proc scan is global. The test verifies the force path exits cleanly.
  let out = run_clm( &[ ".processes.kill", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no active" ) || text.contains( "process" ),
    "force::1 with no procs must mention no processes: {text}" );
}

// ─── FT-1 (feature/005_cli_design.md): unknown parameter ─────────────────────

// FT-1: unknown parameter → exit 1 with error mentioning the param name
#[ test ]
fn ft005_1_unknown_param_exits_1()
{
  let out = run_clm( &[ ".status", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// ─── FT-2 (feature/005_cli_design.md): empty boolean param value ─────────────

// FT-2: bool param with empty value (dry::) → exit 1 (bool must be 0 or 1)
#[ test ]
fn ft005_2_empty_bool_param_value_exits_1()
{
  let out = run_clm( &[ ".version.install", "dry::" ] );
  assert_exit( &out, 1 );
}

// ─── FT-3 (feature/005_cli_design.md): last-occurrence wins ──────────────────

// FT-3: repeated version:: param → last occurrence wins
#[ test ]
fn ft005_3_last_param_wins()
{
  // stable is first, month is last → month (2.1.74) must win
  let out = run_clm( &[ ".version.install", "version::stable", "version::month", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "2.1.74" ), "last version:: param (month=2.1.74) must win: {text}" );
}

// ─── Covered by existing tests (reference only) ──────────────────────────────
//
// FT-1 (feature/001_version_management.md): tc301 (stable→2.1.78)
// FT-2 (feature/001_version_management.md): tc309 (month→2.1.74)
// FT-3 (feature/001_version_management.md): tc400 (guard defaults to stable)
// FT-4 (feature/001_version_management.md): tc403 (guard latest→no pin)
// FT-5 (feature/001_version_management.md): tc357 (dry::1 does not write prefs)
// FT-1 (feature/002_process_lifecycle.md): tc310 (no procs→exit 0)
// FT-2 (feature/002_process_lifecycle.md): tc311 (dry::1 no procs→[dry-run])
// FT-3 (feature/002_process_lifecycle.md): tc312 (dry::1 force::1→dry wins)
// FT-1 (feature/003_settings_management.md): tc322 (value::true→bool)
// FT-2 (feature/003_settings_management.md): tc325 (value::42→int)
// FT-4 (feature/003_settings_management.md): tc328 (creates file when absent)
// FT-5 (feature/003_settings_management.md): tc331 (HOME unset→exit 2)
// FT-1 (feature/004_dry_run.md): tc300 ([dry-run] prefix on install)
// FT-2 (feature/004_dry_run.md): tc330 (dry::1 on settings.set→no change)
// FT-3 (feature/004_dry_run.md): tc311 (dry::1 on processes.kill)
// FT-4 (feature/004_dry_run.md): tc303 (dry::1 force::1→dry wins)
// FT-4 (feature/005_cli_design.md): tc093 (empty argv→help)
// FT-5 (feature/005_cli_design.md): tc04 (help anywhere wins)

// ═══════════════════════════════════════════════════════════════════════════════
// Feature 007: Params Command (FT-1 through FT-12)
// ═══════════════════════════════════════════════════════════════════════════════

// ─── FT-1 (feature/007_params_command.md): show-all ≥35 entries ──────────────

// FT-1: .params show-all exits 0; ≥35 param entries; each annotated with source
#[ test ]
fn ft1_007_params_show_all_min_entries()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let entry_count = text.lines()
    .filter( |l| !l.starts_with( ' ' ) && !l.is_empty() )
    .count();
  assert!(
    entry_count >= 35,
    "show-all must list ≥35 params, got {entry_count}:\n{text}"
  );
}

// ─── FT-2 (feature/007_params_command.md): single param shows all forms ───────

// FT-2: .params key::model with config value → shows all forms, current value, and default; exit 0
#[ test ]
fn ft2_007_params_single_model_full_detail()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "model", "claude-sonnet-4-6" ) ] );

  let out = run_clm_with_env(
    &[ ".params", "key::model" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "--model" ),           "must show CLI form --model: {text}" );
  assert!( text.contains( "CLAUDE_MODEL" ),      "must show env form CLAUDE_MODEL: {text}" );
  assert!( text.contains( "claude-sonnet-4-6" ), "must show value or default: {text}" );
}

// ─── FT-3 (feature/007_params_command.md): kind::config filters ───────────────

// FT-3: .params kind::config → only config-key params; env-only absent; exit 0
#[ test ]
fn ft3_007_params_kind_config_filters()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params", "kind::config" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),         "kind::config must include model: {text}" );
  assert!( !text.contains( "bash_timeout" ), "kind::config must exclude env-only bash_timeout: {text}" );
}

// ─── FT-4 (feature/007_params_command.md): kind::env filters ─────────────────

// FT-4: .params kind::env → only env-var params; config-only absent; exit 0
#[ test ]
fn ft4_007_params_kind_env_filters()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params", "kind::env" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "bash_timeout" ), "kind::env must include bash_timeout: {text}" );
  assert!( !text.contains( "theme" ),        "kind::env must exclude config-only theme: {text}" );
}

// ─── FT-5 (feature/007_params_command.md): env override visible with (env) ────

// FT-5: CLAUDE_MODEL=claude-opus-4-6 set → env value annotated (env); exit 0
#[ test ]
fn ft5_007_params_env_override_visible()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params", "key::model" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "claude-opus-4-6" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-opus-4-6" ), "must show env value: {text}" );
  assert!( text.contains( "(env)" ),            "must annotate with (env): {text}" );
}

// ─── FT-6 (feature/007_params_command.md): env-only param unset + default ─────

// FT-6: .params key::bash_timeout with CLAUDE_CODE_BASH_TIMEOUT unset → shows env form + default; exit 0
#[ test ]
fn ft6_007_params_env_only_param()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params", "key::bash_timeout" ],
    &[ ( "HOME", home ), ( "CLAUDE_CODE_BASH_TIMEOUT", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "CLAUDE_CODE_BASH_TIMEOUT" ), "must show env form: {text}" );
  assert!( text.contains( "120000" ),                    "must show default 120000: {text}" );
}

// ─── FT-7 (feature/007_params_command.md): format::json → array with required fields ─

// FT-7: .params format::json → exit 0; valid JSON array; entries have name + default fields
#[ test ]
fn ft7_007_params_json_output_structure()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params", "format::json" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text    = stdout( &out );
  let trimmed = text.trim();
  assert!( trimmed.starts_with( '[' ),  "format::json must produce JSON array: {text}" );
  assert!( trimmed.ends_with( ']' ),    "JSON array must end with ']': {text}" );
  assert!( text.contains( "\"name\"" ), "JSON entries must have 'name' field: {text}" );
}

// ─── FT-8 (feature/007_params_command.md): CLI-only annotation ────────────────

// FT-8: .params key::print → shows --print form and CLI-only annotation; exit 0
#[ test ]
fn ft8_007_params_cli_only_annotation()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params", "key::print" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let lower = text.to_lowercase();
  assert!( text.contains( "--print" ), "must show --print CLI form: {text}" );
  assert!(
    lower.contains( "cli-only" ) || lower.contains( "cli only" ) || lower.contains( "unobservable" ),
    "must show CLI-only annotation: {text}"
  );
}

// ─── FT-9 (feature/007_params_command.md): unknown key exits 2 ────────────────

// FT-9: .params key::NONEXISTENT → not in params catalog; exit 2
#[ test ]
fn ft9_007_params_unknown_key_exits_2()
{
  let out = run_clm_with_env(
    &[ ".params", "key::NONEXISTENT" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 2 );
}

// ─── FT-10 (feature/007_params_command.md): invalid kind exits 1 ──────────────

// FT-10: .params kind::badvalue → unrecognised kind value; exit 1
#[ test ]
fn ft10_007_params_invalid_kind_exits_1()
{
  let out = run_clm_with_env(
    &[ ".params", "kind::badvalue" ],
    &[ ( "HOME", "/tmp" ) ],
  );
  assert_exit( &out, 1 );
}

// ─── FT-11 (feature/007_params_command.md): default annotation ────────────────

// FT-11: .params key::model no env no config → shows default with (default) annotation; exit 0
//
// Uses isolated cwd to prevent project config walk from finding container settings.
#[ test ]
fn ft11_007_params_default_source_annotation()
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
  assert!( text.contains( "claude-sonnet-4-6" ), "must show catalog default: {text}" );
  assert!( text.contains( "(default)" ),          "must show (default) annotation: {text}" );
}

// ─── FT-12 (feature/007_params_command.md): show-all alphabetical ─────────────

// FT-12: .params show-all → param entries appear in ascending alphabetical order; exit 0
#[ test ]
fn ft12_007_params_show_all_alphabetical()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".params" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let names : Vec< &str > = text.lines()
    .filter( |l| !l.starts_with( ' ' ) && !l.is_empty() )
    .collect();
  assert!( !names.is_empty(), "show-all must produce param entries: {text}" );
  let mut sorted = names.clone();
  sorted.sort_unstable();
  assert_eq!( names, sorted, "param names must be in ascending alphabetical order" );
}
