//! Sort and recommendation strategies for the quota table.
//!
//! `sort_indices` is the core sort function; `find_next_for_strategy` and
//! `strategy_metric` are consumed by `render.rs` for the footer recommendation block.

use crate::output::format_duration_secs;
use super::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy };
use super::format::{ five_hour_left, prefer_weekly, renewal_secs };

// ── Sort ──────────────────────────────────────────────────────────────────────

/// Return indices into `accounts` sorted by `strategy` and `desc`.
///
/// Each strategy has a canonical direction (its `default_desc()`). Passing
/// `desc = Some(!strategy.default_desc())` inverts the canonical order.
///
/// For `drain` and `reset`, exhausted accounts (`5h Left ≤ 15%`) are always
/// appended last regardless of `desc`. For `name` and `endurance`, `desc`
/// reverses the whole slice (no exhausted floor).
///
/// See `docs/feature/020_usage_sort_strategies.md` for full algorithm specs.
#[ allow( clippy::too_many_lines ) ]
pub( crate ) fn sort_indices(
  accounts  : &[ AccountQuota ],
  strategy  : SortStrategy,
  desc      : Option< bool >,
  prefer    : PreferStrategy,
  now_secs  : u64,
) -> Vec< usize >
{
  let effective_desc = desc.unwrap_or( strategy.default_desc() );
  // `reversed`: true when effective direction deviates from the canonical direction.
  let reversed = effective_desc != strategy.default_desc();

  let all : Vec< usize > = ( 0..accounts.len() ).collect();

  match strategy
  {
    SortStrategy::Name =>
    {
      let mut v = all;
      v.sort_by( |&a, &b| accounts[ a ].name.cmp( &accounts[ b ].name ) );
      if reversed { v.reverse(); }
      v
    }

    SortStrategy::Endurance =>
    {
      let reset_secs_of = |i : usize| -> Option< u64 >
      {
        if let Ok( data ) = &accounts[ i ].result
        {
          data.five_hour.as_ref()
            .and_then( |p| p.resets_at.as_deref() )
            .and_then( claude_quota::iso_to_unix_secs )
            .map( |t| t.saturating_sub( now_secs ) )
        }
        else { None }
      };

      let ( mut qualified, mut unqualified ) : ( Vec< usize >, Vec< usize > ) =
        all.into_iter().partition( |&i|
        {
          reset_secs_of( i ).is_some_and( |r| ( 900..=3600 ).contains( &r ) )
            && prefer_weekly( &accounts[ i ], prefer ) >= 30.0
        } );

      // Qualified canonical: highest weekly first, then soonest reset.
      qualified.sort_by( |&a, &b|
      {
        let wa = prefer_weekly( &accounts[ a ], prefer );
        let wb = prefer_weekly( &accounts[ b ], prefer );
        wb.partial_cmp( &wa ).unwrap_or( core::cmp::Ordering::Equal )
          .then_with( ||
          {
            let ra = reset_secs_of( a ).unwrap_or( u64::MAX );
            let rb = reset_secs_of( b ).unwrap_or( u64::MAX );
            ra.cmp( &rb )
          } )
      } );

      // Unqualified canonical: highest 5h_left first; tiebreak highest weekly.
      unqualified.sort_by( |&a, &b|
      {
        let la = five_hour_left( &accounts[ a ] );
        let lb = five_hour_left( &accounts[ b ] );
        lb.partial_cmp( &la ).unwrap_or( core::cmp::Ordering::Equal )
          .then_with( ||
          {
            let wa = prefer_weekly( &accounts[ a ], prefer );
            let wb = prefer_weekly( &accounts[ b ], prefer );
            wb.partial_cmp( &wa ).unwrap_or( core::cmp::Ordering::Equal )
          } )
      } );

      let mut result = qualified;
      result.extend( unqualified );
      if reversed { result.reverse(); }
      result
    }

    SortStrategy::Drain =>
    {
      let ( mut non_exhausted, exhausted_vec ) : ( Vec< usize >, Vec< usize > ) =
        all.into_iter().partition( |&i| five_hour_left( &accounts[ i ] ) > 15.0 );

      // Canonical: ascending prefer_weekly (lowest 7d Left first); tiebreak ascending 5h_left.
      non_exhausted.sort_by( |&a, &b|
      {
        let wa = prefer_weekly( &accounts[ a ], prefer );
        let wb = prefer_weekly( &accounts[ b ], prefer );
        wa.partial_cmp( &wb ).unwrap_or( core::cmp::Ordering::Equal )
          .then_with( ||
          {
            let la = five_hour_left( &accounts[ a ] );
            let lb = five_hour_left( &accounts[ b ] );
            la.partial_cmp( &lb ).unwrap_or( core::cmp::Ordering::Equal )
          } )
      } );

      if reversed { non_exhausted.reverse(); }
      non_exhausted.extend( exhausted_vec );
      non_exhausted
    }

    SortStrategy::Renew =>
    {
      // Fix(BUG-229): sort key is min(7d_reset, sub_renewal) — subscription renewal is a
      //   significant quota event just like 7d reset; ignoring it caused wrong sort order.
      // Root cause: original code used only seven_day.resets_at, missing subscription leg.
      // Pitfall: renewal_secs returns None when no account data is present; treat as u64::MAX
      //   (never fires), matching the convention for absent reset timers.
      let renewal_event_secs = |i : usize| -> u64
      {
        let aq = &accounts[ i ];
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

      let ( mut non_exhausted, exhausted_vec ) : ( Vec< usize >, Vec< usize > ) =
        all.into_iter().partition( |&i| five_hour_left( &accounts[ i ] ) > 15.0 );

      // Canonical: ascending min(7d_reset, sub_renewal) (soonest event first); tiebreak ascending prefer_weekly.
      non_exhausted.sort_by( |&a, &b|
      {
        renewal_event_secs( a ).cmp( &renewal_event_secs( b ) )
          .then_with( ||
          {
            let wa = prefer_weekly( &accounts[ a ], prefer );
            let wb = prefer_weekly( &accounts[ b ], prefer );
            wa.partial_cmp( &wb ).unwrap_or( core::cmp::Ordering::Equal )
          } )
      } );

      if reversed { non_exhausted.reverse(); }
      non_exhausted.extend( exhausted_vec );
      non_exhausted
    }

    SortStrategy::Expires =>
    {
      // Sort by token expiry (expires_at_ms) ascending — accounts expiring soonest appear first.
      // Accounts with expires_at_ms == 0 (unknown expiry) are placed at the end.
      let expiry_secs_of = |i : usize| -> u64
      {
        let ms = accounts[ i ].expires_at_ms;
        if ms == 0 { u64::MAX } else { ms / 1000 }
      };
      let mut v = all;
      v.sort_by_key( |&a| expiry_secs_of( a ) );
      if reversed { v.reverse(); }
      v
    }

    SortStrategy::Renews =>
    {
      // Sort by subscription renewal timer ascending — accounts whose billing cycle renews
      // soonest appear first. Accounts without subscription data score u64::MAX (placed last).
      let renews_secs_of = |i : usize| -> u64
      {
        let aq = &accounts[ i ];
        renewal_secs(
          aq.renewal_at.as_deref(),
          aq.account.as_ref().map( |a| a.org_created_at.as_str() ),
          now_secs,
        ).map_or( u64::MAX, |( s, _ )| s )
      };
      let mut v = all;
      v.sort_by_key( |&a| renews_secs_of( a ) );
      if reversed { v.reverse(); }
      v
    }

    // sort::next is always resolved to Drain or Endurance in parse_usage_params
    // before sort_indices is called — this arm is unreachable in production code.
    SortStrategy::Next => unreachable!( "sort::Next must be resolved to a concrete strategy in parse_usage_params" ),
  }
}

// ── Next-account recommendation ───────────────────────────────────────────────

/// Return the first eligible (non-current, non-active, non-expired, `Ok`) account
/// from a pre-sorted index slice that also satisfies `extra`, or `None` when none exist.
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
    if ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) == 0 { continue; }
    if aq.result.is_err() { continue; }
    if !extra( aq ) { continue; }
    return Some( idx );
  }
  None
}

