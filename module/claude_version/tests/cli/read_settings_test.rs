//! Integration tests for `.settings.show` and `.settings.get` — E8, E9.
//!
//! Also covers required-flag error format (TC-237–TC-239) and JSON type
//! preservation (TC-241, TC-490–TC-492).
//!
//! ## E8 — `.settings.show`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-161 | `.settings.show` file missing → exit 2 | N | 2 |
//! | TC-162 | `.settings.show` empty {} → empty output, exit 0 | P | 0 |
//! | TC-163 | `.settings.show` valid settings → keys shown, exit 0 | P | 0 |
//! | TC-164 | `.settings.show v::0` → key=value format | P | 0 |
//! | TC-167 | `.settings.show format::json` → valid JSON | P | 0 |
//! | TC-170 | `.settings.show` malformed file → exit 2 | N | 2 |
//! | TC-171 | `.settings.show` HOME not set → exit 2 | N | 2 |
//! | IT-5 | `bogus::x` → exit 1, unknown parameter | N | 1 |
//! | IT-6 | `format::xml` → exit 1, unknown format | N | 1 |
//! | IT-7 | `v::3` → exit 1, out of range | N | 1 |
//! | IT-8 | stdout non-empty; stderr empty | P | 0 |
//!
//! ## E9 — `.settings.get`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-174 | `.settings.get` no `key::` → exit 1 | N | 1 |
//! | TC-176 | `.settings.get key::existing` → value, exit 0 | P | 0 |
//! | TC-177 | `.settings.get key::nonexistent` → exit 2 | N | 2 |
//! | TC-179 | `.settings.get v::0` → bare value only | P | 0 |
//! | TC-180 | `.settings.get v::1` → "key: value" | P | 0 |
//! | TC-182 | `.settings.get format::json` → {"key":"..","value":..} | P | 0 |
//! | TC-184 | `.settings.get` file missing → exit 2 | N | 2 |
//!
//! ## FR — Required-flag error format
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-237 | `.settings.get` without `key::` → error mentions `key::` | N | 1 |
//! | TC-238 | `.settings.set` without `key::` → error mentions `key::` | N | 1 |
//! | TC-239 | `.settings.set key::foo` without `value::` → error mentions `value::` | N | 1 |
//!
//! ## JSON type preservation
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-241 | `.settings.show format::json` preserves bool/number types | P | 0 |
//! | TC-490 | `.settings.get key::boolKey format::json` bool → `"value":true` (unquoted) | P | 0 |
//! | TC-491 | `.settings.get key::numKey format::json` number → `"value":42` (unquoted) | P | 0 |
//! | TC-492 | `.settings.get key::strKey format::json` string → `"value":"hello"` (quoted) | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{
  assert_exit, run_clv, run_clv_with_env, stderr, stdout, write_settings,
};

// ─── E8: settings show ───────────────────────────────────────────────────────

// TC-161: file missing → exit 2
#[ test ]
fn tc161_settings_show_file_missing_exits_2()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// TC-162: empty {} → empty output, exit 0
#[ test ]
fn tc162_settings_show_empty_file_exits_0()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[] );
  let out = run_clv_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().is_empty(), "empty settings must produce no output, got: {text}" );
}

// TC-163: valid settings → keys shown, exit 0
#[ test ]
fn tc163_settings_show_valid_file()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myValue" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "myKey" ),   "output must contain key 'myKey': {text}" );
  assert!( text.contains( "myValue" ), "output must contain value 'myValue': {text}" );
}

// TC-164: v::0 → key=value format
#[ test ]
fn tc164_settings_show_v0_key_equals_value()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "alpha", "beta" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.show", "v::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "alpha=beta" ) || text.contains( "alpha: beta" ),
    "v::0 must format key=value, got: {text}"
  );
}

// TC-167: format::json → valid JSON
#[ test ]
fn tc167_settings_show_format_json()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "k1", "v1" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.show", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "format::json must start with '{{': {text}" );
  assert!( text.contains( "k1" ), "JSON must contain key 'k1': {text}" );
}

