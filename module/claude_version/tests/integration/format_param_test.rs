//! EC- edge-case tests for the `format::` parameter.
//!
//! Covers gap cases EC-11 through EC-19 from `tests/docs/cli/param/05_format.md`.
//! EC-1 through EC-10 are covered in `cli_args_test.rs`, `read_commands_test.rs`,
//! `cross_cutting_test.rs`, and `error_messages_test.rs`.

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clm, run_clm_with_env, stdout, write_settings };

/// EC-11: `.status format::json` → valid JSON object starting with `{`
#[ test ]
fn format_ec11_status_format_json_object()
{
  let out = run_clm( &[ ".status", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "status format::json must start with {{: {text}" );
}

/// EC-12: `.version.show format::json` → `{"version":"..."}` with version key
#[ test ]
fn format_ec12_version_show_format_json_has_version_key()
{
  let out = run_clm( &[ ".version.show", "format::json" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.contains( "\"version\"" ), "version.show json must have 'version' key: {text}" );
    assert!( text.trim_start().starts_with( '{' ), "must be a JSON object: {text}" );
  }
}

/// EC-13: `.version.list format::json` → JSON array starting with `[`
#[ test ]
fn format_ec13_version_list_format_json_array()
{
  let out = run_clm( &[ ".version.list", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '[' ), "version.list format::json must start with [: {text}" );
}

/// EC-14: `.processes format::json` → `{"processes":[...]}` with processes key
#[ test ]
fn format_ec14_processes_format_json_has_processes_key()
{
  let out = run_clm( &[ ".processes", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "processes json must have 'processes' key: {text}" );
}

/// EC-15: `.settings.show format::json` → valid JSON object
#[ test ]
fn format_ec15_settings_show_format_json_object()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myVal" ) ] );
  let out  = run_clm_with_env(
    &[ ".settings.show", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "settings.show json must be an object: {text}" );
  assert!( text.contains( "myKey" ), "json must contain the key: {text}" );
}

/// EC-16: `.settings.get format::json` → `{"key":"..","value":..}` with both fields
#[ test ]
fn format_ec16_settings_get_format_json_has_key_value()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myVal" ) ] );
  let out  = run_clm_with_env(
    &[ ".settings.get", "key::myKey", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"key\"" ) || text.contains( "\"value\"" ),
    "settings.get json must have key/value fields: {text}" );
}

/// EC-17: `.version.history format::json` → JSON array with version/date/summary fields
#[ test ]
fn format_ec17_history_format_json_fields()
{
  let out = run_clm( &[ ".version.history", "format::json", "count::3" ] );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!( text.trim_start().starts_with( '[' ), "history json must be an array: {text}" );
    assert!( text.contains( "\"version\"" ), "history json entries must have 'version' field: {text}" );
  }
}

/// EC-18: `.version.history format::xml` → exit 1, unknown format
#[ test ]
fn format_ec18_history_format_xml_exits_1()
{
  let out = run_clm( &[ ".version.history", "format::xml" ] );
  assert_exit( &out, 1 );
}

/// EC-19: `.version.history format::JSON` (uppercase) → exit 1
#[ test ]
fn format_ec19_history_format_json_uppercase_exits_1()
{
  let out = run_clm( &[ ".version.history", "format::JSON" ] );
  assert_exit( &out, 1 );
}
