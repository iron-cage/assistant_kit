//! Integration tests for stalest.rs — stale-first fetch-set reduction (TSK-499).
//!
//! Covers selection semantics (`select_stalest`), the rotate-bypass predicate
//! (`reduction_applies`), and the `fetch_quota_for_list` skip gate that renders
//! non-selected accounts from cache instead of fetching.
//!
//! Test Matrix (task `task/claude_profile/499_stalest_k_prefetch_subset_selection.md`):
//!
//! | Case | Aspect | Input | Expected Output |
//! |------|--------|-------|-----------------|
//! | T01a | selection picks K oldest | 4 caches aged 4h/3h/2h/30m, k=3 | the three oldest selected |
//! | T01b | gate skips non-selected | subset {a} over 3-account fleet | a takes fetch path; b, c cache-rendered |
//! | T01c | non-selected untouched | subset {a} over 3-account fleet | b.json / c.json byte-identical after call |
//! | T02  | missing cache ranks oldest | 1 of 3 accounts has no cache, k=1 | the no-cache account selected |
//! | T03a | rotate bypasses reduction | predicate over stalest/rotate combos | `rotate::1` disables reduction |
//! | T03b | api routes through predicate | api.rs source | `reduction_applies(` guards subset before fetch |
//! | T06  | full fleet rows preserved | subset {b} over 3-account fleet | 3 rows, original order, all named |
//! | T07  | `max_age` drains oldest first | ages 3.0h/2.5h/2.2h/10m, k=1, cap 7200 | 3.0h → 2.5h → 2.2h; 10m never; then empty |
//! | —    | tie-break by list order | equal `fetched_at`, k=1 | first-listed account selected |
//! | —    | k exceeds fleet | 2 accounts, k=10 | both selected |
//!
//! Offline observability: all `Account` fixtures carry `expires_at_ms : 1` (already
//! expired), so a selected account that passes the stale gate hits the BUG-233
//! locally-expired-token skip with zero HTTP. That skip's `Err` then enters the
//! BUG-296 cache-fallback arm — so accounts that must PROVE the fetch path was
//! taken are written cacheless: the fallback finds nothing and the row keeps
//! `Err( "token expired (local)" )`, `cached : false`. A gated (non-selected)
//! account with a cache returns the `approximate_quota` row — `Ok`, `cached : true`.
//! The pair of markers proves which path each account took without any network
//! access. The production-level "selected accounts' `fetched_at` advances"
//! observable (task AF1) requires a live HTTP success and is therefore represented
//! here by its offline equivalent: fetch-path reached for selected, cache files
//! byte-identical for non-selected.

use claude_profile::usage::test_bridge::{ select_stalest, reduction_applies, fetch_quota_for_list };
use claude_profile::account::{ Account, AccountBackend };
use std::collections::HashSet;

/// Fixed "now" for deterministic selection tests: 2026-08-16T12:00:00Z, derived via
/// the same ISO parser the selector uses — immune to manual epoch-arithmetic drift.
fn now_secs() -> u64
{
  claude_profile_core::account::parse_iso_utc_secs( "2026-08-16T12:00:00Z" ).unwrap()
}

/// Build an owned Anthropic-backend `Account` with an already-expired token.
///
/// `expires_at_ms : 1` forces the BUG-233 locally-expired skip for any account that
/// reaches the fetch path, guaranteeing zero HTTP in every test in this file.
fn mk_account( name : &str ) -> Account
{
  Account
  {
    name              : name.to_string(),
    subscription_type : "pro".to_string(),
    rate_limit_tier   : String::new(),
    expires_at_ms     : 1,
    is_active         : false,
    email             : String::new(),
    display_name      : String::new(),
    billing           : String::new(),
    model             : String::new(),
    tagged_id         : String::new(),
    uuid              : String::new(),
    capabilities      : Vec::new(),
    organization_uuid : String::new(),
    organization_name : String::new(),
    org_role          : String::new(),
    workspace_uuid    : String::new(),
    workspace_name    : String::new(),
    host              : String::new(),
    role              : String::new(),
    owner             : String::new(),
    is_owned          : true,
    claim_lock        : false,
    reserve           : false,
    renewal_at        : None,
    backend           : AccountBackend::Anthropic,
    base_url          : None,
    redirect_model    : None,
    inference_provider : String::new(),
    tags : Vec::new(),
  }
}

