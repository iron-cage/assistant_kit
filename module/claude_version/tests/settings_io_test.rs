//! Settings I/O unit tests.
//!
//! # Test Matrix
//!
//! | TC | Description | F/L | P/N |
//! |----|-------------|-----|-----|
//! | TC-027 | `infer_type("true")` → `Bool` | F10/L2 | P |
//! | TC-028 | `infer_type("false")` → `Bool` | F10/L3 | P |
//! | TC-029 | `infer_type("0")` → `Number` (not Bool) | F10/L4 | P |
//! | TC-030 | `infer_type("1")` → `Number` (not Bool) | F10/L5 | P |
//! | TC-031 | `infer_type("42")` → `Number` | F10/L6 | P |
//! | TC-032 | `infer_type("3.14")` → `Number` | F10/L7 | P |
//! | TC-033 | `infer_type("hello")` → `Str` | F10/L8 | P |
//! | TC-034 | `infer_type("")` → `Str` | F10/L9 | P |
//! | TC-035 | `infer_type("TRUE")` → `Str` (case-sensitive) | F10/L12 | P |
//! | TC-036 | `infer_type("False")` → `Str` (case-sensitive) | F10/L12 | P |
//! | TC-037 | `infer_type(" ")` → `Str` (whitespace-only) | F10 special | P |
//! | TC-038 | `infer_type("-1")` → `Number` | F10 boundary | P |
//! | TC-039 | `set_setting` creates file when file is absent | F13/L2 | P |
//! | TC-040 | `set_setting` adds new key to existing file | F13/L1 new | P |
//! | TC-041 | `set_setting` overwrites existing key | F13/L1 exist | P |
//! | TC-042 | `set_setting bool "true"` → JSON `true` (no quotes) | F10/L2 | P |
//! | TC-043 | `set_setting number "42"` → JSON `42` (no quotes) | F10/L6 | P |
//! | TC-044 | `set_setting number "0"` → JSON `0` (not `false`) | F10/L4 | P |
//! | TC-045 | `set_setting empty string` → JSON `""` | F10/L9 | P |
//! | TC-046 | `set_setting value with '"'` → JSON-escaped | F10/L11 | P |
//! | TC-047 | `set_setting value with '\'` → JSON-escaped | F10/L11 | P |
//! | TC-048 | `set_setting key with dot notation` → stored as literal | F9/L5 | P |
//! | TC-049 | `set_setting` atomic write: temp file removed, final file correct | F21/L1 | P |
//! | TC-050 | `set_setting` on read-only dir → error | F21/L2 | N |
//! | TC-051 | `get_setting` returns value for existing key | F9/L2 | P |
//! | TC-052 | `get_setting` returns `None` for missing key | F9/L3 | P |
//! | TC-053 | `get_setting` on missing file → `Err(NotFound)` | F13/L2 | N |
//! | TC-054 | `read_all_settings` empty `{}` → empty vec | F13/L3 | P |
//! | TC-055 | `read_all_settings` valid file → all key-value pairs | F13/L1 | P |
//! | TC-056 | `read_all_settings` missing file → `Err(NotFound)` | F13/L2 | N |
//! | TC-057 | `read_all_settings` malformed JSON → `Err` | F13/L4 | N |
//! | TC-058 | `set_setting` roundtrip: set then get returns same value | F10 all | P |
//! | TC-059 | `set_setting` preserves existing keys when adding new | F13/L1 | P |
//! | TC-060 | `set_setting` value with newline → stored safely | F10 special | P |
//! | TC-061 | `infer_type("NaN")` → `Str` (non-finite float) | F10/bug | P |
//! | TC-062 | `infer_type("inf")` → `Str` | F10/bug | P |
//! | TC-063 | `infer_type("infinity")` → `Str` | F10/bug | P |
//! | TC-064 | `infer_type("-inf")` → `Str` | F10/bug | P |
//! | TC-065 | Non-finite case variants → `Str` | F10/bug | P |
//! | TC-066 | `set_setting("NaN")` roundtrip produces valid JSON | F10/bug | P |
//! | TC-067 | `set_setting("inf")` roundtrip produces valid JSON | F10/bug | P |
//! | TC-068 | `set_env_var` overwrites existing env var value | env | P |
//! | TC-069 | `set_env_var` creates file when absent | env | P |
//! | TC-070 | `remove_env_var` removes key from env block | env | P |
//! | TC-071 | `remove_env_var` no-op when key absent | env | P |
//! | TC-072 | `remove_env_var` no-op when file absent | env | P |
//! | TC-073 | real-world settings.json with nested objects round-trips correctly | F21 | P |
//! | TC-074 | `infer_type("null")` → `Raw` (bare JSON null, not string) | F10/null | P |
//! | TC-075 | null value survives `set_setting` round-trip as bare `null` | F10/null | P |
//! | TC-076 | `read_all_settings` preserves nested objects verbatim | F15a | P |
//! | TC-077 | `set_setting` preserves nested `env` object during round-trip | F15a | P |
//! | TC-078 | `set_setting` preserves `enabledPlugins` nested object | F15a | P |
//! | TC-079 | `infer_type` detects raw JSON objects → `Raw` | F10 | P |
//! | TC-080 | `infer_type` detects raw JSON arrays → `Raw` | F10 | P |
//! | TC-081 | `set_env_var` creates `env` block when none exists | env | P |
//! | TC-082 | `set_env_var` updates existing `env` block, preserves other vars | env | P |

