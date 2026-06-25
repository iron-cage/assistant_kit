// Path-referenced test module for touch.rs — compiled as `mod tests` via `#[path]`.
// Lives in src/usage/ (not tests/) to access pub(crate) apply_touch
// without widening its visibility. See src/usage/readme.md § Inline Test Exception.

use super::apply_touch;
use crate::usage::types::{ SubprocessModel, SubprocessEffort };
use crate::usage::test_support::mk_aq_with_resets_at;
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
    store.path().join( crate::account::active_marker_filename() ),
    "alice@example.com",
  ).unwrap();

  std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
  let paths = crate::ClaudePaths::with_home( fake_home.path() );

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
    store.path().join( crate::account::active_marker_filename() )
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
  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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

/// CC-B6: Error account with `touch_idle=false` in cache — error guard fires FIRST.
///
/// Guard ordering: (1) error guard → (2) `touch_idle` guard → (3) `all_running` guard.
/// An account with `result=Err` must be caught by the error guard before the
/// `touch_idle` guard is even consulted. The trace must say "error account", not "`touch_idle=false`".
#[ test ]
fn test_apply_touch_error_account_skips_before_touch_idle_guard()
{
  use std::io::Read;
  use crate::usage::test_support::mk_aq_err;

  let store = tempfile::TempDir::new().unwrap();

  // Cache has touch_idle=false keyed by the error account's name ("bad@example.com").
  // If the error guard were absent, the touch_idle guard would consult this entry and
  // emit "touch_idle=false" — making the !captured.contains("touch_idle") assertion
  // a real guard-ordering test, not a vacuous pass.
  claude_profile_core::account::write_cache_string(
    store.path(), "bad@example.com", "fetched_at",
    &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool(
    store.path(), "bad@example.com", "touch_idle", false,
  );

  // Error account: result=Err → error guard fires at top of apply_touch.
  let mut aq = mk_aq_err();

  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

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

  // Error guard fires first: trace says "error account", not "touch_idle=false".
  assert!(
    captured.contains( "error account" ),
    "apply_touch must emit 'error account' skip for Err result; got:\n{captured}",
  );
  assert!(
    !captured.contains( "touch_idle" ),
    "apply_touch must NOT reach touch_idle guard for error accounts; got:\n{captured}",
  );
}

/// CC-A1: `write_quota_cache` preserves `touch_idle=false` written before the call.
///
/// Fix A (`apply_post_switch_touch`) calls `write_cache_bool(touch_idle, false)` and
/// THEN calls `write_quota_cache`. The two calls must compose correctly:
/// `write_quota_cache` reads the existing cache, preserves `touch_idle`, and writes
/// the updated quota data. After the call, `read_quota_cache` must still return
/// `touch_idle=Some(false)` so Fix B's guard in `apply_touch` can fire.
///
/// This is the critical Fix A + Fix B integration invariant.
#[ test ]
fn test_write_quota_cache_preserves_touch_idle_false()
{
  let store = tempfile::TempDir::new().unwrap();

  // Step 1: write touch_idle=false (as apply_post_switch_touch does at api.rs:339-341).
  claude_profile_core::account::write_cache_bool(
    store.path(), "test@example.com", "touch_idle", false,
  );

  // Step 2: call write_quota_cache (as Fix A does at api.rs:362).
  // This must read the existing cache, preserve touch_idle=false, and write updated quota.
  claude_profile_core::account::write_quota_cache(
    store.path(), "test@example.com",
    None, // five_hour
    None, // seven_day
    None, // seven_day_sonnet
  );

  // Step 3: read back — touch_idle must survive write_quota_cache.
  let entry = claude_profile_core::account::read_quota_cache( store.path(), "test@example.com" )
    .expect( "read_quota_cache must return Some after write_quota_cache (fetched_at is present)" );

  assert_eq!(
    entry.touch_idle,
    Some( false ),
    "write_quota_cache must preserve touch_idle=false written before the call; \
     Fix A + Fix B integration broken if this fails",
  );
}

// ── apply_touch trigger behavioral tests ─────────────────────────────────

/// BUG-211 AC-02 / FT-02 behavioral: `apply_touch` fires for idle accounts but does NOT call `switch_account`.
///
/// Fix(BUG-211): snapshot+restore removed from `apply_touch`. When `five_hour.resets_at`
/// is `None` (idle), `apply_touch` calls `refresh_account_token` but does NOT follow up
/// with a restore `switch_account` call — so the live credentials file is never written.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-02]
#[ test ]
fn it_apply_touch_trigger_fires_resets_at_none()
{
  let dir       = tempfile::TempDir::new().unwrap();
  let store     = dir.path().join( "store" );
  let fake_home = dir.path().join( "home" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( fake_home.join( ".claude" ) ).unwrap();
  std::fs::write(
    store.join( "test@example.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  std::fs::write(
    store.join( crate::account::active_marker_filename() ),
    "test@example.com",
  ).unwrap();
  let mut aq = mk_aq_with_resets_at( None );
  let paths  = crate::ClaudePaths::with_home( &fake_home );
  apply_touch( &mut aq, &store, Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  // Fix(BUG-211): no switch_account in apply_touch restore → live credentials file must NOT exist.
  assert!(
    !paths.credentials_file().exists(),
    "BUG-211: apply_touch must not call switch_account; live credentials file must not exist",
  );
}

/// AC-02 behavioral: `apply_touch` skips when `resets_at` is `Some` (already active 5h window).
///
/// When `five_hour.resets_at` is present, `apply_touch` returns early without calling
/// `refresh_account_token`. The live credentials file is never written.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-02]
#[ test ]
fn it_apply_touch_trigger_skips_resets_at_some()
{
  let dir       = tempfile::TempDir::new().unwrap();
  let store     = dir.path().join( "store" );
  let fake_home = dir.path().join( "home" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( fake_home.join( ".claude" ) ).unwrap();
  std::fs::write(
    store.join( "test@example.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  std::fs::write(
    store.join( crate::account::active_marker_filename() ),
    "test@example.com",
  ).unwrap();
  let mut aq = mk_aq_with_resets_at( Some( "2099-01-01T00:00:00Z" ) );
  let paths  = crate::ClaudePaths::with_home( &fake_home );
  apply_touch( &mut aq, &store, Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto, false );
  // Trigger skipped → early return → live credentials file NOT written.
  assert!(
    !fake_home.join( ".claude" ).join( ".credentials.json" ).exists(),
    "apply_touch must not enter refresh path when resets_at is Some (already active)",
  );
}

/// FT-20 BUG-289 MRE: `apply_touch` fires on every call when `son_running=false`
/// (5h+7d running, Sonnet 7d absent) — Haiku subprocess cannot open the 7d-Sonnet window.
///
/// # Root Cause
///
/// `resolve_model(Auto, _aq)` ignored `_aq`; the `Auto` arm unconditionally returned Haiku.
/// When `five_h_running=true AND d7_running=true AND son_idle=true`, Haiku subprocesses
/// cannot activate the 7d-Sonnet window (`seven_day_sonnet.resets_at` stays `None`).
/// On each `.usage` call, `apply_touch` sees `son_running=false` → `all_running=false` →
/// trigger fires → Haiku subprocess → no-op → trigger fires again. Infinite loop.
///
/// # Why Not Caught
///
/// All prior touch trigger tests covered the 5h-idle case (`resets_at=None`) or the
/// all-timers-present skip case. The `son_idle-only` scenario — 5h and 7d running, Sonnet
/// timer absent — was never tested. FT-20 was absent from
/// `tests/docs/feature/24_session_touch.md` until TSK-292.
///
/// # Fix Applied
///
/// TSK-292 (BUG-289): `resolve_model` now reads `aq.result` in the `Auto` arm.
/// When `son_idle=true`, returns `Specific("claude-sonnet-4-6")` instead of Haiku
/// (`son_idle` gate; Fix: BUG-289, BUG-290). Sonnet-family API calls activate the
/// 7d-Sonnet window, clearing `son_idle` and breaking the loop.
///
/// # Prevention
///
/// Model-capability interactions must be tested with two-call non-vacuous design: Call A
/// proves the trigger fires for the given state; Call B proves the state persists (pre-fix
/// loop proof). The companion test `it_imodel_auto_selects_sonnet_when_son_idle`
/// in `subprocess.rs` verifies `resolve_model` returns Sonnet when `son_idle=true`
/// (BUG-289 positive fix test).
///
/// # Pitfall
///
/// Call A and Call B must use separate `TempDir` stores and fresh `AccountQuota` objects
/// to prevent state leakage. `claude_paths=None` keeps the test hermetic — `run_isolated`
/// is invoked (emitting `"run_isolated: invoking"`) but fails to find the binary, returning
/// `None` credentials. The credential file MUST be present in the store so `read_token`
/// succeeds and execution reaches the `"run_isolated: invoking"` trace line.
///
/// Spec: [`tests/docs/feature/24_session_touch.md` FT-20]
#[ doc = "bug_reproducer(BUG-289)" ]
#[ test ]
fn test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call()
{
  use std::io::Read;
  use crate::usage::test_support::mk_aq_with_son_idle;

  // Call A: prove the trigger fires for son_running=false (non-vacuity anchor).
  // If the guard were absent, touch would be skipped as "already active" (all_running=true)
  // and Call A would emit "skipped" — not "run_isolated: invoking".
  {
    let store_a = tempfile::TempDir::new().unwrap();
    // Credential file required: read_token must succeed so "run_isolated: invoking" is emitted.
    std::fs::write(
      store_a.path().join( "test@example.com.credentials.json" ),
      r#"{"accessToken":"tok-a","expiresAt":9999999999999}"#,
    ).unwrap();

    // Account state: five_h_running=true, d7_running=true, son_running=false.
    // seven_day=Some({resets_at:Some(...)}) — explicit d7_running (not map_or(true) path).
    let mut aq_a = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq_a.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 0.0,
        resets_at   : Some( "2026-06-14T10:00:00Z".to_string() ),
      } );
    }

    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch( &mut aq_a, store_a.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let mut captured_a = String::new();
    stderr_buf.read_to_string( &mut captured_a ).unwrap();

    assert!(
      captured_a.contains( "run_isolated: invoking" ),
      "call A: touch must fire (run_isolated invoked) for son_running=false; got:\n{captured_a}",
    );
  }

  // Call B: prove the trigger fires AGAIN with identical fresh state — BUG-289 loop proof.
  // Separate store and fresh aq prevent state leakage from Call A.
  // Pre-fix: Haiku subprocess cannot open the 7d-Sonnet window → resets_at stays None →
  // son_running=false on every call → trigger fires every time (infinite loop).
  {
    let store_b = tempfile::TempDir::new().unwrap();
    std::fs::write(
      store_b.path().join( "test@example.com.credentials.json" ),
      r#"{"accessToken":"tok-b","expiresAt":9999999999999}"#,
    ).unwrap();

    let mut aq_b = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq_b.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 0.0,
        resets_at   : Some( "2026-06-14T10:00:00Z".to_string() ),
      } );
    }

    let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );
    apply_touch( &mut aq_b, store_b.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );
    let mut captured_b = String::new();
    stderr_buf.read_to_string( &mut captured_b ).unwrap();

    assert!(
      captured_b.contains( "run_isolated: invoking" ),
      "call B: touch must fire AGAIN for identical son_running=false state (BUG-289 loop); got:\n{captured_b}",
    );
  }
}

