//! `ListMode` type compliance and validation tests for `claude_version`.
//!
//! Spec: `tests/docs/cli/type/10_list_mode.md` (TC-1 through TC-6)
//!
//! # Coverage Map
//!
//! | Spec | ID | Function |
//! |------|----|----------|
//! | cli/type/10_list_mode.md | TC-1 | `list_mode_tc1_aliases_shows_table` |
//! | cli/type/10_list_mode.md | TC-2 | `list_mode_tc2_history_shows_entries` |
//! | cli/type/10_list_mode.md | TC-3 | `list_mode_tc3_absent_defaults_to_aliases` |
//! | cli/type/10_list_mode.md | TC-4 | `list_mode_tc4_uppercase_exits_1` |
//! | cli/type/10_list_mode.md | TC-5 | `list_mode_tc5_unknown_exits_1` |
//! | cli/type/10_list_mode.md | TC-6 | `list_mode_tc6_empty_exits_1` |

use crate::subprocess_helpers::{ assert_exit, run_clv, stderr, stdout };

/// Alias names that only ever appear in `mode::aliases` output.
const ALIAS_MARKERS : [ &str; 3 ] = [ "latest", "stable", "month" ];

// TC-1: mode::aliases → alias table
#[ test ]
fn list_mode_tc1_aliases_shows_table()
{
  let out = run_clv( &[ ".version.list", "mode::aliases" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for marker in ALIAS_MARKERS
  {
    assert!( text.contains( marker ), "aliases output must contain {marker}: {text}" );
  }
}

// TC-2: mode::history → release history (always exit 0 — falls back to a compiled-in
// snapshot with a stderr advisory when network is unavailable; never exit 2)
#[ test ]
fn list_mode_tc2_history_shows_entries()
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

// TC-3: absent mode:: → defaults to aliases
#[ test ]
fn list_mode_tc3_absent_defaults_to_aliases()
{
  let default_out  = run_clv( &[ ".version.list" ] );
  let explicit_out = run_clv( &[ ".version.list", "mode::aliases" ] );
  assert_exit( &default_out, 0 );
  assert_eq!( stdout( &default_out ), stdout( &explicit_out ) );
}

// TC-4: mode::ALIASES → exit 1 (case-sensitive)
#[ test ]
fn list_mode_tc4_uppercase_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::ALIASES" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "mode::ALIASES rejection must produce an error message: {err}" );
}

// TC-5: mode::bogus → exit 1 (unknown variant)
#[ test ]
fn list_mode_tc5_unknown_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::bogus" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "bogus" ), "stderr must name the invalid mode: {err}" );
  assert!( err.contains( "aliases" ) && err.contains( "history" ), "stderr must list the valid mode set: {err}" );
}

// TC-6: mode:: (empty) → exit 1
#[ test ]
fn list_mode_tc6_empty_exits_1()
{
  let out = run_clv( &[ ".version.list", "mode::" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "mode" ), "stderr must reference mode:: or empty value: {err}" );
}
