// Integration tests for sort.rs — sort_indices, status_group_of, StatusGroup.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{
  sort_indices, status_group_of, StatusGroup,
  FAR_FUTURE_MS, mk_aq_sort, mk_aq_sort_weekly, mk_aq_with_reset,
  mk_aq_with_7d_reset, mk_aq_cancelled,
};
use claude_profile::usage::test_bridge::types::{ SortStrategy, PreferStrategy };

// ── sort_indices: name strategy ──────────────────────────────────────────────

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

// ── sort_indices: renew strategy ─────────────────────────────────────────────

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
  assert_eq!( accounts[ idx[ 0 ] ].name, "first@test.com",  "all-exhausted renew: alphabetical within exhausted tier" );
  assert_eq!( accounts[ idx[ 1 ] ].name, "second@test.com", "all-exhausted renew: alphabetical within exhausted tier" );
  assert_eq!( accounts[ idx[ 2 ] ].name, "third@test.com",  "all-exhausted renew: alphabetical within exhausted tier" );
}

// ── sort_indices: determinism (BUG-259 MRE) ──────────────────────────────────

/// BUG-259 MRE — `sort::renew` with all keys tied must produce alphabetical order.
///
/// # Root Cause
/// `sort_by` had no final name tiebreaker; when `renewal_event_secs` and `prefer_weekly`
/// are identical (same-token accounts), row order depended on filesystem `read_dir`
/// iteration — non-deterministic across runs.
///
/// # Why Not Caught
/// `it008_lim_it_accounts_in_alpha_order` tests this end-to-end but requires live
/// tokens; it is flaky (passes when the filesystem returns accounts in alpha order).
///
/// # Fix Applied
/// Added `.then_with(|| accounts[a].name.cmp(&accounts[b].name))` as a final
/// tiebreaker to every `sort_by` closure in `sort_indices`, and added a name sort
/// on each `exhausted_vec` before appending to `non_exhausted`.
///
/// # Prevention
/// Unit test creates accounts in reverse alphabetical order (charlie, bravo, alpha)
/// with identical sort keys and asserts alphabetical output.
///
/// # Pitfall
/// `sort_by_key` was converted to `sort_by` for `Expires`/`Renews` — `sort_by_key`
/// does not support chaining a name tiebreaker.
#[ doc = "bug_reproducer(BUG-259)" ]
#[ test ]
fn mre_bug259_sort_renew_alphabetical_when_all_keys_tied()
{
  // Inserted in reverse alphabetical order — before fix, output order was non-deterministic.
  let accounts = vec![
    mk_aq_sort( "charlie@test.com", 50.0, FAR_FUTURE_MS ),
    mk_aq_sort( "bravo@test.com",   50.0, FAR_FUTURE_MS ),
    mk_aq_sort( "alpha@test.com",   50.0, FAR_FUTURE_MS ),
  ];
  let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "alpha@test.com",
    "renew: identical-key accounts must sort alphabetically; got: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
  assert_eq!( accounts[ idx[ 1 ] ].name, "bravo@test.com",   "second alphabetically" );
  assert_eq!( accounts[ idx[ 2 ] ].name, "charlie@test.com", "third alphabetically" );
}

// ── 4-group status partition (AC-12) ─────────────────────────────────────────

/// AC-12 — 4-group partition: h-exhausted (Group 2) ranks above weekly-exhausted (Group 3).
#[ test ]
fn test_4group_h_exhausted_ranks_before_weekly_exhausted()
{
  let accounts = vec![
    mk_aq_sort_weekly( "weekly_exhausted@test.com", 50.0, 96.0, 96.0 ), // Group 3
    mk_aq_sort_weekly( "h_exhausted@test.com",      90.0, 50.0, 50.0 ), // Group 2
  ];
  let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "h_exhausted@test.com",
    "4-group: h-exhausted (G2) must rank before weekly-exhausted (G3); got: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
  assert_eq!( accounts[ idx[ 1 ] ].name, "weekly_exhausted@test.com" );
}

/// AC-12 — 4-group partition: green (Group 1) ranks above h-exhausted (Group 2).
#[ test ]
fn test_4group_green_ranks_before_h_exhausted()
{
  let accounts = vec![
    mk_aq_sort_weekly( "h_exhausted@test.com", 90.0, 50.0, 50.0 ), // Group 2: five_hour_left=10 ≤ 15
    mk_aq_sort_weekly( "green@test.com",        50.0, 50.0, 50.0 ), // Group 1: both thresholds ok
  ];
  let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "green@test.com",
    "4-group: green (G1) must rank before h-exhausted (G2); got: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
}