/// Write `{name}.json` with a quota cache block stamped `fetched_at`, plus the
/// credentials file the account needs to be structurally valid. No `owner` key —
/// the account is owned, so the G1 gate passes and the stale gate is reachable.
fn write_cached_account( store : &std::path::Path, name : &str, fetched_at : &str )
{
  let meta = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : fetched_at,
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 70.0 }
    }
  } );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    serde_json::to_string_pretty( &meta ).map( | s | s + "\n" ).unwrap(),
  ).unwrap();
  std::fs::write(
    store.join( format!( "{name}.credentials.json" ) ),
    r#"{"accessToken":"tok","expiresAt":1}"#,
  ).unwrap();
}

/// Write only the credentials file — no `{name}.json`, hence no cache block.
fn write_cacheless_account( store : &std::path::Path, name : &str )
{
  std::fs::write(
    store.join( format!( "{name}.credentials.json" ) ),
    r#"{"accessToken":"tok","expiresAt":1}"#,
  ).unwrap();
}

// ── Selection semantics ───────────────────────────────────────────────────────

/// T01a: `stalest::3` over caches aged 4h/3h/2h/30m selects exactly the three oldest.
#[ test ]
fn selection_picks_k_oldest()
{
  let store    = tempfile::TempDir::new().unwrap();
  let accounts = vec!
  [
    mk_account( "a@test.com" ),
    mk_account( "b@test.com" ),
    mk_account( "c@test.com" ),
    mk_account( "d@test.com" ),
  ];
  write_cached_account( store.path(), "a@test.com", "2026-08-16T08:00:00Z" ); // 4 h
  write_cached_account( store.path(), "b@test.com", "2026-08-16T09:00:00Z" ); // 3 h
  write_cached_account( store.path(), "c@test.com", "2026-08-16T10:00:00Z" ); // 2 h
  write_cached_account( store.path(), "d@test.com", "2026-08-16T11:30:00Z" ); // 30 m

  let selected = select_stalest( &accounts, store.path(), 3, 0, now_secs() );

  let expected : HashSet< String > =
    [ "a@test.com", "b@test.com", "c@test.com" ].iter().map( | s | ( *s ).to_string() ).collect();
  assert_eq!( selected, expected, "T01: k=3 must select exactly the 3 oldest caches" );
}

/// T02: an account with no cache block ranks infinitely stale and wins `stalest::1`.
#[ test ]
fn selection_missing_cache_ranks_oldest()
{
  let store    = tempfile::TempDir::new().unwrap();
  let accounts = vec!
  [
    mk_account( "a@test.com" ),
    mk_account( "b@test.com" ),
    mk_account( "c@test.com" ),
  ];
  write_cached_account( store.path(), "a@test.com", "2026-08-16T11:00:00Z" );
  write_cacheless_account( store.path(), "b@test.com" );
  write_cached_account( store.path(), "c@test.com", "2026-08-16T11:30:00Z" );

  let selected = select_stalest( &accounts, store.path(), 1, 0, now_secs() );

  let expected : HashSet< String > = [ "b@test.com".to_string() ].into_iter().collect();
  assert_eq!( selected, expected, "T02: the cacheless account must rank oldest and be selected" );
}

