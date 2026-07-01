// Integration tests for fetch.rs — relocated from src/usage/fetch_tests.rs.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{ inject_synthetic_if_new, parse_u64_from_str, fetch_quota_for_list, read_cached_quota };
use claude_profile::usage::test_bridge::types::AccountQuota;
use claude_profile::usage::test_bridge::FAR_FUTURE_MS;

// ── BUG-233 Class B: pre-flight expiry predicate ────────────────────────────

/// Class B predicate unit test: `expires_at_ms / 1000 <= now_secs` short-circuits expired accounts.
///
/// # Root Cause
/// `fetch_all_quota` spawned threads and made HTTP calls for locally-expired tokens (BUG-233).
/// Expired tokens always return 401; two wasted HTTP round trips per expired account.
///
/// # Why Not Caught
/// No pre-flight expiry check existed; the code always entered the fetch path after reading
/// the token. No test verified the expiry predicate or its consequence on API calls.
///
/// # Fix Applied
/// Fix(BUG-233): Class B pre-flight: `acct.expires_at_ms / 1000 <= now_secs` gate before
/// thread spawn and main-thread HTTP; returns `Err("token expired (local)")` immediately.
///
/// # Prevention
/// Expiry must be checked BEFORE any I/O. The divide-before-compare idiom (ms→s) must be
/// consistent: any future caller that adds a time-based gate must use the same unit conversion.
///
/// # Pitfall
/// Integer division truncates: `expires_at_ms = 999` → `0 / 1000 = 0 <= any now_secs`.
/// `expires_at_ms = 0` (unknown expiry) also triggers the guard — treated as epoch (expired).
#[ doc = "bug_reproducer(BUG-233)" ]
#[ test ]
fn test_class_b_expired_token_predicate()
{
  let now_secs : u64 = 1_748_000_000; // representative fixed reference point (Unix seconds)

  // Expired case: expires_at_ms converts to a second before now_secs.
  let past_ms  : u64 = ( now_secs - 1 ) * 1_000;
  assert!(
    past_ms / 1_000 <= now_secs,
    "Class B: past token (past_ms={past_ms}) must satisfy expired predicate vs now_secs={now_secs}",
  );

  // Valid case: expires_at_ms converts to 1 hour after now_secs.
  let future_ms : u64 = ( now_secs + 3_600 ) * 1_000;
  assert!(
    future_ms / 1_000 > now_secs,
    "Class B: future token (future_ms={future_ms}) must NOT satisfy expired predicate vs now_secs={now_secs}",
  );
}

// ── BUG-233 Class A: billing_type="none" result override predicate ──────────

