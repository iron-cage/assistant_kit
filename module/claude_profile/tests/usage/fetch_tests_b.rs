// Integration tests for fetch.rs — Part B.
// Continuation of `fetch_tests.rs`.

use claude_profile::usage::test_bridge::{ fetch_quota_for_list, read_cached_quota };

/// CC-7: G1 gate — non-owned account with NO cache → Err("not owned"), cached=false.
///
/// When `{name}.json` has a foreign owner but no quota cache, G1 returns
/// `Err("not owned")` with `cached=false`. The render shows `—` for all
/// quota columns.
#[ test ]
fn cc7_non_owned_no_cache()
{
  let store = tempfile::TempDir::new().unwrap();

  // owner != current_identity(), NO cache section.
  let meta = serde_json::json!( { "owner" : "other@remote" } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).map( | s | s + "\n" ).unwrap(),
  ).unwrap();

  std::fs::write(
    store.path().join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();

  let accounts = vec![ claude_profile::account::Account
  {
    name              : "alice@test.com".to_string(),
    subscription_type : "pro".to_string(),
    rate_limit_tier   : String::new(),
    expires_at_ms     : u64::MAX / 2,
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
    renewal_at        : None,
  } ];

  let absent_live = store.path().join( ".absent_credentials.json" );
  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

  assert_eq!( results.len(), 1, "CC-7: must return exactly 1 AccountQuota" );
  let aq = &results[ 0 ];
  assert!( !aq.is_owned, "CC-7: G1 gate must set is_owned=false" );
  assert!( !aq.cached, "CC-7: no cache → cached must be false" );
  assert!(
    aq.result.is_err(),
    "CC-7: no cache → result must be Err; got: {:?}", aq.result,
  );
  let err = aq.result.as_ref().unwrap_err();
  assert!(
    err.contains( "not owned" ),
    "CC-7: error message must contain 'not owned'; got: {err}",
  );
}

// ── Feature 040: history pipeline tests ────────────────────────────────────

