//! `get_string_setting` unit tests
//!
//! ## Purpose
//!
//! Verify `get_string_setting` only returns a value when the underlying JSON
//! type is a plain string, rejecting numbers, bools, null, and nested
//! objects/arrays as "no preference" (`None`) rather than coercing them to
//! a string. Also covers the parser's malformed-value error path.
//!
//! ## Coverage
//!
//! - Plain JSON string value → `Some`
//! - JSON number, bool, null, object, array values → `None`
//! - Absent key → `None`
//! - Absent file → `Err(NotFound)`
//! - Malformed value token (unquoted, non-literal) → `Err(InvalidData)`
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
//! | `get_string_setting_malformed_value_returns_invalid_data` | `"k": undefined` → `Err(InvalidData)` |
//! | `set_setting_traces_function_and_parameters` | parameter-trace structural guard (task 313, T07): first statement is `eprintln!` naming the function, `path`, `key`, `raw_value` |
//! | `remove_setting_traces_function_and_parameters` | parameter-trace structural guard (task 313, T08): first statement is `eprintln!` naming the function, `path`, `key` |
//! | `set_env_var_traces_function_and_parameters` | parameter-trace structural guard (task 313, T09): first statement is `eprintln!` naming the function, `path`, `key`, `value` |
//! | `remove_env_var_traces_function_and_parameters` | parameter-trace structural guard (task 313, T10): first statement is `eprintln!` naming the function, `path`, `key` |

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

#[test]
fn get_string_setting_malformed_value_returns_invalid_data()
{
  let dir  = tempfile::TempDir::new().expect( "temp dir" );
  let path = write_settings( dir.path(), r#"{"subprocess_model": undefined}"# );
  let err  = get_string_setting( &path, "subprocess_model" ).unwrap_err();
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidData );
}

// ─── Parameter-trace structural guards (Task 313: T07-T10) ────────────────────

/// Extract the body of `fn {name}(...) { ... }` from `src` via brace-depth
/// counting. A naive "scan to next `pub fn`" heuristic is fragile when the
/// next function is hundreds of lines away with no other `eprintln!` between
/// — brace counting finds the exact matching close-brace instead.
fn extract_fn_body<'a>( src : &'a str, name : &str ) -> &'a str
{
  let sig        = format!( "fn {name}(" );
  let fn_start   = src.find( &sig ).unwrap_or_else( || panic!( "{name} not found in source" ) );
  let brace_start = src[ fn_start.. ].find( '{' )
    .unwrap_or_else( || panic!( "{name} body opening brace not found" ) ) + fn_start;

  let mut depth = 0usize;
  let mut end   = brace_start;
  for ( i, ch ) in src[ brace_start.. ].char_indices()
  {
    match ch
    {
      '{' => depth += 1,
      '}' =>
      {
        depth -= 1;
        if depth == 0 { end = brace_start + i; break; }
      }
      _ => {}
    }
  }
  &src[ brace_start + 1..end ]
}

#[test]
fn set_setting_traces_function_and_parameters()
{
  let src        = include_str!( "../src/settings_io.rs" );
  let body       = extract_fn_body( src, "set_setting" );
  let first_stmt = body.trim_start().split( ';' ).next().unwrap().trim();

  assert!(
    first_stmt.starts_with( "eprintln!" ),
    "set_setting must emit eprintln! as its first statement, got: {first_stmt:?}"
  );
  assert!(
    first_stmt.contains( "set_setting" ) && first_stmt.contains( "path" )
      && first_stmt.contains( "key" ) && first_stmt.contains( "raw_value" ),
    "trace line must name the function and all 3 parameters (path, key, raw_value): {first_stmt:?}"
  );
  assert_eq!(
    body.matches( "eprintln!" ).count(), 1,
    "set_setting must have exactly one eprintln! call, found {}", body.matches( "eprintln!" ).count()
  );
}

#[test]
fn remove_setting_traces_function_and_parameters()
{
  let src        = include_str!( "../src/settings_io.rs" );
  let body       = extract_fn_body( src, "remove_setting" );
  let first_stmt = body.trim_start().split( ';' ).next().unwrap().trim();

  assert!(
    first_stmt.starts_with( "eprintln!" ),
    "remove_setting must emit eprintln! as its first statement, got: {first_stmt:?}"
  );
  assert!(
    first_stmt.contains( "remove_setting" ) && first_stmt.contains( "path" ) && first_stmt.contains( "key" ),
    "trace line must name the function and both parameters (path, key): {first_stmt:?}"
  );
  assert_eq!(
    body.matches( "eprintln!" ).count(), 1,
    "remove_setting must have exactly one eprintln! call, found {}", body.matches( "eprintln!" ).count()
  );
}

#[test]
fn set_env_var_traces_function_and_parameters()
{
  let src        = include_str!( "../src/settings_io.rs" );
  let body       = extract_fn_body( src, "set_env_var" );
  let first_stmt = body.trim_start().split( ';' ).next().unwrap().trim();

  assert!(
    first_stmt.starts_with( "eprintln!" ),
    "set_env_var must emit eprintln! as its first statement, got: {first_stmt:?}"
  );
  assert!(
    first_stmt.contains( "set_env_var" ) && first_stmt.contains( "path" )
      && first_stmt.contains( "key" ) && first_stmt.contains( "value" ),
    "trace line must name the function and all 3 parameters (path, key, value): {first_stmt:?}"
  );
  assert_eq!(
    body.matches( "eprintln!" ).count(), 1,
    "set_env_var must have exactly one eprintln! call, found {}", body.matches( "eprintln!" ).count()
  );
}

#[test]
fn remove_env_var_traces_function_and_parameters()
{
  let src        = include_str!( "../src/settings_io.rs" );
  let body       = extract_fn_body( src, "remove_env_var" );
  let first_stmt = body.trim_start().split( ';' ).next().unwrap().trim();

  assert!(
    first_stmt.starts_with( "eprintln!" ),
    "remove_env_var must emit eprintln! as its first statement, got: {first_stmt:?}"
  );
  assert!(
    first_stmt.contains( "remove_env_var" ) && first_stmt.contains( "path" ) && first_stmt.contains( "key" ),
    "trace line must name the function and both parameters (path, key): {first_stmt:?}"
  );
  assert_eq!(
    body.matches( "eprintln!" ).count(), 1,
    "remove_env_var must have exactly one eprintln! call, found {}", body.matches( "eprintln!" ).count()
  );
}
