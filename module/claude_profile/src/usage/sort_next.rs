// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
//! Next-account recommendation strategies.
//!
//! `find_next_for_strategy` and `strategy_metric` are consumed by `render.rs`
//! for the footer recommendation block. `find_first_eligible` is a private helper.

use crate::output::format_duration_secs;
use super::sort::sort_indices;
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, WEEKLY_EXHAUSTION_THRESHOLD };
use super::format::{ seven_day_left, renewal_secs, next_event_raw };

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
    // Fix(BUG-317): cancelled subscription — never eligible for rotation.
    // Root cause: billing_type="none" accounts passed all existing gates (quota, expiry,
    //   ownership) and could be recommended as next despite being permanently unusable.
    // Pitfall: account=None is ambiguous (API fetch failed); only gate when billing_type
    //   is definitively "none" with account data present.
    if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" ) { continue; }
    let Ok( data ) = &aq.result else { continue; };
    if data.five_hour.as_ref().is_some_and( |p| p.utilization >= 85.0 ) { continue; }
    if ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) == 0 { continue; }
    // Gate 9 (Claim-locked): unconditional — no force::1 bypass at the eligibility layer,
    // unlike the force-bypassable G9 explicit-command gate on .account.use/assignee:: (Feature 070).
    if aq.claim_lock { continue; }
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
/// All strategies skip weekly-exhausted accounts (`seven_day_left ≤ WEEKLY_EXHAUSTION_THRESHOLD`) via
/// the `extra` predicate — an exhausted account has negligible remaining capacity
/// regardless of its renewal timing.
pub fn find_next_for_strategy(
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
      find_first_eligible( accounts, &sorted, now_secs, |aq| seven_day_left( aq ) > WEEKLY_EXHAUSTION_THRESHOLD && ( !gate_ownership || aq.is_owned ) )
    }
    SortStrategy::Renew =>
    {
      // Fix(BUG-291): delegate to sort_indices(Renew) — unifies sort order and recommendation.
      // Root cause: an independent .filter().min_by() used five_hour_left ascending as tiebreaker;
      //   sort_indices(Renew) uses prefer_weekly ascending. Any fix to sort never propagated here.
      // Pitfall: prefer_weekly ascending means LOWER weekly capacity is preferred (benefits most
      //   from the upcoming renewal) — differs from the now-removed BUG-243 five_hour_left rationale.
      // Fix(BUG-292): weekly-floor gate via extra predicate — same floor as the now-removed
      //   drain (BUG-206) and endurance (BUG-287) strategies lacked.
      // Root cause: exhausted accounts could be recommended by renew when they had the
      //   soonest 7d reset event, despite having negligible remaining capacity.
      // Fix(BUG-324): gate changed from `prefer_weekly > 5.0` to `seven_day_left > WEEKLY_EXHAUSTION_THRESHOLD`
      //   — eligibility is model-agnostic; prefer_weekly is correct only for sort-order tiebreaks.
      // Pitfall: a weekly-exhausted account's imminent reset does not make it a useful target —
      //   skip it regardless of renewal timing.
      let sorted = sort_indices( accounts, SortStrategy::Renew, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs, |aq| seven_day_left( aq ) > WEEKLY_EXHAUSTION_THRESHOLD && ( !gate_ownership || aq.is_owned ) )
    }
    SortStrategy::Renews =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Renews, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs, |aq| seven_day_left( aq ) > WEEKLY_EXHAUSTION_THRESHOLD && ( !gate_ownership || aq.is_owned ) )
    }
  }
}

/// Format the key metric string for one strategy recommendation line.
///
/// Used in the single-strategy footer (`→ Next (strategy): name   metric`).
pub fn strategy_metric(
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
