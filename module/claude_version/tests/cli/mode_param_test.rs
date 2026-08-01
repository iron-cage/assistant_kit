//! EC- edge-case tests for the `mode::` parameter.
//!
//! Spec: `tests/docs/cli/param/14_mode.md` (EC-1 through EC-7)
//!
//! # Coverage Map
//!
//! | Spec | ID | Function |
//! |------|----|----------|
//! | cli/param/14_mode.md | EC-1 | `mode_ec1_aliases_shows_alias_table` |
//! | cli/param/14_mode.md | EC-2 | `mode_ec2_history_shows_release_history` |
//! | cli/param/14_mode.md | EC-3 | `mode_ec3_absent_defaults_to_aliases` |
//! | cli/param/14_mode.md | EC-4 | `mode_ec4_invalid_exits_1` |
//! | cli/param/14_mode.md | EC-5 | `mode_ec5_empty_exits_1` |
//! | cli/param/14_mode.md | EC-6 | `mode_ec6_uppercase_exits_1` |
//! | cli/param/14_mode.md | EC-7 | `mode_ec7_count_inert_under_aliases` |

use crate::subprocess_helpers::{ assert_exit, run_clv, stderr, stdout };

/// Alias names that only ever appear in `mode::aliases` output.
const ALIAS_MARKERS : [ &str; 3 ] = [ "latest", "stable", "month" ];

// EC-1: mode::aliases → alias listing shown
#[ test ]
fn mode_ec1_aliases_shows_alias_table()
{
  let out = run_clv( &[ ".version.list", "mode::aliases" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for marker in ALIAS_MARKERS
  {
    assert!( text.contains( marker ), "aliases output must contain {marker}: {text}" );
  }
}

// EC-2: mode::history → release history shown (always exit 0 — falls back to a
// compiled-in snapshot with a stderr advisory when network is unavailable)
#[ test ]
fn mode_ec2_history_shows_release_history()
{
  let out = run_clv( &[ ".version.list", "mode::history" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.is_empty(), "history output must be non-empty" );
  for marker in ALIAS_MARKERS
  {
    assert!( !text.contains( marker ), "history output must not contain alias marker {marker}: {text}" );
  }
}

// EC-3: absent mode:: → defaults to aliases
#[ test ]
fn mode_ec3_absent_defaults_to_aliases()
{
  let default_out  = run_clv( &[ ".version.list" ] );
  let explicit_out = run_clv( &[ ".version.list", "mode::aliases" ] );
  assert_exit( &default_out, 0 );
  assert_eq!( stdout( &default_out ), stdout( &explicit_out ) );
}

// EC-4: mode::invalid → exit 1; stderr names valid values
#[ test ]
fn mode_ec4_invalid_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::invalid" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "aliases" ) && err.contains( "history" ), "stderr must list the valid mode set: {err}" );
}

// EC-5: mode:: (empty) → exit 1
#[ test ]
fn mode_ec5_empty_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "mode" ), "stderr must reference mode:: or empty value: {err}" );
}

// EC-6: mode::ALIASES (uppercase) → exit 1 (case-sensitive)
#[ test ]
fn mode_ec6_uppercase_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::ALIASES" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "mode::ALIASES rejection must produce an error message: {err}" );
}

// EC-7: count:: accepted but inert under mode::aliases
#[ test ]
fn mode_ec7_count_inert_under_aliases()
{
  let with_count    = run_clv( &[ ".version.list", "mode::aliases", "count::5" ] );
  let without_count = run_clv( &[ ".version.list", "mode::aliases" ] );
  assert_exit( &with_count, 0 );
  assert_eq!( stdout( &with_count ), stdout( &without_count ), "count:: must have no effect under mode::aliases" );
}
