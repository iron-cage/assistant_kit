//! Pre/post-switch touch context and model-override helpers.
//!
//! Extracted from `api.rs` — standalone `pub(crate)` functions used by both
//! `account_use_routine` (via module exports) and `usage_routine` (via `apply_model_override`).

use claude_quota::OauthUsageData;
use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort, OPUS_OVERRIDE_THRESHOLD };
use super::subprocess::{ resolve_model, resolve_effort };
use claude_profile_core::account::trace_ts;

// ── TouchCtx ─────────────────────────────────────────────────────────────────

/// Opaque context holding pre-fetched data for the post-switch idle touch.
///
/// Created by [`pre_switch_touch_ctx`] before the account switch; consumed by
/// [`apply_post_switch_touch`] after. `commands.rs` treats this as a black box.
#[ derive( Debug ) ]
pub struct TouchCtx
{
  /// Pre-fetched quota data used to resolve the subprocess model.
  pub( super ) quota : OauthUsageData,
}

#[ cfg( any( test, feature = "testing" ) ) ]
#[ allow( missing_docs, clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
impl TouchCtx
{
  pub fn for_test( quota : claude_quota::OauthUsageData ) -> Self
  {
    Self { quota }
  }
}

/// Result of the pre-switch quota probe for `.account.use`.
///
/// Distinguishes two outcomes: quota available (always spawn subprocess) or unavailable.
// Fix(BUG-238): pre_switch_touch_ctx() returned None for already-active accounts,
//   skipping apply_post_switch_touch() and its BUG-225 Sonnet→Opus override entirely.
// Root cause: quota data was discarded for active accounts — only idle accounts
//   got a TouchCtx, coupling the model override to subprocess dispatch.
// Pitfall: any post-switch side-effect gated on touch_ctx.is_some() is invisible
//   for already-active accounts; always check if the effect needs quota data vs idle state.
// Fix(BUG-285): AlreadyActive variant removed — the is_idle oracle used server-side
//   resets_at as proxy for local subprocess identity (category error). Always return
//   NeedTouch; the subprocess is idempotent and exits immediately when already active.
// Root cause: resets_at is set by any session on any machine — not specific to the local
//   subprocess. Using it as an identity oracle conflates two unrelated concepts.
// Pitfall: never use server-side timer state to infer local subprocess lifecycle.
#[ derive( Debug ) ]
pub enum PreSwitchOutcome
{
  /// Quota fetched — spawn subprocess touch after switch.
  NeedTouch( TouchCtx ),
  /// Quota unavailable (read error, auth error, fetch error) — no override possible.
  Unavailable,
}

// ── Validators ────────────────────────────────────────────────────────────────

/// Validate an `imodel::` string value.
///
/// Returns `Err(message)` if unrecognised. Called by `account_use_routine` during
/// argument parsing, before any switch occurs.
pub fn validate_imodel_str( s : &str ) -> Result< (), String >
{
  SubprocessModel::parse( s ).map( |_| () )
}

/// Validate an `effort::` string value.
///
/// Returns `Err(message)` if unrecognised. Called by `account_use_routine` during
/// argument parsing, before any switch occurs.
pub fn validate_effort_str( s : &str ) -> Result< (), String >
{
  SubprocessEffort::parse( s ).map( |_| () )
}

// ── Expired-token refresh helper ──────────────────────────────────────────────