use std::fs;
use std::io::ErrorKind;

use claude_version::settings_io::{ infer_type, StoredAs, get_setting, set_setting, set_env_var, remove_env_var, read_all_settings };
use tempfile::TempDir;

// ─── infer_type ─────────────────────────────────────────────────────────────

// TC-027
#[ test ]
fn tc027_true_inferred_as_bool()
{
  assert_eq!( infer_type( "true" ), StoredAs::Bool );
}

// TC-028
#[ test ]
fn tc028_false_inferred_as_bool()
{
  assert_eq!( infer_type( "false" ), StoredAs::Bool );
}

// TC-029: "0" must be Number, not Bool
#[ test ]
fn tc029_zero_inferred_as_number_not_bool()
{
  assert_eq!( infer_type( "0" ), StoredAs::Number );
}

// TC-030: "1" must be Number, not Bool
#[ test ]
fn tc030_one_inferred_as_number_not_bool()
{
  assert_eq!( infer_type( "1" ), StoredAs::Number );
}

// TC-031
#[ test ]
fn tc031_integer_inferred_as_number()
{
  assert_eq!( infer_type( "42" ), StoredAs::Number );
}

// TC-032
#[ test ]
fn tc032_float_inferred_as_number()
{
  assert_eq!( infer_type( "3.14" ), StoredAs::Number );
}

// TC-033
#[ test ]
fn tc033_plain_string_inferred_as_str()
{
  assert_eq!( infer_type( "hello" ), StoredAs::Str );
}

// TC-034
#[ test ]
fn tc034_empty_string_inferred_as_str()
{
  assert_eq!( infer_type( "" ), StoredAs::Str );
}

// TC-035
#[ test ]
fn tc035_uppercase_true_inferred_as_str()
{
  assert_eq!( infer_type( "TRUE" ), StoredAs::Str );
}

// TC-036
#[ test ]
fn tc036_mixed_case_false_inferred_as_str()
{
  assert_eq!( infer_type( "False" ), StoredAs::Str );
}

// TC-037
#[ test ]
fn tc037_whitespace_only_inferred_as_str()
{
  assert_eq!( infer_type( " " ), StoredAs::Str );
}

// TC-038
#[ test ]
fn tc038_negative_integer_inferred_as_number()
{
  assert_eq!( infer_type( "-1" ), StoredAs::Number );
}

// TC-061: "NaN" must be Str (non-finite float → not valid JSON number)
//
// ## Root Cause
//
// `infer_type` used `f64::from_str` without checking `is_finite()`.
// Rust parses `"NaN"`, `"inf"`, `"infinity"` (and case variants) as valid
// `f64` values, but these are NOT valid JSON number literals.  Writing them
// as bare values (e.g., `"key": NaN`) produces invalid JSON that cannot be
// read back, corrupting the settings file.
//
// ## Why Not Caught
//
// Existing tests only covered well-behaved inputs ("42", "3.14", "-1").
// No test exercised non-finite float parsing.
//
// ## Fix Applied
//
// Added `is_finite()` guard: `f64::from_str(s).map_or(false, |n| n.is_finite())`.
//
// ## Prevention
//
// This test covers all non-finite float variants accepted by Rust's parser.
//
// ## Pitfall
//
// `f64::from_str` silently accepts locale-agnostic special values that are
// not valid in JSON, YAML, or TOML.  Always gate with `is_finite()` before
// using parsed floats in serialization contexts.
#[ test ]
fn tc061_nan_inferred_as_str()
{
  assert_eq!( infer_type( "NaN" ), StoredAs::Str );
}