// ── G4: non-owned accounts skipped by apply_touch ─────────────────────────

/// FT-07 (AC-07): `apply_touch()` skips non-owned accounts; emits timestamped trace with `not owned`.
///
/// G4 gate fires when `aq.is_owned == false`:
/// - No subprocess is spawned.
/// - With `trace=true`: stderr contains `" · touch  {name}  skipped (reason: not owned)"`.
///
/// Pitfall: `mk_aq_with_resets_at` sets `is_owned=true`; must be overridden to `false`.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-07]
#[ test ]
fn ft07_touch_skips_non_owned_with_trace()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();

  // Build idle account (resets_at=None triggers touch normally, but G4 overrides).
  let mut aq = mk_aq_with_resets_at( None );
  // G4: override is_owned to false — account owned by a different machine.
  aq.is_owned = false;

  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

  // trace=true; claude_paths=None so subprocess cannot fire even if G4 is bypassed.
  apply_touch( &mut aq, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  let mut captured = String::new();
  stderr_buf.read_to_string( &mut captured ).unwrap();

  // G4 trace line must be present.
  assert!(
    captured.contains( "not owned" ),
    "FT-07: G4 gate must emit 'not owned' trace line; got:\n{captured}",
  );
  assert!(
    captured.contains( " · touch  " ),
    "FT-07: trace line must contain ' · touch  '; got:\n{captured}",
  );
}

