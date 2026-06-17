//! EC- edge-case tests for the `key::` parameter.
//!
//! Covers gap cases from `tests/docs/cli/param/06_key.md`.
//! EC-3 and EC-4 are covered in `integration/mutation_commands_test.rs` (tc320, tc332).
//! EC-10 is covered in `integration/read_commands_test.rs` and `error_messages_test.rs`.

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stderr, stdout, write_settings };

/// EC-1: `key::existing` on `.settings.get` → returns value, exit 0
#[ test ]
fn key_ec1_existing_key_returns_value()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myVal" ) ] );
  let out  = run_clm_with_env(
    &[ ".settings.get", "key::myKey" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "myVal" ), "output must contain the stored value: {text}" );
}

/// EC-2: `key::nonexistent` → exit 2, key not found
#[ test ]
fn key_ec2_nonexistent_key_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "otherKey", "val" ) ] );
  let out  = run_clm_with_env(
    &[ ".settings.get", "key::nosuchkey" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

/// EC-5: `key::` (empty value) on `.settings.get` → exit 1
#[ test ]
fn key_ec5_empty_key_on_get_exits_1()
{
  let out = run_clm( &[ ".settings.get", "key::" ] );
  assert_exit( &out, 1 );
}

/// EC-6: `key::` only accepted by `.settings.get` and `.settings.set`
#[ test ]
fn key_ec6_command_scope_rejects_on_status()
{
  let out = run_clm( &[ ".status", "key::foo" ] );
  assert_exit( &out, 1 );
}

/// EC-7: `key::a b c` (key with spaces) → behavior defined by spec
#[ test ]
fn key_ec7_key_with_spaces_behavior()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clm_with_env(
    &[ ".settings.set", "key::a b c", "value::x" ],
    &[ ( "HOME", home ) ],
  );
  // The spec says "behavior is defined" — check it is consistent (exit 0 or exit 1, no crash)
  let code = out.status.code().unwrap_or( -1 );
  assert!( code == 0 || code == 1,
    "key with spaces must exit 0 or 1 consistently, got: {code}" );
}

/// EC-8: `key::foo.bar` (dot in key name) → stored and retrieved as given
#[ test ]
fn key_ec8_dot_in_key_round_trips()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let set_out = run_clm_with_env(
    &[ ".settings.set", "key::foo.bar", "value::baz" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &set_out, 0 );
  let get_out = run_clm_with_env(
    &[ ".settings.get", "key::foo.bar" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &get_out, 0 );
  let text = stdout( &get_out );
  assert!( text.contains( "baz" ), "key foo.bar must round-trip to value baz: {text}" );
}

/// EC-9: `key::foo bar` (space in key) → stored as given opaque string
#[ test ]
fn key_ec9_space_in_key_round_trips()
{
  let dir     = TempDir::new().unwrap();
  let home    = dir.path().to_str().unwrap();
  let set_out = run_clm_with_env(
    &[ ".settings.set", "key::foo bar", "value::baz" ],
    &[ ( "HOME", home ) ],
  );
  let code = set_out.status.code().unwrap_or( -1 );
  // Spec allows exit 0 or exit 1 — if exit 0, verify round-trip
  if code == 0
  {
    let get_out = run_clm_with_env(
      &[ ".settings.get", "key::foo bar" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &get_out, 0 );
    let text = stdout( &get_out );
    assert!( text.contains( "baz" ), "key 'foo bar' round-trip must return baz: {text}" );
  }
  else
  {
    assert_eq!( code, 1, "key with space must exit 0 or 1, not {code}" );
  }
}

/// EC-10: Without `key::` on `.settings.get` → error message contains `key::`
#[ test ]
fn key_ec10_missing_key_error_contains_key_token()
{
  let out = run_clm( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!( combined.contains( "key::" ), "error must mention 'key::': {combined}" );
}

/// EC-11: Without `key::` on `.settings.set` → error message contains `key::`
#[ test ]
fn key_ec11_missing_key_on_set_error_contains_key_token()
{
  let out = run_clm( &[ ".settings.set", "value::dark" ] );
  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!( combined.contains( "key::" ), "error must mention 'key::': {combined}" );
}