// TC-062: "inf" must be Str
#[ test ]
fn tc062_inf_inferred_as_str()
{
  assert_eq!( infer_type( "inf" ), StoredAs::Str );
}

// TC-063: "infinity" must be Str
#[ test ]
fn tc063_infinity_inferred_as_str()
{
  assert_eq!( infer_type( "infinity" ), StoredAs::Str );
}

// TC-064: "-inf" must be Str
#[ test ]
fn tc064_negative_inf_inferred_as_str()
{
  assert_eq!( infer_type( "-inf" ), StoredAs::Str );
}

// TC-065: case variants of non-finite floats must be Str
#[ test ]
fn tc065_non_finite_case_variants_inferred_as_str()
{
  for val in &[ "nan", "NAN", "Inf", "INF", "Infinity", "INFINITY", "-Infinity", "+inf" ]
  {
    assert_eq!(
      infer_type( val ),
      StoredAs::Str,
      "'{val}' must be Str, not Number"
    );
  }
}

// TC-066: set_setting with NaN must produce readable JSON (roundtrip)
#[ test ]
fn tc066_set_nan_produces_valid_json_roundtrip()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "x", "NaN" ).unwrap();

  // File must be readable without error
  let pairs = read_all_settings( &path ).unwrap();
  assert_eq!( pairs.len(), 1 );
  assert_eq!( pairs[ 0 ].0, "x" );
  assert_eq!( pairs[ 0 ].1, "NaN" );
}

// TC-067: set_setting with inf must produce readable JSON (roundtrip)
#[ test ]
fn tc067_set_inf_produces_valid_json_roundtrip()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "x", "inf" ).unwrap();

  let pairs = read_all_settings( &path ).unwrap();
  assert_eq!( pairs.len(), 1 );
  assert_eq!( pairs[ 0 ].0, "x" );
  assert_eq!( pairs[ 0 ].1, "inf" );
}

// ─── set_setting ────────────────────────────────────────────────────────────

// TC-039
#[ test ]
fn tc039_set_setting_creates_file_when_absent()
{
  let dir = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  assert!( !path.exists() );
  set_setting( &path, "key1", "val1" ).unwrap();
  assert!( path.exists(), "settings.json must be created" );
}

// TC-040
#[ test ]
fn tc040_set_setting_adds_new_key_to_existing_file()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "a", "1" ).unwrap();
  set_setting( &path, "b", "2" ).unwrap();
  let v = get_setting( &path, "a" ).unwrap();
  assert_eq!( v.as_deref(), Some( "1" ) );
  let v2 = get_setting( &path, "b" ).unwrap();
  assert_eq!( v2.as_deref(), Some( "2" ) );
}

// TC-041
#[ test ]
fn tc041_set_setting_overwrites_existing_key()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "k", "old" ).unwrap();
  set_setting( &path, "k", "new" ).unwrap();
  let v = get_setting( &path, "k" ).unwrap();
  assert_eq!( v.as_deref(), Some( "new" ) );
}

// TC-042: bool "true" → JSON `true` (no quotes)
#[ test ]
fn tc042_bool_true_written_without_quotes()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "flag", "true" ).unwrap();
  let raw = fs::read_to_string( &path ).unwrap();
  assert!( raw.contains( "true" ), "JSON must contain bare true" );
  assert!( !raw.contains( "\"true\"" ), "JSON must not quote boolean" );
}

// TC-043: number "42" → JSON `42` (no quotes)
#[ test ]
fn tc043_integer_written_without_quotes()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "count", "42" ).unwrap();
  let raw = fs::read_to_string( &path ).unwrap();
  assert!( raw.contains( "42" ), "JSON must contain bare 42" );
  assert!( !raw.contains( "\"42\"" ), "JSON must not quote integer" );
}

// TC-044: "0" → JSON `0` (not `false`)
#[ test ]
fn tc044_zero_written_as_number_not_false()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "n", "0" ).unwrap();
  let raw = fs::read_to_string( &path ).unwrap();
  assert!( raw.contains( ": 0" ) || raw.contains( ":0" ), "JSON must contain 0 as number" );
  assert!( !raw.contains( "false" ), "0 must not be written as false" );
}

// TC-045: empty string → JSON `""`
#[ test ]
fn tc045_empty_string_written_as_empty_json_string()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "s", "" ).unwrap();
  let raw = fs::read_to_string( &path ).unwrap();
  assert!( raw.contains( "\"\"" ), "empty string must appear as \"\" in JSON" );
}

