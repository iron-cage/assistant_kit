//! EC- edge-case tests for the `version::` parameter.
//!
//! Covers gap cases from `tests/docs/cli/param/01_version.md`.
//! EC-1 through EC-6 and EC-15/EC-16 are already implemented in
//! `cli_args_test.rs` and `integration/mutation_commands_test.rs`.

use crate::helpers::{ assert_exit, run_clm, stdout };

/// EC-7: `version::LATEST` → wrong case, exit 1
#[ test ]
fn version_ec7_latest_wrong_case_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::LATEST" ] );
  assert_exit( &out, 1 );
}

/// EC-8: `version::MONTH` → wrong case, exit 1
#[ test ]
fn version_ec8_month_wrong_case_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::MONTH" ] );
  assert_exit( &out, 1 );
}

/// EC-9: `version::` only accepted by `.version.install` and `.version.guard`
#[ test ]
fn version_ec9_command_scope_rejects_on_processes()
{
  let out = run_clm( &[ ".processes", "version::stable" ] );
  assert_exit( &out, 1 );
}

/// EC-10: `version::stable dry::1` → resolves to stable alias
#[ test ]
fn version_ec10_stable_alias_dry()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "output must reference stable alias: {text}" );
}

/// EC-11: `version::month dry::1` → resolves to pinned semver
#[ test ]
fn version_ec11_month_alias_dry()
{
  let out = run_clm( &[ ".version.install", "version::month", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "month" ), "output must reference month alias: {text}" );
}

/// EC-12: `version::latest dry::1` → no-lock unlock mode
#[ test ]
fn version_ec12_latest_alias_dry()
{
  let out = run_clm( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "latest" ), "output must reference latest alias: {text}" );
}

/// EC-13: `version::1.2.3 dry::1` → exact semver accepted
#[ test ]
fn version_ec13_exact_semver_dry()
{
  let out = run_clm( &[ ".version.install", "version::1.2.3", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "1.2.3" ), "output must contain 1.2.3: {text}" );
}

/// EC-14: `version::2.1.50 dry::1` → older semver accepted
#[ test ]
fn version_ec14_older_semver_dry()
{
  let out = run_clm( &[ ".version.install", "version::2.1.50", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "2.1.50" ), "output must contain 2.1.50: {text}" );
}
