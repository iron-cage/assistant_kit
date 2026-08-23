//! Unit tests for `src/cli/format.rs` — timestamp rendering.
//!
//! Relocated out of a `#[ cfg( test ) ]` module in the source file: every test
//! in this crate lives under `tests/`. `claude_storage::cli::format` is
//! `#[ doc( hidden ) ] pub` for exactly this purpose (see `src/cli/mod.rs`).

use claude_storage::cli::format::{ format_clock, format_timestamp };
use chrono::{ DateTime, Local, Utc };

/// `format_clock` must convert to local time, not slice raw UTC digits.
///
/// ## Root Cause
///
/// `format_clock` extracted `HH:MM` by slicing the raw UTC timestamp string
/// directly, never converting off the wire format — so the displayed clock
/// was silently mislabeled as local time while actually being UTC, correct
/// only for readers in UTC+0. See the `Fix(issue-time-not-local)` comment
/// at `src/cli/format.rs:365`.
///
/// ## Why Not Caught
///
/// No test existed for `format_clock` at all before this fix — the function
/// had zero coverage, so a hardcoded raw-UTC-slice bug had nothing to catch
/// it.
///
/// ## Fix Applied
///
/// `format_clock` now parses the timestamp as RFC 3339 via `chrono`,
/// converts `with_timezone( &Local )`, then formats — a genuine timezone
/// conversion instead of a raw string slice. This test computes its
/// expected value via that same `chrono` conversion, so the assertion holds
/// regardless of the machine's own timezone — it fails if the conversion is
/// dropped (regressing to a raw UTC slice) or the format string changes,
/// not because of where it runs.
///
/// ## Prevention
///
/// When a function displays a timestamp to a human, always convert through
/// a real timezone-aware type before formatting — never slice fields
/// directly out of a UTC wire-format string, even though the substring
/// looks like a valid clock reading. Caveat: on a runner whose local
/// timezone is UTC, this assertion is trivially true under both the fixed
/// and the pre-fix (raw-slice) code, so it cannot catch a regression back
/// to raw slicing on such a runner specifically. Forcing a non-UTC zone
/// deterministically would need `TZ` env mutation (`std::env::set_var`,
/// `unsafe` on newer toolchains) — not justified for a Non-Blocking gap
/// when every realistic non-UTC machine, including the one this bug was
/// originally reported from, is covered.
///
/// ## Pitfall
///
/// A bare `HH:MM` with no UTC/zone marker reads as local time to any user
/// — never display one without converting first.
// test_kind: bug_reproducer(issue-time-not-local)
#[test]
fn test_format_clock_converts_to_local_timezone()
{
  let ts = "2025-12-02T09:57:02.237Z";
  let expected = ts.parse::< DateTime< Utc > >().unwrap().with_timezone( &Local ).format( "%H:%M" ).to_string();
  assert_eq!( format_clock( ts ), expected );
}

#[test]
fn test_format_clock_falls_back_to_raw_on_unparseable_input()
{
  assert_eq!( format_clock( "not-a-timestamp" ), "not-a-timestamp" );
}

/// `format_timestamp` must convert to local time, not slice raw UTC digits.
///
/// ## Root Cause
///
/// Same root cause as `format_clock`'s own `Fix(issue-time-not-local)` note
/// (`src/cli/format.rs:365`): `format_timestamp` extracted `{date} HH:MM`
/// by slicing the raw UTC timestamp string directly, never converting off
/// the wire format — so the displayed value was silently mislabeled as
/// local time while actually being UTC.
///
/// ## Why Not Caught
///
/// No test existed for `format_timestamp` at all before this fix — same
/// gap as `format_clock`, just for the sibling function.
///
/// ## Fix Applied
///
/// Same fix as `format_clock`: parses via `chrono`, converts
/// `with_timezone( &Local )`, then formats. This test computes its
/// expected value via that same conversion, so the assertion holds
/// regardless of the machine's own timezone.
///
/// ## Prevention
///
/// See `format_clock`'s own test doc comment above for the general rule.
/// Same UTC-runner caveat applies here: this assertion cannot distinguish
/// the fixed code from the pre-fix raw-slice code on a runner whose local
/// timezone happens to be UTC.
///
/// ## Pitfall
///
/// A bare `{date} HH:MM` with no UTC/zone marker reads as local time to
/// any user — never display one without converting first.
// test_kind: bug_reproducer(issue-time-not-local)
#[test]
fn test_format_timestamp_converts_to_local_timezone()
{
  let ts = "2025-12-02T09:57:02.237Z";
  let expected = ts.parse::< DateTime< Utc > >().unwrap().with_timezone( &Local ).format( "%Y-%m-%d %H:%M" ).to_string();
  assert_eq!( format_timestamp( ts ), expected );
}

#[test]
fn test_format_timestamp_falls_back_to_raw_on_unparseable_input()
{
  assert_eq!( format_timestamp( "not-a-timestamp" ), "not-a-timestamp" );
}
