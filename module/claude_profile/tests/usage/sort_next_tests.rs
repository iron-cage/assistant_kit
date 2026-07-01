// Integration tests for sort_next.rs — relocated from src/usage/sort_next_tests.rs.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::find_next_for_strategy;
use claude_profile::usage::test_bridge::sort_indices;
use claude_profile::usage::test_bridge::types::{ SortStrategy, PreferStrategy };
use claude_profile::usage::test_bridge::
{
  FAR_FUTURE_MS,
  mk_aq_sort, mk_aq_with_7d_reset,
  reset_iso_at,
};

// ── find_next_for_strategy ────────────────────────────────────────────────

/// `find_next_for_strategy` returns `Some` when eligible candidates exist; `None` when all accounts are `is_current`.
#[ test ]
fn test_find_next_for_strategy_some_when_eligible_none_when_all_current()
{
  let now = 0u64;
  let mut a = mk_aq_sort( "a@test.com", 20.0, FAR_FUTURE_MS );
  a.is_current = true;
  let b = mk_aq_sort( "b@test.com", 30.0, FAR_FUTURE_MS );  // 70% left
  let c = mk_aq_sort( "c@test.com", 60.0, FAR_FUTURE_MS );  // 40% left
  let accounts = vec![ a, b, c ];

  let winner_a = find_next_for_strategy( &accounts, SortStrategy::Name, PreferStrategy::Any, now, false );
  assert!( winner_a.is_some(), "find_next_for_strategy must return Some when eligible candidates exist" );
  let winner_idx = winner_a.unwrap();
  assert_eq!(
    accounts[ winner_idx ].name, "b@test.com",
    "name strategy winner must be b@test.com (first eligible alphabetically); got index {winner_idx}",
  );

  let mut a2 = mk_aq_sort( "a@test.com", 20.0, FAR_FUTURE_MS );
  let mut b2 = mk_aq_sort( "b@test.com", 30.0, FAR_FUTURE_MS );
  let mut c2 = mk_aq_sort( "c@test.com", 60.0, FAR_FUTURE_MS );
  a2.is_current = true;
  b2.is_current = true;
  c2.is_current = true;
  let all_current = vec![ a2, b2, c2 ];

  let winner_b = find_next_for_strategy( &all_current, SortStrategy::Name, PreferStrategy::Any, now, false );
  assert!( winner_b.is_none(), "find_next_for_strategy must return None when all accounts are is_current" );
}

/// All strategies skip `is_occupied_elsewhere` accounts.
#[ test ]
fn test_all_strategies_skip_occupied_elsewhere()
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
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
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
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &no_free, strategy, PreferStrategy::Any, now, false );
    assert!(
      result.is_none(),
      "{strategy:?}: must return None when only occupied + current remain",
    );
  }
}

/// All strategies skip h-exhausted accounts (5h Left ≤ 15%).
#[ test ]
fn test_all_strategies_skip_h_exhausted()
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
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
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
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &boundary_accounts, strategy, PreferStrategy::Any, now, false );
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
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &no_healthy, strategy, PreferStrategy::Any, now, false );
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
fn test_cc_five_hour_none_not_h_exhausted()
{
  let now = 0u64;
  // A: five_hour = None (no 5h period data at all)
  let mut a = mk_aq_sort( "no5h@test.com", 50.0, FAR_FUTURE_MS );
  if let Ok( ref mut d ) = a.result { d.five_hour = None; }
  // B: current (ineligible)
  let mut b = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
  b.is_current = true;
  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 0 ),
      "{strategy:?}: five_hour=None must NOT be treated as h-exhausted",
    );
  }
}

/// Corner case: utilization=84.9 (just below 85.0 threshold) → account IS eligible.
#[ test ]
fn test_cc_h_exhausted_boundary_below_threshold()
{
  let now = 0u64;
  let a = mk_aq_sort( "just_below@test.com", 84.9, FAR_FUTURE_MS );
  let mut b = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
  b.is_current = true;
  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 0 ),
      "{strategy:?}: utilization=84.9 (15.1% left) must be eligible — only >= 85.0 is h-exhausted",
    );
  }
}

