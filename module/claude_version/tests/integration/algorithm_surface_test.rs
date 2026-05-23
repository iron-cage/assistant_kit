//! Algorithm surface tests (AC- prefix) for `claude_version`.
//!
//! Implements test cases from `tests/docs/algorithm/001_settings_type_inference.md`
//! that are not already covered by `mutation_commands_test.rs`.
//!
//! # Coverage Map
//!
//! | AC-ID | Function |
//! |-------|----------|
//! | AC-1  | covered by `tc322_settings_set_stores_boolean_true` |
//! | AC-2  | covered by `tc323_settings_set_stores_boolean_false` |
//! | AC-3  | covered by `tc324_settings_set_zero_stored_as_number` |
//! | AC-4  | `ac004_float_inference` |
//! | AC-5  | `ac005_nan_stores_string` |
//! | AC-6  | covered by `tc326_settings_set_stores_string` |

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm_with_env };

// ─── AC-4: finite float stored as JSON float ──────────────────────────────────

// AC-4: value::3.14 → settings.json contains "pi": 3.14 (unquoted float)
#[ test ]
fn ac004_float_inference()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::pi", "value::3.14" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"pi\": 3.14" ), "3.14 must be stored as bare float: {content}" );
  assert!( !content.contains( "\"pi\": \"3.14\"" ), "3.14 must NOT be quoted: {content}" );
}

// ─── AC-5: NaN / inf strings stored as JSON string ───────────────────────────

// AC-5: value::nan → settings.json contains "bad": "nan" (quoted — not a float)
#[ test ]
fn ac005_nan_stores_string()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::bad", "value::nan" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"bad\": \"nan\"" ), "nan must be stored as quoted string: {content}" );
}
