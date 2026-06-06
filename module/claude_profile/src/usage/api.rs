//! Public API surface for the `.usage` command and account-use touch context.
//!
//! Exports: `PreSwitchOutcome`, `validate_imodel_str`, `validate_effort_str`,
//! `pre_switch_touch_ctx`, `apply_model_override`, `apply_post_switch_touch`, `usage_routine`.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use claude_quota::OauthUsageData;
use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort, UsageOutputFormat };
use super::subprocess::{ resolve_model, resolve_effort };
use super::fetch::fetch_all_quota;
use super::render::{ render_text, render_json, render_tsv, render_plain, extract_get_field };
use super::live::execute_live_mode;
use super::refresh::apply_refresh;
use super::touch::apply_touch;
use super::params::parse_usage_params;
use super::sort::find_next_for_strategy;
use super::format::{ five_hour_left, seven_day_left, status_emoji };

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
  /// Raw credentials JSON read from the account credential file before the switch.
  credentials_json : String,
  /// Pre-fetched quota data used to resolve the subprocess model.
  quota            : OauthUsageData,
}

/// Result of the pre-switch quota probe for `.account.use`.
///
/// Distinguishes three outcomes so the caller can apply the model override
/// for all fetch-succeeded cases, not just idle accounts.
// Fix(BUG-238): pre_switch_touch_ctx() returned None for already-active accounts,
//   skipping apply_post_switch_touch() and its BUG-225 Sonnet→Opus override entirely.
// Root cause: quota data was discarded for active accounts — only idle accounts
//   got a TouchCtx, coupling the model override to subprocess dispatch.
// Pitfall: any post-switch side-effect gated on touch_ctx.is_some() is invisible
//   for already-active accounts; always check if the effect needs quota data vs idle state.
#[ derive( Debug ) ]
pub( crate ) enum PreSwitchOutcome
{
  /// Quota fetched, account idle — needs subprocess touch after switch.
  NeedTouch( TouchCtx ),
  /// Quota fetched, account already active — model override only, no subprocess.
  AlreadyActive { quota : OauthUsageData },
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
  };
  let model     = super::subprocess::resolve_model( &aq, imodel );
  let pre_args  = super::subprocess::effort_pre_args( &model, effort );
  crate::account::refresh_account_token(
    name, credential_store, Some( paths ), trace, "account.use", model, &pre_args,
  ).is_some()
}

// ── Pre/post-switch touch context ─────────────────────────────────────────────

/// Pre-fetch quota for `name` and return a [`TouchCtx`] when the account is idle.
///
/// Returns `None` when any of the following hold:
/// - credentials file missing or lacks `accessToken`
/// - quota API fetch fails
/// - account already has an active 5h reset countdown (`five_hour.resets_at.is_some()`)
///
/// When `trace` is true, emits `[trace] account.use  {name}  {step}` lines to stderr
/// for each internal operation, including the reason when `None` is returned.
///
/// Called BEFORE the switch so the target account's credential file still holds the
/// pre-switch token. The returned `TouchCtx` is passed through the switch and consumed
/// by [`apply_post_switch_touch`] after `switch_account()` returns.
// Fix(BUG-207): `pre_switch_touch_ctx` had no `trace` param — credential read, quota fetch,
//   idle check, and skip-reason were all invisible; the caller always saw "switched to '{name}'".
// Root cause: Feature 027 scope explicitly deferred trace:: as "Out of Scope"; no rule required
//   trace:: on commands performing fetch operations.
// Pitfall: Any command extended to perform HTTP/file/subprocess operations must add trace:: in
//   the same pass — grep [trace] emission sites in source and verify each emitting command registers trace::.
// Fix(BUG-210): `pre_switch_touch_ctx` emitted no model/effort trace in the already-active branch.
// Root cause: model/effort resolution lived in `apply_post_switch_touch` only — unreachable when
//   the already-active skip path fires.
// Pitfall: any new skip path that bypasses `apply_post_switch_touch` will also miss model/effort
//   unless it calls `resolve_model`/`resolve_effort` directly.
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
  let is_idle = quota.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ).is_none();
  if is_idle
  {
    if trace { eprintln!( "[trace] account.use  {name}  idle check: resets_at=absent → idle" ) }
    PreSwitchOutcome::NeedTouch( TouchCtx { credentials_json, quota } )
  }
  else
  {
    if trace { eprintln!( "[trace] account.use  {name}  idle check: resets_at=present → already active" ) }
    PreSwitchOutcome::AlreadyActive { quota }
  }
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
/// already-active accounts. When `seven_day_sonnet` remaining is below 20% and the
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
  let sonnet_left = quota.seven_day_sonnet.as_ref().map_or( 0.0, | p | 100.0 - p.utilization );
  if sonnet_left < 20.0
  {
    let overrode = crate::account::override_session_model_to_opus( paths );
    if overrode && trace
    {
      eprintln!( "[trace] {label}  {name}  model override: sonnet→opus (7d(Son) left={sonnet_left:.0}%)" );
    }
  }
}

