//! EC- edge-case tests for the `value::` parameter.
//!
//! Covers gap cases from `tests/docs/cli/param/07_value.md`.

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout };

fn read_settings_json( home_dir : &std::path::Path ) -> String
{
  let path = home_dir.join( ".claude" ).join( "settings.json" );
  std::fs::read_to_string( &path ).unwrap_or_default()
}

/// EC-1: `value::true` → JSON boolean `true` (unquoted)
#[ test ]
fn value_ec1_true_stored_as_boolean()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::flag", "value::true" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"flag\": true" ) || json.contains( "\"flag\":true" ),
    "value::true must be stored as unquoted boolean: {json}" );
  assert!( !json.contains( "\"true\"" ), "must not be stored as quoted string: {json}" );
}

/// EC-2: `value::0` → JSON integer `0` (NOT boolean false)
#[ test ]
fn value_ec2_zero_stored_as_integer_not_boolean()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::n", "value::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"n\": 0" ) || json.contains( "\"n\":0" ),
    "value::0 must be stored as integer 0: {json}" );
  assert!( !json.contains( "\"n\": false" ) && !json.contains( "\"n\":false" ),
    "value::0 must NOT be stored as boolean false: {json}" );
}

/// EC-4: `value::` only for `.settings.set` — rejected on `.settings.get`
#[ test ]
fn value_ec4_command_scope_rejects_on_get()
{
  let out = run_clv( &[ ".settings.get", "key::k", "value::v" ] );
  assert_exit( &out, 1 );
}

/// EC-5: `value::1.5` → JSON float (not integer)
#[ test ]
fn value_ec5_float_stored_as_json_float()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::f", "value::1.5" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "1.5" ), "value::1.5 must be stored as float: {json}" );
  assert!( !json.contains( "\"1.5\"" ), "float must not be quoted: {json}" );
}

/// EC-6: `value::NaN` → JSON string (not a number)
#[ test ]
fn value_ec6_nan_stored_as_string()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::x", "value::NaN" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"NaN\"" ), "value::NaN must be stored as quoted string: {json}" );
}

/// EC-7: Round-trip — set then get returns identical value
#[ test ]
fn value_ec7_round_trip_set_get()
{
  let dir     = TempDir::new().unwrap();
  let home    = dir.path().to_str().unwrap();
  let set_out = run_clv_with_env(
    &[ ".settings.set", "key::roundtrip", "value::hello" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &set_out, 0 );
  let get_out = run_clv_with_env(
    &[ ".settings.get", "key::roundtrip" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &get_out, 0 );
  let text = stdout( &get_out );
  assert!( text.contains( "hello" ), "round-trip must return stored value: {text}" );
}

/// EC-8: `value::Infinity` → JSON string (infinite float not stored as number)
#[ test ]
fn value_ec8_infinity_stored_as_string()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::x", "value::Infinity" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"Infinity\"" ), "value::Infinity must be stored as quoted string: {json}" );
}

/// EC-9: `value::true false` (space in value) → JSON string
#[ test ]
fn value_ec9_space_in_value_stored_as_string()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::x", "value::true false" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"true false\"" ),
    "space-containing value must be stored as quoted string: {json}" );
}

/// EC-11: `value::false` → JSON boolean `false` (unquoted)
#[ test ]
fn value_ec11_false_stored_as_boolean()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::flag", "value::false" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"flag\": false" ) || json.contains( "\"flag\":false" ),
    "value::false must be stored as unquoted boolean: {json}" );
  assert!( !json.contains( "\"false\"" ), "must not be stored as quoted string: {json}" );
}

/// EC-12: `value::42` → JSON integer `42` (unquoted)
#[ test ]
fn value_ec12_integer_stored_unquoted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::count", "value::42" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"count\": 42" ) || json.contains( "\"count\":42" ),
    "value::42 must be stored as unquoted integer: {json}" );
  assert!( !json.contains( "\"42\"" ), "must not be stored as quoted string: {json}" );
}

/// EC-13: `value::hello` → JSON string `"hello"` (quoted)
#[ test ]
fn value_ec13_string_stored_quoted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::name", "value::hello" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let json = read_settings_json( dir.path() );
  assert!( json.contains( "\"hello\"" ), "value::hello must be stored as quoted string: {json}" );
}

/// EC-10: `value::` (empty) → exit 1; empty value not accepted
#[ test ]
fn value_ec10_empty_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".settings.set", "key::theme", "value::" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

/// EC-14: `key::theme` present but no `value::` → exit 1
#[ test ]
fn value_ec14_missing_value_exits_1()
{
  let out = run_clv( &[ ".settings.set", "key::theme" ] );
  assert_exit( &out, 1 );
}

/// EC-15: without `value::` → error message contains the string `value::`
#[ test ]
fn value_ec15_missing_value_error_mentions_value_token()
{
  let out = run_clv( &[ ".settings.set", "key::theme" ] );
  assert_exit( &out, 1 );
  let err = crate::subprocess_helpers::stderr( &out );
  assert!(
    err.contains( "value::" ),
    "error must mention 'value::' token: {err}"
  );
}
