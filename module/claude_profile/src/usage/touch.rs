//! Session-touch logic for idle quota windows.
//!
//! `apply_touch` activates an idle 5h (or 7d) session window by spawning an isolated
//! subprocess, then re-fetches the quota so the table reflects the concrete timer value.

use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort };
use super::subprocess::{ resolve_model, effort_pre_args };
use super::fetch::{ read_token, parse_u64_from_str };
use super::format::{ five_hour_left, seven_day_left };

// ── Touch ─────────────────────────────────────────────────────────────────────

/// Activate an idle 5h session window for `aq` by spawning an isolated subprocess.
///
/// The trigger requires both conditions:
/// - `aq.result.is_ok()` — account must have valid quota data (not an auth error).
/// - `five_hour.resets_at.is_none()` — 5h window is idle (no active session).
///
/// After a successful touch, quota is re-fetched so the table shows the concrete
/// `5h Reset` value. If the subprocess or re-fetch fails the account row is unchanged
/// (touch failure is non-aborting — other accounts and the render continue normally).
///
/// The original active account is restored unconditionally inside this call before
/// using the new credentials. This prevents a stale active marker if the process is
/// interrupted between touches.
// Fix(BUG-211): snapshot+restore removed — same rationale as apply_refresh.
//   refresh_account_token passes update_marker=false; _active never written during
//   per-account touch cycling; the post-loop restore raced with concurrent .account.use.
//   See bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md.
// Root cause: apply_touch was added after apply_refresh and inherited the same flawed
//   snapshot+restore pattern (see BUG-211).
// Pitfall: do NOT re-introduce snapshot+restore here — the fix is in save(), not here.
pub( crate ) fn apply_touch(
  aq               : &mut AccountQuota,
  credential_store : &std::path::Path,
  claude_paths     : Option< &crate::ClaudePaths >,
  trace            : bool,
  imodel           : SubprocessModel,
  effort           : SubprocessEffort,
)
{
  // Guard: errored accounts are never touched; trigger requires valid quota data.
  // Fix(BUG-202): bare return produced no trace for error-tier accounts.
  // Root cause: error guard preceded all trace emission points (lines 1506-1510).
  // Pitfall: multiple early-return guards each need their own trace emission.
  let Ok( ref data ) = aq.result else
  {
    if trace { eprintln!( "[trace] touch  {}  skipped (reason: error account)", aq.name ); }
    return;
  };

  // Guard: skip accounts with all timers running, h-exhausted, or 7d-exhausted.
  // AC-02: trigger fires when ANY quota timer is absent and quota is valid (not exhausted).
  // Fix(BUG-214): d7_left guard skips 7d-weekly-exhausted accounts (seven_day_left <= 0%).
  // Root cause(BUG-214): guard lacked seven_day_left check; 7d-exhausted accounts fired
  //   subprocess that received HTTP 429 (~2.3s penalty, no session opened).
  // Fix(BUG-215): replace is_idle (single 5h timer) with all_running (3-timer check).
  // Root cause(BUG-215): is_idle only checked five_hour.resets_at; accounts with 5h active
  //   but 7d/7d-Son timer absent were skipped as "already active" — touch never started
  //   the missing quota window.
  // Pitfall: map_or(true, ...) for 7d/7d-Son — field absent means no weekly tracking on
  //   the plan; treat as "running" to avoid spurious touch for dimensions that don't exist.
  let five_h_running = data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ).is_some();
  let d7_running     = data.seven_day.as_ref().map_or( true, |p| p.resets_at.is_some() );
  let son_running    = data.seven_day_sonnet.as_ref().map_or( true, |p| p.resets_at.is_some() );
  let all_running    = five_h_running && d7_running && son_running;
  let h_left  = five_hour_left( aq );
  let d7_left = seven_day_left( aq );
  if all_running || h_left <= 15.0 || d7_left <= 0.0
  {
    if trace
    {
      let reason = if all_running    { "already active" }
        else if h_left  <= 15.0     { "h-exhausted"    }
        else                         { "7d-exhausted"   };
      eprintln!( "[trace] touch  {}  skipped (reason: {})", aq.name, reason );
    }
    return;
  }

  let model    = resolve_model( aq, imodel );
  let pre_args = effort_pre_args( &model, effort );
  let new_creds = crate::account::refresh_account_token(
    &aq.name, credential_store, claude_paths, trace, "touch", model, &pre_args,
  );

  // Update expiry if credentials were returned (optional — touch may return None).
  if let Some( ref creds ) = new_creds
  {
    if let Some( exp_ms ) = crate::output::jwt_exp_ms( creds )
    {
      aq.expires_at_ms = exp_ms;
    }
    else if let Some( exp_ms ) = parse_u64_from_str( creds, "expiresAt" )
    {
      aq.expires_at_ms = exp_ms;
    }
  }

  // Re-read token AFTER subprocess — the pre-subprocess token is stale.
  // AC-03: unconditional re-fetch regardless of whether subprocess returned credentials.
  let Ok( token ) = read_token( credential_store, &aq.name ) else { return; };
  if let Ok( new_data ) = claude_quota::fetch_oauth_usage( &token )
  {
    aq.result = Ok( new_data );
    if let Ok( acct ) = claude_quota::fetch_oauth_account( &token )
    {
      aq.account = Some( acct );
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::apply_touch;
  use crate::usage::types::{ SubprocessModel, SubprocessEffort };
  use crate::usage::test_support::mk_aq_with_resets_at;
  use tempfile::TempDir;

  /// FT-15 / BUG-211 MRE — `apply_touch` does NOT write live credentials file (no `switch_account`).
  ///
  /// Fix(BUG-211): the snapshot+restore pattern was removed from `apply_touch`. With
  /// `trace=true`, no `[trace] touch  {name}  restore switch_account: OK/Err` line is
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

    // trace=true: Fix(BUG-211) — no restore; no [trace] restore switch_account line emitted.
    apply_touch( &mut aq, store.path(), Some( &paths ), true, SubprocessModel::Auto, SubprocessEffort::Auto );

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

    // Capture stderr — [trace] skip lines go to stderr via eprintln!.
    let mut stderr_buf = gag::BufferRedirect::stderr().expect( "stderr capture failed" );

    // claude_paths=None: subprocess never spawns (no binary path) even if guard misses.
    // trace=true: [trace] skip line emitted only when guard fires.
    apply_touch(
      &mut aq,
      store.path(),
      None,
      true,
      SubprocessModel::Auto,
      SubprocessEffort::Auto,
    );

    let mut captured = String::new();
    stderr_buf.read_to_string( &mut captured ).unwrap();

    // Fix: guard fires → "[trace] touch  test@example.com  skipped (reason: 7d-exhausted)".
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

    // Capture stderr — [trace] skip lines go to stderr via eprintln!.
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
    apply_touch( &mut aq, &store, Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto );
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
    apply_touch( &mut aq, &store, Some( &paths ), false, SubprocessModel::Auto, SubprocessEffort::Auto );
    // Trigger skipped → early return → live credentials file NOT written.
    assert!(
      !fake_home.join( ".claude" ).join( ".credentials.json" ).exists(),
      "apply_touch must not enter refresh path when resets_at is Some (already active)",
    );
  }
}
