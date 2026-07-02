//! `.account.inspect` command handler and helpers.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::cmd_context::{ require_claude_paths, require_credential_store };
use super::cmd_args::{ io_err_to_error_data, resolve_account_name };
use super::account_inspect_render::{
  inspect_derive_status, extract_access_token,
  build_inspect_snapshot, format_inspect_text, format_inspect_json,
};
use claude_profile_core::account::trace_ts;

/// Resolve the account name for `.account.inspect`.
///
/// Uses `name::` if provided; falls back to the per-machine active marker file.
fn resolve_inspect_name( cmd : &VerifiedCommand, store : &std::path::Path ) -> Result< String, ErrorData >
{
  match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if s.is_empty() =>
      Err( ErrorData::new( ErrorCode::ArgumentMissing, "name:: value cannot be empty".to_string() ) ),
    Some( Value::String( s ) ) => resolve_account_name( s, store ),
    _ =>
    {
      let marker = store.join( crate::account::active_marker_filename() );
      std::fs::read_to_string( &marker )
        .ok()
        .map( | s | s.trim().to_string() )
        .filter( | s | !s.is_empty() )
        .ok_or_else( || ErrorData::new(
          ErrorCode::InternalError,
          "name:: omitted and no active account — pass name:: explicitly".to_string(),
        ) )
    }
  }
}

/// Call endpoint 002 (account) with trace logging.
fn inspect_call_account(
  tok   : &str,
  trace : bool,
  name  : &str,
) -> Result< claude_quota::OauthAccountData, claude_quota::QuotaError >
{
  if tok.is_empty()
  {
    return Err( claude_quota::QuotaError::HttpTransport( "no token".to_string() ) );
  }
  if trace { eprintln!( "{}account.inspect  {name}  GET /api/oauth/account", trace_ts() ) }
  let r = claude_quota::fetch_oauth_account( tok );
  if trace
  {
    match &r
    {
      Ok( a )  => eprintln!( "{}account.inspect  {name}  account OK  tagged_id={}", trace_ts(), a.tagged_id ),
      Err( e ) => eprintln!( "{}account.inspect  {name}  account ERR  {e}", trace_ts() ),
    }
  }
  r
}

/// Call endpoint 001 (usage) with trace logging.
fn inspect_call_usage(
  tok   : &str,
  trace : bool,
  name  : &str,
) -> Result< claude_quota::OauthUsageData, claude_quota::QuotaError >
{
  if tok.is_empty()
  {
    return Err( claude_quota::QuotaError::HttpTransport( "no token".to_string() ) );
  }
  if trace { eprintln!( "{}account.inspect  {name}  GET /api/oauth/usage", trace_ts() ) }
  let r = claude_quota::fetch_oauth_usage( tok );
  if trace
  {
    match &r
    {
      Ok( _ )  => eprintln!( "{}account.inspect  {name}  usage OK", trace_ts() ),
      Err( e ) => eprintln!( "{}account.inspect  {name}  usage ERR  {e}", trace_ts() ),
    }
  }
  r
}

/// Call endpoint 005 (roles) with trace logging.
fn inspect_call_roles(
  tok   : &str,
  trace : bool,
  name  : &str,
) -> Result< claude_quota::ClaudeCliRolesData, claude_quota::QuotaError >
{
  if tok.is_empty()
  {
    return Err( claude_quota::QuotaError::HttpTransport( "no token".to_string() ) );
  }
  if trace { eprintln!( "{}account.inspect  {name}  GET roles", trace_ts() ) }
  let r = claude_quota::fetch_claude_cli_roles( tok );
  if trace
  {
    match &r
    {
      Ok( rd ) => eprintln!( "{}account.inspect  {name}  roles OK  org={}", trace_ts(), rd.organization_name ),
      Err( e ) => eprintln!( "{}account.inspect  {name}  roles ERR  {e}", trace_ts() ),
    }
  }
  r
}