/// T07: `max_age::7200` eligibility with `stalest::1` drains oldest-first and
/// never touches the under-threshold account.
///
/// Ages 3.0 h / 2.5 h / 2.2 h / 10 m against a 7200 s threshold: three successive
/// calls (marking each winner fresh in between, as a successful fetch would) select
/// 3.0 h, then 2.5 h, then 2.2 h; the fourth call finds no eligible account.
#[ test ]
fn selection_max_age_drains_oldest_first()
{
  let store    = tempfile::TempDir::new().unwrap();
  let accounts = vec!
  [
    mk_account( "a@test.com" ),
    mk_account( "b@test.com" ),
    mk_account( "c@test.com" ),
    mk_account( "d@test.com" ),
  ];
  write_cached_account( store.path(), "a@test.com", "2026-08-16T09:00:00Z" ); // 3.0 h = 10800 s
  write_cached_account( store.path(), "b@test.com", "2026-08-16T09:30:00Z" ); // 2.5 h =  9000 s
  write_cached_account( store.path(), "c@test.com", "2026-08-16T09:48:00Z" ); // 2.2 h =  7920 s
  write_cached_account( store.path(), "d@test.com", "2026-08-16T11:50:00Z" ); // 10 m =    600 s

  let fresh = "2026-08-16T12:00:00Z";

  let first = select_stalest( &accounts, store.path(), 1, 7200, now_secs() );
  assert_eq!(
    first,
    [ "a@test.com".to_string() ].into_iter().collect::< HashSet< _ > >(),
    "T07: first call must select the 3.0 h account (oldest eligible)",
  );
  write_cached_account( store.path(), "a@test.com", fresh );

  let second = select_stalest( &accounts, store.path(), 1, 7200, now_secs() );
  assert_eq!(
    second,
    [ "b@test.com".to_string() ].into_iter().collect::< HashSet< _ > >(),
    "T07: second call must drain the 2.5 h account",
  );
  write_cached_account( store.path(), "b@test.com", fresh );

  let third = select_stalest( &accounts, store.path(), 1, 7200, now_secs() );
  assert_eq!(
    third,
    [ "c@test.com".to_string() ].into_iter().collect::< HashSet< _ > >(),
    "T07: third call must drain the 2.2 h account",
  );
  write_cached_account( store.path(), "c@test.com", fresh );

  let fourth = select_stalest( &accounts, store.path(), 1, 7200, now_secs() );
  assert!(
    fourth.is_empty(),
    "T07: with every cache under the 7200 s threshold no account is eligible; got {fourth:?}",
  );
}

/// Equal `fetched_at` values tie-break by original list order (deterministic).
#[ test ]
fn selection_tie_breaks_by_list_order()
{
  let store    = tempfile::TempDir::new().unwrap();
  let accounts = vec![ mk_account( "b@test.com" ), mk_account( "a@test.com" ) ];
  write_cached_account( store.path(), "a@test.com", "2026-08-16T10:00:00Z" );
  write_cached_account( store.path(), "b@test.com", "2026-08-16T10:00:00Z" );

  let selected = select_stalest( &accounts, store.path(), 1, 0, now_secs() );

  let expected : HashSet< String > = [ "b@test.com".to_string() ].into_iter().collect();
  assert_eq!(
    selected, expected,
    "equal ages must resolve by list position — first-listed wins, independent of name order",
  );
}

/// `k` larger than the fleet selects every account (no panic, no truncation error).
#[ test ]
fn selection_k_exceeding_fleet_selects_all()
{
  let store    = tempfile::TempDir::new().unwrap();
  let accounts = vec![ mk_account( "a@test.com" ), mk_account( "b@test.com" ) ];
  write_cached_account( store.path(), "a@test.com", "2026-08-16T10:00:00Z" );
  write_cached_account( store.path(), "b@test.com", "2026-08-16T11:00:00Z" );

  let selected = select_stalest( &accounts, store.path(), 10, 0, now_secs() );

  assert_eq!( selected.len(), 2, "k=10 over a 2-account fleet must select both accounts" );
}

// ── Rotate bypass ─────────────────────────────────────────────────────────────

/// T03a: the reduction predicate — active only when `stalest > 0` and rotate is off.
///
/// `rotate::1` needs a complete fresh ranking to pick a winner, so the reducer is
/// bypassed entirely (rotation freshness contract, task 499 / task 496 C5).
#[ test ]
fn reduction_predicate_rotate_bypasses()
{
  assert!(  reduction_applies( 2, false ), "stalest::2 without rotate must reduce" );
  assert!( !reduction_applies( 2, true ),  "T03: stalest::2 + rotate::1 must bypass the reducer" );
  assert!( !reduction_applies( 0, false ), "stalest absent (0) must not reduce" );
  assert!( !reduction_applies( 0, true ),  "neither param set must not reduce" );
}

/// T03b: `usage_routine` routes the subset decision through `reduction_applies`
/// before the fetch call — source-order proof, same technique as the BUG-245 test.
#[ test ]
fn api_routes_reduction_through_predicate()
{
  let src = include_str!( "../../src/usage/api.rs" );
  let predicate_pos = src.find( "reduction_applies(" )
    .expect( "T03: usage_routine must gate the stale subset via reduction_applies()" );
  let fetch_pos = src.find( "fetch_quota_for_list(" )
    .expect( "fetch_quota_for_list call must exist in api.rs" );
  assert!(
    predicate_pos < fetch_pos,
    "T03: the reduction decision must be made BEFORE fetch_quota_for_list — \
     predicate at byte {predicate_pos}, fetch at byte {fetch_pos}",
  );
}

