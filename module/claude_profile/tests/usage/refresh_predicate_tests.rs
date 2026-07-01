// Integration tests for refresh_predicate.rs — should_refresh decision logic.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::should_refresh;
use claude_profile::usage::test_bridge::types::AccountQuota;
use claude_profile::usage::test_bridge::FAR_FUTURE_MS;

/// SR-1 — 401 triggers refresh regardless of `expires_at_ms` (far-future token).
#[ test ]
fn test_should_refresh_401_triggers()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : FAR_FUTURE_MS,
    result        : Err( "HTTP transport error: HTTP 401".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!( should_refresh( &aq, 0 ), "401 must trigger refresh" );
}

/// SR-2 — 403 triggers refresh regardless of `expires_at_ms`.
#[ test ]
fn test_should_refresh_403_triggers()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : FAR_FUTURE_MS,
    result        : Err( "HTTP transport error: HTTP 403".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!( should_refresh( &aq, 0 ), "403 must trigger refresh" );
}

/// SR-3 — 429 + locally expired (`expires_at_ms=0`, `now_secs=9999`) triggers refresh.
///
/// Verifies BUG-156 fix: a rate-limited account with a stale (past) `expiresAt`
/// must enter the refresh path so the credentials file gets updated.
#[ doc = "bug_reproducer(BUG-156)" ]
#[ test ]
fn test_should_refresh_mre_bug156_429_expired_triggers()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : 0, // locally expired
    result        : Err( "HTTP transport error: HTTP 429".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!(
    should_refresh( &aq, 9_999 ),
    "429+expired must trigger refresh (BUG-156), expires=0 now=9999",
  );
}

/// SR-4 — 429 with non-expired token must NOT trigger refresh.
///
/// When the local `expiresAt` is in the future, 429 means the token is valid but
/// the account is rate-limited. Refreshing would add a 30-second wait with no benefit.
#[ test ]
fn test_should_refresh_429_valid_token_no_trigger()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : FAR_FUTURE_MS, // not expired
    result        : Err( "HTTP transport error: HTTP 429".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq, 0 ),
    "429 with valid (non-expired) token must NOT trigger refresh",
  );
}

/// SR-5 — 429 with `expires_at_ms` exactly equal to `now_secs * 1000` → triggers refresh.
#[ test ]
fn test_should_refresh_429_exact_boundary_expired_triggers()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : 5_000,
    result        : Err( "HTTP transport error: HTTP 429".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!(
    should_refresh( &aq, 5 ),
    "429 with expires_at_ms=5000, now_secs=5 → 5000/1000=5<=5 → must trigger refresh",
  );
}

/// SR-6 — 429 with `expires_at_ms` one second in the future → no refresh triggered.
#[ test ]
fn test_should_refresh_429_one_sec_future_no_trigger()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : 6_000,  // one second ahead of now_secs=5
    result        : Err( "HTTP transport error: HTTP 429".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq, 5 ),
    "429 with expires_at_ms=6000, now_secs=5 → 6000/1000=6<=5 false → must not trigger refresh",
  );
}