/// AC-12 — 4-group partition: weekly-exhausted (Group 3) ranks above dead/red (Group 4).
///
/// Fix(BUG-321): original test used a both-exhausted account as G4 — premise-incorrect.
///   Both-exhausted is G3 (recoverable), not G4 (dead). Using `mk_aq_cancelled` (G4) instead.
#[ test ]
fn test_4group_weekly_exhausted_ranks_before_red()
{
  // G4 Dead: billing_type="none" (cancelled) — zzz@ sorts last alphabetically within G4
  let dead    = mk_aq_cancelled(   "zzz@test.com",             50.0, 20.0 );
  // G3 WeeklyExhausted: 5h=50% (ok), 7d=4% left (≤ 5%) — only weekly-exhausted
  let weekly  = mk_aq_sort_weekly( "weekly_only@test.com",     50.0, 96.0, 96.0 );
  // G3 WeeklyExhausted: 5h=10% left (≤ 15%), 7d=4% left (≤ 5%) — both-exhausted, Fix(BUG-321)
  let both_ex = mk_aq_sort_weekly( "weekly_both@test.com",     90.0, 96.0, 96.0 );
  let accounts = vec![ dead, weekly, both_ex ];
  let idx = sort_indices( &accounts, SortStrategy::Name, None, PreferStrategy::Any, 0 );
  // Both G3 accounts must rank before G4 Dead regardless of alphabetical order
  let pos_weekly  = idx.iter().position( |&i| accounts[ i ].name == "weekly_only@test.com" ).unwrap();
  let pos_both_ex = idx.iter().position( |&i| accounts[ i ].name == "weekly_both@test.com" ).unwrap();
  let pos_dead    = idx.iter().position( |&i| accounts[ i ].name == "zzz@test.com" ).unwrap();
  assert!(
    pos_weekly < pos_dead,
    "4-group: weekly-only (G3) must rank before dead (G4); order: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
  assert!(
    pos_both_ex < pos_dead,
    "Fix(BUG-321): both-exhausted (G3) must rank before dead (G4); order: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
}

/// AC-03 / AC-12 — 4-group partition: `desc::1` preserves group order (groups are never reversed).
#[ test ]
fn test_4group_desc1_preserves_group_order()
{
  let accounts = vec![
    mk_aq_sort_weekly( "green@test.com",       50.0, 50.0, 50.0 ), // Group 1
    mk_aq_sort_weekly( "h_exhausted@test.com", 90.0, 50.0, 50.0 ), // Group 2
  ];
  let idx = sort_indices( &accounts, SortStrategy::Name, Some( true ), PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "green@test.com",
    "4-group: desc::1 must not reverse group order; green (G1) must still rank before h-exhausted (G2); got: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
}

// ── BUG-299 MRE ──────────────────────────────────────────────────────────────

/// BUG-299 MRE — account with 5h=0%, 7d Left=32%, 7d(Son)=5% must be `HExhausted` under `prefer::any`.
///
/// # Root Cause
/// `status_group_of()` used `prefer_weekly( aq, prefer ) > 5.0` for the weekly boundary.
/// Under `prefer::any`, `prefer_weekly = min(7d_left, 7d_son_left) = min(32%, 5%) = 5.0`.
/// `5.0 > 5.0` is false → account misclassified as Red instead of `HExhausted`.
///
/// # Why Not Caught
/// All existing AC-12 tests use accounts where `7d_util == 7d_son_util`, so
/// `prefer_weekly(any) = min(x, x) = x`, identical to `seven_day_left`. The
/// divergence only appears when `7d_util != 7d_son_util` with `prefer::any`.
///
/// # Fix Applied
/// Changed `sort.rs:35` from `prefer_weekly( aq, prefer ) > 5.0` to
/// `seven_day_left( aq ) > 5.0`. Removed `prefer` param from `status_group_of()`.
///
/// # Prevention
/// MRE test uses `7d_util != 7d_son_util` to exercise the divergence path.
///
/// # Pitfall
/// `prefer_weekly(any) = min(7d, 7d_son)` can be strictly less than `seven_day_left`
/// when the two weekly quotas differ — even when neither quota individually is low.
/// Group boundary must use model-agnostic raw `7d Left` only.
#[ doc = "bug_reproducer(BUG-299)" ]
#[ test ]
fn mre_bug299_h_exhausted_misclassified_as_red_prefer_any()
{
  let accounts = vec![
    mk_aq_sort_weekly( "account-a",   100.0, 68.0, 95.0 ), // G2 HExhausted: 5h=0% ≤ 15%, 7d=32% > 5%
    mk_aq_sort_weekly( "red-account", 100.0, 96.0, 96.0 ), // G3 WeeklyExhausted: both exhausted (Fix BUG-321)
  ];
  let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "account-a",
    "BUG-299: account with 7d Left=32% must be HExhausted (G2), ranking before WeeklyExhausted (G3); got: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
  assert_eq!( accounts[ idx[ 1 ] ].name, "red-account" );
}

// ── status_group_of boundary: GAP-7 ──────────────────────────────────────────

/// GAP-7a — `status_group_of` assigns `HExhausted` when `five_hour.utilization = 85.0` exactly.
///
/// `five_hour_left = 100.0 - 85.0 = 15.0`; guard is `> 15.0` (strict) → `h5_ok = false`.
/// `seven_day = None` → `seven_day_left = 100.0 > 5.0` → `d7_ok = true`.
/// Result: `(false, true)` → `HExhausted`.
#[ test ]
fn mre_bug_gap7_status_group_of_h_exhausted_at_exactly_15_pct_left()
{
  let aq = mk_aq_sort( "test@x.com", 85.0, FAR_FUTURE_MS );  // 15% left exactly
  assert!(
    matches!( status_group_of( &aq ), StatusGroup::HExhausted ),
    "utilization=85.0 (15% left) must be HExhausted (strict > 15.0 guard; boundary is NOT green)",
  );
}

/// GAP-7b — `status_group_of` assigns `WeeklyExhausted` when `seven_day_left = 5.0` exactly.
///
/// `seven_day.utilization = 95.0` → `seven_day_left = 5.0`; guard is `> 5.0` (strict) → `d7_ok = false`.
/// `five_hour.utilization = 0.0` → `five_hour_left = 100.0 > 15.0` → `h5_ok = true`.
/// Result: `(true, false)` → `WeeklyExhausted`.
#[ test ]
fn mre_bug_gap7_status_group_of_weekly_exhausted_at_exactly_5_pct_left()
{
  let aq = mk_aq_sort_weekly( "test@x.com", 0.0, 95.0, 0.0 );  // seven_day_left = 5% exactly
  assert!(
    matches!( status_group_of( &aq ), StatusGroup::WeeklyExhausted ),
    "seven_day.util=95.0 (5% left) must be WeeklyExhausted (strict > 5.0 guard; boundary is NOT green)",
  );
}

// ── BUG-317 MRE: cancelled subscription ──────────────────────────────────────

/// BUG-317 MRE — cancelled subscription (`billing_type="none"`) must be classified Red
/// even when quota looks healthy.
///
/// # Root Cause
/// `status_group_of()` checked only `result.is_err()` and quota thresholds. It never
/// inspected `billing_type`. An account with `billing_type="none"` and 50% 5h / 80% 7d
/// (both thresholds pass) was classified Green — appearing as 🟢 despite being permanently
/// dead (no new JWT after subscription expiry).
///
/// # Why Not Caught
/// All existing `status_group_of` tests used `account = None` (no subscription data).
/// The `billing_type` field was never present in any sort-related test fixture.
///
/// # Fix Applied
/// Added cancelled-subscription gate to `status_group_of()` (sort.rs): checks
/// `aq.account.as_ref().is_some_and(|a| a.billing_type == "none")` → returns Red,
/// before quota threshold evaluation.
///
/// # Prevention
/// New helper `mk_aq_cancelled` always sets `billing_type="none"` with account data
/// present. Tests that need "good quota but dead account" must use it exclusively.
///
/// # Pitfall
/// `account = None` (account-API fetch failed) is ambiguous — do NOT classify as Red.
/// Only `account = Some({billing_type: "none"})` is the definitive cancelled signal.
#[ doc = "bug_reproducer(BUG-317)" ]
#[ test ]
fn mre_bug317_cancelled_subscription_classified_red()
{
  let cancelled = mk_aq_cancelled( "cancelled@test.com", 50.0, 20.0 );
  assert!(
    matches!( status_group_of( &cancelled ), StatusGroup::Red ),
    "BUG-317: billing_type='none' must be Red regardless of quota; got non-Red",
  );
}

/// BUG-317 regression — cancelled account sorts after weekly-exhausted in `sort::renew`.
#[ test ]
fn mre_bug317_cancelled_ranks_after_weekly_exhausted()
{
  let weekly    = mk_aq_sort_weekly( "weekly@test.com", 0.0, 96.0, 0.0 );
  let cancelled = mk_aq_cancelled( "cancelled@test.com", 50.0, 20.0 );
  let accounts  = vec![ cancelled, weekly ];
  let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "weekly@test.com",
    "BUG-317: weekly-exhausted (G3) must rank before cancelled/Red (G4); got: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
  assert_eq!( accounts[ idx[ 1 ] ].name, "cancelled@test.com" );
}

/// BUG-321 MRE — both-exhausted account sorts in G3 (weekly-exhausted), before G4 Dead.
///
/// # Root Cause
/// `status_group_of()` used `( false, false ) => StatusGroup::Red`. Both quota dimensions
/// below threshold with `result=Ok` is recoverable (7d reset restores both windows) — it
/// belongs in G3 alongside single-weekly-exhausted accounts, not G4 Dead.
///
/// # Fix Applied
/// Changed `( false, false ) => StatusGroup::Red` to
/// `( false, false ) => StatusGroup::WeeklyExhausted` in `status_group_of()`. No new enum
/// variant; no `sort_indices()` array resize (4-slot partition unchanged).
///
/// # Pitfall
/// The Dead gate (`billing_type="none"` + `result.is_err()`) fires BEFORE the quota tuple
/// match — both-exhausted only reaches the tuple when those guards are clear.
#[ doc = "bug_reproducer(BUG-321)" ]
#[ test ]
fn mre_bug321_both_exhausted_sorts_in_weekly_group()
{
  let both_exh = mk_aq_sort_weekly( "zzz@test.com", 94.0, 96.0, 0.0 );
  let dead     = mk_aq_cancelled(   "aaa@test.com", 50.0, 20.0 );
  let accounts = vec![ dead, both_exh ];
  let idx = sort_indices( &accounts, SortStrategy::Name, None, PreferStrategy::Any, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "zzz@test.com",
    "BUG-321: both-exhausted (G3) must sort before dead/cancelled (G4); order: {:?}",
    idx.iter().map( |&i| &accounts[ i ].name ).collect::< Vec< _ > >(),
  );
  assert_eq!( accounts[ idx[ 1 ] ].name, "aaa@test.com" );
}

/// BUG-321 MRE — full 4-group partition: Green → h-exhausted → weekly-exhausted (incl. both-exhausted) → Dead.
///
/// # Root Cause
/// `( false, false ) => StatusGroup::Red` placed both-exhausted in G4 alongside
/// cancelled/error accounts. G4 means permanently dead — requires external action to recover.
/// Both-exhausted is recoverable (7d reset restores both windows) and belongs in G3.
///
/// # Fix Applied
/// `( _, false ) => StatusGroup::WeeklyExhausted` — both `(true, false)` and `(false, false)`
/// merge into G3. Dead classification fires before this match via `result.is_err()` and
/// `billing_type="none"` guards — both-exhausted-with-Ok-result is never dead.
///
/// # Pitfall
/// `(false, false)` with `result=Ok` is G3 (recoverable), NOT G4 (dead). Dead requires an
/// explicit `result=Err` or `billing_type="none"` — the quota tuple alone is not sufficient.
#[ doc = "bug_reproducer(BUG-321)" ]
#[ test ]
fn mre_bug321_four_group_partition_order()
{
  let green      = mk_aq_sort_weekly( "green@test.com",      10.0, 10.0, 0.0 ); // G1
  let h_exh      = mk_aq_sort_weekly( "h_exh@test.com",      90.0, 10.0, 0.0 ); // G2
  let weekly_exh = mk_aq_sort_weekly( "weekly_exh@test.com", 10.0, 98.0, 0.0 ); // G3
  let both_exh   = mk_aq_sort_weekly( "both_exh@test.com",   94.0, 96.0, 0.0 ); // G3 (Fix BUG-321)
  let dead       = mk_aq_cancelled(   "dead@test.com",        50.0, 20.0      ); // G4
  let accounts   = vec![ green, h_exh, weekly_exh, both_exh, dead ];
  let idx        = sort_indices( &accounts, SortStrategy::Name, None, PreferStrategy::Any, 0 );
  let name_order : Vec< &str > = idx.iter().map( |&i| accounts[ i ].name.as_str() ).collect();
  assert_eq!(
    name_order,
    vec![ "green@test.com", "h_exh@test.com", "both_exh@test.com", "weekly_exh@test.com", "dead@test.com" ],
    "BUG-321: 4-group partition must produce G1→G2→G3(both+weekly alpha)→G4; got: {name_order:?}",
  );
}

/// CC-059/CC-060 — `prefer_weekly` with absent period data treats account as fully available.
#[ test ]
fn test_prefer_weekly_none_periods_treated_as_full()
{
  let accounts = vec![
    mk_aq_sort_weekly( "has_data@test.com", 50.0, 60.0, 60.0 ),
    mk_aq_sort(        "no_data@test.com",  50.0, FAR_FUTURE_MS ),
  ];
  let idx = sort_indices( &accounts, SortStrategy::Renew, None, PreferStrategy::Opus, 0 );
  assert_eq!(
    accounts[ idx[ 0 ] ].name, "has_data@test.com",
    "has_data ranks first (alphabetical tiebreaker within same green group)",
  );
  assert_eq!(
    accounts[ idx[ 1 ] ].name, "no_data@test.com",
    "no_data (None seven_day = 100% left) stays in same group, ranks second alphabetically",
  );
}
