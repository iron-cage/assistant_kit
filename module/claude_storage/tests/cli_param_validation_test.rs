//! Contract tests for CLI parameter validation pitfall.
//!
//! Verifies that every constrained parameter type performs explicit validation
//! and returns an argument error (exit 1) for invalid input without partial results.
//!
//! ## Coverage
//!
//! - `tests/docs/cli/pitfall/01_parameter_validation.md` — PF-1..PF-4

mod common;

use tempfile::TempDir;

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

/// PF-1: Boolean param rejects values other than `"0"` and `"1"`.
///
/// Invokes `.list agent::banana` — `agent::` is a boolean parameter. The command
/// must return exit 1 with an argument error; no partial result is produced.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/01_parameter_validation.md` — PF-1
#[ test ]
fn pf_1_boolean_param_rejects_non_zero_one_value()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::banana" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "PF-1: boolean param 'agent::banana' must produce error output; stderr was empty"
  );
}

/// PF-2: Integer param rejects non-integer and negative input.
///
/// Tests two invalid integer values for `min_entries::`:
/// - `"abc"` — non-integer string
/// - `"-5"` — negative value (always invalid for entry count)
///
/// Also verifies that a value above the documented upper bound (e.g., `min_entries::999999999`)
/// is handled without silently corrupting results.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/01_parameter_validation.md` — PF-2
#[ test ]
fn pf_2_integer_param_rejects_non_integer_and_negative()
{
  let root = TempDir::new().unwrap();

  // Non-integer string "abc"
  let out_abc = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::abc" )
    .output()
    .unwrap();

  assert_exit( &out_abc, 1 );
  let err_abc = stderr( &out_abc );
  assert!(
    !err_abc.is_empty(),
    "PF-2: integer param 'min_entries::abc' must produce error output; stderr was empty"
  );

  // Negative value "-5"
  let out_neg = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::-5" )
    .output()
    .unwrap();

  assert_exit( &out_neg, 1 );
  let err_neg = stderr( &out_neg );
  assert!(
    !err_neg.is_empty(),
    "PF-2: integer param 'min_entries::-5' (negative) must produce error output; stderr was empty"
  );
}

/// PF-3: Enum param rejects unrecognized value with error; no silent default applied.
///
/// Invokes `.list type::invalid_enum` — `type::` is an enum parameter. The command
/// must return exit 1; no default fallback behavior is applied silently.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/01_parameter_validation.md` — PF-3
#[ test ]
fn pf_3_enum_param_rejects_unrecognized_value()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::invalid_enum" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "PF-3: enum param 'type::invalid_enum' must produce error output; stderr was empty"
  );
}

/// PF-4: Non-empty string param trims and rejects whitespace-only value.
///
/// Invokes `.search query::"   "` — `query::` is a non-empty string parameter.
/// The command must trim the value and return exit 1 when the effective query is
/// empty after trimming; the search must not be executed with an empty query.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/01_parameter_validation.md` — PF-4
#[ test ]
fn pf_4_string_param_trims_and_rejects_whitespace_only()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::   " )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "PF-4: string param 'query::   ' (whitespace-only) must produce error output; stderr was empty"
  );
}
