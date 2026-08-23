// Integration tests for forecast.rs — burn-rate time-to-exhaustion estimate and
// `.usage` alert lines (task 544). Accesses pub(crate) items through
// claude_profile::usage::test_bridge (testing feature).
//
// ## Test Matrix (task 544)
//
// | id   | scenario                                        | expected                                             |
// |------|-------------------------------------------------|------------------------------------------------------|
// | FC-01 | alice 2026-08-20 ring replay (0→100 in ~31m)     | estimate < 15m threshold at intermediate samples → alert fires mid-burn |
// | FC-02 | flat ring (idle account)                         | no estimate, no alert                                |
// | FC-03 | ring spanning a `resets_at` rollover             | cross-rollover sample discarded; in-window slope only |
// | FC-04 | <3 in-window samples                             | suppressed (no estimate)                             |
// | FC-05 | negative slope (rollover artifact)               | no estimate, no negative time                        |
// | FC-06 | anchored window already elapsed                  | no in-window samples, no forecast                    |
// | FC-07 | burn_warnings renders the alert line             | `⚠ 5h burn` line for the burning account only        |
// | FC-08 | threshold boundary / override                    | ~54m estimate silent at 15m horizon, alerted at 60m  |
// | FC-09 | extrapolation reaches 100                        | forecast suppressed (exhaustion is a row fact)       |

use claude_profile::usage::test_bridge::{ h5_in_window_samples, time_to_exhaustion, burn_warnings, mk_named_aq, reset_iso_at };
use claude_profile_core::account::HistoryEntry;

/// Captured 2026-08-20 alice burn (5h window resetting 12:00:00Z):
/// utilization 0.0 at 09:06:31Z (t=1787216791) → 100.0 at 09:37:36Z (t=1787218656).
const RESET_UNIX : u64 = 1_787_227_200; // 2026-08-20T12:00:00Z
const BURN_T0    : u64 = 1_787_216_791; // 09:06:31Z, utilization 0.0
const BURN_SPAN  : u64 = 1_865;         // seconds to 100.0 (09:37:36Z)

fn entry( t : u64, util : f64, reset_iso : &str ) -> HistoryEntry
{
  HistoryEntry { t, h5 : Some( ( util, reset_iso.to_string() ) ), d7 : None, sn : None }
}

/// Utilization on the captured burn line at time `t` (linear 0→100 over `BURN_SPAN`).
fn burn_util_at( t : u64 ) -> f64
{
  ( t.saturating_sub( BURN_T0 ) as f64 ) * 100.0 / ( BURN_SPAN as f64 )
}

/// FC-01 (AC-01, AC-02): replaying the captured burn at the watchdog's ~3-minute
/// sampling cadence alerts mid-burn — before exhaustion — under the default 15m horizon.
///
/// Given: the ring as it would have stood at each intermediate sample — two real
///   pre-burn in-window zeros (07:43:06Z, 08:02:16Z, as captured) plus samples
///   interpolated on the recorded 0→100 line every 180s from 09:06:31Z.
/// When: `time_to_exhaustion` is evaluated at the moment of each new sample.
/// Then: at 09:12:31Z (~19% used) the estimate is Some but ABOVE the 900s horizon
///   (no premature alert); at 09:24:31Z (~58% used) it is BELOW 900s — the alert
///   fires ~13 minutes before the 09:37:36Z exhaustion, not after.
///
/// Anti-faking note: the two flat pre-burn zeros sit inside the same window; a
/// whole-window least-squares slope would average them in and stay above the
/// horizon at 09:24:31Z — this test discriminates recent-slope from whole-window-LS.
#[ test ]
fn forecast_alice_replay_alerts_mid_burn()
{
  let reset = reset_iso_at( RESET_UNIX, 0 );
  let mut ring = vec!
  [
    entry( 1_787_211_786, 0.0, &reset ), // 07:43:06Z — captured pre-burn zero
    entry( 1_787_212_936, 0.0, &reset ), // 08:02:16Z — captured pre-burn zero
  ];

  let mut alert_fired_at = None;
  for k in 0..=6_u64
  {
    let t = BURN_T0 + k * 180;
    ring.push( entry( t, burn_util_at( t ), &reset ) );
    let samples = h5_in_window_samples( &ring, t );
    assert_eq!( samples.len(), ring.len(), "FC-01: all same-window samples must pass the filter" );
    let est = time_to_exhaustion( &samples, t );
    if k == 2
    {
      let e = est.expect( "FC-01: ≥3 samples at 09:12:31Z must yield an estimate" );
      assert!( e.tte_secs >= 900, "FC-01: at ~19% used the estimate must not alert yet; got {}s", e.tte_secs );
    }
    if let Some( e ) = est
    {
      if e.tte_secs < 900 && alert_fired_at.is_none()
      {
        alert_fired_at = Some( ( t, e ) );
      }
    }
  }

  let ( t_alert, e ) = alert_fired_at.expect( "FC-01: the alert must fire at an intermediate sample" );
  assert!( t_alert < BURN_T0 + BURN_SPAN, "FC-01: alert must precede the 09:37:36Z exhaustion" );
  assert!( e.tte_secs > 0, "FC-01: mid-burn estimate must be a positive countdown" );
  // Captured burn rate ≈ 3.2%/min — the estimate must carry it (±0.5 tolerance).
  assert!( ( e.rate_pct_per_min - 3.2 ).abs() < 0.5, "FC-01: rate ≈3.2%/min, got {}", e.rate_pct_per_min );
}

