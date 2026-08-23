// BUG-289 — son_running=false guard and re-fetch loop; Auto→Haiku cannot open Sonnet 7d window
// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]

//! Session-touch logic for idle quota windows.
//!
//! `apply_touch` activates an idle 5h (or 7d) session window by spawning an isolated
//! subprocess, then re-fetches the quota so the table reflects the concrete timer value.

use std::io::Write as _;
use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort };
use super::subprocess::{ resolve_model, effort_pre_args };
use super::fetch::{ read_token, parse_u64_from_str };
use super::format::{ five_hour_left, seven_day_left };
use claude_profile_core::account::trace_ts;

// ── Touch ─────────────────────────────────────────────────────────────────────

/// Trust window for the `touch_idle=false` cache flag, in seconds — one full 5h session
/// window. Within it a recorded touch implies the window is running regardless of what the
/// (possibly propagation-lagged) quota endpoint reports; past it the flag is stale and
/// ignored, since the touched window itself has expired and a fresh touch is warranted.
pub const TOUCH_GRACE_SECS : u64 = 18_000;

/// Longest gap between a touch and a later quota fetch for which that fetch reporting no
/// 5h window is *uninformative* rather than *refuting* — the quota endpoint's own
/// propagation lag after a session start.
///
/// Beyond this gap, a fetch that still reports no window is positive evidence that the
/// touch opened none: the endpoint had ample opportunity to report one and did not.
/// BUG-488 measured the lag at 48s; the shortest observed refutation was 22 minutes
/// (BUG-552's i15). 300s sits an order of magnitude below that floor with margin above
/// the observed lag.
pub const TOUCH_PROPAGATION_SECS : u64 = 300;

/// Current wall clock in Unix seconds, or 0 if the clock predates the epoch.
fn now_unix_secs() -> u64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, | d | d.as_secs() )
}

