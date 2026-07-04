//! Smoke tests: verify the binary builds and is reachable.
//!
//! ## Test Matrix
//!
//! | TC | Input | Expected |
//! |----|-------|----------|
//! | SM-1 | `CARGO_BIN_EXE_claude_version` macro | Resolves to non-empty path |
//! | SM-2 | `CARGO_BIN_EXE_clv` macro | Resolves to non-empty path |

/// SM-1 — `CARGO_BIN_EXE_claude_version` resolves to a non-empty path.
#[ test ]
fn sm1_binary_path_is_non_empty()
{
  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  assert!( !bin.is_empty(), "CARGO_BIN_EXE_claude_version must resolve to a non-empty path" );
}

/// SM-2 — `CARGO_BIN_EXE_clv` resolves to a non-empty path.
#[ test ]
fn sm2_clv_alias_is_non_empty()
{
  let bin = env!( "CARGO_BIN_EXE_clv" );
  assert!( !bin.is_empty(), "CARGO_BIN_EXE_clv must resolve to a non-empty path" );
}