/// FC-02 (AC-03): a flat (idle) ring yields no estimate — zero slope never alerts.
#[ test ]
fn forecast_flat_ring_no_estimate()
{
  let reset = reset_iso_at( 2_000_000_000, 0 );
  let ring : Vec< HistoryEntry > = ( 0..4_u64 )
    .map( | k | entry( 1_999_990_000 + k * 180, 0.0, &reset ) )
    .collect();
  let now     = 1_999_990_000 + 3 * 180;
  let samples = h5_in_window_samples( &ring, now );
  assert_eq!( samples.len(), 4 );
  assert!( time_to_exhaustion( &samples, now ).is_none(), "FC-02: flat ring must yield None" );
}

/// FC-03 (AC-01): a sample from before a `resets_at` rollover is discarded; the
/// slope comes from in-window samples only.
///
/// Given: one old-window sample (utilization 80, resets 5h earlier) followed by
///   three new-window samples climbing 0→5→10 at 180s spacing.
/// When: filtered and estimated at the last sample's time.
/// Then: exactly 3 samples survive; the estimate equals the in-window math
///   ((100−10) / (5/180s) = 3240s) — the 80→0 cross-rollover drop never enters
///   (it would have produced a negative slope and no estimate at all).
#[ test ]
fn forecast_rollover_sample_discarded()
{
  let old_reset = reset_iso_at( 2_000_000_000, 0 );
  let new_reset = reset_iso_at( 2_000_000_000 + 18_000, 0 );
  let base      = 2_000_000_100_u64;
  let ring = vec!
  [
    entry( base - 600, 80.0, &old_reset ), // previous window — must be discarded
    entry( base,             0.0,  &new_reset ),
    entry( base + 180,       5.0,  &new_reset ),
    entry( base + 360,       10.0, &new_reset ),
  ];
  let now     = base + 360;
  let samples = h5_in_window_samples( &ring, now );
  assert_eq!( samples.len(), 3, "FC-03: cross-rollover sample must be discarded" );
  let e = time_to_exhaustion( &samples, now ).expect( "FC-03: 3 in-window samples must estimate" );
  assert_eq!( e.tte_secs, 3240, "FC-03: (100−10)/(5/180) = 3240s from in-window slope only" );
}

/// FC-04: fewer than 3 in-window samples — suppressed, even on a steep climb.
#[ test ]
fn forecast_under_three_samples_suppressed()
{
  let reset = reset_iso_at( 2_000_000_000, 0 );
  let ring = vec!
  [
    entry( 1_999_990_000, 10.0, &reset ),
    entry( 1_999_990_180, 60.0, &reset ),
  ];
  let now     = 1_999_990_180;
  let samples = h5_in_window_samples( &ring, now );
  assert_eq!( samples.len(), 2 );
  assert!( time_to_exhaustion( &samples, now ).is_none(), "FC-04: <3 samples must suppress the estimate" );
}

/// FC-05: a within-window utilization decrease (API artifact) yields no estimate
/// and never a negative time.
#[ test ]
fn forecast_negative_slope_no_estimate()
{
  let reset = reset_iso_at( 2_000_000_000, 0 );
  let ring = vec!
  [
    entry( 1_999_990_000, 100.0, &reset ),
    entry( 1_999_990_180, 50.0,  &reset ),
    entry( 1_999_990_360, 20.0,  &reset ),
  ];
  let now     = 1_999_990_360;
  let samples = h5_in_window_samples( &ring, now );
  assert!( time_to_exhaustion( &samples, now ).is_none(), "FC-05: negative slope must yield None" );
}

