//! `get_string_setting` unit tests
//!
//! ## Purpose
//!
//! Verify `get_string_setting` only returns a value when the underlying JSON
//! type is a plain string, rejecting numbers, bools, null, and nested
//! objects/arrays as "no preference" (`None`) rather than coercing them to
//! a string.
//!
//! ## Coverage
//!
//! - Plain JSON string value → `Some`
//! - JSON number, bool, null, object, array values → `None`
//! - Absent key → `None`
//! - Absent file → `Err(NotFound)`
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `get_string_setting_returns_plain_string` | `"k": "v"` → `Some("v")` |
//! | `get_string_setting_rejects_number` | `"k": 42` → `None` |
//! | `get_string_setting_rejects_bool` | `"k": true` → `None` |
//! | `get_string_setting_rejects_null` | `"k": null` → `None` |
//! | `get_string_setting_rejects_object` | `"k": {"a":1}` → `None` |
//! | `get_string_setting_rejects_array` | `"k": [1,2]` → `None` |
//! | `get_string_setting_absent_key_returns_none` | missing key → `None` |
//! | `get_string_setting_missing_file_returns_not_found` | missing file → `Err(NotFound)` |

use claude_core::settings_io::get_string_setting;

fn write_settings( dir : &std::path::Path, raw_json : &str ) -> std::path::PathBuf
{
  let path = dir.join( "prefs.json" );
  std::fs::write( &path, raw_json ).expect( "write settings file" );
  path
}

#[test]
fn get_string_setting_returns_plain_string()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": "claude-opus-4-8"}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), Some( "claude-opus-4-8".to_string() ) );
}

#[test]
fn get_string_setting_rejects_number()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": 42}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), None );
}

#[test]
fn get_string_setting_rejects_bool()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": true}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), None );
}

#[test]
fn get_string_setting_rejects_null()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": null}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), None );
}

#[test]
fn get_string_setting_rejects_object()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": {"a": 1}}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), None );
}

#[test]
fn get_string_setting_rejects_array()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": [1, 2]}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), None );
}

#[test]
fn get_string_setting_absent_key_returns_none()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"other_key": "v"}"# );
  assert_eq!( get_string_setting( &path, "subprocess_model" ).unwrap(), None );
}

#[test]
fn get_string_setting_missing_file_returns_not_found()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = dir.path().join( "does_not_exist.json" );
  let err  = get_string_setting( &path, "subprocess_model" ).unwrap_err();
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}