/// Persist the touch coordination flags for `name` after a successful touch subprocess:
/// `last_touch_at` = now, `touch_idle` = false — written into the credential store's
/// `{name}.json` cache, the same file `touch_skip_reason` reads them back from.
/// Single canonical write site for both touch paths (`apply_touch` loop and
/// `apply_post_switch_touch`).
// Fix(BUG-488): flags previously went to `paths.base()` (~/.claude/{name}.json) on the
//   switch path and nowhere at all on the .usage touch loop path, so the sole reader
//   (touch_skip_reason, keyed on the credential store) could never see them.
// Root cause: write_cache_*'s first param is the credential store; apply_post_switch_touch
//   passed paths.base() — the same directory confusion its own BUG-207/BUG-318 comments
//   already document for sibling call sites in that function.
// Pitfall: the flags are only trusted while `last_touch_at` is younger than
//   TOUCH_GRACE_SECS — nothing ever writes `touch_idle=true` back, so an age-ungated
//   read would skip the account forever once the flag lands.
pub fn mark_touched( credential_store : &std::path::Path, name : &str )
{
  claude_profile_core::account::write_cache_string(
    credential_store, name, "last_touch_at", &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool( credential_store, name, "touch_idle", false );
}

/// `Some(instant)` — the cache's `last_touch_at` as Unix seconds when it parses and is
/// younger than `TOUCH_GRACE_SECS`; `None` otherwise. `now_secs` is injected so callers
/// (and tests) share one clock reading rather than each sampling their own.
///
/// Freshness only. Callers deciding whether to *act* on a touch claim want
/// `corroborated_touch_at`, which adds the refutation check (BUG-552); this bare form is
/// the grace half of that predicate and is not a display or skip gate on its own.
///
/// Fix(BUG-552): a `-> bool` sibling (`touched_within_grace`) used to wrap this and was
///   what both consumers called. Removed with the fix rather than left in place: it
///   discards the instant, which is the exact shape BUG-551 was filed over, and a
///   convenience bool named "touched recently" is precisely what a future caller would
///   reach for by mistake.
pub fn touch_instant_within_grace(
  cache    : &claude_profile_core::account::QuotaCacheEntry,
  now_secs : u64,
) -> Option< u64 >
{
  cache.last_touch_at.as_deref()
    .and_then( claude_profile_core::account::parse_iso_utc_secs )
    .filter( | t | now_secs.saturating_sub( *t ) < TOUCH_GRACE_SECS )
}

/// `Some(instant)` — the touch instant when the cache carries a touch claim that is both
/// within `TOUCH_GRACE_SECS` and **not refuted** by a later fetch; `None` otherwise.
///
/// A touch claim is refuted when a quota fetch made more than `TOUCH_PROPAGATION_SECS`
/// after the touch still reports no 5h window: the endpoint had the opportunity to report
/// one and did not, so no session was opened. Single corroboration predicate shared by the
/// `touch_idle` skip guard in `touch_skip_reason` and the display derivation in
/// `derive_touched_recently` — both must judge a claim by the same evidence or skip and
/// display semantics drift apart.
// Fix(BUG-552): a touch that refreshed the OAuth token but opened no session still stamped
//   the coordination flags, so the row claimed "(touched)" for the full 5h grace AND the
//   shared skip guard refused to re-touch it for that same window — the wrong state
//   guaranteed its own persistence, with nothing able to falsify it.
// Root cause: apply_touch gates mark_touched on refresh_account_token's Some, which proves
//   only that the token refreshed at subprocess startup — performed before the API call
//   that opens the window, and captured even on timeout (refresh.rs:60-64).
// Pitfall: absence of a window is only evidence once a fetch strictly later than the touch
//   has had the chance to observe one — compare against `fetched_at`, never wall clock, or
//   a merely-slow sweep reads as a refutation.
pub fn corroborated_touch_at(
  cache    : &claude_profile_core::account::QuotaCacheEntry,
  now_secs : u64,
) -> Option< u64 >
{
  let touched_at = touch_instant_within_grace( cache, now_secs )?;
  let has_window = cache.five_hour.as_ref().is_some_and( | ( _, resets_at ) | resets_at.is_some() );
  let fetched_at = cache.fetched_at.as_str();
  let refuted    = !has_window
    && claude_profile_core::account::parse_iso_utc_secs( fetched_at )
      .is_some_and( | f | f.saturating_sub( touched_at ) > TOUCH_PROPAGATION_SECS );
  if refuted { None } else { Some( touched_at ) }
}

/// Set `touched_at_secs` on every account whose cache carries a corroborated touch record
/// — `last_touch_at` within `TOUCH_GRACE_SECS` and unrefuted by the cache's own quota
/// (`corroborated_touch_at`) — the cross-invocation half of the touched-display signal.
/// `apply_touch` covers the touching invocation itself in-memory; this pass covers every
/// later invocation inside the grace window, where the skip guard (correctly) prevents a
/// re-touch but the quota endpoint may still report the window idle.
///
/// The instant, not a bare "yes it was touched" flag, is what the render layer needs: it
/// is the anchor `format::projected_window_end_secs` floors and offsets to produce the
/// `~in Xh Ym` countdown the `5h Reset` column shows in place of a server `resets_at`.
// Fix(BUG-488): without this pass the `(touched)` marker died with the touching
//   invocation — the very next `.usage` run rendered a just-touched account as plain
//   idle ("5h Reset —") again until the endpoint caught up, re-creating the original
//   misleading-table symptom for every run after the first.
// Root cause: `touched_recently` was in-memory only; the persistent form of the same
//   fact (the mark_touched cache flags) was consulted by the skip guard but never by
//   the display path.
// Pitfall: rows already flagged in-memory are skipped (cheap), and rows are flagged
//   regardless of current `resets_at` — the render layer alone decides visibility, so
//   an endpoint that caught up simply shows the real reset time instead.
pub fn derive_touched_recently( accounts : &mut [ AccountQuota ], credential_store : &std::path::Path )
{
  let now_secs = now_unix_secs();
  for aq in accounts.iter_mut()
  {
    if aq.touched_at_secs.is_some()
    {
      continue;
    }
    if let Some( cache ) = claude_profile_core::account::read_quota_cache( credential_store, &aq.name )
    {
      if cache.touch_idle == Some( false )
      {
        aq.touched_at_secs = corroborated_touch_at( &cache, now_secs );
      }
    }
  }
}

/// Compute the skip reason for `apply_touch`, or `None` if the account should proceed.
///
/// Pure decision function — mirrors, in order, the 6 guards inlined in `apply_touch`:
/// solo-skip → G4 not-owned → occupied-elsewhere → error-account → `touch_idle=false`
/// (cache) → already-active/h-exhausted/7d-exhausted. `credential_store` is required
/// because the `touch_idle` guard reads `claude_profile_core::account::read_quota_cache`,
/// keyed by account name — that guard's outcome is not derivable from `aq` alone.
pub fn touch_skip_reason(
  aq               : &AccountQuota,
  credential_store : &std::path::Path,
  solo             : bool,
) -> Option< &'static str >
{
  // Solo gate: non-current accounts are never touched when solo::1.
  // Fires before G4 — avoids credential reads for solo-skipped accounts.
  if solo && !aq.is_current
  {
    return Some( "solo-skip" );
  }

  // G4: Non-owned accounts are never touched — subprocess spawning on foreign credentials forbidden.
  if !aq.is_owned
  {
    return Some( "skipped (reason: not owned)" );
  }

  // Fix(BUG-302): occupancy guard — owned accounts in use on another machine must not be touched.
  // Root cause: G4 was written when is_occupied_elsewhere was not yet available (Feature 036).
  // Pitfall: is_owned and is_occupied_elsewhere are independent — an owned account can also be
  //   occupied; both guards must fire independently, not as a combined condition.
  if aq.is_occupied_elsewhere
  {
    return Some( "skipped (reason: occupied elsewhere)" );
  }

  // Feature 071: redirect-backend rows carry a placeholder Err, but "error account" is the
  // wrong story — there is no Anthropic session to touch at all. Checked before the generic
  // error guard so the trace names the real reason; wording matches `.account.use`'s own
  // pre-switch skip ("skipped (reason: redirect backend)").
  if aq.is_redirect_backend()
  {
    return Some( "skipped (reason: redirect backend)" );
  }

  // Guard: errored accounts are never touched; trigger requires valid quota data.
  // Fix(BUG-202): bare return produced no trace for error-tier accounts.
  // Root cause: error guard preceded all trace emission points (lines 1506-1510).
  // Pitfall: multiple early-return guards each need their own trace emission.
  let Ok( ref data ) = aq.result else
  {
    return Some( "skipped (reason: error account)" );
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
    // Fix(BUG-488): the flag is trusted only while last_touch_at is younger than
    //   TOUCH_GRACE_SECS (one 5h window) — before this gate existed, one landed flag
    //   would have skipped the account forever, because no code path ever writes
    //   touch_idle=true back.
    // Root cause: BUG-288-FixB added the read with no expiry semantics; the defect
    //   stayed invisible because the write side targeted the wrong directory, so the
    //   guard never actually fired.
    // Pitfall: absent/unparseable last_touch_at means "no trustworthy touch on record" —
    //   fall through to the API-state guards, never skip on the bare flag alone.
    // Fix(BUG-552): corroborated_touch_at replaces the bare grace check — a touch refuted
    //   by a later fetch reporting no window must not suppress its own retry, or the
    //   account stays un-touchable for the full 5h grace with no way to recover.
    // Root cause: the flag was trusted for TOUCH_GRACE_SECS on the strength of the touch
    //   subprocess alone, and TOUCH_GRACE_SECS is exactly the window it vouches for — so
    //   the claim expired at the same moment the window it asserted would have.
    // Pitfall: falling through here does NOT mean the account is re-touched — the
    //   exhaustion guards below still apply, and a genuinely 7d-exhausted account (the
    //   population where refuted touches concentrate) is still skipped by d7_left <= 0.0.
    if cache.touch_idle == Some( false ) && corroborated_touch_at( &cache, now_unix_secs() ).is_some()
    {
      return Some( "skipped (reason: touch_idle=false)" );
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
  // Fix(TSK-418): h-exhausted threshold changed from `<= 15.0` (borrowed from the
  //   display/sort H_EXHAUSTED_THRESHOLD, TSK-190) to `<= 0.0` (true/full exhaustion) —
  //   matching the already-correct `d7_left <= 0.0` sibling pattern.
  // Root cause: TSK-196 (BUG-177/BUG-178) reused H_EXHAUSTED_THRESHOLD by false analogy;
  //   a touch subprocess still succeeds and extends the window at any nonzero remaining quota.
  // Pitfall: do not reintroduce a shared threshold with the display/sort classification —
  //   "is a touch worth firing" and "should a human be warned" are different questions that
  //   only happened to share a constant by historical accident.
  let five_h_running = data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ).is_some();
  let d7_running     = data.seven_day.as_ref().map_or( true, |p| p.resets_at.is_some() );
  let son_running    = data.seven_day_sonnet.as_ref().map_or( true, |p| p.resets_at.is_some() );
  let all_running    = five_h_running && d7_running && son_running;
  let h_left  = five_hour_left( aq );
  let d7_left = seven_day_left( aq );
  if all_running || h_left <= 0.0 || d7_left <= 0.0
  {
    return Some( if all_running    { "skipped (reason: already active)" }
      else if h_left  <= 0.0      { "skipped (reason: h-exhausted)"    }
      else                         { "skipped (reason: 7d-exhausted)"   } );
  }

  None
}

/// Activate an idle 5h session window for `aq` by spawning an isolated subprocess.
///
/// The trigger requires both conditions:
/// - `aq.result.is_ok()` — account must have valid quota data (not an auth error).
/// - `five_hour.resets_at.is_none()` — 5h window is idle (no active session).
///
/// After a successful touch, quota is re-fetched so the table shows the concrete
/// `5h Reset` value. When the quota endpoint has not yet propagated the new session
/// (`five_hour.resets_at` still absent in the re-fetch — see BUG-488), the row carries
/// `touched_at_secs` so the text render projects a `~in Xh Ym` countdown from that instant
/// instead of showing the idle `—`, and the `last_touch_at`/`touch_idle` cache flags carry
/// the fact across invocations. If the
/// subprocess or re-fetch fails the account row is unchanged
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
pub fn apply_touch(
  aq               : &mut AccountQuota,
  credential_store : &std::path::Path,
  claude_paths     : Option< &crate::ClaudePaths >,
  trace            : bool,
  imodel           : SubprocessModel,
  effort           : SubprocessEffort,
  solo             : bool,
)
{
  if let Some( reason ) = touch_skip_reason( aq, credential_store, solo )
  {
    if trace { let _ = writeln!( std::io::stderr(), "{}touch  {}  {}", trace_ts(), aq.name, reason ); }
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
    // Fix(BUG-488): a successful touch persists the coordination flags and marks the row
    //   touched for this invocation's render — before this, the loop path wrote neither,
    //   so nothing bridged the quota endpoint's propagation lag and the table rendered
    //   every just-touched account as still idle ("5h Reset —", 100%).
    // Root cause: only apply_post_switch_touch wrote the flags (and to the wrong
    //   directory); the AC-03 re-fetch below can still see the pre-touch idle state
    //   because the endpoint lags session starts.
    // Pitfall: gate on new_creds — refresh_account_token returns None on any failure,
    //   and flags written for a failed touch would wrongly suppress the retry for
    //   TOUCH_GRACE_SECS.
    mark_touched( credential_store, &aq.name );
    aq.touched_at_secs = Some( now_unix_secs() );
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
    // Fix(BUG-309): re-fetch block cleared only aq.result — cached flag, cache_age_secs,
    //   and write_quota_cache were all absent, mirroring the BUG-256 omission in refresh.rs.
    //   {name}.json retained pre-touch quota (resets_at=null); cache-fallback accounts kept
    //   ~ markers and (Xh ago) label even after a successful live re-fetch.
    // Root cause: apply_touch was implemented after Fix(BUG-256) corrected apply_refresh,
    //   but the three post-fetch mutations were never propagated to this re-fetch block.
    // Pitfall: extract h5/d7/sn BEFORE moving new_data into aq.result — use-after-move otherwise.
    let h5 = new_data.five_hour.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
    let d7 = new_data.seven_day.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
    let sn = new_data.seven_day_sonnet.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
    claude_profile_core::account::write_quota_cache( credential_store, &aq.name, h5, d7, sn );
    aq.result         = Ok( new_data );
    aq.cached         = false;
    aq.cache_age_secs = None;
    if let Ok( acct ) = claude_quota::fetch_oauth_account( &token )
    {
      aq.account = Some( acct );
    }
  }
}
