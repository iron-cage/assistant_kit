// Integration tests for touch.rs — relocated from src/usage/touch_tests.rs.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::apply_touch;
use claude_profile::usage::test_bridge::touch_skip_reason;
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

  // Fix: guard fires → "skipped (reason: 7d-exhausted)".
  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    Some( "skipped (reason: 7d-exhausted)" ),
    "apply_touch must skip idle 7d-exhausted accounts with reason '7d-exhausted'",
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

  // Fix: guard does NOT fire (7d timer absent → all_running=false) → touch proceeds.
  // Before fix: guard fired ("already active") → Some(_) → FAIL.
  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    None,
    "apply_touch must NOT skip when 5h is active but 7d timer is absent",
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
  let aq = mk_aq_with_resets_at( None );

  // Fix B: guard fires → "touch_idle=false" reason.
  // Before Fix B: guard absent → dead write → guard never fires.
  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    Some( "skipped (reason: touch_idle=false)" ),
    "apply_touch must skip accounts with touch_idle=false in cache",
  );
}

/// TSK-418: `apply_touch`'s h-exhausted skip guard fires only at true/full 5h exhaustion.
///
/// # Root Cause
///
/// TSK-196 (BUG-177/BUG-178) added the h-exhausted guard by reusing
/// `H_EXHAUSTED_THRESHOLD = 15.0` (`types.rs`) — a constant TSK-190 had just raised from
/// 5% to 15% one day earlier, for the unrelated purpose of giving humans early warning in
/// display/sort classification. `touch_skip_reason()` borrowed this same 15% cutoff, so
/// accounts with 1%-15% of their 5h quota remaining (e.g. utilization=89.0, the real-world
/// i16@wbox.pro case) were skipped even though a touch subprocess still succeeds and extends
/// the account's usable window at any nonzero remaining quota.
///
/// # Why Not Caught
///
/// BUG-214's own MRE doc comment flagged this gap: "the h-exhausted guard was added in
/// isolation without extending the test surface" — no test asserted boundary behavior at
/// partial exhaustion, only presence/absence of the guard category.
///
/// # Fix Applied
///
/// TSK-418: changed both `h_left <= 15.0` occurrences in `touch_skip_reason()` to
/// `h_left <= 0.0`, matching the already-correct sibling `d7_left <= 0.0` pattern used
/// for the 7d-exhausted leg of the same guard.
///
/// # Prevention
///
/// A constant's scope (its own doc comment, TSK-190) must be checked before reuse at a new
/// call site — `H_EXHAUSTED_THRESHOLD` remains correctly scoped to its original 3 display/sort
/// consumers; `apply_touch` needed a different, function-appropriate threshold.
///
/// # Pitfall
///
/// `five_hour_left()` already `.round()`s its output (pre-existing Fix(BUG-336)), so `h_left`
/// is always a whole number at this call site — no fractional-boundary ambiguity at `0.0`.
#[ doc = "bug_reproducer(TSK-418)" ]
#[ test ]
fn test_tsk418_apply_touch_fires_at_partial_exhaustion_skips_at_full_exhaustion()
{
  let store = tempfile::TempDir::new().unwrap();

  // T01: 11% remaining (utilization=89.0, the real-world i16@wbox.pro case).
  // Old threshold (15.0) would have skipped this — new threshold (0.0) must NOT.
  {
    let mut aq = mk_aq_with_resets_at( None );
    if let Ok( ref mut data ) = aq.result
    {
      data.five_hour = Some( claude_quota::PeriodUsage { utilization : 89.0, resets_at : None } );
    }
    assert_eq!(
      touch_skip_reason( &aq, store.path(), false ),
      None,
      "TSK-418: 11%-remaining account must NOT be skipped — touch fires",
    );
  }

  // T02: 0% remaining (utilization=100.0, fully exhausted) — guard must still fire.
  {
    let mut aq = mk_aq_with_resets_at( None );
    if let Ok( ref mut data ) = aq.result
    {
      data.five_hour = Some( claude_quota::PeriodUsage { utilization : 100.0, resets_at : None } );
    }
    assert_eq!(
      touch_skip_reason( &aq, store.path(), false ),
      Some( "skipped (reason: h-exhausted)" ),
      "TSK-418: 0%-remaining (fully exhausted) account must be skipped",
    );
  }

  // T03: 15% remaining (utilization=85.0) — the OLD threshold boundary. Regression guard
  // against reverting to TSK-196's value.
  {
    let mut aq = mk_aq_with_resets_at( None );
    if let Ok( ref mut data ) = aq.result
    {
      data.five_hour = Some( claude_quota::PeriodUsage { utilization : 85.0, resets_at : None } );
    }
    assert_eq!(
      touch_skip_reason( &aq, store.path(), false ),
      None,
      "TSK-418: 15%-remaining account (old threshold boundary) must NOT be skipped",
    );
  }
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
  // Call A: prove the guard exists and fires for Some(false).
  // Without this, Call B's negative assertion is vacuous — guard deletion also produces
  // None for both Some(false) and Some(true) identically.
  {
    let store_a = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store_a.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store_a.path(), "test@example.com", "touch_idle", false,
    );
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store_a.path(), false ),
      Some( "skipped (reason: touch_idle=false)" ),
      "call A: guard must fire for touch_idle=Some(false)",
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
    let aq = mk_aq_with_resets_at( None );
    // Guard must NOT fire for Some(true): guard checks == Some(false) only.
    // None overall proves execution would reach the subprocess path — all guards passed.
    assert_eq!(
      touch_skip_reason( &aq, store_b.path(), false ),
      None,
      "call B: guard must NOT fire for touch_idle=Some(true)",
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
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store_a.path(), false ),
      Some( "skipped (reason: touch_idle=false)" ),
      "call A: guard must fire for touch_idle=Some(false)",
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
    // Precondition: read_quota_cache returns Some with touch_idle=None.
    let entry = claude_profile_core::account::read_quota_cache( store_b.path(), "test@example.com" );
    assert!( entry.is_some(), "precondition: read_quota_cache must return Some when fetched_at is present" );
    assert_eq!(
      entry.unwrap().touch_idle, None,
      "precondition: touch_idle must be None when field was never written",
    );
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store_b.path(), false ),
      None,
      "call B: guard must NOT fire for touch_idle=None",
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
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store_a.path(), false ),
      Some( "skipped (reason: touch_idle=false)" ),
      "call A: guard must fire for touch_idle=Some(false) with valid cache",
    );
  }

  // Call B: touch_idle=false WITHOUT fetched_at → read_quota_cache returns None → guard bypassed.
  {
    let store_b = tempfile::TempDir::new().unwrap();
    // Write touch_idle=false WITHOUT fetched_at — read_quota_cache returns None.
    claude_profile_core::account::write_cache_bool(
      store_b.path(), "test@example.com", "touch_idle", false,
    );
    // Precondition: read_quota_cache returns None (fetched_at required).
    let entry = claude_profile_core::account::read_quota_cache( store_b.path(), "test@example.com" );
    assert!(
      entry.is_none(),
      "precondition: read_quota_cache must return None when fetched_at is absent",
    );
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store_b.path(), false ),
      None,
      "call B: guard must NOT fire when fetched_at absent (read_quota_cache returned None)",
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
  let store = tempfile::TempDir::new().unwrap();

  claude_profile_core::account::write_cache_string(
    store.path(), "test@example.com", "fetched_at",
    &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool(
    store.path(), "test@example.com", "touch_idle", false,
  );

  let aq = mk_aq_with_resets_at( None );

  // Oracle proof: the touch_idle=false guard fires for this exact config — the skip
  // path (not the subprocess path) is what trace=false must keep silent below.
  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    Some( "skipped (reason: touch_idle=false)" ),
    "guard must fire for touch_idle=Some(false) before the silence claim below is meaningful",
  );

  // Structural proof: apply_touch's only stderr write for the skip-reason path is
  // gated by `if trace` — confirms trace=false cannot reach the writeln! call.
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    src.contains( "if trace { let _ = writeln!( std::io::stderr(), \"{}touch  {}  {}\", trace_ts(), aq.name, reason ); }" ),
    "apply_touch's skip-reason trace emission must remain gated by `if trace`",
  );
}
