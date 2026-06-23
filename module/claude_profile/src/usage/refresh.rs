//! Token-refresh loop for account credentials.
//!
//! `apply_refresh` drives the refresh loop across an accounts slice, delegating
//! to `crate::account::refresh_account_token` for the full lifecycle.
//! The decision predicate (`should_refresh`) lives in `refresh_predicate`.

use super::refresh_predicate::should_refresh;

use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort };
use super::subprocess::{ resolve_model, effort_pre_args };
use super::fetch::{ read_token, parse_u64_from_str };

// ── reason_label ─────────────────────────────────────────────────────────────

// Fix(BUG-306): inline reason block had no is_occupied_elsewhere branch —
//   owned+non-cached+occupied-elsewhere accounts fell through to "ok".
//   Root cause: three-branch inline block (not-owned / cached / else) predates
//   the occupancy predicate gate (BUG-303); no branch was added when the gate
//   was introduced.
//   Pitfall: every new predicate gate in should_refresh must have a corresponding
//   branch in reason_label — the predicate–reason 1:1 contract.
pub( crate ) fn reason_label( aq : &AccountQuota, now_secs : u64 ) -> &str
{
  if !aq.is_owned
  {
    "not owned"
  }
  else if aq.cached
  {
    if ( aq.expires_at_ms / 1000 ) <= now_secs { "cached-expired" }
    else { "cached" }
  }
  else if aq.is_occupied_elsewhere
  {
    "occupied elsewhere"
  }
  else
  {
    aq.result.as_ref().err().map_or( "ok", String::as_str )
  }
}

// ── Refresh loop ──────────────────────────────────────────────────────────────