// TC-046: value with double-quote → JSON-escaped
#[ test ]
fn tc046_value_with_double_quote_escaped()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "s", r#"say "hi""# ).unwrap();
  let v = get_setting( &path, "s" ).unwrap();
  assert_eq!( v.as_deref(), Some( r#"say "hi""# ) );
}

// TC-047: value with backslash → JSON-escaped
#[ test ]
fn tc047_value_with_backslash_escaped()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "s", r"back\slash" ).unwrap();
  let v = get_setting( &path, "s" ).unwrap();
  assert_eq!( v.as_deref(), Some( r"back\slash" ) );
}

// TC-048: key with dot notation stored as literal key
#[ test ]
fn tc048_key_with_dot_stored_as_literal()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "foo.bar", "val" ).unwrap();
  let v = get_setting( &path, "foo.bar" ).unwrap();
  assert_eq!( v.as_deref(), Some( "val" ) );
}

// TC-049: atomic write leaves no .tmp file
#[ test ]
fn tc049_atomic_write_leaves_no_tmp_file()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "k", "v" ).unwrap();
  let tmp = dir.path().join( "settings.json.tmp" );
  assert!( !tmp.exists(), "temp file must not remain after successful write" );
}

// TC-050: write to read-only directory → error
// Fix(issue-108): set_mode(0o555) via PermissionsExt is Unix-only.
#[ cfg( unix ) ]
#[ test ]
fn tc050_write_to_read_only_dir_returns_error()
{
  use std::os::unix::fs::PermissionsExt;
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  // Make directory read-only
  let mut perms = fs::metadata( dir.path() ).unwrap().permissions();
  perms.set_mode( 0o555 );
  fs::set_permissions( dir.path(), perms ).unwrap();
  let result = set_setting( &path, "k", "v" );
  // Restore permissions so TempDir can clean up
  let mut perms2 = fs::metadata( dir.path() ).unwrap().permissions();
  perms2.set_mode( 0o755 );
  fs::set_permissions( dir.path(), perms2 ).unwrap();
  assert!( result.is_err(), "write to read-only dir must fail" );
}

// ─── get_setting ────────────────────────────────────────────────────────────

// TC-051
#[ test ]
fn tc051_get_setting_returns_value_for_existing_key()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "mykey", "myval" ).unwrap();
  let v = get_setting( &path, "mykey" ).unwrap();
  assert_eq!( v.as_deref(), Some( "myval" ) );
}

// TC-052
#[ test ]
fn tc052_get_setting_returns_none_for_missing_key()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "existing", "1" ).unwrap();
  let v = get_setting( &path, "nonexistent" ).unwrap();
  assert_eq!( v, None );
}

// TC-053
#[ test ]
fn tc053_get_setting_on_missing_file_returns_not_found()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "no_such_file.json" );
  let result = get_setting( &path, "k" );
  assert!( result.is_err() );
  assert_eq!(
    result.unwrap_err().kind(),
    ErrorKind::NotFound,
    "missing file must return NotFound error"
  );
}

// ─── read_all_settings ──────────────────────────────────────────────────────

// TC-054
#[ test ]
fn tc054_read_all_settings_empty_object_returns_empty_vec()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, "{}" ).unwrap();
  let pairs = read_all_settings( &path ).unwrap();
  assert!( pairs.is_empty(), "empty JSON object must produce empty vec" );
}

// TC-055
#[ test ]
fn tc055_read_all_settings_returns_all_key_value_pairs()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"a": "hello", "b": "world"}"# ).unwrap();
  let pairs = read_all_settings( &path ).unwrap();
  assert_eq!( pairs.len(), 2 );
  let map : std::collections::HashMap< _, _ > = pairs.into_iter().collect();
  assert_eq!( map.get( "a" ).map( String::as_str ), Some( "hello" ) );
  assert_eq!( map.get( "b" ).map( String::as_str ), Some( "world" ) );
}

// TC-056
#[ test ]
fn tc056_read_all_settings_missing_file_returns_not_found()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "no_such.json" );
  let result = read_all_settings( &path );
  assert!( result.is_err() );
  assert_eq!( result.unwrap_err().kind(), ErrorKind::NotFound );
}

// TC-057
#[ test ]
fn tc057_read_all_settings_malformed_json_returns_error()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, "{ not valid json" ).unwrap();
  let result = read_all_settings( &path );
  assert!( result.is_err(), "malformed JSON must return error" );
}

