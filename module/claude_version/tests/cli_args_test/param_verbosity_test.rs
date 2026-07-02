//! `v::` / `verbosity::` parameter tests.
//!
//! Covers range validation, non-integer rejection, last-wins semantics, and
//! the canonical `verbosity::` key (alias parity with `v::`).
//!
//! ## TC-N tests
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-005 | `v::` empty value → exit 1 | N |
//! | TC-006 | `v::3` out of range → exit 1 | N |
//! | TC-007 | `v::abc` non-integer → exit 1 | N |
//! | TC-008 | `v::0` accepted via `.status` → exit 0 | P |
//! | TC-010 | Last `v::` wins when duplicated | P |
//! | TC-022 | `v::0` produces consistent output | P |
//! | TC-484 | `verbosity::3` (canonical) rejected same as `v::3` | N |
//! | TC-485 | `verbosity::-1` (canonical negative) rejected | N |
//! | TC-486 | `verbosity::0` (canonical) accepted, exits 0 | P |
//!
//! ## EC-N tests (`04_v.md`)
//! | Function | Spec | Description | Kind |
//! |----------|------|-------------|------|
//! | `verbosity_ec5_absent_defaults_to_1` | `04_v` | absent `v::` ≡ `v::1` | P |
//! | `verbosity_ec8_negative_exits_1` | `04_v` | `v::-1` → exit 1 | N |
//! | `verbosity_ec11_command_scope_settings_set` | `04_v` | `.settings.set` `v::1` → exit 1 | N |

use crate::subprocess_helpers::{ run, out_stdout, out_stderr, code };

// TC-005: v:: empty value → exit 1
#[ test ]
fn tc005_verbosity_empty_value()
{
  let out = run( &[ ".status", "v::" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "v::" ), "must mention v::: {err}" );
}

