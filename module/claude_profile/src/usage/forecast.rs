//! Burn-rate forecast — time-to-exhaustion estimate from the quota history ring (task 544).
//!
//! Consumes the Feature 040 measurement ring (`history[].h5`) to answer "this
//! account will exhaust its 5h window in ~N minutes" and renders a warning line
//! under the `.usage` table when the estimate falls below the `alert::` horizon.
//!
//! # Design decisions (task 544)
//!
//! - **Recent slope, not whole-window least-squares.** The captured 2026-08-20
//!   incident ring holds ~2h of flat zeros followed by a 31-minute 0→100 ramp
//!   inside one 5h window. A least-squares fit over every in-window sample
//!   averages the idle prefix into the slope and underestimates an active burn —
//!   it would have alerted after exhaustion, not during. The slope is therefore
//!   the delta over the last two in-window samples, which tracks the current burn.
//! - **Confidence gate.** No estimate is produced from fewer than 3 in-window
//!   samples — a lone pair is too noisy to render a number that looks
//!   authoritative (task 544 Requirements).
//! - **Window identity by `resets_at` proximity.** The API jitters `resets_at`
//!   by ~1s across samples of the same window, so equality cannot identify a
//!   window. Samples whose `resets_at` is within `WINDOW_IDENTITY_TOLERANCE_S`
//!   of the newest sample's belong to the current window; anything further is a
//!   rollover artifact and is discarded (cross-rollover deltas are invalid).
//! - **Prediction only.** Once the extrapolated utilization reaches 100 the
//!   forecast is suppressed — actual exhaustion is visible on the row itself,
//!   and a stale ring extrapolated far past its last sample self-suppresses the
//!   same way instead of alerting forever.

use core::fmt::Write as _;
use super::types::AccountQuota;

/// Default `alert::` horizon in minutes (task 544: "default in the ~15-minute range").
pub const DEFAULT_ALERT_WITHIN_MIN : u64 = 15;

/// Max seconds two `resets_at` values may differ while describing the same 5h window.
///
/// Observed same-window jitter is ~1s; consecutive windows differ by minutes to
/// hours. 300s separates the two regimes with wide margins on both sides.
pub const WINDOW_IDENTITY_TOLERANCE_S : u64 = 300;

/// 5h window duration in seconds (matches Feature 040's reset-boundary table).
/// Shared with `format::projected_window_end_secs`, which projects a window end from a
/// touch instant — one window-length constant, never a second literal beside it.
pub const WINDOW_5H_S : u64 = 18_000;

/// A time-to-exhaustion estimate derived from the history ring.
#[ derive( Debug, Clone, Copy ) ]
pub struct BurnEstimate
{
  /// Estimated seconds from now until the 5h window reaches 100% utilization.
  pub tte_secs         : u64,
  /// Burn rate behind the estimate, in utilization percent per minute.
  pub rate_pct_per_min : f64,
}

/// Extract the current-window 5h samples `(t, utilization)` from a history ring.
///
/// The newest entry carrying an `h5` sample with a parseable `resets_at` anchors
/// the current window. Returns empty when no anchor exists or the anchored window
/// has already elapsed (utilization resets — nothing to forecast). Samples are
/// kept only when their `resets_at` is within `WINDOW_IDENTITY_TOLERANCE_S` of
/// the anchor AND their timestamp is inside the window span — both guards are
/// needed: the tolerance drops rollover artifacts the time filter would admit,
/// the time filter drops pre-window samples stamped with a same-window reset.
#[ must_use ]
#[ inline ]
pub fn h5_in_window_samples( entries : &[ claude_profile_core::account::HistoryEntry ], now_secs : u64 ) -> Vec< ( u64, f64 ) >
{
  let Some( anchor ) = entries.iter().rev().find_map( | e |
    e.h5.as_ref().and_then( |( _, r )| claude_quota::iso_to_unix_secs( r ) ) )
  else
  {
    return Vec::new();
  };
  if anchor <= now_secs
  {
    return Vec::new();
  }
  let window_start = anchor.saturating_sub( WINDOW_5H_S );
  entries.iter().filter_map( | e |
  {
    let ( u, r ) = e.h5.as_ref()?;
    let rs = claude_quota::iso_to_unix_secs( r )?;
    ( rs.abs_diff( anchor ) <= WINDOW_IDENTITY_TOLERANCE_S && e.t >= window_start )
      .then_some( ( e.t, *u ) )
  } ).collect()
}

/// Estimate seconds until 100% utilization from current-window samples.
///
/// Returns `None` when: fewer than 3 samples (confidence gate), the last-two
/// slope is zero or negative (idle account, or a decrease that only a rollover
/// artifact could produce), or the extrapolated utilization has already reached
/// 100 (exhaustion is a fact on the row, not a forecast).
#[ must_use ]
#[ inline ]
pub fn time_to_exhaustion( samples : &[ ( u64, f64 ) ], now_secs : u64 ) -> Option< BurnEstimate >
{
  if samples.len() < 3
  {
    return None;
  }
  let ( t_prev, y_prev ) = samples[ samples.len() - 2 ];
  let ( t_last, y_last ) = samples[ samples.len() - 1 ];
  let dt = t_last.saturating_sub( t_prev ) as f64;
  if dt < 1.0
  {
    return None;
  }
  let slope = ( y_last - y_prev ) / dt; // utilization %/s
  if slope <= 0.0
  {
    return None;
  }
  let y_now = y_last + slope * ( now_secs.saturating_sub( t_last ) as f64 );
  if y_now >= 100.0
  {
    return None;
  }
  let tte = ( 100.0 - y_now.max( 0.0 ) ) / slope;
  if !tte.is_finite()
  {
    return None;
  }
  // Cap at 1 year — far beyond any real window; keeps the cast lossless in range.
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  let tte_secs = tte.clamp( 0.0, 31_536_000.0 ) as u64;
  Some( BurnEstimate { tte_secs, rate_pct_per_min : slope * 60.0 } )
}

/// Build the burn-warning lines for every account forecast to exhaust its 5h
/// window within `alert_within_min` minutes. Empty when `alert_within_min` is 0
/// (alerts disabled) or no account is sub-threshold.
///
/// Each line is `⚠`-marked and every number is labeled as an estimate (`~`
/// duration, `≈` rate) — task 544 Requirements.
#[ must_use ]
#[ inline ]
pub fn burn_warnings(
  accounts         : &[ AccountQuota ],
  credential_store : &std::path::Path,
  alert_within_min : u64,
  now_secs         : u64,
) -> String
{
  if alert_within_min == 0
  {
    return String::new();
  }
  let mut out = String::new();
  for aq in accounts
  {
    let entries = claude_profile_core::account::read_history( credential_store, &aq.name );
    let samples = h5_in_window_samples( &entries, now_secs );
    let Some( est ) = time_to_exhaustion( &samples, now_secs ) else { continue; };
    if est.tte_secs < alert_within_min * 60
    {
      let _ = writeln!(
        out,
        "⚠ 5h burn · {} · ~{} to exhaustion (≈{:.1}%/min)",
        aq.name,
        crate::output::format_duration_secs( est.tte_secs ),
        est.rate_pct_per_min,
      );
    }
  }
  out
}


// Tests live in tests/usage/forecast_tests.rs (integration tests via test_bridge).
