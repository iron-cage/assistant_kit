//! Sort strategies for the quota table.
//!
//! `sort_indices` is the core sort function. Recommendation strategies
//! (`find_next_for_strategy`, `strategy_metric`) live in `sort_next`.

pub( crate ) use super::sort_next::{ find_next_for_strategy, strategy_metric };

use super::types::{ AccountQuota, SortStrategy, PreferStrategy };
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
        cached        : false,
        cache_age_secs : None,
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

}
