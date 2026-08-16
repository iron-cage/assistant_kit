// Integration tests for touch.rs — Part B.
// Continuation of `touch_tests.rs`.

use claude_profile::usage::test_bridge::apply_touch;
use claude_profile::usage::test_bridge::touch_skip_reason;
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
  use claude_profile::usage::test_bridge::mk_aq_err;

  let store = tempfile::TempDir::new().unwrap();

  // Cache has touch_idle=false keyed by the error account's name ("bad@example.com").
  // If the error guard were absent, touch_skip_reason would consult this entry and
  // return the touch_idle reason instead — making this a real guard-ordering test.
  claude_profile_core::account::write_cache_string(
    store.path(), "bad@example.com", "fetched_at",
    &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool(
    store.path(), "bad@example.com", "touch_idle", false,
  );

  // Error account: result=Err → error guard fires before the touch_idle guard.
  let aq = mk_aq_err();

  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    Some( "skipped (reason: error account)" ),
    "error guard must fire before touch_idle guard is consulted",
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
/// When `son_idle=true`, returns `Specific("claude-sonnet-5")` instead of Haiku
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
/// to prevent state leakage. Converted from gag-based stderr capture to direct
/// `touch_skip_reason()` oracle calls — no credential file or subprocess is needed: the
/// oracle is the pure decision function `apply_touch` calls first, so asserting it returns
/// `None` on both calls is equivalent to proving the trigger fires on every invocation.
///
/// Spec: [`tests/docs/feature/24_session_touch.md` FT-20]
#[ doc = "bug_reproducer(BUG-289)" ]
#[ test ]
fn test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call()
{
  use claude_profile::usage::test_bridge::mk_aq_with_son_idle;

  // Call A: prove the trigger fires (touch_skip_reason returns None) for son_running=false.
  // If any guard fired instead, touch_skip_reason would return Some(_).
  {
    let store_a = tempfile::TempDir::new().unwrap();

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

    assert_eq!(
      touch_skip_reason( &aq_a, store_a.path(), false ),
      None,
      "call A: touch must fire (no guard skips) for son_running=false",
    );
  }

  // Call B: prove the trigger fires AGAIN with identical fresh state — BUG-289 loop proof.
  // Separate store and fresh aq prevent state leakage from Call A.
  // Pre-fix: Haiku subprocess cannot open the 7d-Sonnet window → resets_at stays None →
  // son_running=false on every call → touch_skip_reason returns None every time (infinite loop).
  {
    let store_b = tempfile::TempDir::new().unwrap();

    let mut aq_b = mk_aq_with_son_idle();
    if let Ok( ref mut data ) = aq_b.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 0.0,
        resets_at   : Some( "2026-06-14T10:00:00Z".to_string() ),
      } );
    }

    assert_eq!(
      touch_skip_reason( &aq_b, store_b.path(), false ),
      None,
      "call B: touch must fire AGAIN for identical son_running=false state (BUG-289 loop)",
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
  let store = tempfile::TempDir::new().unwrap();

  // Build idle account (resets_at=None triggers touch normally, but G4 overrides).
  let mut aq = mk_aq_with_resets_at( None );
  // G4: override is_owned to false — account owned by a different machine.
  aq.is_owned = false;

  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    Some( "skipped (reason: not owned)" ),
    "FT-07: G4 gate must skip with reason 'not owned'",
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
  let store = tempfile::TempDir::new().unwrap();
  // mk_aq_with_resets_at defaults: is_current=false, is_owned=true — exact preconditions.
  let aq = mk_aq_with_resets_at( None );

  // solo=true: solo gate fires, returns before any other guard is consulted.
  assert_eq!(
    touch_skip_reason( &aq, store.path(), true ),
    Some( "solo-skip" ),
    "EC-8: solo gate must skip non-current account with 'solo-skip'",
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
  let store = tempfile::TempDir::new().unwrap();

  // Build idle account (resets_at=None triggers touch by timer state alone).
  let mut aq = mk_aq_with_resets_at( None );
  // Owned by this machine (passes G4) but occupied by another machine (fires occupancy guard).
  aq.is_owned = true;
  aq.is_occupied_elsewhere = true;

  assert_eq!(
    touch_skip_reason( &aq, store.path(), false ),
    Some( "skipped (reason: occupied elsewhere)" ),
    "FT-22: occupancy guard must skip with reason 'occupied elsewhere'",
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

// ── BUG-488: age-gated touch_idle guard + mark_touched writer ─────────────

/// BUG-488 MRE: the `touch_idle=false` guard is age-gated on `last_touch_at`.
///
/// # Root Cause
///
/// Three coupled defects. (1) The `.usage touch::1` loop path (`apply_touch`) never wrote
/// the BUG-288-FixB coordination flags after a successful touch — only the switch path did.
/// (2) The switch path wrote them to `paths.base()` while the sole reader
/// (`touch_skip_reason`) reads `credential_store/{name}.json` — a write/read directory
/// mismatch (BUG-207/BUG-318 family) that made the flag mechanism fully inert. (3) Nothing
/// ever writes `touch_idle=true`, so the flag has no expiry — a naive directory fix alone
/// would have permanently disabled touching for any account once one flag landed.
///
/// # Why Not Caught
///
/// Every flag-guard test hand-built cache state with `write_cache_bool` in the same
/// `TempDir` the guard reads, so the production write/read directory split was invisible;
/// and no test aged `last_touch_at`, so the missing-expiry hazard had no coverage.
///
/// # Fix Applied
///
/// Fix(BUG-488): `mark_touched` is the single writer (fresh `last_touch_at` +
/// `touch_idle=false`, into `credential_store`); both touch paths call it; the guard in
/// `touch_skip_reason` fires only when `touch_idle == Some(false)` AND `last_touch_at`
/// parses and is younger than `TOUCH_GRACE_SECS` (18 000 s = one 5h window).
///
/// # Prevention
///
/// A skip-flag with a writer and no expiry is a latent permanent-skip: any new
/// coordination flag must pair its state with a timestamp and age-gate the reader.
///
/// # Pitfall
///
/// Scenario C (flag present, `last_touch_at` absent) is the pre-BUG-488 legacy-cache
/// shape — it must NOT fire the guard, otherwise caches written by the old code would
/// suppress touching forever after upgrade.
#[ doc = "bug_reproducer(BUG-488)" ]
#[ test ]
fn test_mre_bug488_touch_idle_guard_age_gated()
{
  use claude_profile::usage::test_bridge::mark_touched;

  // Scenario A: fresh mark_touched → guard fires (recently touched, endpoint still lagging).
  {
    let store = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    mark_touched( store.path(), "test@example.com" );
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store.path(), false ),
      Some( "skipped (reason: touch_idle=false)" ),
      "A: guard must fire when mark_touched ran within TOUCH_GRACE_SECS",
    );
  }

  // Scenario B: stale last_touch_at (2020) → grace expired → guard must NOT fire.
  {
    let store = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store.path(), "test@example.com", "touch_idle", false,
    );
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "last_touch_at", "2020-01-01T00:00:00Z",
    );
    let aq = mk_aq_with_resets_at( None );
    assert_eq!(
      touch_skip_reason( &aq, store.path(), false ),
      None,
      "B: guard must NOT fire once last_touch_at is older than TOUCH_GRACE_SECS",
    );
  }

  // Scenario C: touch_idle=false with NO last_touch_at (legacy pre-BUG-488 cache) →
  // guard must NOT fire — the forever-skip regression guard.
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
    assert_eq!(
      touch_skip_reason( &aq, store.path(), false ),
      None,
      "C: bare touch_idle=false without last_touch_at must not skip (no expiry otherwise)",
    );
  }
}