/// Corner case: account is both occupied AND h-exhausted — first guard rejects it.
#[ test ]
fn test_cc_occupied_and_h_exhausted_skipped()
{
  let now = 0u64;
  let mut a = mk_aq_sort( "both@test.com", 92.0, FAR_FUTURE_MS );
  a.is_occupied_elsewhere = true;
  let b = mk_aq_sort( "good@test.com", 50.0, FAR_FUTURE_MS );
  let mut c = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
  c.is_current = true;
  let accounts = vec![ a, b, c ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 1 ),
      "{strategy:?}: account with both occupied + h-exhausted must be skipped",
    );
  }
}

/// Corner case: `is_active = true` — account active on this machine (not current session) is skipped.
///
/// Gate 1 of `find_first_eligible`: `if aq.is_current || aq.is_active { continue; }`.
/// Existing tests only exercise `is_current = true`; this covers the `is_active = true` branch
/// independently (a logged-in account on this machine that is not the current active session).
#[ test ]
fn test_cc_is_active_skips_account()
{
  let now = 0u64;
  // A: active on this machine (not the current session) → gate 1 fires via is_active branch
  let mut a = mk_aq_sort( "active@test.com", 50.0, FAR_FUTURE_MS );
  a.is_active = true;
  // B: free, eligible
  let b = mk_aq_sort( "free@test.com", 50.0, FAR_FUTURE_MS );

  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 1 ),
      "{strategy:?}: is_active=true must be skipped; free@test.com (index 1) must be selected",
    );
  }

  // All active — no eligible candidate remains
  let mut a2 = mk_aq_sort( "active@test.com", 50.0, FAR_FUTURE_MS );
  a2.is_active = true;
  let all_active = vec![ a2 ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &all_active, strategy, PreferStrategy::Any, now, false );
    assert!( result.is_none(), "{strategy:?}: all-active must return None" );
  }
}

/// Corner case: expired token gate — `Ok(data)` account with past `expires_at_ms` is skipped.
///
/// Gate 5 of `find_first_eligible`:
///   `if ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) == 0 { continue; }`.
/// Fires when `expires_at_ms / 1000 ≤ now_secs`. Distinct from gate 3 (`result.is_err()`):
/// account has valid quota data but a stale credential token.
///
/// Boundary: `expires_at_ms / 1000 == now_secs` → 0 secs remaining → expired.
/// One-past: `expires_at_ms / 1000 == now_secs + 1` → 1 sec remaining → eligible.
#[ test ]
fn test_cc_expired_ok_account_skipped()
{
  let now_secs : u64 = 2_000;
  // A: Ok data, token expired (1000 ms → 1 sec ≤ now=2000) → gate 5 skips it
  let a = mk_aq_sort( "expired@test.com", 50.0, 1_000 );
  // B: valid token → eligible
  let b = mk_aq_sort( "valid@test.com", 50.0, FAR_FUTURE_MS );

  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now_secs, false );
    assert_eq!(
      result, Some( 1 ),
      "{strategy:?}: expired Ok account must be skipped; valid@test.com (index 1) must win",
    );
  }

  // Boundary: expires_at_ms / 1000 == now_secs → saturating_sub == 0 → still expired
  let at_boundary = mk_aq_sort( "boundary@test.com", 50.0, now_secs * 1000 );
  let accounts_boundary = vec![ at_boundary ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts_boundary, strategy, PreferStrategy::Any, now_secs, false );
    assert!(
      result.is_none(),
      "{strategy:?}: boundary-expired account (0 secs remaining) must be skipped",
    );
  }

  // One-past boundary: expires_secs == now_secs + 1 → 1 sec remaining → eligible
  let one_sec_left = mk_aq_sort( "one_sec@test.com", 50.0, ( now_secs + 1 ) * 1000 );
  let accounts_valid = vec![ one_sec_left ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts_valid, strategy, PreferStrategy::Any, now_secs, false );
    assert_eq!(
      result, Some( 0 ),
      "{strategy:?}: account with 1 second remaining must be eligible",
    );
  }
}