/// Emit the `model:` and `subprocess: skipped` trace lines for already-active accounts.
///
/// Takes `quota` by value because `AccountQuota.result` consumes it.
/// Called from `account_use_routine()` when the outcome is `AlreadyActive`.
pub( crate ) fn trace_already_active(
  name       : &str,
  quota      : OauthUsageData,
  imodel_str : &str,
  effort_str : &str,
)
{
  let imodel     = SubprocessModel::parse( imodel_str ).unwrap_or( SubprocessModel::Auto );
  let effort     = SubprocessEffort::parse( effort_str ).unwrap_or( SubprocessEffort::Auto );
  let aq         = AccountQuota
  {
    name                 : name.to_string(),
    is_current           : false,
    is_active            : false,
    is_occupied_elsewhere : false,
    expires_at_ms        : 0,
    result               : Ok( quota ),
    account              : None,
    host                 : String::new(),
    role                 : String::new(),
    renewal_at           : None,
  };
  let model      = resolve_model( &aq, imodel );
  let effort_val = resolve_effort( &model, effort );
  let model_str  = match &model
  {
    claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(),
    _                                                => "keep-current",
  };
  let effort_label = effort_val.unwrap_or( "(none)" );
  eprintln!( "[trace] account.use  {name}  model: {model_str}  effort: {effort_label}" );
  eprintln!( "[trace] account.use  {name}  subprocess: skipped (reason: already active)" );
}

