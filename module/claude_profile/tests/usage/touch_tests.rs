// Integration tests for touch.rs — relocated from src/usage/touch_tests.rs.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::apply_touch;
use claude_profile::usage::test_bridge::types::{ SubprocessModel, SubprocessEffort };
use claude_profile::usage::test_bridge::mk_aq_with_resets_at;
use tempfile::TempDir;

/// FT-15 / BUG-211 MRE — `apply_touch` does NOT write live credentials file (no `switch_account`).
///
/// Fix(BUG-211): the snapshot+restore pattern was removed from `apply_touch`. With
/// `trace=true`, no restore `switch_account` trace line is
/// emitted — the restore step no longer exists.
///
/// # Root Cause
/// The original `apply_touch` read the active marker before calling `refresh_account_token`
/// and then unconditionally called `switch_account(snapshot, ...)` after. This restore
/// created a TOCTOU race: a concurrent `.account.use` switch during the ~35s subprocess
/// window was silently overwritten by the restore.
///
/// # Why Not Caught
/// BUG-208 tests verified that the restore EXECUTED (live creds written). No test verified
/// that the live creds file is NOT written when the restore is absent.
///
/// # Fix Applied
/// BUG-211: removed snapshot+restore from `apply_touch`; `refresh_account_token` passes
/// `update_marker=false` to `save()` so background touch never writes `_active`.
///
/// # Prevention
/// This test guards absence of `switch_account` in `apply_touch`: after a touch cycle,
/// `paths.credentials_file()` must NOT exist (no `switch_account` was called), and the
/// active marker must remain at its pre-call value.
///
/// # Pitfall
/// If restore is re-introduced, `credentials_file()` will exist after the call — the
/// `!exists()` assertion is the regression guard.
#[ doc = "bug_reproducer(BUG-211)" ]
#[ test ]
fn test_apply_touch_mre_bug208_restore_trace_emitted()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();

  // Alice's credential file in store — present but must NOT be copied to the live file.
  std::fs::write(
    store.path().join( "alice@example.com.credentials.json" ),
    r#"{"accessToken":"alice-touch-restore-tok","expiresAt":9999999999999}"#,
  ).unwrap();

  std::fs::write(
    store.path().join( claude_profile::account::active_marker_filename() ),
    "alice@example.com",
  ).unwrap();

  std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
  let paths = claude_profile::ClaudePaths::with_home( fake_home.path() );

  // test@example.com is idle (resets_at=None) — triggers apply_touch.
  // No credential file for test@example.com — refresh_account_token returns None.
  let mut aq = mk_aq_with_resets_at( None );

  // trace=true: Fix(BUG-211) — no restore; no restore switch_account trace line emitted.
  apply_touch( &mut aq, store.path(), Some( &paths ), true, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  // Fix(BUG-211): no switch_account → live credentials file must NOT exist.
  assert!(
    !paths.credentials_file().exists(),
    "BUG-211: apply_touch must not call switch_account; live credentials file must not exist",
  );

  // Active marker is unchanged (was "alice@example.com", never touched).
  let marker = std::fs::read_to_string(
    store.path().join( claude_profile::account::active_marker_filename() )
  ).unwrap();
  assert_eq!(
    marker, "alice@example.com",
    "BUG-211: active marker must be unchanged after apply_touch cycle (no restore)",
  );
}