/// Attempt OAuth token refresh for a locally-expired account credential.
///
/// Resolves the subprocess model from `imodel_str`/`effort_str`, then calls
/// `refresh_account_token()` to spawn an isolated subprocess and rewrite the
/// per-account credential file with a fresh token.
///
/// Returns `true` when `refresh_account_token` succeeds (returns `Some`);
/// `false` when it returns `None` (subprocess failed or credential file lacks `accessToken`).
///
/// # Fix(BUG-230)
/// Called from `account_use_routine` when `expiresAt` is in the past and `refresh::1`.
/// Root cause: BUG-213 guard exited 3 without attempting refresh — token expiry is
///   recoverable via OAuth refresh, not a fatal condition when `refresh::1`.
/// Pitfall: this path requires `credential_store` (not `paths.credentials_file()`) because
///   the per-account file is the refresh source, not the live session credentials file.
pub fn attempt_expired_token_refresh(
  name             : &str,
  credential_store : &std::path::Path,
  paths            : &crate::ClaudePaths,
  trace            : bool,
  imodel_str       : &str,
  effort_str       : &str,
) -> bool
{
  let imodel    = SubprocessModel::parse( imodel_str ).unwrap_or( SubprocessModel::Auto );
  let effort    = SubprocessEffort::parse( effort_str ).unwrap_or( SubprocessEffort::Auto );
  // Build a minimal AccountQuota for model resolution.
  // result=Err("401") drives auto model selection to Opus (conservative when no quota data).
  let aq        = AccountQuota
  {
    fallback_reason : None,
    name                 : name.to_string(),
    is_current           : false,
    is_active            : false,
    is_occupied_elsewhere : false,
    expires_at_ms        : 0,
    result               : Err( "401".to_string() ),
    account              : None,
    host                 : String::new(),
    role                 : String::new(),
    renewal_at           : None,
    cached               : false,
    cache_age_secs       : None,
    org_created_at       : None,
    is_owned             : true,
    owner                : String::new(),
  };
  let model     = super::subprocess::resolve_model( &aq, imodel );
  let pre_args  = super::subprocess::effort_pre_args( &model, effort );
  crate::account::refresh_account_token(
    name, credential_store, Some( paths ), trace, "account.use", model, &pre_args,
  ).is_some()
}

// ── Pre/post-switch touch context ─────────────────────────────────────────────

/// Pre-fetch quota for `name` and return a touch context for post-switch subprocess dispatch.
///
/// Returns [`PreSwitchOutcome::Unavailable`] when any of the following hold:
/// - credentials file missing or lacks `accessToken`
/// - quota API fetch fails
///
/// Returns [`PreSwitchOutcome::NeedTouch`] when quota is successfully fetched.
/// The subprocess is always dispatched; it exits immediately when the account is already active.
///
/// When `trace` is true, emits `YYYY-MM-DD · HH:MM:SS · account.use  {name}  {step}` lines to stderr
/// for each internal operation, including the reason when `Unavailable` is returned.
///
/// Called BEFORE the switch so the target account's credential file still holds the
/// pre-switch token. The returned `TouchCtx` is passed through the switch and consumed
/// by [`apply_post_switch_touch`] after `switch_account()` returns.
// Fix(BUG-207): `pre_switch_touch_ctx` had no `trace` param — credential read, quota fetch,
//   and skip-reason were all invisible; the caller always saw "switched to '{name}'".
// Root cause: Feature 027 scope explicitly deferred trace:: as "Out of Scope"; no rule required
//   trace:: on commands performing fetch operations.
// Pitfall: Any command extended to perform HTTP/file/subprocess operations must add trace:: in
//   the same pass — grep trace_ts() call sites in source and verify each emitting command registers trace::.
#[ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
pub fn pre_switch_touch_ctx(
  name       : &str,
  store_path : &std::path::Path,
  trace      : bool,
  _imodel_str : &str,
  _effort_str : &str,
) -> PreSwitchOutcome
{
  let path = store_path.join( format!( "{name}.credentials.json" ) );
  if trace { eprintln!( "{}account.use  {name}  reading {}", trace_ts(), path.display() ) }
  let credentials_json = match std::fs::read_to_string( &path )
  {
    Ok( s )  => { if trace { eprintln!( "{}account.use  {name}  reading: OK", trace_ts() ) } s }
    Err( e ) =>
    {
      if trace
      {
        eprintln!( "{}account.use  {name}  reading: Err({e})", trace_ts() );
        eprintln!( "{}account.use  {name}  subprocess: skipped (reason: fetch failed)", trace_ts() );
      }
      return PreSwitchOutcome::Unavailable;
    }
  };
  let Some( token ) = crate::account::parse_string_field( &credentials_json, "accessToken" ) else
  {
    if trace
    {
      eprintln!( "{}account.use  {name}  quota fetch: Err(no accessToken in credentials)", trace_ts() );
      eprintln!( "{}account.use  {name}  subprocess: skipped (reason: fetch failed)", trace_ts() );
    }
    return PreSwitchOutcome::Unavailable;
  };
  let quota = match claude_quota::fetch_oauth_usage( &token )
  {
    Ok( q )  => { if trace { eprintln!( "{}account.use  {name}  quota fetch: OK", trace_ts() ) } q }
    Err( e ) =>
    {
      if trace
      {
        eprintln!( "{}account.use  {name}  quota fetch: Err({e})", trace_ts() );
        eprintln!( "{}account.use  {name}  subprocess: skipped (reason: fetch failed)", trace_ts() );
      }
      return PreSwitchOutcome::Unavailable;
    }
  };
  // Fix(BUG-285): removed is_idle check — resets_at is server-side state set by any session
  //   on any machine; using it as a local subprocess identity oracle is a category error.
  //   Always return NeedTouch; the subprocess (claude --print .) is idempotent.
  // Root cause: same as enum declaration site — server-side timer ≠ local identity.
  // Pitfall: idempotent subprocess is always safe to spawn; the guard to remove was wrong-level.
  if trace { eprintln!( "{}account.use  {name}  subprocess: scheduled (idle check removed)", trace_ts() ) }
  PreSwitchOutcome::NeedTouch( TouchCtx { quota } )
}