// ─── roundtrip & preservation ───────────────────────────────────────────────

// TC-058: roundtrip
#[ test ]
fn tc058_set_then_get_roundtrip()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );

  for ( key, val ) in &[ ( "k1", "hello" ), ( "k2", "42" ), ( "k3", "true" ), ( "k4", "" ) ]
  {
    set_setting( &path, key, val ).unwrap();
    let v = get_setting( &path, key ).unwrap();
    assert_eq!(
      v.as_deref(),
      Some( *val ),
      "roundtrip failed for key={key} val={val}"
    );
  }
}

// TC-059: preserves existing keys
#[ test ]
fn tc059_set_new_key_preserves_existing_keys()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "existing", "alpha" ).unwrap();
  set_setting( &path, "new_key",  "beta"  ).unwrap();
  let v = get_setting( &path, "existing" ).unwrap();
  assert_eq!( v.as_deref(), Some( "alpha" ), "existing key must be preserved" );
}

// TC-060: value with newline stored safely
#[ test ]
fn tc060_value_with_newline_stored_safely()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_setting( &path, "s", "line1\nline2" ).unwrap();
  let v = get_setting( &path, "s" ).unwrap();
  assert_eq!( v.as_deref(), Some( "line1\nline2" ) );
}

// ─── Nested JSON round-trip ─────────────────────────────────────────────────

// TC-076: read_all_settings preserves nested objects
#[ test ]
fn tc076_read_all_settings_preserves_nested_objects()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"env": {"TZ": "Europe/Kyiv"}, "autoUpdates": false}"# ).unwrap();
  let pairs = read_all_settings( &path ).unwrap();
  assert_eq!( pairs.len(), 2 );
  assert_eq!( pairs[ 1 ].0, "autoUpdates" );
  assert_eq!( pairs[ 1 ].1, "false" );
  // env value should be the raw nested object
  assert_eq!( pairs[ 0 ].0, "env" );
  assert!( pairs[ 0 ].1.contains( "TZ" ), "env must contain TZ: {}", pairs[ 0 ].1 );
}

// TC-077: set_setting preserves nested objects during round-trip
#[ test ]
fn tc077_set_setting_preserves_nested_objects()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"env": {"TZ": "Europe/Kyiv"}, "autoUpdates": false}"# ).unwrap();
  set_setting( &path, "autoUpdates", "true" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "TZ" ), "env block must survive round-trip: {content}" );
  assert!( content.contains( "true" ), "autoUpdates must be updated: {content}" );
}

// TC-078: set_setting preserves enabledPlugins nested object
#[ test ]
fn tc078_set_setting_preserves_plugins_object()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"enabledPlugins": {"foo@bar": true}, "key1": "val1"}"# ).unwrap();
  set_setting( &path, "key1", "val2" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "enabledPlugins" ), "plugins must survive: {content}" );
  assert!( content.contains( "foo@bar" ), "plugin entry must survive: {content}" );
  assert!( content.contains( "val2" ), "key1 must be updated: {content}" );
}

// TC-079: infer_type detects raw JSON objects
#[ test ]
fn tc079_infer_type_detects_raw_object()
{
  assert_eq!( infer_type( r#"{"key": "value"}"# ), StoredAs::Raw );
}

// TC-080: infer_type detects raw JSON arrays
#[ test ]
fn tc080_infer_type_detects_raw_array()
{
  assert_eq!( infer_type( "[1, 2, 3]" ), StoredAs::Raw );
}

// ─── set_env_var / remove_env_var ───────────────────────────────────────────

// TC-081: set_env_var creates env block when absent
#[ test ]
fn tc081_set_env_var_creates_env_block()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"autoUpdates": false}"# ).unwrap();
  set_env_var( &path, "DISABLE_AUTOUPDATER", "1" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "DISABLE_AUTOUPDATER" ), "must contain env var: {content}" );
  assert!( content.contains( "\"1\"" ), "value must be quoted string: {content}" );
  assert!( content.contains( "autoUpdates" ), "existing keys preserved: {content}" );
}

// TC-082: set_env_var updates existing env block
#[ test ]
fn tc082_set_env_var_updates_existing_env()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"env": {"TZ": "Europe/Kyiv"}, "autoUpdates": false}"# ).unwrap();
  set_env_var( &path, "DISABLE_AUTOUPDATER", "1" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "TZ" ), "existing env vars preserved: {content}" );
  assert!( content.contains( "DISABLE_AUTOUPDATER" ), "new env var added: {content}" );
}