/// Corner case: `gate_ownership = true` — non-owned accounts are skipped.
///
/// `extra` predicate: `seven_day_left(aq) > WEEKLY_EXHAUSTION_THRESHOLD && ( !gate_ownership || aq.is_owned )`.
/// When `gate_ownership = true`, the `aq.is_owned` check must pass. All existing tests pass
/// `gate_ownership = false` (bypassing this check). This test exercises the ownership-gate path.
#[ test ]
fn test_cc_gate_ownership_rejects_non_owned()
{
  let now = 0u64;
  // A: not owned, alphabetically first — must be tried first and skipped under gate_ownership=true
  // (using "aaa_" prefix to guarantee it sorts before "zzz_" regardless of strategy tiebreaker)
  let mut a = mk_aq_sort( "aaa_unowned@test.com", 50.0, FAR_FUTURE_MS );
  a.is_owned = false;
  // B: owned, alphabetically second — selected only when unowned is properly rejected
  let b = mk_aq_sort( "zzz_owned@test.com", 50.0, FAR_FUTURE_MS );

  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    // gate_ownership=true: aaa_unowned is tried first (alphabetically), rejected by is_owned gate
    let with_gate = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, true );
    assert_eq!(
      with_gate, Some( 1 ),
      "{strategy:?} gate_ownership=true: aaa_unowned (is_owned=false) must be skipped; zzz_owned (index 1) must win",
    );

    // gate_ownership=false: aaa_unowned (index 0) is selected — ownership gate not enforced
    let no_gate = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      no_gate, Some( 0 ),
      "{strategy:?} gate_ownership=false: aaa_unowned (is_owned=false, index 0) must be selected \
       — gate disabled means unowned accounts pass; if guard were absent, gate_ownership=true \
       would also return Some(0) instead of Some(1) above",
    );
  }

  // Only non-owned with gate_ownership=true — no eligible candidate
  let mut a2 = mk_aq_sort( "aaa_unowned@test.com", 50.0, FAR_FUTURE_MS );
  a2.is_owned = false;
  let all_unowned = vec![ a2 ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &all_unowned, strategy, PreferStrategy::Any, now, true );
    assert!(
      result.is_none(),
      "{strategy:?}: all-non-owned with gate_ownership=true must return None",
    );
  }
}

/// Corner case: `result = Err(...)` account is skipped via `find_next_for_strategy` integration.
///
/// Gate 3 of `find_first_eligible`: `let Ok( data ) = &aq.result else { continue; }`.
/// The error account is named "aaa_" so all sort strategies try it first; if gate 3 were
/// removed, the error account would be returned as `Some(0)` instead of falling through to
/// the valid account at index 1.
#[ test ]
fn test_cc_err_account_skipped_via_find_next_for_strategy()
{
  let now = 0u64;
  // A: Err account (named "aaa_" to sort first under all strategies)
  let mut a = mk_aq_sort( "aaa_error@test.com", 50.0, FAR_FUTURE_MS );
  a.result = Err( "missing accessToken".to_string() );
  // B: Ok account — eligible fallback
  let b = mk_aq_sort( "zzz_valid@test.com", 50.0, FAR_FUTURE_MS );

  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 1 ),
      "{strategy:?}: Err account (aaa_error, index 0) must be skipped by gate 3; zzz_valid (index 1) must win",
    );
  }
}

