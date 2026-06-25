//! Public API surface for the `.usage` command and account-use touch context.
//!
//! Exports: `PreSwitchOutcome`, `validate_imodel_str`, `validate_effort_str`,
//! `pre_switch_touch_ctx`, `apply_post_switch_touch`, `usage_routine`.
//! Internal: `apply_model_override` (used within api.rs; not re-exported from the `usage` module).

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use claude_quota::OauthUsageData;
use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort, UsageOutputFormat };
use super::subprocess::{ resolve_model, resolve_effort };
use super::fetch::fetch_quota_for_list;
use super::render::{ render_text, render_json, render_tsv, render_plain, extract_get_field };
use super::live::execute_live_mode;
use super::refresh::apply_refresh;
use super::touch::apply_touch;
use super::params::parse_usage_params;
use super::sort::find_next_for_strategy;
use super::format::{ five_hour_left, seven_day_left, status_emoji, OPUS_OVERRIDE_THRESHOLD };

// ── no_color post-processor ────────────────────────────────────────────────────

/// Strip emoji and replace status symbols with plain-text equivalents.
///
/// Replaces: `🟢`→`ok`, `🟡`→`warn`, `🔴`→`err`, `→`→`->`, `✓`→`*`.
/// Used when `no_color::1` is set (AC-14 / TSK-224).
fn apply_no_color( s : String ) -> String
{
  s
    .replace( "🟢", "ok" )
    .replace( "🟡", "warn" )
    .replace( "🔴", "err" )
    .replace( '→', "->" )
    .replace( '✓', "*" )
}

// ── TouchCtx ─────────────────────────────────────────────────────────────────

/// Opaque context holding pre-fetched data for the post-switch idle touch.
///
/// Created by [`pre_switch_touch_ctx`] before the account switch; consumed by
/// [`apply_post_switch_touch`] after. `commands.rs` treats this as a black box.
#[ derive( Debug ) ]
pub( crate ) struct TouchCtx
{
  /// Pre-fetched quota data used to resolve the subprocess model.
  quota : OauthUsageData,
}

