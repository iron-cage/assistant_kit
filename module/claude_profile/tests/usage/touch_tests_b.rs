// Integration tests for touch.rs — Part B.
// Continuation of `touch_tests.rs`.

use claude_profile::usage::test_bridge::apply_touch;
use claude_profile::usage::test_bridge::types::{ SubprocessModel, SubprocessEffort };
use claude_profile::usage::test_bridge::mk_aq_with_resets_at;

/// CC-B6: Error account with `touch_idle=false` in cache — error guard fires FIRST.
///
/// Guard ordering: (1) error guard → (2) `touch_idle` guard → (3) `all_running` guard.
/// An account with `result=Err` must be caught by the error guard before the
/// `touch_idle` guard is even consulted. The trace must say "error account", not "`touch_idle=false`".
#[ test ]
fn test_apply_touch_error_account_skips_before_touch_idle_guard()
{
  use std::io::Read;
  use claude_profile::usage::test_bridge::mk_aq_err;

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

  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
    store.join( claude_profile::account::active_marker_filename() ),
    "test@example.com",
  ).unwrap();
  let mut aq = mk_aq_with_resets_at( None );
  let paths  = claude_profile::ClaudePaths::with_home( &fake_home );
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
    store.join( claude_profile::account::active_marker_filename() ),
    "test@example.com",
  ).unwrap();
  let mut aq = mk_aq_with_resets_at( Some( "2099-01-01T00:00:00Z" ) );
  let paths  = claude_profile::ClaudePaths::with_home( &fake_home );
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
  use claude_profile::usage::test_bridge::mk_aq_with_son_idle;

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

    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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

    let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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

  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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

  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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

  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
  let fn_start = src.find( "pub fn apply_touch(" ).expect( "apply_touch not found" );

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