// TC-068: set_env_var overwrites existing env var
#[ test ]
fn tc068_set_env_var_overwrites_existing()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"env": {"DISABLE_AUTOUPDATER": "0"}}"# ).unwrap();
  set_env_var( &path, "DISABLE_AUTOUPDATER", "1" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "\"1\"" ), "value must be updated to 1: {content}" );
}

// TC-069: set_env_var creates file when absent
#[ test ]
fn tc069_set_env_var_creates_file()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  set_env_var( &path, "DISABLE_AUTOUPDATER", "1" ).unwrap();
  assert!( path.exists(), "file must be created" );
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "DISABLE_AUTOUPDATER" ), "must contain var: {content}" );
}

// TC-070: remove_env_var removes key from env block
#[ test ]
fn tc070_remove_env_var_removes_key()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"env": {"TZ": "UTC", "DISABLE_AUTOUPDATER": "1"}}"# ).unwrap();
  remove_env_var( &path, "DISABLE_AUTOUPDATER" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( !content.contains( "DISABLE_AUTOUPDATER" ), "var must be removed: {content}" );
  assert!( content.contains( "TZ" ), "other env vars preserved: {content}" );
}

// TC-071: remove_env_var no-op when key absent
#[ test ]
fn tc071_remove_env_var_noop_when_absent()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"env": {"TZ": "UTC"}}"# ).unwrap();
  remove_env_var( &path, "NONEXISTENT" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "TZ" ), "original content preserved: {content}" );
}

// TC-072: remove_env_var no-op when file absent
#[ test ]
fn tc072_remove_env_var_noop_when_file_absent()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  assert!( remove_env_var( &path, "FOO" ).is_ok() );
}

// TC-074: infer_type("null") → Raw (bare null, not quoted "null")
//
// Root Cause: infer_type had no special case for "null" — it fell through
// to Str, causing null → "null" corruption during set_setting round-trip.
//
// Why Not Caught: No test covered null values in settings.json.
//
// Fix Applied: Added "null" match arm in infer_type returning StoredAs::Raw.
//
// Prevention: Always test JSON primitive types including null in type inference.
//
// Pitfall: JSON has four non-string primitive types (true, false, null, number).
// Forgetting any one causes silent data corruption on round-trip.
#[ test ]
fn tc074_infer_type_null_is_raw()
{
  assert_eq!( infer_type( "null" ), StoredAs::Raw );
}

// TC-075: null value survives set_setting round-trip
//
// test_kind: bug_reproducer(null-roundtrip)
//
// Root Cause: null values were stringified to "null" during round-trip because
// infer_type returned Str, causing json_serialize_flat_object to quote them.
//
// Why Not Caught: Existing round-trip tests only covered strings, numbers, and bools.
//
// Fix Applied: infer_type now returns Raw for "null", so it passes through unquoted.
//
// Prevention: Include null in round-trip test matrix alongside other JSON primitives.
//
// Pitfall: Any JSON value type not handled by infer_type gets silently quoted.
#[ test ]
fn tc075_null_value_survives_roundtrip()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  fs::write( &path, r#"{"existing": null, "other": "hello"}"# ).unwrap();
  // Modify a different key
  set_setting( &path, "other", "world" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  // null must remain as bare null, not become "null"
  assert!(
    content.contains( ": null" ) || content.contains( ":null" ),
    "null must survive as bare null, not become \"null\": {content}"
  );
  assert!(
    !content.contains( "\"null\"" ),
    "null must not be quoted: {content}"
  );
}

// TC-073: real-world settings.json with nested objects round-trips correctly
#[ test ]
fn tc073_real_world_settings_roundtrip()
{
  let dir  = TempDir::new().unwrap();
  let path = dir.path().join( "settings.json" );
  let real_json = r#"{
  "env": {
    "TZ": "Europe/Kyiv"
  },
  "enabledPlugins": {
    "rust-analyzer-lsp@claude-plugins-official": true
  },
  "skipDangerousModePermissionPrompt": true,
  "autoUpdates": false
}"#;
  fs::write( &path, real_json ).unwrap();
  // Modify one key and verify the rest survives
  set_setting( &path, "autoUpdates", "true" ).unwrap();
  let content = fs::read_to_string( &path ).unwrap();
  assert!( content.contains( "TZ" ), "env.TZ must survive: {content}" );
  assert!( content.contains( "enabledPlugins" ), "plugins must survive: {content}" );
  assert!( content.contains( "skipDangerousModePermissionPrompt" ), "skip must survive: {content}" );
}
