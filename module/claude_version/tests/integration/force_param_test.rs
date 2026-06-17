//! EC- edge-case tests for the `force::` parameter.
//!
//! Covers gap cases from `tests/docs/cli/param/03_force.md`.
//! EC-1, EC-3..EC-6, EC-10, EC-11 are covered in `cli_args_test.rs`,
//! `mutation_commands_test.rs`, and `cross_cutting_test.rs`.

use crate::helpers::{ assert_exit, run_clm, stdout };

/// EC-7: `force::` only for `.version.install`, `.version.guard`, `.processes.kill`
#[ test ]
fn force_ec7_command_scope_rejects_on_settings_set()
{
  let out = run_clm( &[ ".settings.set", "key::k", "value::v", "force::1" ] );
  assert_exit( &out, 1 );
}

/// EC-8: Default (absent) → `force::0` — guard active, no forced reinstall
#[ test ]
fn force_ec8_default_force_zero()
{
  let out = run_clm( &[ ".version.guard", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "[forced]" ) && !text.contains( "force" ),
    "without force::1, guard must not show force indicator: {text}"
  );
}

/// EC-9: `force::0` explicit → same as absent
#[ test ]
fn force_ec9_explicit_zero_same_as_absent()
{
  let out_absent  = run_clm( &[ ".version.guard", "dry::1" ] );
  let out_zero    = run_clm( &[ ".version.guard", "force::0", "dry::1" ] );
  assert_exit( &out_absent, 0 );
  assert_exit( &out_zero, 0 );
  let text_absent = stdout( &out_absent );
  let text_zero   = stdout( &out_zero );
  assert_eq!( text_absent, text_zero, "force::0 must produce same output as absent force::" );
}

/// EC-10: `dry::1 force::1` on `.processes.kill` → dry wins
#[ test ]
fn force_ec10_processes_kill_dry_wins()
{
  let out = run_clm( &[ ".processes.kill", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ) || text.contains( "dry" ) || text.contains( "no active" ),
    "dry must win over force on .processes.kill: {text}"
  );
}

/// EC-11: `.version.guard force::1 dry::1` → dry wins
#[ test ]
fn force_ec11_version_guard_dry_wins_over_force()
{
  let out = run_clm( &[ ".version.guard", "force::1", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry must win over force on .version.guard: {text}" );
}