/// `.account.inspect` — show identity, subscription, org, and quota for one account via live endpoints.
///
/// Calls endpoints 002 (account), 005 (roles), and 001 (usage) independently.
/// Falls back to local `{name}.json` snapshot per-endpoint on failure.
/// Quota (endpoint 001) falls back to the local quota cache on transient errors (e.g. 429);
/// the section is omitted only when both the live call and the cache are unavailable.
///
/// # Errors
///
/// - Exit 1: invalid `format::` value.
/// - Exit 2: credential store absent; account not found; credential file absent.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn account_inspect_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace   = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let refresh = crate::output::parse_int_flag( &cmd, "refresh", 1 )?;
  let format  = match cmd.arguments.get( "format" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => "text".to_string(),
  };
  if !matches!( format.as_str(), "text" | "json" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "format:: must be `text` or `json`, got `{format}`" ),
    ) );
  }

  let credential_store = require_credential_store()?;
  let name             = resolve_inspect_name( &cmd, &credential_store )?;
  let cred_path        = credential_store.join( format!( "{name}.credentials.json" ) );

  if !cred_path.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "credential file not found: {}", cred_path.display() ),
    ) );
  }

  let cred_str = std::fs::read_to_string( &cred_path )
    .map_err( |e| io_err_to_error_data( &e, "account inspect" ) )?;
  let ( tok_label, status_bare, expires_in_secs, is_expired ) = inspect_derive_status( &cred_str );
  if trace { eprintln!( "{}account.inspect  {name}  status: {tok_label}", trace_ts() ) }

  let mut live_token = extract_access_token( &cred_str );

  if is_expired && refresh != 0
  {
    if trace { eprintln!( "{}account.inspect  {name}  token expired → attempting refresh", trace_ts() ) }
    let paths     = require_claude_paths()?;
    let refreshed = crate::usage::attempt_expired_token_refresh(
      &name, &credential_store, &paths, trace, "auto", "auto",
    );
    if refreshed
    {
      if trace { eprintln!( "{}account.inspect  {name}  refresh OK — re-reading token", trace_ts() ) }
      live_token = std::fs::read_to_string( &cred_path ).ok()
        .and_then( | s | extract_access_token( &s ) );
    }
    else if trace
    {
      eprintln!( "{}account.inspect  {name}  refresh failed — proceeding with stale token", trace_ts() );
    }
  }

  let tok        = live_token.as_deref().unwrap_or( "" );
  let ep_account = inspect_call_account( tok, trace, &name );
  let ep_roles   = inspect_call_roles( tok, trace, &name );
  let ep_usage   =
  {
    let live = inspect_call_usage( tok, trace, &name );
    // Cache fallback for transient errors (429, network failure, etc.).
    // Auth failures (401, 403) indicate the token is bad — don't serve stale data.
    if live.is_err() && !live.as_ref().unwrap_err().to_string().contains( "HTTP 401" )
                     && !live.as_ref().unwrap_err().to_string().contains( "HTTP 403" )
    {
      let now_secs = std::time::SystemTime::now()
        .duration_since( std::time::UNIX_EPOCH )
        .map_or( 0, |d| d.as_secs() );
      if let Some( ( data, age ) ) = crate::usage::read_cached_quota( &credential_store, &name, now_secs )
      {
        if trace { eprintln!( "{}account.inspect  {name}  usage cache fallback ({age}s old)", trace_ts() ) }
        Ok( data )
      }
      else { live }
    }
    else { live }
  };

  let meta_json = std::fs::read_to_string( credential_store.join( format!( "{name}.json" ) ) )
    .unwrap_or_default();
  let snap      = build_inspect_snapshot( &meta_json, &meta_json );

  let live_count  = [ ep_account.is_ok(), ep_roles.is_ok(), ep_usage.is_ok() ]
    .iter().filter( |&&b| b ).count();
  let data_source = match live_count { 3 => "live", 0 => "snapshot", _ => "partial_snapshot" };

  let output = if format == "json"
  {
    format_inspect_json(
      &name, status_bare, expires_in_secs,
      &ep_account, &ep_roles, &ep_usage, &snap, data_source,
    )
  }
  else
  {
    format_inspect_text( &name, &tok_label, &ep_account, &ep_roles, &ep_usage, &snap )
  };

  Ok( OutputData::new( output, &format ) )
}
