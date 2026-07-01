//! Main routine for the `.usage` command.
//!
//! Switch-context helpers live in `api_switch.rs`; mutation-dispatch helpers
//! live in `api_dispatch.rs`.
//! Exports: `PreSwitchOutcome`, `validate_imodel_str`, `validate_effort_str`,
//! `pre_switch_touch_ctx`, `apply_post_switch_touch`, `usage_routine`.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use super::types::UsageOutputFormat;
use super::fetch::fetch_quota_for_list;
use super::render::{ render_text, render_json, render_tsv, render_plain, extract_get_field };
use super::live::execute_live_mode;
use super::refresh::apply_refresh;
use super::touch::apply_touch;
use super::params::parse_usage_params;
use super::sort::find_next_for_strategy;
use super::format::{ five_hour_left, seven_day_left, status_emoji };
use super::api_switch::apply_model_override;
use super::api_dispatch::handle_mutation_dispatch;
pub use super::api_switch::{
  PreSwitchOutcome,
  validate_imodel_str, validate_effort_str,
  attempt_expired_token_refresh,
  pre_switch_touch_ctx,
  apply_post_switch_touch,
};


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

  // ── Mutation dispatch (Feature 037 + 064 + 065 — unified with .accounts) ────
  if let Some( output ) = handle_mutation_dispatch( &cmd, params.trace, params.force, &credential_store )?
  {
    return Ok( output );
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
  // effortLevel is model-derived and written by the model override above (TSK-335).
  // session_effort is used only for the Current line display; Next line uses model-derived effort.
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
    UsageOutputFormat::Plain => render_plain( &accounts, params.sort, params.desc, params.prefer, &params.cols, session_model, session_effort, Some( &credential_store ), params.who, params.rotate && !params.force ),
    UsageOutputFormat::Value => String::new(),
    UsageOutputFormat::Text  => render_text( &accounts, params.sort, params.desc, params.prefer, &params.cols, session_model, session_effort, Some( &credential_store ), params.who, params.rotate && !params.force ),
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
    use crate::commands::cmd_args::{ is_dry, io_err_to_error_data };

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

    // AC-05..AC-07: model + effort override for winner (Feature 062, unconditional — TSK-335).
    // Fix(carry-forward/TSK-335): removed `if let Some(se) = session_effort { set_session_effort(...) }`.
    // Root cause: carry-forward called after apply_model_override, overwriting its model-derived effort
    //   with the stale pre-rotation effortLevel from the previous account's settings.json.
    // Pitfall: apply_model_override now owns all effort writes (unconditional); callers must not
    //   call set_session_effort after it or the model-derived value will be clobbered.
    if let Ok( ref winner_data ) = accounts[ winner_idx ].result
    {
      apply_model_override( winner_data, &claude_paths, params.trace, "usage rotate", &winner_name );
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
