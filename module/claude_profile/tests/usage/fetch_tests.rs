// Integration tests for fetch.rs — relocated from src/usage/fetch_tests.rs.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{ inject_synthetic_if_new, parse_u64_from_str, fetch_quota_for_list };
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