/// BUG-488: `mark_touched` writes both flags to the store the guard reads, and
/// `write_quota_cache` carry-forward preserves them.
///
/// The write/read directory mismatch (Root Cause face 2 in the MRE above) is only fixed
/// if the writer and the reader agree on the directory: `mark_touched( store, name )`
/// must produce a cache `read_quota_cache( store, name )` sees with `touch_idle=Some(false)`
/// and a parseable `last_touch_at` — and a subsequent quota write must not drop them.
#[ test ]
fn test_bug488_mark_touched_roundtrip_survives_write_quota_cache()
{
  use claude_profile::usage::test_bridge::mark_touched;

  let store = tempfile::TempDir::new().unwrap();

  claude_profile_core::account::write_cache_string(
    store.path(), "test@example.com", "fetched_at",
    &claude_profile_core::account::chrono_now_utc(),
  );
  mark_touched( store.path(), "test@example.com" );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), "test@example.com" )
    .expect( "cache must be readable after fetched_at + mark_touched" );
  assert_eq!( entry.touch_idle, Some( false ), "mark_touched must write touch_idle=false" );
  let stamp = entry.last_touch_at.expect( "mark_touched must write last_touch_at" );
  assert!(
    claude_profile_core::account::parse_iso_utc_secs( &stamp ).is_some(),
    "last_touch_at must be parseable ISO-UTC; got {stamp:?}",
  );

  // Quota write after the touch (AC-03 re-fetch persists) must carry the flags forward.
  claude_profile_core::account::write_quota_cache(
    store.path(), "test@example.com", None, None, None,
  );
  let entry2 = claude_profile_core::account::read_quota_cache( store.path(), "test@example.com" )
    .expect( "cache must remain readable after write_quota_cache" );
  assert_eq!(
    entry2.touch_idle, Some( false ),
    "write_quota_cache must preserve touch_idle written by mark_touched",
  );
  assert!(
    entry2.last_touch_at.is_some(),
    "write_quota_cache must preserve last_touch_at written by mark_touched",
  );
}

