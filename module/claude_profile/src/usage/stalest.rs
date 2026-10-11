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

// Fix(BUG-559): every slot this reducer handed out went to an account that could not use
//   it, so a fleet on `stalest::K` refreshed nothing at all and its live accounts sat
//   frozen for days. Two distinct starvation sources, one per patch below:
//   1. A redirect-backend account has no Anthropic quota to fetch, so it never writes a
//      cache, so it ranks `u64::MAX` forever and wins slot 1 on every single tick.
//   2. A permanently dead account's fetch always fails, so `write_quota_cache` — reachable
//      only on success — never advances its `fetched_at`, so it stays the stalest live
//      candidate forever and wins slot 2 on every single tick.
// Root cause shared by both: staleness was defined as "time since last SUCCESS", which for
//   an account that can never succeed is unbounded and monotonically increasing. A ranking
//   meant to spread refreshes evenly instead pinned itself to exactly the accounts that
//   could not benefit, and starved the ones that could.
// Pitfall: (1) is an exclusion, (2) is a re-definition — do not collapse them. Excluding
//   dead accounts too would strand them: nothing would ever re-fetch one, so a resubscribed
//   account could never be discovered alive again. Ranking by last *attempt* keeps them in
//   the rotation at a fair cadence instead of at the front of the queue forever.
/// Select the fetch set: the `k` accounts whose last fetch *attempt* is oldest.
///
/// Age ranking:
/// - Ranked by `max(fetched_at, last_error_at, last_attempt_at)` — the last attempt, not the
///   last success. Every failed live fetch stamps `last_attempt_at` (BUG-591), so an account
///   that fails every time still has its turn come round, but does not monopolise the fetch
///   set by never advancing its success timestamp.
/// - Redirect-backend accounts (Feature 071) are excluded outright: they have no Anthropic
///   quota to fetch, so a slot spent on one produces nothing on every tick, forever.
/// - Only accounts the fetch loop's gates pass are ranked at all. `passes_fetch_gates` drops a
///   `backend: redirect` account, one another machine owns, any account but the live session
///   here under `solo::1`, and one occupied on another machine unless it's the live session
///   here. A slot spent on any of them refreshes nothing, because skipped picks are never
///   backfilled (BUG-583).
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
  live_creds_file  : &std::path::Path,
  solo             : bool,
  k                : u32,
  max_age          : u64,
  now_secs         : u64,
) -> HashSet< String >
{
  // Fix(BUG-583): rank only accounts the fetch loop's gates pass. A pick that R1, G1, the solo
  //   gate or G1b skips refreshes nothing, and the loop never hands its slot to the next account.
  // Root cause: eligibility was decided twice, here on `inference_provider` alone and in the
  //   fetch loop on backend, owner, `solo::1` and occupancy, and the two disagreed.
  // Pitfall: a new skip gate in `fetch_quota_for_list` belongs in `passes_fetch_gates` too,
  //   or its picks waste slots again.
  let occupied_elsewhere = crate::account::other_machines_active( credential_store );
  let live_token         = super::fetch::read_live_token( live_creds_file );
  let mut ranked : Vec< ( usize, u64, &str ) > = accounts
    .iter()
    .enumerate()
    // Gate 10's own convention: an empty `inference_provider` means "anthropic", never a
    // wildcard. Only a non-anthropic backend is genuinely quota-less and excludable.
    .filter( | ( _, acct ) | acct.inference_provider.is_empty() || acct.inference_provider == "anthropic" )
    .filter( | ( _, acct ) |
    {
      super::fetch::passes_fetch_gates(
        acct, credential_store, &occupied_elsewhere, live_token.as_ref(), solo,
      )
    } )
    .map( |( idx, acct )|
    {
      // Fix(BUG-591): rank on `last_attempt_at` too, the stamp every failed live fetch leaves.
      // Root cause: `fetched_at` moves only on success and `last_error_at` only on the Class-A
      //   verdict, so an account failing any other way kept its old success as its age.
      // Pitfall: `max` of all three, never "prefer the stamp" — an old stamp read from another
      //   host's file must lose to a newer success, as an old `last_error_at` does (BUG-559).
      let age = claude_profile_core::account::read_quota_cache( credential_store, &acct.name )
        .and_then( | entry |
        {
          use claude_profile_core::account::parse_iso_utc_secs as parse;
          let ok      = parse( &entry.fetched_at );
          let fail    = entry.last_error_at.as_deref().and_then( parse );
          let attempt = entry.last_attempt_at.as_deref().and_then( parse );
          // Newest of the three instants — each one is an attempt that has already run.
          ok.max( fail ).max( attempt )
        } )
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

/// The `trace::1` line naming the set `select_stalest` chose, without the timestamp prefix:
/// `stalest  selected (k=K, max_age=SECSs): a, b`.
///
/// Names sort alphabetically, so the same set always traces the same way. An empty set
/// traces as `none`, so a tick with nothing staler than `max_age` says so outright.
// implicit_hasher: pub only for the test bridge — every caller passes the default-hasher set
// from select_stalest, as with fetch_quota_for_list.
#[ allow( clippy::implicit_hasher ) ]
#[ must_use ]
#[ inline ]
pub fn selection_trace( selected : &HashSet< String >, k : u32, max_age : u64 ) -> String
{
  let mut names : Vec< &str > = selected.iter().map( String::as_str ).collect();
  names.sort_unstable();
  let names = if names.is_empty() { "none".to_string() } else { names.join( ", " ) };
  format!( "stalest  selected (k={k}, max_age={max_age}s): {names}" )
}
