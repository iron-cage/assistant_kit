// Integration tests for sort_next.rs — Part B.
// Continuation of `sort_next_tests.rs`.

use claude_profile::usage::test_bridge::{ find_next_for_strategy, strategy_metric };
use claude_profile::usage::test_bridge::sort_indices;
use claude_profile::usage::test_bridge::types::{ AccountQuota, SortStrategy, PreferStrategy };
use claude_profile::usage::test_bridge::
{
  FAR_FUTURE_MS,
  mk_aq_sort, mk_aq_sort_weekly, mk_aq_with_7d_reset, mk_aq_with_7d_reset_util,
  mk_aq_cancelled,
  reset_iso_at,
};

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
    fallback_reason : None,
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
    org_created_at : None,
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
    fallback_reason : None,
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
    org_created_at : None,
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

// ── GAP-8: find_first_eligible gate 4 at exactly 85% utilization ─────────────

// ── BUG-317 MRE: cancelled subscription eligibility gate ─────────────────

/// BUG-317 MRE — `find_first_eligible` must skip cancelled accounts (`billing_type="none"`).
///
/// # Root Cause
/// `find_first_eligible()` checked `is_current`, `is_occupied_elsewhere`, `result.is_err()`,
/// `five_hour.utilization >= 85.0`, `expires_at_ms`, and the `extra` predicate — but never
/// `billing_type`. A cancelled account with good quota passed all these gates and was
/// recommended as the next rotation target despite being permanently unusable after JWT expiry.
///
/// # Why Not Caught
/// Existing `find_first_eligible` tests never set `account = Some({billing_type: "none"})`.
/// Tests with `account = None` do not trigger the gate (ambiguous — API fetch failure).
///
/// # Fix Applied
/// Added `billing_type="none"` gate to `find_first_eligible()` (`sort_next.rs`), immediately
/// after `is_occupied_elsewhere` check: `if aq.account.as_ref().is_some_and(|a| a.billing_type == "none") { continue; }`
///
/// # Prevention
/// Test uses `mk_aq_cancelled` which always sets confirmed-cancelled `account` data.
/// Any test adding a rotation-candidate gate must add the analogous cancelled-account case.
///
/// # Pitfall
/// `account = None` is NOT gated — account-API fetch failure is ambiguous and does not
/// confirm cancellation. Only `billing_type = "none"` with `account = Some` is definitive.
#[ doc = "bug_reproducer(BUG-317)" ]
#[ test ]
fn mre_bug317_cancelled_not_recommended_by_find_next()
{
  let now = 0u64;
  // Single account: cancelled subscription, good quota (would pass extra predicate before fix).
  // Before fix: recommended by all strategies (quota passes, not expired, owned).
  // After fix: gated by billing_type="none" → all strategies return None.
  let cancelled = mk_aq_cancelled( "cancelled@test.com", 20.0, 20.0 );
  let accounts  = vec![ cancelled ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert!(
      result.is_none(),
      "BUG-317: {strategy:?} must not recommend cancelled account (billing_type='none'); got idx {result:?}",
    );
  }
}

/// GAP-8 — `find_first_eligible` gate 4 fires at exactly `five_hour.utilization = 85.0`.
///
/// Gate 4: `data.five_hour.as_ref().is_some_and(|p| p.utilization >= 85.0)` → skip.
/// At exactly 85.0, the condition `>= 85.0` is satisfied → account is excluded.
/// All three strategies must skip this account → `find_next_for_strategy` returns `None`.
#[ test ]
fn mre_bug_gap8_find_first_eligible_at_exactly_85_utilization()
{
  let now  = 0u64;
  let acct = mk_aq_sort( "aaa@test.com", 85.0, FAR_FUTURE_MS );  // exactly 85% → gate 4 fires
  let accounts = vec![ acct ];
  for strategy in [ SortStrategy::Renew, SortStrategy::Name, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert!(
      result.is_none(),
      "{strategy:?}: account with five_hour.utilization=85.0 must be skipped by gate 4 (>= 85.0); got: {result:?}",
    );
  }
}

// ── BUG-324 MRE: eligibility gate must be model-agnostic ─────────────────

