//! Refresh predicate for quota errors.
//!
//! `should_refresh` decides whether a quota error warrants a credential refresh.
//! Declared `pub(super)` so `refresh.rs` can re-import without leaking outside
//! the `usage` module.
//!
//! ## Architectural Constraint: No Proactive (Approaching-Expiry) Arm
//!
//! It is permanently forbidden to add an arm that triggers `should_refresh` when a token is
//! valid but approaching expiry (`expires_secs > now_secs && expires_secs <= now_secs + MARGIN`).
//!
//! **Why:** `refresh_account_token()` calls `run_isolated(["--print", "."])`. When the access
//! token is still valid, Claude Code uses it as-is and exits without performing an OAuth refresh,
//! returning `credentials=None`. An approaching-expiry arm would invoke `refresh_account_token()`
//! and get `credentials=None` back — a silent no-op that wastes 35 seconds per account per poll.
//!
//! **Spec reference:** `docs/feature/017_token_refresh.md` line 8 explicitly marks "proactive
//! expiry detection before any API call" as **Out of Scope**.
//!
//! **Mechanism reference:** `docs/invariant/008_single_token_refresh_entry.md` — the `expiresAt=1`
//! trick only works because it forces Claude Code to treat the AT as expired before calling the
//! OAuth server. A genuinely valid AT cannot be force-refreshed this way via `run_isolated`.
//!
//! SR-11 (`sr11_approaching_expiry_must_not_trigger_refresh`) enforces this constraint in tests.
//! If this constraint is ever proposed for removal, first resolve the subprocess limitation —
//! until `run_isolated` supports proactive token rotation, the arm cannot be made functional.
//! See BUG-323 for the full investigation history.

use super::types::AccountQuota;

// ── Refresh predicate ─────────────────────────────────────────────────────────

