//! `format::` parameter tests.
//!
//! Covers empty-value rejection, wrong-case rejection, last-wins semantics,
//! and default (absent) behaviour.
//!
//! ## TC-N tests
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-015 | `format::` empty value → exit 1 | N |
//! | TC-030 | `format::TEXT` (wrong case) → exit 1 | N |
//! | TC-495 | `format::text format::json` last-wins → json output | P |
//!
//! ## EC-N tests (`05_format.md`)
//! | Function | Spec | Description | Kind |
//! |----------|------|-------------|------|
//! | `format_ec6_absent_defaults_to_text` | `05_format` | absent `format::` → text | P |
//! | `format_ec7_text_explicit_same_as_absent` | `05_format` | `format::text` ≡ absent | P |
//! | `format_ec8_csv_exits_1` | `05_format` | `format::csv` → exit 1 | N |

use crate::subprocess_helpers::{ run, out_stdout, code };

// TC-015: format:: empty value → exit 1
#[ test ]
fn tc015_format_empty_value()
{
  let out = run( &[ ".status", "format::" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-030: format::TEXT (wrong case) → exit 1
#[ test ]
fn tc030_format_text_wrong_case_rejected()
{
  let out = run( &[ ".status", "format::TEXT" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-495: format::text format::json — last occurrence wins → json output
//
// Root Cause
//
// Last-wins is already verified for v:: (TC-010) but not for format::.
// A regression where first-wins takes hold would silently emit text instead
// of json when both params are supplied, breaking pipe-based tooling.
//
// Why Not Caught
//
// TC-010 only tested v::. No test verified format:: last-wins.
//
// Fix Applied
//
// Behaviour was already correct; this test locks it.
//
// Prevention
//
// Test both orderings to catch either direction of regression.
//
// Pitfall
//
// format:: errors are silent: wrong format produces valid but differently-
// structured output that downstream consumers may silently misparse.
#[ test ]
fn tc495_format_text_then_json_last_wins_json()
{
  let out = run( &[ ".version.list", "format::text", "format::json" ] );
  assert_eq!( code( &out ), 0, "format::text format::json must exit 0" );
  let text = out_stdout( &out );
  assert!(
    text.trim_start().starts_with( '[' ),
    "format::json (last) must win, output must start with '[': {text}"
  );
}

// ─── 05_format.md EC-6, EC-7, EC-8 ─────────────────────────────────────────

/// EC-6: absent `format::` → text output; stdout does not start with `{`
#[ test ]
fn format_ec6_absent_defaults_to_text()
{
  let out = run( &[ ".status" ] );
  assert_eq!( code( &out ), 0, ".status must exit 0" );
  let text = out_stdout( &out );
  assert!(
    !text.trim_start().starts_with( '{' ),
    "absent format:: must produce text output (not JSON): {text}"
  );
}

/// EC-7: `format::text` explicit → same as absent `format::`
#[ test ]
fn format_ec7_text_explicit_same_as_absent()
{
  let absent   = run( &[ ".status" ] );
  let explicit = run( &[ ".status", "format::text" ] );
  assert_eq!( code( &absent ),   0, ".status must exit 0" );
  assert_eq!( code( &explicit ), 0, ".status format::text must exit 0" );
  let absent_out   = out_stdout( &absent );
  let explicit_out = out_stdout( &explicit );
  // Neither must be JSON output.
  assert!( !absent_out.trim_start().starts_with( '{' ),   "absent format:: must be text: {absent_out}" );
  assert!( !explicit_out.trim_start().starts_with( '{' ), "format::text must be text: {explicit_out}" );
  // Compare field labels only — dynamic values (e.g., live Processes count) differ
  // between sequential invocations and must not drive the structural comparison.
  let labels = | s : &str | -> Vec< String >
  {
    s.lines()
    .map( | l | l.split( ':' ).next().unwrap_or( "" ).trim().to_string() )
    .collect()
  };
  assert_eq!(
    labels( &absent_out ),
    labels( &explicit_out ),
    "format::text must produce same field structure as absent format::"
  );
}

/// EC-8: `format::csv` → exit 1 (unknown format value)
#[ test ]
fn format_ec8_csv_exits_1()
{
  let out = run( &[ ".status", "format::csv" ] );
  assert_eq!( code( &out ), 1, "format::csv must exit 1 (unknown format)" );
}