#[ cfg( test ) ]
impl TouchCtx
{
  fn for_test( quota : claude_quota::OauthUsageData ) -> Self
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
#[ derive( Debug ) ]
pub( crate ) enum PreSwitchOutcome
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
pub( crate ) fn validate_imodel_str( s : &str ) -> Result< (), String >
{
  SubprocessModel::parse( s ).map( |_| () )
}

/// Validate an `effort::` string value.
///
/// Returns `Err(message)` if unrecognised. Called by `account_use_routine` during
/// argument parsing, before any switch occurs.
pub( crate ) fn validate_effort_str( s : &str ) -> Result< (), String >
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
pub( crate ) fn attempt_expired_token_refresh(
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
/// When `trace` is true, emits `[trace] account.use  {name}  {step}` lines to stderr
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
//   the same pass — grep [trace] emission sites in source and verify each emitting command registers trace::.
pub( crate ) fn pre_switch_touch_ctx(
  name       : &str,
  store_path : &std::path::Path,
  trace      : bool,
  _imodel_str : &str,
  _effort_str : &str,
) -> PreSwitchOutcome
{
  let path = store_path.join( format!( "{name}.credentials.json" ) );
  if trace { eprintln!( "[trace] account.use  {name}  reading {}", path.display() ) }
  let credentials_json = match std::fs::read_to_string( &path )
  {
    Ok( s )  => { if trace { eprintln!( "[trace] account.use  {name}  reading: OK" ) } s }
    Err( e ) =>
    {
      if trace
      {
        eprintln!( "[trace] account.use  {name}  reading: Err({e})" );
        eprintln!( "[trace] account.use  {name}  subprocess: skipped (reason: fetch failed)" );
      }
      return PreSwitchOutcome::Unavailable;
    }
  };
  let Some( token ) = crate::account::parse_string_field( &credentials_json, "accessToken" ) else
  {
    if trace
    {
      eprintln!( "[trace] account.use  {name}  quota fetch: Err(no accessToken in credentials)" );
      eprintln!( "[trace] account.use  {name}  subprocess: skipped (reason: fetch failed)" );
    }
    return PreSwitchOutcome::Unavailable;
  };
  let quota = match claude_quota::fetch_oauth_usage( &token )
  {
    Ok( q )  => { if trace { eprintln!( "[trace] account.use  {name}  quota fetch: OK" ) } q }
    Err( e ) =>
    {
      if trace
      {
        eprintln!( "[trace] account.use  {name}  quota fetch: Err({e})" );
        eprintln!( "[trace] account.use  {name}  subprocess: skipped (reason: fetch failed)" );
      }
      return PreSwitchOutcome::Unavailable;
    }
  };
  // Fix(BUG-285): removed is_idle check — resets_at is server-side state set by any session
  //   on any machine; using it as a local subprocess identity oracle is a category error.
  //   Always return NeedTouch; the subprocess (claude --print .) is idempotent.
  if trace { eprintln!( "[trace] account.use  {name}  subprocess: scheduled (idle check removed)" ) }
  PreSwitchOutcome::NeedTouch( TouchCtx { quota } )
}

/// Spawn an isolated subprocess to activate the idle 5h session window for `name`.
///
/// Called AFTER `switch_account()` succeeds. Uses quota data fetched before the switch
/// (held in `ctx`) for model resolution. The subprocess is fire-and-forget; any
/// failure is silently ignored — the switch has already succeeded.
///
/// When `trace` is true, emits `[trace] account.use  {name}  model: ...  effort: ...` and
/// `[trace] account.use  {name}  subprocess: spawned` to stderr after dispatching.
/// When the Sonnet→Opus override fires (BUG-225), also emits
/// `[trace] account.use  {name}  model override: sonnet→opus (7d(Son) left={N}%)`.
///
/// `imodel_str` and `effort_str` must have been pre-validated by [`validate_imodel_str`]
/// / [`validate_effort_str`]; the `parse()` calls below are infallible on validated input.
/// Quota-aware Sonnet→Opus session model override (BUG-225 fix).
///
/// Called AFTER `switch_account()` for every fetch-succeeded case — both idle and
/// already-active accounts. When `seven_day_sonnet` remaining is below 15% and the
/// current session model is Sonnet (or empty), overrides `~/.claude/settings.json`
/// to `claude-opus-4-6`.
///
/// # Limitation (BUG-226)
///
/// When the quota fetch failed entirely (`PreSwitchOutcome::Unavailable`), this
/// function is not called — the snapshot model from `switch_account()` is kept as-is.
// Fix(BUG-244): apply_model_override was never called from usage_routine; trace prefix was hardcoded.
// Root cause: function had no label param; caller context (account.use vs usage) was indistinguishable.
// Pitfall: insert the usage_routine call BEFORE row-filter pipeline — filters can remove is_current from slice.
pub( crate ) fn apply_model_override(
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
    let sonnet_left = 100.0 - sonnet.utilization;
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
          let _ = writeln!( std::io::stderr(), "[trace] {label}  {name}  model override: sonnet→opus (7d(Son) left={sonnet_left:.0}%)" );
        }
      }
    }
    else
    {
      let overrode = crate::account::override_session_model_to_sonnet( paths );
      if overrode && trace
      {
        use std::io::Write as _;
        let _ = writeln!( std::io::stderr(), "[trace] {label}  {name}  model override: opus→sonnet (7d(Son) left={sonnet_left:.0}%)" );
      }
    }
  }
  else
  {
    // Sonnet tier absent — write "sonnet" conservatively (absent tier ≠ exhausted).
    let _ = crate::account::override_session_model_to_sonnet( paths );
  }
  // Fix(BUG-312): effortLevel never initialized; footer always omitted effort.
  // Root cause: set_session_effort() only called in .usage rotate::1 (carry-forward);
  //   neither .account.use nor plain .usage ever initialized effortLevel in settings.json.
  // Pitfall: only initialize — never overwrite user-configured effort.
  if claude_profile_core::account::get_session_effort( paths ).is_none()
  {
    claude_profile_core::account::set_session_effort( paths, "low" );
  }
}