/// SR-7 — Ok result never triggers refresh.
#[ test ]
fn test_should_refresh_ok_no_trigger()
{
  let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : 0,
    result        : Ok( quota ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!( !should_refresh( &aq, 9_999 ), "Ok result must not trigger refresh" );
}

/// SR-8 — Generic (non-HTTP) error does not trigger refresh.
#[ test ]
fn test_should_refresh_generic_error_no_trigger()
{
  let aq = AccountQuota
  {
    name          : "a@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : 0,
    result        : Err( "connection refused".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!( !should_refresh( &aq, 9_999 ), "generic error must not trigger refresh" );
}

// ── BUG-235 MRE: locally-expired tokens must trigger refresh ─────────────────

/// MRE for BUG-235: `should_refresh` returns `true` for `Err("token expired (local)")`.
///
/// # Root Cause
/// `should_refresh` only checked for "401", "403", and "429+expired". The string
/// `"token expired (local)"` matched none of those patterns, so accounts whose access
/// token had passed `expiresAt` were permanently skipped — `should_refresh` returned
/// `false` even though the OAuth refresh token was still valid and renewal was possible.
///
/// # Why Not Caught
/// No test covered the `"token expired (local)"` error string. The three handled error
/// classes (401/403/429) all originate from HTTP responses; the local-expiry error is
/// generated BEFORE any HTTP call is attempted, making it an invisible fourth class that
/// fell through all existing match arms.
///
/// # Fix Applied
/// Fix(BUG-235): added `"token expired (local)"` arm to `should_refresh()` — returns
/// `true` unconditionally when that string is present. No expiry re-check is needed:
/// the error string itself implies `expires_at_ms ≤ now`.
///
/// # Prevention
/// Any new error string produced by pre-API skips (e.g., `"token expired (local)"`) must
/// also be added to `should_refresh`; otherwise the account is silently frozen red.
///
/// # Pitfall
/// The access token and OAuth refresh token have independent lifetimes. A past `expiresAt`
/// does NOT mean the refresh token is expired. Always attempt renewal for locally-expired
/// tokens — the subprocess will fail fast if the refresh token is also expired.
#[ doc = "bug_reproducer(BUG-235)" ]
#[ test ]
fn mre_bug235_locally_expired_triggers_should_refresh()
{
  let aq = AccountQuota
  {
    name          : "i11@wbox.pro".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : 0,
    result        : Err( "token expired (local)".to_string() ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
  };
  assert!(
    should_refresh( &aq, 9_999 ),
    "BUG-235: Err(\"token expired (local)\") must trigger should_refresh — \
     OAuth refresh token may still be valid even when access token is locally expired",
  );
}

// ── BUG-255 MRE: cached + expired account must trigger should_refresh ─────────

/// MRE for BUG-255: `should_refresh` returns `true` for cached, locally-expired account.
///
/// # Root Cause
/// When `fetch_all_quota` fails to fetch live quota data, the cache fallback path converts
/// `Err` to `Ok(cached_data)` and sets `cached=true`. All existing guards in `should_refresh`
/// matched only `Err` variants — the cached `Ok` result bypassed every check, leaving the
/// account permanently unrefreshed even when its access token was locally expired.
///
/// # Why Not Caught
/// SR-7 (`test_should_refresh_ok_no_trigger`) has `cached=false`; it tests the "live Ok"
/// case only. The `cached=true` + expired combination was never tested — no guard existed.
///
/// # Fix Applied
/// Fix(BUG-255): added explicit `aq.cached && expired` arm to `should_refresh()` — returns
/// `true` when the result came from the cache fallback AND the local token is expired.
///
/// # Prevention
/// When adding new quota-result paths that produce `Ok` (e.g., future cache variants),
/// verify that `should_refresh` still handles expired-token cases correctly. Do not rely
/// solely on Err-pattern guards; check `cached` and `expires_at_ms` independently.
///
/// # Pitfall
/// The fix must NOT trigger for cached accounts with a valid (non-expired) token — a cache
/// hit is legitimately Ok if `expires_at_ms / 1000 > now_secs`. Only expired+cached entries
/// need refreshing. SR-10 covers the non-expired cached case.
#[ doc = "bug_reproducer(BUG-255)" ]
#[ test ]
fn mre_bug255_cache_defeats_refresh()
{
  let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let aq = AccountQuota
  {
    name                  : "alice@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : 0,
    result                : Ok( quota ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 3600 ),
    is_owned              : true,
    owner                : String::new(),
  };
  assert!(
    should_refresh( &aq, 9_999 ),
    "BUG-255: cached+expired account (result=Ok, cached=true, expires_at_ms=0) must trigger \
     should_refresh — cache fallback converts Err to Ok, defeating the existing Err-pattern guards",
  );
}

/// SR-10 — cached account with valid (non-expired) token does NOT trigger refresh.
#[ test ]
fn test_should_refresh_cached_valid_token_no_trigger()
{
  let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let aq = AccountQuota
  {
    name                  : "alice@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : u64::MAX,
    result                : Ok( quota ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 60 ),
    is_owned              : true,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq, 9_999 ),
    "cached account with valid (non-expired) token must NOT trigger refresh",
  );
}

// ── G2: non-owned accounts must never be refreshed ───────────────────────────

/// FT-06 (AC-06): `should_refresh()` returns `false` when `aq.is_owned == false`.
///
/// G2 gate fires before any other check — ownership enforcement is the first guard.
/// Even a 401 or locally-expired non-owned account must not trigger refresh.
#[ test ]
fn ft06_should_refresh_false_when_not_owned()
{
  let aq_401 = AccountQuota
  {
    name                  : "alice@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : 0,
    result                : Err( "HTTP 401".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : false,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq_401, 9_999 ),
    "FT-06: G2 — non-owned account with 401 must NOT trigger refresh",
  );

  let aq_expired = AccountQuota
  {
    name                  : "alice@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : 0,
    result                : Err( "token expired (local)".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : false,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq_expired, 9_999 ),
    "FT-06: G2 — non-owned account with locally-expired token must NOT trigger refresh",
  );

  let aq_429_expired = AccountQuota
  {
    name                  : "alice@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : 1_000,
    result                : Err( "HTTP 429".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : false,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq_429_expired, 9_999 ),
    "CC-8: G2 — non-owned 429+expired must NOT trigger refresh",
  );

  let aq_cached_expired = AccountQuota
  {
    name                  : "alice@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : 1_000,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : None,
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 300 ),
    is_owned              : false,
    owner                : String::new(),
  };
  assert!(
    !should_refresh( &aq_cached_expired, 9_999 ),
    "CC-8b: G2 — non-owned cached+expired must NOT trigger refresh",
  );
}

// ── SR-11: approaching-expiry MUST NOT trigger should_refresh ────────────────

/// SR-11 — A token that is valid but approaching expiry must NOT trigger refresh.
///
/// This test enforces the architectural constraint documented in the module doc:
/// `should_refresh()` has no approaching-expiry arm and must never have one until
/// `run_isolated` supports proactive token rotation (currently impossible).
///
/// # Why Not Caught (BUG-323)
/// The reactive predicate + 60-minute polling gap allows non-active tokens to expire
/// unrefreshed. A proactive arm was proposed as the fix (BUG-323). Investigation showed
/// the fix is unavailable: `run_isolated(["--print", "."])` with a valid AT returns
/// `credentials=None` — no OAuth refresh occurs.
///
/// # If You Are Considering Removing This Test
/// Do not remove this test without first verifying that `run_isolated` now supports
/// proactive AT rotation with a valid token. Otherwise removing it would re-introduce
/// a silent no-op arm. See BUG-323 history for prior investigation.
#[ test ]
fn sr11_approaching_expiry_must_not_trigger_refresh()
{
  let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let now_secs : u64 = 100_000;
  let aq = AccountQuota
  {
    name                  : "a@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : ( now_secs + 600 ) * 1000,
    result                : Ok( quota ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
  };
  assert!(
    !should_refresh( &aq, now_secs ),
    "SR-11: Ok token approaching expiry (10 min remaining) must NOT trigger should_refresh — \
     run_isolated with a valid AT returns credentials=None; proactive refresh is a silent no-op. \
     See module doc and BUG-323 for constraint rationale.",
  );
}

// ── BUG-303 MRE: owned+occupied accounts must not trigger should_refresh ──────

/// MRE for BUG-303: `should_refresh` returns `false` for owned account with `is_occupied_elsewhere == true`.
///
/// # Root Cause
/// G2 at `refresh_predicate.rs:32` checked `!aq.is_owned` only. When `is_owned=true` and
/// `is_occupied_elsewhere=true`, the guard passed — the 401 arm fired and returned `true`,
/// triggering `apply_refresh` → `refresh_account_token` → credential write. The other machine
/// holds those credentials in its live session; the write invalidates it.
///
/// # Fix Applied
/// Fix(BUG-303): extended G2 to `if !aq.is_owned || aq.is_occupied_elsewhere { return false; }`.
///
/// # Pitfall
/// Do NOT collapse `is_owned` and `!is_occupied_elsewhere` into a single flag — they have
/// different sources and lifecycle semantics (permanent until unclaim vs transient per session).
#[ doc = "bug_reproducer(BUG-303)" ]
#[ test ]
fn mre_bug303_should_refresh_false_for_occupied_elsewhere()
{
  let aq = AccountQuota
  {
    name                  : "alice@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : true,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Err( "HTTP transport error: HTTP 401".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
  };
  assert!(
    !should_refresh( &aq, 9_999 ),
    "BUG-303: owned+occupied account with 401 must NOT trigger should_refresh — \
     credential mutation while another machine uses this account would invalidate its live session",
  );
}
