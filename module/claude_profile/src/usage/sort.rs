// Items are pub for test_bridge re-export; missing_inline suppressed — sort_indices is too large to inline.
#![ allow( clippy::missing_inline_in_public_items ) ]
//! Sort strategies for the quota table.
//!
//! `sort_indices` is the core sort function. Recommendation strategies
//! (`find_next_for_strategy`, `strategy_metric`) live in `sort_next`.
//!
//! **4-group status partition** (AC-12 / `020_usage_sort_strategies.md`):
//! All accounts are partitioned into 4 fixed status groups before any sort strategy
//! applies. Group order is always: 🟢 Green → 🟡 h-exhausted → 🟡 weekly-exhausted → 🔴 Red.
//! `desc::` reverses row order within each group; group order is never reversed.

pub( crate ) use super::sort_next::{ find_next_for_strategy, strategy_metric };

use super::types::{ AccountQuota, SortStrategy, PreferStrategy, H_EXHAUSTED_THRESHOLD, WEEKLY_EXHAUSTION_THRESHOLD };
use super::format::{ five_hour_left, prefer_weekly, seven_day_left, renewal_secs };

// ── Status group ──────────────────────────────────────────────────────────────

/// Four status groups for the quota table (AC-12, see `020_usage_sort_strategies.md`).
///
/// Variants are ordered — lower discriminant = higher position in table.
#[ derive( Debug, PartialEq, Eq, PartialOrd, Ord ) ]
pub enum StatusGroup
{
  /// Both windows available: 5h > 15%, 7d > 5%.
  Green,
  /// 5h exhausted (≤ 15%), 7d still available (> 5%).
  HExhausted,
  /// 7d exhausted (≤ 5%); includes both-exhausted (Fix BUG-321).
  WeeklyExhausted,
  /// Error result or cancelled subscription (`billing_type="none"`).
  Red,
}

/// Assign an account to its status group.
#[ must_use ]
pub fn status_group_of( aq : &AccountQuota ) -> StatusGroup
{
  if aq.result.is_err() { return StatusGroup::Red; }
  // Fix(BUG-317): cancelled subscription is permanently unusable → Red regardless of quota.
  // Root cause: billing_type was not checked; accounts with billing_type="none" appeared 🟡
  //   or 🟢 based solely on quota, hiding permanently dead status from the user.
  // Pitfall: account may be None (account-API fetch failed) — only classify Red when
  //   account data is present and billing_type is definitively "none". Absent data is
  //   ambiguous; do not penalize it.
  if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" )
  {
    return StatusGroup::Red;
  }
  let h5_ok = five_hour_left( aq ) > H_EXHAUSTED_THRESHOLD;
  // Fix(BUG-299): use raw seven_day_left for d7_ok — group boundaries are model-agnostic per AC-12.
  // Root cause: prefer_weekly(any) = min(7d, 7d_son) can be ≤ 5.0 when 7d_son ≤ 5% even if
  //   seven_day_left > 5%, misclassifying h-exhausted accounts as Red instead of HExhausted.
  // Pitfall: prefer_weekly is correct for sort::renew tiebreak;
  //   wrong for group boundary predicates and eligibility gates — always use raw single-metric functions.
  let d7_ok = seven_day_left( aq ) > WEEKLY_EXHAUSTION_THRESHOLD;
  match ( h5_ok, d7_ok )
  {
    ( true,  true  ) => StatusGroup::Green,
    ( false, true  ) => StatusGroup::HExhausted,
    // Fix(BUG-321): both-exhausted (5h ≤ 15% AND 7d ≤ 5%) → G3 WeeklyExhausted, not G4 Red.
    // 7d is the binding constraint: when 7d resets, 5h will have long since reset.
    // Recovery is identical to single-weekly-exhausted — no separate group needed.
    // Dead classification (result=Err / billing_type="none") fires BEFORE this match.
    // Root cause: BUG-319 fix added `(false,false)→Red` with incorrect premise that
    //   both-exhausted = dead; `result=Ok` with depleted quota is recoverable, not dead.
    // Pitfall: Dead (G4) is exclusively for unrecoverable states — never use quota
    //   threshold patterns to classify accounts as Red.
    ( _, false ) => StatusGroup::WeeklyExhausted,
  }
}

// ── Sort ──────────────────────────────────────────────────────────────────────

/// Return indices into `accounts` sorted by `strategy` and `desc`.
///
/// Each strategy has a canonical direction (its `default_desc()`). Passing
/// `desc = Some(true)` inverts the within-group sort order.
///
/// Accounts are first partitioned into 4 status groups (Green → `HExhausted` →
/// `WeeklyExhausted` → Red). The chosen strategy sorts within each group.
/// `desc::` reverses the within-group sort only — group order is always fixed.
///
/// See `docs/feature/020_usage_sort_strategies.md` for full algorithm specs.
#[ must_use ]
pub fn sort_indices(
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

  // Partition into 4 status groups. Sort within each group by strategy.
  // Fix(BUG-259): name as final tiebreaker makes sort deterministic when numeric keys tie.
  // Root cause: without name tiebreaker, row order depended on filesystem read_dir order.
  // Pitfall: partition() preserves insertion order, not sort order — always sort partitions.
  let mut groups : [ Vec< usize >; 4 ] = [ vec![], vec![], vec![], vec![] ];
  for ( i, aq ) in accounts.iter().enumerate()
  {
    let g = match status_group_of( aq )
    {
      StatusGroup::Green           => 0,
      StatusGroup::HExhausted      => 1,
      StatusGroup::WeeklyExhausted => 2,
      StatusGroup::Red             => 3,
    };
    groups[ g ].push( i );
  }

  match strategy
  {
    SortStrategy::Name =>
    {
      for group in &mut groups
      {
        group.sort_by( |&a, &b| accounts[ a ].name.cmp( &accounts[ b ].name ) );
        if reversed { group.reverse(); }
      }
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

      // Canonical: ascending min(7d_reset, sub_renewal) (soonest event first); tiebreak ascending prefer_weekly, then name.
      // Pitfall: sorts on floating-point (prefer_weekly) require unwrap_or for NaN handling — never use cmp directly.
      for group in &mut groups
      {
        group.sort_by( |&a, &b|
        {
          renewal_event_secs( a ).cmp( &renewal_event_secs( b ) )
            .then_with( ||
            {
              let wa = prefer_weekly( &accounts[ a ], prefer );
              let wb = prefer_weekly( &accounts[ b ], prefer );
              wa.partial_cmp( &wb ).unwrap_or( core::cmp::Ordering::Equal )
            } )
            .then_with( || accounts[ a ].name.cmp( &accounts[ b ].name ) )
        } );
        if reversed { group.reverse(); }
      }
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

      for group in &mut groups
      {
        group.sort_by( |&a, &b|
          renews_secs_of( a ).cmp( &renews_secs_of( b ) )
            .then_with( || accounts[ a ].name.cmp( &accounts[ b ].name ) )
        );
        if reversed { group.reverse(); }
      }
    }
  }

  // Flatten groups in fixed order (Green → HExhausted → WeeklyExhausted → Red).
  let mut result = Vec::with_capacity( accounts.len() );
  for group in groups { result.extend( group ); }
  result
}


// Tests live in tests/usage/sort_tests.rs (integration tests via test_bridge).
