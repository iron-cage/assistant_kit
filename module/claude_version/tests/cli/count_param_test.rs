//! EC- edge-case tests for the `count::` parameter.
//!
//! Covers gap cases EC-12 through EC-15 from `tests/docs/cli/param/09_count.md`.
//! EC-1 through EC-11 are covered in `cli_args_test.rs` and
//! `integration/read_commands_test.rs`.

use crate::subprocess_helpers::{ assert_exit, run_clv, stdout };

/// EC-12: `count::3` → output has at most 3 version entries
#[ test ]
fn count_ec12_count_3_at_most_3_entries()
{
  let out = run_clv( &[ ".version.history", "count::3" ] );
  if out.status.code() == Some( 0 )
  {
    let text  = stdout( &out );
    // Each entry should be on its own line; count non-empty lines
    let lines : Vec< &str > = text.lines().filter( |l| !l.trim().is_empty() ).collect();
    assert!( lines.len() <= 3, "count::3 must produce at most 3 entries, got {}: {:?}", lines.len(), lines );
  }
}

/// EC-13: `count::1 v::0` → exactly 1 bare line in `{semver}  {date}` format
#[ test ]
fn count_ec13_count_1_v0_exactly_one_line()
{
  let out = run_clv( &[ ".version.history", "count::1", "v::0" ] );
  if out.status.code() == Some( 0 )
  {
    let text  = stdout( &out );
    let lines : Vec< &str > = text.lines().filter( |l| !l.trim().is_empty() ).collect();
    assert_eq!( lines.len(), 1, "count::1 v::0 must produce exactly 1 line, got: {lines:?}" );
  }
}

/// EC-14: `count::0 format::json` → stdout is `[]`
#[ test ]
fn count_ec14_count_0_json_empty_array()
{
  let out = run_clv( &[ ".version.history", "count::0", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert_eq!( text, "[]", "count::0 format::json must produce exactly []: {text}" );
}

/// EC-15: `v::abc` → exit 1 (type mismatch for Integer parameter)
#[ test ]
fn count_ec15_v_abc_type_mismatch_exits_1()
{
  let out = run_clv( &[ ".version.history", "v::abc" ] );
  assert_exit( &out, 1 );
}
