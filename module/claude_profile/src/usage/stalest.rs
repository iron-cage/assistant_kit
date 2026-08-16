//! Stale-first fetch-set reduction for `.usage` (TSK-499).
//!
//! `stalest::K` selects the K accounts whose quota cache is oldest and gates HTTP
//! to that subset; every other account renders from cache via the existing
//! degradation path (`approximate_quota`). `max_age::SECS` narrows eligibility to
//! accounts staler than the threshold, so a fully-fresh fleet fetches nothing.
//! Consumed by the watchdog's staggered refresh schedule (task 496).

use std::collections::HashSet;

/// Whether the stale-first reducer is active for this invocation.
///
/// `rotate::1` bypasses the reducer entirely: rotation picks a winner from a
/// complete fresh ranking and must never run on staggered stale data (rotation
/// freshness contract — tasks 499/496). Validation has already rejected
/// `stalest::0` as an explicit value, so `stalest == 0` here means "absent".
#[ must_use ]
#[ inline ]
pub fn reduction_applies( stalest : u32, rotate : bool ) -> bool
{
  stalest > 0 && !rotate
}

/// Select the fetch set: the `k` accounts with the oldest cache `fetched_at`.
///
/// Age ranking:
/// - Missing cache, or an unparseable `fetched_at`, ranks infinitely stale
///   (`u64::MAX`) — an account without a usable cache should be refreshed first.
///   This deliberately differs from `fetch_cache.rs`'s `unwrap_or( now )` (age 0):
///   there a broken timestamp must not masquerade as fresh data for display;
///   here it must not hide an account from refresh.
/// - With `max_age > 0`, only accounts strictly staler than `max_age` seconds are
///   eligible; the result may then hold fewer than `k` names (possibly zero).
/// - Equal ages tie-break by original list position, keeping repeated invocations
///   deterministic for a stable account list.
#[ must_use ]
#[ inline ]
pub fn select_stalest(
  accounts         : &[ crate::account::Account ],
  credential_store : &std::path::Path,
  k                : u32,
  max_age          : u64,
  now_secs         : u64,
) -> HashSet< String >
{
  let mut ranked : Vec< ( usize, u64, &str ) > = accounts
    .iter()
    .enumerate()
    .map( |( idx, acct )|
    {
      let age = claude_profile_core::account::read_quota_cache( credential_store, &acct.name )
        .and_then( | entry | claude_profile_core::account::parse_iso_utc_secs( &entry.fetched_at ) )
        .map_or( u64::MAX, | then | now_secs.saturating_sub( then ) );
      ( idx, age, acct.name.as_str() )
    } )
    .collect();
  if max_age > 0
  {
    ranked.retain( |( _, age, _ )| *age > max_age );
  }
  ranked.sort_unstable_by( | a, b | b.1.cmp( &a.1 ).then( a.0.cmp( &b.0 ) ) );
  ranked.into_iter().take( usize::try_from( k ).unwrap_or( usize::MAX ) ).map( |( _, _, name )| name.to_string() ).collect()
}