// ── BUG-302 MRE: occupied-elsewhere accounts skipped by apply_touch ────────────

/// EC-8 (061): `apply_touch` solo gate — non-current owned account is skipped with
/// timestamped `touch  {name}  solo-skip` line when `solo=true`.
///
/// With `solo=true`, the solo gate fires before G4 (non-owned check) for any account
/// where `aq.is_current=false`. The account here is `is_owned=true` — without the solo
/// gate it would proceed to the `all_running` check. With `solo=true` it is skipped
/// immediately and the trace confirms the reason.
///
/// Non-vacuity anchor: `solo=false` (in all other touch tests) reaches the timer check
/// and emits `read credentials:` or a timer trace — proving the solo gate does not fire
/// for `solo=false`.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-8]
#[ test ]
fn ec8_solo_gate_skips_non_current_with_trace()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();
  // mk_aq_with_resets_at defaults: is_current=false, is_owned=true — exact preconditions.
  let mut aq = mk_aq_with_resets_at( None );

  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

  // trace=true, solo=true: solo gate fires, emits skip trace, returns before timer check.
  apply_touch( &mut aq, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, true );

  let mut captured = String::new();
  stderr_buf.read_to_string( &mut captured ).unwrap();

  assert!(
    captured.contains( "solo-skip" ),
    "EC-8: solo gate must emit 'solo-skip' trace for non-current account; got:\n{captured}",
  );
  assert!(
    captured.contains( "test@example.com" ),
    "EC-8: trace must name the skipped account; got:\n{captured}",
  );
  // Solo gate fires before timer/credentials check — no subprocess path reached.
  assert!(
    !captured.contains( "read credentials:" ),
    "EC-8: solo gate must fire BEFORE credential read; got:\n{captured}",
  );
}