/// BUG-214 MRE: `apply_touch` must skip idle accounts with 7d quota fully exhausted.
///
/// # Root Cause
///
/// `apply_touch()` guard checked `five_hour_left(aq) <= 15.0` (h-exhausted) but not
/// `seven_day_left(aq) <= 0.0`. An idle account with `seven_day.utilization=100%` passed
/// the guard → `refresh_account_token` called → subprocess fired → HTTP 429 (~2.3s
/// penalty with no session opened). No "7d-exhausted" trace emitted.
///
/// # Why Not Caught
///
/// The h-exhausted guard was added in isolation without extending the test surface to
/// weekly exhaustion as a skip criterion. `tests/docs/feature/024_session_touch.md`
/// FT-16 was absent until this session.
///
/// # Fix Applied
///
/// Added `seven_day_left()` helper + extended guard:
/// `!is_idle || h_left <= 15.0 || d7_left <= 0.0`. Three-arm reason string:
/// "already active" / "h-exhausted" / "7d-exhausted".
///
/// # Prevention
///
/// Any new quota dimension added to `apply_touch` skip logic must have a corresponding
/// FT in `tests/docs/feature/024_session_touch.md` before implementation.
///
/// # Pitfall
///
/// `mk_aq_with_resets_at(None)` sets `seven_day=None`. `seven_day_left()` treats
/// `seven_day=None` as 100.0 left (not exhausted — absent data ≠ exhausted). Must
/// explicitly set `seven_day=Some(PeriodUsage{utilization:100.0,...})` to express
/// "fully exhausted with data present". Using `None` would test the wrong code path.
#[ doc = "bug_reproducer(BUG-214)" ]
#[ test ]
fn test_mre_bug214_apply_touch_skips_7d_exhausted_account()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();

  // idle (resets_at=None → five_hour_left=50% → NOT h-exhausted)
  // but 7d fully exhausted (utilization=100% → seven_day_left=0%).
  let mut aq = mk_aq_with_resets_at( None );
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day = Some( claude_quota::PeriodUsage
    {
      utilization : 100.0,
      resets_at   : None,
    } );
  }

  // Capture stderr — timestamped skip lines go to stderr via eprintln!.
  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

  // claude_paths=None: subprocess never spawns (no binary path) even if guard misses.
  // trace=true: skip line emitted only when guard fires.
  apply_touch(
    &mut aq,
    store.path(),
    None,
    true,
    SubprocessModel::Auto,
    SubprocessEffort::Auto,
    false,
  );

  let mut captured = String::new();
  stderr_buf.read_to_string( &mut captured ).unwrap();

  // Fix: guard fires → "YYYY-MM-DD · HH:MM:SS · touch  test@example.com  skipped (reason: 7d-exhausted)".
  assert!(
    captured.contains( "7d-exhausted" ),
    "apply_touch must emit '7d-exhausted' trace for idle 7d-exhausted account; got:\n{captured}",
  );
}

/// BUG-215 MRE: `apply_touch` must fire when 5h is active but the 7d timer is absent.
///
/// # Root Cause
///
/// `apply_touch()` computed `is_idle = five_hour.resets_at.is_none()`. When the 5h
/// timer is running (`resets_at=Some`), `is_idle=false` → `!is_idle=true` → guard
/// fires → "already active" skip, even though `seven_day.resets_at` is absent (7d
/// timer not started). The account's 7d quota window was never activated by touch.
///
/// # Why Not Caught
///
/// All prior trigger tests covered only the 5h-idle case (`resets_at=None` triggers
/// touch). The case where 5h is active but 7d is absent was never tested.
/// `tests/docs/feature/024_session_touch.md` FT-17 was absent before this session.
///
/// # Fix Applied
///
/// Replaced `is_idle` (single 5h timer) with `all_running` (3-timer):
/// `five_h_running && seven_d_running && seven_ds_running`.
/// `seven_d_running = seven_day.as_ref().map_or(true, |p| p.resets_at.is_some())`.
/// Guard fires only when all timers are running or quota is exhausted.
///
/// # Prevention
///
/// Any new quota dimension added to `apply_touch` skip logic must have a
/// corresponding FT in `tests/docs/feature/024_session_touch.md` before
/// implementation.
///
/// # Pitfall
///
/// `seven_day=None` (field absent) → `map_or(true)` → `seven_d_running=true`
/// (no timer to start — no weekly tracking on this plan). Only
/// `seven_day=Some({resets_at:None})` (field present, timer absent) triggers
/// touch. Do NOT use `seven_day=None` for this test — absent field ≠ absent timer.
#[ doc = "bug_reproducer(BUG-215)" ]
#[ test ]
fn test_mre_bug215_apply_touch_fires_when_7d_timer_absent()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();

  // 5h timer running (Some), utilization=50% → five_hour_left=50.0 (NOT h-exhausted).
  // seven_day field present: utilization=0.0 → seven_day_left=100.0 (NOT 7d-exhausted).
  // seven_day.resets_at=None → 7d timer absent (period exists but countdown not started).
  let mut aq = mk_aq_with_resets_at( Some( "2099-01-01T00:00:00Z" ) );
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day = Some( claude_quota::PeriodUsage
    {
      utilization : 0.0,
      resets_at   : None,
    } );
  }

  // Capture stderr — timestamped skip lines go to stderr via eprintln!.
  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

  // claude_paths=None: no subprocess spawns regardless of guard outcome.
  // trace=true: "skipped" line emitted ONLY when guard fires.
  apply_touch(
    &mut aq,
    store.path(),
    None,
    true,
    SubprocessModel::Auto,
    SubprocessEffort::Auto,
    false,
  );

  let mut captured = String::new();
  stderr_buf.read_to_string( &mut captured ).unwrap();

  // Fix: guard does NOT fire (7d timer absent → all_running=false) → no "skipped" line.
  // Before fix: guard fires ("already active") → captured contains "skipped" → FAIL.
  assert!(
    !captured.contains( "skipped" ),
    "apply_touch must NOT emit 'skipped' for 5h active but 7d timer absent; got:\n{captured}",
  );
}

