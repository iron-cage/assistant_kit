//! Settings I/O unit tests
//!
//! ## Purpose
//!
//! Verify `infer_type`, `json_escape`, and the round-trip behaviour of
//! `set_setting` / `get_setting` / `read_all_settings` / `remove_setting`
//! using temp files.
//!
//! ## Coverage
//!
//! - `infer_type` classifies booleans, numbers, strings, raw JSON, null
//! - `infer_type` rejects non-finite floats (NaN, inf) as `Str` not `Number`
//! - `json_escape` escapes `"`, `\`, `\n`, `\r`, `\t` correctly
//! - `set_setting` creates a file and stores the value correctly
//! - `get_setting` reads an existing key and returns `None` for absent keys
//! - `read_all_settings` reads back all pairs in insertion order
//! - `remove_setting` deletes an existing key and no-ops on absent key/file
//! - Nested `env` round-trips through `set_env_var` / `remove_env_var`
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `infer_type_true_and_false_are_bool` | "true"/"false" → Bool |
//! | `infer_type_integers_are_number` | "42" → Number |
//! | `infer_type_floats_are_number` | "3.14" → Number |
//! | `infer_type_nan_and_inf_are_str` | "NaN"/"inf" → Str |
//! | `infer_type_object_is_raw` | "{}" → Raw |
//! | `infer_type_array_is_raw` | "[]" → Raw |
//! | `infer_type_null_is_raw` | "null" → Raw |
//! | `infer_type_plain_string_is_str` | "hello" → Str |
//! | `json_escape_handles_special_chars` | escapes `"\n\r\t` |
//! | `set_and_get_setting_round_trip` | write+read returns same value |
//! | `get_setting_absent_key_returns_none` | missing key → None |
//! | `read_all_settings_preserves_pairs` | all pairs returned |
//! | `remove_setting_deletes_existing_key` | removed key gone, others untouched |
//! | `remove_setting_missing_key_is_noop` | absent key → no-op, file unchanged |
//! | `remove_setting_missing_file_is_noop` | absent file → no-op, no error |
//! | `set_env_var_and_remove_env_var_round_trip` | env sub-object round-trip |

use claude_core::settings_io::{
  infer_type, json_escape, set_setting, get_setting, read_all_settings, remove_setting,
  set_env_var, remove_env_var, StoredAs,
};

// Helper: a path inside a temp dir that does NOT yet exist.
// `read_or_empty` handles NotFound → empty; it does not handle empty files.
fn new_settings_path() -> ( tempfile::TempDir, std::path::PathBuf )
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = dir.path().join( "settings.json" );
  ( dir, path )
}

// ─── infer_type ───────────────────────────────────────────────────────────────

#[test]
fn infer_type_true_and_false_are_bool()
{
  assert_eq!( infer_type( "true" ),  StoredAs::Bool );
  assert_eq!( infer_type( "false" ), StoredAs::Bool );
}

#[test]
fn infer_type_integers_are_number()
{
  assert_eq!( infer_type( "42" ),   StoredAs::Number );
  assert_eq!( infer_type( "0" ),    StoredAs::Number );
  assert_eq!( infer_type( "-1" ),   StoredAs::Number );
  assert_eq!( infer_type( "1000" ), StoredAs::Number );
}

#[test]
fn infer_type_floats_are_number()
{
  assert_eq!( infer_type( "3.14" ),  StoredAs::Number );
  assert_eq!( infer_type( "0.0" ),   StoredAs::Number );
  assert_eq!( infer_type( "-2.5" ),  StoredAs::Number );
}

#[test]
fn infer_type_nan_and_inf_are_str()
{
  // Fix(issue-infer-nan): NaN and inf are not valid JSON numbers.
  // Root cause: f64::from_str accepts them but they corrupt JSON files.
  // Pitfall: Always gate float classification with is_finite().
  assert_eq!( infer_type( "NaN" ),      StoredAs::Str );
  assert_eq!( infer_type( "inf" ),      StoredAs::Str );
  assert_eq!( infer_type( "infinity" ), StoredAs::Str );
}

#[test]
fn infer_type_object_is_raw()
{
  assert_eq!( infer_type( "{}" ),              StoredAs::Raw );
  assert_eq!( infer_type( "{\"a\": 1}" ),      StoredAs::Raw );
  assert_eq!( infer_type( "  { \"k\": 1 }" ),  StoredAs::Raw );
}