/// BUG-488 structural: `apply_touch`'s success block calls `mark_touched` and sets
/// `aq.touched_recently` — gated on `new_creds`, before the AC-03 re-fetch.
///
/// `refresh_account_token` returns `None` on any failure, so anchoring both statements
/// inside the `if let Some( ref creds ) = new_creds` block proves a failed touch writes
/// no flags (which would wrongly suppress the retry for `TOUCH_GRACE_SECS`).
#[ test ]
fn test_bug488_apply_touch_success_block_marks_touched()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  let fn_start = src.find( "pub fn apply_touch(" ).expect( "apply_touch not found" );
  let body     = &src[ fn_start.. ];

  let guard_pos = body
    .find( "if let Some( ref creds ) = new_creds" )
    .expect( "BUG-488: new_creds success guard not found in apply_touch" );
  let mark_pos = body
    .find( "mark_touched( credential_store, &aq.name );" )
    .expect( "BUG-488: mark_touched call not found in apply_touch" );
  let touched_pos = body
    .find( "aq.touched_recently = true;" )
    .expect( "BUG-488: touched_recently assignment not found in apply_touch" );
  let refetch_pos = body
    .find( "if let Ok( new_data ) = claude_quota::fetch_oauth_usage(" )
    .expect( "AC-03 re-fetch block not found in apply_touch" );

  assert!(
    guard_pos < mark_pos && mark_pos < refetch_pos,
    "BUG-488: mark_touched must sit inside the new_creds success block (after its guard, \
     before the AC-03 re-fetch) — a failed touch must write no flags",
  );
  assert!(
    guard_pos < touched_pos && touched_pos < refetch_pos,
    "BUG-488: aq.touched_recently must be set inside the new_creds success block only",
  );
  assert_eq!(
    body.matches( "mark_touched(" ).count(), 1,
    "BUG-488: apply_touch must call mark_touched exactly once (success block only)",
  );
}

