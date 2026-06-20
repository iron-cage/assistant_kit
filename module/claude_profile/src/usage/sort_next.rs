//! Next-account recommendation strategies.
//!
//! `find_next_for_strategy` and `strategy_metric` are consumed by `render.rs`
//! for the footer recommendation block. `find_first_eligible` is a private helper.

use crate::output::format_duration_secs;
use super::sort::sort_indices;
use super::types::{ AccountQuota, SortStrategy, PreferStrategy };
use super::format::{ prefer_weekly, renewal_secs, next_event_raw };

// ── Next-account recommendation ───────────────────────────────────────────────

/// Return the first eligible (non-current, non-active, non-occupied, non-h-exhausted,
/// non-expired, `Ok`) account from a pre-sorted index slice that also satisfies `extra`,
/// or `None` when none exist.
fn find_first_eligible< F >(
  accounts  : &[ AccountQuota ],
  sorted    : &[ usize ],
  now_secs  : u64,
  extra     : F,
) -> Option< usize >
where F : Fn( &AccountQuota ) -> bool
{
  for &idx in sorted
  {
    let aq = &accounts[ idx ];
    if aq.is_current || aq.is_active { continue; }
    if aq.is_occupied_elsewhere { continue; }
    let Ok( data ) = &aq.result else { continue; };
    if data.five_hour.as_ref().is_some_and( |p| p.utilization >= 85.0 ) { continue; }
    if ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) == 0 { continue; }
    if !extra( aq ) { continue; }
    return Some( idx );
  }
  None
}

/// Find the recommended next account for a given `SortStrategy`.
///
/// All strategies sort via `sort_indices()` then pick the first eligible
/// (non-current, non-active, non-occupied, non-h-exhausted, non-expired, `Ok`)
/// account via `find_first_eligible`.
/// All strategies skip weekly-exhausted accounts (`prefer_weekly ≤ 5.0`) via
/// the `extra` predicate — an exhausted account has negligible remaining capacity
/// regardless of its renewal timing.
pub( crate ) fn find_next_for_strategy(
  accounts       : &[ AccountQuota ],
  strategy       : SortStrategy,
  prefer         : PreferStrategy,
  now_secs       : u64,
  gate_ownership : bool,
) -> Option< usize >
{
  match strategy
  {
    SortStrategy::Name =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Name, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs, |aq| prefer_weekly( aq, prefer ) > 5.0 && ( !gate_ownership || aq.is_owned ) )
    }
    SortStrategy::Renew =>
    {
      // Fix(BUG-291): delegate to sort_indices(Renew) — unifies sort order and recommendation.
      // Root cause: an independent .filter().min_by() used five_hour_left ascending as tiebreaker;
      //   sort_indices(Renew) uses prefer_weekly ascending. Any fix to sort never propagated here.
      // Pitfall: prefer_weekly ascending means LOWER weekly capacity is preferred (benefits most
      //   from the upcoming renewal) — differs from the now-removed BUG-243 five_hour_left rationale.
      // Fix(BUG-292): weekly-floor gate (prefer_weekly > 5.0) via extra predicate — same floor
      //   as the now-removed drain (BUG-206) and endurance (BUG-287) strategies lacked.
      // Root cause: exhausted accounts (prefer_weekly ≤ 5.0) could be recommended by renew when
      //   they had the soonest 7d reset event, despite having negligible remaining capacity.
      // Pitfall: a weekly-exhausted account's imminent reset does not make it a useful target —
      //   skip it regardless of renewal timing.
      let sorted = sort_indices( accounts, SortStrategy::Renew, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs, |aq| prefer_weekly( aq, prefer ) > 5.0 && ( !gate_ownership || aq.is_owned ) )
    }
    SortStrategy::Renews =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Renews, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs, |aq| prefer_weekly( aq, prefer ) > 5.0 && ( !gate_ownership || aq.is_owned ) )
    }
  }
}

/// Format the key metric string for one strategy recommendation line.
///
/// Used in the single-strategy footer (`→ Next (strategy): name   metric`).
pub( crate ) fn strategy_metric(
  aq       : &AccountQuota,
  strategy : SortStrategy,
  _prefer  : PreferStrategy,
  now_secs : u64,
) -> String
{
  match strategy
  {
    SortStrategy::Name => String::new(),
    SortStrategy::Renew =>
    {
      let Ok( data ) = &aq.result else { return String::new(); };
      // Use → Next format: min(7d_reset, sub_renewal) shown as `in {dur} {event}`.
      // Matches the → Next table column so the footer metric is immediately comparable.
      let d7_secs = data.seven_day.as_ref()
        .and_then( |p| p.resets_at.as_deref() )
        .and_then( claude_quota::iso_to_unix_secs )
        .map( |t| t.saturating_sub( now_secs ) );
      let sub_pair = renewal_secs(
        aq.renewal_at.as_deref(),
        aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
        now_secs,
      );
      let ( sub_s, sub_est ) = match sub_pair
      {
        Some( ( s, est ) ) => ( Some( s ), est ),
        None               => ( None, false ),
      };
      match next_event_raw( d7_secs, sub_s, sub_est )
      {
        None                             => "\u{2014}".to_string(),
        Some( ( secs, prefix, true  ) ) => format!( "~in {} {prefix}", format_duration_secs( secs ) ),
        Some( ( secs, prefix, false ) ) => format!( "in {} {prefix}",  format_duration_secs( secs ) ),
      }
    }
    SortStrategy::Renews =>
    {
      let sub_pair = renewal_secs(
        aq.renewal_at.as_deref(),
        aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
        now_secs,
      );
      match sub_pair
      {
        Some( ( s, false ) ) => format!( "renews in {}", format_duration_secs( s ) ),
        Some( ( s, true  ) ) => format!( "~renews in {}", format_duration_secs( s ) ),
        None                 => String::new(),
      }
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
#[ path = "sort_next_tests.rs" ]
mod tests;