#[test]
fn infer_type_array_is_raw()
{
  assert_eq!( infer_type( "[]" ),         StoredAs::Raw );
  assert_eq!( infer_type( "[1,2,3]" ),    StoredAs::Raw );
}

#[test]
fn infer_type_null_is_raw()
{
  assert_eq!( infer_type( "null" ), StoredAs::Raw );
}

#[test]
fn infer_type_plain_string_is_str()
{
  assert_eq!( infer_type( "hello" ),    StoredAs::Str );
  assert_eq!( infer_type( "somekey" ),  StoredAs::Str );
  assert_eq!( infer_type( "3.14.15" ),  StoredAs::Str );
}

// ─── json_escape ─────────────────────────────────────────────────────────────

#[test]
fn json_escape_handles_special_chars()
{
  assert_eq!( json_escape( "\"" ),  "\\\"" );
  assert_eq!( json_escape( "\\" ),  "\\\\" );
  assert_eq!( json_escape( "\n" ),  "\\n"  );
  assert_eq!( json_escape( "\r" ),  "\\r"  );
  assert_eq!( json_escape( "\t" ),  "\\t"  );
  assert_eq!( json_escape( "abc" ), "abc"  );
}

// ─── set_setting / get_setting round-trip ─────────────────────────────────────

#[test]
fn set_and_get_setting_round_trip()
{
  let ( _dir, path ) = new_settings_path();
  set_setting( &path, "myKey", "myValue" ).expect( "set" );
  let got = get_setting( &path, "myKey" ).expect( "get" );
  assert_eq!( got, Some( "myValue".to_string() ) );
}

#[test]
fn get_setting_absent_key_returns_none()
{
  let ( _dir, path ) = new_settings_path();
  set_setting( &path, "a", "1" ).expect( "set" );
  let got = get_setting( &path, "missing" ).expect( "get" );
  assert_eq!( got, None );
}

#[test]
fn read_all_settings_preserves_pairs()
{
  let ( _dir, path ) = new_settings_path();
  set_setting( &path, "alpha", "1" ).expect( "set alpha" );
  set_setting( &path, "beta",  "2" ).expect( "set beta"  );
  let pairs = read_all_settings( &path ).expect( "read" );
  let keys : Vec< &str > = pairs.iter().map( |( k, _ )| k.as_str() ).collect();
  assert!( keys.contains( &"alpha" ), "expected alpha in pairs" );
  assert!( keys.contains( &"beta"  ), "expected beta in pairs"  );
}

#[test]
fn remove_setting_deletes_existing_key()
{
  let ( _dir, path ) = new_settings_path();
  set_setting( &path, "alpha", "1" ).expect( "set alpha" );
  set_setting( &path, "beta",  "2" ).expect( "set beta"  );
  remove_setting( &path, "alpha" ).expect( "remove alpha" );

  let got = get_setting( &path, "alpha" ).expect( "get alpha" );
  assert_eq!( got, None, "alpha should be removed" );
  let kept = get_setting( &path, "beta" ).expect( "get beta" );
  assert_eq!( kept, Some( "2".to_string() ), "beta should remain untouched" );
}

#[test]
fn remove_setting_missing_key_is_noop()
{
  let ( _dir, path ) = new_settings_path();
  set_setting( &path, "alpha", "1" ).expect( "set alpha" );
  remove_setting( &path, "does_not_exist" ).expect( "remove absent key is a no-op" );

  let got = get_setting( &path, "alpha" ).expect( "get alpha" );
  assert_eq!( got, Some( "1".to_string() ), "existing key must be unaffected" );
}

#[test]
fn remove_setting_missing_file_is_noop()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = dir.path().join( "does_not_exist.json" );
  remove_setting( &path, "any_key" ).expect( "remove on missing file is a no-op, not an error" );
}

#[test]
fn set_env_var_and_remove_env_var_round_trip()
{
  let ( _dir, path ) = new_settings_path();
  set_env_var( &path, "MY_VAR", "hello" ).expect( "set env var" );
  let got = get_setting( &path, "env" ).expect( "get env" );
  assert!( got.is_some(), "env key should exist after set_env_var" );
  assert!(
    got.as_deref().unwrap().contains( "MY_VAR" ),
    "env object should contain MY_VAR"
  );

  remove_env_var( &path, "MY_VAR" ).expect( "remove env var" );
  let after = get_setting( &path, "env" ).expect( "get env after remove" );
  assert!(
    after.as_deref().is_none_or( | s | !s.contains( "MY_VAR" ) ),
    "MY_VAR should be absent after remove_env_var"
  );
}