/// BUG-488 (cross-invocation): `derive_touched_recently` re-derives the display signal
/// from the persisted cache flags, so the `(touched)` marker survives past the touching
/// invocation for as long as the flags are fresh.
///
/// # Root Cause
///
/// `touched_recently` was set only in-memory by `apply_touch` — it died with the touching
/// invocation. The very next `.usage` run (skip guard correctly preventing a re-touch)
/// rendered the just-touched account as plain idle (`5h Reset —`) again until the quota
/// endpoint caught up — re-creating the original misleading-table symptom for every run
/// after the first.
///
/// # Why Not Caught
///
/// The initial BUG-488 tests covered the touching invocation (in-memory set) and the skip
/// guard, but no test exercised a second invocation's display state — the derive pass did
/// not exist.
///
/// # Fix Applied
///
/// Fix(BUG-488): `derive_touched_recently( &mut accounts, credential_store )` runs
/// unconditionally in the `.usage` pipeline after the touch loop; it sets
/// `touched_recently` for every account whose cache carries `touch_idle=false` plus a
/// `last_touch_at` within `TOUCH_GRACE_SECS`, sharing the `touched_within_grace`
/// predicate with the skip guard so display and skip semantics can never drift.
///
/// # Prevention
///
/// Scenarios below pin all four derive outcomes: fresh flags set the field, stale flags
/// don't, absent flags don't, and an already-set field survives with no cache at all.
/// The structural test that follows pins the pipeline call site.
///
/// # Pitfall
///
/// The derive pass must NOT require this invocation to have touched anything — its whole
/// point is `touch::0` / skip-guard invocations. It reads the same store the guard reads;
/// a future writer/reader directory split would silently kill it (the roundtrip test
/// above guards that seam).
#[ doc = "bug_reproducer(BUG-488)" ]
#[ test ]
fn test_bug488_derive_touched_recently_from_cache_flags()
{
  use claude_profile::usage::test_bridge::{ derive_touched_recently, mark_touched };

  // Scenario A: fresh flags (as mark_touched writes them) → field derived true.
  {
    let store = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    mark_touched( store.path(), "test@example.com" );
    let mut accounts = vec![ mk_aq_with_resets_at( None ) ];
    derive_touched_recently( &mut accounts, store.path() );
    assert!(
      accounts[ 0 ].touched_recently,
      "A: fresh cache flags must derive touched_recently=true on a later invocation",
    );
  }

  // Scenario B: stale flags (last_touch_at 2020) → grace expired → field stays false.
  {
    let store = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    claude_profile_core::account::write_cache_bool(
      store.path(), "test@example.com", "touch_idle", false,
    );
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "last_touch_at", "2020-01-01T00:00:00Z",
    );
    let mut accounts = vec![ mk_aq_with_resets_at( None ) ];
    derive_touched_recently( &mut accounts, store.path() );
    assert!(
      !accounts[ 0 ].touched_recently,
      "B: stale last_touch_at must not derive the display signal",
    );
  }

  // Scenario C: no flags at all (fetched_at only) → field stays false.
  {
    let store = tempfile::TempDir::new().unwrap();
    claude_profile_core::account::write_cache_string(
      store.path(), "test@example.com", "fetched_at",
      &claude_profile_core::account::chrono_now_utc(),
    );
    let mut accounts = vec![ mk_aq_with_resets_at( None ) ];
    derive_touched_recently( &mut accounts, store.path() );
    assert!(
      !accounts[ 0 ].touched_recently,
      "C: no touch on record must not derive the display signal",
    );
  }

  // Scenario D: field already set in-memory (touching invocation) survives with an
  // empty store — the derive pass skips flagged rows rather than re-deriving them.
  {
    let store = tempfile::TempDir::new().unwrap();
    let mut aq = mk_aq_with_resets_at( None );
    aq.touched_recently = true;
    let mut accounts = vec![ aq ];
    derive_touched_recently( &mut accounts, store.path() );
    assert!(
      accounts[ 0 ].touched_recently,
      "D: in-memory signal from apply_touch must survive the derive pass unchanged",
    );
  }
}

/// BUG-488 structural: the derive pass runs in the `.usage` pipeline — after the touch
/// loop (so the touching invocation's freshly-written flags are already on disk) and
/// before render dispatch, unconditionally (not gated on `touch::1`).
#[ test ]
fn test_bug488_derive_pass_wired_after_touch_loop()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );

  let touch_loop_pos = src
    .find( "if params.touch == 1" )
    .expect( "bulk touch block not found in api.rs" );
  let derive_pos = src
    .find( "derive_touched_recently( &mut accounts, &credential_store );" )
    .expect( "BUG-488: derive_touched_recently pipeline call not found in api.rs" );
  let render_dispatch_pos = src
    .find( "UsageOutputFormat::Text" )
    .expect( "render dispatch not found in api.rs" );

  assert!(
    touch_loop_pos < derive_pos && derive_pos < render_dispatch_pos,
    "BUG-488: derive_touched_recently must run after the touch loop and before render \
     dispatch (touch_loop={touch_loop_pos}, derive={derive_pos}, render={render_dispatch_pos})",
  );
}
