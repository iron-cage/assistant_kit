//! Next-account recommendation strategies.
//!
//! `find_next_for_strategy` and `strategy_metric` are consumed by `render.rs`
//! for the footer recommendation block. `find_first_eligible` is a private helper.

use crate::output::format_duration_secs;
use super::sort::sort_indices;
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy };
use super::format::{ five_hour_left, prefer_weekly, renewal_secs };

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

/// Find the recommended next account for a specific `next` strategy.
///
/// `Endurance` and `Drain` sort via `sort_indices()` then pick the first
/// eligible (non-current, non-active, non-occupied, non-h-exhausted,
/// non-expired, `Ok`) account.
/// `Drain` additionally skips accounts where `prefer_weekly ≤ 5.0` — a
/// weekly-exhausted account has too little remaining capacity to drain.
/// `Renew` picks the eligible account whose minimum renewal event
/// (min of `7d_resets_at` and `subscription_renewal`) fires soonest. Absent timers
/// score as `u64::MAX` (account never started — treated as furthest out).
pub( crate ) fn find_next_for_strategy(
  accounts  : &[ AccountQuota ],
  strategy  : NextStrategy,
  prefer    : PreferStrategy,
  now_secs  : u64,
) -> Option< usize >
{
  match strategy
  {
    NextStrategy::Renew =>
    {
      // Fix(BUG-229): criterion is min(7d_reset, sub_renewal) — the soonest quota
      //   renewal event, whether a weekly window reset or a subscription billing cycle.
      // Root cause: previous code used min(h5, d7); 5h is NOT a renewal event, and
      //   subscription renewal was completely ignored.
      // Pitfall: absent timers must score u64::MAX (never fires), not 0 (immediately).
      let renewal_event_secs_of = |aq : &AccountQuota| -> u64
      {
        let Ok( data ) = &aq.result else { return u64::MAX; };
        let d7 = data.seven_day.as_ref()
          .and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs )
          .map_or( u64::MAX, |t| t.saturating_sub( now_secs ) );
        let sub = renewal_secs(
          aq.renewal_at.as_deref(),
          aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
          now_secs,
        ).map_or( u64::MAX, |( s, _ )| s );
        d7.min( sub )
      };
      ( 0..accounts.len() )
        .filter( |&i|
        {
          let aq = &accounts[ i ];
          let Ok( data ) = &aq.result else { return false; };
          !aq.is_current && !aq.is_active
            && !aq.is_occupied_elsewhere
            && data.five_hour.as_ref().map_or( true, |p| p.utilization < 85.0 )
            && ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) > 0
        } )
        .min_by( |&a, &b|
        {
          let ra = renewal_event_secs_of( &accounts[ a ] );
          let rb = renewal_event_secs_of( &accounts[ b ] );
          // Fix(BUG-243): composite tiebreaker — lower five_hour_left (more depleted) preferred
          //   on equal renewal time; depleted account benefits more from the upcoming renewal event.
          // Root cause: single-key .min_by_key() fell through to input-slice order on ties.
          // Pitfall: five_hour_left returns 100.0 for None (absent data) — treated as fully loaded,
          //   so accounts with no session data are deprioritised on tie (conservative).
          let ha = five_hour_left( &accounts[ a ] );
          let hb = five_hour_left( &accounts[ b ] );
          // Fix(BUG-260): add name tiebreaker — when both sort keys tie, min_by fell through to
          //   input-slice / filesystem order (account::list() read_dir) without this.
          // Root cause: BUG-259 added the name tiebreaker to sort_indices (sort.rs) but this
          //   min_by closure is an independent code path not covered by that fix.
          // Pitfall: sort_indices and find_next_for_strategy(Renew) implement the same sort
          //   semantics independently — a fix to one never propagates to the other automatically.
          ra.cmp( &rb )
            .then_with( || ha.total_cmp( &hb ) )
            .then_with( || accounts[ a ].name.cmp( &accounts[ b ].name ) )
        } )
    }
    NextStrategy::Endurance =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Endurance, None, prefer, now_secs );
      // Fix(BUG-287): endurance arm had no weekly-floor gate; |_| true allowed
      //   weekly-exhausted (prefer_weekly ≤ 5.0) accounts to be selected when
      //   unqualified tier sorted five_hour_left DESC placed them first.
      // Root cause: BUG-206 fix added > 5.0 gate only to drain arm; endurance
      //   arm was a parallel gap not fixed at the time.
      // Pitfall: any new find_first_eligible call site must include a weekly-floor
      //   gate — |_| true is not safe when weekly-exhausted accounts can appear.
      find_first_eligible( accounts, &sorted, now_secs, |aq| prefer_weekly( aq, prefer ) > 5.0 )
    }
    NextStrategy::Drain =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Drain, None, prefer, now_secs );
      // Fix(BUG-206): skip weekly-exhausted accounts — prefer_weekly ≤ 5.0 means nothing meaningful to drain.
      // Root cause: Round 1 used > 0.0 gate; correct boundary is > 5.0 (aligns with status_emoji 🟢/🟡 threshold).
      // Pitfall: ascending sort + > 0.0 gate naturally selects lowest non-zero (1-5%) accounts first;
      //   eligibility gate must use the UI tier boundary (> 5.0), not the mathematical zero.
      find_first_eligible( accounts, &sorted, now_secs, |aq| prefer_weekly( aq, prefer ) > 5.0 )
    }
  }
}