// Fix(BUG-207): `apply_post_switch_touch` had no `trace` param — model/effort resolution
//   and subprocess spawn were invisible; only the missing [trace] lines in `pre_switch_touch_ctx`
//   were apparent; both functions required the same fix.
// Root cause: Same as `pre_switch_touch_ctx` — Feature 027 "Out of Scope" deferral.
// Pitfall: When a function is split across pre/post phases, both halves need the same diagnostic
//   param — adding trace:: to one without the other leaves half the operation blind.
// Pitfall: `credential_store` must be `PersistPaths::credential_store()` — NOT `paths.base()`.
//   `paths.base()` is `~/.claude/` (Claude config dir); the credential store is
//   `~/.persistent/claude/credential/`. Passing `paths.base()` causes `refresh_account_token`
//   to silently fail — `{name}.credentials.json` doesn't exist in `~/.claude/`, so
//   `refresh_token_with_live_path` returns `None` immediately without rotating the RT.
pub( crate ) fn apply_post_switch_touch(
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
  apply_model_override( &ctx.quota, paths, trace, "account.use", name );
  // Build a minimal AccountQuota to reuse the existing resolve_model() path.
  let aq = AccountQuota
  {
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
  if trace { eprintln!( "[trace] account.use  {name}  model: {model_str}  effort: {effort_label}" ) }
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
  if trace { eprintln!( "[trace] account.use  {name}  subprocess: spawned" ) }
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
  //   paths.base() = ~/.claude/ (Claude config dir); the credential file lives in credential_store.
  //   Reading paths.base()/{name}.credentials.json silently returns Err (file absent) — quota cache
  //   was never updated after touch. Same fix applied to write_quota_cache call below.
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

// ── Main routine ──────────────────────────────────────────────────────────────

/// `.usage` — show live quota utilization for all saved accounts.
///
/// Enumerates `{credential_store}/*.credentials.json`, fetches rate-limit
/// headers per account, and renders a `data_fmt` table (or JSON array with
/// `format::json`).
///
/// # Errors
///
/// Returns `ErrorData` (exit 2) if HOME/PRO is unset or the credential store
/// exists but cannot be read. Per-account API errors are displayed inline.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn usage_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Fix(TSK-224): format:: is now parsed inside parse_usage_params so that usage-specific
  //   format values (tsv, plain, value) are handled without touching the shared OutputFormat enum.
  // Root cause: OutputFormat is shared across all commands; adding usage-only variants would
  //   require exhaustive match updates in all other command handlers.
  // Pitfall: `OutputOptions::from_cmd` was the prior format parser; it is no longer called from
  //   usage_routine — do NOT reintroduce it here, as it would reject `format::tsv`.
  let params = parse_usage_params( &cmd )?;

  // Live-mode guards — fire BEFORE any network fetch, only when live::1 (AC-31).
  // Pitfall: placing these inside execute_live_mode() (after fetch_all_quota) would
  // require live credentials for offline guard tests it22–it24.
  if params.live == 1
  {
    if params.format == UsageOutputFormat::Json
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "live monitor mode is incompatible with format::json".to_string(),
      ) );
    }
    if params.interval < 30
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "interval must be >= 30".to_string(),
      ) );
    }
    if params.jitter > params.interval
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "jitter must not exceed interval".to_string(),
      ) );
    }
  }

  let persist_paths    = crate::PersistPaths::new()
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot resolve storage root: {e}" ),
    ) )?;
  let credential_store = persist_paths.credential_store();
  let live_creds_file  = crate::ClaudePaths::new()
    .map_or_else( || std::path::PathBuf::from( "/dev/null" ), |p| p.credentials_file() );

  // ── Mutation dispatch (Feature 037 + 064 — unified with .accounts) ─────────
  {
    use unilang::types::Value;
    use core::fmt::Write as _;
    use crate::commands::shared::{ is_dry, resolve_account_name, io_err_to_error_data };

    // REMOVED_TOGGLE checks: assign, unclaim, for, active → migration messages (Feature 064/065).
    if cmd.arguments.contains_key( "assign" )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "assign:: REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)".to_string(),
      ) );
    }
    if cmd.arguments.contains_key( "unclaim" )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "unclaim:: REMOVED — use owner::0 name::X instead (or owner::0 alone to batch-clear)".to_string(),
      ) );
    }
    if cmd.arguments.contains_key( "for" )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "for:: REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)".to_string(),
      ) );
    }
    if cmd.arguments.contains_key( "active" )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "active:: REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)".to_string(),
      ) );
    }

    // ── assignee:: dispatch (Feature 065) ────────────────────────────────────
    if let Some( Value::String( av ) ) = cmd.arguments.get( "assignee" )
    {
      let av = if av == "0"
      {
        // Sentinel "0" expands to current machine identity ($USER@$HOSTNAME).
        claude_profile_core::account::current_identity()
      }
      else
      {
        av.clone()
      };
      let san = | s : &str | -> String
      {
        s.chars().map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } ).collect()
      };
      let ( usr_raw, mch_raw ) = av.split_once( '@' ).ok_or_else( || ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "assignee:: must be USER@MACHINE format (no '@' found) — or use assignee::0 for current machine".to_string(),
      ) )?;
      if usr_raw.is_empty()
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          "assignee:: user component (left of '@') must not be empty".to_string(),
        ) );
      }
      if mch_raw.is_empty()
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          "assignee:: machine component (right of '@') must not be empty".to_string(),
        ) );
      }
      let su      = san( usr_raw );
      let sm      = san( mch_raw );
      let marker  = format!( "_active_{sm}_{su}" );
      let display = format!( "{su}@{sm}" );

      let raw_name = match cmd.arguments.get( "name" )
      {
        Some( Value::String( s ) ) => s.clone(),
        _ => String::new(),
      };
      let name_arg = if raw_name.is_empty()
      {
        raw_name.clone()
      }
      else
      {
        resolve_account_name( &raw_name, &credential_store )?
      };

      if !name_arg.is_empty()
      {
        // Assign: write marker pointing to name_arg.
        let cred_path = credential_store.join( format!( "{name_arg}.credentials.json" ) );
        if !cred_path.exists()
        {
          return Err( ErrorData::new(
            ErrorCode::ArgumentTypeMismatch,
            format!( "account '{name_arg}' not found in credential store" ),
          ) );
        }
        if is_dry( &cmd )
        {
          return Ok( OutputData::new(
            format!( "[dry-run] would assign {name_arg} for {display}  \u{2192}  {marker}\n" ),
            "text",
          ) );
        }
        std::fs::write( credential_store.join( &marker ), name_arg.as_bytes() )
          .map_err( | e | io_err_to_error_data( &e, "usage assignee" ) )?;
        if params.trace { eprintln!( "[trace] usage assignee  write marker: {marker}  →  {name_arg}" ) }
        return Ok( OutputData::new(
          format!( "assigned {name_arg} for {display}  \u{2192}  {marker}\n" ),
          "text",
        ) );
      }
      // Unassign: clear the marker file.
      if is_dry( &cmd )
      {
        return Ok( OutputData::new(
          format!( "[dry-run] would unassign {display}  \u{2192}  {marker} cleared\n" ),
          "text",
        ) );
      }
      let marker_path = credential_store.join( &marker );
      if marker_path.exists()
      {
        std::fs::remove_file( &marker_path )
          .map_err( | e | io_err_to_error_data( &e, "usage assignee unassign" ) )?;
      }
      if params.trace { eprintln!( "[trace] usage assignee  cleared marker: {marker}" ) }
      return Ok( OutputData::new(
        format!( "unassigned {display}  \u{2192}  {marker} cleared\n" ),
        "text",
      ) );
    }

    // owner:: param — explicit ownership assignment/release (Feature 063 + 064).
    let owner_value = match cmd.arguments.get( "owner" )
    {
      Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
      Some( Value::String( _ ) ) =>
        return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch,
          "owner:: value must be non-empty — use owner::0 to clear ownership".into() ) ),
      _ => None,
    };

    // ── owner:: explicit ownership assignment/release (Feature 063 + 064) ──────
    if let Some( ref ov ) = owner_value
    {
      let is_sentinel = ov.as_str() == "0";
      let force       = params.force;
      let is_dry_run  = is_dry( &cmd );

      let raw_name = match cmd.arguments.get( "name" )
      {
        Some( Value::String( s ) ) => s.clone(),
        _ => String::new(),
      };
      let name_arg = if raw_name.is_empty() || raw_name.contains( ',' )
      {
        // Comma-list — defer per-component resolution to dispatch below.
        raw_name.clone()
      }
      else
      {
        resolve_account_name( &raw_name, &credential_store )?
      };

      if raw_name.is_empty()
      {
        // No name:: → batch-clear (owner::0 only; owner::VALUE requires name::).
        if !is_sentinel
        {
          return Err( ErrorData::new(
            ErrorCode::ArgumentTypeMismatch,
            "owner::USER@MACHINE requires name:: to specify the target account".to_string(),
          ) );
        }
        // Batch-clear all accounts currently owned by this identity.
        // Unowned and foreign-owned accounts are skipped with a "skip" message (AC-09).
        let all_accounts = crate::account::list( &credential_store )
          .map_err( |e| ErrorData::new(
            ErrorCode::InternalError,
            format!( "cannot read credential store: {e}" ),
          ) )?;
        let mut out = String::new();
        for acct in &all_accounts
        {
          let json_path = credential_store.join( format!( "{}.json", acct.name ) );
          // No metadata file → silently skip (no ownership info to act on).
          if !json_path.exists() { continue; }
          let acct_owner = crate::account::read_owner( &credential_store, &acct.name );
          if acct_owner.is_empty()
          {
            // Unowned — nothing to clear; skip with message (AC-09).
            writeln!( out, "skip {}", acct.name ).unwrap();
            continue;
          }
          if !force && !crate::account::is_owned( &acct_owner )
          {
            // Owned by another identity — skip with message (AC-09).
            if params.trace { eprintln!( "[trace] usage owner  batch-skip (foreign owner): {}  owner={acct_owner}", acct.name ) }
            writeln!( out, "skip {}", acct.name ).unwrap();
            continue;
          }
          if is_dry_run
          {
            writeln!( out, "[dry-run] would clear owner of {}", acct.name ).unwrap();
            continue;
          }
          crate::account::write_owner( &acct.name, &credential_store, "" )
            .map_err( |e| io_err_to_error_data( &e, "usage owner batch-clear" ) )?;
          if params.trace { eprintln!( "[trace] usage owner  cleared: {}  was={acct_owner}", acct.name ) }
          writeln!( out, "unclaimed {}", acct.name ).unwrap();
        }
        return Ok( OutputData::new( out, "text" ) );
      }

      // name:: present — resolve each component (comma-list supported for owner:: ops).
      let target_names : Vec< String > = if raw_name.contains( ',' )
      {
        raw_name.split( ',' )
          .map( | part | resolve_account_name( part.trim(), &credential_store ) )
          .collect::< Result< Vec< _ >, _ > >()?
      }
      else
      {
        vec![ name_arg ]
      };

      let mut out = String::new();
      for name in &target_names
      {
        let json_path = credential_store.join( format!( "{name}.json" ) );
        if !json_path.exists()
        {
          return Err( ErrorData::new(
            ErrorCode::InternalError,
            format!( "account not found: {name}" ),
          ) );
        }
        // G8 ownership gate — evaluated per account, even in dry-run (AC-16/AC-17).
        let acct_owner = crate::account::read_owner( &credential_store, name );
        if !force && !crate::account::is_owned( &acct_owner )
        {
          return Err( ErrorData::new(
            ErrorCode::ArgumentTypeMismatch,
            format!( "ownership violation: {name} is owned by {acct_owner}" ),
          ) );
        }
        if is_dry_run
        {
          if is_sentinel
          {
            writeln!( out, "[dry-run] would clear owner of {name}" ).unwrap();
          }
          else
          {
            writeln!( out, "[dry-run] would set owner of {name} to {ov}" ).unwrap();
          }
          continue;
        }
        let new_owner = if is_sentinel { "" } else { ov.as_str() };
        crate::account::write_owner( name, &credential_store, new_owner )
          .map_err( |e| io_err_to_error_data( &e, "usage owner" ) )?;
        if params.trace
        {
          eprintln!( "[trace] usage owner  write_owner: OK  name={name} identity={}", if is_sentinel { "(cleared)" } else { ov } );
        }
        if is_sentinel
        {
          writeln!( out, "unclaimed {name}" ).unwrap();
        }
        else
        {
          writeln!( out, "owned {name} by {ov}" ).unwrap();
        }
      }
      return Ok( OutputData::new( out, "text" ) );
    }
  }

  if params.live == 1
  {
    return execute_live_mode( &credential_store, &live_creds_file, &params );
  }

  // BUG-245/246 fix: pre-filter before HTTP fetch loop.
  // account::list() reads the _active_{hostname}_{user} filesystem marker — no HTTP required.
  // When only_active::1, retain reduces the list to ≤1 account before fetch_quota_for_list
  // runs the HTTP loop. apply_touch (below) then also evaluates only the 1-entry slice.
  // Pitfall: model-override block uses is_current (not is_active) and must stay ABOVE the
  //   filter pipeline; it is placed after fetch_quota_for_list returns (line ~455+).
  let mut acct_list : Vec< crate::account::Account > = crate::account::list( &credential_store )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot read credential store: {e}" ),
    ) )?;
  if params.only_active { acct_list.retain( |aq| aq.is_active ); }
  let mut accounts = fetch_quota_for_list( &acct_list, &credential_store, &live_creds_file, false, params.trace, params.solo );

  // Retry-once per account on 401/403 auth errors or 429+locally-expired: if
  // refresh::1 and any account's quota fetch failed with an auth error OR a
  // rate-limit response while its local `expiresAt` is past, refresh that token
  // via an isolated subprocess, then re-fetch only that account's quota.
  // Pure 429 with a non-expired local token is not retried — the token is valid.
  if params.refresh == 1
  {
    let claude_paths = crate::ClaudePaths::new();
    apply_refresh( &mut accounts, &credential_store, claude_paths.as_ref(), params.trace, params.imodel, params.effort, params.solo );
  }

  // touch::1: activate idle 5h windows — runs after refresh so post-refresh results
  // are touched (an account that was refreshed and now has valid quota with no resets_at
  // will be touched; an account that still errors after refresh is skipped by apply_touch).
  if params.touch == 1
  {
    let claude_paths = crate::ClaudePaths::new();
    for aq in &mut accounts
    {
      apply_touch( aq, &credential_store, claude_paths.as_ref(), params.trace, params.imodel, params.effort, params.solo );
    }
  }

  // ── Session-model override (BUG-244: .usage path, AC-32) ──────────────────
  // Must run BEFORE row-filter pipeline — filters can remove is_current from slice.
  // When set_model:: is explicit, write the requested model and skip apply_model_override.
  {
    let claude_paths = crate::ClaudePaths::new();
    if let Some( ref claude_paths ) = claude_paths
    {
      if let Some( ref sm ) = params.set_model
      {
        let model_id = super::types::validate_set_model( sm ).ok().flatten();
        claude_profile_core::account::set_session_model( claude_paths, model_id );
      }
      else if let Some( current ) = accounts.iter().find( |aq| aq.is_current )
      {
        if let Ok( ref data ) = current.result
        {
          apply_model_override( data, claude_paths, params.trace, "usage", &current.name );
        }
      }
    }
  }

  // ── Row filter pipeline (TSK-223) ─────────────────────────────────────────
  // Applied after sort/tier (which runs inside render_text), before render.
  // Filters are AND-composition; count/offset applied last as a window.
  {
    use std::time::{ SystemTime, UNIX_EPOCH };
    let now_secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();

    // only_next: retain only the recommended next account.
    if params.only_next
    {
      let best_opt = find_next_for_strategy( &accounts, params.sort, params.prefer, now_secs, params.rotate && !params.force );
      accounts = match best_opt
      {
        Some( i ) => { let w = accounts.swap_remove( i ); vec![ w ] }
        None      => vec![],
      };
    }

    // Boolean row filters.
    // only_active: pre-filtered before HTTP in fetch_quota_for_list (BUG-245/246 fix).
    // Fix(BUG-317): cancelled accounts (billing_type="none") have result=Ok but are permanently
    //   unusable — exclude them from only_valid and exclude_exhausted results.
    // Root cause: only_valid checked result.is_ok() without inspecting billing_type; cancelled
    //   accounts passed through and appeared as valid. exclude_exhausted delegated to
    //   status_emoji which was also unaware of billing_type.
    // Pitfall: account=None is ambiguous (API fetch failed); is_some_and guards correctly.
    if params.only_valid
    {
      accounts.retain( |aq| aq.result.is_ok() && !aq.account.as_ref().is_some_and( |a| a.billing_type == "none" ) );
    }
    if params.exclude_exhausted { accounts.retain( |aq| status_emoji( aq ) == "🟢" ); }

    // Threshold filters: only applied to accounts with valid quota data.
    // Accounts with no valid quota (Err) pass through — absent data ≠ exhausted.
    if params.min_5h > 0
    {
      let threshold = f64::from( params.min_5h );
      accounts.retain( |aq| aq.result.is_err() || five_hour_left( aq ) >= threshold );
    }
    if params.min_7d > 0
    {
      let threshold = f64::from( params.min_7d );
      accounts.retain( |aq| aq.result.is_err() || seven_day_left( aq ) >= threshold );
    }

    // Pagination window (applied last, after all boolean/threshold filters).
    if params.offset > 0
    {
      let off = usize::try_from( params.offset ).unwrap_or( usize::MAX );
      accounts = accounts.into_iter().skip( off ).collect();
    }
    if params.count > 0 { accounts.truncate( usize::try_from( params.count ).unwrap_or( usize::MAX ) ); }
  }
  // ── End filter pipeline ────────────────────────────────────────────────────

  // abs::1 is registered for future absolute-count display; no-op until API exposes counts.
  let _ = params.abs;

  // `get::` extraction: output the requested field from the first row as a bare string.
  // When accounts is empty after filtering, output nothing (exit 0, empty stdout).
  if let Some( field ) = params.get
  {
    use std::time::{ SystemTime, UNIX_EPOCH };
    let now_secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
    let value = accounts.first()
      .map_or_else( String::new, |aq| extract_get_field( aq, field, now_secs ) );
    let content = if value.is_empty() { String::new() } else { format!( "{value}\n" ) };
    return Ok( OutputData::new( content, "text" ) );
  }

  // Read session state once for the footer; both render_text and render_plain consume it.
  // Reads settings.json once; extracts "model" and "effortLevel".
  // effortLevel is carried forward after rotation when present (Feature 062, AC-06).
  let settings_content = crate::ClaudePaths::new()
    .and_then( |p| std::fs::read_to_string( p.settings_file() ).ok() );
  let session_model_str  = settings_content.as_deref()
    .and_then( |s| crate::account::parse_string_field( s, "model" ) );
  let session_effort_str = settings_content.as_deref()
    .and_then( |s| crate::account::parse_string_field( s, "effortLevel" ) );
  let session_model  = session_model_str.as_deref();
  let session_effort = session_effort_str.as_deref();

  let content = match params.format
  {
    UsageOutputFormat::Json  => render_json( &accounts ),
    UsageOutputFormat::Tsv   => render_tsv( &accounts, params.sort, params.desc, params.prefer, &params.cols ),
    UsageOutputFormat::Plain => render_plain( &accounts, params.sort, params.desc, params.prefer, &params.cols, session_model, session_effort, Some( &credential_store ), params.who ),
    UsageOutputFormat::Value => String::new(),
    UsageOutputFormat::Text  => render_text( &accounts, params.sort, params.desc, params.prefer, &params.cols, session_model, session_effort, Some( &credential_store ), params.who ),
  };

  let content = if params.no_color && params.format != UsageOutputFormat::Tsv
  {
    apply_no_color( content )
  }
  else
  {
    content
  };

  // ── Rotation dispatch (Feature 038 — AC-01..AC-09) ────────────────────────
  // Post-render: find winner → no-eligible check → dry-run → switch → touch → emit.
  // Guard ensures non-rotate invocations are completely unaffected.
  if params.rotate
  {
    use std::time::{ SystemTime, UNIX_EPOCH };
    use crate::commands::shared::{ is_dry, io_err_to_error_data };

    let now_secs       = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
    // gate_ownership: true when rotate::1 without force::1 — G5 applies (AC-05).
    let gate_ownership = !params.force;
    let winner_opt     = find_next_for_strategy( &accounts, params.sort, params.prefer, now_secs, gate_ownership );
    let Some( winner_idx ) = winner_opt
    else
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "no eligible account to rotate to".to_string(),
      ) );
    };
    let winner_name = accounts[ winner_idx ].name.clone();

    if is_dry( &cmd )
    {
      let msg = format!( "{content}\n[dry-run] would switch to '{winner_name}'\n" );
      return Ok( OutputData::new( msg, "text" ) );
    }

    let claude_paths = crate::ClaudePaths::new().ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "$HOME is not set; cannot switch account".to_string(),
    ) )?;

    crate::account::switch_account( &winner_name, &credential_store, &claude_paths )
      .map_err( |e| io_err_to_error_data( &e, "usage rotate" ) )?;

    // AC-05: model override for winner (Feature 062)
    if let Ok( ref winner_data ) = accounts[ winner_idx ].result
    {
      apply_model_override( winner_data, &claude_paths, params.trace, "usage rotate", &winner_name );
    }
    // AC-06/AC-07: carry-forward effort when present; no default injected when absent
    if let Some( se ) = session_effort
    {
      claude_profile_core::account::set_session_effort( &claude_paths, se );
    }

    apply_touch( &mut accounts[ winner_idx ], &credential_store, Some( &claude_paths ), params.trace, params.imodel, params.effort, params.solo );

    // Fix(BUG-310): re-sync live credentials after post-rotation touch.
    // Root cause: switch_account copies store→live BEFORE apply_touch; the touch subprocess
    //   may refresh the token, writing to STORE only via save(update_marker=false); live retains
    //   stale pre-refresh credentials that may be server-invalidated.
    // Pitfall: do NOT call switch_account again — it re-writes _active marker and patches
    //   .claude.json redundantly; a targeted credential file copy suffices.
    let src_cred = credential_store.join( format!( "{winner_name}.credentials.json" ) );
    let _ = std::fs::copy( &src_cred, claude_paths.credentials_file() );

    let msg = format!( "{content}\nswitched to '{winner_name}'\n" );
    return Ok( OutputData::new( msg, "text" ) );
  }

  Ok( OutputData::new( content, "text" ) )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
// Exception to tests-in-tests/ rule: pub(crate) fns (pre_switch_touch_ctx, apply_model_override,
// apply_post_switch_touch, TouchCtx) are not accessible from the external tests/ directory.
#[ path = "api_tests.rs" ]
mod tests;