// ── Fetch gate wiring ─────────────────────────────────────────────────────────

/// T01b: with a subset, selected accounts take the fetch path while non-selected
/// accounts return the cache-degradation row — no HTTP for either (expired tokens).
/// The selected account is cacheless so the BUG-296 cache fallback cannot mask the
/// fetch-path marker (see file header).
#[ test ]
fn fetch_gate_skips_non_selected()
{
  let store = tempfile::TempDir::new().unwrap();
  write_cacheless_account( store.path(), "a@test.com" );
  for name in [ "b@test.com", "c@test.com" ]
  {
    write_cached_account( store.path(), name, "2026-08-16T08:00:00Z" );
  }
  let accounts = vec!
  [
    mk_account( "a@test.com" ),
    mk_account( "b@test.com" ),
    mk_account( "c@test.com" ),
  ];
  let subset : HashSet< String > = [ "a@test.com".to_string() ].into_iter().collect();
  let absent_live = store.path().join( ".absent_credentials.json" );

  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false, Some( &subset ) );

  assert_eq!( results.len(), 3, "all 3 accounts must produce a row" );
  let a = &results[ 0 ];
  assert!(
    !a.cached && a.result.as_ref().err().is_some_and( | e | e.contains( "token expired" ) ),
    "T01: selected account must take the fetch path (expired-token skip), got cached={} result={:?}",
    a.cached, a.result,
  );
  for aq in &results[ 1.. ]
  {
    assert!(
      aq.cached && aq.result.is_ok(),
      "T01: non-selected {} must be cache-rendered (cached=true, Ok), got cached={} result={:?}",
      aq.name, aq.cached, aq.result,
    );
  }
}

/// T01c (task AF1, offline half): the gate leaves non-selected accounts' cache
/// files byte-identical — no write, no touch, no timestamp churn.
#[ test ]
fn fetch_gate_leaves_non_selected_files_untouched()
{
  let store = tempfile::TempDir::new().unwrap();
  for name in [ "a@test.com", "b@test.com", "c@test.com" ]
  {
    write_cached_account( store.path(), name, "2026-08-16T08:00:00Z" );
  }
  let accounts = vec!
  [
    mk_account( "a@test.com" ),
    mk_account( "b@test.com" ),
    mk_account( "c@test.com" ),
  ];
  let subset : HashSet< String > = [ "a@test.com".to_string() ].into_iter().collect();
  let absent_live = store.path().join( ".absent_credentials.json" );

  let b_before = std::fs::read_to_string( store.path().join( "b@test.com.json" ) ).unwrap();
  let c_before = std::fs::read_to_string( store.path().join( "c@test.com.json" ) ).unwrap();

  let _results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false, Some( &subset ) );

  let b_after = std::fs::read_to_string( store.path().join( "b@test.com.json" ) ).unwrap();
  let c_after = std::fs::read_to_string( store.path().join( "c@test.com.json" ) ).unwrap();
  assert_eq!( b_before, b_after, "AF1: non-selected b@test.com.json must be byte-identical" );
  assert_eq!( c_before, c_after, "AF1: non-selected c@test.com.json must be byte-identical" );
}

/// T06: a subset run preserves the full fleet in the result — same row count,
/// same order — so downstream rendering (`get::`/`format::` variants included)
/// receives the identical row set a full run would, differing only in provenance.
#[ test ]
fn fetch_subset_preserves_full_fleet_rows()
{
  let store = tempfile::TempDir::new().unwrap();
  for name in [ "a@test.com", "b@test.com", "c@test.com" ]
  {
    write_cached_account( store.path(), name, "2026-08-16T08:00:00Z" );
  }
  let accounts = vec!
  [
    mk_account( "a@test.com" ),
    mk_account( "b@test.com" ),
    mk_account( "c@test.com" ),
  ];
  let subset : HashSet< String > = [ "b@test.com".to_string() ].into_iter().collect();
  let absent_live = store.path().join( ".absent_credentials.json" );

  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false, Some( &subset ) );

  let names : Vec< &str > = results.iter().map( | aq | aq.name.as_str() ).collect();
  assert_eq!(
    names,
    vec![ "a@test.com", "b@test.com", "c@test.com" ],
    "T06: subset reduction must not drop or reorder rows — it only changes fetch provenance",
  );
}