/// BUG-324 MRE — green account with `7d(Son)=0%` must be eligible under all strategies and preferences.
///
/// # Root Cause
/// `find_first_eligible()` gate 7 used `prefer_weekly(aq, prefer) > 5.0` (model-aware).
/// Under `prefer::any`, `prefer_weekly = min(7d_left, 7d_son_left) = min(31%, 0%) = 0.0`,
/// which is `≤ 5.0` — blocking the account despite `seven_day_left = 31% > 5%`.
/// Same class as BUG-299 (fixed in `sort.rs`), but left unfixed in `sort_next.rs`.
///
/// # Why Not Caught
/// All existing `find_next_for_strategy` tests use `mk_aq_sort` (no Sonnet tier) or
/// `mk_aq_sort_weekly` with equal `seven_day_util == seven_day_sonnet_util`, making
/// `prefer_weekly(any) == seven_day_left`. The divergence only manifests when
/// `seven_day_sonnet_util > seven_day_util` under `prefer::any` or `prefer::sonnet`.
///
/// # Fix Applied
/// Gate 7 changed to `seven_day_left(aq) > WEEKLY_EXHAUSTION_THRESHOLD` — model-agnostic.
/// `prefer_weekly` remains correct for sort-order tiebreaks only (within `sort_indices`).
///
/// # Prevention
/// Any new `find_first_eligible` call site must use raw `seven_day_left`, never `prefer_weekly`.
/// Tests that exercise gate 7 must use divergent `seven_day_util != seven_day_sonnet_util` values.
///
/// # Pitfall
/// `prefer_weekly(any) = min(7d, 7d_son)` — absent Sonnet tier (`None`) falls back to `7d Left`
/// (not 0). Exhaustion only fires when `seven_day_sonnet = Some({util: 100%})` is present.
/// Model-aware gate incorrectly excluded healthy accounts when Sonnet tier is depleted.
#[ doc = "bug_reproducer(BUG-324)" ]
#[ test ]
fn mre_bug324_green_account_eligible_when_7d_son_exhausted()
{
  let now = 0u64;
  // target: 5h_util=0.0 (5h Left=100%), 7d_util=69.0 (7d Left=31%), 7d_son_util=100.0 (7d(Son)=0%).
  // prefer_weekly(any) = min(31%, 0%) = 0.0 ≤ 5.0 → BLOCKED before fix.
  // seven_day_left    = 31.0 > 5.0          → ELIGIBLE after fix.
  let target = mk_aq_sort_weekly( "aaa_target@test.com", 0.0, 69.0, 100.0 );
  // current: force the engine to look past is_current accounts.
  let mut current = mk_aq_sort( "zzz_current@test.com", 20.0, FAR_FUTURE_MS );
  current.is_current = true;
  let accounts = vec![ target, current ];

  for strategy in [ SortStrategy::Name, SortStrategy::Renew, SortStrategy::Renews ]
  {
    for prefer in [ PreferStrategy::Any, PreferStrategy::Opus, PreferStrategy::Sonnet ]
    {
      let result = find_next_for_strategy( &accounts, strategy, prefer, now, false );
      assert_eq!(
        result, Some( 0 ),
        "BUG-324: {strategy:?}/{prefer:?} — green account with 7d Left=31%, 7d(Son)=0% must be eligible (index 0); \
         gate 7 must use seven_day_left not prefer_weekly; got: {result:?}",
      );
    }
  }
}

/// BUG-324 regression — sole green candidate with `7d(Son)=0%` must be selected under all strategies.
///
/// # Root Cause
/// Same as `mre_bug324_green_account_eligible_when_7d_son_exhausted` — gate 7 was model-aware.
/// When the only non-blocked candidate has `7d_son=0%` and `prefer=any`, rotation returns `None`
/// before fix despite one eligible account existing.
///
/// # Why Not Caught
/// No test constructed a scenario where the SOLE remaining candidate is blocked by the divergent
/// `prefer_weekly` gate. Existing tests always had a "healthy" fallback with equal weekly values.
///
/// # Fix Applied
/// `seven_day_left(aq) > WEEKLY_EXHAUSTION_THRESHOLD` — sole candidate with raw 7d=31% passes.
///
/// # Prevention
/// Regression test confirms that having `seven_day_sonnet = Some({util: 100%})` on the only
/// available account does NOT prevent rotation when `seven_day_left > 5%`.
///
/// # Pitfall
/// Missing Sonnet tier (`seven_day_sonnet = None`) never triggers the bug — `prefer_weekly(any)`
/// falls back to `seven_day_left`. The bug only fires when `seven_day_sonnet = Some(...)` with
/// high utilization is present alongside lower overall `seven_day` utilization.
#[ doc = "bug_reproducer(BUG-324)" ]
#[ test ]
fn mre_bug324_sole_green_candidate_7d_son_zero_returns_some()
{
  let now = 0u64;
  // Sole eligible: 7d Left=31%, 7d(Son)=0% — the only candidate that can pass all gates.
  let sole = mk_aq_sort_weekly( "aaa_sole@test.com", 0.0, 69.0, 100.0 );
  // Blocked: is_current.
  let mut b_current = mk_aq_sort( "bbb_current@test.com", 20.0, FAR_FUTURE_MS );
  b_current.is_current = true;
  // Blocked: is_active.
  let mut b_active = mk_aq_sort( "ccc_active@test.com", 20.0, FAR_FUTURE_MS );
  b_active.is_active = true;
  // Blocked: token expired.
  let b_expired = mk_aq_sort( "ddd_expired@test.com", 20.0, 0 );  // expires_at_ms=0 → gate 5
  // Blocked: h-exhausted (gate 4: utilization=92.0 >= 85.0).
  let b_hexhausted = mk_aq_sort( "eee_hexhausted@test.com", 92.0, FAR_FUTURE_MS );

  let accounts = vec![ sole, b_current, b_active, b_expired, b_hexhausted ];
  for strategy in [ SortStrategy::Name, SortStrategy::Renew, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 0 ),
      "BUG-324: {strategy:?} — sole green candidate with 7d(Son)=0% must be selected (index 0); \
       rotation must not return None; got: {result:?}",
    );
  }
}