/// FT-22 (AC-17): `apply_touch()` skips owned accounts with `is_occupied_elsewhere=true`;
/// emits timestamped trace with `occupied elsewhere`.
///
/// # Root Cause
/// G4 at `touch.rs:46` checked `!aq.is_owned` only. When `is_owned=true` and
/// `is_occupied_elsewhere=true`, G4 passed — the idle-timer check ran and fired the touch
/// subprocess. Two machines sent concurrent prompts through the same credential set:
/// quota burned at 2× rate with no warning.
///
/// # Why Not Caught
/// `ft07_touch_skips_non_owned_with_trace` only tested the `is_owned=false` case.
/// The `is_owned=true, is_occupied_elsewhere=true` combination was never tested — G4 was
/// written before `is_occupied_elsewhere` was introduced (Feature 036 / TSK-293).
///
/// # Fix Applied
/// Fix(BUG-302): added occupancy guard immediately after G4 block:
/// `if aq.is_occupied_elsewhere { ... return; }` with skip-reason trace.
/// The guard fires before any timer checks — owned+occupied accounts are treated
/// identically to non-owned accounts for subprocess invocation.
///
/// # Prevention
/// Any new subprocess-spawning gate must check BOTH `!is_owned` AND `!is_occupied_elsewhere`.
/// Ownership grants authorization to use credentials; occupancy signals concurrent use.
///
/// # Pitfall
/// `mk_aq_with_resets_at` defaults `is_owned=true, is_occupied_elsewhere=false`. Must explicitly
/// set `is_occupied_elsewhere=true` to test the occupancy path — NOT `is_owned=false` (that
/// tests G4, not the occupancy guard).
#[ doc = "bug_reproducer(BUG-302)" ]
#[ test ]
fn ft_touch_skips_occupied_elsewhere_with_trace()
{
  use std::io::Read;

  let store = tempfile::TempDir::new().unwrap();

  // Build idle account (resets_at=None triggers touch by timer state alone).
  let mut aq = mk_aq_with_resets_at( None );
  // Owned by this machine (passes G4) but occupied by another machine (fires occupancy guard).
  aq.is_owned = true;
  aq.is_occupied_elsewhere = true;

  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

  // trace=true; claude_paths=None so subprocess cannot fire even if guard is bypassed.
  apply_touch( &mut aq, store.path(), None, true, SubprocessModel::Auto, SubprocessEffort::Auto, false );

  let mut captured = String::new();
  stderr_buf.read_to_string( &mut captured ).unwrap();

  // Occupancy guard trace line must be present.
  assert!(
    captured.contains( "occupied elsewhere" ),
    "FT-22: occupancy guard must emit 'occupied elsewhere' trace line; got:\n{captured}",
  );
  assert!(
    captured.contains( " · touch  " ),
    "FT-22: trace line must contain ' · touch  '; got:\n{captured}",
  );
  // No subprocess must fire.
  assert!(
    !captured.contains( "run_isolated: invoking" ),
    "FT-22: no subprocess must be spawned for occupied-elsewhere account; got:\n{captured}",
  );
}

