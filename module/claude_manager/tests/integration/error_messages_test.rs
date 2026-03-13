//! Error message content tests — verify user-facing error messages are helpful.
//!
//! # Test Matrix
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 311 | Unknown command `.nonexistent` → stderr mentions "unknown command" | N | 1 |
//! | 312 | Unknown `bogus::1` param → stderr shows error | N | 1 |
//! | 313 | Unknown `bogus::x` on known command → exit 1 | N | 1 |
//! | 314 | `v::3` out of range → stderr mentions valid range | N | 1 |
//! | 316 | `format::xml` unknown → stderr mentions valid formats | N | 1 |
//! | 317 | `.settings.get` missing `key::` → stderr contains "key" | N | 1 |
//! | 318 | `.settings.set` missing `value::` → stderr contains "value" | N | 1 |
//! | 320a | `.settings.show` empty HOME → stderr mentions "HOME" | N | 2 |
//! | 322 | `.version.show` no claude → stderr mentions "PATH" | N | 2 |
//! | 323 | `.version.install version::STABLE` → stderr mentions case | N | 1 |
//! | 324 | `.settings.get key::absent` → stderr mentions key name | N | 2 |
//! | 326 | First arg `::value` → exit 1 | N | 1 |

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stderr };

// TC-311: unknown command → stderr mentions "unknown command"
#[ test ]
fn tc311_unknown_command_error_mentions_available()
{
  let out = run_clm( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "unknown command" ) || err.contains( "Error" ),
    "unknown command error must be informative: {err}"
  );
}

// TC-312: unknown param::value → stderr shows error
#[ test ]
fn tc312_unknown_param_error()
{
  let out = run_clm( &[ ".status", "bogus::1" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "unknown parameter" ),
    "unknown param error must mention unknown parameter: {err}"
  );
}

// TC-313: unknown bogus:: on known command → exit 1
#[ test ]
fn tc313_unknown_param_exits_1()
{
  let out = run_clm( &[ ".status", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-314: v::3 out of range → mentions valid range
#[ test ]
fn tc314_verbosity_out_of_range_error_message()
{
  let out = run_clm( &[ ".status", "v::3" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0, 1, or 2" ) || err.contains( "out of range" ),
    "verbosity error must mention range: {err}"
  );
}

// TC-316: format::xml → mentions valid formats
#[ test ]
fn tc316_format_unknown_error_mentions_valid()
{
  let out = run_clm( &[ ".status", "format::xml" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "text" ) && err.contains( "json" ),
    "format error must mention text/json: {err}"
  );
}

// TC-317: .settings.get missing key:: → error contains "key"
#[ test ]
fn tc317_settings_get_missing_key_error_contains_key()
{
  let out = run_clm( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key" ), "error must mention key: {err}" );
}

// TC-318: .settings.set missing value:: → error contains "value"
#[ test ]
fn tc318_settings_set_missing_value_error_contains_value()
{
  let out = run_clm( &[ ".settings.set", "key::k" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value" ), "error must mention value: {err}" );
}

// TC-320a: .settings.show with empty HOME → stderr mentions HOME
#[ test ]
fn tc320a_settings_show_no_home_error_mentions_home()
{
  let out = run_clm_with_env( &[ ".settings.show" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "HOME" ), "error must mention HOME: {err}" );
}

// TC-322: .version.show no claude binary → mentions "not found"
//
// Uses a temp HOME (no symlink) and empty PATH so neither the
// symlink-based check nor `claude --version` can find a binary.
#[ test ]
fn tc322_version_show_no_claude_error()
{
  let dir = tempfile::TempDir::new().unwrap();
  let fake_home = dir.path().to_str().unwrap();
  let out = run_clm_with_env(
    &[ ".version.show" ],
    &[ ( "PATH", "" ), ( "HOME", fake_home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "PATH" ) || err.contains( "not found" ),
    "error must mention PATH or not found: {err}"
  );
}

// TC-323: .version.install version::STABLE → error about version
#[ test ]
fn tc323_version_install_wrong_case_error()
{
  let out = run_clm( &[ ".version.install", "version::STABLE" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "STABLE" ) || err.contains( "version" ),
    "error must mention the invalid version: {err}"
  );
}

// TC-324: .settings.get key::absent on valid file → mentions key name
#[ test ]
fn tc324_settings_get_absent_key_error_mentions_key()
{
  let dir = tempfile::TempDir::new().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{}" ).unwrap();
  let out = run_clm_with_env(
    &[ ".settings.get", "key::absent_key" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "absent_key" ), "error must mention the key name: {err}" );
}

// TC-326: first arg is ::value → exit 1
#[ test ]
fn tc326_first_arg_param_syntax_exits_1()
{
  let out = run_clm( &[ "::value" ] );
  assert_exit( &out, 1 );
}