// TC-170: malformed JSON → exit 2
#[ test ]
fn tc170_settings_show_malformed_file_exits_2()
{
  let dir = TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{ bad json!!" ).unwrap();
  let out = run_clv_with_env(
    &[ ".settings.show" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// TC-171: HOME not set → exit 2
#[ test ]
fn tc171_settings_show_no_home_exits_2()
{
  let out = run_clv_with_env( &[ ".settings.show" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

// IT-5: `bogus::x` → exit 1 (unknown parameter)
#[ test ]
fn it05_settings_show_bogus_param_exits_1()
{
  let out = run_clv( &[ ".settings.show", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// IT-6: `format::xml` → exit 1 (unknown format)
#[ test ]
fn it06_settings_show_format_xml_exits_1()
{
  let out = run_clv( &[ ".settings.show", "format::xml" ] );
  assert_exit( &out, 1 );
}

// IT-7: `v::3` → exit 1 (out of range)
#[ test ]
fn it07_settings_show_v3_exits_1()
{
  let out = run_clv( &[ ".settings.show", "v::3" ] );
  assert_exit( &out, 1 );
}

// IT-8: stdout is non-empty; stderr is empty on successful invocation
#[ test ]
fn it08_settings_show_stdout_only()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clv_with_env( &[ ".settings.show" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !stdout( &out ).is_empty(), "stdout must be non-empty for valid settings" );
  assert!( stderr( &out ).is_empty(), "stderr must be empty on success: {}", stderr( &out ) );
}

// ─── E9: settings get ────────────────────────────────────────────────────────

// TC-174: no key:: → exit 1
#[ test ]
fn tc174_settings_get_no_key_exits_1()
{
  let out = run_clv( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
}

// TC-176: key::existing → value, exit 0
#[ test ]
fn tc176_settings_get_existing_key()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "myKey", "myValue" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.get", "key::myKey" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "myValue" ), "output must contain 'myValue': {text}" );
}

// TC-177: key::nonexistent → exit 2
#[ test ]
fn tc177_settings_get_missing_key_exits_2()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "existing", "val" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.get", "key::nosuchkey" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// TC-179: v::0 → bare value only (no label)
#[ test ]
fn tc179_settings_get_v0_bare_value()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "k", "thevalue" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.get", "key::k", "v::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "thevalue",
    "v::0 must be bare value only, got: {text}"
  );
}

// TC-180: v::1 → "key: value"
#[ test ]
fn tc180_settings_get_v1_labeled()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "mykey", "myval" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.get", "key::mykey", "v::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "mykey" ) && text.contains( "myval" ),
    "v::1 must show 'key: value', got: {text}" );
}

// TC-182: format::json → {"key":"..","value":".."}
#[ test ]
fn tc182_settings_get_format_json()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "alpha", "omega" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.get", "key::alpha", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"key\"" ) && text.contains( "\"value\"" ),
    "format::json must have 'key' and 'value' fields: {text}" );
}

// TC-184: file missing → exit 2
#[ test ]
fn tc184_settings_get_file_missing_exits_2()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.get", "key::anything" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
}

// ─── FR: required-flag error format ──────────────────────────────────────────

// TC-237: .settings.get without `key::` → error contains "key:: is required"
#[ test ]
fn tc237_settings_get_missing_key_error_format()
{
  let out = run_clv( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key:: is required" ), "error must contain 'key:: is required': {err}" );
}

// TC-238: .settings.set without `key::` → error contains "key:: is required"
#[ test ]
fn tc238_settings_set_missing_key_error_format()
{
  let out = run_clv( &[ ".settings.set" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key:: is required" ), "error must contain 'key:: is required': {err}" );
}

// TC-239: .settings.set with `key::` but no `value::` → "value:: is required"
#[ test ]
fn tc239_settings_set_missing_value_error_format()
{
  let out = run_clv( &[ ".settings.set", "key::foo" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value:: is required" ), "error must contain 'value:: is required': {err}" );
}

// ─── JSON type preservation ───────────────────────────────────────────────────

// TC-241: .settings.show format::json preserves bool and number types
#[ test ]
fn tc241_settings_show_json_preserves_types()
{
  let dir = tempfile::TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    "{\"boolKey\":true,\"numKey\":42,\"strKey\":\"hello\"}"
  ).unwrap();
  let out = run_clv_with_env(
    &[ ".settings.show", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ":true" ), "bool must be unquoted in JSON: {text}" );
  assert!( text.contains( ":42" ), "number must be unquoted in JSON: {text}" );
  assert!( text.contains( ":\"hello\"" ), "string must be quoted in JSON: {text}" );
}

// Root Cause: settings_get_routine always emitted value as a quoted JSON string
//   regardless of its actual type — {"key":"autoUpdates","value":"true"} instead
//   of {"key":"autoUpdates","value":true}.
// Why Not Caught: TC-182 only checked for presence of "key"/"value" fields, not
//   that the value type matched the stored type.
// Fix Applied: JSON branch now calls infer_type(v) and branches on StoredAs to
//   emit bare Bool/Number/Raw or quoted Str, matching settings_show_routine.
// Prevention: TC-490–492 assert the specific JSON value representations.
// Pitfall: write_settings helper quotes all values as strings; use direct JSON
//   writes with raw booleans/numbers to test real Claude settings behaviour.

// TC-490: settings_get bool value → JSON output has unquoted true
#[ test ]
fn tc490_settings_get_json_bool_unquoted()
{
  let dir = tempfile::TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{\"autoUpdates\":true}" ).unwrap();
  let out = run_clv_with_env(
    &[ ".settings.get", "key::autoUpdates", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\"value\":true" ),
    "bool value must be unquoted in JSON (was: value:\"true\"): {text}",
  );
}

// TC-491: settings_get number value → JSON output has unquoted number
#[ test ]
fn tc491_settings_get_json_number_unquoted()
{
  let dir = tempfile::TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{\"maxRetries\":42}" ).unwrap();
  let out = run_clv_with_env(
    &[ ".settings.get", "key::maxRetries", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\"value\":42" ),
    "number value must be unquoted in JSON (was: value:\"42\"): {text}",
  );
}

// TC-492: settings_get string value → JSON output has quoted string
#[ test ]
fn tc492_settings_get_json_string_quoted()
{
  let dir = tempfile::TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{\"name\":\"hello\"}" ).unwrap();
  let out = run_clv_with_env(
    &[ ".settings.get", "key::name", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\"value\":\"hello\"" ),
    "string value must be quoted in JSON: {text}",
  );
}