// ── BUG-324 corner cases: gate 7 boundary + model-agnostic eligibility ────

/// CC — Gate 7 boundary: `seven_day_left = 5.0` exactly → account SKIPPED in eligibility.
///
/// `seven_day_util = 95.0` → `seven_day_left = 100.0 - 95.0 = 5.0`.
/// Gate 7: `5.0 > WEEKLY_EXHAUSTION_THRESHOLD (5.0) = false` → gate fires → skipped.
/// Strict `>` operator — exactly at threshold is exhausted, not eligible.
/// Complements `sort.rs` GAP-7b (`status_group_of` boundary) with the eligibility-gate path.
#[ test ]
fn test_cc_gate7_boundary_exactly_5pct_skipped_in_eligibility()
{
  let now = 0u64;
  // seven_day_left = 5.0 exactly (boundary). seven_day_sonnet_util = 0.0 (no divergence).
  let target = mk_aq_sort_weekly( "aaa_target@test.com", 0.0, 95.0, 0.0 );
  let mut current = mk_aq_sort( "zzz_current@test.com", 20.0, FAR_FUTURE_MS );
  current.is_current = true;
  let accounts = vec![ target, current ];

  for strategy in [ SortStrategy::Name, SortStrategy::Renew, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert!(
      result.is_none(),
      "{strategy:?}: seven_day_left=5.0 (exactly at threshold) must be SKIPPED (strict > 5.0); got: {result:?}",
    );
  }
}

/// CC — Gate 7 just above boundary: `seven_day_left = 5.5` (rounds to 6) → account ELIGIBLE.
///
/// `seven_day_util = 94.5` → `seven_day_left = 100.0 - 94.5 = 5.5` (exact tie-break value).
/// Gate 7: `round(5.5) = 6.0` (round-half-away-from-zero), `6.0 > 5.0 = true` → eligible.
///
/// Fix(BUG-336): originally used `seven_day_util=94.99` (`left=5.01`) — once `seven_day_left()`
///   rounds its return value (this file's own BUG-336 fix), 5.01 rounds DOWN to 5.0 (the
///   threshold), no longer demonstrating "just above". Recalibrated to the new narrowest
///   above-threshold margin: 94.5 (left=5.5), the exact tie-break point that rounds up to 6.
#[ test ]
fn test_cc_gate7_just_above_boundary_eligible()
{
  let now = 0u64;
  let target = mk_aq_sort_weekly( "aaa_target@test.com", 0.0, 94.5, 0.0 );
  let mut current = mk_aq_sort( "zzz_current@test.com", 20.0, FAR_FUTURE_MS );
  current.is_current = true;
  let accounts = vec![ target, current ];

  for strategy in [ SortStrategy::Name, SortStrategy::Renew, SortStrategy::Renews ]
  {
    let result = find_next_for_strategy( &accounts, strategy, PreferStrategy::Any, now, false );
    assert_eq!(
      result, Some( 0 ),
      "{strategy:?}: seven_day_left=5.5 (rounds to 6, just above threshold) must be ELIGIBLE; got: {result:?}",
    );
  }
}