// ── BUG-309 MRE: re-fetch block must clear cached metadata and write cache file ─

/// MRE for BUG-309: `apply_touch` re-fetch block clears `cached` flag, `cache_age_secs`,
/// and writes fresh quota data to `{name}.json` via `write_quota_cache()`.
///
/// # Root Cause
///
/// The re-fetch block in `apply_touch` only set `aq.result = Ok(new_data)` — it did not
/// clear `aq.cached` or `aq.cache_age_secs`, so `render.rs` kept the `~` prefix on every
/// quota cell and the `(Xh ago)` age label for cache-fallback accounts. `write_quota_cache`
/// was also absent, so `{name}.json` retained stale pre-touch quota (with `resets_at=null`)
/// across restarts. Same class of omission as BUG-256 (refresh.rs retry-OK arm) and
/// BUG-288 (`apply_post_switch_touch` re-fetch block), but in `apply_touch`.
///
/// # Why Not Caught
///
/// No test guarded the content of the `apply_touch` re-fetch block. `apply_touch` was
/// implemented after Fix(BUG-256) corrected `apply_refresh`, but the three mutations were
/// never propagated. Fix(BUG-288) addressed `apply_post_switch_touch` in `api.rs` but did
/// not audit `apply_touch` in `touch.rs` for the same missing mutations.
///
/// # Fix Applied
///
/// Fix(BUG-309): in the re-fetch block of `apply_touch`, extract h5/d7/sn references
/// BEFORE moving `new_data` into `aq.result`, then call `write_quota_cache`, and set
/// `aq.cached = false` and `aq.cache_age_secs = None`.
///
/// # Prevention
///
/// This test greps the source of the re-fetch block for the three AC-18 mutations.
/// Any merge conflict or refactor that drops them will cause this test to fail.
///
/// # Pitfall
///
/// The `write_quota_cache` call must appear BEFORE `aq.result = Ok( new_data )` —
/// h5/d7/sn borrow from `new_data`; moving it first would be use-after-move.
/// The order check below enforces this structural constraint statically.
///
/// Spec: [`tests/docs/feature/24_session_touch.md` FT-23]
#[ doc = "bug_reproducer(BUG-309)" ]
#[ test ]
fn mre_bug309_apply_touch_refetch_writes_cache_and_clears_cached_flag()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  let fn_start = src.find( "pub( crate ) fn apply_touch(" ).expect( "apply_touch not found" );

  // Locate the re-fetch block within the function body.
  let refetch_rel = src[ fn_start.. ]
    .find( "if let Ok( new_data ) = claude_quota::fetch_oauth_usage(" )
    .expect( "BUG-309: re-fetch block `if let Ok( new_data ) = claude_quota::fetch_oauth_usage(` not found in apply_touch" );
  let refetch_start = fn_start + refetch_rel;

  // The re-fetch block is the last statement in apply_touch — slice from here to end.
  let refetch_block = &src[ refetch_start.. ];

  // AC-18 check 1: aq.cached must be cleared to false.
  assert!(
    refetch_block.contains( "aq.cached         = false" ),
    "BUG-309: apply_touch re-fetch block must set `aq.cached = false` to clear ~ prefix from render",
  );

  // AC-18 check 2: aq.cache_age_secs must be cleared to None.
  assert!(
    refetch_block.contains( "aq.cache_age_secs = None" ),
    "BUG-309: apply_touch re-fetch block must set `aq.cache_age_secs = None` to remove (Xh ago) label",
  );

  // AC-18 check 3: write_quota_cache must be called with fresh data.
  assert!(
    refetch_block.contains( "write_quota_cache(" ),
    "BUG-309: apply_touch re-fetch block must call write_quota_cache to persist fresh data to {{name}}.json",
  );

  // Order check: write_quota_cache must appear before the move of new_data into aq.result.
  let cache_write_pos = refetch_block.find( "write_quota_cache(" ).unwrap();
  let result_move_pos = refetch_block
    .find( "aq.result         = Ok( new_data )" )
    .expect( "BUG-309: `aq.result = Ok( new_data )` not found in apply_touch re-fetch block" );
  assert!(
    cache_write_pos < result_move_pos,
    "BUG-309: write_quota_cache must appear before `aq.result = Ok( new_data )` — \
     h5/d7/sn borrow from new_data and would be use-after-move otherwise",
  );
}