/// Find the recommended next account for a specific `next` strategy.
///
/// `Endurance` and `Drain` sort via `sort_indices()` then pick the first
/// eligible (non-current, non-active, non-expired, `Ok`) account.
/// `Drain` additionally skips accounts where `prefer_weekly == 0` — nothing
/// remains to drain, so recommending them would be self-defeating.
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
          !aq.is_current && !aq.is_active
            && ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) > 0
            && aq.result.is_ok()
        } )
        .min_by_key( |&i| renewal_event_secs_of( &accounts[ i ] ) )
    }
    NextStrategy::Endurance =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Endurance, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs, |_| true )
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
  let expires_in_secs = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
  let expires_str     = format_duration_secs( expires_in_secs );
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
      let weekly_pct = prefer_weekly( aq, prefer );
      format!( "{session_pct:.0}% session, {weekly_pct:.0}% 7d left, expires in {expires_str}" )
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
mod tests
{
  use super::*;
  use crate::usage::test_support::
  {
    FAR_FUTURE_MS,
    mk_aq_sort, mk_aq_sort_weekly, mk_aq_with_reset, mk_aq_with_7d_reset,
    reset_iso_at,
  };

  // ── sort_indices: name strategy ──────────────────────────────────────────

  /// AC-01 — `sort::name` (default) produces alphabetical order.
  #[ test ]
  fn test_sort_name_alphabetical()
  {
    let accounts = vec![
      mk_aq_sort( "zzz@test.com", 50.0, FAR_FUTURE_MS ),
      mk_aq_sort( "aaa@test.com", 50.0, FAR_FUTURE_MS ),
      mk_aq_sort( "mmm@test.com", 50.0, FAR_FUTURE_MS ),
    ];
    let indices = sort_indices( &accounts, SortStrategy::Name, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "aaa@test.com" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "mmm@test.com" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "zzz@test.com" );
  }

  /// AC-01 / AC-05 — `sort::name desc::1` produces Z→A.
  #[ test ]
  fn test_sort_name_desc_reverses()
  {
    let accounts = vec![
      mk_aq_sort( "aaa@test.com", 50.0, FAR_FUTURE_MS ),
      mk_aq_sort( "zzz@test.com", 50.0, FAR_FUTURE_MS ),
    ];
    let indices = sort_indices( &accounts, SortStrategy::Name, Some( true ), PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "zzz@test.com", "desc::1 must reverse name order" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "aaa@test.com" );
  }

  // ── sort_indices: drain strategy ─────────────────────────────────────────

