//! Integration tests for `.help` — E1.
//!
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-01 | `clv .` → help output, exit 0 | P | 0 |
//! | TC-02 | `clv` (empty argv) → help output, exit 0 | P | 0 |

use crate::subprocess_helpers::{ assert_exit, run_clv, stdout };

// ─── E1: help ────────────────────────────────────────────────────────────────

// TC-01
#[ test ]
fn tc01_dot_alias_shows_help()
{
  let out = run_clv( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".status" ), "expected help listing, got: {text}" );
}

// TC-02
#[ test ]
fn tc02_empty_argv_shows_help()
{
  let out = run_clv( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".status" ), "expected help listing, got: {text}" );
}
