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
  name          : String,
  active        : bool,
  expires_at_ms : u64,
  /// `Ok` = live headers fetched; `Err` = reason string (expired, network, etc.).
  result        : Result< RateLimitData, String >,
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
    results.push( AccountQuota
    {
      name          : acct.name.clone(),
      active        : acct.is_active,
      expires_at_ms : acct.expires_at_ms,
      result,
    } );
  }

  Ok( results )
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Compute the `Expires` cell value for a given token expiry and current time.
///
/// Returns `"EXPIRED"` when `expires_at_ms / 1000 ≤ now_secs` (saturating), or
/// `"in Xh Ym"` when the token is still valid.
fn compute_expires_cell( expires_at_ms : u64, now_secs : u64 ) -> String
{
  let remaining = ( expires_at_ms / 1000 ).saturating_sub( now_secs );
  if remaining == 0
  {
    "EXPIRED".to_string()
  }
  else
  {
    format!( "in {}", format_duration_secs( remaining ) )
  }
}

/// Shorten verbose quota error strings for display in the Status column.
///
/// `QuotaError::MissingHeader` (displays as `"rate-limit header missing: …"`) is
/// shortened to `"no header"`. All other strings pass through unchanged.
/// The caller is responsible for wrapping the result in parentheses.
fn shorten_error( reason : &str ) -> &str
{
  if reason.starts_with( "rate-limit header missing:" )
  {
    "no header"
  }
  else
  {
    reason
  }
}

/// Find the index of the recommended next account in an already-sorted slice.
///
/// Selects the non-active account with the highest `5h Left` among those with
/// valid quota data and a non-expired token (`expires_in_secs > 0`). Ties are
/// broken alphabetically — the first (alphabetically) account with equal `5h Left`
/// wins because the input is already alpha-sorted and strict-greater comparison
/// is used.
fn find_recommendation( accounts : &[ AccountQuota ], now_secs : u64 ) -> Option< usize >
{
  let mut best_idx    : Option< usize > = None;
  let mut best_5h_left : f64            = -1.0;

  for ( idx, aq ) in accounts.iter().enumerate()
  {
    if aq.active { continue; }
    let expires_in_secs = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
    if expires_in_secs == 0 { continue; }
    if let Ok( data ) = &aq.result
    {
      let left = ( 1.0 - data.utilization_5h ) * 100.0;
      if left > best_5h_left
      {
        best_5h_left = left;
        best_idx     = Some( idx );
      }
    }
  }

  best_idx
}

// ── Output renderers ───────────────────────────────────────────────────────────

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// When ≥2 accounts have valid quota data and a recommendation exists, appends
/// a footer line: `Valid: X / Y   →  Next: name  (N% session left, token expires in Xh Ym)`.
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

  let best_idx = find_recommendation( accounts, now_secs );

  let headers = vec![
    String::new(),
    "Account".to_string(),
    "Expires".to_string(),
    "5h Left".to_string(),
    "5h Reset".to_string(),
    "7d Left".to_string(),
    "7d Reset".to_string(),
    "Status".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for ( idx, aq ) in accounts.iter().enumerate()
  {
    let flag_cell = if aq.active
    {
      "✓".to_string()
    }
    else if best_idx == Some( idx )
    {
      "→".to_string()
    }
    else
    {
      String::new()
    };

    let expires_cell = compute_expires_cell( aq.expires_at_ms, now_secs );

    match &aq.result
    {
      Ok( data ) =>
      {
        builder = builder.add_row( vec![
          flag_cell.into(),
          aq.name.clone().into(),
          expires_cell.into(),
          format!( "{:.0}%", ( 1.0 - data.utilization_5h ) * 100.0 ).into(),
          format!( "in {}", format_duration_secs( data.reset_5h.saturating_sub( now_secs ) ) ).into(),
          format!( "{:.0}%", ( 1.0 - data.utilization_7d ) * 100.0 ).into(),
          format!( "in {}", format_duration_secs( data.reset_7d.saturating_sub( now_secs ) ) ).into(),
          data.status.clone().into(),
        ] );
      }
      Err( reason ) =>
      {
        let dash = "\u{2014}".to_string();
        builder = builder.add_row( vec![
          flag_cell.into(),
          aq.name.clone().into(),
          expires_cell.into(),
          dash.clone().into(),
          dash.clone().into(),
          dash.clone().into(),
          dash.into(),
          format!( "({})", shorten_error( reason ) ).into(),
        ] );
      }
    }
  }

  let view  = builder.build_view();
  let table = Format::format( &TableFormatter::new(), &view ).unwrap_or_default();
  let body  = format!( "Quota\n\n{table}\n" );

  // Footer: shown when ≥2 valid accounts and a recommendation exists.
  let valid_count = accounts.iter().filter( |aq| aq.result.is_ok() ).count();
  if valid_count >= 2
  {
    if let Some( idx ) = best_idx
    {
      let rec = &accounts[ idx ];
      if let Ok( data ) = &rec.result
      {
        let expires_in_secs = ( rec.expires_at_ms / 1000 ).saturating_sub( now_secs );
        let expires_str     = format_duration_secs( expires_in_secs );
        let footer = format!(
          "Valid: {} / {}   →  Next: {}  ({:.0}% session left, token expires in {})\n",
          valid_count,
          accounts.len(),
          rec.name,
          ( 1.0 - data.utilization_5h ) * 100.0,
          expires_str,
        );
        return format!( "{body}{footer}" );
      }
    }
  }

  body
}

/// Render quota results as a JSON array (one object per account).
///
/// Every row includes `expires_in_secs`. Successful accounts include quota
/// fields using `_left_pct` naming (remaining, not consumed); failed accounts
/// include `error`.
fn render_json( accounts : &[ AccountQuota ] ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty()
  {
    return "[]\n".to_string();
  }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  let mut parts = Vec::with_capacity( accounts.len() );
  for aq in accounts
  {
    let name_esc        = json_escape( &aq.name );
    let active_str      = if aq.active { "true" } else { "false" };
    let expires_in_secs = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
    let entry = match &aq.result
    {
      Ok( data ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"active\":{active_str},\"expires_in_secs\":{expires_in_secs},\
\"session_5h_left_pct\":{:.0},\"session_5h_resets_in_secs\":{},\
\"weekly_7d_left_pct\":{:.0},\"weekly_7d_resets_in_secs\":{},\"status\":\"{}\"}}",
          ( 1.0 - data.utilization_5h ) * 100.0,
          data.reset_5h.saturating_sub( now_secs ),
          ( 1.0 - data.utilization_7d ) * 100.0,
          data.reset_7d.saturating_sub( now_secs ),
          json_escape( &data.status ),
        )
      }
      Err( reason ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"active\":{active_str},\"expires_in_secs\":{expires_in_secs},\"error\":\"{}\"}}",
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