/// Apply the Sonnet→Opus (or Opus→Sonnet) session model override based on quota utilization.
///
/// Called AFTER `switch_account()` for every fetch-succeeded case — both idle and
/// already-active accounts. When `seven_day_sonnet` remaining is below 15% and the
/// current session model is Sonnet (or empty), overrides `~/.claude/settings.json`
/// to `claude-opus-4-8`.
///
/// # Limitation (BUG-226)
///
/// When the quota fetch failed entirely (`PreSwitchOutcome::Unavailable`), this
/// function is not called — the snapshot model from `switch_account()` is kept as-is.
// Fix(BUG-244): apply_model_override was never called from usage_routine; trace prefix was hardcoded.
// Root cause: function had no label param; caller context (account.use vs usage) was indistinguishable.
// Pitfall: insert the usage_routine call BEFORE row-filter pipeline — filters can remove is_current from slice.
#[ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
pub fn apply_model_override(
  quota : &OauthUsageData,
  paths : &crate::ClaudePaths,
  trace : bool,
  label : &str,
  name  : &str,
)
{
  // Fix(BUG-300): map_or(0.0,...) treated seven_day_sonnet=None as 0% left — Opus override fired
  //   unconditionally for any account without a Sonnet tier.
  // Root cause: None means "tier absent/unknown", not "fully exhausted"; 0.0 < 20.0 always fires.
  // Pitfall: guard override on Some(ref sonnet) — absent tier must never trigger quota-exhaustion logic.
  // Fix(BUG-311): was one-way (sonnet→opus only); settings.json retained stale "opus" after switching
  //   to an account with sufficient Sonnet quota — no code wrote "sonnet" back.
  // Root cause: the else-branch was absent; model state was never reset when quota recovered.
  // Pitfall: use override_session_model_to_sonnet() to avoid redundant writes when already "sonnet".
  if let Some( ref sonnet ) = quota.seven_day_sonnet
  {
    // Fix(BUG-331): compared raw `sonnet_left` against threshold but rounded only for the trace
    //   messages, so the logged percentage could read identically regardless of which branch
    //   actually fired — and unlike the display-only pct_emoji case, the wrong branch here has
    //   a real side effect (it flips the session model).
    //   Root cause: same class as pct_emoji — `sonnet_left` computed once but consumed twice,
    //   raw for the comparison, rounded for the `{sonnet_left:.0}%` trace text.
    //   Pitfall: always round once and reuse the rounded value for both the branch comparison
    //   and the trace text; never compare a raw float against a threshold when the log shows
    //   a rounded value derived from the same float.
    let sonnet_left = ( 100.0 - sonnet.utilization ).round();
    if sonnet_left < OPUS_OVERRIDE_THRESHOLD
    {
      let overrode = crate::account::override_session_model_to_opus( paths );
      if overrode
      {
        claude_profile_core::account::write_cache_string(
          paths.base(), name, "model_override", "opus",
        );
        if trace
        {
          use std::io::Write as _;
          let _ = writeln!( std::io::stderr(), "{}{label}  {name}  model override: sonnet→opus (7d(Son) left={sonnet_left:.0}%)  effort→max", trace_ts() );
        }
      }
      // Fix(H2/TSK-335): effort synced unconditionally — stable sessions must get effort even when model unchanged.
      // Root cause: gating set_session_effort on overrode=true left effortLevel stale when model already matched target.
      // Pitfall: write AFTER the overrode block so model is finalized before effort is set.
      claude_profile_core::account::set_session_effort( paths, "max" );
    }
    else
    {
      let overrode = crate::account::override_session_model_to_sonnet( paths );
      if overrode && trace
      {
        use std::io::Write as _;
        let _ = writeln!( std::io::stderr(), "{}{label}  {name}  model override: opus→sonnet (7d(Son) left={sonnet_left:.0}%)  effort→high", trace_ts() );
      }
      // Fix(H2/TSK-335): effort synced unconditionally — stable sessions must get effort even when model unchanged.
      // Root cause: gating set_session_effort on overrode=true left effortLevel stale when model already matched target.
      // Pitfall: write AFTER the overrode block so model is finalized before effort is set.
      claude_profile_core::account::set_session_effort( paths, "high" );
    }
  }
  else
  {
    // Sonnet tier absent — write "sonnet" conservatively (absent tier ≠ exhausted).
    // Fix(BUG-322): also reset effort when model changes away from opus.
    // Root cause: same as site 1 — effort was never paired with model at any fix site except site 1.
    // Fix(H2/TSK-335): effort synced unconditionally regardless of override_session_model_to_sonnet return.
    // Root cause: gating effort on return value left stable sessions stale when model already matched.
    // Pitfall: call override_session_model_to_sonnet for its side effect; always write "high" after.
    let _ = crate::account::override_session_model_to_sonnet( paths );
    claude_profile_core::account::set_session_effort( paths, "high" );
  }
  // Fix(BUG-312): effortLevel never initialized; footer always omitted effort.
  // Root cause: set_session_effort() only called in .usage rotate::1 (carry-forward);
  //   neither .account.use nor plain .usage ever initialized effortLevel in settings.json.
  // Pitfall: after TSK-335, this guard is unreachable — all 3 branches above write effort
  //   unconditionally. Retained as safety net for unforeseen absent-effort states.
  if claude_profile_core::account::get_session_effort( paths ).is_none()
  {
    claude_profile_core::account::set_session_effort( paths, "high" );
  }
}