/// BUG-288-FixB MRE: `apply_touch` must skip when `touch_idle=false` in quota cache.
///
/// # Root Cause
///
/// `apply_post_switch_touch` writes `touch_idle=false` to the quota cache at
/// `api.rs:330-332` after its subprocess activates the account. Before Fix B,
/// `apply_touch` never read this flag — the write was a dead write with zero read
/// sites. On the next `.usage` call, `apply_touch` saw `five_hour.resets_at=None`
/// (API propagation lag — the new session hadn't reached the quota endpoint yet)
/// and spawned a redundant second subprocess for an account already activated.
///
/// # Why Not Caught
///
/// Fix A (BUG-288) addressed the common case: `apply_post_switch_touch` now
/// re-fetches quota and writes `resets_at=Some(...)` to cache, so `apply_touch`
/// normally sees `all_running=true` and skips. The propagation-lag case (quota
/// endpoint returns stale `resets_at=None` even after re-fetch) was not tested.
/// `tests/docs/feature/024_session_touch.md` FT-19 was absent until this session.
///
/// # Fix Applied
///
/// Fix B (TSK-291): added `touch_idle` cache read to `apply_touch` at
/// `touch.rs:59-66`. After the error-account guard and before the `all_running`
/// timer check, `apply_touch` reads the quota cache; if `touch_idle=Some(false)`,
/// the account is skipped with a distinguishable trace line.
///
/// # Prevention
///
/// Any new coordination flag written by one touch path and consumed by another must
/// have both a write-site test and a read-site test. See AC-16 in
/// `tests/docs/feature/024_session_touch.md` FT-19.
///
/// # Pitfall
///
/// `touch_idle=None` (cache absent or field unset) does NOT trigger the skip guard —
/// the guard checks `== Some(false)` exclusively. An account with no cache entry
/// proceeds to the `all_running` check as normal.
///
/// **Test setup — `fetched_at` required by `read_quota_cache`**: calling
/// `write_cache_bool(..., "touch_idle", false)` alone writes
/// `{ "cache": { "touch_idle": false } }` without a `"fetched_at"` field.
/// `read_quota_cache` returns `None` when `"fetched_at"` is absent — the guard
/// is never entered and no trace line is emitted. Any test that requires
/// `read_quota_cache` to return `Some` must call
/// `write_cache_string(..., "fetched_at", &chrono_now_utc())` BEFORE
/// `write_cache_bool`. This caused Phase 5 reiteration during BUG-288 Fix B
/// implementation (2026-06-13).
#[ doc = "bug_reproducer(BUG-288-FixB)" ]
#[ test ]
fn test_mre_bug288_apply_touch_skips_touch_idle_false()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();

  // Simulate apply_post_switch_touch writing touch_idle=false after its subprocess
  // (api.rs:330-332). The account is "already activated" by that subprocess, but
  // the quota endpoint hasn't propagated the new session's resets_at yet.
  // fetched_at must be written first — read_quota_cache returns None if absent.
  claude_profile_core::account::write_cache_string(
    store.path(), "test@example.com", "fetched_at",
    &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool(
    store.path(), "test@example.com", "touch_idle", false,
  );

  // Account is idle (resets_at=None) — would qualify for touch by timer state alone.
  // The touch_idle=false guard must intercept BEFORE the all_running check.
  let mut aq = mk_aq_with_resets_at( None );

  // Capture stderr — timestamped skip line goes to stderr via eprintln!.
  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

  // claude_paths=None: subprocess never spawns regardless of guard outcome.
  // trace=true: "YYYY-MM-DD · HH:MM:SS · touch  <name>  skipped (reason: touch_idle=false)" emitted by guard.
  apply_touch(
    &mut aq,
    store.path(),
    None,
    true,
    SubprocessModel::Auto,
    SubprocessEffort::Auto,
    false,
  );

  let mut captured = String::new();
  stderr_buf.read_to_string( &mut captured ).unwrap();

  // Fix B: guard fires → "touch_idle=false" in skip trace.
  // Before Fix B: guard absent → dead write → no "touch_idle=false" in trace.
  assert!(
    captured.contains( "touch_idle=false" ),
    "apply_touch must emit 'touch_idle=false' skip trace for account with \
     touch_idle=false in cache; got:\n{captured}",
  );
}

