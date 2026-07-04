//! `.models` command handler — list available Claude models.
//!
//! Offline mode (`offline::1`) returns the static [`claude_quota::STATIC_MODELS`]
//! catalog without any network call. Live mode (default) fetches the authoritative
//! list from the Anthropic `/v1/models` endpoint using the active account token.
//!
//! Supports `name::` substring filter and `format::table|text|json` output.

use core::fmt::Write;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::output::{ OutputFormat, OutputOptions };
use super::cmd_context::require_credential_store;
use super::account_inspect_render::extract_access_token;
use super::cmd_args::io_err_to_error_data;

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.models` — list available Claude API models.
///
/// **Offline mode** (`offline::1`): returns [`claude_quota::STATIC_MODELS`] with no network call.
/// **Live mode** (default): fetches from `GET /v1/models` using the active account token.
///
/// `name::VALUE` filters by case-insensitive substring match on the model ID.
/// Zero matches is not an error — exits 0 with empty output.
///
/// `format::table` (default) — human-readable table. `format::text` — one ID per line.
/// `format::json` — JSON array.
///
/// # Errors
///
/// Returns `Err(ErrorData)` with `InternalError` if the active account token cannot be loaded
/// (live mode only).
#[ inline ]
pub fn models_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts    = OutputOptions::from_cmd( &cmd )?;
  let offline = matches!( cmd.arguments.get( "offline" ), Some( Value::Integer( 1 ) ) );
  let name_filter = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.to_ascii_lowercase() ),
    _                                            => None,
  };

  // Fetch model list
  let all_models : Vec< claude_quota::ModelInfo > = if offline
  {
    claude_quota::STATIC_MODELS.to_vec()
  }
  else
  {
    let token = fetch_active_token()?;
    claude_quota::fetch_models( &token )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?
  };

  // Apply name filter
  let models : Vec< &claude_quota::ModelInfo > = all_models.iter()
    .filter( | m |
    {
      match &name_filter
      {
        Some( f ) => m.id.to_ascii_lowercase().contains( f.as_str() ),
        None      => true,
      }
    } )
    .collect();

  let output = match opts.format
  {
    OutputFormat::Json  => render_json( &models ),
    OutputFormat::Text  => render_text( &models ),
    OutputFormat::Table => render_table( &models ),
  };

  Ok( OutputData::new( output, "text" ) )
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Get the OAuth access token for the active account.
fn fetch_active_token() -> Result< String, ErrorData >
{
  let store  = require_credential_store()?;
  let marker = store.join( crate::account::active_marker_filename() );
  let name   = std::fs::read_to_string( &marker )
    .ok()
    .map( | s | s.trim().to_string() )
    .filter( | s | !s.is_empty() )
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "no active account — use offline::1 or set an active account with .account.use".to_string(),
    ) )?;
  let cred_path = store.join( format!( "{name}.credentials.json" ) );
  let cred_str  = std::fs::read_to_string( &cred_path )
    .map_err( | e | io_err_to_error_data( &e, ".models" ) )?;
  extract_access_token( &cred_str )
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "active account has no valid access token — refresh the account first".to_string(),
    ) )
}

/// Render models as a human-readable table.
///
/// Columns: `ID | Display Name | Context | Max Out | Ext Think`
fn render_table( models : &[ &claude_quota::ModelInfo ] ) -> String
{
  const ID_MIN : usize = 2;  // "ID"
  const DN_MIN : usize = 12; // "Display Name"

  let id_w = models.iter().map( | m | m.id.len() ).max()
    .unwrap_or( ID_MIN ).max( ID_MIN );
  let dn_w = models.iter().map( | m | m.display_name.len() ).max()
    .unwrap_or( DN_MIN ).max( DN_MIN );

  let mut out = String::new();

  // Header
  writeln!( out, "{:<id_w$} | {:<dn_w$} | Context | Max Out | Ext Think", "ID", "Display Name" ).unwrap();
  // Separator
  writeln!( out, "{} + {} + ------- + ------- + ----------", "-".repeat( id_w ), "-".repeat( dn_w ) ).unwrap();

  for m in models
  {
    let ctx     = m.max_input_tokens.map_or_else( || "-".to_string(), fmt_k );
    let max_out = m.max_tokens.map_or_else( || "-".to_string(), fmt_k );
    let ext     = if m.capabilities.contains( &"extended-thinking" ) { "yes" } else { "no" };
    writeln!( out, "{:<id_w$} | {:<dn_w$} | {:<7} | {:<7} | {ext}", m.id, m.display_name, ctx, max_out ).unwrap();
  }
  out
}

/// Render models as one ID per line (`format::text`).
fn render_text( models : &[ &claude_quota::ModelInfo ] ) -> String
{
  let mut out = String::new();
  for m in models
  {
    out.push_str( m.id );
    out.push( '\n' );
  }
  out
}

/// Render models as a JSON array (`format::json`).
fn render_json( models : &[ &claude_quota::ModelInfo ] ) -> String
{
  let mut json = String::from( "[" );
  for ( i, m ) in models.iter().enumerate()
  {
    if i > 0 { json.push( ',' ); }
    let id_esc = escape_json_str( m.id );
    let dn_esc = escape_json_str( m.display_name );
    write!( json, "{{\"id\":\"{id_esc}\",\"display_name\":\"{dn_esc}\"" ).unwrap();
    match m.created_at
    {
      Some( ca ) =>
      {
        let ca_esc = escape_json_str( ca );
        write!( json, ",\"created_at\":\"{ca_esc}\"" ).unwrap();
      }
      None => json.push_str( ",\"created_at\":null" ),
    }
    match m.max_input_tokens
    {
      Some( n ) => write!( json, ",\"max_input_tokens\":{n}" ).unwrap(),
      None      => json.push_str( ",\"max_input_tokens\":null" ),
    }
    match m.max_tokens
    {
      Some( n ) => write!( json, ",\"max_tokens\":{n}" ).unwrap(),
      None      => json.push_str( ",\"max_tokens\":null" ),
    }
    json.push_str( ",\"capabilities\":[" );
    for ( j, cap ) in m.capabilities.iter().enumerate()
    {
      if j > 0 { json.push( ',' ); }
      let cap_esc = escape_json_str( cap );
      write!( json, "\"{cap_esc}\"" ).unwrap();
    }
    json.push_str( "]}" );
  }
  json.push_str( "]\n" );
  json
}

/// Escape a string for embedding in a JSON string value (`"` → `\"`, `\` → `\\`).
fn escape_json_str( s : &str ) -> String
{
  let mut out = String::with_capacity( s.len() );
  for ch in s.chars()
  {
    match ch
    {
      '"'  => out.push_str( "\\\"" ),
      '\\' => out.push_str( "\\\\" ),
      _    => out.push( ch ),
    }
  }
  out
}

/// Format a token count as a compact K string (e.g. `200_000` → "200K").
fn fmt_k( n : u64 ) -> String
{
  if n >= 1_000 { format!( "{}K", n / 1_000 ) } else { n.to_string() }
}
