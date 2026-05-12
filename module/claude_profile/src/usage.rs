//! `.usage` command — all-accounts live quota table.
//!
//! Fetches live rate-limit utilization for every saved account via
//! `claude_quota::fetch_rate_limits()` and renders results as a `data_fmt` table.
//! Accounts are enumerated from the credential store in alphabetical order.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

use claude_quota::RateLimitData;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ OutputFormat, OutputOptions, format_duration_secs, json_escape };

// ── Per-account quota result ───────────────────────────────────────────────────

struct AccountQuota
{
  name   : String,
  active : bool,
  /// `Ok` = live headers fetched; `Err` = reason string (expired, network, etc.).
  result : Result< RateLimitData, String >,
}

// ── Fetch helpers ──────────────────────────────────────────────────────────────

/// Read the OAuth access token from a named account credentials file.
///
/// Returns `Err(reason)` on I/O failure or missing `accessToken` field;
/// the reason is stored inline per-account and does not abort the full fetch.
fn read_token( credential_store : &std::path::Path, name : &str ) -> Result< String, String >
{
  let path    = credential_store.join( format!( "{name}.credentials.json" ) );
  let content = std::fs::read_to_string( &path )
    .map_err( |e| format!( "cannot read credentials: {e}" ) )?;
  crate::account::parse_string_field( &content, "accessToken" )
    .ok_or_else( || "missing accessToken".to_string() )
}

/// Enumerate all saved accounts and fetch their live quota data.
///
/// Accounts are listed in alphabetical order (delegated to `account::list()`).
/// Per-account failures are stored inline in `AccountQuota::result`; only
/// fatal errors (credential store unreadable) propagate as `ErrorData`.
fn fetch_all_quota( credential_store : &std::path::Path ) -> Result< Vec< AccountQuota >, ErrorData >
{
  let accounts = crate::account::list( credential_store )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot read credential store: {e}" ),
    ) )?;

  let mut results = Vec::with_capacity( accounts.len() );
  for acct in &accounts
  {
    let result = match read_token( credential_store, &acct.name )
    {
      Ok( token ) => claude_quota::fetch_rate_limits( &token ).map_err( |e| e.to_string() ),
      Err( e )    => Err( e ),
    };
    results.push( AccountQuota { name : acct.name.clone(), active : acct.is_active, result } );
  }

  Ok( results )
}

// ── Output renderers ───────────────────────────────────────────────────────────

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
fn render_text( accounts : &[ AccountQuota ] ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty()
  {
    return "Quota\n\n  (no accounts configured)\n".to_string();
  }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  let headers = vec![
    "Account".to_string(),
    "Session (5h)".to_string(),
    "Weekly (7d)".to_string(),
    "Status".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for aq in accounts
  {
    let account_cell = if aq.active
    {
      format!( "{} (✓)", aq.name )
    }
    else
    {
      aq.name.clone()
    };

    let ( session_cell, weekly_cell, status_cell ) = match &aq.result
    {
      Ok( data ) =>
      {
        (
          format!(
            "{:.0}% / {}",
            data.utilization_5h * 100.0,
            format_duration_secs( data.reset_5h.saturating_sub( now_secs ) ),
          ),
          format!(
            "{:.0}% / {}",
            data.utilization_7d * 100.0,
            format_duration_secs( data.reset_7d.saturating_sub( now_secs ) ),
          ),
          data.status.clone(),
        )
      }
      Err( reason ) =>
      {
        ( "\u{2014}".to_string(), "\u{2014}".to_string(), format!( "({reason})" ) )
      }
    };

    builder = builder.add_row( vec![
      account_cell.into(),
      session_cell.into(),
      weekly_cell.into(),
      status_cell.into(),
    ] );
  }

  let view  = builder.build_view();
  let table = Format::format( &TableFormatter::new(), &view ).unwrap_or_default();
  format!( "Quota\n\n{table}\n" )
}

/// Render quota results as a JSON array (one object per account).
///
/// Successful accounts include quota fields; failed accounts include `error`.
fn render_json( accounts : &[ AccountQuota ] ) -> String
{
  if accounts.is_empty()
  {
    return "[]\n".to_string();
  }

  let mut parts = Vec::with_capacity( accounts.len() );
  for aq in accounts
  {
    let name_esc   = json_escape( &aq.name );
    let active_str = if aq.active { "true" } else { "false" };
    let entry = match &aq.result
    {
      Ok( data ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"active\":{active_str},\
\"session_5h_pct\":{:.0},\"session_5h_resets_in_secs\":{},\
\"weekly_7d_pct\":{:.0},\"weekly_7d_resets_in_secs\":{},\"status\":\"{}\"}}",
          data.utilization_5h * 100.0,
          data.reset_5h,
          data.utilization_7d * 100.0,
          data.reset_7d,
          json_escape( &data.status ),
        )
      }
      Err( reason ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"active\":{active_str},\"error\":\"{}\"}}",
          json_escape( reason ),
        )
      }
    };
    parts.push( entry );
  }

  format!( "[\n  {}\n]\n", parts.join( ",\n  " ) )
}

// ── Command handler ────────────────────────────────────────────────────────────

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
#[ inline ]
pub fn usage_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts             = OutputOptions::from_cmd( &cmd )?;
  let credential_store = crate::PersistPaths::new()
    .map( | p | p.credential_store() )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot resolve storage root: {e}" ),
    ) )?;

  let accounts = fetch_all_quota( &credential_store )?;
  let content  = match opts.format
  {
    OutputFormat::Json => render_json( &accounts ),
    OutputFormat::Text => render_text( &accounts ),
  };

  Ok( OutputData::new( content, "text" ) )
}
