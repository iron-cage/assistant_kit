// Path-referenced test module for sort_next.rs — compiled as `mod tests` via `#[path]`.
// Lives in src/usage/ (not tests/) to access pub(crate) find_next_for_strategy and strategy_metric
// without widening their visibility. See src/usage/readme.md § Inline Test Exception.

  use super::{ find_next_for_strategy, strategy_metric };
  use crate::usage::sort::sort_indices;
  use crate::usage::types::{ AccountQuota, SortStrategy, PreferStrategy };
  use crate::usage::test_support::
  {
    FAR_FUTURE_MS,
    mk_aq_sort, mk_aq_with_7d_reset, mk_aq_with_7d_reset_util,
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
  /// `extra` predicate: `prefer_weekly(aq, prefer) > 5.0 && ( !gate_ownership || aq.is_owned )`.
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

      // gate_ownership=false: aaa_unowned is eligible when ownership gate is off
      let no_gate = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
      assert!(
        no_gate.is_some(),
        "{strategy:?} gate_ownership=false: non-owned must be eligible when ownership gate is off",
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

  /// BUG-229 MRE: `sort::renew` (`find_next_for_strategy`) must pick the account with the soonest
  /// subscription renewal when it fires before any 7d reset.
  ///
  /// # Root Cause
  /// `find_next_for_strategy::Renew` closure used `h5.min(d7)` — 5h is not a renewal
  /// event, and subscription renewal was never consulted.
  ///
  /// # Why Not Caught
  /// All prior `sort::renew` tests set `renewal_at: None`, exercising only the 7d leg.
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
    let winner   = find_next_for_strategy( &accounts, SortStrategy::Renew, PreferStrategy::Any, now, false );

    assert_eq!(
      winner, Some( 1 ),
      "BUG-229: sort::renew must pick b (sub 30min < c 7d 1h); got: {winner:?}",
    );
    assert_eq!( accounts[ winner.unwrap() ].name, "b@test.com",
      "BUG-229: winner name must be b@test.com" );
  }

  /// BUG-229 MRE: `strategy_metric(Renew)` must show `→ Next` format: the soonest of
  /// `min(7d_reset, sub_renewal)` as `"in {dur} {event}"` or `"~in {dur} {event}"` (estimated).
  ///
  /// # Root Cause
  /// Previous format was `"{pct}% session, 5h resets in {h5} / 7d resets in {d7}"` — the
  /// criterion timers (d7 + sub) were not shown; session% and 5h are irrelevant to renew.
  ///
  /// # Why Not Caught
  /// No test asserted the renew metric format before this fix.
  ///
  /// # Fix Applied
  /// Renew arm now calls `next_event_raw(d7_secs, sub_s, sub_est)` and formats as
  /// `"in {dur} {prefix}"` or `"~in {dur} {prefix}"`, matching the `→ Next` column.
  /// Event labels: `+7d` for 7d weekly reset, `$ren` for subscription renewal.
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
      is_owned      : true,
      owner                : String::new(),
    };

    let metric = strategy_metric( &aq, SortStrategy::Renew, PreferStrategy::Any, now);

    // Sub (1h) < 7d reset (24h) → `next_event_raw` picks sub → "in 1h $ren" (exact, no ~).
    assert!(
      metric.contains( "$ren" ),
      "BUG-229: renew metric with exact sub must show '$ren' event: {metric}",
    );
    assert!(
      !metric.contains( "~in" ),
      "BUG-229: exact sub renewal must not have '~in' estimation prefix: {metric}",
    );
    assert!(
      !metric.contains( "7d resets in" ),
      "BUG-229: renew metric must not show old '7d resets in' format: {metric}",
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
      is_owned      : true,
      owner                : String::new(),
    };

    let metric = strategy_metric( &aq, SortStrategy::Renew, PreferStrategy::Any, now);

    // No sub, 7d reset in 1h → `next_event_raw` picks 7d reset → "in 1h +7d" (exact, no ~).
    assert!(
      metric.contains( "+7d" ),
      "BUG-229: renew metric without sub must show '+7d' event: {metric}",
    );
    assert!(
      !metric.contains( "renews" ),
      "BUG-229: renew metric without sub must not show 'renews': {metric}",
    );
    assert!(
      !metric.contains( "7d resets in" ),
      "BUG-229: renew metric must not show old '7d resets in' format: {metric}",
    );
    assert!(
      !metric.contains( "session" ),
      "BUG-229: renew metric must not show session%%: {metric}",
    );
  }

  // ── BUG-224: sort::renews ─────────────────────────────────────────────────

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
    let idx = sort_indices( &accounts, SortStrategy::Renews, None, PreferStrategy::Any, now);
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

  /// `sort::renew` (`find_next_for_strategy`) deterministic when all numeric keys tied (BUG-260).
  ///
  /// # Root Cause
  /// `find_next_for_strategy(Renew)` `min_by` at `sort_next.rs:99` had only 2 comparison
  /// levels (`renewal_event_secs` primary, `five_hour_left` secondary). When both tie,
  /// `min_by` returns the first matching iterator index — input-slice / filesystem iteration
  /// order (non-deterministic).
  ///
  /// # Why Not Caught
  /// BUG-243 added the `five_hour_left` tiebreaker but stopped there. BUG-259 added the
  /// name tiebreaker to `sort_indices` (`sort.rs`) but the `sort_next.rs` Renew arm was out
  /// of scope because it owns its own `min_by` closure rather than delegating to
  /// `sort_indices` (unlike Endurance and Drain arms).
  ///
  /// # Fix Applied
  /// Added `.then_with( || accounts[ a ].name.cmp( &accounts[ b ].name ) )` after
  /// `ha.total_cmp( &hb )` — same pattern as BUG-259 fix in `sort.rs:170`.
  ///
  /// # Prevention
  /// When adding a numeric tiebreaker to a `min_by`/`sort_by` closure, always add a final
  /// name tiebreaker for determinism — every sort in this module should end with name cmp.
  ///
  /// # Pitfall
  /// `min_by` (not `sort_by`): for `sort_by` the stable sort preserves order on equal; for
  /// `min_by` the iterator stops at the first minimum found (first in slice order wins on
  /// tie). This makes the missing tiebreaker silently non-deterministic.
  ///
  /// Spec: [`docs/feature/020_usage_sort_strategies.md` AC-04]
  #[ doc = "bug_reproducer(BUG-260)" ]
  #[ test ]
  fn mre_bug260_renew_nondeterministic_when_fully_tied()
  {
    let now_secs : u64 = 1_700_000_000;
    // Both accounts: util=0.0 → five_hour_left=100%; same 7d reset offset → identical
    // renewal_event_secs. All numeric keys fully tied → name tiebreaker fires.
    // Slice order is reverse-alpha (zorro at 0, alice at 1); alphabetical winner is alice.
    let zorro = mk_aq_with_7d_reset( "zorro@test.com", 0.0, now_secs, 10_800 );
    let alice = mk_aq_with_7d_reset( "alice@test.com", 0.0, now_secs, 10_800 );

    let result = find_next_for_strategy(
      &[ zorro, alice ],
      SortStrategy::Renew,
      PreferStrategy::Any,
      now_secs,
      false,
    );
    assert_eq!(
      result,
      Some( 1 ),
      "BUG-260: tied candidates must resolve alphabetically; alice@test.com (index 1) must win over zorro@test.com (index 0)",
    );
  }

  /// # BUG-292 Reproducer
  ///
  /// `sort::renew` must skip weekly-exhausted accounts (`prefer_weekly` ≤ 5.0) even
  /// when they have the soonest 7d reset event. Before this fix, a weekly-exhausted
  /// account with an imminent 7d reset was recommended because the `Renew` arm had no
  /// `prefer_weekly > 5.0` gate.
  ///
  /// # Root Cause
  /// `find_next_for_strategy(Renew)` lacked the weekly-floor gate present in `Drain`
  /// (BUG-206) and `Endurance` (BUG-287). The renew arm's qualification predicate did
  /// not include `prefer_weekly > 5.0`, allowing exhausted accounts with a soonest reset
  /// to pass all filters and be recommended.
  ///
  /// # Why Not Caught
  /// No test exercised the path where a weekly-exhausted account has a sooner
  /// `seven_day.resets_at` than a healthy candidate. All prior renew-next tests used
  /// `mk_aq_with_7d_reset` (hardcoded `seven_day.util=0.0` → `prefer_weekly=100%`) so
  /// the weekly-exhaustion path was never reached.
  ///
  /// # Fix Applied
  /// Replace the independent `.filter().min_by()` with `sort_indices(Renew)` +
  /// `find_first_eligible(extra=|aq| prefer_weekly(aq, prefer) > 5.0)`.
  ///
  /// # Prevention
  /// Any new `find_first_eligible` call site must include a weekly-floor gate.
  /// `|_| true` is not safe when weekly-exhausted accounts can appear in the input.
  ///
  /// # Pitfall
  /// Use `mk_aq_with_7d_reset_util` (not `mk_aq_with_7d_reset`) when a non-zero
  /// `seven_day.utilization` is needed — the `_7d_reset` variant hardcodes `util=0.0`.
  ///
  /// Spec: [`docs/feature/020_usage_sort_strategies.md` AC-04]
  #[ doc = "bug_reproducer(BUG-292)" ]
  #[ test ]
  fn mre_bug292_renew_skips_weekly_exhausted_even_with_soonest_renewal()
  {
    let now = 0u64;
    // exhausted@test: seven_day_util=96.0 → prefer_weekly=4.0 (≤ 5.0, weekly-exhausted).
    //   7d reset fires in 1h (SOONEST event) — before fix this account was recommended.
    //   five_hour_util=0.0 → five_hour_left=100% — NOT h-exhausted; passes all old filters.
    let exhausted = mk_aq_with_7d_reset_util( "exhausted@test.com", 0.0, 96.0, now, 3_600 );
    // healthy@test: seven_day_util=40.0 → prefer_weekly=60.0 (> 5.0, qualifies).
    //   7d reset fires in 24h (later event) — must be selected after fix.
    let healthy   = mk_aq_with_7d_reset_util( "healthy@test.com",   0.0, 40.0, now, 86_400 );

    let idx = find_next_for_strategy( &[ exhausted, healthy ], SortStrategy::Renew, PreferStrategy::Any, now, false );
    assert!( idx.is_some(), "BUG-292: renew must find a candidate (healthy@test.com is eligible)" );
    assert_eq!(
      idx.unwrap(), 1,
      "BUG-292: renew must skip exhausted@test.com (prefer_weekly=4.0 ≤ 5.0) and pick healthy@test.com (index 1); got {idx:?}",
    );
  }

  /// # BUG-291 Reproducer
  ///
  /// `sort::renew` tiebreaker (`find_next_for_strategy`) must match `sort_indices(Renew)`. Before this fix,
  /// `sort_indices(Renew)` used `prefer_weekly` ascending while `find_next_for_strategy(Renew)`
  /// used `five_hour_left` ascending — an account with lower hourly depletion (but higher weekly
  /// capacity) would rank first in sort but second in next selection.
  ///
  /// # Root Cause
  /// BUG-243 added `five_hour_left` as tiebreaker to the independent `find_next_for_strategy(Renew)`
  /// closure without updating `sort_indices(Renew)`. The two diverged silently; code even
  /// acknowledged this at the now-removed pitfall comment ("a fix to one never propagates").
  ///
  /// # Why Not Caught
  /// No test exercised the tiebreaker path where two accounts have identical renewal events
  /// but different `five_hour_left` vs `prefer_weekly` rankings.
  ///
  /// # Fix Applied
  /// Replace the independent `.filter().min_by()` with `sort_indices(Renew)` +
  /// `find_first_eligible` — sort order and recommendation always use the same algorithm.
  ///
  /// # Prevention
  /// `find_next_for_strategy` arms MUST delegate to `sort_indices` — never implement an
  /// independent sort closure. Any future change to `sort_indices` propagates automatically.
  ///
  /// # Pitfall
  /// `prefer_weekly` ascending means LOWER weekly capacity is preferred first (account benefits
  /// more from the upcoming renewal event). This differs from BUG-243's `five_hour_left`
  /// ascending rationale (more hourly depletion preferred). The two are NOT equivalent.
  ///
  /// Spec: [`docs/feature/020_usage_sort_strategies.md` AC-04]
  #[ doc = "bug_reproducer(BUG-291)" ]
  #[ test ]
  fn mre_bug291_renew_next_tiebreaker_matches_sort_indices()
  {
    let now = 0u64;
    // alice: prefer_weekly=90.0 (d7_util=10.0), five_hour_left=20% (h5_util=80.0).
    //   LOW five_hour_left → wins OLD BUG-243 tiebreaker. HIGH prefer_weekly → loses sort_indices.
    // bob:   prefer_weekly=40.0 (d7_util=60.0), five_hour_left=80% (h5_util=20.0).
    //   LOW prefer_weekly → wins sort_indices(Renew) tiebreaker. HIGH five_hour_left → loses BUG-243.
    // Both accounts: identical 7d reset at now+3600 → primary key tied, tiebreaker decides.

    // Step 1: sort_indices(Renew) ranks bob first (prefer_weekly 40 < alice 90).
    let alice_s = mk_aq_with_7d_reset_util( "alice@test.com", 80.0, 10.0, now, 3_600 );
    let bob_s   = mk_aq_with_7d_reset_util( "bob@test.com",   20.0, 60.0, now, 3_600 );
    let sorted  = sort_indices( &[ alice_s, bob_s ], SortStrategy::Renew, None, PreferStrategy::Any, now);
    assert_eq!(
      sorted[ 0 ], 1,
      "BUG-291: sort_indices(Renew) must rank bob (prefer_weekly=40) before alice (prefer_weekly=90); got {sorted:?}",
    );

    // Step 2: find_next_for_strategy(Renew) must agree with sort_indices — selects bob (index 1).
    let alice_n = mk_aq_with_7d_reset_util( "alice@test.com", 80.0, 10.0, now, 3_600 );
    let bob_n   = mk_aq_with_7d_reset_util( "bob@test.com",   20.0, 60.0, now, 3_600 );
    let idx     = find_next_for_strategy( &[ alice_n, bob_n ], SortStrategy::Renew, PreferStrategy::Any, now, false );
    assert_eq!(
      idx, Some( 1 ),
      "BUG-291: sort::renew tiebreaker must match sort_indices(Renew) — bob (prefer_weekly=40) must win, not alice; got {idx:?}",
    );
  }
