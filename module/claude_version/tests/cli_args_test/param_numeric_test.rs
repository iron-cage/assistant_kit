//! Numeric parameter tests: `count::`, `interval::`, `version::`.
//!
//! Covers empty/invalid `version::` values, semver format enforcement, and
//! overflow boundary checks for `count::` and `interval::` (u64 max vs i64 max).
//!
//! | TC | Description | Kind |
//! |----|-------------|------|
//! | TC-016 | `version::` empty value → exit 1 | N |
//! | TC-028 | `.version.install version::1.2.3.4` → exit 1 | N |
//! | TC-029 | `.version.install version::01.02.03` → exit 1 | N |
//! | TC-487 | `count::18446744073709551615` (u64 max) → clear error, exit 1 | N |
//! | TC-488 | `count::9223372036854775807` (i64 max) → accepted | P |
//! | TC-491 | `interval::18446744073709551615` (u64 max) → clear error, exit 1 | N |

use crate::subprocess_helpers::{ run, out_stderr, code };

// TC-016: version:: empty value → exit 1
#[ test ]
fn tc016_version_param_empty_value()
{
  let out = run( &[ ".version.install", "version::" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-028: 4-part semver rejected
#[ test ]
fn tc028_four_part_semver_rejected()
{
  let out = run( &[ ".version.install", "version::1.2.3.4" ] );
  assert_eq!( code( &out ), 1 );
}

// TC-029: leading-zero semver rejected
//
// Fix(issue-leading-zeros): leading zeros are not valid semver.
// Root cause: original digits-only check did not reject leading zeros.
// Pitfall: the installer silently accepts leading-zero versions but they
// 404, leaving the user without an installed binary after hot-swap.
#[ test ]
fn tc029_leading_zero_semver_rejected()
{
  let out = run( &[ ".version.install", "version::01.02.03", "dry::1" ] );
  assert_eq!( code( &out ), 1, "leading-zero semver must be rejected" );
}

// TC-487: count::18446744073709551615 (u64 max, exceeds i64 max) → clear error, exit 1
//
// Root Cause
//
// The adapter parsed count:: with u64 (accepting values > i64::MAX), then
// passed the raw string to unilang, which uses i64 internally. The unilang
// type parser then emitted a cryptic "number too large to fit in target type"
// error instead of the adapter's user-friendly "must be a non-negative integer"
// message.
//
// Why Not Caught
//
// Tests only used small values (0, 1, 10, 66). The u64/i64 boundary was not
// exercised.
//
// Fix Applied
//
// Adapter now rejects count:: / interval:: values > i64::MAX with a clear
// "value too large" message before the token reaches unilang.
//
// Prevention
//
// TC-487 reproduces the overflow scenario; TC-488 ensures the valid boundary
// (i64::MAX) is still accepted.
//
// Pitfall
//
// Documenting count:: as "non-negative integer" implies the full u64 range is
// valid. Without the upper bound check, values just above i64::MAX sneak through
// the adapter only to be rejected with an unhelpful internal error later.
#[ test ]
fn tc487_count_u64_max_rejected_with_clear_error()
{
  let out = run( &[ ".version.history", "count::18446744073709551615" ] );
  assert_eq!( code( &out ), 1, "count::u64_max must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "count::" ),
    "error must mention count: {err}"
  );
  // Must NOT produce the cryptic unilang "number too large to fit in target type" message.
  assert!(
    !err.contains( "fit in target type" ),
    "must not expose internal type error: {err}"
  );
}

// TC-488: count::9223372036854775807 (i64::MAX) accepted
#[ test ]
fn tc488_count_i64_max_accepted()
{
  // count::i64::MAX passes through the adapter without error.
  // Use .version.history; it may fail at the network level (exit 2) but must NOT exit 1
  // due to count:: validation error.
  let out = run( &[ ".version.history", "count::9223372036854775807" ] );
  // Must not exit 1 (which would indicate a count:: validation failure)
  assert_ne!( code( &out ), 1, "count::i64_max must not be rejected by adapter (exit must not be 1)" );
}

// TC-491: interval::u64max (exceeds i64::MAX) → clear error, exit 1
//
// Root Cause
//
// Same overflow boundary as count:: (TC-487): the adapter parses interval::
// as u64, then rejects values above i64::MAX before they reach unilang's i64
// parser. Without this guard, u64_max would produce a cryptic type-error.
//
// Why Not Caught
//
// TC-487/TC-488 document the count:: boundary but no parallel tests existed
// for interval::, leaving the overflow guard path untested for that param.
//
// Fix Applied
//
// Both count:: and interval:: share the same `validate_non_neg_int` path —
// the fix was already present; this test locks it down.
//
// Prevention
//
// Any non-negative integer param added in future must have a corresponding
// u64_max rejection test alongside its i64_max acceptance note.
//
// Pitfall
//
// Documenting a param as "non-negative integer" implies the full u64 range.
// Without an explicit upper-bound check (i64::MAX), values just above the
// boundary reach unilang and produce an opaque internal error.
#[ test ]
fn tc491_interval_u64_max_rejected_with_clear_error()
{
  let out = run( &[ ".version.guard", "interval::18446744073709551615" ] );
  assert_eq!( code( &out ), 1, "interval::u64_max must be rejected (exit 1)" );
  let err = out_stderr( &out );
  assert!(
    err.contains( "interval::" ),
    "error must mention interval: {err}"
  );
  assert!(
    !err.contains( "fit in target type" ),
    "must not expose internal type error: {err}"
  );
}
