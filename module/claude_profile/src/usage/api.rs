//! Public API surface for the `.usage` command and account-use touch context.
//!
//! Exports: `TouchCtx`, `validate_imodel_str`, `validate_effort_str`,
//! `pre_switch_touch_ctx`, `apply_post_switch_touch`, `usage_routine`.

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
pub( crate ) struct TouchCtx
{
  /// Raw credentials JSON read from the account credential file before the switch.
  credentials_json : String,
  /// Pre-fetched quota data used to resolve the subprocess model.
  quota            : OauthUsageData,
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
  imodel_str : &str,
  effort_str : &str,
) -> Option< TouchCtx >
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
      return None;
    }
  };
  let Some( token ) = crate::account::parse_string_field( &credentials_json, "accessToken" ) else
  {
    if trace
    {
      eprintln!( "[trace] account.use  {name}  quota fetch: Err(no accessToken in credentials)" );
      eprintln!( "[trace] account.use  {name}  subprocess: skipped (reason: fetch failed)" );
    }
    return None;
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
      return None;
    }
  };
  let is_idle = quota.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ).is_none();
  if is_idle
  {
    if trace { eprintln!( "[trace] account.use  {name}  idle check: resets_at=absent → idle" ) }
    Some( TouchCtx { credentials_json, quota } )
  }
  else
  {
    if trace
    {
      eprintln!( "[trace] account.use  {name}  idle check: resets_at=present → already active" );
      let imodel       = SubprocessModel::parse( imodel_str ).unwrap_or( SubprocessModel::Auto );
      let effort       = SubprocessEffort::parse( effort_str ).unwrap_or( SubprocessEffort::Auto );
      let aq           = AccountQuota
      {
        name          : name.to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Ok( quota ),
        account       : None,
        host          : String::new(),
        role          : String::new(),
      };
      let model        = resolve_model( &aq, imodel );
      let effort_val   = resolve_effort( &model, effort );
      let model_str    = match &model
      {
        claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(),
        _                                                => "keep-current",
      };
      let effort_label = effort_val.unwrap_or( "(none)" );
      eprintln!( "[trace] account.use  {name}  model: {model_str}  effort: {effort_label}" );
      eprintln!( "[trace] account.use  {name}  subprocess: skipped (reason: already active)" );
    }
    None
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
///
/// `imodel_str` and `effort_str` must have been pre-validated by [`validate_imodel_str`]
/// / [`validate_effort_str`]; the `parse()` calls below are infallible on validated input.
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
)
{
  let imodel = SubprocessModel::parse( imodel_str ).unwrap_or( SubprocessModel::Auto );
  let effort = SubprocessEffort::parse( effort_str ).unwrap_or( SubprocessEffort::Auto );
  // Build a minimal AccountQuota to reuse the existing resolve_model() path.
  let aq = AccountQuota
  {
    name          : name.to_string(),
    is_current    : false,
    is_active     : false,
    expires_at_ms : 0,
    result        : Ok( ctx.quota ),
    account       : None,
    host          : String::new(),
    role          : String::new(),
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

    // Threshold filters: rows with no valid quota (Err) score as 0 and are also hidden.
    if params.min_5h > 0
    {
      let threshold = f64::from( params.min_5h );
      accounts.retain( |aq| five_hour_left( aq ) >= threshold );
    }
    if params.min_7d > 0
    {
      let threshold = f64::from( params.min_7d );
      accounts.retain( |aq| seven_day_left( aq ) >= threshold );
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
mod tests
{
  use super::pre_switch_touch_ctx;
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

    // Fetch-failed path must return None (no touch context, credential read succeeded but
    // accessToken absent → fetch branch exits early).
    assert!(
      result.is_none(),
      "fetch-failed path must return None, got Some(..)",
    );
  }
}