/// Return `true` when `apply_refresh` should attempt a token refresh for `aq`.
///
/// Triggers on:
/// - 401 or 403 — authentication failure; token rejected by the server.
/// - "token expired (local)" — `fetch_all_quota` skipped the API call because
///   `expiresAt` is in the past; the OAuth **refresh** token is still valid and
///   Claude Code will renew the access token automatically via `run_isolated()`.
/// - 429 AND locally expired (`expires_at_ms / 1000 ≤ now_secs`) — the per-account
///   credentials file may be stale (Claude Code updated the live session file but not
///   the saved per-account copy). Refreshing updates both the token and `expiresAt`.
/// - `cached=true` AND locally expired — the cache fallback in `fetch_all_quota`
///   converts `Err` to `Ok(cached_data)`; all Err guards are bypassed. An expired
///   cached entry needs a fresh token just as much as an Err entry does (BUG-255).
///
/// Returns `false` for 429 with a non-expired local token: the token is valid;
/// refreshing would add a 30-second subprocess wait with no benefit.
/// Returns `false` for cached accounts with a valid (non-expired) token: the cache
/// hit is legitimate; there is nothing to refresh.
pub( crate ) fn should_refresh( aq : &AccountQuota, now_secs : u64 ) -> bool
{
  // G2: Non-owned accounts must never be refreshed — credential mutation forbidden.
  // Fix(BUG-303): add is_occupied_elsewhere guard — credential mutation for owned-but-occupied
  //   accounts invalidates the live session on the other machine.
  // Root cause: G2 was written when is_occupied_elsewhere was not yet available (Feature 036).
  // Pitfall: is_owned and is_occupied_elsewhere are independent flags; both can be true simultaneously.
  if !aq.is_owned || aq.is_occupied_elsewhere { return false; }

  if matches!( aq.result, Err( ref e ) if e.contains( "401" ) || e.contains( "403" ) )
  {
    return true;
  }
  // Fix(BUG-235): refresh when token is locally expired — OAuth refresh token still valid.
  // Root cause: `fetch_all_quota` stores Err("token expired (local)") and skips the API call
  //   (correct — avoids a guaranteed 401). But `should_refresh` only checked for "401"/"403"/"429";
  //   "token expired (local)" matched none of them, so the account was silently skipped.
  // Pitfall: the access token and refresh token have independent lifetimes; a past `expiresAt`
  //   does NOT mean the refresh token is expired. Always attempt renewal for locally-expired tokens.
  if matches!( aq.result, Err( ref e ) if e.contains( "token expired (local)" ) )
  {
    return true;
  }
  // Fix(BUG-156): also refresh when rate-limited AND locally expired.
  // Root cause: 429+expired accounts were unconditionally excluded; the guard
  //   assumed "429 = valid token" but a past `expiresAt` indicates the per-account
  //   file may be stale — the token may need refreshing despite the 429 response.
  // Pitfall: don't refresh ALL 429 accounts (as task 142 did) — that adds a
  //   pointless 30-second wait for valid-but-rate-limited accounts.
  if matches!( aq.result, Err( ref e ) if e.contains( "429" ) )
    && ( aq.expires_at_ms / 1000 ) <= now_secs
  {
    return true;
  }
  // Fix(BUG-255): refresh cached+expired account — cache fallback converts Err to Ok.
  // Root cause: when fetch fails, `fetch_all_quota` returns Ok(cached_data) with cached=true.
  //   All existing guards match only Err variants; a cached account with Ok result and an
  //   expired local token is never refreshed, leaving stale credentials permanently.
  // Pitfall: don't refresh ALL cached accounts — only those with an expired token; a cached
  //   account with a valid (non-expired) token needs no credential refresh.
  aq.cached && ( aq.expires_at_ms / 1000 ) <= now_secs
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::should_refresh;
  use crate::usage::types::AccountQuota;
  use crate::usage::test_support::FAR_FUTURE_MS;


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
  ///
  /// The guard uses `(expires_at_ms / 1000) <= now_secs`.  When `expires_at_ms = 5000`
  /// and `now_secs = 5`, `5000/1000 = 5 <= 5` is `true` — the token is treated as expired.
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
  ///
  /// When `expires_at_ms = 6000` and `now_secs = 5`, `6000/1000 = 6 <= 5` is `false` —
  /// the token is still valid; no refresh triggered.
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
      expires_at_ms         : 0,  // 0 / 1000 = 0 ≤ any now_secs → locally expired
      result                : Ok( quota ),
      account               : None,
      host                  : String::new(),
      role                  : String::new(),
      renewal_at            : None,
      cached                : true,  // cache fallback used — Err was converted to Ok
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
  ///
  /// Contrast with BUG-255 MRE: the fix only applies when `expires_at_ms` is expired.
  /// A cache hit with a live token is legitimately Ok — refreshing would waste 30 s.
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
      expires_at_ms         : u64::MAX,  // far future → not expired
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

  // ── G2: non-owned accounts must never be refreshed ────────────────────────

  /// FT-06 (AC-06): `should_refresh()` returns `false` when `aq.is_owned == false`.
  ///
  /// G2 gate fires before any other check — ownership enforcement is the first guard.
  /// Even a 401 or locally-expired non-owned account must not trigger refresh.
  ///
  /// Spec: [`tests/docs/feature/036_account_ownership.md` FT-06]
  #[ test ]
  fn ft06_should_refresh_false_when_not_owned()
  {
    // 401 error + not owned → G2 fires first → false.
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

    // Locally expired + not owned → G2 fires first → false.
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

    // CC-8: 429 + locally expired + not owned → G2 fires first → false.
    let aq_429_expired = AccountQuota
    {
      name                  : "alice@test.com".to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms         : 1_000,   // expired at 1 second
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

    // CC-8b: cached + expired + not owned → G2 fires first → false.
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

  // ── SR-11: approaching-expiry MUST NOT trigger should_refresh (constraint) ───

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
  /// `credentials=None` — no OAuth refresh occurs. `feature/017` line 8 also explicitly
  /// marks proactive expiry detection as Out of Scope.
  ///
  /// # What This Test Asserts
  /// An `Ok` result with a token expiring in 10 minutes (within any plausible polling margin)
  /// returns `false`. This is NOT "the predicate ignores an important case" — it is the
  /// correct and required behavior given the subprocess constraint.
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
    // Token expires in 10 minutes (600s) — well within any proactive margin.
    let aq = AccountQuota
    {
      name                  : "a@test.com".to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms         : ( now_secs + 600 ) * 1000,  // 10 min remaining
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
  /// holds those credentials in its live session; the write invalidates it. No warning emitted.
  ///
  /// # Why Not Caught
  /// `ft06_should_refresh_false_when_not_owned` only tested the `is_owned=false` case.
  /// The `is_owned=true, is_occupied_elsewhere=true` combination was never tested — G2 was
  /// written before `is_occupied_elsewhere` was introduced (Feature 036 / TSK-293).
  ///
  /// # Fix Applied
  /// Fix(BUG-303): extended G2 to `if !aq.is_owned || aq.is_occupied_elsewhere { return false; }`.
  /// The occupancy guard fires before any Err-pattern checks — owned+occupied accounts are
  /// treated identically to non-owned accounts for the purpose of credential mutation.
  ///
  /// # Prevention
  /// Any new credential-mutation gate must check BOTH `is_owned` AND `!is_occupied_elsewhere`.
  /// These flags are independent: ownership grants authorization; occupancy denotes concurrent use.
  ///
  /// # Pitfall
  /// Do NOT collapse these into a single `can_mutate_credentials` flag — the two concepts have
  /// different sources (`owner` field in JSON vs `_active_*` marker file from another machine)
  /// and different lifecycle semantics (permanent until unclaim vs transient per session).
  #[ doc = "bug_reproducer(BUG-303)" ]
  #[ test ]
  fn mre_bug303_should_refresh_false_for_occupied_elsewhere()
  {
    // Owned account with a concurrent active session on another machine.
    // 401 error would normally trigger refresh — occupancy guard must block it.
    let aq = AccountQuota
    {
      name                  : "alice@example.com".to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : true,   // another machine's _active_* marker names this account
      expires_at_ms         : FAR_FUTURE_MS,
      result                : Err( "HTTP transport error: HTTP 401".to_string() ),
      account               : None,
      host                  : String::new(),
      role                  : String::new(),
      renewal_at            : None,
      cached                : false,
      cache_age_secs        : None,
      is_owned              : true,   // this machine owns the credentials
      owner                 : String::new(),
    };
    assert!(
      !should_refresh( &aq, 9_999 ),
      "BUG-303: owned+occupied account with 401 must NOT trigger should_refresh — \
       credential mutation while another machine uses this account would invalidate its live session",
    );
  }
}