/// `None` subset (reducer inactive) leaves every account on the ordinary path —
/// the 7th parameter's absent form is behavior-neutral. Cacheless fixtures keep the
/// fetch-path marker visible (BUG-296 fallback would otherwise mask it — see header).
#[ test ]
fn fetch_none_subset_fetches_every_account()
{
  let store = tempfile::TempDir::new().unwrap();
  for name in [ "a@test.com", "b@test.com" ]
  {
    write_cacheless_account( store.path(), name );
  }
  let accounts    = vec![ mk_account( "a@test.com" ), mk_account( "b@test.com" ) ];
  let absent_live = store.path().join( ".absent_credentials.json" );

  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false, None );

  for aq in &results
  {
    assert!(
      !aq.cached && aq.result.as_ref().err().is_some_and( | e | e.contains( "token expired" ) ),
      "None subset: {} must take the ordinary fetch path, got cached={} result={:?}",
      aq.name, aq.cached, aq.result,
    );
  }
}

// ── BUG-559: refresh-slot starvation ─────────────────────────────────────────

/// Write a per-host volatile cache file directly, with optional `last_error_at`.
///
/// Targets `cache/{any}/{name}.json` — `read_volatile_candidates` scans every subtree
/// under `cache/`, so the subdirectory name is irrelevant to the read path and a fixed
/// literal keeps the fixture independent of this host's own slug.
fn write_volatile_cache( store : &std::path::Path, name : &str, fetched_at : &str, last_error_at : Option< &str > )
{
  let dir = store.join( "cache" ).join( "fixturehost_fixtureuser" );
  std::fs::create_dir_all( &dir ).unwrap();
  let mut obj = serde_json::json!(
  {
    "fetched_at" : fetched_at,
    "five_hour"  : { "utilization" : 30.0 }
  } );
  if let Some( at ) = last_error_at
  {
    let o = obj.as_object_mut().unwrap();
    o.insert( "last_error".into(), serde_json::Value::String( "no subscription".into() ) );
    o.insert( "last_error_at".into(), serde_json::Value::String( at.to_string() ) );
  }
  std::fs::write( dir.join( format!( "{name}.json" ) ), serde_json::to_string_pretty( &obj ).unwrap() ).unwrap();
  std::fs::write(
    store.join( format!( "{name}.credentials.json" ) ),
    r#"{"accessToken":"tok","expiresAt":1}"#,
  ).unwrap();
}