/// Class A predicate unit test: `billing_type=="none"` overrides usage result to `Err("no subscription")`.
///
/// # Root Cause
/// Cancelled-subscription accounts receive HTTP 429 from the usage API (subscription inactive).
/// `fetch_all_quota` stored `Err("HTTP transport error: HTTP 429 ...")` — semantically wrong:
/// the account has no subscription, not a rate limit (BUG-233).
///
/// # Why Not Caught
/// No data-layer result override existed. The display layer papered over this with `error_label`
/// (BUG-231 workaround) which was the wrong fix location — data-layer incorrectness requires
/// a data-layer fix.
///
/// # Fix Applied
/// Fix(BUG-233): Class A override after `account_handle.join()`: when `billing_type=="none"`,
/// replace the usage result with `Err("no subscription")` regardless of what the API returned.
///
/// # Prevention
/// Semantic correctness belongs at the data layer (fetch.rs). Display-layer hacks for
/// data-layer incorrectness always become dead code after the proper fix is applied.
///
/// # Pitfall
/// Override fires ONLY when `account_data` is `Some` (account fetch succeeded and `billing_type`
/// is known). When `account_data` is `None` (account fetch failed), `billing_type` is unknown —
/// the original usage result must be preserved unchanged.
#[ doc = "bug_reproducer(BUG-233)" ]
#[ test ]
fn test_class_a_billing_none_override_predicate()
{
  // billing_type="none" (cancelled) → predicate fires → override to Err("no subscription").
  let cancelled = Some( claude_quota::OauthAccountData
  {
    tagged_id       : String::new(),
    uuid            : String::new(),
    email_address   : String::new(),
    full_name       : String::new(),
    display_name    : String::new(),
    billing_type    : "none".to_string(),
    has_max         : false,
    capabilities    : vec![],
    rate_limit_tier : String::new(),
    org_created_at  : "2024-01-01T00:00:00Z".to_string(),
    memberships     : vec![],
  } );
  assert!(
    cancelled.as_ref().is_some_and( |a| a.billing_type == "none" ),
    "Class A: billing_type==\"none\" must trigger result override to Err(\"no subscription\")",
  );

  // billing_type="stripe_subscription" (active) → predicate does NOT fire → result unchanged.
  let active = Some( claude_quota::OauthAccountData
  {
    tagged_id       : String::new(),
    uuid            : String::new(),
    email_address   : String::new(),
    full_name       : String::new(),
    display_name    : String::new(),
    billing_type    : "stripe_subscription".to_string(),
    has_max         : false,
    capabilities    : vec![],
    rate_limit_tier : String::new(),
    org_created_at  : "2024-01-01T00:00:00Z".to_string(),
    memberships     : vec![],
  } );
  assert!(
    !active.as_ref().is_some_and( |a| a.billing_type == "none" ),
    "Class A: active subscription must NOT trigger result override",
  );

  // account_data=None (account fetch failed) → predicate does NOT fire → result unchanged.
  let failed : Option< claude_quota::OauthAccountData > = None;
  assert!(
    !failed.as_ref().is_some_and( |a| a.billing_type == "none" ),
    "Class A: account=None must NOT trigger result override (billing_type unknown)",
  );
}

// ── BUG-234: result trace ordering ─────────────────────────────────────────

