// Integration tests for live.rs — secs_to_hms_utc formatting.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::secs_to_hms_utc;

/// C15 — Zero seconds → "00:00:00".
#[ test ]
fn test_secs_to_hms_utc_zero()
{
  assert_eq!( secs_to_hms_utc( 0 ), "00:00:00" );
}

/// C16 — End of day → "23:59:59".
#[ test ]
fn test_secs_to_hms_utc_end_of_day()
{
  assert_eq!( secs_to_hms_utc( 86399 ), "23:59:59" );
}

/// C17 — Exactly one day wraps to "00:00:00".
#[ test ]
fn test_secs_to_hms_utc_day_wrap()
{
  assert_eq!( secs_to_hms_utc( 86400 ), "00:00:00" );
}

/// C18 — Mid-day timestamp.
#[ test ]
fn test_secs_to_hms_utc_midday()
{
  assert_eq!( secs_to_hms_utc( 45045 ), "12:30:45" );
}
