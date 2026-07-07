//! Unit tests for `gate_max_attempts_from`.
//!
//! Tests the concurrency gate's attempt-limit fallback-parsing logic in isolation,
//! without spawning a subprocess or waiting on a poll loop.
//!
//! # Root Cause
//! `t11_invalid_max_attempts_env_var_falls_back_to_default` in `concurrency_gate_test.rs`
//! asserts only that the process exited, with exit code `0`, and with no `panic`
//! substring in stderr, within a 10s deadline against a 3s-lived occupier polling at
//! 1s intervals — any fallback value roughly ≥ 4 produces the exact same observable
//! pass on all 3 assertions, so no existing test pins the literal fallback value.
//!
//! # Why Not Caught
//! T11's black-box subprocess-timing design proves the fallback is "large enough" to
//! outlast the deadline, but not "exactly 1000" — only a direct function-level
//! assertion can distinguish the two.
//!
//! # Fix Applied
//! Extracted the parse-or-default logic into a pure `gate_max_attempts_from(raw:
//! Option<&str>) -> u32`, exposed via the existing `#[doc(hidden)] pub use` chain
//! mirroring `render_summary`/`resolve_fields`/`extract_session_id`.
//!
//! # Prevention
//! A new numeric env-var override with a default should always get a direct unit
//! test pinning the literal default, not just a black-box timing test.
//!
//! # Pitfall
//! Never reach for `std::env::set_var`/`remove_var` in this crate's tests — extract
//! a pure function taking the raw value as a parameter instead.
#![ cfg( feature = "enabled" ) ]

use claude_runner::gate_max_attempts_from;

#[ test ]
fn gate_max_attempts_from_none_returns_1000()
{
  assert_eq!( gate_max_attempts_from( None ), 1000 );
}

#[ test ]
fn gate_max_attempts_from_invalid_string_returns_1000()
{
  assert_eq!( gate_max_attempts_from( Some( "notanumber" ) ), 1000 );
}

#[ test ]
fn gate_max_attempts_from_valid_string_returns_parsed_value()
{
  assert_eq!( gate_max_attempts_from( Some( "7" ) ), 7 );
}
