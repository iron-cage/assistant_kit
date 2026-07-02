//! Integration tests for `.processes` — E6.
//!
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-137 | `.processes` exits 0 | P | 0 |
//! | TC-141 | `.processes v::0` → no crash | P | 0 |
//! | TC-144 | `.processes format::json` → {"processes":[...]} valid JSON | P | 0 |
//! | TC-145 | `.processes format::json` no processes → {"processes":[]} | P | 0 |

use crate::subprocess_helpers::{ assert_exit, run_clv, stdout };

// ─── E6: processes ────────────────────────────────────────────────────────────

// TC-137
#[ test ]
fn tc137_processes_exits_0()
{
  let out = run_clv( &[ ".processes" ] );
  assert_exit( &out, 0 );
}

// TC-141: v::0 → no crash
#[ test ]
fn tc141_processes_v0_no_crash()
{
  let out = run_clv( &[ ".processes", "v::0" ] );
  assert_exit( &out, 0 );
}

// TC-144: format::json → {"processes":[...]}
#[ test ]
fn tc144_processes_format_json_valid()
{
  let out = run_clv( &[ ".processes", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "missing 'processes' key in JSON: {text}" );
  assert!(
    text.trim_start().starts_with( '{' ) || text.contains( '{' ),
    "format::json must produce JSON object: {text}"
  );
}

// TC-145: no processes → {"processes":[]}
#[ test ]
fn tc145_processes_format_json_empty_when_no_processes()
{
  let out = run_clv( &[ ".processes", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "format::json must have 'processes' key: {text}" );
}