// TC-006: v::3 → exit 1 (out of range)
#[ test ]
fn tc006_verbosity_out_of_range()
{
  let out = run( &[ ".status", "v::3" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!(
    err.contains( "out of range" ) || err.contains( "0, 1, or 2" ),
    "must mention range: {err}"
  );
}

// TC-007: v::abc → exit 1
#[ test ]
fn tc007_verbosity_non_integer()
{
  let out = run( &[ ".status", "v::abc" ] );
  assert_eq!( code( &out ), 1 );
  let err = out_stderr( &out );
  assert!( err.contains( "v::" ), "must mention v::: {err}" );
}

// TC-008: v::0 accepted
#[ test ]
fn tc008_verbosity_0_accepted()
{
  let out = run( &[ ".status", "v::0" ] );
  assert_eq!( code( &out ), 0 );
}

// TC-010: last v:: wins
#[ test ]
fn tc010_last_verbosity_wins()
{
  let out = run( &[ ".status", "v::2", "v::0" ] );
  assert_eq!( code( &out ), 0 );
  let text = out_stdout( &out );
  // v::0 = bare output (no labels)
  assert!( !text.contains( "Version:" ), "last v::0 must win: {text}" );
}

// TC-022: v::0 produces consistent output
#[ test ]
fn tc022_v_param_consistent()
{
  let out_a = run( &[ ".version.list", "v::0" ] );
  let out_b = run( &[ ".version.list", "v::0" ] );
  assert_eq!( code( &out_a ), 0 );
  assert_eq!( code( &out_b ), 0 );
  assert_eq!(
    out_stdout( &out_a ), out_stdout( &out_b ),
    "v::0 must produce identical output"
  );
}

// TC-484: verbosity::3 (canonical key) rejected like v::3
//
// Root Cause
//
// The adapter validated `v::` (alias) but not `verbosity::` (canonical key).
// `verbosity::3` bypassed range checks: u8::try_from(3) succeeds, and the
// handler silently treated it as level 2 (v >= 2 branch).
//
// Why Not Caught
//
// All existing tests used `v::N` (alias form). No test supplied `verbosity::N`
// (canonical form) with an out-of-range value.
//
// Fix Applied
//
// Adapter now validates both `v::` and `verbosity::` in the same branch,
// rejecting any value outside 0–2 with a clear error message using the key
// name the user supplied.
//
// Prevention
//
// TC-484/TC-485 lock the canonical-key path; TC-006 already guards the alias.
//
// Pitfall
//
// Skipping canonical-key validation creates an asymmetry: `v::3` fails but
// `verbosity::3` silently succeeds, misleading users about accepted values.
#[ test ]
fn tc484_verbosity_canonical_out_of_range_rejected()
{
  let out = run( &[ ".status", "verbosity::3" ] );
  assert_eq!( code( &out ), 1, "verbosity::3 must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "out of range" ) || err.contains( "0, 1, or 2" ) || err.contains( "verbosity::" ),
    "error must mention range or verbosity: {err}"
  );
}

// TC-485: verbosity::-1 (canonical negative) rejected
//
// Root Cause
//
// Same bypass as TC-484: `verbosity::` skipped adapter range validation.
// parse::<u8>() on "-1" fails with `InvalidDigit`, so the error is actually
// "must be 0, 1, or 2" — but before the fix this branch was never reached,
// and unilang parsed -1 as i64 then u8::try_from(-1).unwrap_or(1) silently
// produced verbosity=1.
//
// Why Not Caught
//
// Only `v::-1` was tried. `verbosity::-1` was not tested.
//
// Fix Applied
//
// Same fix as TC-484: canonical key now goes through the same validation path.
//
// Prevention
//
// TC-485 covers the negative-value path for the canonical key.
//
// Pitfall
//
// Without the fix, `verbosity::-1` silently defaulted to verbosity=1 instead
// of exiting with a clear error, masking a user typo.
#[ test ]
fn tc485_verbosity_canonical_negative_rejected()
{
  let out = run( &[ ".status", "verbosity::-1" ] );
  assert_eq!( code( &out ), 1, "verbosity::-1 must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "verbosity::" ),
    "error must mention verbosity: {err}"
  );
}

// TC-486: verbosity::0 (canonical) accepted, exits 0
#[ test ]
fn tc486_verbosity_canonical_zero_accepted()
{
  let out = run( &[ ".status", "verbosity::0" ] );
  assert_eq!( code( &out ), 0, "verbosity::0 is valid, must exit 0" );
  // v::0 produces bare output (no "Version:" label)
  assert!( !out_stdout( &out ).contains( "Version:" ), "verbosity::0 must not show labels" );
}

// ─── 04_v.md EC-5, EC-8, EC-11 ──────────────────────────────────────────────

/// EC-5: absent `v::` defaults to `v::1` (output identical to explicit `v::1`)
#[ test ]
fn verbosity_ec5_absent_defaults_to_1()
{
  let absent   = run( &[ ".version.list" ] );
  let explicit = run( &[ ".version.list", "v::1" ] );
  assert_eq!( code( &absent ),   0, ".version.list must exit 0" );
  assert_eq!( code( &explicit ), 0, ".version.list v::1 must exit 0" );
  assert_eq!(
    out_stdout( &absent ),
    out_stdout( &explicit ),
    "absent v:: must produce same output as v::1"
  );
}

/// EC-8: `v::-1` → exit 1 (negative verbosity value)
#[ test ]
fn verbosity_ec8_negative_exits_1()
{
  let out = run( &[ ".version.list", "v::-1" ] );
  assert_eq!( code( &out ), 1, "v::-1 must exit 1" );
}

/// EC-11: `.settings.set` `v::1` → exit 1 (`v::` not accepted by mutation commands)
#[ test ]
fn verbosity_ec11_command_scope_settings_set()
{
  let out = run( &[ ".settings.set", "v::1" ] );
  assert_eq!( code( &out ), 1, ".settings.set must not accept v:: param" );
}