/// CC — BUG-324 class at narrowest margin: `seven_day_left = 5.5` (rounds to 6), `seven_day_sonnet_left = 0%`.
///
/// `seven_day_util = 94.5` → `seven_day_left = 5.5` → rounds to 6.0 (above threshold).
/// `seven_day_sonnet_util = 100.0` → `seven_day_sonnet_left = 0.0`.
/// `prefer_weekly(Any) = min(6.0, 0.0) = 0.0` — pre-BUG-324-fix: blocked (`0.0 ≤ 5.0`).
/// `seven_day_left = 5.5` (rounds to 6.0) — post-BUG-324-fix: eligible (`6.0 > 5.0`).
///
/// Fix(BUG-336): originally used `seven_day_util=94.99` (`left=5.01`) — once `seven_day_left()`
///   rounds its return value (this file's own BUG-336 fix), 5.01 rounds DOWN to 5.0 (the
///   threshold), which would incorrectly re-block this account and no longer exercise the
///   BUG-324 divergence this test targets. Recalibrated to 94.5 (left=5.5, the exact tie-break
///   that rounds up to 6) to keep the BUG-324 regression margin clear of the BUG-336 boundary.
#[ test ]
fn test_cc_bug324_divergent_at_boundary_eligible()
{
  let now = 0u64;
  let target = mk_aq_sort_weekly( "aaa_target@test.com", 0.0, 94.5, 100.0 );
  let mut current = mk_aq_sort( "zzz_current@test.com", 20.0, FAR_FUTURE_MS );
  current.is_current = true;
  let accounts = vec![ target, current ];

  for strategy in [ SortStrategy::Name, SortStrategy::Renew, SortStrategy::Renews ]
  {
    for prefer in [ PreferStrategy::Any, PreferStrategy::Opus, PreferStrategy::Sonnet ]
    {
      let result = find_next_for_strategy( &accounts, strategy, prefer, now, false );
      assert_eq!(
        result, Some( 0 ),
        "BUG-324 boundary: {strategy:?}/{prefer:?} — seven_day_left=5.5 (rounds to 6), 7d_son_left=0% \
         must be ELIGIBLE; got: {result:?}",
      );
    }
  }
}

/// CC — `prefer::sonnet` with absent Sonnet tier: account eligible via raw `seven_day_left`.
///
/// `seven_day_util = 50.0` → `seven_day_left = 50.0` (well above 5%).
/// `seven_day_sonnet = None` → `prefer_weekly(Sonnet) = 0.0` (absent = unknown = 0%).
/// Pre-fix: `0.0 > 5.0 = false` → BLOCKED (any account without Sonnet tier ineligible
///   under `prefer::sonnet`).
/// Post-fix: `seven_day_left = 50.0 > 5.0 = true` → ELIGIBLE (model-agnostic gate).
#[ test ]
fn test_cc_prefer_sonnet_absent_tier_eligible()
{
  let now = 0u64;
  // seven_day = Some(util=50.0), seven_day_sonnet = None.
  let mut target = mk_aq_sort_weekly( "aaa_target@test.com", 0.0, 50.0, 0.0 );
  if let Ok( ref mut d ) = target.result { d.seven_day_sonnet = None; }
  let mut current = mk_aq_sort( "zzz_current@test.com", 20.0, FAR_FUTURE_MS );
  current.is_current = true;
  let accounts = vec![ target, current ];

  let result = find_next_for_strategy(
    &accounts, SortStrategy::Renew, PreferStrategy::Sonnet, now, false,
  );
  assert_eq!(
    result, Some( 0 ),
    "prefer::sonnet + absent Sonnet tier: seven_day_left=50.0 > 5.0 must be ELIGIBLE; \
     pre-fix: prefer_weekly(Sonnet)=0.0 would block; got: {result:?}",
  );
}

/// CC — `prefer::sonnet` with Sonnet exhausted: account eligible via raw `seven_day_left`.
///
/// `seven_day_util = 50.0` → `seven_day_left = 50.0`.
/// `seven_day_sonnet_util = 100.0` → `seven_day_sonnet_left = 0%`.
/// `prefer_weekly(Sonnet) = 100.0 - 100.0 = 0.0` — pre-fix: blocked.
/// `seven_day_left = 50.0 > 5.0` — post-fix: eligible.
#[ test ]
fn test_cc_prefer_sonnet_exhausted_tier_eligible()
{
  let now = 0u64;
  let target = mk_aq_sort_weekly( "aaa_target@test.com", 0.0, 50.0, 100.0 );
  let mut current = mk_aq_sort( "zzz_current@test.com", 20.0, FAR_FUTURE_MS );
  current.is_current = true;
  let accounts = vec![ target, current ];

  let result = find_next_for_strategy(
    &accounts, SortStrategy::Renew, PreferStrategy::Sonnet, now, false,
  );
  assert_eq!(
    result, Some( 0 ),
    "prefer::sonnet + Sonnet exhausted (100%% util): seven_day_left=50.0 > 5.0 must be ELIGIBLE; \
     pre-fix: prefer_weekly(Sonnet)=0.0 would block; got: {result:?}",
  );
}
