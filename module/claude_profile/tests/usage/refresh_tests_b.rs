// Integration tests for refresh.rs — Part B (split from src/usage/refresh_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::apply_refresh;
use claude_profile::usage::test_bridge::types::{ AccountQuota, SubprocessModel, SubprocessEffort };
use claude_profile::usage::test_bridge::FAR_FUTURE_MS;
use tempfile::TempDir;

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
      fallback_reason : None,
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
      claim_lock : false, reserve : false,
          org_created_at : None,
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
      fallback_reason : None,
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
      claim_lock : false, reserve : false,
          org_created_at : None,
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
    claude_profile::output::jwt_exp_ms( opaque_creds ).is_none(),
    "jwt_exp_ms must return None for opaque sk-ant-oat01 token (no JWT structure); \
     if this fails, jwt_exp_ms was changed to handle opaque tokens — review BUG-170 fix",
  );
}

// ── FT-17 / BUG-211 MRE ──────────────────────────────────────────────────────

/// FT-17 / BUG-211 MRE — `apply_refresh` does NOT write live credentials file (no `switch_account`).
///
/// Fix(BUG-211): the snapshot+restore pattern was removed from `apply_refresh`. With
/// `trace=true`, no restore `switch_account` trace line is
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
/// trace guard was observing eprintln output which is not assertable in nextest.
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
    store.path().join( claude_profile::account::active_marker_filename() ),
    "alice@example.com",
  ).unwrap();

  std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );

  // Bob has 401 but no credential file — refresh_account_token returns None, bob skipped.
  let mut accounts = vec![
    AccountQuota
    {
      fallback_reason : None,
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
      claim_lock : false, reserve : false,
          org_created_at : None,
    },
  ];

  // trace=true: Fix(BUG-211) — no restore switch_account; no restore trace line emitted.
  apply_refresh( &mut accounts, store.path(), Some( &paths ), true, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  // Fix(BUG-211): no switch_account → live credentials file must NOT exist.
  assert!(
    !paths.credentials_file().exists(),
    "BUG-211: apply_refresh must not call switch_account; live credentials file must not exist",
  );

  // Active marker is unchanged (was "alice@example.com", never touched).
  let marker = std::fs::read_to_string(
    store.path().join( claude_profile::account::active_marker_filename() )
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
  let fn_start = src.find( "pub fn apply_refresh(" ).expect( "apply_refresh not found" );

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
/// This test verifies `reason_label()` returns `"not owned"` for a non-owned account with
/// `result: Ok(cached_data)`. Converted from gag-based stderr capture to direct function
/// test — gag captures fd 2 at OS level but Rust test harness intercepts eprintln at IO layer.
///
/// # Pitfall
/// `reason_label` must check `!aq.is_owned` BEFORE consulting `aq.result.err()` — non-owned
/// accounts may have `result = Ok(cached_data)` set by the G1 cache read.
#[ doc = "bug_reproducer(BUG-295)" ]
#[ test ]
fn mre_bug295_apply_refresh_trace_reason_not_owned()
{
  let cached = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let aq = AccountQuota
  {
    fallback_reason : None,
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
    claim_lock : false, reserve : false,
      org_created_at : None,
  };

  let reason = claude_profile::usage::test_bridge::reason_label( &aq, 0 );
  assert_eq!(
    reason, "not owned",
    "BUG-295: reason_label must return 'not owned' for non-owned account, not 'ok'",
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
      fallback_reason : None,
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
      claim_lock : false, reserve : false,
          org_created_at : None,
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
/// This test runs `apply_refresh` and then asserts `touch_skip_reason` — the pure decision
/// function `apply_touch` calls first — returns the "error account" skip reason on the
/// resulting `AccountQuota`. Because `apply_touch` only reaches subprocess-spawning logic
/// when `touch_skip_reason` returns `None`, a `Some( .. )` result here structurally proves
/// no subprocess is invoked.
///
/// # Pitfall
/// Asserting only `is_err()` on Phase 1's output would not by itself prove *which* guard
/// fires in `touch_skip_reason` — the reason string must be checked for equality against
/// exactly `"skipped (reason: error account)"` (the G1 error guard), not merely `Some(_)`,
/// to rule out a later guard also matching by coincidence.
#[ test ]
fn apply_touch_skips_after_refresh_none()
{
  let store       = TempDir::new().unwrap();
  let stale_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let mut accounts = vec![
    AccountQuota
    {
      fallback_reason : None,
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
      claim_lock : false, reserve : false,
          org_created_at : None,
    },
  ];

  // Phase 1: apply_refresh → sets aq.result = Err("refresh token expired").
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  assert!(
    accounts[ 0 ].result.is_err(),
    "pipeline precondition: apply_refresh must set Err result; got: {:?}",
    accounts[ 0 ].result,
  );

  // Phase 2: touch_skip_reason must see Err and return the error-account skip reason.
  assert_eq!(
    claude_profile::usage::test_bridge::touch_skip_reason( &accounts[ 0 ], store.path(), false ),
    Some( "skipped (reason: error account)" ),
    "BUG-297 pipeline: apply_touch must skip with 'error account' reason after refresh None",
  );
}

// ── BUG-298 MRE: owned+cached trace reason must be "cached-expired", not "ok" ──

/// MRE for BUG-298: `apply_refresh` emits `reason: cached-expired` (not `reason: ok`) when
/// `aq.is_owned == true`, `aq.cached == true`, and `aq.result` is `Ok(cached_data)`.
///
/// # Root Cause
/// `fetch.rs:306-313` cache fallback converts `Err→Ok` and sets `aq.cached = true`. The
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
/// This test verifies `reason_label()` returns `"cached-expired"` for an owned+cached+expired
/// account with `result: Ok(stale_quota)`. Converted from gag-based stderr capture to direct
/// function test — gag captures fd 2 at OS level but Rust test harness intercepts eprintln.
///
/// # Pitfall
/// Any trigger path that converts `Err→Ok` (cache fallback, synthetic injection) must add
/// its own reason branch in `reason_label` before the `aq.result.err()` expression.
#[ doc = "bug_reproducer(BUG-298)" ]
#[ test ]
fn mre_bug298_apply_refresh_trace_reason_cached_expired()
{
  let stale_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let aq = AccountQuota
  {
    fallback_reason : None,
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
    claim_lock : false, reserve : false,
      org_created_at : None,
  };

  let reason = claude_profile::usage::test_bridge::reason_label( &aq, 1 );
  assert_eq!(
    reason, "cached-expired",
    "BUG-298: reason_label must return 'cached-expired' for owned+cached+expired, not 'ok'",
  );
}

/// EC-7 (061): `apply_refresh` solo gate — non-current owned account is skipped when `solo=true`.
///
/// Behavioral proof: an account with 401+expired would be refreshed without solo
/// (result changes to `Err("refresh token expired")` via BUG-297 None path).
/// With `solo=true`, the solo gate fires first — result stays at original `Err("401")`.
/// Converted from gag-based stderr capture — gag captures fd 2 at OS level but Rust test
/// harness intercepts eprintln at IO layer, making gag buffer always empty.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-7]
#[ test ]
fn ec7_solo_gate_skips_non_current_with_trace()
{
  let store = TempDir::new().unwrap();
  let mut accounts = vec![ AccountQuota
  {
    fallback_reason : None,
    name                  : "noncurrent@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : 0,  // expired → would trigger refresh without solo
    result                : Err( "HTTP transport error: HTTP 401".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  } ];

  // solo=true: solo gate fires for is_current=false → account skipped → result unchanged.
  apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, true );

  // Result must still be the original 401 — solo gate prevented refresh.
  // Without solo, should_refresh returns true (401+expired), refresh_account_token
  // returns None (empty store), result becomes Err("refresh token expired").
  assert!(
    matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "401" ) ),
    "EC-7: solo gate must skip non-current account, preserving original error; got: {:?}",
    accounts[ 0 ].result,
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
  let ok_quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  let aq = AccountQuota
  {
    fallback_reason : None,
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
    claim_lock : false, reserve : false,
      org_created_at : None,
  };

  // GAP-20: healthy owned+non-cached+Ok path must produce "ok" reason —
  // this is the CORRECT label, unlike BUG-295 (non-owned) and BUG-298 (cached).
  let reason = claude_profile::usage::test_bridge::reason_label( &aq, 0 );
  assert_eq!(
    reason, "ok",
    "GAP-20: reason_label must return 'ok' for owned+non-cached+Ok account",
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
    fallback_reason : None,
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
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!( claude_profile::usage::test_bridge::reason_label( &aq, 0 ), "occupied elsewhere" );
}

// ── BUG-333 reproducer ───────────────────────────────────────────────────

/// BUG-333 — `reason_label()` must check `is_occupied_elsewhere` BEFORE `cached`; an account
/// can be both cached AND occupied-elsewhere, and the reader needs the occupancy signal.
///
/// # Root Cause
/// The three-branch order (`!is_owned` → `cached` → `is_occupied_elsewhere` → else) predates
/// the occupancy predicate gate (BUG-303); no branch reorder happened when co-occurrence
/// became the near-universal case (feature/036 G1b) for occupied-elsewhere accounts after
/// their first fetch — such accounts read from cache on every subsequent call, so `cached`
/// is almost always `true` too, and the old order silently dropped the occupancy signal.
///
/// # Why Not Caught
/// No existing test constructed `cached: true` AND `is_occupied_elsewhere: true` together —
/// all prior tests (`reason_label_cached_expired`, `mre_bug306_*`) set only one flag at a time.
///
/// # Fix Applied
/// Reordered `reason_label()`'s branches: `is_occupied_elsewhere` now checked before `cached`.
///
/// # Prevention
/// Branch-priority label functions must be re-audited for co-occurrence whenever a new
/// predicate gate is added — coverage of each flag in isolation is not sufficient.
///
/// # Pitfall
/// This is the fourth recurrence against this same function/seam (BUG-295, BUG-298, BUG-306
/// are the three prior masked-label instances; see `docs/pitfall/007_label_selection_branch_priority_pitfalls.md`).
///
/// Spec: [`docs/invariant/012_label_selection_requires_cooccurrence_coverage.md`]
#[ doc = "bug_reproducer(BUG-333)" ]
#[ test ]
fn mre_bug333_occupied_elsewhere_not_masked_by_cached()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                  : "occ-cached@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : true,
    expires_at_ms         : 0, // expired → would say "cached-expired" if cached is checked first
    result                : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 999 ),
    is_owned              : true,
    owner                 : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!(
    claude_profile::usage::test_bridge::reason_label( &aq, 1 ),
    "occupied elsewhere",
    "BUG-333: reason_label must return 'occupied elsewhere' even when cached=true; \
     an account can be both cached AND occupied-elsewhere, and the occupancy signal must win",
  );
}

/// Regression — `reason_label` returns `"not owned"` for non-owned account.
#[ test ]
fn reason_label_not_owned()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name : "x".into(), is_current : false, is_active : false,
    is_occupied_elsewhere : false, expires_at_ms : 0,
    result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
    account : None, host : String::new(), role : String::new(),
    renewal_at : None, cached : false, cache_age_secs : None,
    is_owned : false, owner : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!( claude_profile::usage::test_bridge::reason_label( &aq, 0 ), "not owned" );
}

/// Regression — `reason_label` returns `"cached-expired"` for owned+cached account with expired token.
#[ test ]
fn reason_label_cached_expired()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name : "x".into(), is_current : false, is_active : false,
    is_occupied_elsewhere : false, expires_at_ms : 0,
    result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
    account : None, host : String::new(), role : String::new(),
    renewal_at : None, cached : true, cache_age_secs : Some( 999 ),
    is_owned : true, owner : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!( claude_profile::usage::test_bridge::reason_label( &aq, 1 ), "cached-expired" );
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
    fallback_reason : None,
    name : "x".into(), is_current : false, is_active : false,
    is_occupied_elsewhere : false, expires_at_ms : FAR_FUTURE_MS,
    result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
    account : None, host : String::new(), role : String::new(),
    renewal_at : None, cached : true, cache_age_secs : Some( 60 ),
    is_owned : true, owner : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!( claude_profile::usage::test_bridge::reason_label( &aq, 9_999 ), "cached" );
}

/// Regression — `reason_label` returns `"ok"` for owned+non-cached+Ok account.
#[ test ]
fn reason_label_ok()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name : "x".into(), is_current : false, is_active : false,
    is_occupied_elsewhere : false, expires_at_ms : 0,
    result : Ok( claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
    account : None, host : String::new(), role : String::new(),
    renewal_at : None, cached : false, cache_age_secs : None,
    is_owned : true, owner : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!( claude_profile::usage::test_bridge::reason_label( &aq, 0 ), "ok" );
}

/// Regression — `reason_label` returns error string for owned+non-cached+Err account.
#[ test ]
fn reason_label_err()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name : "x".into(), is_current : false, is_active : false,
    is_occupied_elsewhere : false, expires_at_ms : 0,
    result : Err( "HTTP 401 Unauthorized".to_string() ),
    account : None, host : String::new(), role : String::new(),
    renewal_at : None, cached : false, cache_age_secs : None,
    is_owned : true, owner : String::new(),
    claim_lock : false, reserve : false,
      org_created_at : None,
  };
  assert_eq!( claude_profile::usage::test_bridge::reason_label( &aq, 0 ), "HTTP 401 Unauthorized" );
}