/// FT-03 — Cache-fallback path does NOT write a new history entry.
///
/// A transient expired-token error triggers cache-fallback (not 401/403).
/// History is only appended on the success arm — the fallback arm must leave
/// the ring buffer unchanged.
#[ test ]
fn ft03_history_skips_cached_fallback()
{
  let store = tempfile::TempDir::new().unwrap();

  // Pre-write cache with one existing history entry (object-per-entry format).
  let meta = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : "2026-06-01T10:00:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 60.0 },
      "history"    :
      [
        { "t" : 1_748_000_000_u64, "h5" : [ 60.0, "" ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  // Expired token (expiresAt=1ms) — triggers "token expired (local)" error,
  // which is not 401/403 and goes to the cache-fallback arm.
  std::fs::write(
    store.path().join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":1}"#,
  ).unwrap();

  let accounts = vec![ claude_profile::account::Account
  {
    name              : "alice@test.com".to_string(),
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
    renewal_at        : None,
  } ];

  let absent_live = store.path().join( ".absent_credentials.json" );
  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

  // Pipeline must have used cache-fallback.
  assert_eq!( results.len(), 1 );
  assert!( results[ 0 ].cached, "FT-03: result must be cached" );

  // History must still have exactly 1 entry — fallback arm must not append.
  let text = std::fs::read_to_string( store.path().join( "alice@test.com.json" ) ).unwrap();
  let json : serde_json::Value = serde_json::from_str( &text ).unwrap();
  let history_len = json[ "cache" ][ "history" ].as_array().map_or( 0, Vec::len );
  assert_eq!(
    history_len, 1,
    "FT-03: cache-fallback must not append new history entry; got {history_len} entries",
  );
}

/// FT-05 — Approximation is independent per period: absent `sn` history entries leave sn unaffected.
///
/// History has 2 h5 entries (both utilization=70.0, slope=0 → extrapolation=70.0).
/// No sn entries. Cache h5=50.0, sn=50.0.
/// After cache-fallback + approximation: h5 becomes 70.0, sn stays 50.0.
#[ test ]
fn ft05_approx_independent_periods_absent_sn_unaffected()
{
  let store = tempfile::TempDir::new().unwrap();

  // History: 2 h5 entries at 70.0, no sn entries.
  // Both h5 at 70.0 → linear slope=0 → extrapolation=70.0 regardless of now_secs.
  // Cache: five_hour=50.0, seven_day_sonnet=50.0.
  let meta = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at"       : "2026-06-01T10:00:00Z",
      "status"           : "ok",
      "five_hour"        : { "left_pct" : 50.0 },
      "seven_day_sonnet" : { "left_pct" : 50.0 },
      "history" :
      [
        { "t" : 1_748_000_000_u64, "h5" : [ 70.0, "" ], "d7" : null, "sn" : null },
        { "t" : 1_748_003_600_u64, "h5" : [ 70.0, "" ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  std::fs::write(
    store.path().join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":1}"#,
  ).unwrap();

  let accounts = vec![ claude_profile::account::Account
  {
    name              : "alice@test.com".to_string(),
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
    renewal_at        : None,
  } ];

  let absent_live = store.path().join( ".absent_credentials.json" );
  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

  assert_eq!( results.len(), 1 );
  let aq = &results[ 0 ];
  assert!( aq.cached, "FT-05: must be cached result" );

  let data = aq.result.as_ref().expect( "FT-05: result must be Ok(cached)" );
  let h5 = data.five_hour.as_ref().expect( "FT-05: five_hour must be Some" );
  let sn = data.seven_day_sonnet.as_ref().expect( "FT-05: seven_day_sonnet must be Some" );

  // h5: 2 history points both at 70.0 → slope=0 → approx=70.0 (changed from 50.0 cache value).
  assert!(
    ( h5.utilization - 70.0 ).abs() < 1e-6,
    "FT-05: h5.utilization must be 70.0 (approximated from 2 identical history points); got: {}", h5.utilization,
  );
  // sn: no history entries → approx returns None → sn.utilization unchanged at 50.0 (cache).
  assert!(
    ( sn.utilization - 50.0 ).abs() < 1e-6,
    "FT-05: sn.utilization must remain 50.0 (no sn history; absent period unaffected); got: {}", sn.utilization,
  );
}

// ── FT-14..FT-18: read_cached_quota() unit tests (Fix BUG-304) ─────────────

/// FT-14 — `read_cached_quota` returns `None` when no cache entry exists (AC-11 backward-compat).
///
/// Verifies the base-case: absent cache file produces `None`, not a panic or dummy data.
#[ test ]
fn test_read_cached_quota_absent_returns_none()
{
  let store    = tempfile::TempDir::new().unwrap();
  let now_secs = 1_750_000_000_u64;
  let result   = read_cached_quota( store.path(), "nonexistent@test.com", now_secs );
  assert!( result.is_none(), "FT-14: absent cache must return None" );
}

/// FT-15 — `read_cached_quota` returns raw cache values when no `history` key is present (AC-11).
///
/// Old cache format (no `"history"` key) must be treated as 0 measurements — approximation not applied.
#[ test ]
fn test_read_cached_quota_no_history_returns_raw()
{
  let store    = tempfile::TempDir::new().unwrap();
  let now_secs = 1_750_000_000_u64;
  let meta     = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : "2026-06-21T10:00:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 42.0 }
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  let ( data, _, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
    .expect( "FT-15: cache present → must return Some" );
  let h5 = data.five_hour.expect( "FT-15: five_hour must be Some" );
  assert!(
    ( h5.utilization - 42.0 ).abs() < 1e-6,
    "FT-15: no history → raw cache value 42.0 unchanged; got {}", h5.utilization,
  );
}

/// FT-16 — `read_cached_quota` returns raw cache when only 1 history entry exists (AC-04).
///
/// Single measurement is below the ≥2 threshold; polynomial fit not attempted.
#[ test ]
fn test_read_cached_quota_one_history_returns_raw()
{
  let store    = tempfile::TempDir::new().unwrap();
  let now_secs = 1_750_000_000_u64;
  let meta     = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : "2026-06-21T10:00:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 42.0 },
      "history"    :
      [
        { "t" : 1_749_990_000_u64, "h5" : [ 42.0, "" ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  let ( data, _, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
    .expect( "FT-16: cache present → must return Some" );
  let h5 = data.five_hour.expect( "FT-16: five_hour must be Some" );
  assert!(
    ( h5.utilization - 42.0 ).abs() < 1e-6,
    "FT-16: 1 history entry (< 2) → raw cache 42.0 unchanged; got {}", h5.utilization,
  );
}

/// FT-17 — `read_cached_quota` applies quadratic approximation with 3+ history entries (AC-04).
///
/// Three h5 measurements at t0/t1/t2 with linear upward trend (10→25→40). At `now_secs` 1h
/// after t2 (within 2x-span safety), linear extrapolation gives ~55.0 ≠ raw cache 42.0.
///
/// History `resets_at` is `""` (empty) so `iso_to_unix_secs` returns `None` and the window
/// filter keeps all 3 entries unconditionally — same pattern as FT-05 in `approx.rs`.
#[ test ]
fn test_read_cached_quota_applies_approximation()
{
  let store = tempfile::TempDir::new().unwrap();
  // t0=0, t1=+1h, t2=+2h (relative). now_secs = t2 + 1h → within 2x span (span=7200, 2x=14400, delta=3600).
  let t0       = 1_749_990_000_u64;
  let t1       = t0 + 3_600;
  let t2       = t0 + 7_200;
  let now_secs = t2 + 3_600; // 1h after last point; extrapolation gives ~55.0
  let meta = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : "2025-06-15T12:20:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 42.0 },
      "history"    :
      [
        { "t" : t0, "h5" : [ 10.0, "" ], "d7" : null, "sn" : null },
        { "t" : t1, "h5" : [ 25.0, "" ], "d7" : null, "sn" : null },
        { "t" : t2, "h5" : [ 40.0, "" ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  let ( data, _, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
    .expect( "FT-17: cache present → must return Some" );
  let h5 = data.five_hour.expect( "FT-17: five_hour must be Some" );
  // Linear trend 10→25→40 with slope=15/3600 per second; at +1h after t2: 40 + 15 = 55.0.
  assert!(
    ( h5.utilization - 55.0 ).abs() < 5.0,
    "FT-17: 3 history entries → approximation ~55.0 (not raw 42.0); got {}", h5.utilization,
  );
}

/// FT-18 — `read_cached_quota` returns 0.0 when `resets_at` has elapsed (AC-07).
///
/// When `now_secs > resets_at_secs`, the quota window has reset; approximated utilization = 0.0.
#[ test ]
fn test_read_cached_quota_expired_window_returns_zero()
{
  let store = tempfile::TempDir::new().unwrap();
  // resets_at = 2026-06-01T09:00:00Z ≈ unix 1748768400.
  // window_start (h5, 18000s window) = 1748768400 - 18000 = 1748750400.
  // Points at 1748760000 and 1748763600 are within the window.
  // now_secs = 1748900000 > resets_at → expired → returns 0.0.
  let resets_at = "2025-06-01T09:00:00+00:00"; // unix 1748768400
  let meta      = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : "2026-06-01T08:30:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 42.0, "resets_at" : resets_at },
      "history"    :
      [
        { "t" : 1_748_760_000_u64, "h5" : [ 35.0, resets_at ], "d7" : null, "sn" : null },
        { "t" : 1_748_763_600_u64, "h5" : [ 42.0, resets_at ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  let now_secs = 1_748_900_000_u64; // well after resets_at 1748768400
  let ( data, _, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
    .expect( "FT-18: cache present → must return Some" );
  let h5 = data.five_hour.expect( "FT-18: five_hour must be Some" );
  assert!(
    h5.utilization.abs() < 1e-6,
    "FT-18: resets_at elapsed → approximated utilization must be 0.0; got {}", h5.utilization,
  );
}

// ── CC-08: read_cached_quota with exactly 2 history entries → linear approx ─

/// CC-08 — `read_cached_quota` with exactly 2 history entries → linear approximation applied.
///
/// # Root Cause
/// `history.len() >= 2` is the boundary guard. At n=2 the degree selector picks degree 1
/// (linear). This test verifies the minimum required count (2) is sufficient to trigger the
/// approximation path.
///
/// # Why Not Caught
/// FT-14 and FT-15 used 3+ history entries (degree 2 quadratic). The n=2 boundary was not
/// explicitly exercised in isolation.
///
/// # Fix Applied
/// Not a bug fix — correctness coverage for the `>= 2` boundary condition.
///
/// # Prevention
/// Always test the minimum required count to confirm the guard fires at the boundary.
///
/// # Pitfall
/// With n=2 and `now_secs = last_t + span` (not strictly greater), tangent continuation does
/// NOT fire. Linear extrapolation returns `last_y + slope × elapsed`.
#[ test ]
fn cc08_read_cached_quota_two_history_entries_applies_linear()
{
  let store    = tempfile::TempDir::new().unwrap();
  let t0       = 1_750_000_000_u64;
  let t1       = t0 + 3_600;
  let now_secs = t1 + 3_600; // 1h after t1; linear slope = (50-30)/3600 → +20 → 70.0

  // No five_hour.resets_at → resets_at=None → no window filter → both entries survive.
  // Exactly 2 h5 entries triggers history.len()>=2 guard → degree 1 (linear path).
  let meta = serde_json::json!(
  {
    "cache" :
    {
      "fetched_at" : "2026-06-15T00:00:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 42.0 },
      "history"    :
      [
        { "t" : t0, "h5" : [ 30.0, "" ], "d7" : null, "sn" : null },
        { "t" : t1, "h5" : [ 50.0, "" ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  let ( data, _, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
    .expect( "CC-08: cache present → must return Some" );
  let h5 = data.five_hour.expect( "CC-08: five_hour must be Some" );

  // Linear: slope = (50-30)/3600 s⁻¹; elapsed = 3_600s after t1 → 50 + 20 = 70.0.
  // Raw cache is 42.0 — approximation must differ to confirm history.len()>=2 path fired.
  assert!(
    ( h5.utilization - 42.0 ).abs() > 1e-6,
    "CC-08: 2 history entries must apply linear approx, not return raw 42.0; got {}", h5.utilization,
  );
  assert!(
    ( h5.utilization - 70.0 ).abs() < 5.0,
    "CC-08: linear extrapolation at +1h after t1 must be ≈70.0; got {}", h5.utilization,
  );
}

// ── FT-23: G1 non-owned path applies approximation (Fix BUG-304) ────────────

/// FT-23 — G1 non-owned path applies polynomial approximation from history (AC-04, Fix BUG-304).
///
/// Before BUG-304 fix, the G1 path called `read_quota_cache()` directly and returned the raw
/// cached value (42.0). After the fix, it calls `read_cached_quota()` which applies Feature 040
/// polynomial approximation — returning an approximated value different from raw 42.0.
///
/// Three h5 history entries with upward trend (10→25→40) at fixed timestamps far in the past.
/// At real test-execution time (well past the last measurement), the 2x-span safety fires and
/// tangent continuation clamps to 100.0. Assertion: h5 ≠ 42.0 (approximation was applied).
#[ test ]
fn ft23_g1_non_owned_applies_approximation()
{
  let store = tempfile::TempDir::new().unwrap();

  // Empty resets_at in history h5 entries → iso_to_unix_secs("") = None → no window filter
  // → all 3 entries survive at real test-execution time → approximation fires.
  let meta = serde_json::json!(
  {
    "owner" : "other@remote",
    "cache" :
    {
      "fetched_at" : "2026-06-11T10:00:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 42.0, "resets_at" : "2026-06-11T15:00:00+00:00" },
      "history"    :
      [
        { "t" : 1_749_990_000_u64, "h5" : [ 10.0, "" ], "d7" : null, "sn" : null },
        { "t" : 1_749_993_600_u64, "h5" : [ 25.0, "" ], "d7" : null, "sn" : null },
        { "t" : 1_749_997_200_u64, "h5" : [ 40.0, "" ], "d7" : null, "sn" : null }
      ]
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();
  std::fs::write(
    store.path().join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();

  let accounts = vec![ claude_profile::account::Account
  {
    name              : "alice@test.com".to_string(),
    subscription_type : "pro".to_string(),
    rate_limit_tier   : String::new(),
    expires_at_ms     : u64::MAX / 2,
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
    renewal_at        : None,
  } ];

  let absent_live = store.path().join( ".absent_credentials.json" );
  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

  assert_eq!( results.len(), 1 );
  let aq = &results[ 0 ];
  assert!( !aq.is_owned, "FT-23: G1 gate must set is_owned=false" );
  assert!( aq.cached, "FT-23: G1 must return cached=true when cache exists" );

  let data = aq.result.as_ref().expect( "FT-23: result must be Ok (cache exists)" );
  let h5 = data.five_hour.as_ref().expect( "FT-23: five_hour must be Some" );

  // Before BUG-304 fix: h5.utilization == 42.0 (raw cache, no approximation).
  // After fix: approximation fires (3 history entries, upward trend); tangent-continuation
  // clamps to 100.0 at test-execution time (well past last measurement at 1749997200).
  assert!(
    ( h5.utilization - 42.0 ).abs() > 1e-6,
    "FT-23 (BUG-304): G1 non-owned path must apply approximation; \
     raw cache 42.0 was returned — read_cached_quota() not called",
  );
}

/// FT-12 — Non-owned accounts do not append to history (AC-12).
///
/// G1 gate intercepts non-owned accounts before the success arm, so
/// `write_history_entry` is never called for them.
#[ test ]
fn ft12_history_non_owned_skips_append()
{
  let store = tempfile::TempDir::new().unwrap();

  // Foreign-owned account with a quota cache but no history.
  let meta = serde_json::json!(
  {
    "owner" : "other@remote",
    "cache" :
    {
      "fetched_at" : "2026-06-01T10:00:00Z",
      "status"     : "ok",
      "five_hour"  : { "left_pct" : 60.0 }
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).unwrap() + "\n",
  ).unwrap();

  std::fs::write(
    store.path().join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();

  let accounts = vec![ claude_profile::account::Account
  {
    name              : "alice@test.com".to_string(),
    subscription_type : "pro".to_string(),
    rate_limit_tier   : String::new(),
    expires_at_ms     : u64::MAX / 2,
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
    renewal_at        : None,
  } ];

  let absent_live = store.path().join( ".absent_credentials.json" );
  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

  // G1 gate must have fired.
  assert_eq!( results.len(), 1 );
  assert!( !results[ 0 ].is_owned, "FT-12: G1 gate must set is_owned=false" );

  // No history key must have been written to {name}.json.
  let text = std::fs::read_to_string( store.path().join( "alice@test.com.json" ) ).unwrap();
  let json : serde_json::Value = serde_json::from_str( &text ).unwrap();
  let has_history = json[ "cache" ].as_object()
    .is_some_and( |c| c.contains_key( "history" ) );
  assert!(
    !has_history,
    "FT-12: G1-gated non-owned account must not have 'history' written to account json",
  );
}