/// Format the key metric string for one strategy recommendation line.
///
/// Used in both single-strategy (`→ Next: name  (metric)`) and multi-strategy
/// (`Next by strategy:` / `  endurance  name   metric`) footers.
pub( crate ) fn strategy_metric(
  aq       : &AccountQuota,
  strategy : NextStrategy,
  prefer   : PreferStrategy,
  now_secs : u64,
) -> String
{
  let Ok( data ) = &aq.result else { return String::new(); };
  let session_pct = data.five_hour.as_ref().map_or( 0.0, |p| 100.0 - p.utilization );
  match strategy
  {
    NextStrategy::Renew =>
    {
      // Fix(BUG-229): show min(7d_reset, sub_renewal) — the two legs of the renew criterion.
      // Root cause: previous format showed 5h and 7d timers; 5h is not a renewal event.
      // Pitfall: subscription renewal may be absent (no OauthAccountData); show only 7d in that case.
      let d7_str = data.seven_day.as_ref()
        .and_then( |p| p.resets_at.as_deref() )
        .and_then( claude_quota::iso_to_unix_secs )
        .map_or_else( || "\u{2014}".to_string(), |t| format_duration_secs( t.saturating_sub( now_secs ) ) );
      let sub_pair = renewal_secs(
        aq.renewal_at.as_deref(),
        aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
        now_secs,
      );
      match sub_pair
      {
        Some( ( s, false ) ) => format!( "7d resets in {d7_str}, renews in {}", format_duration_secs( s ) ),
        Some( ( s, true  ) ) => format!( "7d resets in {d7_str}, ~renews in {}", format_duration_secs( s ) ),
        None                 => format!( "7d resets in {d7_str}" ),
      }
    }
    NextStrategy::Endurance =>
    {
      let h5_reset_str = data.five_hour.as_ref()
        .and_then( |p| p.resets_at.as_deref() )
        .and_then( claude_quota::iso_to_unix_secs )
        .map_or_else( || "\u{2014}".to_string(), |t| format_duration_secs( t.saturating_sub( now_secs ) ) );
      format!( "{session_pct:.0}% session, 5h resets in {h5_reset_str}" )
    }
    NextStrategy::Drain =>
    {
      let weekly_pct = prefer_weekly( aq, prefer );
      // Fix(BUG-216): label and reset source reflect the binding weekly dimension.
      // Root cause: label was always "7d left" even when prefer_weekly(Any) returned
      //   seven_day_sonnet_left (Sonnet is binding), contradicting the table's "7d Left" column.
      // Pitfall: prefer::Any binds on min(7d, Son); must re-derive left values here because
      //   prefer_weekly() returns only the min, not which input was binding.
      let left_7d     = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
      let left_son    = 100.0 - data.seven_day_sonnet.as_ref().map_or( 0.0, |p| p.utilization );
      let son_binding = matches!( prefer, PreferStrategy::Sonnet )
        || ( matches!( prefer, PreferStrategy::Any ) && left_son < left_7d );
      let weekly_label = if son_binding { "7d(Son) left" } else { "7d left" };
      let reset_label  = if son_binding { "7d(Son) resets in" } else { "7d resets in" };
      let weekly_reset_str = ( if son_binding { data.seven_day_sonnet.as_ref() }
                               else           { data.seven_day.as_ref() } )
        .and_then( |p| p.resets_at.as_deref() )
        .and_then( claude_quota::iso_to_unix_secs )
        .map_or_else( || "\u{2014}".to_string(), |t| format_duration_secs( t.saturating_sub( now_secs ) ) );
      format!( "{weekly_pct:.0}% {weekly_label}, {reset_label} {weekly_reset_str}" )
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
#[ path = "sort_next_tests.rs" ]
mod tests;
