// Integration tests for refresh.rs — Part A (split from src/usage/refresh_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::apply_refresh;
use claude_profile::usage::test_bridge::types::{ AccountQuota, SubprocessModel, SubprocessEffort };
use claude_profile::usage::test_bridge::FAR_FUTURE_MS;
use tempfile::TempDir;

// ── apply_refresh ──────────────────────────────────────────────────────────

/// T01 — `apply_refresh` leaves a 429 error result unchanged (no retry path).
///
/// # Root Cause
/// In task 142, `apply_refresh`'s retry guard included `e.contains("429")` alongside
/// `"401"` and `"403"`. HTTP 429 is a rate-limit response (token is still valid); retrying
/// on 429 triggers an unnecessary token refresh. Task 143 removed 429 from the guard at
/// `usage.rs` line 634, leaving only auth-failure codes (401, 403) as retry triggers.
///
/// # Why Not Caught
/// No test existed for `apply_refresh` behavior with 429 errors before task 143; the guard
/// was added in task 142 without a companion test proving 429 is passed through unchanged.
///
/// # Fix Applied
/// Removed `e.contains("429")` from the retry guard; guard is now
/// `Err(ref e) if e.contains("401") || e.contains("403")` only.
///
/// # Prevention
/// This test verifies the result string is identical after `apply_refresh`, acting as a
/// regression guard against re-adding 429 to the retry trigger conditions.
///
/// # Pitfall
/// Without a credential file in the store, the retry body is unreachable regardless of the
/// guard — `apply_refresh` cannot attempt a refresh and leaves the result unchanged either
/// way. This test validates the guard does not corrupt the result, but is NOT a full guard
/// against re-adding 429: even with the bug restored, this test would still pass (no creds).
/// The `shorten_error` test (T04) provides the stronger behavioral invariant.
#[ doc = "bug_reproducer(BUG-271)" ]
#[ test ]
fn test_apply_refresh_429_not_retried()
{
  let store = TempDir::new().unwrap();
  let mut accounts = vec![
    AccountQuota
    {
      name          : "test-acct".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];

  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  // Fix(BUG-297): 429+expired fires should_refresh → refresh_account_token returns None
  //   (no cred file) → result is now Err("refresh token expired"), not the original 429 error.
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "429+expired: no cred file → refresh_account_token returns None → \
     result must be Err(\"refresh token expired\"); result: {:?}", accounts[ 0 ].result,
  );
}