/// Retry quota fetch for accounts that need token refresh (401/403 auth errors,
/// or 429 rate-limit with locally-expired credentials).
///
/// Uses the account lifecycle when `claude_paths` is available: `switch_account` copies
/// the named account's credentials to the live session, the isolated subprocess refreshes
/// the token via an API call side-effect, then `save` propagates the updated credentials
/// back to the persistent store and all companion files.  Falls back to direct persistent-
/// store reads/writes when `claude_paths` is `None`.  Mutates `accounts` in place.
///
/// Fix(BUG-271) — HTTP 429 removed from unconditional retry guard.
/// Root cause: HTTP 429 is a rate-limit response, not an authentication failure.
/// Pitfall: Task 142 added 429 unconditionally; task 150 removed it. The correct
/// behaviour (BUG-156) is to refresh only when 429 AND locally expired.
// Fix(BUG-211): snapshot+restore pattern removed — refresh_account_token now passes
//   update_marker=false to save(), so _active is never written during per-account
//   cycling. The post-loop restore clobbered concurrent .account.use switches that
//   ran during the ~35s subprocess window.
//   See bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md.
// Root cause: original design assumed background refresh callers own the active-account
//   identity, but .account.use can fire during the ~35s subprocess window.
// Pitfall: do NOT re-introduce snapshot+restore here — the fix is in save(), not here.
pub( crate ) fn apply_refresh(
  accounts         : &mut [ AccountQuota ],
  credential_store : &std::path::Path,
  claude_paths     : Option< &crate::ClaudePaths >,
  trace            : bool,
  imodel           : SubprocessModel,
  effort           : SubprocessEffort,
  solo             : bool,
)
{
  let now_secs = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  for aq in accounts
  {
    // Solo gate: non-current accounts are never refreshed when solo::1.
    // Fires before should_refresh — avoids evaluating the predicate for solo-skipped accounts.
    if solo && !aq.is_current
    {
      if trace { eprintln!( "[trace] refresh  {}  solo-skip", aq.name ); }
      continue;
    }

    let should_retry = should_refresh( aq, now_secs );
    // Fix(BUG-295): reason derived from aq.result.err() silently fell through to "ok"
    //   for non-owned accounts (G1 sets result=Ok(cached_data) for them).
    //   Root cause: ownership gate fires before result is ever an error for non-owned accts.
    //   Pitfall: check !aq.is_owned BEFORE consulting aq.result.err() at trace sites.
    // Fix(BUG-298): owned+cached accounts also have result=Ok (fetch.rs:229-240 cache fallback
    //   converts Err→Ok and sets aq.cached=true); .err() on Ok returns None → constant "ok".
    //   Real trigger: BUG-255 guard (cached && expired). Check aq.cached before aq.result.err().
    //   Pitfall: any trigger path that converts Err→Ok must add its own reason branch here.
    if trace
    {
      eprintln!( "[trace] refresh  {}  should_retry={} (reason: {})", aq.name, should_retry, reason_label( aq, now_secs ) );
    }
    if !should_retry { continue; }

    if trace { eprintln!( "[trace] refresh  {}  attempting token refresh", aq.name ); }
    let model      = resolve_model( aq, imodel );
    let pre_args   = effort_pre_args( &model, effort );
    let Some( new_creds ) = crate::account::refresh_account_token(
      &aq.name, credential_store, claude_paths, trace, "refresh", model, &pre_args,
    )
    else
    {
      if trace
      {
        eprintln!( "[trace] refresh  {}  refresh returned None — skipping retry", aq.name );
      }
      // Fix(BUG-297): set aq.result to Err so apply_touch skips this account.
      // Root cause: else-continue left aq.result=Ok(cached_data) when cache masking was active;
      //   apply_touch at touch.rs:56 guards on Ok and fires a redundant subprocess.
      // Pitfall: Invariant — every continue path in apply_refresh must leave aq.result=Err
      //   if the account cannot proceed; apply_touch uses aq.result as its sole recoverability signal.
      aq.result = Err( "refresh token expired".into() );
      continue;
    };

    // Fix(BUG-162): derive expiry from JWT exp claim — subprocess does not update expiresAt.
    // Root cause: the isolated subprocess writes refreshed accessToken/refreshToken but leaves
    //   expiresAt at the original expired timestamp; re-reading from file gives stale value.
    // Pitfall: expiresAt is a server-issued claim the subprocess cannot update; always derive
    //   post-refresh expiry from crate::output::jwt_exp_ms(), never by re-reading the credentials file.
    if let Some( exp_ms ) = crate::output::jwt_exp_ms( &new_creds )
    {
      aq.expires_at_ms = exp_ms;
    }
    // Fix(BUG-170): fallback to expiresAt field for opaque sk-ant-oat01-* tokens.
    // Root cause: jwt_exp_ms returns None for tokens with no '.' separator (not a JWT);
    //   the if-let above never fires, leaving aq.expires_at_ms at the stale pre-refresh value.
    // Pitfall: use else-if (not a second if-let) — only update from expiresAt when JWT decode
    //   fails; a separate if-let would run even on JWT success and silently overwrite with the
    //   expiresAt field value, which may differ from the JWT exp claim by clock skew.
    else if let Some( exp_ms ) = parse_u64_from_str( &new_creds, "expiresAt" )
    {
      aq.expires_at_ms = exp_ms;
    }

    // Re-read the refreshed token and retry only this account's quota.
    if trace { eprintln!( "[trace] refresh  {}  token refreshed, retrying quota fetch", aq.name ); }
    let Ok( token ) = read_token( credential_store, &aq.name ) else { continue; };
    match claude_quota::fetch_oauth_usage( &token )
    {
      Ok( retried ) =>
      {
        if trace { eprintln!( "[trace] refresh  {}  retry OK", aq.name ); }
        // Fix(BUG-256): retry OK arm cleared only aq.result — cached flag and cache_age_secs
        //   not cleared, causing render.rs to keep ~ prefix and (Xh ago) on every quota cell.
        //   write_quota_cache also absent, so {name}.json still held the stale cached quota.
        // Root cause: merge f83d78d resolved conflict using remote branch, dropping these mutations.
        // Pitfall: extract h5/d7/sn BEFORE moving retried into aq.result — use-after-move otherwise.
        let h5 = retried.five_hour.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        let d7 = retried.seven_day.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        let sn = retried.seven_day_sonnet.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        claude_profile_core::account::write_quota_cache( credential_store, &aq.name, h5, d7, sn );
        aq.result         = Ok( retried );
        aq.cached         = false;
        aq.cache_age_secs = None;
        // Fix(BUG-171): account must be re-fetched after refresh; initial fetch used
        //   the expired token; quota fetch path and account fetch path diverged.
        // Root cause: fetch_oauth_account was added to fetch_all_quota later than apply_refresh;
        //   the refresh retry path never had a corresponding account re-fetch.
        // Pitfall: use if-let, not unconditional .ok() assignment — preserve existing value
        //   on network failure; aq.account = fetch_oauth_account(...).ok() silently destroys
        //   previously-populated account data on transient errors.
        if let Ok( acct ) = claude_quota::fetch_oauth_account( &token )
        {
          aq.account = Some( acct );
        }
      }
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] refresh  {}  retry Err({})", aq.name, e ); }
        // Fix(BUG-156): propagate the retry error to show the current post-refresh status.
        // Root cause: on retry failure the original error (e.g. "401 expired") was kept,
        //   hiding the actual post-refresh state (e.g. "429 rate-limited after refresh").
        // Pitfall: ignoring the retry error masks the true current state after refresh.
        aq.result = Err( e.to_string() );
      }
    }
  }

}


// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
#[ path = "refresh_tests.rs" ]
mod tests;
