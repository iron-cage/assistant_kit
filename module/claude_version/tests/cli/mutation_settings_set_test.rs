//! Integration tests for `.settings.set` — E10.
//!
//! Also covers `SettingsValue` type inference surface tests.
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 320 | no `key::` → exit 1 | N | 1 |
//! | 321 | `key::` but no `value::` → exit 1 | N | 1 |
//! | 322 | `value::true` → stores boolean `true` | P | 0 |
//! | 323 | `value::false` → stores boolean `false` | P | 0 |
//! | 324 | `value::0` → stores number `0` (NOT boolean) | P | 0 |
//! | 325 | `value::42` → stores number `42` | P | 0 |
//! | 326 | `value::hello` → stores quoted `"hello"` | P | 0 |
//! | 327 | `value::` (empty) → exit 1, error mentions "value" | N | 1 |
//! | 328 | creates file when absent | P | 0 |
//! | 329 | updates existing key (no duplication) | P | 0 |
//! | 330 | `dry::1` → shows preview, no file change | P | 0 |
//! | 331 | HOME not set → exit 2 | N | 2 |
//! | 332 | `key::""` (empty key) → exit 1 | N | 1 |
//! | 333 | adds new key to existing file | P | 0 |
//! | 334 | `dry::1` + `value::` empty → exit 1 (validation before dry-run) | N | 1 |
//! | IT-4 | `dry::2` → exit 1, out-of-range boolean | N | 1 |
//! | IT-5 | `bogus::x` → exit 1, unknown param | N | 1 |
//! | IT-6 | `key::foo` without `value::` → exit 1, value required | N | 1 |
//!
//! # Lesson Learned
//!
//! **`write_settings()` helper writes all values as quoted strings** (e.g., `"true"`
//! not `true`). Tests that verify JSON type preservation after `settings set` must
//! re-read the actual file written by the command, not the helper's output.

use tempfile::TempDir;

use crate::subprocess_helpers::{
  assert_exit, run_clv, run_clv_with_env, stderr, stdout, write_settings,
};

// ─── E10: settings set ───────────────────────────────────────────────────────

// TC-320: no key:: → exit 1
#[ test ]
fn tc320_settings_set_missing_key_exits_1()
{
  let out = run_clv( &[ ".settings.set" ] );
  assert_exit( &out, 1 );
}

// TC-321: key:: but no value:: → exit 1
#[ test ]
fn tc321_settings_set_missing_value_exits_1()
{
  let out = run_clv( &[ ".settings.set", "key::foo" ] );
  assert_exit( &out, 1 );
}

// TC-322: value::true → stores boolean true
#[ test ]
fn tc322_settings_set_stores_boolean_true()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::myBool", "value::true" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"myBool\": true" ), "must store bare true: {content}" );
  assert!( !content.contains( "\"myBool\": \"true\"" ), "must NOT quote true: {content}" );
}

// TC-323: value::false → stores boolean false
#[ test ]
fn tc323_settings_set_stores_boolean_false()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::myBool", "value::false" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"myBool\": false" ), "must store bare false: {content}" );
}

// TC-324: value::0 → stores number 0 (NOT boolean false)
#[ test ]
fn tc324_settings_set_zero_stored_as_number()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::testkey", "value::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"testkey\": 0" ), "0 must be stored as number: {content}" );
  assert!( !content.contains( "false" ), "0 must NOT be stored as false: {content}" );
}

// TC-325: value::42 → stores number 42
#[ test ]
fn tc325_settings_set_stores_number()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::num", "value::42" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"num\": 42" ), "must store bare 42: {content}" );
}

// TC-326: value::hello → stores quoted "hello"
#[ test ]
fn tc326_settings_set_stores_string()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::str", "value::hello" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"str\": \"hello\"" ), "must store quoted hello: {content}" );
}

// TC-327: value:: (empty) → exit 1, error must mention "value"
//
// Root Cause
//
// `settings_set_routine` used `require_string_arg` (which allows empty strings)
// for the `value::` parameter instead of `require_nonempty_string_arg`.  The
// FR-04 "empty value → exit 1" rule was silently bypassed: `value::` wrote `""`
// into settings.json and exited 0.
//
// Why Not Caught
//
// TC-327 was originally written as a POSITIVE test ("stores empty string `""`"),
// which codified the buggy behavior.  No test verified that empty `value::` is
// rejected.
//
// Fix Applied
//
// Changed `require_string_arg` to `require_nonempty_string_arg` for `value::` in
// `settings_set_routine`, and removed the now-unused `require_string_arg` helper.
//
// Prevention
//
// This TC-327 now locks down that `value::` (empty) is rejected with exit 1 and
// an error message that mentions the parameter name.  TC-334 covers the dry::1
// case to ensure validation precedes the dry-run short-circuit.
//
// Pitfall
//
// Without this guard, `clv .settings.set key::k value::` appears to succeed but
// writes a meaningless `""` entry — indistinguishable from "not set" via
// `.settings.get`, silently masking the user typo.
#[ test ]
fn tc327_settings_set_empty_value_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::empty", "value::" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value" ), "error must mention 'value': {err}" );
  // File must NOT be created — no side effects on error
  assert!(
    !dir.path().join( ".claude/settings.json" ).exists(),
    "settings.json must not be created on empty-value rejection"
  );
}

