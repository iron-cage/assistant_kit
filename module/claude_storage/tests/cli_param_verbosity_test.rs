//! Edge case tests for the `verbosity::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/19_verbosity.md`
//!
//! ## Coverage
//!
//! - EC-1: Value 0 accepted (silent mode)
//! - EC-2: Value 5 accepted (max allowed)
//! - EC-3: Value 6 rejected with error message
//! - EC-4: Negative value rejected
//! - EC-5: Non-integer string rejected
//! - EC-6: Alias `v::` accepted same as `verbosity::`
//! - EC-7: Omitted uses default of 1
//! - EC-8: Float value rejected

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// EC-1: Value 0 accepted (silent mode).
///
/// ## Purpose
/// Validates that `verbosity::0` is accepted and produces minimal output.
///
/// ## Coverage
/// Exit 0; output is reduced/minimal compared to default level.
///
/// ## Validation Strategy
/// Create fixture. Run `.status ``verbosity::``0`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-1
#[ test ]
fn ec_1_verbosity_0_silent_mode_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-verb", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-2: Value 5 accepted (max allowed).
///
/// ## Purpose
/// Validates that `verbosity::5` is the maximum accepted value.
///
/// ## Coverage
/// Exit 0; full verbose output with all available fields.
///
/// ## Validation Strategy
/// Create fixture. Run `.status ``verbosity::``5`. Assert exit 0 and non-empty output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-2
#[ test ]
fn ec_2_verbosity_5_max_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-verb2", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-2: verbosity::5 must produce non-empty output; got empty"
  );
}

/// EC-3: Value 6 rejected with error message.
///
/// ## Purpose
/// Validates that `verbosity::6` is rejected (out of 0-5 range).
///
/// ## Coverage
/// Exit 1; error message "verbosity must be 0-5, got 6".
///
/// ## Validation Strategy
/// Run `.status ``verbosity::``6`. Assert exit 1 and error mentions verbosity.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-3
#[ test ]
fn ec_3_verbosity_6_rejected()
{
  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( "verbosity::6" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "verbosity" ) && err.contains( '6' ),
    "EC-3: error must mention verbosity and 6; got: {err}"
  );
}

/// EC-4: Negative value rejected.
///
/// ## Purpose
/// Validates that negative verbosity values are rejected.
///
/// ## Coverage
/// Exit 1; error message "verbosity must be 0-5, got -1".
///
/// ## Validation Strategy
/// Run `.status ``verbosity::``-1`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-4
#[ test ]
fn ec_4_verbosity_negative_rejected()
{
  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( "verbosity::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "verbosity" ),
    "EC-4: error must mention verbosity; got: {err}"
  );
}

/// EC-5: Non-integer string rejected.
///
/// ## Purpose
/// Validates that non-integer verbosity values are rejected.
///
/// ## Coverage
/// Exit 1; error message about invalid integer value.
///
/// ## Validation Strategy
/// Run `.status ``verbosity::hig``h`. Assert exit 1 and error is non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-5
#[ test ]
fn ec_5_verbosity_string_rejected()
{
  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( "verbosity::high" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "EC-5: error must be non-empty for non-integer verbosity; got empty stderr"
  );
}

/// EC-6: Alias `v::` accepted same as `verbosity::`.
///
/// ## Purpose
/// Validates that `v::` is an alias for `verbosity::` producing identical results.
///
/// ## Coverage
/// Exit 0; output identical to `verbosity::2` output.
///
/// ## Validation Strategy
/// Create fixture. Run both `verbosity::2` and `v::2`. Assert identical output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-6
#[ test ]
fn ec_6_verbosity_alias_v_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-verb6", "sess", 2 );

  let out_verb = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  let out_alias = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "v::2" )
    .output()
    .unwrap();

  assert_exit( &out_verb, 0 );
  assert_exit( &out_alias, 0 );
  assert_eq!(
    stdout( &out_verb ),
    stdout( &out_alias ),
    "EC-6: v:: alias must produce identical output to verbosity::"
  );
}

/// EC-7: Omitted uses default of 1.
///
/// ## Purpose
/// Validates that omitting `verbosity::` uses the default level of 1.
///
/// ## Coverage
/// Exit 0; standard summary output (equivalent to `verbosity::1`).
///
/// ## Validation Strategy
/// Create fixture. Run `.status` and `.status ``verbosity::``1`. Assert identical output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-7
#[ test ]
fn ec_7_verbosity_omitted_defaults_to_1()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-verb7", "sess", 2 );

  let out_default = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  let out_explicit = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out_default, 0 );
  assert_exit( &out_explicit, 0 );
  assert_eq!(
    stdout( &out_default ),
    stdout( &out_explicit ),
    "EC-7: omitted verbosity must match verbosity::1 output"
  );
}

/// EC-8: Float value rejected.
///
/// ## Purpose
/// Validates that float verbosity values are rejected.
///
/// ## Coverage
/// Exit 1; error message about invalid integer value.
///
/// ## Validation Strategy
/// Run `.status ``verbosity::1``.5`. Assert exit 1 and error is non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_verbosity.md` — EC-8
#[ test ]
fn ec_8_verbosity_float_rejected()
{
  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( "verbosity::1.5" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "EC-8: error must be non-empty for float verbosity; got empty stderr"
  );
}