// ── Fix B guard precision corner cases ───────────────────────────────────

/// CC-B2: `touch_idle=Some(true)` does NOT trigger the skip guard.
///
/// The guard checks `cache.touch_idle == Some(false)` exclusively.
/// Two-call design for non-vacuity:
/// - Call A (`touch_idle=false`): proves the guard EXISTS and fires for `Some(false)`.
///   If the guard were deleted, Call A would emit `"read credentials: Err"` instead of
///   `"touch_idle=false"` — failing Call A's assertion.
/// - Call B (`touch_idle=true`): proves `Some(true)` does not trigger the skip.
#[ test ]
fn test_apply_touch_touch_idle_true_not_skipped_by_guard()
{
  use std::io::Read;

  // Call A: prove the guard exists and fires for Some(false).
  // Without this, Call B's negative assertion is vacuous — guard deletion also produces
  // no "touch_idle" in the trace and reaches the subprocess path identically.
  {
    let store_a = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store_a.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store_a.path(), "test@example.com", "touch_idle", false,
    );
    let mut aq = mk_aq_with_resets_at( None );
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq, store_a.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    assert!(
      captured.contains( "touch_idle=false" ),
      "call A: guard must fire for touch_idle=Some(false); got:\n{captured}",
    );
  }

  // Call B: guard does NOT fire for Some(true) — separate store, fresh aq.
  {
    let store_b = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store_b.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store_b.path(), "test@example.com", "touch_idle", true,
    );
    let mut aq = mk_aq_with_resets_at( None );
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq, store_b.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    // Guard must NOT fire for Some(true): guard checks == Some(false) only.
    assert!(
      !captured.contains( "touch_idle" ),
      "call B: guard must NOT fire for touch_idle=Some(true); got:\n{captured}",
    );
    // Execution reached the subprocess path — proves all guards were passed.
    assert!(
      captured.contains( "read credentials:" ),
      "call B: execution must reach subprocess path when touch_idle=Some(true); got:\n{captured}",
    );
  }
}

/// CC-B3: `touch_idle` absent from cache (but `fetched_at` present) does NOT skip.
///
/// When `read_quota_cache` returns `Some(cache)` but `cache.touch_idle = None`
/// (field was never written), the guard `cache.touch_idle == Some(false)` evaluates
/// to `None == Some(false) → false`. The account proceeds to the `all_running` check.
/// Two-call design for non-vacuity:
/// - Call A (`touch_idle=false`): proves the guard EXISTS and fires for `Some(false)`.
/// - Call B (`touch_idle=None`): proves `None` does not trigger the skip.
#[ test ]
fn test_apply_touch_touch_idle_none_in_cache_not_skipped()
{
  use std::io::Read;

  // Call A: prove the guard exists and fires for Some(false).
  {
    let store_a = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store_a.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store_a.path(), "test@example.com", "touch_idle", false,
    );
    let mut aq = mk_aq_with_resets_at( None );
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq, store_a.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    assert!(
      captured.contains( "touch_idle=false" ),
      "call A: guard must fire for touch_idle=Some(false); got:\n{captured}",
    );
  }

  // Call B: touch_idle=None (field absent) must NOT trigger the guard.
  {
    let store_b = tempfile::TempDir::new().unwrap();
    // Only fetched_at — no touch_idle field → touch_idle=None in cache.
    claude_profile_core::account::write_cache_string(
      store_b.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    // Verify: read_quota_cache returns Some with touch_idle=None.
    let entry = claude_profile_core::account::read_quota_cache( store_b.path(), "test@example.com" );
    assert!( entry.is_some(), "precondition: read_quota_cache must return Some when fetched_at is present" );
    assert_eq!(
      entry.unwrap().touch_idle, None,
      "precondition: touch_idle must be None when field was never written",
    );
    let mut aq = mk_aq_with_resets_at( None );
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq, store_b.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    // Guard must NOT fire for touch_idle=None.
    assert!(
      !captured.contains( "touch_idle" ),
      "call B: guard must NOT fire for touch_idle=None; got:\n{captured}",
    );
    // Execution reached the subprocess path — proves all guards were passed.
    assert!(
      captured.contains( "read credentials:" ),
      "call B: execution must reach subprocess path when touch_idle=None; got:\n{captured}",
    );
  }
}