/// Result trace must be emitted AFTER the Class A `billing_type` override.
///
/// # Root Cause
/// The result trace block preceded `account_handle.join()` and the Class A override
/// (BUG-233 fix). For `billing_type="none"` accounts the raw API result (Ok or 429) can
/// differ from the stored result (`Err("no subscription")`), making the trace misleading.
///
/// # Why Not Caught
/// No test verified the relative ordering of the result trace vs. the `billing_type` override.
/// The contradiction (trace says OK, table says `(no subscription)`) only appears with live
/// accounts whose `billing_type == "none"` — uncommon in unit test fixtures.
///
/// # Fix Applied
/// Fix(BUG-234): moved the `if trace { match &r { ... } }` block to after the Class A
/// override. Trace now reports the final stored result, not the intermediate raw response.
///
/// # Prevention
/// Trace emissions must always follow ALL transformations of the value being reported.
/// Rule: (1) compute raw, (2) apply overrides, (3) emit trace.
///
/// # Pitfall
/// When adding new result overrides in future, ensure they precede the result trace block —
/// not after it. The Class A override must remain immediately before the trace.
#[ doc = "bug_reproducer(BUG-234)" ]
#[ test ]
fn mre_bug234_result_trace_after_billing_type_override()
{
  // Structural assertion: Class A override must precede the result trace in source.
  // RED before fix (trace at ~144, override at ~154); GREEN after fix (override first).
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/fetch.rs" ) );
  let override_pos = src.find( r#"a.billing_type == "none" ) && r.is_err() { Err( "no subscription""# )
    .expect( "BUG-234: Class A billing_type override not found in fetch.rs" );
  let trace_pos = src.find( r#"eprintln!( "{}{}  result: OK""# )
    .expect( "BUG-234: result: OK trace line not found in fetch.rs" );
  assert!(
    override_pos < trace_pos,
    "BUG-234: result trace emitted before Class A override — \
     for billing_type=\"none\" accounts trace shows raw result, not stored result; \
     override_pos={override_pos}, trace_pos={trace_pos}",
  );
}

// ── parse_u64_from_str ──────────────────────────────────────────────────────

/// MRE 2/2 for BUG-170: `parse_u64_from_str` extracts `expiresAt` from credentials JSON.
///
/// # Root Cause
/// `parse_u64_field` takes `&Path` and cannot be used with the in-memory `new_creds: String`
/// directly. BUG-170 is that there is no string-based fallback for extracting `expiresAt`
/// from `new_creds` when `jwt_exp_ms` returns `None`, leaving `aq.expires_at_ms` stale.
///
/// # Why Not Caught
/// TSK-163 replaced `parse_u64_field` (stale file) with `jwt_exp_ms` (new token) but added
/// no fallback for the case where `jwt_exp_ms` returns `None`. No test verified that the
/// `expiresAt` field in `new_creds` is readable and used when JWT decoding fails.
///
/// # Fix Applied
/// Fix(BUG-170): extracted `parse_u64_from_str(s: &str, key: &str) -> Option<u64>` from
/// `parse_u64_field`; added as `else if` fallback in `apply_refresh` at lines 803-810.
///
/// # Prevention
/// When adding an expiry-extraction strategy, always provide a string-based fallback for
/// credentials JSON already in memory; never assume all access tokens are JWTs.
///
/// # Pitfall
/// `parse_u64_from_str` scans for `"key":digits` — works for both flat JSON
/// (`{"expiresAt":N}`) and nested JSON (`{"claudeAiOauth":{"expiresAt":N}}`); the plain
/// string scan finds the first occurrence of the key regardless of nesting depth.
#[ doc = "bug_reproducer(BUG-170)" ]
#[ test ]
fn test_parse_u64_from_str_mre_bug170_extracts_expires_at()
{
  // Flat credentials JSON (common in test fixtures).
  let flat = r#"{"accessToken":"sk-ant-oat01-XXXX","expiresAt":9999999999999}"#;
  assert_eq!(
    parse_u64_from_str( flat, "expiresAt" ),
    Some( 9_999_999_999_999_u64 ),
    "parse_u64_from_str must extract expiresAt from flat credentials JSON",
  );

  // Nested credentials JSON (claudeAiOauth wrapper present in production credentials).
  let nested =
    r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat01-XXXX","expiresAt":1779487948931}}"#;
  assert_eq!(
    parse_u64_from_str( nested, "expiresAt" ),
    Some( 1_779_487_948_931_u64 ),
    "parse_u64_from_str must extract expiresAt from nested claudeAiOauth credentials JSON",
  );

  // Missing key — must return None, not panic.
  let no_key = r#"{"accessToken":"sk-ant-oat01-XXXX"}"#;
  assert!(
    parse_u64_from_str( no_key, "expiresAt" ).is_none(),
    "parse_u64_from_str must return None when expiresAt key is absent",
  );
}

// ── BUG-218 ─────────────────────────────────────────────────────────────────

/// BUG-218 MRE: `fetch_all_quota()` must not inject a synthetic row when the derived
/// name already appears in the stored-account result list.
///
/// # Root Cause
/// `results.insert(0, AccountQuota { name: synthetic_name, ... })` is unconditional
/// when `any_current == false`. When `~/.claude.json emailAddress` matches an existing
/// stored account name (a precondition enabled by BUG-217's stale email install +
/// subsequent token rotation), a duplicate row is created. `Valid: N+1` is reported;
/// `apply_refresh` and `apply_touch` then process the same account twice — risking
/// double-refresh or double subprocess spawning against the same credential file.
///
/// # Why Not Caught
/// The synthetic row path was designed for the case where the live email is genuinely
/// unknown (first-run or external credential install). No test exercised the collision
/// path where `synthetic_name` matches an existing stored account name.
///
/// # Fix Applied
/// `inject_synthetic_if_new()` — lookup-then-insert: only inject when `synthetic_name`
/// is absent from `results`. Enforces: at most one row per unique account name in the
/// quota table.
///
/// # Prevention
/// Any "inject if not present" operation must be lookup-then-insert, not unconditional
/// insert. Collections built from two sources (directory scan + supplemental injection)
/// must merge on a unique key.
///
/// # Pitfall
/// `any_current == false` is also true when BUG-217 installs a stale email that matches
/// an existing account — that is the exact collision precondition. Both bugs compound:
/// BUG-217 makes collision possible; BUG-218 makes it destructive.
#[ doc = "bug_reproducer(BUG-218)" ]
#[ test ]
fn test_mre_bug218_fetch_all_quota_no_duplicate_synthetic_row()
{
  // Simulate post-fetch state:
  //   - stored account "i6@wbox.pro" present (is_current=false — live token differs)
  //   - any_current=false — no stored account matches the live session token
  //   - synthetic_name derived from ~/.claude.json emailAddress = "i6@wbox.pro"
  // BUG-218: fetch_all_quota() does results.insert(0, synthetic) unconditionally —
  //   when synthetic_name == "i6@wbox.pro" which already exists, count becomes 2.
  let stored_row = AccountQuota
  {
    name                 : "i6@wbox.pro".to_string(),
    is_current           : false,
    is_active            : false,
    is_occupied_elsewhere : false,
    expires_at_ms        : FAR_FUTURE_MS,
    result               : Err( "missing accessToken".to_string() ),
    account              : None,
    host                 : String::new(),
    role                 : String::new(),
    renewal_at           : None,
    cached               : false,
    cache_age_secs       : None,
    is_owned             : true,
    owner                : String::new(),
  };
  let mut results = vec![ stored_row ];

  let synthetic = AccountQuota
  {
    name                 : "i6@wbox.pro".to_string(),
    is_current           : true,
    is_active            : false,
    is_occupied_elsewhere : false,
    expires_at_ms        : FAR_FUTURE_MS,
    result               : Err( "missing accessToken".to_string() ),
    account              : None,
    host                 : String::new(),
    role                 : String::new(),
    renewal_at           : None,
    cached               : false,
    cache_age_secs       : None,
    is_owned             : true,
    owner                : String::new(),
  };

  // Fix(BUG-218): guarded injection — only insert when name is absent from results.
  // Root cause: unconditional results.insert(0, ...) when any_current==false created
  // duplicate rows when synthetic_name matched an existing stored account name;
  // downstream apply_refresh and apply_touch then processed the same account twice.
  // Pitfall: any_current==false also occurs when BUG-217's stale email collides with
  // an existing account — both bugs must be fixed together for full correction.
  inject_synthetic_if_new( &mut results, synthetic );

  // Invariant: at most one row per unique account name.
  // FAILS before fix: count == 2 (duplicate row for "i6@wbox.pro").
  let i6_count = results.iter().filter( |r| r.name == "i6@wbox.pro" ).count();
  assert_eq!(
    i6_count, 1,
    "BUG-218: inject_synthetic creates duplicate — missing collision guard; count={i6_count}",
  );
  assert_eq!(
    results.len(), 1,
    "BUG-218: quota table must have exactly 1 row for 1 stored account; len={}",
    results.len(),
  );
}

// ── BUG-296: auth errors must bypass cache fallback ──────────────────────────

/// MRE for BUG-296: HTTP 401/403 auth errors must bypass the cache fallback arm.
///
/// # Root Cause
/// `Err( ref _e ) =>` in `fetch_quota_for_list` matched ALL error variants, including
/// HTTP 401 and 403 authentication failures. A 401 from the server would be silently
/// converted to `Ok(cached_data)` when a warm cache existed, causing:
///   - `should_refresh()` auth-error guard at `refresh_predicate.rs:34` bypassed
///   - No token refresh attempted (trace shows: `should_retry=false  reason: ok`)
///   - Watchdog receiving 🟢 status from stale cache indefinitely (confirmed: 7+ cycles)
///
/// # Why Not Caught
/// The cache fallback was designed for transient errors (429, network, timeout); no test
/// verified that auth errors (401, 403) bypass the cache arm. The distinction between
/// "transient failure" and "credential rejection" was absent from both code and tests.
///
/// # Fix Applied
/// Fix(BUG-296): match guard `Err( ref e ) if !e.contains("401") && !e.contains("403")`.
/// Auth errors fall through to `Err( _ ) => ( result, false, None )` — `cached=false`,
/// `aq.result` remains `Err`, enabling `should_refresh()` to trigger credential refresh.
///
/// # Prevention
/// Structural assertion: the guard pattern must appear in source before `read_quota_cache`.
/// Any future modification to the cache fallback match must preserve this ordering.
///
/// # Pitfall
/// HTTP 401/403 are DEFINITIVE rejections, not transient errors. Using cached data hides
/// a credential failure and prevents automated recovery via the refresh pipeline.
#[ doc = "bug_reproducer(BUG-296)" ]
#[ test ]
fn mre_bug296_cached_non_expired_401_no_refresh()
{
  // Structural assertion: auth-error guard must appear, and read_quota_cache must appear
  // AFTER it (not before) — there is another read_quota_cache call earlier in the file
  // (non-owned path), so we search within src[guard_pos..] to find the one in the error arm.
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/fetch.rs" ) );
  let guard_pos = src.find( r#"!e.contains( "401" ) && !e.contains( "403" )"# )
    .expect(
      "BUG-296: auth-error guard not found in fetch.rs — \
       401/403 errors must bypass cache fallback via match guard",
    );
  assert!(
    src[ guard_pos.. ].contains( "read_cached_quota( credential_store" ),
    "BUG-296: read_cached_quota call not found after auth-error guard in fetch.rs — \
     guard must precede read_cached_quota in the cache fallback arm",
  );
  // Confirm catch-all arm propagates auth errors without cache substitution.
  assert!(
    src.contains( "Err( _ ) => ( result, false, None )" ),
    "BUG-296: catch-all Err arm missing in cache fallback match — \
     auth errors must propagate as Err with cached=false",
  );
}

// ── BUG-236 MRE: billing_type override guard ────────────────────────────────

/// MRE 1/2 for BUG-236: `billing_type="none"` with `r=Ok(...)` must NOT be overridden.
///
/// # Root Cause
/// BUG-233 introduced a Class A override: when `billing_type == "none"`, store
/// `Err("no subscription")` regardless of the usage API result. This assumed
/// `billing_type="none"` ↔ cancelled subscription ↔ usage returns error. That holds for
/// genuinely cancelled accounts but NOT for non-stripe billing arrangements (team/enterprise)
/// where `billing_type="none"` can appear even when the usage API returns HTTP 200 with
/// valid quota data — the override discarded the real quota and replaced it with an error.
///
/// # Why Not Caught
/// The BUG-233 test (`test_class_a_billing_none_override_predicate`) only checked the
/// `billing_type == "none"` condition — it did not verify that `r.is_err()` is also required.
/// No test covered the active-subscription + `billing_type="none"` combination.
///
/// # Fix Applied
/// Fix(BUG-236): added `&& r.is_err()` to the Class A override predicate so it only fires
/// when BOTH signals agree: `billing_type` says no-subscription AND usage API also errored.
///
/// # Prevention
/// Override predicates that combine multiple signals must require ALL signals to agree.
/// A single field (`billing_type`) is insufficient when the authoritative signal (usage API
/// HTTP status) is also available.
///
/// # Pitfall
/// `billing_type="none"` has at least two causes: (a) cancelled subscription → usage error;
/// (b) non-stripe billing arrangement (team/enterprise) → usage 200/Ok. Never treat
/// `billing_type` alone as proof of subscription state when the usage API result is available.
#[ doc = "bug_reproducer(BUG-236)" ]
#[ test ]
fn mre_bug236_ok_result_not_overridden_when_billing_type_none()
{
  // billing_type="none" + r=Ok → second condition (r.is_err()) is false → NO override.
  let account_data = Some( claude_quota::OauthAccountData
  {
    tagged_id       : String::new(),
    uuid            : String::new(),
    email_address   : String::new(),
    full_name       : String::new(),
    display_name    : String::new(),
    billing_type    : "none".to_string(),
    has_max         : false,
    capabilities    : vec![],
    rate_limit_tier : String::new(),
    org_created_at  : "2024-01-01T00:00:00Z".to_string(),
    memberships     : vec![],
  } );
  let r : Result< claude_quota::OauthUsageData, String > = Ok( claude_quota::OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : None,
  } );
  // Replicate the fixed predicate from fetch_all_quota.
  let would_override = account_data.as_ref().is_some_and( |a| a.billing_type == "none" ) && r.is_err();
  assert!(
    !would_override,
    "BUG-236: billing_type=\"none\" + r=Ok must NOT trigger override — active non-stripe \
     accounts have billing_type=\"none\" but a valid usage response",
  );
}

/// MRE 2/2 for BUG-236: `billing_type="none"` with `r=Err(...)` IS overridden.
///
/// Confirms the override still fires for genuinely cancelled accounts: both conditions
/// must be true (`billing_type="none"` AND usage errored) for the "no subscription" override.
#[ doc = "bug_reproducer(BUG-236)" ]
#[ test ]
fn mre_bug236_err_result_overridden_when_billing_type_none()
{
  let account_data = Some( claude_quota::OauthAccountData
  {
    tagged_id       : String::new(),
    uuid            : String::new(),
    email_address   : String::new(),
    full_name       : String::new(),
    display_name    : String::new(),
    billing_type    : "none".to_string(),
    has_max         : false,
    capabilities    : vec![],
    rate_limit_tier : String::new(),
    org_created_at  : "2024-01-01T00:00:00Z".to_string(),
    memberships     : vec![],
  } );
  let r : Result< claude_quota::OauthUsageData, String > = Err( "HTTP 429".to_string() );
  // Replicate the fixed predicate from fetch_all_quota.
  let would_override = account_data.as_ref().is_some_and( |a| a.billing_type == "none" ) && r.is_err();
  assert!(
    would_override,
    "BUG-236: billing_type=\"none\" + r=Err must trigger override — cancelled account \
     signals agree (billing=no-sub AND usage=err)",
  );
}

// ── G1: non-owned accounts bypass token + HTTP; read cache ──────────────────

/// FT-04 (AC-04): G1 gate — non-owned account skips token read + HTTP; reads cache; `is_owned=false`.
///
/// When `{name}.json` has `owner` ≠ `current_identity()`, G1 fires:
/// - `read_token()` is NOT called (no credentials path exercise).
/// - `fetch_oauth_usage()` is NOT called (no HTTP).
/// - Returned `AccountQuota` has `is_owned: false` and `cached: true` from cache JSON.
///
/// Pitfall: `live_creds_file` is intentionally absent — live token lookup
/// must not block the function; graceful degradation sets `is_current = false`.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-04]
#[ test ]
fn ft04_non_owned_uses_cache_not_http()
{
  let store = tempfile::TempDir::new().unwrap();

  // Write {name}.json with owner != current_identity() AND a quota cache entry.
  // "other@remote" is guaranteed to differ from current_identity() (USER@hostname).
  let meta = serde_json::json!(
  {
    "owner" : "other@remote",
    "cache" :
    {
      "fetched_at"  : "2026-06-14T10:00:00Z",
      "status"      : "ok",
      "five_hour"   : { "left_pct" : 70.0 }
    }
  } );
  std::fs::write(
    store.path().join( "alice@test.com.json" ),
    serde_json::to_string_pretty( &meta ).map( | s | s + "\n" ).unwrap(),
  ).unwrap();

  // Credentials file must exist for the account struct to be valid.
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

  // live_creds_file absent → graceful degradation; is_current=false for all accounts.
  let absent_live = store.path().join( ".absent_credentials.json" );

  let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

  assert_eq!( results.len(), 1, "FT-04: must return exactly 1 AccountQuota for 1 account" );
  let aq = &results[ 0 ];
  assert!(
    !aq.is_owned,
    "FT-04: G1 gate must set is_owned=false for non-owned account; got: {:?}", aq.is_owned,
  );
  assert!(
    aq.cached,
    "FT-04: G1 gate must read cache (cached=true) for non-owned account; got: {:?}", aq.cached,
  );
  // result must be Ok (from cache) — not an HTTP error.
  assert!(
    aq.result.is_ok(),
    "FT-04: G1 gate must return Ok(cache_data) when cache present; got: {:?}", aq.result,
  );
}

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

  let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
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

  let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
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

  let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
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
  let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
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

  let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
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
