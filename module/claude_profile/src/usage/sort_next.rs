// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
//! Next-account recommendation strategies.
//!
//! `find_next_for_strategy` and `strategy_metric` are consumed by `render.rs`
//! for the footer recommendation block. `find_first_eligible` is a private helper.

use crate::output::format_duration_secs;
use crate::account::{ TagFilter, eligible };
use super::sort::sort_indices;
use super::types::
{
  AccountQuota, SortStrategy, PreferStrategy,
  WEEKLY_EXHAUSTION_THRESHOLD, H_EXHAUSTED_THRESHOLD, ROTATION_HEADROOM_THRESHOLD,
};
use super::format::{ five_hour_left, seven_day_left, renewal_secs, next_event_raw };

// ── Next-account recommendation ───────────────────────────────────────────────

/// Return the first eligible (non-current, non-active, non-occupied, non-h-exhausted,
/// non-expired, `Ok`) account from a pre-sorted index slice that also satisfies `extra`,
/// or `None` when none exist.
fn find_first_eligible< F >(
  accounts          : &[ AccountQuota ],
  sorted            : &[ usize ],
  now_secs          : u64,
  selected_provider : &str,
  tag_filter        : &TagFilter,
  extra             : F,
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
    // Fix(BUG-557): shared predicate, not the re-derived literal — see `status_emoji`. The
    //   original pitfall above still holds and is now enforced inside the predicate; what
    //   changed is that a *persisted* verdict is no longer ambiguous the way `account=None`
    //   is. Left as the literal, rotation could elect an account it knows is dead.
    //   `is_dead_account()` keeps this gate `result`-independent (BUG-317).
    if aq.is_dead_account() { continue; }
    if aq.result.is_err() { continue; }
    // Fix(audit-h-exhaustion-drift): use the canonical rounded five_hour_left() against
    //   H_EXHAUSTED_THRESHOLD instead of raw `utilization >= 85.0`.
    // Root cause: the complement literal duplicated the threshold (types.rs forbids this)
    //   and compared un-rounded utilization — an 84.6% account displayed as "15% left"
    //   (h-exhausted everywhere else, BUG-331/BUG-336 round-before-compare doctrine) was
    //   still eligible here.
    // Pitfall: eligibility and display must round the same value against the same
    //   constant; sort.rs:51 is the sibling call site this now mirrors.
    if five_hour_left( aq ) <= H_EXHAUSTED_THRESHOLD { continue; }
    if ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) == 0 { continue; }
    // Gate 9 (Claim-locked): unconditional — no force::1 bypass at the eligibility layer,
    // unlike the force-bypassable G9 explicit-command gate on .account.use/assignee:: (Feature 070).
    if aq.claim_lock { continue; }
    // Gate 10 (Provider mismatch): unconditional — no force::1 bypass, mirroring Gate 9.
    // Empty AccountQuota.inference_provider and a default-absent selected_provider both
    // resolve to "anthropic" before comparing (never treated as a wildcard/always-match).
    let effective = if aq.inference_provider.is_empty() { "anthropic" } else { &aq.inference_provider };
    if effective != selected_provider { continue; }
    // Gate 11 (Identity tag filter): unconditional — no force::1 bypass, mirroring Gates 9/10.
    // A default (permit-all) TagFilter passes every account, tagged or not (Feature 076).
    if !eligible( &aq.tags, tag_filter ) { continue; }
    if !extra( aq ) { continue; }
    return Some( idx );
  }
  None
}

// Fix(BUG-558): the weekly gate every strategy applied was `> WEEKLY_EXHAUSTION_THRESHOLD`
//   — a 3% floor. An account 2 points above it with the fleet's earliest 7d reset won
//   `renew` outright, and rotation landed on an account with nine minutes of life left.
// Root cause: the *exclusion* boundary ("is this account spent?") was doing duty as the
//   *selection* bar ("is this account worth switching to?"). Those are different questions
//   and the floor only ever answered the first.
// Pitfall: two passes, not one raised threshold. The second pass is what keeps a
//   fully-depleted fleet rotating at all — collapsing this to a single
//   `> ROTATION_HEADROOM_THRESHOLD` gate would make `rotate::1` report BUG-529's
//   "no eligible account to rotate to" (038/AC-03) whenever every account sat below 15%,
//   turning a degraded-but-working fleet into a hard failure.
/// First eligible account preferring comfortable weekly headroom, falling back to the
/// bare exhaustion floor.
///
/// Pass 1 demands `seven_day_left > ROTATION_HEADROOM_THRESHOLD`; pass 2 repeats the
/// search at `> WEEKLY_EXHAUSTION_THRESHOLD` and runs only when pass 1 found nothing.
/// Both passes walk the same pre-sorted index slice, so the caller's strategy ordering is
/// preserved exactly — headroom filters the candidate set, it never reorders it.
fn find_preferring_headroom(
  accounts          : &[ AccountQuota ],
  sorted            : &[ usize ],
  now_secs          : u64,
  selected_provider : &str,
  tag_filter        : &TagFilter,
  gate_ownership    : bool,
) -> Option< usize >
{
  let owned_ok = | aq : &AccountQuota | !gate_ownership || aq.is_owned;
  let find = | floor : f64 | find_first_eligible(
    accounts, sorted, now_secs, selected_provider, tag_filter,
    | aq | seven_day_left( aq ) > floor && owned_ok( aq ),
  );
  find( ROTATION_HEADROOM_THRESHOLD ).or_else( || find( WEEKLY_EXHAUSTION_THRESHOLD ) )
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
  accounts          : &[ AccountQuota ],
  strategy          : SortStrategy,
  prefer            : PreferStrategy,
  now_secs          : u64,
  gate_ownership    : bool,
  selected_provider : &str,
  tag_filter        : &TagFilter,
) -> Option< usize >
{
  match strategy
  {
    SortStrategy::Name =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Name, None, prefer, now_secs );
      find_preferring_headroom( accounts, &sorted, now_secs, selected_provider, tag_filter, gate_ownership )
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
      find_preferring_headroom( accounts, &sorted, now_secs, selected_provider, tag_filter, gate_ownership )
    }
    SortStrategy::Renews =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Renews, None, prefer, now_secs );
      find_preferring_headroom( accounts, &sorted, now_secs, selected_provider, tag_filter, gate_ownership )
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
      // Fix(BUG-341): read the top-level org_created_at field, not the account-gated path.
      // Root cause: aq.account is None for cache-refreshed accounts on 3 of 4 fetch branches
      //   (docs/feature/033_quota_cache.md), silently dropping the estimate to u64::MAX even
      //   when the top-level AccountQuota.org_created_at (BUG-327/TSK-368) carries the data.
      // Pitfall: render.rs's 7 call sites already read aq.org_created_at.as_deref() directly —
      //   this file was one of the two remaining holdouts of the stale gated pattern.
      let sub_pair = renewal_secs(
        aq.renewal_at.as_deref(),
        aq.org_created_at.as_deref(),
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
      // Fix(BUG-341): read the top-level org_created_at field, not the account-gated path.
      // Root cause: aq.account is None for cache-refreshed accounts on 3 of 4 fetch branches
      //   (docs/feature/033_quota_cache.md), silently dropping the estimate to u64::MAX even
      //   when the top-level AccountQuota.org_created_at (BUG-327/TSK-368) carries the data.
      // Pitfall: render.rs's 7 call sites already read aq.org_created_at.as_deref() directly —
      //   this file was one of the two remaining holdouts of the stale gated pattern.
      let sub_pair = renewal_secs(
        aq.renewal_at.as_deref(),
        aq.org_created_at.as_deref(),
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