// Fix(BUG-207): `apply_post_switch_touch` had no `trace` param — model/effort resolution
//   and subprocess spawn were invisible; only the missing [trace] lines in `pre_switch_touch_ctx`
//   were apparent; both functions required the same fix.
// Root cause: Same as `pre_switch_touch_ctx` — Feature 027 "Out of Scope" deferral.
// Pitfall: When a function is split across pre/post phases, both halves need the same diagnostic
//   param — adding trace:: to one without the other leaves half the operation blind.
pub( crate ) fn apply_post_switch_touch(
  name       : &str,
  ctx        : TouchCtx,
  imodel_str : &str,
  effort_str : &str,
  trace      : bool,
  paths      : &crate::ClaudePaths,
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
  let mut args = match effort_val
  {
    Some( e ) => vec![ "--effort".to_string(), e.to_string() ],
    None      => vec![],
  };
  args.push( "--print".to_string() );
  args.push( ".".to_string() );
  let _ = claude_runner_core::run_isolated( &ctx.credentials_json, args, 120, model );
  if trace { eprintln!( "[trace] account.use  {name}  subprocess: spawned" ) }
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

  if params.live == 1
  {
    return execute_live_mode( &credential_store, &live_creds_file, &params );
  }

  let mut accounts = fetch_all_quota( &credential_store, &live_creds_file, false, params.trace )?;

  // Retry-once per account on 401/403 auth errors or 429+locally-expired: if
  // refresh::1 and any account's quota fetch failed with an auth error OR a
  // rate-limit response while its local `expiresAt` is past, refresh that token
  // via an isolated subprocess, then re-fetch only that account's quota.
  // Pure 429 with a non-expired local token is not retried — the token is valid.
  if params.refresh == 1
  {
    let claude_paths = crate::ClaudePaths::new();
    apply_refresh( &mut accounts, &credential_store, claude_paths.as_ref(), params.trace, params.imodel, params.effort );
  }

  // touch::1: activate idle 5h windows — runs after refresh so post-refresh results
  // are touched (an account that was refreshed and now has valid quota with no resets_at
  // will be touched; an account that still errors after refresh is skipped by apply_touch).
  if params.touch == 1
  {
    let claude_paths = crate::ClaudePaths::new();
    for aq in &mut accounts
    {
      apply_touch( aq, &credential_store, claude_paths.as_ref(), params.trace, params.imodel, params.effort );
    }
  }

  // ── Session-model override (BUG-244: .usage path, AC-32) ──────────────────
  // Must run BEFORE row-filter pipeline — filters can remove is_current from slice.
  {
    let claude_paths = crate::ClaudePaths::new();
    if let Some( ref claude_paths ) = claude_paths
    {
      if let Some( current ) = accounts.iter().find( |aq| aq.is_current )
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

    // only_next: retain only the account that received the → marker.
    if params.only_next
    {
      let best_opt = find_next_for_strategy( &accounts, params.next, params.prefer, now_secs );
      accounts = match best_opt
      {
        Some( i ) => { let w = accounts.swap_remove( i ); vec![ w ] }
        None      => vec![],
      };
    }

    // Boolean row filters.
    if params.only_active       { accounts.retain( |aq| aq.is_active ); }
    if params.only_valid        { accounts.retain( |aq| aq.result.is_ok() ); }
    if params.exclude_exhausted { accounts.retain( |aq| status_emoji( &aq.result ) == "🟢" ); }

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

  let content = match params.format
  {
    UsageOutputFormat::Json  => render_json( &accounts ),
    UsageOutputFormat::Tsv   => render_tsv( &accounts, params.sort, params.desc, params.prefer, &params.cols ),
    UsageOutputFormat::Plain => render_plain( &accounts, params.sort, params.desc, params.prefer, params.next, &params.cols ),
    UsageOutputFormat::Value => String::new(),
    UsageOutputFormat::Text  => render_text( &accounts, params.sort, params.desc, params.prefer, params.next, &params.cols ),
  };

  let content = if params.no_color && params.format != UsageOutputFormat::Tsv
  {
    apply_no_color( content )
  }
  else
  {
    content
  };

  Ok( OutputData::new( content, "text" ) )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
// Exception to tests-in-tests/ rule: pub(crate) fns (pre_switch_touch_ctx, apply_model_override)
// are not accessible from the external tests/ directory — unit tests must live here.
mod tests
{
  use super::{ pre_switch_touch_ctx, apply_model_override, PreSwitchOutcome };
  use tempfile::TempDir;

  /// Structural test: `pre_switch_touch_ctx` with an invalid credential file (no accessToken).
  ///
  /// # Root Cause (BUG-210)
  /// `pre_switch_touch_ctx()` returned `None` in the already-active branch without emitting
  /// `model:` + `effort:` trace lines. Model/effort resolution lived in `apply_post_switch_touch`
  /// only, which is never called on the skip path.
  ///
  /// # Why Not Caught
  /// No test called `pre_switch_touch_ctx` directly with a failing credential read; the only
  /// coverage was via the live `aw29` integration test (`lim_it`, excluded from CI).
  ///
  /// # Fix Applied
  /// Extended `pre_switch_touch_ctx` to accept `imodel_str` and `effort_str` parameters;
  /// added model/effort resolution and trace emission in the already-active branch.
  ///
  /// # Prevention
  /// This test acts as a compile-time guard: if the function reverts to 3 params, this
  /// call (with 5 args) causes a compile error. Also verifies that the fetch-failed path does
  /// NOT emit `model:` lines — the model/effort emission belongs only to quota-fetch-OK paths.
  ///
  /// # Pitfall
  /// The fetch-failed path must NOT emit `model:` even after the BUG-210 fix — model/effort
  /// resolution requires quota data, which is unavailable when fetch fails.
  #[ doc = "bug_reproducer(BUG-210)" ]
  #[ test ]
  fn test_pre_switch_touch_ctx_model_effort_absent_on_fetch_failure()
  {
    let store = TempDir::new().unwrap();

    // Write a credential file with no accessToken — quota fetch will fail.
    std::fs::write(
      store.path().join( "noaccess@example.com.credentials.json" ),
      r#"{"subscriptionType":"pro"}"#,
    ).unwrap();

    // 5-arg signature — compile error before BUG-210 fix extends the function.
    let result = pre_switch_touch_ctx(
      "noaccess@example.com",
      store.path(),
      true,
      "auto",
      "auto",
    );

    // Fetch-failed path must return Unavailable (credential read OK but accessToken absent).
    assert!(
      matches!( result, PreSwitchOutcome::Unavailable ),
      "fetch-failed path must return Unavailable, got {result:?}",
    );
  }

  /// `mre_bug238` — `apply_model_override()` writes opus when 7d(Son) consumed > 80%.
  ///
  /// # Root Cause
  /// `account_use_routine()` matched `PreSwitchOutcome::AlreadyActive` but skipped
  /// `apply_model_override()` — the model override never fired for already-active accounts.
  /// The `AlreadyActive` branch only emitted trace and returned; the extract was not wired in.
  ///
  /// # Why Not Caught
  /// No unit test for `apply_model_override()` existed. Full CLI path requires a live OAuth
  /// token to fetch quota, which is impractical in CI.
  ///
  /// # Fix Applied
  /// `AlreadyActive { quota }` branch in `account_use_routine()` now calls
  /// `apply_model_override(&quota, &paths, trace, &name)` before the trace line.
  ///
  /// # Prevention
  /// Any new `PreSwitchOutcome` match arm that receives `OauthUsageData` must call
  /// `apply_model_override()`. Test the helper directly to stay independent of CLI plumbing.
  ///
  /// # Pitfall
  /// `apply_model_override` only fires when `sonnet_left < 20.0`. Test input must push
  /// `utilization` to ≥ 80.0 (leaving < 20%). Also: the `.claude/` parent dir must exist
  /// before the write or `override_session_model_to_opus` silently no-ops.
  #[ doc = "bug_reproducer(BUG-238)" ]
  #[ test ]
  fn mre_bug238_model_override_fires_for_active_account()
  {
    use claude_quota::{ OauthUsageData, PeriodUsage };
    let dir   = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( dir.path() );
    // Create ~/.claude/ so override_session_model_to_opus can write settings.json.
    std::fs::create_dir_all( paths.base() ).unwrap();

    let quota = OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
    };
    apply_model_override( &quota, &paths, false, "account.use", "test-account" );

    let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
    assert!(
      content.contains( "claude-opus-4-6" ),
      "apply_model_override must write opus to settings.json when 7d(Son) is 90% consumed (10% left), got: {content}",
    );
  }

  // ── BUG-244 tests: label param + usage_routine wiring ─────────────────────

  #[ test ]
  fn t01_model_override_fires_when_sonnet_below_threshold()
  {
    use claude_quota::{ OauthUsageData, PeriodUsage };
    let dir   = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( dir.path() );
    std::fs::create_dir_all( paths.base() ).unwrap();
    let quota = OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
    };
    apply_model_override( &quota, &paths, false, "usage", "test-account" );
    let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
    assert!(
      content.contains( "claude-opus-4-6" ),
      "must write opus when 7d(Son) utilization=90% (10% left), got: {content}",
    );
  }

  #[ test ]
  fn t02_model_override_skips_when_sonnet_above_threshold()
  {
    use claude_quota::{ OauthUsageData, PeriodUsage };
    let dir   = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( dir.path() );
    std::fs::create_dir_all( paths.base() ).unwrap();
    let quota = OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( PeriodUsage { utilization : 70.0, resets_at : None } ),
    };
    apply_model_override( &quota, &paths, false, "usage", "test-account" );
    assert!(
      !paths.settings_file().exists(),
      "must NOT write settings.json when 7d(Son) utilization=70% (30% left)",
    );
  }

  #[ test ]
  fn t03_model_override_skips_when_already_opus()
  {
    use claude_quota::{ OauthUsageData, PeriodUsage };
    let dir   = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( dir.path() );
    std::fs::create_dir_all( paths.base() ).unwrap();
    // Pre-write settings.json with opus already set.
    std::fs::write( paths.settings_file(), r#"{"model":"claude-opus-4-6"}"# ).unwrap();
    let quota = OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
    };
    apply_model_override( &quota, &paths, false, "usage", "test-account" );
    let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
    assert!(
      content.contains( "claude-opus-4-6" ),
      "settings.json must still contain opus after call when already opus, got: {content}",
    );
    assert!(
      !content.contains( "claude-sonnet" ),
      "must not downgrade to sonnet, got: {content}",
    );
  }

  #[ test ]
  fn t04_model_override_skips_on_error_result()
  {
    // Guard test: usage_routine must guard apply_model_override with result.is_ok(),
    // since OauthUsageData is unavailable for Err accounts.
    let src      = include_str!( "api.rs" );
    let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
    let call_rel = src[ fn_start.. ]
      .find( "apply_model_override(" )
      .expect( "BUG-244: apply_model_override must be called in usage_routine" );
    // before_call: from fn_start to call site — ASCII boundaries, no multibyte risk.
    let before_call = &src[ fn_start .. fn_start + call_rel ];
    assert!(
      before_call.contains( "is_ok()" ) || before_call.contains( "Ok( ref " ),
      "apply_model_override call in usage_routine must be guarded by result.is_ok()",
    );
  }

  #[ test ]
  fn t05_apply_model_override_absent_from_usage_routine_before_fix()
  {
    let src      = include_str!( "api.rs" );
    let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
    let body_end = {
      let after = &src[ fn_start + 1.. ];
      let a     = after.find( "\npub fn " ).unwrap_or( after.len() );
      let b     = after.find( "\n#[ cfg( t" ).unwrap_or( after.len() );
      let c     = after.find( "\n#[cfg(t" ).unwrap_or( after.len() );
      fn_start + 1 + a.min( b ).min( c )
    };
    let body       = &src[ fn_start..body_end ];
    let call_count = body.matches( "apply_model_override(" ).count();
    assert_eq!(
      call_count, 1,
      "apply_model_override must be called exactly once in usage_routine body",
    );
  }

  #[ test ]
  fn t06_model_override_skips_for_non_current_account()
  {
    // Guard test: usage_routine must only call apply_model_override for is_current accounts.
    let src      = include_str!( "api.rs" );
    let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
    let call_rel = src[ fn_start.. ]
      .find( "apply_model_override(" )
      .expect( "BUG-244: apply_model_override must be called in usage_routine" );
    // before_call: from fn_start to call site — ASCII boundaries, no multibyte risk.
    let before_call = &src[ fn_start .. fn_start + call_rel ];
    assert!(
      before_call.contains( "is_current" ),
      "apply_model_override call in usage_routine must be guarded by is_current check",
    );
  }

  #[ test ]
  fn t07_model_override_skips_at_exact_20pct_boundary()
  {
    use claude_quota::{ OauthUsageData, PeriodUsage };
    let dir   = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( dir.path() );
    std::fs::create_dir_all( paths.base() ).unwrap();
    // utilization=80.0 → sonnet_left = 100.0 - 80.0 = 20.0; 20.0 < 20.0 == false.
    let quota = OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( PeriodUsage { utilization : 80.0, resets_at : None } ),
    };
    apply_model_override( &quota, &paths, false, "usage", "test-account" );
    assert!(
      !paths.settings_file().exists(),
      "must NOT write at exact 20% boundary (utilization=80.0)",
    );
  }

  #[ test ]
  fn t08_model_override_trace_label_is_usage()
  {
    use claude_quota::{ OauthUsageData, PeriodUsage };
    use std::io::Read;
    let dir   = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( dir.path() );
    std::fs::create_dir_all( paths.base() ).unwrap();
    let quota = OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
    };
    let mut buf = gag::BufferRedirect::stderr().unwrap();
    apply_model_override( &quota, &paths, true, "usage", "test-account" );
    let mut output = String::new();
    buf.read_to_string( &mut output ).unwrap();
    assert!(
      output.contains( "[trace] usage" ),
      "trace output must contain '[trace] usage' when label='usage', got: {output}",
    );
    assert!(
      !output.contains( "[trace] account.use" ),
      "trace output must NOT contain '[trace] account.use' when label='usage', got: {output}",
    );
  }

  /// # MRE: BUG-244 — `usage_routine` never called `apply_model_override`
  ///
  /// ## Root Cause
  /// `usage_routine()` in `src/usage/api.rs` had no call to `apply_model_override()`.
  /// The function existed and worked (called from `.account.use` path) but was never
  /// invoked from the `.usage` command path.
  ///
  /// ## Why Not Caught
  /// No test exercised the full `usage_routine()` → `apply_model_override()` wiring.
  /// Unit tests for `apply_model_override` passed trivially (calling it directly).
  ///
  /// ## Fix Applied
  /// Added `apply_model_override( data, claude_paths, params.trace, "usage", &current.name )`
  /// in `usage_routine()` after the touch loop, before the row-filter pipeline,
  /// guarded by `aq.is_current && aq.result.is_ok()`.
  /// Also added `label: &str` parameter to `apply_model_override` (was hardcoded "account.use").
  ///
  /// ## Prevention
  /// T05 structural test: grep of `usage_routine` body must contain exactly one call.
  ///
  /// ## Pitfall
  /// Insert the call BEFORE the row-filter pipeline (`only_next`/`only_active`/etc.) — those
  /// filters can remove the `is_current` entry from the slice, causing a silent no-op.
  #[ doc = "bug_reproducer(BUG-244)" ]
  #[ test ]
  fn mre_bug244_usage_routine_never_calls_apply_model_override()
  {
    let src      = include_str!( "api.rs" );
    let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
    let body_end = {
      let after = &src[ fn_start + 1.. ];
      let a     = after.find( "\npub fn " ).unwrap_or( after.len() );
      let b     = after.find( "\n#[ cfg( t" ).unwrap_or( after.len() );
      let c     = after.find( "\n#[cfg(t" ).unwrap_or( after.len() );
      fn_start + 1 + a.min( b ).min( c )
    };
    let body = &src[ fn_start..body_end ];
    assert!(
      body.contains( "apply_model_override(" ),
      "BUG-244: apply_model_override must be called from usage_routine — was absent before fix",
    );
  }
}
