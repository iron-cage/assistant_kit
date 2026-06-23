// Path-referenced test module for refresh.rs — compiled as `mod tests` via `#[path]`.
// Lives in src/usage/ (not tests/) to access pub(crate) apply_refresh
// without widening its visibility. See src/usage/readme.md § Inline Test Exception.

  use super::apply_refresh;
  use crate::usage::types::{ AccountQuota, SubprocessModel, SubprocessEffort };
  use crate::usage::test_support::FAR_FUTURE_MS;
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
  /// `should_refresh` fires (`should_retry=true`); `crate::account::refresh_account_token`
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
  /// save); `apply_refresh` delegates via `crate::account::refresh_account_token`; skips the
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
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
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), "alice@example.com" ).unwrap();

    std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );

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
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → no loop body
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    assert!(
      !store.path().join( crate::account::active_marker_filename() ).exists(),
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
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
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), "alice@example.com\n" ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts : Vec< AccountQuota > = vec![];
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
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
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), ws ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts : Vec< AccountQuota > = vec![];
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
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
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), "alice@example.com" ).unwrap();
    let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → no loop body
    apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
    assert_eq!(
      active, "alice@example.com",
      "per-machine active marker must be unchanged when claude_paths=None (no restore possible)",
    );
  }

  /// L10 / FT-15 — `apply_refresh` lifecycle with `trace=true` reaching `run_isolated` invocation.
  ///
  /// `switch_account` succeeds (cred file in store, `.claude/` dir in `fake_home`).
  /// `run_isolated` is invoked but fails fast (no valid claude binary or fake token) →
  /// trace emits `[trace] … run_isolated: Err(…)` or `OK credentials=None` →
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
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
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

  /// FT-04 — `apply_refresh`: 429 + non-expired local token → NOT retried, result unchanged.
  ///
  /// `should_refresh` returns false when 429+non-expired (`expires_at_ms / 1000 > now_secs`):
  /// the local token is valid; the 429 is a genuine rate-limit, not a stale-credential
  /// condition.  `apply_refresh` skips `refresh_account_token` entirely (early `continue`).
  /// The 429 result is left unchanged.
  #[ test ]
  fn test_apply_refresh_ft4_429_valid_token_not_retried()
  {
    let store = TempDir::new().unwrap();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms : FAR_FUTURE_MS,  // non-expired → 429 is genuine rate-limit
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

    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "429" ) ),
      "429 with valid (non-expired) token must NOT be retried; result: {:?}",
      accounts[ 0 ].result,
    );
  }

  /// FT-05 — `apply_refresh` `None`-paths: 429 + expired local token → refresh path
  /// entered, but no credential file in store → `refresh_account_token` returns `None`
  /// → account skipped via `continue` → result unchanged.
  ///
  /// Contrasts with FT-04 (`test_apply_refresh_ft4_429_valid_token_not_retried`):
  ///   FT-04: 429 + non-expired → `should_refresh` returns `false` → refresh path NEVER entered.
  ///   FT-05: 429 + expired    → `should_refresh` returns `true`  → refresh path IS entered,
  ///          but gracefully exits when no per-account credential file exists in the store.
  #[ test ]
  fn test_apply_refresh_ft5_429_expired_refresh_path_entered_no_cred()
  {
    let store = TempDir::new().unwrap();
    // expires_at_ms=0 → 0/1000=0 ≤ now_secs → locally expired → should_refresh=true for 429.
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
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

    // Fix(BUG-297): no cred file → refresh_account_token returns None → result is now
    //   Err("refresh token expired"), not the original 429 error.
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
      "429+expired: no cred file → refresh_account_token None → \
       result must be Err(\"refresh token expired\"); result: {:?}",
      accounts[ 0 ].result,
    );
  }

  // ── BUG-170 MRE: jwt_exp_ms ──────────────────────────────────────────────────

  /// MRE 1/2 for BUG-170: `jwt_exp_ms` returns `None` for opaque `sk-ant-oat01-*` tokens.
  ///
  /// # Root Cause
  /// `jwt_exp_ms` splits `accessToken` on `.` via `splitn(3, '.')`. Opaque `sk-ant-oat01-*`
  /// tokens have no `.` separator — the second `parts.next()?` returns `None` and
  /// `jwt_exp_ms` returns `None`. The `if let Some` guard at `usage.rs:803-806` never fires,
  /// leaving `aq.expires_at_ms` at its stale pre-refresh expired timestamp.
  ///
  /// # Why Not Caught
  /// BUG-162 tests used synthetic JWT-format tokens. No test verified `jwt_exp_ms` behavior
  /// for opaque `sk-ant-oat01-*` tokens, nor that `expires_at_ms` is correct post-refresh
  /// when `jwt_exp_ms` returns `None`.
  ///
  /// # Fix Applied
  /// Fix(BUG-170): `parse_u64_from_str` fallback added after `jwt_exp_ms` in `apply_refresh`.
  /// This test guards the precondition: `jwt_exp_ms` correctly returns `None` for opaque tokens.
  ///
  /// # Prevention
  /// `jwt_exp_ms` returns `None` for any non-JWT token; this is by design. Never "fix"
  /// `jwt_exp_ms` to handle opaque tokens — the correct fix is a separate `expiresAt` fallback.
  ///
  /// # Pitfall
  /// If `jwt_exp_ms` is modified to handle opaque tokens directly (wrong fix), this test
  /// fails, alerting that the `parse_u64_from_str` fallback may be redundant. Preserve the
  /// two-step fallback design regardless — opaque tokens will never have a parseable JWT payload.
  #[ doc = "bug_reproducer(BUG-170)" ]
  #[ test ]
  fn test_jwt_exp_ms_mre_bug170_opaque_returns_none()
  {
    // Opaque sk-ant-oat01-* token: no '.' separator — splitn(3, '.') yields one part.
    let opaque_creds = r#"{"accessToken":"sk-ant-oat01-XXXXXXXXXXXX","expiresAt":9999999999999}"#;
    assert!(
      crate::output::jwt_exp_ms( opaque_creds ).is_none(),
      "jwt_exp_ms must return None for opaque sk-ant-oat01 token (no JWT structure); \
       if this fails, jwt_exp_ms was changed to handle opaque tokens — review BUG-170 fix",
    );
  }

  // ── FT-17 / BUG-211 MRE ──────────────────────────────────────────────────────

  /// FT-17 / BUG-211 MRE — `apply_refresh` does NOT write live credentials file (no `switch_account`).
  ///
  /// Fix(BUG-211): the snapshot+restore pattern was removed from `apply_refresh`. With
  /// `trace=true`, no `[trace] refresh  {name}  restore switch_account: OK/Err` line is
  /// emitted — the restore step no longer exists.
  ///
  /// # Root Cause
  /// The original `apply_refresh` called `switch_account(snapshot, ...)` after the
  /// per-account loop. This restore wrote the live credentials file and updated the active
  /// marker — creating a TOCTOU race with concurrent `.account.use` switches.
  ///
  /// # Why Not Caught
  /// BUG-208 tests verified that the restore EXECUTED (live creds written). No test verified
  /// that the live creds file is NOT written when the restore is absent — the previous
  /// `[trace]` guard was observing eprintln output which is not assertable in nextest.
  ///
  /// # Fix Applied
  /// BUG-211: removed snapshot+restore from `apply_refresh`; `refresh_account_token` passes
  /// `update_marker=false` to `save()` so background refresh never writes `_active`.
  ///
  /// # Prevention
  /// This test guards the absence of `switch_account` in the `apply_refresh` post-loop path:
  /// after a full cycle, `paths.credentials_file()` must NOT exist (`switch_account` not called),
  /// and the active marker must remain at its pre-call value.
  ///
  /// # Pitfall
  /// If restore is re-introduced, `credentials_file()` will exist after the call — the
  /// `!exists()` assertion is the regression guard. Marker assertion alone is insufficient
  /// because the marker is set to the same value by both restore and the pre-call write.
  #[ doc = "bug_reproducer(BUG-211)" ]
  #[ test ]
  fn test_apply_refresh_mre_bug208_restore_trace_emitted()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    // Alice's credential file in store — present but must NOT be copied to the live file.
    std::fs::write(
      store.path().join( "alice@example.com.credentials.json" ),
      r#"{"accessToken":"alice-restore-tok","expiresAt":9999999999999}"#,
    ).unwrap();

    std::fs::write(
      store.path().join( crate::account::active_marker_filename() ),
      "alice@example.com",
    ).unwrap();

    std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );

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

    // trace=true: Fix(BUG-211) — no restore switch_account; no [trace] restore line emitted.
    apply_refresh( &mut accounts, store.path(), Some( &paths ), true, SubprocessModel::Auto, SubprocessEffort::Auto, false );

    // Fix(BUG-211): no switch_account → live credentials file must NOT exist.
    assert!(
      !paths.credentials_file().exists(),
      "BUG-211: apply_refresh must not call switch_account; live credentials file must not exist",
    );

    // Active marker is unchanged (was "alice@example.com", never touched).
    let marker = std::fs::read_to_string(
      store.path().join( crate::account::active_marker_filename() )
    ).unwrap();
    assert_eq!(
      marker, "alice@example.com",
      "BUG-211: active marker must be unchanged after apply_refresh cycle (no restore)",
    );
  }

  // ── BUG-256 MRE: retry OK arm must clear cached metadata and write cache file ─

  /// MRE for BUG-256: retry OK arm clears `cached` flag, `cache_age_secs`, and writes
  /// fresh quota data to `{{name}}.json` via `write_quota_cache()`.
  ///
  /// # Root Cause
  /// The retry OK arm in `apply_refresh` only set `aq.result = Ok(retried)` — it did not
  /// clear `aq.cached` or `aq.cache_age_secs`, so `render.rs` kept the `~` prefix on every
  /// quota cell and the `(Xh ago)` age label on every row. `write_quota_cache` was also
  /// absent, so `{{name}}.json` retained stale cached quota across restarts. The bug was
  /// introduced by merge f83d78d (conflict resolution chose the remote branch, dropping the
  /// three mutations that were in 518d0a4).
  ///
  /// # Why Not Caught
  /// No test guarded the content of the retry OK arm. Mutations were dropped silently by
  /// a merge conflict resolution — only a source-structure assertion catches this class of
  /// omission.
  ///
  /// # Fix Applied
  /// Fix(BUG-256): in the retry OK arm of `apply_refresh`, extract h5/d7/sn references
  /// BEFORE moving `retried` into `aq.result`, then call `write_quota_cache`, and set
  /// `aq.cached = false` and `aq.cache_age_secs = None`.
  ///
  /// # Prevention
  /// This test greps the source of the retry OK arm for the three AC-11 mutations.
  /// Any merge conflict that drops them will cause this test to fail.
  ///
  /// # Pitfall
  /// The `write_quota_cache` call must appear BEFORE `aq.result = Ok( retried )` —
  /// h5/d7/sn borrow from `retried`; moving it first would be use-after-move.
  /// The order check below enforces this structural constraint statically.
  #[ doc = "bug_reproducer(BUG-256)" ]
  #[ test ]
  fn mre_bug256_retry_ok_stale_cached_metadata()
  {
    let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/refresh.rs" ) );
    let fn_start = src.find( "pub( crate ) fn apply_refresh(" ).expect( "apply_refresh not found" );

    // Locate the retry OK arm within the function body.
    let ok_arm_rel = src[ fn_start.. ]
      .find( "Ok( retried ) =>" )
      .expect( "BUG-256: retry OK arm `Ok( retried ) =>` not found in apply_refresh" );
    let ok_arm_start = fn_start + ok_arm_rel;

    // Bound the OK arm: ends at the `Err( e ) =>` arm that follows.
    let err_arm_rel = src[ ok_arm_start.. ]
      .find( "Err( e ) =>" )
      .expect( "Err arm not found after retry OK arm" );
    let ok_arm = &src[ ok_arm_start .. ok_arm_start + err_arm_rel ];

    // AC-11 check 1: aq.cached must be cleared to false.
    assert!(
      ok_arm.contains( "aq.cached         = false" ),
      "BUG-256: retry OK arm must set `aq.cached = false` to clear ~ prefix from render",
    );

    // AC-11 check 2: aq.cache_age_secs must be cleared to None.
    assert!(
      ok_arm.contains( "aq.cache_age_secs = None" ),
      "BUG-256: retry OK arm must set `aq.cache_age_secs = None` to remove (Xh ago) label",
    );

    // AC-11 check 3: write_quota_cache must be called with fresh data.
    assert!(
      ok_arm.contains( "write_quota_cache(" ),
      "BUG-256: retry OK arm must call write_quota_cache to persist fresh data to {{name}}.json",
    );

    // Order check: write_quota_cache must appear before the move of retried into aq.result.
    let cache_write_pos = ok_arm.find( "write_quota_cache(" ).unwrap();
    let result_move_pos = ok_arm.find( "aq.result         = Ok( retried )" )
      .expect( "aq.result = Ok( retried ) not found in retry OK arm" );
    assert!(
      cache_write_pos < result_move_pos,
      "BUG-256: write_quota_cache must appear before `aq.result = Ok( retried )` — \
       h5/d7/sn borrow from retried and would be use-after-move otherwise",
    );
  }

  // ── BUG-295 MRE: non-owned trace reason must be "not owned", not "ok" ────────

  /// MRE for BUG-295: `apply_refresh` emits `reason: not owned` (not `reason: ok`) when
  /// `aq.is_owned == false` and `aq.result` is `Ok(cached_data)`.
  ///
  /// # Root Cause
  /// For non-owned accounts, G1 in `fetch.rs` sets `aq.result = Ok(cached_data)` — the cache
  /// read succeeds. The original trace line derived the reason via
  /// `aq.result.as_ref().err().map_or("ok", String::as_str)` — for `Ok(...)`, `.err()` returns
  /// `None`, yielding `"ok"`. The actual reason the account is skipped is `"not owned"` (via
  /// `should_refresh()` returning `false` because `!aq.is_owned`), not the result value.
  ///
  /// # Why Not Caught
  /// No test captured the trace reason string for non-owned accounts. The misleading `reason: ok`
  /// was only visible during manual trace inspection.
  ///
  /// # Fix Applied
  /// Fix(BUG-295): before consulting `aq.result.err()`, check `!aq.is_owned`. When `is_owned`
  /// is `false`, emit `"not owned"` as the reason regardless of `aq.result`.
  ///
  /// # Prevention
  /// This test captures stderr from `apply_refresh(trace=true)` for a non-owned account with
  /// `result: Ok(cached_data)`. Asserts `reason: not owned` present and `reason: ok` absent.
  ///
  /// # Pitfall
  /// Hold `STDERR_LOCK` before `gag::BufferRedirect::stderr()` — concurrent gag captures
  /// corrupt each other via the shared fd 2.
  #[ doc = "bug_reproducer(BUG-295)" ]
  #[ test ]
  fn mre_bug295_apply_refresh_trace_reason_not_owned()
  {
    let store  = TempDir::new().unwrap();
    let cached = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![
      AccountQuota
      {
        name                  : "alice@remote.com".to_string(),
        is_current            : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms         : FAR_FUTURE_MS,
        result                : Ok( cached ),
        account               : None,
        host                  : String::new(),
        role                  : String::new(),
        renewal_at            : None,
        cached                : true,
        cache_age_secs        : Some( 120 ),
        is_owned              : false,
        owner                 : "other@remote".to_string(),
      },
    ];

    use std::io::Read;
    let _lock = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut buf = gag::BufferRedirect::stderr().unwrap();
    apply_refresh( &mut accounts, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let mut output = String::new();
    buf.read_to_string( &mut output ).unwrap();

    assert!(
      output.contains( "reason: not owned" ),
      "BUG-295: trace must emit 'reason: not owned' for non-owned account; got: {output}",
    );
    assert!(
      !output.contains( "reason: ok" ),
      "BUG-295: trace must NOT emit 'reason: ok' for non-owned account (misleading); got: {output}",
    );
  }

  // ── BUG-297 MRE: refresh None must set aq.result=Err ─────────────────────────

  /// # MRE: BUG-297 — `apply_refresh` None branch leaves `aq.result=Ok(cached_data)`, causing
  /// redundant `apply_touch` subprocess for RT-expired accounts.
  ///
  /// # Root Cause
  /// `refresh.rs:70-77` — the `else { continue; }` branch fires when `refresh_account_token`
  /// returns `None` (OAuth refresh token expired). Before the fix, it continued without mutating
  /// `aq.result`. Result stayed `Ok(cached_data)` — identical to a live healthy fetch — so
  /// `apply_touch` at `touch.rs:56` saw `Ok` and fired a redundant ~1.7s subprocess.
  ///
  /// # Why Not Caught
  /// No test covered the `should_refresh() → refresh_account_token() = None` path. The
  /// `else { continue; }` branch was only reachable with an unrecoverable RT expiry, which no
  /// unit test constructed. CI only exercised the happy path (Some(creds)) and the no-retry path
  /// (`should_refresh=false`).
  ///
  /// # Fix Applied
  /// Fix(BUG-297): added `aq.result = Err("refresh token expired".into());` before `continue;`
  /// in the None branch. Phase-contract invariant restored: every `continue` path in
  /// `apply_refresh` must leave `aq.result=Err` when the account cannot proceed.
  ///
  /// # Prevention
  /// This test uses an empty `TempDir` as `credential_store`, so `refresh_account_token` returns
  /// `None` (no credential file). `should_refresh` fires because `cached=true` and
  /// `expires_at_ms=0` triggers the BUG-255 guard. After `apply_refresh`, asserts
  /// `aq.result = Err("refresh token expired")`.
  ///
  /// # Pitfall
  /// The empty `TempDir` means no credential file exists for the account, so
  /// `read_token(credential_store, &aq.name)` returns `Err`, which makes
  /// `refresh_account_token` return `None` immediately — no subprocess is spawned.
  /// `is_owned=true` is required: non-owned accounts are skipped by `should_refresh`.
  #[ doc = "bug_reproducer(BUG-297)" ]
  #[ test ]
  fn mre_bug297_refresh_none_sets_aq_result_err()
  {
    let store       = TempDir::new().unwrap();
    let stale_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![
      AccountQuota
      {
        name                  : "rt-expired-acct".to_string(),
        is_current            : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms         : 0, // locally expired → should_refresh fires (BUG-255 guard)
        result                : Ok( stale_quota ),
        account               : None,
        host                  : String::new(),
        role                  : String::new(),
        renewal_at            : None,
        cached                : true, // cache masking active — result is Ok(stale)
        cache_age_secs        : Some( 7200 ),
        is_owned              : true, // required: non-owned accounts are skipped by should_refresh
        owner                 : String::new(),
      },
    ];

    apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "refresh token expired" ) ),
      "BUG-297: when refresh_account_token returns None, aq.result must be \
       Err(\"refresh token expired\"); got: {:?}",
      accounts[ 0 ].result,
    );
  }

  // ── BUG-297 pipeline: refresh None → touch skips (no subprocess) ─────────────

  /// Pipeline integration test for BUG-297 fix: after `apply_refresh` sets
  /// `aq.result = Err("refresh token expired")`, `apply_touch` must skip the account
  /// emitting `"skipped (reason: error account)"` and must NOT spawn a subprocess.
  ///
  /// # Root Cause
  /// Before the BUG-297 fix, `apply_refresh` left `aq.result=Ok(cached_data)` when
  /// `refresh_account_token` returned `None` (RT expired). `apply_touch` at `touch.rs:56`
  /// guards on `let Ok(ref data) = aq.result` — seeing `Ok`, it fired a redundant
  /// ~1.7s subprocess that also returned `None` with no useful effect.
  ///
  /// # Why Not Caught
  /// No test exercised the `apply_refresh → apply_touch` pipeline with a None-return path.
  /// The two phases were tested independently; the cross-phase contract (every `continue`
  /// path in `apply_refresh` must leave `aq.result=Err`) was untested.
  ///
  /// # Fix Applied
  /// Fix(BUG-297): `refresh.rs:76-81` now sets `aq.result = Err("refresh token expired")`
  /// before `continue;`. `apply_touch` at `touch.rs:56` sees `Err` and emits
  /// `"skipped (reason: error account)"` without attempting any subprocess.
  ///
  /// # Prevention
  /// This test runs both `apply_refresh` and then `apply_touch` on the same `AccountQuota`
  /// and asserts: (1) touch emits `"error account"` skip trace; (2) touch does NOT emit
  /// `"run_isolated: invoking"` (no subprocess spawned).
  ///
  /// # Pitfall
  /// `claude_paths=None` ensures no subprocess can fire regardless (no credential file path
  /// to switch to). The test confirms the *correct* skip reason fires (`"error account"` from
  /// the G1 error guard), not a later guard that would also prevent subprocess launch.
  #[ test ]
  fn apply_touch_skips_after_refresh_none()
  {
    use std::io::Read;

    let store       = TempDir::new().unwrap();
    let stale_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![
      AccountQuota
      {
        name                  : "rt-expired-touch-pipeline".to_string(),
        is_current            : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms         : 0,
        result                : Ok( stale_quota ),
        account               : None,
        host                  : String::new(),
        role                  : String::new(),
        renewal_at            : None,
        cached                : true,
        cache_age_secs        : Some( 7200 ),
        is_owned              : true,
        owner                 : String::new(),
      },
    ];

    // Phase 1: apply_refresh → sets aq.result = Err("refresh token expired").
    apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

    assert!(
      accounts[ 0 ].result.is_err(),
      "pipeline precondition: apply_refresh must set Err result; got: {:?}",
      accounts[ 0 ].result,
    );

    // Phase 2: apply_touch → must see Err and skip without spawning a subprocess.
    let _lock = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

    super::super::touch::apply_touch(
      &mut accounts[ 0 ],
      store.path(),
      None,
      true,
      SubprocessModel::Auto,
      SubprocessEffort::Auto,
      false,
    );

    let mut captured = String::new();
    buf.read_to_string( &mut captured ).unwrap();

    assert!(
      captured.contains( "error account" ),
      "BUG-297 pipeline: apply_touch must emit 'error account' skip after refresh None; got:\n{captured}",
    );
    assert!(
      !captured.contains( "run_isolated: invoking" ),
      "BUG-297 pipeline: apply_touch must NOT spawn subprocess after refresh None; got:\n{captured}",
    );
  }

  // ── BUG-298 MRE: owned+cached trace reason must be "cached-expired", not "ok" ──

  /// MRE for BUG-298: `apply_refresh` emits `reason: cached-expired` (not `reason: ok`) when
  /// `aq.is_owned == true`, `aq.cached == true`, and `aq.result` is `Ok(cached_data)`.
  ///
  /// # Root Cause
  /// `fetch.rs:229-240` cache fallback converts `Err→Ok` and sets `aq.cached = true`. The
  /// original trace reason expression `aq.result.as_ref().err().map_or("ok", …)` calls `.err()`
  /// on the now-`Ok` result — returning `None` — and produces the constant label `"ok"`. This is
  /// misleading: the actual trigger is the BUG-255 guard (`aq.cached && expired`), not a healthy
  /// fetch. A developer reading `reason: ok` cannot determine why refresh was attempted.
  ///
  /// # Why Not Caught
  /// BUG-295 fixed the `!aq.is_owned` branch of the same expression but reviewed it in isolation.
  /// The `aq.is_owned && aq.cached` case was not in scope for that fix and had no covering test.
  /// BUG-255 added the cached+expired predicate without auditing downstream trace labels.
  ///
  /// # Fix Applied
  /// Fix(BUG-298): `else if aq.cached { "cached-expired" }` branch added before the
  /// `aq.result.err()` expression in the trace reason computation at `refresh.rs`.
  ///
  /// # Prevention
  /// This test captures stderr from `apply_refresh(trace=true)` for an owned+cached+expired
  /// account. Asserts `reason: cached-expired` present; `reason: ok` absent.
  ///
  /// # Pitfall
  /// Hold `STDERR_LOCK` before `gag::BufferRedirect::stderr()` — concurrent gag captures corrupt
  /// each other via the shared fd 2. Any trigger path that converts Err→Ok (cache, synthetic
  /// data injection) must add its own reason branch before `aq.result.err()` at the trace site.
  #[ doc = "bug_reproducer(BUG-298)" ]
  #[ test ]
  fn mre_bug298_apply_refresh_trace_reason_cached_expired()
  {
    let store       = TempDir::new().unwrap();
    let stale_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![
      AccountQuota
      {
        name                  : "cached-owned@box.pro".to_string(),
        is_current            : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms         : 0, // expired → BUG-255 guard fires → should_refresh=true
        result                : Ok( stale_quota ), // cache fallback converted Err→Ok
        account               : None,
        host                  : String::new(),
        role                  : String::new(),
        renewal_at            : None,
        cached                : true,  // cache masking active
        cache_age_secs        : Some( 7200 ),
        is_owned              : true,  // required: non-owned skips with "not owned"
        owner                 : String::new(),
      },
    ];

    use std::io::Read;
    let _lock = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut buf = gag::BufferRedirect::stderr().unwrap();
    apply_refresh( &mut accounts, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let mut output = String::new();
    buf.read_to_string( &mut output ).unwrap();

    assert!(
      output.contains( "reason: cached-expired" ),
      "BUG-298: trace must emit 'reason: cached-expired' for owned+cached+expired account; got: {output}",
    );
    assert!(
      !output.contains( "reason: ok" ),
      "BUG-298: trace must NOT emit 'reason: ok' for owned+cached account (misleading); got: {output}",
    );
  }

  /// EC-7 (061): `apply_refresh` solo gate — non-current owned account is skipped with
  /// `[trace] refresh  {name}  solo-skip` when `solo=true`.
  ///
  /// With `solo=true`, the solo gate fires before G2 (non-owned check) for any account
  /// where `aq.is_current=false`. The account here is `is_owned=true` — without the solo
  /// gate it would proceed to the `should_refresh` check. With `solo=true` it is skipped
  /// immediately and the trace confirms the reason.
  ///
  /// Spec: [`tests/docs/cli/param/61_solo.md` EC-7]
  #[ test ]
  fn ec7_solo_gate_skips_non_current_with_trace()
  {
    use std::io::Read;

    let store = TempDir::new().unwrap();
    let mut accounts = vec![ AccountQuota
    {
      name                  : "noncurrent@example.com".to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms         : FAR_FUTURE_MS,
      result                : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
      account               : None,
      host                  : String::new(),
      role                  : String::new(),
      renewal_at            : None,
      cached                : false,
      cache_age_secs        : None,
      is_owned              : true,
      owner                 : String::new(),
    } ];

    let _lock = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut buf = gag::BufferRedirect::stderr().unwrap();
    apply_refresh( &mut accounts, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, true );
    let mut output = String::new();
    buf.read_to_string( &mut output ).unwrap();

    assert!(
      output.contains( "solo-skip" ),
      "EC-7: solo gate must emit 'solo-skip' trace for non-current account; got: {output}",
    );
    assert!(
      output.contains( "noncurrent@example.com" ),
      "EC-7: trace must name the skipped account; got: {output}",
    );
  }

  // ── GAP-20: trace reason "ok" for owned+non-cached+Ok ────────────────────────

  /// GAP-20 — `apply_refresh` trace reason is `"ok"` for owned, non-cached, `Ok` account.
  ///
  /// Path: `!is_owned` = false → `cached` = false → `result.err()` = None → `map_or("ok", …)` = `"ok"`.
  /// `should_refresh` returns `false` for this account (no auth error, not cached-expired),
  /// so the trace line `should_retry=false (reason: ok)` is emitted and the account is skipped.
  ///
  /// This test documents the CORRECT and EXPECTED behaviour, distinguishing the healthy
  /// non-retry path from the misleading-label bugs fixed in BUG-295 (non-owned) and
  /// BUG-298 (owned+cached).
  #[ test ]
  fn mre_bug_gap20_refresh_trace_reason_ok_owned_non_cached_ok()
  {
    use std::io::Read;

    let store    = TempDir::new().unwrap();
    let ok_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![ AccountQuota
    {
      name                  : "healthy@example.com".to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms         : FAR_FUTURE_MS,  // valid token → no expiry trigger
      result                : Ok( ok_quota ),
      account               : None,
      host                  : String::new(),
      role                  : String::new(),
      renewal_at            : None,
      cached                : false,  // non-cached → BUG-255 guard does not fire
      cache_age_secs        : None,
      is_owned              : true,   // owned → not "not owned"
      owner                 : String::new(),
    } ];

    let _lock = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut buf = gag::BufferRedirect::stderr().unwrap();
    apply_refresh( &mut accounts, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let mut output = String::new();
    buf.read_to_string( &mut output ).unwrap();

    assert!(
      output.contains( "reason: ok" ),
      "GAP-20: owned+non-cached+Ok account must emit 'reason: ok' trace; got: {output}",
    );
    assert!(
      !output.contains( "reason: not owned" ) && !output.contains( "reason: cached-expired" ),
      "GAP-20: must not emit non-owned or cached-expired reason for this path; got: {output}",
    );
  }

  // ── BUG-306 reproducer ──────────────────────────────────────────────────

  /// MRE — `reason_label` returns `"occupied elsewhere"` for owned, non-cached,
  /// occupied-elsewhere account with Ok result.
  ///
  /// # Root Cause
  /// The inline trace reason block at `refresh.rs:72-83` had three branches:
  /// `!is_owned` → `"not owned"`, `aq.cached` → `"cached-expired"`, else →
  /// `aq.result.err().map_or("ok", ...)`. An owned, non-cached, occupied-elsewhere
  /// account fell through to the else arm and showed `reason: ok` — actively
  /// misleading because the account was skipped by the G2 predicate gate.
  ///
  /// # Why Not Caught
  /// No test exercised the trace-reason path for the occupied-elsewhere predicate;
  /// all existing tests covered not-owned, cached-expired, and genuine-ok branches.
  ///
  /// # Fix Applied
  /// Extracted the inline block into `fn reason_label(aq: &AccountQuota) -> &'static str`
  /// with a new `else if aq.is_occupied_elsewhere { "occupied elsewhere" }` branch
  /// after `aq.cached`. Enforces predicate–reason 1:1 contract.
  ///
  /// # Prevention
  /// `reason_label` is a named function directly testable by unit test; future
  /// predicate additions must add a corresponding branch or this test class will
  /// expose the gap.
  ///
  /// # Pitfall
  /// Branch order matters: `is_occupied_elsewhere` must come after `cached` because
  /// cached accounts have their own trace reason regardless of occupancy status.
  #[ doc = "bug_reproducer(BUG-306)" ]
  #[ test ]
  fn mre_bug306_refresh_trace_reason_occupied_elsewhere()
  {
    let aq = AccountQuota
    {
      name                  : "occ@example.com".to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : true,
      expires_at_ms         : FAR_FUTURE_MS,
      result                : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
      account               : None,
      host                  : String::new(),
      role                  : String::new(),
      renewal_at            : None,
      cached                : false,
      cache_age_secs        : None,
      is_owned              : true,
      owner                 : String::new(),
    };
    assert_eq!( super::reason_label( &aq, 0 ), "occupied elsewhere" );
  }

  /// Regression — `reason_label` returns `"not owned"` for non-owned account.
  #[ test ]
  fn reason_label_not_owned()
  {
    let aq = AccountQuota
    {
      name : "x".into(), is_current : false, is_active : false,
      is_occupied_elsewhere : false, expires_at_ms : 0,
      result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
      account : None, host : String::new(), role : String::new(),
      renewal_at : None, cached : false, cache_age_secs : None,
      is_owned : false, owner : String::new(),
    };
    assert_eq!( super::reason_label( &aq, 0 ), "not owned" );
  }

  /// Regression — `reason_label` returns `"cached-expired"` for owned+cached account with expired token.
  #[ test ]
  fn reason_label_cached_expired()
  {
    let aq = AccountQuota
    {
      name : "x".into(), is_current : false, is_active : false,
      is_occupied_elsewhere : false, expires_at_ms : 0,
      result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
      account : None, host : String::new(), role : String::new(),
      renewal_at : None, cached : true, cache_age_secs : Some( 999 ),
      is_owned : true, owner : String::new(),
    };
    assert_eq!( super::reason_label( &aq, 1 ), "cached-expired" );
  }

  /// Regression — `reason_label` returns `"cached"` for owned+cached account with valid token.
  ///
  /// Distinguishes rate-limited cache fallback (token still valid, no refresh needed)
  /// from token-expired cache fallback (token expired, refresh attempted).
  #[ test ]
  fn reason_label_cached_valid()
  {
    let aq = AccountQuota
    {
      name : "x".into(), is_current : false, is_active : false,
      is_occupied_elsewhere : false, expires_at_ms : FAR_FUTURE_MS,
      result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
      account : None, host : String::new(), role : String::new(),
      renewal_at : None, cached : true, cache_age_secs : Some( 60 ),
      is_owned : true, owner : String::new(),
    };
    assert_eq!( super::reason_label( &aq, 9_999 ), "cached" );
  }

  /// Regression — `reason_label` returns `"ok"` for owned+non-cached+Ok account.
  #[ test ]
  fn reason_label_ok()
  {
    let aq = AccountQuota
    {
      name : "x".into(), is_current : false, is_active : false,
      is_occupied_elsewhere : false, expires_at_ms : 0,
      result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
      account : None, host : String::new(), role : String::new(),
      renewal_at : None, cached : false, cache_age_secs : None,
      is_owned : true, owner : String::new(),
    };
    assert_eq!( super::reason_label( &aq, 0 ), "ok" );
  }

  /// Regression — `reason_label` returns error string for owned+non-cached+Err account.
  #[ test ]
  fn reason_label_err()
  {
    let aq = AccountQuota
    {
      name : "x".into(), is_current : false, is_active : false,
      is_occupied_elsewhere : false, expires_at_ms : 0,
      result : Err( "HTTP 401 Unauthorized".to_string() ),
      account : None, host : String::new(), role : String::new(),
      renewal_at : None, cached : false, cache_age_secs : None,
      is_owned : true, owner : String::new(),
    };
    assert_eq!( super::reason_label( &aq, 0 ), "HTTP 401 Unauthorized" );
  }