// ── D3: Bulk touch does NOT write live credentials ────────────────────────

/// Reach test: the bulk touch loop in `api.rs` (lines 669-676) iterates `apply_touch`
/// over all accounts but does NOT perform any `switch_account` or live-credential copy.
/// Live credentials are ONLY written in the rotation dispatch block (step 4d/4e').
///
/// The bulk loop has no `switch_account` preceding it — each `apply_touch` writes to
/// STORE only via `refresh_account_token → save(update_marker=false)`. If `fs::copy` or
/// `switch_account` were added inside the bulk loop, the live session would silently
/// change during a non-rotation `.usage` call — that's a regression.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (reach D3)]
#[ test ]
fn reach_bulk_touch_does_not_write_live_credentials()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );

  // Locate the bulk touch loop: `if params.touch == 1`
  let touch_block_start = src
    .find( "if params.touch == 1" )
    .expect( "bulk touch block not found in api.rs" );

  // The block ends at the next top-level comment block (Session-model override).
  let touch_block_end = src[ touch_block_start.. ]
    .find( "// ── Session-model override" )
    .map_or( src.len(), |rel| touch_block_start + rel );
  let bulk_block = &src[ touch_block_start..touch_block_end ];

  // The bulk loop must NOT contain switch_account or fs::copy — those belong only in rotation.
  assert!(
    !bulk_block.contains( "switch_account(" ),
    "D3: bulk touch loop must NOT call switch_account — live credentials must not change \
    during a non-rotation .usage call.\nbulk block:\n{bulk_block}",
  );
  assert!(
    !bulk_block.contains( "fs::copy" ),
    "D3: bulk touch loop must NOT call fs::copy — live credentials must not change \
    during a non-rotation .usage call.\nbulk block:\n{bulk_block}",
  );
}
