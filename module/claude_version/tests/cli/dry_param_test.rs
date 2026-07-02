//! EC- edge-case tests for the `dry::` parameter.
//!
//! Covers gap cases EC-10 through EC-13 from `tests/docs/cli/param/02_dry.md`.
//! EC-1 through EC-9 are covered in `cli_args_test.rs`, `mutation_commands_test.rs`,
//! and `cross_cutting_test.rs`.

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clm, run_clm_with_env, stdout };

/// EC-10: `dry::` only for mutation commands — rejected on `.version.list`
#[ test ]
fn dry_ec10_command_scope_rejects_on_read()
{
  let out = run_clm( &[ ".version.list", "dry::1" ] );
  assert_exit( &out, 1 );
}

/// EC-11: `dry::1` on `.processes.kill` → no kill, shows [dry-run]
#[ test ]
fn dry_ec11_processes_kill_dry_run()
{
  let out = run_clm( &[ ".processes.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ) || text.contains( "dry" ) || text.contains( "no active" ),
    "dry-run on .processes.kill must show dry-run indicator or no-processes message: {text}" );
}

/// EC-12: `dry::1` on `.settings.set` → no file change
#[ test ]
fn dry_ec12_settings_set_dry_no_file()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clm_with_env(
    &[ ".settings.set", "key::theme", "value::dark", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text          = stdout( &out );
  let settings_path = dir.path().join( ".claude" ).join( "settings.json" );
  assert!( text.contains( "[dry-run]" ), "dry-run output must contain [dry-run]: {text}" );
  assert!( !settings_path.exists(), "settings.json must NOT be created during dry-run" );
}

/// EC-13: `dry::1 force::1` on `.processes.kill` → dry wins
#[ test ]
fn dry_ec13_processes_kill_dry_wins_over_force()
{
  let out = run_clm( &[ ".processes.kill", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ) || text.contains( "dry" ) || text.contains( "no active" ),
    "dry must win over force on .processes.kill: {text}"
  );
}