/// bug_reproducer(BUG-559): a redirect-backend account must never consume a refresh
/// slot — it has no Anthropic quota to fetch, so the slot produces nothing, forever.
///
/// **Root Cause**: staleness was "time since last success". A redirect-backend account
/// can never write a quota cache, so it ranked `u64::MAX` on every tick and won slot 1
/// permanently, while genuinely stale Anthropic accounts behind it were never reached.
///
/// **Why Not Caught**: T02 asserts precisely the opposite for the general case — a
/// missing cache *should* rank oldest — and that remains correct. The defect only exists
/// for accounts structurally incapable of ever producing a cache, a class that did not
/// exist when the reducer was written (Feature 071 predates neither, but the two were
/// never considered together).
///
/// **Fix Applied**: filter non-anthropic `inference_provider` out of the ranking entirely,
/// before ages are computed.
///
/// **Prevention**: the fleet where this was observed ran `stalest::2` with exactly one
/// redirect seat and one dead account — net zero live accounts refreshed per tick. Any
/// change to the ranking key must keep this test and its BUG-557 sibling passing together;
/// either exclusion alone still starves the fleet.
///
/// **Pitfall**: an empty `inference_provider` means "anthropic", never a wildcard — the
/// same convention Gate 10 enforces. Filtering on `!= "anthropic"` without the empty case
/// would exclude the entire ordinary fleet and select nothing at all.
#[ test ]
fn bug_559_redirect_backend_account_never_consumes_a_refresh_slot()
{
  let store = tempfile::tempdir().unwrap();
  let mut redirect = mk_account( "kimi" );
  redirect.inference_provider = "moonshot".to_string();
  // No cache for either: the redirect seat can never have one, and the anthropic account
  // is written cacheless so both would tie at u64::MAX under the pre-fix ranking — making
  // the tie-break by list position, which `kimi` wins, the thing under test.
  std::fs::write( store.path().join( "kimi.credentials.json" ), r#"{"accessToken":"tok","expiresAt":1}"# ).unwrap();
  std::fs::write( store.path().join( "live@acme.com.credentials.json" ), r#"{"accessToken":"tok","expiresAt":1}"# ).unwrap();
  let accounts = vec![ redirect, mk_account( "live@acme.com" ) ];

  let selected = select_stalest( &accounts, store.path(), 1, 0, now_secs() );

  assert_eq!(
    selected, HashSet::from( [ "live@acme.com".to_string() ] ),
    "BUG-559: the sole refresh slot must go to the account that can actually use it",
  );
}

/// bug_reproducer(BUG-559): staleness must rank on the last fetch *attempt*, so an
/// account that fails every time takes its turn instead of monopolising the fetch set.
///
/// **Root Cause**: `write_quota_cache` is reachable only on success, so a permanently
/// dead account's `fetched_at` never advanced. Its age grew without bound, it stayed the
/// stalest live candidate forever, and it won a slot on every single tick — while the
/// accounts actually in use went days without a refresh.
///
/// **Why Not Caught**: every prior test drives accounts whose fetches succeed, where
/// "last success" and "last attempt" are the same instant and the bug is invisible.
///
/// **Fix Applied**: rank on `max(fetched_at, last_error_at)`, with `last_error_at`
/// persisted by BUG-557's `write_quota_cache_error`.
///
/// **Prevention**: this is a *re-definition*, deliberately not an exclusion. Excluding
/// dead accounts would strand them — nothing would ever re-fetch one, so an account that
/// resubscribed could never be discovered alive again.
///
/// **Pitfall**: `max`, not "prefer `last_error_at`". A stale error timestamp on an account
/// that has since succeeded must lose to the fresher `fetched_at`, or a recovered account
/// would be starved exactly as the dead one was.
#[ test ]
fn bug_559_staleness_ranks_on_last_attempt_not_last_success()
{
  let store = tempfile::tempdir().unwrap();
  // Dead: last success 3 days ago, but attempted (and failed) one minute ago.
  write_volatile_cache( store.path(), "dead@acme.com", "2026-08-13T12:00:00Z", Some( "2026-08-16T11:59:00Z" ) );
  // Live: last success an hour ago, no failures.
  write_volatile_cache( store.path(), "live@acme.com", "2026-08-16T11:00:00Z", None );
  let accounts = vec![ mk_account( "dead@acme.com" ), mk_account( "live@acme.com" ) ];

  let selected = select_stalest( &accounts, store.path(), 1, 0, now_secs() );

  assert_eq!(
    selected, HashSet::from( [ "live@acme.com".to_string() ] ),
    "BUG-559: a just-attempted dead account must not outrank an account last reached an hour ago",
  );
}

/// bug_reproducer(BUG-559): a recorded failure must not starve an account that has
/// since fetched successfully — the `max` half of the ranking key.
///
/// **Root Cause**: n/a — this guards the fix's own failure mode rather than the original
/// defect. See `bug_559_staleness_ranks_on_last_attempt_not_last_success`.
///
/// **Why Not Caught**: n/a.
///
/// **Fix Applied**: `ok.max( fail )` selects the newer of the two instants.
///
/// **Prevention**: the obvious alternative implementation — "use `last_error_at` when
/// present, else `fetched_at`" — passes the sibling test above and fails this one,
/// permanently demoting any account that ever failed once.
///
/// **Pitfall**: `write_quota_cache` clears both failure keys on success (BUG-557), so in
/// production this state is transient. It is reachable from another host's subtree
/// winning the freshest-wins merge, which is exactly what this fixture reproduces.
#[ test ]
fn bug_559_stale_failure_does_not_outrank_a_newer_success()
{
  let store = tempfile::tempdir().unwrap();
  // Recovered: failed 3 days ago, succeeded 5 minutes ago — must be the freshest.
  write_volatile_cache( store.path(), "recovered@acme.com", "2026-08-16T11:55:00Z", Some( "2026-08-13T12:00:00Z" ) );
  // Ordinary: last success 2 hours ago, no failures — genuinely the stalest.
  write_volatile_cache( store.path(), "ordinary@acme.com", "2026-08-16T10:00:00Z", None );
  let accounts = vec![ mk_account( "recovered@acme.com" ), mk_account( "ordinary@acme.com" ) ];

  let selected = select_stalest( &accounts, store.path(), 1, 0, now_secs() );

  assert_eq!(
    selected, HashSet::from( [ "ordinary@acme.com".to_string() ] ),
    "BUG-559: an old failure must not make a freshly-succeeded account look stale",
  );
}
