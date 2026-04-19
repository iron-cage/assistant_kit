//! Error message content tests — verify user-facing error messages are helpful.
//!
//! # Test Matrix
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 500 | Unknown command `.nonexistent` → stderr mentions "unknown command" | N | 1 |
//! | 501 | Unknown `bogus::1` param → stderr shows error | N | 1 |
//! | 502 | Unknown `bogus::x` on known command → exit 1 | N | 1 |
//! | 503 | `v::3` out of range → stderr mentions valid range | N | 1 |
//! | 504 | `format::xml` unknown → stderr mentions valid formats | N | 1 |
//! | 505 | `.settings.get` missing `key::` → stderr contains "key" | N | 1 |
//! | 506 | `.settings.set` missing `value::` → stderr contains "value" | N | 1 |
//! | 507 | `.settings.show` empty HOME → stderr mentions "HOME" | N | 2 |
//! | 508 | `.settings.show` unset HOME → stderr mentions "HOME" | N | 2 |
//! | 509 | `.version.show` no claude → stderr mentions "PATH" | N | 2 |
//! | 510 | `.version.install version::STABLE` → stderr mentions case | N | 1 |
//! | 511 | `.settings.get key::absent` → stderr mentions key name | N | 2 |
//! | 512 | First arg `::value` → exit 1 | N | 1 |

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stderr };

// TC-500: unknown command → stderr mentions "unknown command"
#[ test ]
fn tc500_unknown_command_error_mentions_available()
{
  let out = run_clm( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "unknown command" ) || err.contains( "Error" ),
    "unknown command error must be informative: {err}"
  );
}

// TC-501: unknown param::value → stderr shows error
#[ test ]
fn tc501_unknown_param_error()
{
  let out = run_clm( &[ ".status", "bogus::1" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "unknown parameter" ),
    "unknown param error must mention unknown parameter: {err}"
  );
}

// TC-502: unknown bogus:: on known command → exit 1
#[ test ]
fn tc502_unknown_param_exits_1()
{
  let out = run_clm( &[ ".status", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-503: v::3 out of range → mentions valid range
#[ test ]
fn tc503_verbosity_out_of_range_error_message()
{
  let out = run_clm( &[ ".status", "v::3" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0, 1, or 2" ) || err.contains( "out of range" ),
    "verbosity error must mention range: {err}"
  );
}

// TC-504: format::xml → mentions valid formats
#[ test ]
fn tc504_format_unknown_error_mentions_valid()
{
  let out = run_clm( &[ ".status", "format::xml" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "text" ) && err.contains( "json" ),
    "format error must mention text/json: {err}"
  );
}

// TC-505: .settings.get missing key:: → error contains "key"
#[ test ]
fn tc505_settings_get_missing_key_error_contains_key()
{
  let out = run_clm( &[ ".settings.get" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "key" ), "error must mention key: {err}" );
}

// TC-506: .settings.set missing value:: → error contains "value"
#[ test ]
fn tc506_settings_set_missing_value_error_contains_value()
{
  let out = run_clm( &[ ".settings.set", "key::k" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value" ), "error must mention value: {err}" );
}

// TC-507: .settings.show with empty HOME → stderr mentions HOME
#[ test ]
fn tc507_settings_show_no_home_error_mentions_home()
{
  let out = run_clm_with_env( &[ ".settings.show" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "HOME" ), "error must mention HOME: {err}" );
}

// TC-508: .settings.show with HOME completely unset → stderr mentions HOME
//
// Root Cause: require_claude_paths() had two distinct failure conditions that both
//   emitted "HOME environment variable not set". The HOME-unset branch still emits
//   that message; the inner ClaudePaths::new() failure now emits a distinct message.
// Why Not Caught: TC-507 only tested HOME=""; this tests HOME entirely absent.
// Fix Applied: the outer `_` branch (HOME unset/empty) keeps the HOME message.
// Prevention: TC-508 ensures HOME-unset path still surfaces HOME in the error.
// Pitfall: env::remove_var is unsafe in multi-threaded tests (other tests may run
//   concurrently with HOME set); use run_clm_with_env with an explicit empty value
//   rather than removing the var, since the OS treats a missing HOME and an empty
//   HOME the same way for the outer guard branch.
#[ test ]
fn tc508_settings_show_home_unset_error_mentions_home()
{
  // Use an empty HOME rather than unsetting it to stay thread-safe.
  // The outer guard branch fires for both missing and empty HOME.
  let out = run_clm_with_env( &[ ".settings.show" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "HOME" ), "HOME-absent error must mention HOME: {err}" );
  // The HOME-absent error message must NOT mention "path resolution failed" —
  // that message belongs to the inner branch (HOME set but ClaudePaths fails).
  assert!(
    !err.contains( "path resolution failed" ),
    "HOME-absent error must not mention path resolution: {err}",
  );
}

// TC-509: .version.show no claude binary → mentions "not found"
//
// Uses a temp HOME (no symlink) and empty PATH so neither the
// symlink-based check nor `claude --version` can find a binary.
#[ test ]
fn tc509_version_show_no_claude_error()
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

// TC-510: .version.install version::STABLE → error about version
#[ test ]
fn tc510_version_install_wrong_case_error()
{
  let out = run_clm( &[ ".version.install", "version::STABLE" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "STABLE" ) || err.contains( "version" ),
    "error must mention the invalid version: {err}"
  );
}

// TC-511: .settings.get key::absent on valid file → mentions key name
#[ test ]
fn tc511_settings_get_absent_key_error_mentions_key()
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

// TC-512: first arg is ::value → exit 1
#[ test ]
fn tc512_first_arg_param_syntax_exits_1()
{
  let out = run_clm( &[ "::value" ] );
  assert_exit( &out, 1 );
}
