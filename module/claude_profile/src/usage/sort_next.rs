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
          ra.cmp( &rb ).then_with( || ha.total_cmp( &hb ) )
        } )
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
mod tests
{
  use super::{ find_next_for_strategy, strategy_metric };
  use crate::usage::sort::sort_indices;
  use crate::usage::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy };
  use crate::usage::test_support::
  {
    FAR_FUTURE_MS,
    mk_aq_sort, mk_aq_sort_weekly, mk_aq_with_reset, mk_aq_with_7d_reset,
    reset_iso_at,
  };

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

  /// FT-12 of feature/023 — all strategies skip `is_occupied_elsewhere` accounts.
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-12]
  #[ test ]
  fn test_ft12_023_all_strategies_skip_occupied_elsewhere()
  {
    let now = 0u64;
    // A: occupied (parked on another machine), otherwise eligible
    let mut a = mk_aq_sort( "occupied@test.com", 50.0, FAR_FUTURE_MS );
    a.is_occupied_elsewhere = true;
    // B: free, eligible
    let b = mk_aq_sort( "free@test.com", 50.0, FAR_FUTURE_MS );
    // C: current (ineligible)
    let mut c = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    c.is_current = true;

    let accounts = vec![ a, b, c ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now );
      assert_eq!(
        result, Some( 1 ),
        "{strategy:?}: must pick free@test.com (index 1), skipping occupied@test.com",
      );
    }

    // D: only occupied + current — no free candidate
    let mut a2 = mk_aq_sort( "occupied@test.com", 50.0, FAR_FUTURE_MS );
    a2.is_occupied_elsewhere = true;
    let mut c2 = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    c2.is_current = true;
    let no_free = vec![ a2, c2 ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &no_free, strategy, PreferStrategy::Any, now );
      assert!(
        result.is_none(),
        "{strategy:?}: must return None when only occupied + current remain",
      );
    }
  }

  /// FT-13 of feature/023 — all strategies skip h-exhausted accounts (5h Left ≤ 15%).
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-13]
  #[ test ]
  fn test_ft13_023_all_strategies_skip_h_exhausted()
  {
    let now = 0u64;
    // A: h-exhausted (utilization=92.0 → 8% left, well below 15%)
    let a = mk_aq_sort( "exhausted@test.com", 92.0, FAR_FUTURE_MS );
    // B: healthy (utilization=70.0 → 30% left)
    let b = mk_aq_sort( "healthy@test.com", 70.0, FAR_FUTURE_MS );
    // C: current (ineligible)
    let mut c = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    c.is_current = true;

    let accounts = vec![ a, b, c ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now );
      assert_eq!(
        result, Some( 1 ),
        "{strategy:?}: must pick healthy@test.com (index 1), skipping h-exhausted (8% left)",
      );
    }

    // Boundary: exactly at threshold (utilization=85.0 → 15% left → h-exhausted per AC-12)
    let boundary = mk_aq_sort( "boundary@test.com", 85.0, FAR_FUTURE_MS );
    let b2 = mk_aq_sort( "healthy@test.com", 70.0, FAR_FUTURE_MS );
    let mut c2 = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    c2.is_current = true;
    let boundary_accounts = vec![ boundary, b2, c2 ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &boundary_accounts, strategy, PreferStrategy::Any, now );
      assert_eq!(
        result, Some( 1 ),
        "{strategy:?}: utilization=85.0 (exactly 15% left) must be treated as h-exhausted",
      );
    }

    // D: only h-exhausted + current — no healthy candidate
    let a3 = mk_aq_sort( "exhausted@test.com", 92.0, FAR_FUTURE_MS );
    let mut c3 = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    c3.is_current = true;
    let no_healthy = vec![ a3, c3 ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &no_healthy, strategy, PreferStrategy::Any, now );
      assert!(
        result.is_none(),
        "{strategy:?}: must return None when only h-exhausted + current remain",
      );
    }
  }

  /// Corner case: `five_hour = None` → account is NOT h-exhausted (conservative: absent ≠ exhausted).
  ///
  /// All three strategies must treat missing 5h data as eligible.
  #[ test ]
  fn test_cc_023_five_hour_none_not_h_exhausted()
  {
    let now = 0u64;
    // A: five_hour = None (no 5h period data at all)
    let mut a = mk_aq_sort( "no5h@test.com", 50.0, FAR_FUTURE_MS );
    if let Ok( ref mut d ) = a.result { d.five_hour = None; }
    // B: current (ineligible)
    let mut b = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    b.is_current = true;
    let accounts = vec![ a, b ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now );
      assert_eq!(
        result, Some( 0 ),
        "{strategy:?}: five_hour=None must NOT be treated as h-exhausted",
      );
    }
  }

  /// Corner case: utilization=84.9 (just below 85.0 threshold) → account IS eligible.
  #[ test ]
  fn test_cc_023_h_exhausted_boundary_below_threshold()
  {
    let now = 0u64;
    let a = mk_aq_sort( "just_below@test.com", 84.9, FAR_FUTURE_MS );
    let mut b = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    b.is_current = true;
    let accounts = vec![ a, b ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now );
      assert_eq!(
        result, Some( 0 ),
        "{strategy:?}: utilization=84.9 (15.1% left) must be eligible — only >= 85.0 is h-exhausted",
      );
    }
  }

  /// Corner case: account is both occupied AND h-exhausted — first guard rejects it.
  #[ test ]
  fn test_cc_023_occupied_and_h_exhausted_skipped()
  {
    let now = 0u64;
    let mut a = mk_aq_sort( "both@test.com", 92.0, FAR_FUTURE_MS );
    a.is_occupied_elsewhere = true;
    let b = mk_aq_sort( "good@test.com", 50.0, FAR_FUTURE_MS );
    let mut c = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
    c.is_current = true;
    let accounts = vec![ a, b, c ];
    for strategy in [ NextStrategy::Renew, NextStrategy::Endurance, NextStrategy::Drain ]
    {
      let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now );
      assert_eq!(
        result, Some( 1 ),
        "{strategy:?}: account with both occupied + h-exhausted must be skipped",
      );
    }
  }

  // ── strategy_metric ───────────────────────────────────────────────────────

  /// FT-14 of feature/023 — endurance footer shows `session + 5h_reset`, not `7d left + expires`.
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-14]
  #[ test ]
  fn test_ft14_023_endurance_footer_shows_5h_reset()
  {
    let now : u64 = 1_700_000_000;
    // 80% session (utilization=20.0), 5h resets in 2h 30m (9000s)
    let mut aq = mk_aq_with_reset( "end@test.com", 20.0, now, 9000 );
    // Populate seven_day so the absence assertion for "90%" is meaningful even with data present
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } );
    }

    let metric = strategy_metric( &aq, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!(
      metric.contains( "80% session" ),
      "endurance metric must contain '80% session'; got: {metric}",
    );
    assert!(
      metric.contains( "5h resets in 2h 30m" ),
      "endurance metric must contain '5h resets in 2h 30m'; got: {metric}",
    );
    // Must NOT contain irrelevant weekly/expiry metrics
    assert!(
      !metric.contains( "7d left" ),
      "endurance metric must not contain '7d left'; got: {metric}",
    );
    assert!(
      !metric.contains( "expires" ),
      "endurance metric must not contain 'expires'; got: {metric}",
    );
    assert!(
      !metric.contains( "90" ),
      "endurance metric must not contain weekly pct '90'; got: {metric}",
    );
  }

  /// FT-15 of feature/023 — `next::renew` prefers lower `5h_left` account on equal renewal time.
  ///
  /// When two accounts share an identical `renewal_event_secs_of()` value, the tiebreaker must
  /// select the more session-depleted account (lower `5h_left`) — it benefits more from the same
  /// upcoming renewal event.
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-15]
  #[ test ]
  fn test_ft15_023_renew_tiebreaker_prefers_lower_5h_left()
  {
    let now_secs : u64 = 1_700_000_000;
    // Both accounts use reset_offset=10_800 (3h) → identical seven_day.resets_at → identical
    // renewal_event_secs_of() → tiebreaker fires.
    // A: util=77.0 → five_hour_left=23% (depleted — lower = preferred on tie)
    // B: util=0.0  → five_hour_left=100% (fully loaded — higher = deprioritised on tie)

    // A-first ordering: primary key ties; A wins tiebreaker (23% < 100%)
    let result_ab = find_next_for_strategy(
      &[
        mk_aq_with_7d_reset( "a@test.com", 77.0, now_secs, 10_800 ),
        mk_aq_with_7d_reset( "b@test.com", 0.0,  now_secs, 10_800 ),
      ],
      NextStrategy::Renew,
      PreferStrategy::Any,
      now_secs,
    );
    assert_eq!(
      result_ab,
      Some( 0 ),
      "A-first: must pick a@test.com (index 0, 23% 5h_left)",
    );

    // B-first ordering: .min_by_key() would return Some(0) = B (wrong — slice order wins).
    // After the fix (.min_by with composite key), must return Some(1) = A (tiebreaker fires).
    let result_ba = find_next_for_strategy(
      &[
        mk_aq_with_7d_reset( "b@test.com", 0.0,  now_secs, 10_800 ),
        mk_aq_with_7d_reset( "a@test.com", 77.0, now_secs, 10_800 ),
      ],
      NextStrategy::Renew,
      PreferStrategy::Any,
      now_secs,
    );
    assert_eq!(
      result_ba,
      Some( 1 ),
      "B-first: must pick a@test.com (index 1, 23% 5h_left) — tiebreaker fires over B (100%)",
    );
  }

  /// Corner case: endurance footer with `five_hour = None` → "0% session, 5h resets in —".
  #[ test ]
  fn test_cc_023_endurance_footer_five_hour_none()
  {
    let now : u64 = 1_700_000_000;
    let aq = AccountQuota
    {
      name              : "none5h@test.com".to_string(),
      is_current        : false,
      is_active                 : false,
      is_occupied_elsewhere     : false,
      expires_at_ms     : FAR_FUTURE_MS,
      result            : Ok( claude_quota::OauthUsageData
      {
        five_hour        : None,
        seven_day        : None,
        seven_day_sonnet : None,
      } ),
      account           : None,
      host              : String::new(),
      role              : String::new(),
      renewal_at        : None,
      cached            : false,
      cache_age_secs    : None,
    };
    let metric = strategy_metric( &aq, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!(
      metric.contains( "0% session" ),
      "five_hour=None → session_pct must be 0%; got: {metric}",
    );
    assert!(
      metric.contains( "5h resets in \u{2014}" ),
      "five_hour=None → reset must show em-dash; got: {metric}",
    );
  }

  /// Corner case: endurance footer with `five_hour.resets_at = None` → em-dash for reset timer.
  #[ test ]
  fn test_cc_023_endurance_footer_resets_at_none()
  {
    let now : u64 = 1_700_000_000;
    let aq = AccountQuota
    {
      name              : "no_reset@test.com".to_string(),
      is_current        : false,
      is_active                 : false,
      is_occupied_elsewhere     : false,
      expires_at_ms     : FAR_FUTURE_MS,
      result            : Ok( claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage { utilization : 30.0, resets_at : None } ),
        seven_day        : None,
        seven_day_sonnet : None,
      } ),
      account           : None,
      host              : String::new(),
      role              : String::new(),
      renewal_at        : None,
      cached            : false,
      cache_age_secs    : None,
    };
    let metric = strategy_metric( &aq, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!(
      metric.contains( "70% session" ),
      "utilization=30 → 70% session; got: {metric}",
    );
    assert!(
      metric.contains( "5h resets in \u{2014}" ),
      "resets_at=None → reset must show em-dash; got: {metric}",
    );
  }

  /// Corner case: endurance footer with `resets_at` in the past → `saturating_sub` gives 0 → "0m".
  #[ test ]
  fn test_cc_023_endurance_footer_resets_at_in_past()
  {
    let now : u64 = 1_700_000_000;
    // resets_at is 1000s before now
    let past_iso = reset_iso_at( 0, now - 1000 );
    let aq = AccountQuota
    {
      name              : "past@test.com".to_string(),
      is_current        : false,
      is_active                 : false,
      is_occupied_elsewhere     : false,
      expires_at_ms     : FAR_FUTURE_MS,
      result            : Ok( claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage
        {
          utilization : 40.0,
          resets_at   : Some( past_iso ),
        } ),
        seven_day        : None,
        seven_day_sonnet : None,
      } ),
      account           : None,
      host              : String::new(),
      role              : String::new(),
      renewal_at        : None,
      cached            : false,
      cache_age_secs    : None,
    };
    let metric = strategy_metric( &aq, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!(
      metric.contains( "60% session" ),
      "utilization=40 → 60% session; got: {metric}",
    );
    assert!(
      metric.contains( "5h resets in 0m" ),
      "resets_at in past → saturating_sub=0 → '0m'; got: {metric}",
    );
  }

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
        cached        : false,
        cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
      cached        : false,
      cache_age_secs : None,
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