/// B2 — `apply_refresh` does not corrupt a successful Ok result.
///
/// An account with a valid quota result must remain Ok after `apply_refresh`;
/// the guard only fires on Err results containing "401" or "403".
#[ test ]
fn test_apply_refresh_ok_result_unchanged()
{
  let store = TempDir::new().unwrap();
  let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let mut accounts = vec![
    AccountQuota
    {
      name          : "ok-acct".to_string(),
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
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  assert!( accounts[ 0 ].result.is_ok(), "Ok result must not be changed by apply_refresh" );
}

/// B3 — `apply_refresh` leaves a generic network error unchanged (not an auth error).
///
/// Only "401" and "403" substrings trigger the retry guard; unrelated error
/// strings pass through without entering the retry path.
#[ test ]
fn test_apply_refresh_generic_error_unchanged()
{
  let store   = TempDir::new().unwrap();
  let err_msg = "network timeout after 30s".to_string();
  let mut accounts = vec![
    AccountQuota
    {
      name          : "net-acct".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( err_msg.clone() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e == &err_msg ),
    "generic error must be unchanged; result: {:?}", accounts[ 0 ].result,
  );
}

// ── apply_refresh: corner cases ─────────────────────────────────────────────

/// C1 — `apply_refresh` on an empty accounts slice is a no-op.
#[ test ]
fn test_apply_refresh_empty_accounts()
{
  let store = TempDir::new().unwrap();
  let mut accounts : Vec< AccountQuota > = vec![];
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  assert!( accounts.is_empty(), "empty slice must remain empty" );
}

/// C2 / FT-14 — `apply_refresh` `None`-paths: 401 + no credential file → result unchanged.
///
/// `should_refresh` fires (`should_retry=true`); `claude_profile::account::refresh_account_token`
/// is called with `paths=None`; internally it reads `{store}/{name}.credentials.json`
/// which is absent, so it returns `None`; `apply_refresh` skips the account via
/// `continue` without modifying the result.
#[ test ]
fn test_apply_refresh_401_no_cred_file()
{
  let store = TempDir::new().unwrap();
  let mut accounts = vec![
    AccountQuota
    {
      name          : "ghost@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  // Fix(BUG-297): 401 fires should_refresh → refresh_account_token returns None
  //   (no cred file) → result is now Err("refresh token expired"), not the original 401 error.
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "401: no cred file → refresh_account_token returns None → \
     result must be Err(\"refresh token expired\"); result: {:?}", accounts[ 0 ].result,
  );
}

/// C3 — `apply_refresh` with 403 error but no credential file on disk.
///
/// Same as C2 but with HTTP 403. Both 401 and 403 are auth-error triggers,
/// but without a credential file the retry body is unreachable.
#[ test ]
fn test_apply_refresh_403_no_cred_file()
{
  let store = TempDir::new().unwrap();
  let mut accounts = vec![
    AccountQuota
    {
      name          : "ghost@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 403".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  // Fix(BUG-297): 403 fires should_refresh → refresh_account_token returns None
  //   (no cred file) → result is now Err("refresh token expired"), not the original 403 error.
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "403: no cred file → refresh_account_token returns None → \
     result must be Err(\"refresh token expired\"); result: {:?}", accounts[ 0 ].result,
  );
}

/// C4 / FT-07 — `apply_refresh` with mixed results: refresh failure does not affect siblings.
///
/// Four accounts: Ok, 429+expired (`expires_at_ms=0`), 401, generic error.
/// After `apply_refresh`, the 401 and the 429+expired accounts enter the retry guard
/// but stay unchanged (no credential file → `refresh_account_token` returns `None`
/// → `continue`).  Ok and generic error are untouched (Ok never retries; generic
/// error has no auth/429 signal).  Implements FT-07: refresh failure in one account
/// does not corrupt any sibling's result.
#[ test ]
fn test_apply_refresh_mixed_accounts()
{
  let store = TempDir::new().unwrap();
  let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let mut accounts = vec![
    AccountQuota
    {
      name          : "a@ok.com".to_string(),
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
      is_owned       : true,
      owner                : String::new(),
    },
    AccountQuota
    {
      name          : "b@ratelimited.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
    AccountQuota
    {
      name          : "c@expired.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
    AccountQuota
    {
      name          : "d@network.com".to_string(),
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
      is_owned       : true,
      owner                : String::new(),
    },
  ];

  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  assert!( accounts[ 0 ].result.is_ok(), "Ok account must remain Ok" );
  // Fix(BUG-297): 429+expired and 401 both fire should_refresh → refresh_account_token
  //   returns None (no cred file) → result is now Err("refresh token expired").
  assert!(
    matches!( accounts[ 1 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "429+expired: no cred file → result must be Err(\"refresh token expired\"); result: {:?}",
    accounts[ 1 ].result,
  );
  assert!(
    matches!( accounts[ 2 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "401: no cred file → result must be Err(\"refresh token expired\"); result: {:?}",
    accounts[ 2 ].result,
  );
  assert!(
    matches!( accounts[ 3 ].result, Err( ref e ) if e == "connection refused" ),
    "generic error must be unchanged",
  );
}

/// C5 — `apply_refresh` with trace=true does not panic.
///
/// Verifies the trace code path executes without crashing, even when the
/// credential file is absent and the retry path short-circuits.
#[ test ]
fn test_apply_refresh_trace_does_not_panic()
{
  let store = TempDir::new().unwrap();
  let mut accounts = vec![
    AccountQuota
    {
      name          : "trace@test.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
}

// ── apply_refresh: lifecycle (Some(paths)) ──────────────────────────────────

/// L1 — `apply_refresh` skips lifecycle path when `switch_account` fails (no cred file).
///
/// # Root Cause
/// Before BUG-165, `apply_refresh` bypassed `switch_account` entirely, writing credentials
/// directly to the persistent store while leaving the live session stale. After the fix,
/// `apply_refresh` calls `switch_account` first when `claude_paths` is `Some`; if it fails
/// (account not found in store), the account is skipped and its error result is left unchanged.
///
/// # Why Not Caught
/// All prior inline tests passed `apply_refresh(..., None, ...)`, exercising only the `None`
/// (fallback/test) branch. Zero tests exercised `Some(paths)` (lifecycle/production branch).
///
/// # Fix Applied
/// BUG-165: extracted `refresh_account_token` (full lifecycle: switch → refresh →
/// save); `apply_refresh` delegates via `claude_profile::account::refresh_account_token`; skips the
/// account with `continue` if `refresh_account_token` returns `None`.
///
/// # Prevention
/// This test guards the `Some(paths)` early-exit: when the credential file is absent,
/// `refresh_account_token` returns `None` and `apply_refresh` must `continue` without
/// corrupting the account result.
///
/// # Pitfall
/// Tests where the credential file exists will reach `refresh_account_token`, which internally
/// spawns the `claude` binary and blocks for up to 35 s. Only test scenarios where the
/// credential file is absent (causing `None` early-exit) to avoid subprocess blocking.
#[ test ]
fn test_apply_refresh_lifecycle_switch_fails_result_unchanged()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // No alice@example.com.credentials.json in store — switch_account returns NotFound.
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts = vec![
    AccountQuota
    {
      name          : "alice@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];

  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  // Fix(BUG-297): switch_account fails (no cred file) → refresh_account_token returns None
  //   → result is now Err("refresh token expired"), not the original 401 error.
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "lifecycle: 401 + switch fails → refresh_account_token None → \
     result must be Err(\"refresh token expired\"); result: {:?}",
    accounts[ 0 ].result,
  );
}

/// L2 — `apply_refresh` restores the original active account after the refresh cycle.
///
/// FT-13 / BUG-211 MRE — `apply_refresh` does NOT call `switch_account` after per-account cycling.
///
/// Fix(BUG-211): the snapshot+restore pattern was removed from `apply_refresh`.
/// `refresh_account_token` passes `update_marker=false` to `save()`, so `_active` is
/// never written during per-account cycling — no restore is needed or performed.
///
/// # Root Cause
/// The original `apply_refresh` snapshotted the active marker before the loop and called
/// `switch_account(snapshot, ...)` after the loop. This created a TOCTOU race:
/// a concurrent `.account.use` switch during the ~35s subprocess window was silently
/// overwritten by the post-loop restore.
///
/// # Why Not Caught
/// All prior tests verified that the restore SUCCEEDED (live creds file written, marker
/// restored). No test verified that the live creds file was NOT written when no restore
/// should occur — making the absence of side-effects the guard.
///
/// # Fix Applied
/// BUG-211: removed snapshot+restore from `apply_refresh`; `refresh_account_token` now
/// passes `update_marker=false` to `save()` so background refresh never writes `_active`.
///
/// # Prevention
/// This test guards absence of `switch_account` in `apply_refresh`: after a full refresh
/// cycle, the live credentials file must NOT exist (no `switch_account` wrote it) and the
/// active marker must be unchanged from its pre-call value.
///
/// # Pitfall
/// If snapshot+restore is re-introduced into `apply_refresh`, this test fails because
/// `switch_account` writes the live credentials file — the `!credentials_file().exists()`
/// assertion is the critical guard for regression.
#[ doc = "bug_reproducer(BUG-211)" ]
#[ test ]
fn test_apply_refresh_lifecycle_active_marker_unchanged()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();

  // Alice's credential file in store — present but must NOT be copied to the live file.
  std::fs::write(
    store.path().join( "alice@example.com.credentials.json" ),
    r#"{"accessToken":"alice-token"}"#,
  ).unwrap();

  // Set active account to alice before the loop.
  std::fs::write( store.path().join( claude_profile::account::active_marker_filename() ), "alice@example.com" ).unwrap();

  std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );

  // Bob has 401 but no credential file — refresh_account_token returns None, bob skipped.
  let mut accounts = vec![
    AccountQuota
    {
      name          : "bob@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];

  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  // Fix(BUG-211): no switch_account in apply_refresh → live credentials file must NOT exist.
  assert!(
    !paths.credentials_file().exists(),
    "BUG-211: apply_refresh must not call switch_account; live credentials file must not exist",
  );

  // Active marker is unchanged (set to "alice@example.com" before call, never touched).
  let active = std::fs::read_to_string( store.path().join( claude_profile::account::active_marker_filename() ) ).unwrap();
  assert_eq!(
    active, "alice@example.com",
    "per-machine active marker must be unchanged throughout refresh cycle (BUG-211 fix)",
  );
}

/// L3 — `apply_refresh` lifecycle: 429+expired + `Some(paths)` + no cred file → skipped.
///
/// 429 with an expired local token meets `should_refresh` but `switch_account` fails
/// (no cred file in the persistent store), so the account is skipped and the result
/// is left unchanged — same guarantee as L1 but for the 429+expired trigger path.
#[ test ]
fn test_apply_refresh_lifecycle_429_expired_switch_fails_unchanged()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts = vec![
    AccountQuota
    {
      name          : "alice@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,  // expired: 0/1000=0 <= now_secs
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  // Fix(BUG-297): switch_account fails (no cred file) → refresh_account_token returns None
  //   → result is now Err("refresh token expired"), not the original 429 error.
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "lifecycle: 429+expired + switch fails → refresh_account_token None → \
     result must be Err(\"refresh token expired\"); result: {:?}",
    accounts[ 0 ].result,
  );
}

/// FT-03 — `apply_refresh` lifecycle: 403 + `Some(paths)` + no cred file → result unchanged.
///
/// 403 meets `should_refresh` (authentication failure, identical to 401) but
/// `switch_account` fails (no credential file in store), so `refresh_account_token`
/// returns `None` and `apply_refresh` skips the account via `continue`.  The 403
/// result is left unchanged — confirms 403 enters the refresh path, not the
/// non-trigger `continue` guard.
#[ test ]
fn test_apply_refresh_lifecycle_ft3_403_no_cred_result_unchanged()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // No alice@example.com.credentials.json — switch_account returns NotFound.
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts = vec![
    AccountQuota
    {
      name          : "alice@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,  // non-expired; 403 triggers regardless of expiry
      result        : Err( "HTTP transport error: HTTP 403".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];

  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  // Fix(BUG-297): switch_account fails (no cred file) → refresh_account_token returns None
  //   → result is now Err("refresh token expired"), not the original 403 error.
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "lifecycle: 403 + switch fails → refresh_account_token None → \
     result must be Err(\"refresh token expired\"); result: {:?}",
    accounts[ 0 ].result,
  );
}

/// L4 — `apply_refresh` lifecycle: cred file exists but `{home}/.claude/` dir missing
/// → `fs::copy` fails inside `switch_account` → account is skipped, result unchanged.
///
/// `switch_account` copies the credential to a temp file inside `{home}/.claude/`.
/// If that directory does not exist, `fs::copy` returns an `Err`, causing `apply_refresh`
/// to `continue` without modifying the account result.
#[ test ]
fn test_apply_refresh_lifecycle_copy_fails_no_dot_claude_dir()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // Cred file exists — check_switch_preconditions passes.
  std::fs::write(
    store.path().join( "alice@example.com.credentials.json" ),
    r#"{"accessToken":"tok"}"#,
  ).unwrap();
  // {fake_home}/.claude/ deliberately NOT created → fs::copy target parent missing.
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts = vec![
    AccountQuota
    {
      name          : "alice@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  // Fix(BUG-297): fs::copy fails (no .claude/ dir) → switch_account returns Err
  //   → refresh_account_token returns None → result is now Err("refresh token expired").
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
    "lifecycle: 401 + fs::copy fails (no .claude/ dir) → refresh_account_token None → \
     result must be Err(\"refresh token expired\"); result: {:?}",
    accounts[ 0 ].result,
  );
}

/// L5 — `apply_refresh` does not create the active marker file when it was absent before.
///
/// Fix(BUG-211): `apply_refresh` no longer reads or writes `_active`. If no marker file
/// exists before the call, none is created after — the function never touches the marker.
#[ test ]
fn test_apply_refresh_lifecycle_no_active_file_no_restore()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → no loop body
  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  assert!(
    !store.path().join( claude_profile::account::active_marker_filename() ).exists(),
    "per-machine active marker must not be created when it was absent before apply_refresh",
  );
}

/// L6 — `apply_refresh` with `trace=true` and refresh skip (no cred file) does not panic.
///
/// Exercises the trace code path for the refresh loop: `should_refresh` triggers for the
/// 401 account, `refresh_account_token` is called, `switch_account` fails inside it
/// (no cred file), `refresh_account_token` returns `None`, and `apply_refresh` skips
/// the account. Fix(BUG-211): no post-loop restore; the function returns cleanly.
#[ test ]
fn test_apply_refresh_lifecycle_trace_switch_fails_no_panic()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts = vec![
    AccountQuota
    {
      name          : "trace@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  // Must not panic — switch_account fails (no cred file), trace logs to stderr.
  apply_refresh( &mut accounts, store.path(), Some( &paths ), true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
}

/// L7 — active marker with trailing newline is unchanged after `apply_refresh` (no restore).
///
/// Fix(BUG-211): `apply_refresh` no longer reads or writes `_active`. A marker written
/// as `"alice@example.com\n"` before the call remains exactly `"alice@example.com\n"` after —
/// no trim, no `switch_account`, no modification of any kind.
#[ test ]
fn test_apply_refresh_lifecycle_active_newline_trimmed_restore()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  std::fs::write( store.path().join( claude_profile::account::active_marker_filename() ), "alice@example.com\n" ).unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts : Vec< AccountQuota > = vec![];
  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  let active = std::fs::read_to_string( store.path().join( claude_profile::account::active_marker_filename() ) ).unwrap();
  assert_eq!(
    active, "alice@example.com\n",
    "active marker must be unchanged after apply_refresh (BUG-211 fix: no restore); got: {active:?}",
  );
}

/// L8 — `apply_refresh` leaves an existing active marker file with whitespace-only content unchanged.
///
/// Fix(BUG-211): `apply_refresh` never reads or writes `_active`. A pre-existing whitespace-
/// only marker remains exactly as written — no trim, no `switch_account`, no modification.
#[ test ]
fn test_apply_refresh_lifecycle_active_whitespace_only_no_restore()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let ws = "   \n  ";
  std::fs::write( store.path().join( claude_profile::account::active_marker_filename() ), ws ).unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts : Vec< AccountQuota > = vec![];
  apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  let active = std::fs::read_to_string( store.path().join( claude_profile::account::active_marker_filename() ) ).unwrap();
  assert_eq!(
    active, ws,
    "apply_refresh must not modify the active marker file (BUG-211 fix); content must be unchanged",
  );
}

/// L9 — `claude_paths = None`: active marker file is unchanged after `apply_refresh`.
///
/// Fix(BUG-211): `apply_refresh` never reads or writes `_active` regardless of whether
/// `claude_paths` is `Some` or `None`. A pre-existing marker is unchanged in both cases.
#[ test ]
fn test_apply_refresh_none_paths_active_unchanged()
{
  let store = TempDir::new().unwrap();
  std::fs::write( store.path().join( claude_profile::account::active_marker_filename() ), "alice@example.com" ).unwrap();
  let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → no loop body
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  let active = std::fs::read_to_string( store.path().join( claude_profile::account::active_marker_filename() ) ).unwrap();
  assert_eq!(
    active, "alice@example.com",
    "per-machine active marker must be unchanged when claude_paths=None (no restore possible)",
  );
}

/// L10 / FT-15 — `apply_refresh` lifecycle with `trace=true` reaching `run_isolated` invocation.
///
/// `switch_account` succeeds (cred file in store, `.claude/` dir in `fake_home`).
/// `run_isolated` is invoked but fails fast (no valid claude binary or fake token) →
/// trace emits `YYYY-MM-DD · HH:MM:SS · … run_isolated: Err(…)` or `OK credentials=None` →
/// `refresh_account_token` returns `None` → account skipped → no panic.
///
/// # Root Cause
/// Before BUG-166, `refresh_account_token` had no `trace` parameter. The `apply_refresh`
/// `trace` arg was accepted but never forwarded, making the lifecycle completely opaque:
/// all failure paths returned `None` silently. Running `clp .usage refresh::1 trace::1`
/// showed only "refresh returned None — skipping retry" with no step-level detail.
///
/// # Why Not Caught
/// The trace parameter existed in `apply_refresh` but there were no tests verifying
/// it actually reached `refresh_account_token`. Silent pass-through was undetectable.
///
/// # Fix Applied
/// BUG-166: added `trace: bool` as a 4th parameter to `refresh_account_token`;
/// replaced all bare `?` operators with explicit `match` + `if trace { eprintln!(...) }` blocks.
///
/// # Prevention
/// This test guards the full call chain: `apply_refresh(trace=true)` →
/// `refresh_account_token(trace=true)` → `run_isolated` invocation. If the trace
/// parameter is ever dropped between layers, this test still passes (no panic),
/// but the trace output would be missing. The `account_refresh_test::art_some_paths_run_isolated_invoked_trace_no_panic`
/// test covers the `refresh_account_token` function directly.
///
/// # Pitfall
/// Tests using "does not panic" cannot assert stderr content — nextest does not
/// capture `eprintln!` output for unit test assertions. This is the correct pattern
/// for trace tests.
#[ test ]
fn test_apply_refresh_lifecycle_l10_trace_run_isolated_invoked_no_panic()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // Cred file in store AND .claude/ dir present — switch_account succeeds.
  std::fs::write(
    store.path().join( "alice@example.com.credentials.json" ),
    r#"{"accessToken":"fake-tok","expiresAt":9999999999999}"#,
  ).unwrap();
  std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );
  let mut accounts = vec![
    AccountQuota
    {
      name          : "alice@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : 0,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
      cached        : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    },
  ];
  // Must not panic — switch_account succeeds; run_isolated invoked; fails fast (fake creds).
  apply_refresh( &mut accounts, store.path(), Some( &paths ), true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
}