/// Corner case: expired `Ok` account is rejected even with `gate_ownership = true`.
///
/// Gate 5 (token expiry) fires before Gate 6 (`extra` / ownership check). Verifies gate
/// ordering: an owned account with a stale token is still rejected, and the valid-token
/// account is selected even though the ownership gate is also active.
#[ test ]
fn test_cc_expired_ok_with_gate_ownership_true()
{
  let now_secs : u64 = 2_000;
  // A: Ok data, owned, token expired — gate 5 must reject before gate 6 evaluates
  // (named "aaa_" to sort first so the gate ordering is actually exercised)
  let a = mk_aq_sort( "aaa_expired_owned@test.com", 50.0, 1_000 );  // expires at 1s < now=2000s
  // B: valid token, owned — eligible under gate_ownership=true
  let b = mk_aq_sort( "zzz_valid_owned@test.com", 50.0, FAR_FUTURE_MS );

  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now_secs, true );
    assert_eq!(
      result, Some( 1 ),
      "{strategy:?} gate_ownership=true: expired owned account (gate 5) must be skipped; zzz_valid_owned (index 1) must win",
    );
  }
}

/// Corner case: `five_hour = None` passes gate 4 even when `gate_ownership = true`.
///
/// Gate 4: `data.five_hour.as_ref().is_some_and( |p| p.utilization >= 85.0 )`.
/// `None.is_some_and(...)` = false → not skipped. With `gate_ownership = true` and
/// `aq.is_owned = true`, the account should pass both gate 4 and gate 6.
#[ test ]
fn test_cc_five_hour_none_with_gate_ownership_true()
{
  let now = 0u64;
  // A: five_hour=None, is_owned=true — must pass gate 4 (not h-exhausted) and gate 6 (owned)
  let mut a = mk_aq_sort( "no5h_owned@test.com", 50.0, FAR_FUTURE_MS );
  if let Ok( ref mut d ) = a.result { d.five_hour = None; }
  // B: current (ineligible, provides only contrast)
  let mut b = mk_aq_sort( "current@test.com", 50.0, FAR_FUTURE_MS );
  b.is_current = true;

  let accounts = vec![ a, b ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, true );
    assert_eq!(
      result, Some( 0 ),
      "{strategy:?}: five_hour=None with gate_ownership=true must be eligible (owned, not h-exhausted)",
    );
  }
}

// ── strategy_metric ───────────────────────────────────────────────────────

/// `sort::renew` uses alphabetical name as tiebreaker on equal renewal time (BUG-260/BUG-291).
///
/// When two accounts share an identical `renewal_event_secs_of()` value and identical
/// `prefer_weekly`, the tiebreaker resolves alphabetically by name — same determinism rule
/// as `sort_indices(Renew)`.
///
/// Spec: [`docs/feature/020_usage_sort_strategies.md` AC-04]
#[ test ]
fn test_sort_renew_tiebreaker_alphabetical_when_equal_renewal()
{
  let now_secs : u64 = 1_700_000_000;
  // Both accounts use reset_offset=10_800 (3h) → identical seven_day.resets_at → identical
  // renewal_event_secs_of() → tiebreaker fires.
  // mk_aq_with_7d_reset sets seven_day.util=0 for both → prefer_weekly tied → name decides.
  // A: "a@test.com" < "b@test.com" alphabetically → a wins both orderings.

  // A-first ordering: primary key ties; A wins (alphabetically first)
  let result_ab = find_next_for_strategy(
    &[
      mk_aq_with_7d_reset( "a@test.com", 77.0, now_secs, 10_800 ),
      mk_aq_with_7d_reset( "b@test.com", 0.0,  now_secs, 10_800 ),
    ],
    SortStrategy::Renew,
    PreferStrategy::Any,
    now_secs,
    false,
  );
  assert_eq!(
    result_ab,
    Some( 0 ),
    "A-first: must pick a@test.com (index 0, alphabetically first)",
  );

  // B-first ordering: must still return Some(1) = A (name tiebreaker fires, not slice order).
  let result_ba = find_next_for_strategy(
    &[
      mk_aq_with_7d_reset( "b@test.com", 0.0,  now_secs, 10_800 ),
      mk_aq_with_7d_reset( "a@test.com", 77.0, now_secs, 10_800 ),
    ],
    SortStrategy::Renew,
    PreferStrategy::Any,
    now_secs,
    false,
  );
  assert_eq!(
    result_ba,
    Some( 1 ),
    "B-first: must pick a@test.com (index 1) — name tiebreaker fires over B",
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
  let indices  = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, now);

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
