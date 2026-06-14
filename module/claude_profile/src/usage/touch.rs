// BUG-289 task/claude_profile/bug/289_son_running_false_haiku_touch_infinite_loop.md — son_running=false guard and re-fetch loop; Auto→Haiku cannot open Sonnet 7d window

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
  // G4: Non-owned accounts are never touched — subprocess spawning on foreign credentials forbidden.
  if !aq.is_owned
  {
    if trace { eprintln!( "[trace] touch  {}  skipped (reason: not owned)", aq.name ); }
    return;
  }

  // Guard: errored accounts are never touched; trigger requires valid quota data.
  // Fix(BUG-202): bare return produced no trace for error-tier accounts.
  // Root cause: error guard preceded all trace emission points (lines 1506-1510).
  // Pitfall: multiple early-return guards each need their own trace emission.
  let Ok( ref data ) = aq.result else
  {
    if trace { eprintln!( "[trace] touch  {}  skipped (reason: error account)", aq.name ); }
    return;
  };

  // Fix(BUG-288-FixB): read touch_idle flag written by apply_post_switch_touch.
  //   When touch_idle=false, a subprocess already activated this account — skip as
  //   defense-in-depth for API propagation lag (resets_at may not yet reflect the
  //   new session at the quota endpoint even after Fix A's re-fetch).
  // Root cause: api.rs:330-332 writes touch_idle=false with zero read sites — dead write.
  // Pitfall: server-side quota propagation can lag; local cache flag is the only
  //   coordination signal not subject to that lag.
  if let Some( cache ) = claude_profile_core::account::read_quota_cache( credential_store, &aq.name )
  {
    if cache.touch_idle == Some( false )
    {
      if trace { eprintln!( "[trace] touch  {}  skipped (reason: touch_idle=false)", aq.name ) }
      return;
    }
  }

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
#[ path = "touch_tests.rs" ]
mod tests;