  /// AC-03 — `sort::drain` places exhausted (≤15% `5h_left`) accounts last.
  /// Non-exhausted sorted by `prefer_weekly` ascending (lowest 7d Left first).
  #[ test ]
  fn test_sort_drain_exhausted_sunk_rest_ascending()
  {
    let accounts = vec![
      mk_aq_sort_weekly( "exhausted@test.com",   99.0, 40.0, 40.0 ),  // h-exhausted (1% 5h left), 60% 7d Left
      mk_aq_sort_weekly( "low_weekly@test.com",  30.0, 70.0, 70.0 ),  // 30% 7d Left — lowest weekly
      mk_aq_sort_weekly( "high_weekly@test.com", 30.0,  0.0,  0.0 ),  // 100% 7d Left
    ];
    let indices = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "low_weekly@test.com",  "lowest 7d Left non-exhausted must be first" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "high_weekly@test.com", "highest 7d Left non-exhausted second" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "exhausted@test.com",   "h-exhausted must be last" );
  }

  /// AC-03 + AC-05 — `sort::drain desc::1` reverses non-exhausted; exhausted stays last.
  #[ test ]
  fn test_sort_drain_desc_reverses_non_exhausted_only()
  {
    let accounts = vec![
      mk_aq_sort( "exhausted@test.com", 99.0, FAR_FUTURE_MS ),  // ≤15% — sunk
      mk_aq_sort( "low@test.com",       75.0, FAR_FUTURE_MS ),  // 25% left
      mk_aq_sort( "high@test.com",      30.0, FAR_FUTURE_MS ),  // 70% left
    ];
    let indices = sort_indices( &accounts, SortStrategy::Drain, Some( true ), PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "high@test.com",     "desc::1 drain: highest non-exhausted first" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "low@test.com",      "desc::1 drain: second" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "exhausted@test.com","exhausted must still be last" );
  }

  /// AC-06 — `sort::drain` without explicit `desc::` equals `desc::0` (lowest first).
  #[ test ]
  fn test_sort_drain_default_equals_desc0()
  {
    let accounts = vec![
      mk_aq_sort( "high@test.com", 30.0, FAR_FUTURE_MS ),  // 70% left
      mk_aq_sort( "low@test.com",  75.0, FAR_FUTURE_MS ),  // 25% left
    ];
    let idx_default = sort_indices( &accounts, SortStrategy::Drain, None,          PreferStrategy::Any, 0 );
    let idx_desc0   = sort_indices( &accounts, SortStrategy::Drain, Some( false ), PreferStrategy::Any, 0 );
    assert_eq!( idx_default, idx_desc0, "drain default must equal desc::0" );
    assert_eq!( accounts[ idx_default[ 0 ] ].name, "low@test.com", "lowest first with default drain" );
  }

  /// CC-044 — `sort::drain` with all accounts exhausted preserves input order.
  #[ test ]
  fn test_sort_drain_all_exhausted_preserves_input_order()
  {
    let accounts = vec![
      mk_aq_sort( "first@test.com",  99.0, FAR_FUTURE_MS ),  // 1% left — exhausted
      mk_aq_sort( "second@test.com", 97.0, FAR_FUTURE_MS ),  // 3% left — exhausted
      mk_aq_sort( "third@test.com",  95.0, FAR_FUTURE_MS ),  // 5% left — exhausted
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ idx[ 0 ] ].name, "first@test.com",  "all-exhausted drain: input order preserved" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "second@test.com", "all-exhausted drain: input order preserved" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "third@test.com",  "all-exhausted drain: input order preserved" );
  }

  /// CC-026 — `sort::drain prefer::sonnet` primary sort key: lowest `7d(Son)` ascending.
  #[ test ]
  fn test_sort_drain_prefer_sonnet_primary()
  {
    let accounts = vec![
      mk_aq_sort_weekly( "low_son@test.com",  50.0, 0.0, 80.0 ),
      mk_aq_sort_weekly( "high_son@test.com", 50.0, 0.0, 20.0 ),
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Sonnet, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "low_son@test.com",
      "prefer::sonnet drain primary: lower 7d(Son) left must be first",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "high_son@test.com",
      "prefer::sonnet drain primary: higher 7d(Son) left must be second",
    );
  }

  /// CC-027 — `sort::drain prefer::any` primary sort key: lowest `min(7d Left, 7d(Son))` ascending.
  #[ test ]
  fn test_sort_drain_prefer_any_primary()
  {
    let accounts = vec![
      mk_aq_sort_weekly( "high_any@test.com", 50.0, 30.0, 40.0 ),
      mk_aq_sort_weekly( "low_any@test.com",  50.0, 70.0, 60.0 ),
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "low_any@test.com",
      "prefer::any drain primary: lower min(7d,Son) left must be first",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "high_any@test.com",
      "prefer::any drain primary: higher min(7d,Son) left must be second",
    );
  }

  /// AC-08 — `prefer::opus` governs drain primary sort key; lowest `prefer_weekly` wins.
  #[ test ]
  fn test_prefer_opus_primary_in_drain()
  {
    let accounts = vec![
      mk_aq_sort_weekly( "low7d@test.com",  50.0, 80.0, 20.0 ),  // 7d Left=20%
      mk_aq_sort_weekly( "high7d@test.com", 50.0, 20.0, 80.0 ),  // 7d Left=80%
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Opus, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "low7d@test.com",
      "prefer::opus drain primary: lower 7d Left must be first; got: {:?}", accounts[ idx[ 0 ] ].name,
    );
  }

  /// CC-058 — Account with `five_hour: None` is treated as non-exhausted (conservative 100% left).
  #[ test ]
  fn test_sort_drain_none_five_hour_treated_as_non_exhausted()
  {
    use claude_quota::OauthUsageData;
    let mk_no_fh = |name : &str| -> AccountQuota
    {
      AccountQuota
      {
        name          : name.to_string(),
        is_current    : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms : FAR_FUTURE_MS,
        result        : Ok( OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
        account       : None,
        host          : String::new(),
        role          : String::new(),
        renewal_at    : None,
      }
    };
    let accounts = vec![
      mk_aq_sort( "low@test.com",       75.0, FAR_FUTURE_MS ),  // 25% left
      mk_no_fh(   "no_fh@test.com"                          ),  // None → 100% assumed
      mk_aq_sort( "exhausted@test.com", 99.0, FAR_FUTURE_MS ),  // 1% left — sunk
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ idx[ 0 ] ].name, "low@test.com",       "25% left drains first" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "no_fh@test.com",     "None five_hour = 100% left: last among non-exhausted" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "exhausted@test.com", "exhausted always sunk to bottom" );
  }

  // ── sort_indices: renew strategy ─────────────────────────────────────────

  /// AC-04 — `sort::renew` places exhausted accounts last; non-exhausted sorted by soonest `7d Reset`.
  #[ test ]
  fn test_sort_renew_soonest_first_exhausted_last()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_7d_reset( "late@test.com",      30.0, now, 7200  ),  // 70% left, 2h 7d reset
      mk_aq_with_7d_reset( "exhausted@test.com", 99.0, now, 600   ),  // ≤15% left — exhausted
      mk_aq_with_7d_reset( "soon@test.com",      30.0, now, 600   ),  // 70% left, 10min 7d reset
    ];
    let indices = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, now );
    assert_eq!( accounts[ indices[ 0 ] ].name, "soon@test.com",      "soonest 7d reset must be first" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "late@test.com",      "later 7d reset second" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "exhausted@test.com", "exhausted must be last" );
  }

  /// AC-06 / FT-14 — `sort::renew` without explicit `desc::` equals `desc::0` (soonest reset first).
  ///
  /// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-14]
  #[ test ]
  fn test_sort_renew_default_equals_desc0()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_7d_reset( "late@test.com",  30.0, now, 86400 ),  // resets in 24h
      mk_aq_with_7d_reset( "early@test.com", 30.0, now, 3600  ),  // resets in 1h (soonest)
    ];
    let idx_default = sort_indices( &accounts, SortStrategy::Renew, None,          PreferStrategy::Any, now );
    let idx_desc0   = sort_indices( &accounts, SortStrategy::Renew, Some( false ), PreferStrategy::Any, now );
    assert_eq!( idx_default, idx_desc0, "renew default must equal desc::0" );
    assert_eq!( accounts[ idx_default[ 0 ] ].name, "early@test.com", "soonest reset first with default renew" );
  }

  /// CC-012 — `sort::renew desc::1` reverses non-exhausted tier; exhausted floor unchanged.
  #[ test ]
  fn test_sort_renew_desc1_reverses_non_exhausted_only()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_7d_reset( "soon@test.com",      30.0, now, 600  ),  // 70% left, 10min 7d reset
      mk_aq_with_7d_reset( "late@test.com",      30.0, now, 7200 ),  // 70% left, 2h 7d reset
      mk_aq_with_7d_reset( "exhausted@test.com", 99.0, now, 600  ),  // ≤15% left — sunk
    ];
    let idx = sort_indices( &accounts, SortStrategy::Renew, Some( true ), PreferStrategy::Any, now );
    assert_eq!( accounts[ idx[ 0 ] ].name, "late@test.com",      "desc::1 renew: latest 7d reset first" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "soon@test.com",      "desc::1 renew: soonest 7d reset second" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "exhausted@test.com", "exhausted must still be last" );
  }

  /// CC-045 — `sort::renew` with all accounts exhausted preserves input order.
  #[ test ]
  fn test_sort_renew_all_exhausted_preserves_input_order()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "first@test.com",  99.0, now, 600  ),  // 1% left — exhausted
      mk_aq_with_reset( "second@test.com", 97.0, now, 7200 ),  // 3% left — exhausted
      mk_aq_with_reset( "third@test.com",  95.0, now, 3600 ),  // 5% left — exhausted
    ];
    let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, now );
    assert_eq!( accounts[ idx[ 0 ] ].name, "first@test.com",  "all-exhausted renew: input order preserved" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "second@test.com", "all-exhausted renew: input order preserved" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "third@test.com",  "all-exhausted renew: input order preserved" );
  }

  // ── sort_indices: endurance strategy ─────────────────────────────────────

  /// AC-06 — `sort::endurance` without explicit `desc::` equals `desc::1` (qualified first).
  #[ test ]
  fn test_sort_endurance_default_equals_desc1()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "unqualified@test.com", 50.0, now, 18000 ), // 5h reset — too far
      mk_aq_with_reset( "qualified@test.com",   50.0, now, 1800  ), // 30min reset ✓
    ];
    let mut accounts = accounts;
    if let Ok( ref mut data ) = accounts[ 1 ].result
    {
      data.seven_day = Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } );
    }

    let idx_default = sort_indices( &accounts, SortStrategy::Endurance, None,         PreferStrategy::Any, now );
    let idx_desc1   = sort_indices( &accounts, SortStrategy::Endurance, Some( true ), PreferStrategy::Any, now );
    assert_eq!( idx_default, idx_desc1, "endurance default must equal desc::1" );
    assert_eq!( accounts[ idx_default[ 0 ] ].name, "qualified@test.com", "qualified must be first with default" );
  }

  /// AC-07 — `prefer::sonnet` uses `7d(Son)` for endurance qualification.
  #[ test ]
  fn test_prefer_sonnet_qualifies_by_sonnet_quota()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "target@test.com", 50.0, now, 1800 ), // 30min reset
    ];
    let mut accounts = accounts;
    if let Ok( ref mut data ) = accounts[ 0 ].result
    {
      data.seven_day        = Some( claude_quota::PeriodUsage { utilization : 90.0, resets_at : None } );
      data.seven_day_sonnet = Some( claude_quota::PeriodUsage { utilization : 65.0, resets_at : None } );
    }

    use super::super::format::prefer_weekly;
    assert!( prefer_weekly( &accounts[ 0 ], PreferStrategy::Sonnet ) >= 30.0, "prefer::sonnet must return ≥30%" );
    assert!( prefer_weekly( &accounts[ 0 ], PreferStrategy::Any    ) <  30.0, "prefer::any must return <30%" );
    assert!( prefer_weekly( &accounts[ 0 ], PreferStrategy::Opus   ) <  30.0, "prefer::opus must return <30%" );

    let idx_any    = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Any,    now );
    let idx_sonnet = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Sonnet, now );
    let idx_opus   = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Opus,   now );
    assert_eq!( idx_any.len(),    1 );
    assert_eq!( idx_sonnet.len(), 1 );
    assert_eq!( idx_opus.len(),   1 );
  }

  /// CC-059/CC-060 — `prefer_weekly` with absent period data treats account as fully available.
  #[ test ]
  fn test_prefer_weekly_none_periods_treated_as_full()
  {
    let accounts = vec![
      mk_aq_sort_weekly( "has_data@test.com", 50.0, 60.0, 60.0 ),  // 7d_left=40%
      mk_aq_sort(        "no_data@test.com",  50.0, FAR_FUTURE_MS ), // seven_day=None → 100%
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Opus, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "has_data@test.com",
      "has_data (40% left) must rank first under drain ascending prefer_weekly (lowest first)",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "no_data@test.com",
      "no_data (None seven_day = 100% left) must rank second",
    );
  }

  // ── find_next_for_strategy ────────────────────────────────────────────────

  /// FT-02 of feature/023 — `find_next_for_strategy` returns `Some` when eligible; `None` when all current.
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-02]
  #[ test ]
  fn test_ft02_023_find_next_for_strategy_some_when_eligible_none_when_all_current()
  {
    let now = 0u64;
    let mut a = mk_aq_sort( "a@test.com", 20.0, FAR_FUTURE_MS );
    a.is_current = true;
    let b = mk_aq_sort( "b@test.com", 30.0, FAR_FUTURE_MS );  // 70% left
    let c = mk_aq_sort( "c@test.com", 60.0, FAR_FUTURE_MS );  // 40% left
    let accounts = vec![ a, b, c ];

    let winner_a = find_next_for_strategy( &accounts, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!( winner_a.is_some(), "find_next_for_strategy must return Some when eligible candidates exist" );
    let winner_idx = winner_a.unwrap();
    assert_eq!(
      accounts[ winner_idx ].name, "b@test.com",
      "endurance winner must be b@test.com (highest 5h_left); got index {winner_idx}",
    );

    let mut a2 = mk_aq_sort( "a@test.com", 20.0, FAR_FUTURE_MS );
    let mut b2 = mk_aq_sort( "b@test.com", 30.0, FAR_FUTURE_MS );
    let mut c2 = mk_aq_sort( "c@test.com", 60.0, FAR_FUTURE_MS );
    a2.is_current = true;
    b2.is_current = true;
    c2.is_current = true;
    let all_current = vec![ a2, b2, c2 ];

    let winner_b = find_next_for_strategy( &all_current, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!( winner_b.is_none(), "find_next_for_strategy must return None when all accounts are is_current" );
  }

  /// FT-06 of feature/009 — endurance tiebreaker: higher expiry wins when `5h Left` is tied.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-06]
  #[ test ]
  fn test_ft06_009_endurance_tiebreaker_higher_expiry_wins()
  {
    let now_ms   = 1_700_000_000_000u64;
    let now_secs = now_ms / 1000;

    let a = mk_aq_sort( "a@x.com", 50.0, now_ms + 7_200_000 );  // 2h expiry
    let b = mk_aq_sort( "b@x.com", 50.0, now_ms + 3_600_000 );  // 1h expiry
    let accounts = vec![ a, b ];

    let idx = find_next_for_strategy( &accounts, NextStrategy::Endurance, PreferStrategy::Any, now_secs );
    assert_eq!( idx, Some( 0 ), "endurance tiebreaker must pick a@x.com (higher expiry); got: {idx:?}" );
    assert_eq!( accounts[ idx.unwrap() ].name, "a@x.com", "winner must be a@x.com" );
  }

  /// FT-04/023 unit A — drain picks lowest non-exhausted (> 15% left) account first.
  #[ test ]
  fn test_find_next_drain_picks_lowest_nonexhausted()
  {
    let now    = 0u64;
    let a = mk_aq_sort( "a@test.com", 80.0, FAR_FUTURE_MS );  // 20% left — non-exhausted
    let b = mk_aq_sort( "b@test.com", 20.0, FAR_FUTURE_MS );  // 80% left — non-exhausted
    let accounts = vec![ b, a ];  // intentionally reversed to confirm sort

    let idx = find_next_for_strategy( &accounts, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!( idx.is_some(), "drain must find a winner among two non-exhausted accounts" );
    assert_eq!(
      accounts[ idx.unwrap() ].name, "a@test.com",
      "drain must pick a@test.com (20% left, lowest non-exhausted); got index {idx:?}",
    );
  }

  /// FT-04/023 unit B — drain puts exhausted accounts (≤ 15% left) after non-exhausted.
  #[ test ]
  fn test_find_next_drain_prefers_nonexhausted_over_exhausted()
  {
    let now       = 0u64;
    let exhausted = mk_aq_sort( "exhausted@test.com", 97.0, FAR_FUTURE_MS );  // 3% left — exhausted
    let healthy   = mk_aq_sort( "healthy@test.com",   20.0, FAR_FUTURE_MS );  // 80% left — non-exhausted
    let accounts  = vec![ exhausted, healthy ];

    let idx = find_next_for_strategy( &accounts, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!( idx.is_some(), "drain must find a winner when at least one non-exhausted account exists" );
    assert_eq!(
      accounts[ idx.unwrap() ].name, "healthy@test.com",
      "drain must pick healthy (80% left, non-exhausted) before exhausted (3% left); got index {idx:?}",
    );
  }

  /// FT-09/023 (BUG-206) — drain never recommends `prefer_weekly ≤ 5.0` accounts (weekly-exhausted, 🟡 tier).
  ///
  /// # Root Cause
  /// Round 1 fix used `> 0.0`; accounts in (0.0, 5.0] (🟡 tier) were still admitted.
  /// drain sort ascending puts lowest-weekly accounts first, so a 1% account ranked before 10%.
  ///
  /// # Why Not Caught
  /// Original MRE only tested the `== 0` boundary; the (0.0, 5.0] gap was untested.
  ///
  /// # Fix Applied
  /// `find_first_eligible` predicate: `prefer_weekly(aq, prefer) > 5.0` (aligns with
  /// `status_emoji` 🟢/🟡 boundary: 7d Left ≤ 5% = 🟡 = weekly-exhausted = skip).
  ///
  /// # Prevention
  /// Eligibility gate must use the UI tier boundary (> 5.0), not the mathematical zero;
  /// cover the full ≤ 5.0 range in the MRE, not just the `== 0` boundary.
  ///
  /// # Pitfall
  /// Verify BUG-206 reproducer with `PreferStrategy::Any` — `prefer_weekly=min(7d,Son)`,
  /// so Sonnet fully exhausted (`Son util=100%`) drives `prefer_weekly` to 0 even if 7d has quota.
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-09]
  #[ doc = "bug_reproducer(BUG-206)" ]
  #[ test ]
  fn mre_bug_206_drain_skips_prefer_weekly_zero_accounts()
  {
    let now = 0u64;

    let weekly_zero = mk_aq_sort_weekly( "weekly_zero@test.com", 0.0, 96.0, 100.0 );
    let weekly_ten  = mk_aq_sort_weekly( "weekly_ten@test.com",  0.0, 85.0, 90.0 );
    let accounts    = vec![ weekly_zero, weekly_ten ];

    let idx = find_next_for_strategy( &accounts, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!( idx.is_some(), "BUG-206: drain must find weekly_ten (10%) even when weekly_zero (0%) exists" );
    assert_eq!(
      accounts[ idx.unwrap() ].name, "weekly_ten@test.com",
      "BUG-206: drain must skip weekly_zero (prefer_weekly=0) and pick weekly_ten (10%); got {idx:?}",
    );

    let zero_a  = mk_aq_sort_weekly( "zero_a@test.com", 0.0, 96.0, 100.0 );
    let zero_b  = mk_aq_sort_weekly( "zero_b@test.com", 0.0, 99.0, 100.0 );
    let all_zero = vec![ zero_a, zero_b ];

    let idx2 = find_next_for_strategy( &all_zero, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!(
      idx2.is_none(),
      "BUG-206: drain must return None when all accounts have prefer_weekly=0; got {idx2:?}",
    );

    // BUG-206 reopen: accounts in (0.0, 5.0] (🟡 tier) must also be skipped.
    let weekly_zero_r = mk_aq_sort_weekly( "weekly_zero_r@test.com", 0.0, 96.0, 100.0 );  // 0%
    let weekly_one    = mk_aq_sort_weekly( "weekly_one@test.com",    0.0, 99.0,  99.0 );  // 1%
    let weekly_ten_r  = mk_aq_sort_weekly( "weekly_ten_r@test.com",  0.0, 85.0,  90.0 );  // 10%
    let accounts_r    = vec![ weekly_zero_r, weekly_one, weekly_ten_r ];

    let idx3 = find_next_for_strategy( &accounts_r, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!( idx3.is_some(), "BUG-206 reopen: drain must find weekly_ten_r (10%)" );
    assert_eq!(
      accounts_r[ idx3.unwrap() ].name, "weekly_ten_r@test.com",
      "BUG-206 reopen: drain must skip both 0% and 1% (≤ 5.0); got {idx3:?}",
    );

    let lo_a   = mk_aq_sort_weekly( "lo_a@test.com", 0.0, 96.0, 100.0 );  // 0%
    let lo_b   = mk_aq_sort_weekly( "lo_b@test.com", 0.0, 99.0,  99.0 );  // 1%
    let all_lo = vec![ lo_a, lo_b ];

    let idx4 = find_next_for_strategy( &all_lo, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!(
      idx4.is_none(),
      "BUG-206 reopen: drain must return None when all accounts have prefer_weekly ≤ 5.0; got {idx4:?}",
    );
  }

  // ── strategy_metric ───────────────────────────────────────────────────────

  /// bug_reproducer(BUG-173): endurance unqualified sort must tiebreak by highest weekly.
  ///
  /// # Root Cause
  /// `unqualified.sort_by` compared only `five_hour_left` — when multiple accounts had
  /// identical 5h utilization, insertion order silently selected the wrong account.
  ///
  /// # Why Not Caught
  /// Existing sort tests used accounts with distinct `five_hour_left` values, so the
  /// tiebreaker path was never exercised.
  ///
  /// # Fix Applied
  /// Added `.then_with(prefer_weekly)` to the `unqualified.sort_by` closure.
  ///
  /// # Prevention
  /// This test constructs 3 accounts with identical `five_hour.utilization` but varying
  /// `seven_day.utilization`, asserting deterministic sort order.
  ///
  /// # Pitfall
  /// The tiebreaker uses `prefer_weekly(prefer)` — the `prefer` parameter must be
  /// forwarded, not hardcoded. Changing the prefer strategy changes which weekly field
  /// is used for the tiebreaker.
  #[ doc = "bug_reproducer(BUG-173)" ]
  #[ test ]
  fn test_bug173_mre_endurance_unqualified_prefers_highest_weekly()
  {
    let make_account = |name : &str, five_h_util : f64, seven_d_util : f64| -> AccountQuota
    {
      AccountQuota
      {
        name          : name.to_string(),
        is_current    : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms : u64::MAX,
        result        : Ok( claude_quota::OauthUsageData
        {
          five_hour : Some( claude_quota::PeriodUsage { utilization : five_h_util, resets_at : None } ),
          seven_day : Some( claude_quota::PeriodUsage { utilization : seven_d_util, resets_at : None } ),
          seven_day_sonnet : None,
        } ),
        account : None,
        host    : String::new(),
        role    : String::new(),
        renewal_at    : None,
      }
    };

    // All three have five_hour.utilization = 50.0 (50% left).
    // Weekly utilization differs: 98%, 0%, 27% → weekly LEFT = 2%, 100%, 73%.
    let accounts = vec![
      make_account( "acct_a", 50.0, 98.0 ),  // 2% weekly left
      make_account( "acct_b", 50.0,  0.0 ),  // 100% weekly left
      make_account( "acct_c", 50.0, 27.0 ),  // 73% weekly left
    ];

    let sorted = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Any, 0 );

    // Expected canonical: highest weekly left first → B(100%), C(73%), A(2%).
    assert_eq!(
      sorted, vec![ 1, 2, 0 ],
      "BUG-173: endurance unqualified sort must tiebreak by weekly; \
       expected [B=1,C=2,A=0], got {sorted:?}",
    );
  }

  /// BUG-182 MRE: drain footer must show weekly metric (matching drain's `prefer_weekly` sort key).
  ///
  /// # Root Cause
  /// `strategy_metric` drain arm formatted `session_pct` (from `five_hour.utilization`)
  /// after TSK-194 changed drain's primary sort key to `prefer_weekly` ascending.
  ///
  /// # Why Not Caught
  /// TSK-194 only tested sort ORDER; no test existed for the footer metric string.
  ///
  /// # Fix Applied
  /// Drain arm now computes `prefer_weekly(aq, prefer)` and `seven_day.resets_at`.
  ///
  /// # Prevention
  /// Footer metric tests now assert content substring matching the sort criterion.
  ///
  /// # Pitfall
  /// When changing a sort key, audit ALL downstream consumers — not just the comparator.
  #[ doc = "bug_reproducer(BUG-182)" ]
  #[ test ]
  fn test_bug182_mre_drain_footer_shows_weekly_metric()
  {
    let now = 1_700_000_000_u64;
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 40.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 60.0,
        resets_at   : Some( reset_iso_at( now, 3600 ) ),
      } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 80.0, resets_at : None } ),
    };
    let aq = AccountQuota
    {
      name : "test@example.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : ( now + 18000 ) * 1000, result : Ok( data ), account : None,
      host : String::new(), role : String::new(),
      renewal_at    : None,
    };

    let metric = strategy_metric( &aq, NextStrategy::Drain, PreferStrategy::Any, now );

    assert!( metric.contains( "7d(Son) left" ),     "drain footer must show binding label: {metric}" );
    assert!( metric.contains( "7d(Son) resets in" ), "drain footer must show binding reset: {metric}" );
    assert!( !metric.contains( "session" ),          "drain footer must NOT show session metric: {metric}" );
  }

  #[ doc = "bug_reproducer(BUG-182)" ]
  #[ test ]
  fn test_bug182_drain_footer_prefer_sonnet()
  {
    let now = 1_700_000_000_u64;
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 40.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 60.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage
      {
        utilization : 80.0,
        resets_at   : Some( reset_iso_at( now, 7200 ) ),
      } ),
    };
    let aq = AccountQuota
    {
      name : "test@example.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : ( now + 18000 ) * 1000, result : Ok( data ), account : None,
      host : String::new(), role : String::new(),
      renewal_at    : None,
    };

    let metric = strategy_metric( &aq, NextStrategy::Drain, PreferStrategy::Sonnet, now );

    assert!( metric.contains( "20% 7d(Son) left" ),  "sonnet drain must show sonnet weekly: {metric}" );
    assert!( metric.contains( "7d(Son) resets in" ), "sonnet drain must show binding reset: {metric}" );
  }

  #[ doc = "bug_reproducer(BUG-182)" ]
  #[ test ]
  fn test_bug182_drain_footer_prefer_opus()
  {
    let now = 1_700_000_000_u64;
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 40.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 60.0,
        resets_at   : Some( reset_iso_at( now, 3600 ) ),
      } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 80.0, resets_at : None } ),
    };
    let aq = AccountQuota
    {
      name : "test@example.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : ( now + 18000 ) * 1000, result : Ok( data ), account : None,
      host : String::new(), role : String::new(),
      renewal_at    : None,
    };

    let metric = strategy_metric( &aq, NextStrategy::Drain, PreferStrategy::Opus, now );

    assert!( metric.contains( "40% 7d left" ),   "opus drain must show opus weekly: {metric}" );
    assert!( metric.contains( "7d resets in" ),   "opus drain must show weekly reset: {metric}" );
  }

  #[ doc = "bug_reproducer(BUG-182)" ]
  #[ test ]
  fn test_bug182_drain_footer_no_weekly_data()
  {
    let now = 1_700_000_000_u64;
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 40.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let aq = AccountQuota
    {
      name : "test@example.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : ( now + 18000 ) * 1000, result : Ok( data ), account : None,
      host : String::new(), role : String::new(),
      renewal_at    : None,
    };

    let metric = strategy_metric( &aq, NextStrategy::Drain, PreferStrategy::Any, now );

    assert!( metric.contains( "100% 7d left" ), "no-data drain must show 100%%: {metric}" );
    assert!( metric.contains( "\u{2014}" ),      "no-data drain must show em-dash for reset: {metric}" );
  }

  /// `strategy_metric` drain arm: Son-binding footer must show `"7d(Son) left"` not `"7d left"`.
  ///
  /// # Root Cause
  /// The drain arm format string was static: `"{pct:.0}% 7d left, 7d resets in …"`.
  /// When Son was binding, the label said `7d left` — matching the overall-7d column
  /// header while showing a different (lower) number.
  ///
  /// # Why Not Caught
  /// BUG-182 tests only asserted generic `contains("7d left")` without distinguishing
  /// which quota was binding. No test had Son < 7d AND asserted the absence of `"7d left"`.
  ///
  /// # Fix Applied
  /// Added `son_binding` boolean; label and reset source are selected dynamically.
  /// `son_binding = matches!(prefer, Sonnet) || (matches!(prefer, Any) && left_son < left_7d)`.
  ///
  /// # Prevention
  /// Tests must assert the EXACT label string AND negate the old label.
  /// Distinct `resets_at` timestamps (T1 ≠ T2) required so reset-source selection is verifiable.
  ///
  /// # Pitfall
  /// `"7d left"` is not a substring of `"7d(Son) left"` (space vs parenthesis after `7d`).
  /// Use `contains("7d(Son) left")` for Son-binding assertions.
  #[ doc = "bug_reproducer(BUG-216)" ]
  #[ test ]
  fn mre_bug_216_drain_footer_label_sonnet_binding()
  {
    let now = 1_700_000_000_u64;
    // Son (left=39%) < 7d (left=61%) → Son is binding.
    let data = claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 39.0,
        resets_at   : Some( reset_iso_at( now, 3600 ) ),  // T1: resets in 1h
      } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage
      {
        utilization : 61.0,
        resets_at   : Some( reset_iso_at( now, 7200 ) ),  // T2: resets in 2h
      } ),
    };
    let aq = AccountQuota
    {
      name : "son_binding@test.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : ( now + 86400 ) * 1000, result : Ok( data ), account : None,
      host : String::new(), role : String::new(),
      renewal_at    : None,
    };

    let result = strategy_metric( &aq, NextStrategy::Drain, PreferStrategy::Any, now );

    assert!(
      result.contains( "39% 7d(Son) left" ),
      "BUG-216: Son-binding drain must show '39% 7d(Son) left'; got: {result}",
    );
    assert!(
      !result.contains( "7d left," ),
      "BUG-216: old '7d left,' label must be absent when Son is binding; got: {result}",
    );
    assert!(
      result.contains( "7d(Son) resets in" ),
      "BUG-216: Son-binding reset label must be '7d(Son) resets in'; got: {result}",
    );
    assert!(
      !result.contains( "7d resets in" ),
      "BUG-216: old '7d resets in' label must be absent when Son is binding; got: {result}",
    );
  }

  /// Regression guard: when 7d is binding, footer must still show `"7d left"`.
  #[ doc = "bug_reproducer(BUG-216)" ]
  #[ test ]
  fn mre_bug_216_drain_footer_label_7d_binding()
  {
    let now = 1_700_000_000_u64;
    // 7d (left=39%) < Son (left=61%) → 7d is binding.
    let data = claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 61.0,
        resets_at   : Some( reset_iso_at( now, 3600 ) ),  // T1: resets in 1h
      } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage
      {
        utilization : 39.0,
        resets_at   : Some( reset_iso_at( now, 7200 ) ),  // T2: resets in 2h
      } ),
    };
    let aq = AccountQuota
    {
      name : "7d_binding@test.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
      expires_at_ms : ( now + 86400 ) * 1000, result : Ok( data ), account : None,
      host : String::new(), role : String::new(),
      renewal_at    : None,
    };

    let result = strategy_metric( &aq, NextStrategy::Drain, PreferStrategy::Any, now );

    assert!(
      result.contains( "39% 7d left" ),
      "BUG-216 regression: 7d-binding drain must show '39% 7d left'; got: {result}",
    );
    assert!(
      !result.contains( "7d(Son) left" ),
      "BUG-216 regression: '7d(Son) left' must be absent when 7d is binding; got: {result}",
    );
    assert!(
      result.contains( "7d resets in" ),
      "BUG-216 regression: 7d-binding reset label must be '7d resets in'; got: {result}",
    );
  }

  // ── BUG-229: renew criterion must use min(7d_reset, sub_renewal) ──────────

  /// BUG-229 MRE: `sort::renew` must rank the account with the soonest subscription
  /// renewal above one with a sooner 7d reset when the subscription fires first.
  ///
  /// # Root Cause
  /// `sort_indices::Renew` closure used only `seven_day.resets_at`, ignoring the
  /// subscription renewal leg entirely.
  ///
  /// # Why Not Caught
  /// All prior renew sort tests set `renewal_at: None`, so the subscription leg
  /// was always `u64::MAX` and the 7d timer always won by default.
  ///
  /// # Fix Applied
  /// `renewal_event_secs` closure computes `d7.min(sub)` where `sub` comes from
  /// `renewal_secs(aq.renewal_at, aq.account.org_created_at, now_secs)`.
  ///
  /// # Prevention
  /// All renew sort tests that exercise the primary sort key must include at least
  /// one account with `renewal_at` set shorter than the 7d reset.
  ///
  /// # Pitfall
  /// `mk_aq_with_7d_reset` sets `renewal_at: None`; to test subscription leg, mutate
  /// the struct after construction or build it directly.
  #[ doc = "bug_reproducer(BUG-229)" ]
  #[ test ]
  fn mre_bug229_sort_renew_subscription_sooner_than_7d_ranks_first()
  {
    let now : u64 = 1_700_000_000;
    // Account A: 7d_reset = 1h, no subscription → renewal_event = 3600s.
    let acct_a = mk_aq_with_7d_reset( "a@test.com", 30.0, now, 3600 );
    // Account B: 7d_reset = 24h, subscription renewal = 30min → renewal_event = min(86400,1800) = 1800s.
    let mut acct_b = mk_aq_with_7d_reset( "b@test.com", 30.0, now, 86400 );
    acct_b.renewal_at = Some( reset_iso_at( now, 1800 ) );

    let accounts = vec![ acct_a, acct_b ];
    let indices  = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, now );

    assert_eq!(
      accounts[ indices[ 0 ] ].name, "b@test.com",
      "BUG-229: sort::renew must rank b first (sub 30min < a 7d 1h); got: {}",
      accounts[ indices[ 0 ] ].name,
    );
    assert_eq!(
      accounts[ indices[ 1 ] ].name, "a@test.com",
      "BUG-229: sort::renew must rank a second; got: {}",
      accounts[ indices[ 1 ] ].name,
    );
  }

  /// BUG-229 MRE: `next::renew` must pick the account with the soonest subscription
  /// renewal when it fires before any 7d reset.
  ///
  /// # Root Cause
  /// `find_next_for_strategy::Renew` closure used `h5.min(d7)` — 5h is not a renewal
  /// event, and subscription renewal was never consulted.
  ///
  /// # Why Not Caught
  /// All prior `next::renew` tests set `renewal_at: None`, exercising only the 7d leg.
  ///
  /// # Fix Applied
  /// `renewal_event_secs_of` closure computes `d7.min(sub)` using `renewal_secs`.
  ///
  /// # Prevention
  /// `find_next_for_strategy` tests must exercise the subscription leg with a concrete
  /// `renewal_at` value that fires before the 7d timer of the competing account.
  ///
  /// # Pitfall
  /// `renewal_secs` returns `None` for accounts with no `renewal_at` and no
  /// `account.org_created_at`; those score `u64::MAX` for the sub leg (correct: never fires).
  #[ doc = "bug_reproducer(BUG-229)" ]
  #[ test ]
  fn mre_bug229_find_next_renew_picks_account_with_sooner_subscription()
  {
    let now : u64 = 1_700_000_000;
    // Account A: is_current → skip.
    let mut acct_a = mk_aq_sort( "a@test.com", 30.0, FAR_FUTURE_MS );
    acct_a.is_current = true;
    // Account B: 7d_reset = 24h, subscription renewal = 30min → event = 1800s.
    let mut acct_b = mk_aq_with_7d_reset( "b@test.com", 30.0, now, 86400 );
    acct_b.renewal_at = Some( reset_iso_at( now, 1800 ) );
    // Account C: 7d_reset = 1h, no subscription → event = 3600s.
    let acct_c = mk_aq_with_7d_reset( "c@test.com", 30.0, now, 3600 );

    let accounts = vec![ acct_a, acct_b, acct_c ];
    let winner   = find_next_for_strategy( &accounts, NextStrategy::Renew, PreferStrategy::Any, now );

    assert_eq!(
      winner, Some( 1 ),
      "BUG-229: next::renew must pick b (sub 30min < c 7d 1h); got: {winner:?}",
    );
    assert_eq!( accounts[ winner.unwrap() ].name, "b@test.com",
      "BUG-229: winner name must be b@test.com" );
  }

  /// BUG-229 MRE: `strategy_metric(Renew)` must show `"7d resets in X, renews in Y"`
  /// when subscription data is present (exact), and `"7d resets in X"` only when absent.
  ///
  /// # Root Cause
  /// Previous format was `"{pct}% session, 5h resets in {h5} / 7d resets in {d7}"` — the
  /// criterion timers (d7 + sub) were not shown; session% and 5h are irrelevant to renew.
  ///
  /// # Why Not Caught
  /// No test asserted the renew metric format; only drain/endurance metric tests existed.
  ///
  /// # Fix Applied
  /// Renew arm now computes `d7_str` and `sub_pair` from `renewal_secs`, producing
  /// `"7d resets in {d7}, renews in {sub}"` or `"7d resets in {d7}"` when no sub data.
  ///
  /// # Prevention
  /// Test all three branches: exact sub, estimated sub (via `org_created_at`), no sub.
  ///
  /// # Pitfall
  /// `strategy_metric` receives `&AccountQuota` — the test must include `renewal_at` and/or
  /// `account: Some(OauthAccountData{...})` on the struct, not on the `OauthUsageData` inside.
  #[ doc = "bug_reproducer(BUG-229)" ]
  #[ test ]
  fn mre_bug229_strategy_metric_renew_exact_sub_shows_both_timers()
  {
    let now  = 1_700_000_000_u64;
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 30.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 50.0,
        resets_at   : Some( reset_iso_at( now, 86400 ) ),  // 7d reset in 24h
      } ),
      seven_day_sonnet : None,
    };
    let aq = AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : Some( reset_iso_at( now, 3600 ) ),  // exact sub renewal in 1h
    };

    let metric = strategy_metric( &aq, NextStrategy::Renew, PreferStrategy::Any, now );

    assert!(
      metric.contains( "7d resets in" ),
      "BUG-229: renew metric with sub must show '7d resets in': {metric}",
    );
    assert!(
      metric.contains( "renews in" ),
      "BUG-229: renew metric with exact sub must show 'renews in': {metric}",
    );
    assert!(
      !metric.contains( "~renews" ),
      "BUG-229: exact sub renewal must not have '~' prefix: {metric}",
    );
    assert!(
      !metric.contains( "session" ),
      "BUG-229: renew metric must not show session%%: {metric}",
    );
    assert!(
      !metric.contains( "5h resets" ),
      "BUG-229: renew metric must not show 5h timer: {metric}",
    );
  }

  #[ doc = "bug_reproducer(BUG-229)" ]
  #[ test ]
  fn mre_bug229_strategy_metric_renew_no_sub_shows_7d_only()
  {
    let now  = 1_700_000_000_u64;
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 30.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 50.0,
        resets_at   : Some( reset_iso_at( now, 3600 ) ),
      } ),
      seven_day_sonnet : None,
    };
    let aq = AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,  // no subscription data
    };

    let metric = strategy_metric( &aq, NextStrategy::Renew, PreferStrategy::Any, now );

    assert!(
      metric.contains( "7d resets in" ),
      "BUG-229: renew metric without sub must still show '7d resets in': {metric}",
    );
    assert!(
      !metric.contains( "renews" ),
      "BUG-229: renew metric without sub must not show 'renews': {metric}",
    );
    assert!(
      !metric.contains( "session" ),
      "BUG-229: renew metric must not show session%%: {metric}",
    );
  }

  // ── BUG-224: sort::expires and sort::renews ───────────────────────────────

  /// BUG-224: `sort::expires` sorts by `expires_at_ms` ascending — soonest expiry first.
  ///
  /// # Root Cause (BUG-224)
  /// No sort strategy exposed token expiry or billing renewal as sort dimensions; users
  /// wanting to see which accounts expire soonest had no direct way to order them.
  ///
  /// # Fix Applied
  /// `SortStrategy::Expires` arm sorts by `expires_at_ms / 1000` ascending; accounts with
  /// `expires_at_ms == 0` (unknown expiry) score `u64::MAX` and appear last.
  ///
  /// # Prevention
  /// Asserts that the account with the soonest expiry appears first and the account with
  /// no expiry data (0) appears last.
  ///
  /// # Pitfall
  /// `expires_at_ms == 0` means unknown, not epoch — treat as `u64::MAX`, not smallest.
  #[ test ]
  fn test_sort_expires_ascending()
  {
    let soon_ms   = 1_700_000_000_000_u64;  // expires soonest
    let later_ms  = 1_800_000_000_000_u64;  // expires later
    let unknown   = 0_u64;                   // unknown expiry → last

    let accounts = vec![
      mk_aq_sort( "later@test.com",   50.0, later_ms  ),
      mk_aq_sort( "unknown@test.com", 50.0, unknown   ),
      mk_aq_sort( "soon@test.com",    50.0, soon_ms   ),
    ];
    let idx = sort_indices( &accounts, SortStrategy::Expires, None, PreferStrategy::Any, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "soon@test.com",
      "sort::expires: soonest expiry must be first",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "later@test.com",
      "sort::expires: later expiry must be second",
    );
    assert_eq!(
      accounts[ idx[ 2 ] ].name, "unknown@test.com",
      "sort::expires: unknown expiry (expires_at_ms=0) must be last",
    );
  }

  /// BUG-224: `sort::renews` sorts by subscription renewal timer ascending — soonest renewal first.
  ///
  /// # Root Cause (BUG-224)
  /// See `test_sort_expires_ascending`. Billing renewal was not a sort dimension.
  ///
  /// # Fix Applied
  /// `SortStrategy::Renews` arm sorts by `renewal_secs(aq.renewal_at, org_created_at, now)`;
  /// accounts with no renewal data score `u64::MAX` and appear last.
  ///
  /// # Prevention
  /// Asserts account with soonest `renewal_at` appears first; account with no `renewal_at` last.
  ///
  /// # Pitfall
  /// `renewal_at` is an ISO-8601 string on `AccountQuota`. Use `reset_iso_at` to build
  /// deterministic timestamps relative to `now`.
  #[ test ]
  fn test_sort_renews_ascending()
  {
    let now : u64 = 1_700_000_000;
    // Account: renewal in 1h.
    let mut acct_soon = mk_aq_sort( "soon_renew@test.com", 50.0, FAR_FUTURE_MS );
    acct_soon.renewal_at = Some( reset_iso_at( now, 3_600 ) );
    // Account: renewal in 24h.
    let mut acct_later = mk_aq_sort( "later_renew@test.com", 50.0, FAR_FUTURE_MS );
    acct_later.renewal_at = Some( reset_iso_at( now, 86_400 ) );
    // Account: no renewal data.
    let acct_none = mk_aq_sort( "no_renew@test.com", 50.0, FAR_FUTURE_MS );

    let accounts = vec![ acct_later, acct_none, acct_soon ];
    let idx = sort_indices( &accounts, SortStrategy::Renews, None, PreferStrategy::Any, now );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "soon_renew@test.com",
      "sort::renews: soonest renewal must be first",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "later_renew@test.com",
      "sort::renews: later renewal must be second",
    );
    assert_eq!(
      accounts[ idx[ 2 ] ].name, "no_renew@test.com",
      "sort::renews: no renewal data must be last (scored as u64::MAX)",
    );
  }
}