/// CC-B4 (pitfall): `fetched_at` absent → `read_quota_cache` returns `None` → guard bypassed.
///
/// `write_cache_bool(touch_idle, false)` alone writes
/// `{ "cache": { "touch_idle": false } }` without a `fetched_at` field.
/// `read_quota_cache` requires `fetched_at` and returns `None` when absent —
/// the `touch_idle` guard is never entered. The account proceeds to `all_running`.
///
/// Two-call design for non-vacuity:
/// - Call A (`touch_idle=false` + `fetched_at`): proves the guard EXISTS and fires.
/// - Call B (`touch_idle=false`, NO `fetched_at`): proves it is bypassed when cache returns `None`.
///
/// This is the pitfall documented in `test_mre_bug288_apply_touch_skips_touch_idle_false`
/// — test setup MUST call `write_cache_string(fetched_at, ...)` before `write_cache_bool`.
#[ test ]
fn test_apply_touch_touch_idle_false_fetched_at_absent_guard_bypassed()
{
  use std::io::Read;

  // Call A: prove the guard exists and fires when cache is valid (fetched_at present).
  {
    let store_a = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store_a.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store_a.path(), "test@example.com", "touch_idle", false,
    );
    let mut aq = mk_aq_with_resets_at( None );
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq, store_a.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    assert!(
      captured.contains( "touch_idle=false" ),
      "call A: guard must fire for touch_idle=Some(false) with valid cache; got:\n{captured}",
    );
  }

  // Call B: touch_idle=false WITHOUT fetched_at → read_quota_cache returns None → guard bypassed.
  {
    let store_b = tempfile::TempDir::new().unwrap();
    // Write touch_idle=false WITHOUT fetched_at — read_quota_cache returns None.
    claude_profile_core::account::write_cache_bool(
      store_b.path(), "test@example.com", "touch_idle", false,
    );
    // Verify: read_quota_cache returns None (fetched_at required).
    let entry = claude_profile_core::account::read_quota_cache( store_b.path(), "test@example.com" );
    assert!(
      entry.is_none(),
      "precondition: read_quota_cache must return None when fetched_at is absent",
    );
    let mut aq = mk_aq_with_resets_at( None );
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq, store_b.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    // Guard bypassed (read_quota_cache returned None) → no "touch_idle" skip trace.
    assert!(
      !captured.contains( "touch_idle" ),
      "call B: guard must NOT fire when fetched_at absent (read_quota_cache returned None); \
       got:\n{captured}",
    );
    // Execution reached the subprocess path — proves guard was genuinely bypassed.
    assert!(
      captured.contains( "read credentials:" ),
      "call B: execution must reach subprocess path when guard is bypassed (fetched_at absent); \
       got:\n{captured}",
    );
  }
}

/// CC-B5: `trace=false` + `touch_idle=Some(false)` → silent skip, zero stderr output.
///
/// Two-call design for non-vacuity:
/// 1. `trace=true` call proves the guard fires and emits `"touch_idle=false"` (positive anchor).
/// 2. `trace=false` call with identical state proves the same guard fires silently.
///
/// If the guard were deleted, call 1 would NOT emit `"touch_idle=false"` — it would emit
/// `"read credentials: Err(...)"` instead — failing the first assertion. So the silence
/// assertion in call 2 is only reached when the guard is confirmed present and working.
#[ test ]
fn test_apply_touch_touch_idle_false_silent_when_trace_disabled()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();

  claude_profile_core::account::write_cache_string(
    store.path(), "test@example.com", "fetched_at",
    &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool(
    store.path(), "test@example.com", "touch_idle", false,
  );

  let mut aq = mk_aq_with_resets_at( None );

  // Call 1: trace=true — proves the guard fires and emits the skip reason.
  // Without this, the silence assertion in call 2 would be vacuously true (guard deleted
  // → subprocess path → trace=false → empty stderr regardless).
  {
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq,
      store.path(),
      None,
      true,
      SubprocessModel::Auto,
      SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    assert!(
      captured.contains( "touch_idle=false" ),
      "call 1 (trace=true): apply_touch must emit 'touch_idle=false' skip; got:\n{captured}",
    );
  }

  // Call 2: trace=false — same cache state, same aq; guard fires but emits nothing.
  {
    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch(
      &mut aq,
      store.path(),
      None,
      false,
      SubprocessModel::Auto,
      SubprocessEffort::Auto, false
    );
    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();
    assert!(
      captured.is_empty(),
      "call 2 (trace=false): apply_touch must emit nothing to stderr on touch_idle=false skip; \
       got:\n{captured}",
    );
  }
}
