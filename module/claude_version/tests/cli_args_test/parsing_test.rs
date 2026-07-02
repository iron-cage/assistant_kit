//! Core command and parameter parsing tests.
//!
//! Covers command recognition (single-word, two-word, unknown), `param::value`
//! syntax enforcement, unknown param keys, and bare token rejection.
//!
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-004 | Unknown `bogus::1` param → exit 1 | N |
//! | TC-011 | Single-word subcommand `.status` parsed | P |
//! | TC-012 | Two-word subcommand `.version.list` parsed | P |
//! | TC-014 | Unknown command `.nonexistent` → exit 1 | N |
//! | TC-024 | Bare token without `::` after command → exit 1 | N |
//! | TC-025 | Bare token without `.` prefix and without `::` → exit 1 | N |
//! | TC-027 | `--` double-dash token → exit 1 (not `param::value`) | N |
//! | TC-031 | Command without dot prefix → exit 1, mentions '.' | N |
//! | TC-032 | Unknown param key `nope::x` → exit 1, mentions "unknown parameter" | N |

use crate::subprocess_helpers::{ run, out_stderr, code };

// TC-004: unknown bogus::1 param → exit 1
#[ test ]
fn tc004_unknown_param_exits_1()
{
  let out = run( &[ ".status", "bogus::1" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.to_lowercase().contains( "unknown parameter" ), "must mention unknown parameter: {err}" );
}

// TC-011: single-word subcommand
#[ test ]
fn tc011_single_word_subcommand()
{
  let out = run( &[ ".status" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-012: two-word subcommand
#[ test ]
fn tc012_two_word_subcommand()
{
  let out = run( &[ ".version.list" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-014: unknown command → exit 1
#[ test ]
fn tc014_unknown_command()
{
  let out = run( &[ ".nonexistent" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "not found" ), "must mention not found: {err}" );
}

// TC-024: bare token without :: after command → rejected
#[ test ]
fn tc024_bare_token_after_command_rejected()
{
  let out = run( &[ ".version.show", "extra" ] );
  assert_eq!( code( &out ), 1, "bare token after command must exit 1" );
  let err = out_stderr( &out );
  assert!( err.contains( "param::value" ), "must mention param::value syntax: {err}" );
}

// TC-025: bare token without dot prefix and without :: → rejected
#[ test ]
fn tc025_bare_token_without_dot_prefix()
{
  let out = run( &[ "status" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "'.'" ), "must mention dot prefix requirement: {err}" );
}

// TC-027: -- double-dash token → rejected as non-param::value
#[ test ]
fn tc027_double_dash_rejected()
{
  let out = run( &[ ".status", "--" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "param::value" ), "-- must be rejected as non-param::value: {err}" );
}

// TC-031: command without dot prefix → exit 1, mentions '.'
#[ test ]
fn tc031_command_without_dot_prefix()
{
  let out = run( &[ "version" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "'.'" ), "must mention dot prefix: {err}" );
}

// TC-032: unknown param key → exit 1, mentions "unknown parameter"
#[ test ]
fn tc032_unknown_param_key()
{
  let out = run( &[ ".status", "nope::x" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.to_lowercase().contains( "unknown parameter" ), "must mention unknown parameter: {err}" );
}