// TC-328: creates file when absent
#[ test ]
fn tc328_settings_set_creates_file_when_absent()
{
  let dir = TempDir::new().unwrap();
  let settings_path = dir.path().join( ".claude/settings.json" );
  assert!( !settings_path.exists(), "precondition: settings file must not exist" );
  let out = run_clv_with_env(
    &[ ".settings.set", "key::newkey", "value::newval" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( settings_path.exists(), "settings file must be created" );
}

// TC-329: updates existing key (no duplication)
#[ test ]
fn tc329_settings_set_updates_existing_key()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "mykey", "old" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.set", "key::mykey", "value::new" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"mykey\": \"new\"" ), "must update to new value: {content}" );
  assert_eq!(
    content.matches( "mykey" ).count(), 1,
    "key must appear exactly once (no duplication): {content}"
  );
}

// TC-330: dry::1 → shows preview, no file change
#[ test ]
fn tc330_settings_set_dry_shows_preview_no_write()
{
  let dir = TempDir::new().unwrap();
  let settings_path = dir.path().join( ".claude/settings.json" );
  let out = run_clv_with_env(
    &[ ".settings.set", "key::k", "value::v", "dry::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must contain [dry-run]: {text}" );
  assert!( !settings_path.exists(), "dry-run must not create file" );
}

// TC-331: HOME not set → exit 2
#[ test ]
fn tc331_settings_set_no_home_exits_2()
{
  let out = run_clv_with_env(
    &[ ".settings.set", "key::k", "value::v" ],
    &[ ( "HOME", "" ) ],
  );
  assert_exit( &out, 2 );
}

// TC-332: key::"" (empty key) → exit 1
#[ test ]
fn tc332_settings_set_empty_key_exits_1()
{
  let out = run_clv( &[ ".settings.set", "key::", "value::v" ] );
  assert_exit( &out, 1 );
}

// TC-333: adds new key to existing file preserving existing keys
#[ test ]
fn tc333_settings_set_adds_new_key_preserves_existing()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "existing", "val" ) ] );
  let out = run_clv_with_env(
    &[ ".settings.set", "key::added", "value::new" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "existing" ), "must preserve existing key: {content}" );
  assert!( content.contains( "added" ), "must contain new key: {content}" );
}

// TC-334: dry::1 + value:: empty → exit 1 (validation before dry-run)
//
// Ensures that the empty-value check (FR-04) is evaluated BEFORE the dry-run
// short-circuit inside `settings_set_routine`.  Without the fix, the dry-run
// branch was reached first and printed "[dry-run] would set k =  (string)"
// with exit 0 — making the user believe the command was valid.
#[ test ]
fn tc334_settings_set_empty_value_with_dry_still_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::k", "value::", "dry::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value" ), "error must mention 'value': {err}" );
  // No file created, no dry-run output — validation fires before dry-run
  assert!(
    !dir.path().join( ".claude/settings.json" ).exists(),
    "settings.json must not be created on empty-value rejection"
  );
  assert!(
    !stdout( &out ).contains( "[dry-run]" ),
    "dry-run output must not appear when value:: is empty"
  );
}

// ─── Type Surface: SettingsValue (inference) ─────────────────────────────────

// Type test: value::true → JSON boolean true (not quoted string)
#[ test ]
fn tc_settings_value_bool_true_inferred()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::flag", "value::true" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"flag\": true" ), "must infer bool true: {content}" );
  assert!( !content.contains( "\"flag\": \"true\"" ), "must NOT store as string: {content}" );
}

// Type test: value::false → JSON boolean false (not quoted string)
#[ test ]
fn tc_settings_value_bool_false_inferred()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::flag", "value::false" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"flag\": false" ), "must infer bool false: {content}" );
  assert!( !content.contains( "\"flag\": \"false\"" ), "must NOT store as string: {content}" );
}

// Type test: value::42 → JSON integer 42 (not string "42")
#[ test ]
fn tc_settings_value_integer_inferred()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::count", "value::42" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"count\": 42" ), "must infer integer 42: {content}" );
  assert!( !content.contains( "\"count\": \"42\"" ), "must NOT store as string: {content}" );
}

// Type test: value::3.14 → JSON float 3.14 (not string "3.14")
#[ test ]
fn tc_settings_value_float_inferred()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::rate", "value::3.14" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"rate\": 3.14" ), "must infer float 3.14: {content}" );
  assert!( !content.contains( "\"rate\": \"3.14\"" ), "must NOT store as string: {content}" );
}

// Type test: value::dark → JSON string "dark" (string fallback)
#[ test ]
fn tc_settings_value_string_fallback()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::theme", "value::dark" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"theme\": \"dark\"" ), "must store as quoted string: {content}" );
}

// Type test: value::NaN → JSON string "NaN" (non-finite float not valid JSON)
#[ test ]
fn tc_settings_value_nan_as_string()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::special", "value::NaN" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"special\": \"NaN\"" ), "NaN must be stored as string: {content}" );
}

// ─── settings.set IT-4..IT-6: negative edge cases ────────────────────────────

// IT-4: `dry::2` → exit 1 (out-of-range boolean value)
#[ test ]
fn it04_settings_set_dry2_exits_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::k", "value::v", "dry::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
}

// IT-5: `bogus::x` → exit 1 (unknown parameter)
#[ test ]
fn it05_settings_set_bogus_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::k", "value::v", "bogus::x" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
}

// IT-6: `key::foo` without `value::` → exit 1 (value required)
#[ test ]
fn it06_settings_set_key_without_value_exits_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".settings.set", "key::foo" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
}
