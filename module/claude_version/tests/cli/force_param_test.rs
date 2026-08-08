//! EC- edge-case tests for the `force::` parameter.
//!
//! Covers gap cases from `tests/docs/cli/param/03_force.md`.
//! EC-1, EC-3..EC-6, EC-10, EC-11 are covered in `cli_args_test.rs`,
//! `mutation_commands_test.rs`, and `cross_cutting_test.rs`.

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout, write_settings };

/// EC-7: `force::` only for `.version.install`, `.version.guard`, `.ps.kill`
#[ test ]
fn force_ec7_command_scope_rejects_on_settings_set()
{
  let out = run_clv( &[ ".settings.set", "key::k", "value::v", "force::1" ] );
  assert_exit( &out, 1 );
}

/// EC-8: Default (absent) → `force::0` — guard active, no forced reinstall
#[ test ]
fn force_ec8_default_force_zero()
{
  let out = run_clv( &[ ".version.guard", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "[forced]" ) && !text.contains( "force" ),
    "without force::1, guard must not show force indicator: {text}"
  );
}

/// EC-9: `force::0` explicit → same as absent
///
/// Uses an isolated HOME with a known symlink and settings so parallel tests
/// cannot disturb the installed-version state between the two subprocess calls.
#[ test ]
fn force_ec9_explicit_zero_same_as_absent()
{
  let dir = tempfile::TempDir::new().expect( "tempdir" );
  write_settings( dir.path(), &[
    ( "preferredVersionSpec",     "stable"  ),
    ( "preferredVersionResolved", "2.1.220" ),
  ] );
  // Provide a stable symlink so get_version_from_symlink returns "2.1.220".
  let local_bin = dir.path().join( ".local" ).join( "bin" );
  std::fs::create_dir_all( &local_bin ).expect( "create .local/bin" );
  std::os::unix::fs::symlink( "2.1.220", local_bin.join( "claude" ) )
    .expect( "claude symlink" );
  let home = dir.path().to_str().expect( "utf8 home" );
  let env  = &[ ( "HOME", home ) ];
  let out_absent = run_clv_with_env( &[ ".version.guard", "dry::1" ], env );
  let out_zero   = run_clv_with_env( &[ ".version.guard", "force::0", "dry::1" ], env );
  assert_exit( &out_absent, 0 );
  assert_exit( &out_zero, 0 );
  let text_absent = stdout( &out_absent );
  let text_zero   = stdout( &out_zero );
  assert_eq!( text_absent, text_zero, "force::0 must produce same output as absent force::" );
}

/// EC-10: `dry::1 force::1` on `.ps.kill` → dry wins
#[ test ]
fn force_ec10_ps_kill_dry_wins()
{
  let out = run_clv( &[ ".ps.kill", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ) || text.contains( "dry" ) || text.contains( "no active" ),
    "dry must win over force on .ps.kill: {text}"
  );
}

/// EC-11: `.version.guard force::1 dry::1` → dry wins
#[ test ]
fn force_ec11_version_guard_dry_wins_over_force()
{
  let out = run_clv( &[ ".version.guard", "force::1", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry must win over force on .version.guard: {text}" );
}
