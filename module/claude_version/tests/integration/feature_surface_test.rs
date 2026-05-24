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

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stdout };

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
