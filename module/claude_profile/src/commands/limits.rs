//! `.account.limits` command handler.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use claude_quota::RateLimitData;
use crate::output::{ OutputFormat, OutputOptions, json_escape, format_duration_secs };
use super::cmd_args::{ io_err_to_error_data, resolve_account_name };
use super::cmd_context::{ require_claude_paths, require_credential_store };
use claude_profile_core::account::trace_ts;

// ── Single-consumer helpers ───────────────────────────────────────────────────

/// Verify the active-account credentials file exists.
///
/// Returns the path to `~/.claude/.credentials.json` if present, or `Err`
/// (exit 2) with an actionable error message if no active credentials are found.
fn require_active_credentials( paths : &crate::ClaudePaths ) -> Result< std::path::PathBuf, ErrorData >
{
  let creds = paths.credentials_file();
  if !creds.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      "no active account \u{2014} run `claude auth login` to authenticate".to_string(),
    ) );
  }
  Ok( creds )
}

/// Read the OAuth access token from a credentials file.
///
/// Searches for `accessToken` in the credential JSON using `parse_string_field`.
/// Works for both the active credentials file and saved named account files
/// because the field search is position-independent.
fn read_auth_token( creds_path : &std::path::Path ) -> Result< String, ErrorData >
{
  claude_profile_core::account::read_access_token_from_file( creds_path )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "credentials missing 'accessToken' \u{2014} re-authenticate with `claude auth login`: {e}" ),
    ) )
}

/// Format rate-limit data as human-readable text: labelled with reset durations.
fn format_rate_limits_text( data : &RateLimitData ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  let pct_session       = format!( "{:.0}", data.utilization_5h * 100.0 );
  let pct_weekly        = format!( "{:.0}", data.utilization_7d * 100.0 );
  let reset_session_str = format_duration_secs( data.reset_5h.saturating_sub( now_secs ) );
  let reset_weekly_str  = format_duration_secs( data.reset_7d.saturating_sub( now_secs ) );
  let status            = &data.status;
  format!( "Session (5h):  {pct_session}% consumed, resets in {reset_session_str}\nWeekly (7d):   {pct_weekly}% consumed, resets in {reset_weekly_str}\nStatus:        {status}\n" )
}

/// Format rate-limit data as a JSON object.
fn format_rate_limits_json( data : &RateLimitData ) -> String
{
  let pct_session  = format!( "{:.0}", data.utilization_5h * 100.0 );
  let pct_weekly   = format!( "{:.0}", data.utilization_7d * 100.0 );
  let ts_session   = data.reset_5h;
  let ts_weekly    = data.reset_7d;
  let status_esc   = json_escape( &data.status );
  format!( "{{\n  \"session_5h_pct\": {pct_session},\n  \"session_5h_reset_ts\": {ts_session},\n  \"weekly_7d_pct\": {pct_weekly},\n  \"weekly_7d_reset_ts\": {ts_weekly},\n  \"status\": \"{status_esc}\"\n}}\n" )
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.account.limits` — show rate-limit utilization for the selected account (FR-18).
///
/// Makes a lightweight `POST /v1/messages` to fetch `anthropic-ratelimit-unified-*`
/// response headers; outputs session (5h) and weekly (7d) utilization percentages.
///
/// # Errors
///
/// Returns `ErrorData` if:
/// - HOME is unset (exit 2)
/// - `name::` contains invalid characters (exit 1)
/// - Named account does not exist (exit 2)
/// - No active credentials are configured (exit 2)
/// - Credentials missing `accessToken` (exit 2)
/// - HTTP transport fails or rate-limit headers absent (exit 2)
#[ inline ]
pub fn account_limits_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts             = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "{}account.limits  store: {}", trace_ts(), credential_store.display() ) }

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };

  let creds_path = if raw_name.is_empty()
  {
    require_active_credentials( &paths )?
  }
  else
  {
    let name_arg = resolve_account_name( &raw_name, &credential_store )?;
    crate::account::validate_name( &name_arg )
      .map_err( | e | io_err_to_error_data( &e, "account limits" ) )?;
    let path = credential_store.join( format!( "{name_arg}.credentials.json" ) );
    if !path.exists()
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "account '{name_arg}' not found" ),
      ) );
    }
    path
  };

  let token = read_auth_token( &creds_path )?;
  let data  = claude_quota::fetch_rate_limits( &token )
    .map_err( |e| ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  let text = match opts.format
  {
    OutputFormat::Json  => format_rate_limits_json( &data ),
    OutputFormat::Text  => format_rate_limits_text( &data ),
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };
  Ok( OutputData::new( text, "text" ) )
}