/// FC-06: when the anchored window has already elapsed, no samples qualify —
/// utilization resets at the boundary, so there is nothing to forecast.
#[ test ]
fn forecast_elapsed_window_no_samples()
{
  let reset = reset_iso_at( 2_000_000_000, 0 );
  let ring = vec!
  [
    entry( 1_999_990_000, 40.0, &reset ),
    entry( 1_999_990_180, 60.0, &reset ),
    entry( 1_999_990_360, 80.0, &reset ),
  ];
  let now = 2_000_000_001; // past the 2_000_000_000 reset
  assert!( h5_in_window_samples( &ring, now ).is_empty(), "FC-06: elapsed window yields no samples" );
}

/// FC-09: once the extrapolated utilization reaches 100, the forecast is
/// suppressed — exhaustion is a fact shown on the row, not a prediction; a stale
/// climbing ring self-suppresses the same way instead of alerting forever.
#[ test ]
fn forecast_extrapolated_exhaustion_suppressed()
{
  let reset = reset_iso_at( 2_000_000_000, 0 );
  let ring = vec!
  [
    entry( 1_999_990_000, 50.0,  &reset ),
    entry( 1_999_990_180, 75.0,  &reset ),
    entry( 1_999_990_360, 100.0, &reset ),
  ];
  let now     = 1_999_990_360;
  let samples = h5_in_window_samples( &ring, now );
  assert!( time_to_exhaustion( &samples, now ).is_none(), "FC-09: at 100% there is nothing left to forecast" );
}

// ── burn_warnings (store-backed alert lines) ─────────────────────────────────

/// Write a 3-sample climbing ring for `name` into `store` via the real Feature 040
/// writer, ending `end_util` at `t_last` with `step_util` per 180s.
fn write_burn_ring( store : &std::path::Path, name : &str, t_last : u64, end_util : f64, step_util : f64, reset_iso : &str )
{
  for i in ( 0..3_u64 ).rev()
  {
    let t = t_last - i * 180;
    let u = end_util - ( i as f64 ) * step_util;
    claude_profile_core::account::write_history_entry( store, name, t, Some( ( u, reset_iso ) ), None, None );
  }
}

/// FC-07 (AC-02, AC-03): `burn_warnings` renders the `⚠ 5h burn` line for the
/// burning account only, labels the numbers as estimates, and `alert::0` disables.
#[ test ]
fn forecast_burn_warnings_line_rendered()
{
  let tmp   = tempfile::TempDir::new().expect( "tempdir" );
  let store = tmp.path();
  let now   = 2_000_000_000_u64;
  let reset = reset_iso_at( now + 3_600, 0 );

  // "burner": 40→70 over 360s (5%/min) → ~10m to exhaustion.
  write_burn_ring( store, "burner", now, 70.0, 15.0, &reset );
  // "idle": flat zeros.
  write_burn_ring( store, "idle", now, 0.0, 0.0, &reset );

  let accounts = vec![ mk_named_aq( "burner", 70.0, 10.0 ), mk_named_aq( "idle", 0.0, 0.0 ) ];

  let out = burn_warnings( &accounts, store, 15, now );
  assert!( out.contains( "⚠ 5h burn · burner" ), "FC-07: burner line expected, got: {out}" );
  assert!( out.contains( "to exhaustion" ) && out.contains( "%/min" ), "FC-07: estimate labels expected, got: {out}" );
  assert!( out.contains( '~' ) && out.contains( '≈' ), "FC-07: numbers must be marked as estimates, got: {out}" );
  assert!( !out.contains( "idle" ), "FC-07: idle account must not alert, got: {out}" );

  let disabled = burn_warnings( &accounts, store, 0, now );
  assert!( disabled.is_empty(), "FC-07: alert::0 must disable the footer entirely" );
}

/// FC-08: the horizon parameter gates the line — a ~54-minute estimate is silent
/// at the default 15-minute horizon and rendered at a 60-minute override.
#[ test ]
fn forecast_burn_warnings_threshold_boundary()
{
  let tmp   = tempfile::TempDir::new().expect( "tempdir" );
  let store = tmp.path();
  let now   = 2_000_000_000_u64;
  let reset = reset_iso_at( now + 7_200, 0 );

  // 0→10 over 360s (5%/180s) → (100−10)/(5/180) = 3240s ≈ 54m.
  write_burn_ring( store, "slow", now, 10.0, 5.0, &reset );
  let accounts = vec![ mk_named_aq( "slow", 10.0, 5.0 ) ];

  assert!( burn_warnings( &accounts, store, 15, now ).is_empty(), "FC-08: 54m estimate must stay silent at 15m horizon" );
  let out = burn_warnings( &accounts, store, 60, now );
  assert!( out.contains( "⚠ 5h burn · slow" ), "FC-08: 54m estimate must alert at 60m horizon, got: {out}" );
}