// Fix(BUG-207): `apply_post_switch_touch` had no `trace` param — model/effort resolution
//   and subprocess spawn were invisible; only the missing trace lines in `pre_switch_touch_ctx`
//   were apparent; both functions required the same fix.
// Root cause: Same as `pre_switch_touch_ctx` — Feature 027 "Out of Scope" deferral.
// Pitfall: When a function is split across pre/post phases, both halves need the same diagnostic
//   param — adding trace:: to one without the other leaves half the operation blind.
// Pitfall: `credential_store` must be `PersistPaths::credential_store()` — NOT `paths.base()`.
//   `paths.base()` is `~/.claude/` (Claude config dir); the credential store is
//   `~/.persistent/claude/credential/`. Passing `paths.base()` causes `refresh_account_token`
//   to silently fail — `{name}.credentials.json` doesn't exist in `~/.claude/`, so
//   `refresh_token_with_live_path` returns `None` immediately without rotating the RT.
#[ allow( missing_docs, clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
pub fn apply_post_switch_touch(
  name             : &str,
  ctx              : TouchCtx,
  imodel_str       : &str,
  effort_str       : &str,
  trace            : bool,
  paths            : &crate::ClaudePaths,
  credential_store : &std::path::Path,
)
{
  let imodel = SubprocessModel::parse( imodel_str ).unwrap_or( SubprocessModel::Auto );
  let effort = SubprocessEffort::parse( effort_str ).unwrap_or( SubprocessEffort::Auto );
  // Fix(BUG-225): delegate to apply_model_override for the Sonnet→Opus check.
  // Root cause: switch_account() restored snapshot model without checking current Sonnet
  //   quota utilization — restored "sonnet" persisted even when 7d(Son) was exhausted.
  // Pitfall: always apply quota-aware model override AFTER restoring the snapshot model;
  //   snapshot model is stale by definition.
  apply_model_override( &ctx.quota, paths, trace, "account.use", name );
  // Build a minimal AccountQuota to reuse the existing resolve_model() path.
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                 : name.to_string(),
    is_current           : false,
    is_active            : false,
    is_occupied_elsewhere : false,
    expires_at_ms        : 0,
    result               : Ok( ctx.quota ),
    account              : None,
    host                 : String::new(),
    role                 : String::new(),
    renewal_at           : None,
    cached               : false,
    cache_age_secs       : None,
    org_created_at       : None,
    is_owned             : true,
    owner                : String::new(),
  };
  let model        = resolve_model( &aq, imodel );
  let effort_val   = resolve_effort( &model, effort );
  let model_str    = match &model
  {
    claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(),
    _                                                => "keep-current",
  };
  let effort_label = effort_val.unwrap_or( "(none)" );
  if trace { eprintln!( "{}account.use  {name}  model: {model_str}  effort: {effort_label}", trace_ts() ) }
  let extra_pre_args = match effort_val
  {
    Some( e ) => vec![ "--effort".to_string(), e.to_string() ],
    None      => vec![],
  };
  // AC-34 / Invariant 008: route through refresh_account_token instead of direct run_isolated.
  // refresh_account_token internally appends ["--print", "."] and applies:
  //   - expiresAt=1 manipulation (Feature 017 AC-32): forces RT rotation on every call
  //   - live credential sync for current account (Feature 017 AC-33): avoids redundant subprocess
  let _ = crate::account::refresh_account_token(
    name, credential_store, Some( paths ), trace, "account.use", model, &extra_pre_args,
  );
  // Persist touch timestamp to cache (Feature 033 AC-06).
  claude_profile_core::account::write_cache_string(
    paths.base(), name, "last_touch_at", &claude_profile_core::account::chrono_now_utc(),
  );
  claude_profile_core::account::write_cache_bool(
    paths.base(), name, "touch_idle", false,
  );
  if trace { eprintln!( "{}account.use  {name}  subprocess: spawned", trace_ts() ) }
  // AC-21: post-subprocess quota re-fetch (best-effort, non-aborting on failure).
  // Persists updated resets_at to cache so subsequent .usage sees the newly-activated
  // session window, preventing the double-subprocess race (BUG-288).
  // Fix(BUG-288): apply_post_switch_touch previously omitted this re-fetch,
  //   causing apply_touch to see stale resets_at = None and spawn a redundant subprocess.
  // Root cause: AC-21 was not defined when this function was first written; the re-fetch
  //   was present in apply_touch (Feature 024 AC-03) but not mirrored here.
  // Pitfall: any post-switch touch function must re-fetch quota after subprocess to keep
  //   the cache consistent with the newly-activated session window.
  // Fix(BUG-318): use credential_store (not paths.base()) for the AC-21 re-fetch credential read.
  // Root cause: paths.base() = ~/.claude/ (Claude config dir); credential files live in
  //   credential_store. Reading paths.base()/{name}.credentials.json silently returns Err —
  //   quota cache was never updated after touch.
  // Pitfall: paths.base() and credential_store are distinct directory roots; never substitute one
  //   for the other when reading per-account credential files.
  let cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  if let Ok( fresh_json ) = std::fs::read_to_string( &cred_path )
  {
    if let Some( token ) = crate::account::parse_string_field( &fresh_json, "accessToken" )
    {
      if let Ok( new_data ) = claude_quota::fetch_oauth_usage( &token )
      {
        let h5 = new_data.five_hour.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        let d7 = new_data.seven_day.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        let sn = new_data.seven_day_sonnet.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        claude_profile_core::account::write_quota_cache( credential_store, name, h5, d7, sn );
      }
    }
  }
}
