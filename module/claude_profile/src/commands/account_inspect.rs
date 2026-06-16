//! `.account.inspect` command handler and helpers.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::shared::{ require_claude_paths, require_credential_store, io_err_to_error_data, resolve_account_name, caps_to_json };
use crate::output::json_escape;

/// Snapshot data read from `{name}.json` for per-endpoint fallback.
struct InspectSnapshot
{
  tagged_id     : String,
  uuid          : String,
  email_address : String,
  full_name     : String,
  display_name  : String,
  billing_type  : String,
  has_max       : bool,
  org_name      : String,
  org_uuid      : String,
  org_role      : String,
  ws_uuid       : String,
  ws_name       : String,
}

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

/// Derive display label, bare status word, seconds-until-expiry, and expired flag from credentials JSON.
fn inspect_derive_status( cred_str : &str ) -> ( String, &'static str, u64, bool )
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_ms = u64::try_from(
    SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_millis()
  ).unwrap_or( u64::MAX );
  let needle = "\"expiresAt\":";
  let exp_ms = cred_str.find( needle ).and_then( | pos |
  {
    let rest = cred_str[ pos + needle.len().. ].trim_start();
    let end  = rest.find( | c : char | !c.is_ascii_digit() ).unwrap_or( rest.len() );
    rest[ ..end ].parse::< u64 >().ok()
  } );
  match exp_ms
  {
    None        => ( "unknown".to_string(), "unknown", 0, false ),
    Some( exp ) if now_ms <= exp =>
    {
      let rem_s = ( exp - now_ms ) / 1000;
      let h     = rem_s / 3600;
      let m     = ( rem_s % 3600 ) / 60;
      ( format!( "🟢 valid (expires in {h}h {m}m)" ), "valid", rem_s, false )
    }
    Some( exp ) =>
    {
      let ago_s = ( now_ms - exp ) / 1000;
      let h     = ago_s / 3600;
      let m     = ( ago_s % 3600 ) / 60;
      ( format!( "🔴 expired ({h}h {m}m ago)" ), "expired", 0, true )
    }
  }
}

/// Extract the raw `accessToken` string value from a credentials JSON string.
fn extract_access_token( cred_str : &str ) -> Option< String >
{
  let pos  = cred_str.find( "\"accessToken\":" )?;
  let rest = cred_str[ pos + "\"accessToken\":".len().. ].trim_start();
  if !rest.starts_with( '"' ) { return None; }
  let inner = &rest[ 1.. ];
  let end   = inner.find( '"' )?;
  Some( inner[ ..end ].to_string() )
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
  if trace { eprintln!( "[trace] account.inspect  {name}  GET /api/oauth/account" ) }
  let r = claude_quota::fetch_oauth_account( tok );
  if trace
  {
    match &r
    {
      Ok( a )  => eprintln!( "[trace] account.inspect  {name}  account OK  tagged_id={}", a.tagged_id ),
      Err( e ) => eprintln!( "[trace] account.inspect  {name}  account ERR  {e}" ),
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
  if trace { eprintln!( "[trace] account.inspect  {name}  GET /api/oauth/usage" ) }
  let r = claude_quota::fetch_oauth_usage( tok );
  if trace
  {
    match &r
    {
      Ok( _ )  => eprintln!( "[trace] account.inspect  {name}  usage OK" ),
      Err( e ) => eprintln!( "[trace] account.inspect  {name}  usage ERR  {e}" ),
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
  if trace { eprintln!( "[trace] account.inspect  {name}  GET roles" ) }
  let r = claude_quota::fetch_claude_cli_roles( tok );
  if trace
  {
    match &r
    {
      Ok( rd ) => eprintln!( "[trace] account.inspect  {name}  roles OK  org={}", rd.organization_name ),
      Err( e ) => eprintln!( "[trace] account.inspect  {name}  roles ERR  {e}" ),
    }
  }
  r
}

/// Build per-endpoint snapshot data from `{name}.json`.
fn build_inspect_snapshot( claude_json : &str, roles_json : &str ) -> InspectSnapshot
{
  let oa_pos        = claude_json.find( "\"oauthAccount\":" );
  let tagged_id     = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "taggedId" ) )
    .unwrap_or_default();
  let uuid          = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "uuid" ) )
    .unwrap_or_default();
  let email_address = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "emailAddress" ) )
    .unwrap_or_default();
  let full_name     = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "fullName" ) )
    .unwrap_or_default();
  let display_name  = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "displayName" ) )
    .unwrap_or_default();
  let billing_type  = oa_pos
    .and_then( | p | crate::account::parse_string_field( &claude_json[ p.. ], "billingType" ) )
    .unwrap_or_default();
  let has_max       = claude_json.contains( "\"claude_max\"" );
  let org_name      = crate::account::parse_string_field( roles_json, "organization_name" ).unwrap_or_default();
  let org_uuid      = crate::account::parse_string_field( roles_json, "organization_uuid" ).unwrap_or_default();
  let org_role      = crate::account::parse_string_field( roles_json, "organization_role" ).unwrap_or_default();
  let ws_uuid       = crate::account::parse_string_field( roles_json, "workspace_uuid" ).unwrap_or_default();
  let ws_name       = crate::account::parse_string_field( roles_json, "workspace_name" ).unwrap_or_default();
  InspectSnapshot { tagged_id, uuid, email_address, full_name, display_name, billing_type, has_max, org_name, org_uuid, org_role, ws_uuid, ws_name }
}

/// Format a utilization reset timestamp as a countdown string like `"resets in 1d 5h"`.
fn format_reset_countdown( resets_at : Option< &String > ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let Some( ts_str ) = resets_at else { return String::new() };
  let Some( reset_ts ) = claude_quota::iso_to_unix_secs( ts_str ) else { return String::new() };
  let now_s = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
  if reset_ts <= now_s { return ", resets now".to_string(); }
  let rem = reset_ts - now_s;
  let d = rem / 86_400;
  let h = ( rem % 86_400 ) / 3_600;
  let m = ( rem % 3_600 ) / 60;
  if d > 0      { format!( ", resets in {d}d {h}h" ) }
  else if h > 0 { format!( ", resets in {h}h {m}m" ) }
  else          { format!( ", resets in {m}m" ) }
}

/// Render `.account.inspect` output as human-readable text.
#[ allow( clippy::too_many_lines ) ]
#[ allow( clippy::format_push_string ) ]
fn format_inspect_text(
  name       : &str,
  tok_label  : &str,
  ep_account : &Result< claude_quota::OauthAccountData, claude_quota::QuotaError >,
  ep_roles   : &Result< claude_quota::ClaudeCliRolesData, claude_quota::QuotaError >,
  ep_usage   : &Result< claude_quota::OauthUsageData, claude_quota::QuotaError >,
  snap       : &InspectSnapshot,
) -> String
{
  let mut out = String::new();

  // Header: account name and token status.
  out.push_str( &format!( "{:<17}{name}\n", "Account:" ) );

  // Name and Email (endpoint 002 or snapshot fallback).
  if let Ok( a ) = ep_account
  {
    if !a.full_name.is_empty()
    {
      let name_display = if !a.display_name.is_empty() && a.display_name != a.full_name
      {
        format!( "{} ({})", a.full_name, a.display_name )
      }
      else { a.full_name.clone() };
      out.push_str( &format!( "{:<17}{name_display}\n", "Name:" ) );
    }
    if !a.email_address.is_empty()
    {
      out.push_str( &format!( "{:<17}{}\n", "Email:", a.email_address ) );
    }
  }
  else
  {
    if !snap.full_name.is_empty()
    {
      let name_display = if !snap.display_name.is_empty() && snap.display_name != snap.full_name
      {
        format!( "{} ({}) (snapshot)", snap.full_name, snap.display_name )
      }
      else { format!( "{} (snapshot)", snap.full_name ) };
      out.push_str( &format!( "{:<17}{name_display}\n", "Name:" ) );
    }
    if !snap.email_address.is_empty()
    {
      out.push_str( &format!( "{:<17}{} (snapshot)\n", "Email:", snap.email_address ) );
    }
  }

  out.push_str( &format!( "{:<17}{tok_label}\n", "Status:" ) );

  // Identity fields: tagged ID and UUID (endpoint 002 or snapshot fallback).
  let ( tagged_id_s, uuid_s ) = match ep_account
  {
    Ok( a )  => ( a.tagged_id.clone(), a.uuid.clone() ),
    Err( _ ) =>
    (
      if snap.tagged_id.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.tagged_id ) },
      if snap.uuid.is_empty()      { "N/A".to_string() } else { format!( "{} (snapshot)", snap.uuid ) },
    ),
  };
  out.push_str( &format!( "{:<17}{tagged_id_s}\n{:<17}{uuid_s}\n\n", "Tagged ID:", "UUID:" ) );

  // Memberships (endpoint 002).
  match ep_account
  {
    Ok( a ) =>
    {
      let ms = &a.memberships;
      out.push_str( &format!( "{:<17}{}\n", "Memberships:", ms.len() ) );
      let sel_idx    = if ms.is_empty() { 0 } else { claude_quota::select_membership_index( ms ) };
      let show_sel   = ms.len() > 1;
      let bt_width   = ms.iter().map( | m | m.billing_type.len() ).max().unwrap_or( 4 );
      for ( i, m ) in ms.iter().enumerate()
      {
        let caps_str = if m.capabilities.is_empty() { "[]".to_string() }
          else { format!( "[{}]", m.capabilities.join( ", " ) ) };
        let marker   = if show_sel && i == sel_idx { "  \u{2190} selected" } else { "" };
        out.push_str( &format!(
          "  [{i}]  billing_type={:<bt_width$}  has_max={:<5}  capabilities={caps_str}{marker}\n",
          m.billing_type,
          if m.has_max { "true" } else { "false" },
        ) );
      }
    }
    Err( e ) => out.push_str( &format!( "{:<17}endpoint unavailable ({e})\n", "Memberships:" ) ),
  }
  out.push( '\n' );

  // Billing, Has Max, Capabilities, Tier (from selected membership or snapshot).
  let ( billing_s, has_max_s ) = match ep_account
  {
    Ok( a ) => ( a.billing_type.clone(), if a.has_max { "yes" } else { "no" }.to_string() ),
    Err( _ ) =>
    (
      if snap.billing_type.is_empty() { "N/A".to_string() }
        else { format!( "{} (snapshot)", snap.billing_type ) },
      if snap.billing_type.is_empty() { "N/A".to_string() }
        else { format!( "{} (snapshot)", if snap.has_max { "yes" } else { "no" } ) },
    ),
  };
  out.push_str( &format!( "{:<17}{billing_s}\n{:<17}{has_max_s}\n", "Billing:", "Has Max:" ) );

  if let Ok( a ) = ep_account
  {
    let caps_str = if a.capabilities.is_empty() { "[]".to_string() }
      else { format!( "[{}]", a.capabilities.join( ", " ) ) };
    out.push_str( &format!( "{:<17}{caps_str}\n", "Capabilities:" ) );
    if !a.rate_limit_tier.is_empty()
    {
      out.push_str( &format!( "{:<17}{}\n", "Tier:", a.rate_limit_tier ) );
    }
  }
  out.push( '\n' );

  // Quota utilization (endpoint 001 — no snapshot fallback).
  if let Ok( u ) = ep_usage
  {
    if let Some( ref p ) = u.five_hour
    {
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      let pct = ( p.utilization * 100.0 ).round() as u64;
      let reset = format_reset_countdown( p.resets_at.as_ref() );
      out.push_str( &format!( "{:<17}{pct}% consumed{reset}\n", "Session (5h):" ) );
    }
    if let Some( ref p ) = u.seven_day
    {
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      let pct = ( p.utilization * 100.0 ).round() as u64;
      let reset = format_reset_countdown( p.resets_at.as_ref() );
      out.push_str( &format!( "{:<17}{pct}% consumed{reset}\n", "Weekly (7d):" ) );
    }
    if let Some( ref p ) = u.seven_day_sonnet
    {
      #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
      let pct = ( p.utilization * 100.0 ).round() as u64;
      let reset = format_reset_countdown( p.resets_at.as_ref() );
      out.push_str( &format!( "{:<17}{pct}% consumed{reset}\n", "Sonnet (7d):" ) );
    }
    out.push( '\n' );
  }

  // Org fields (endpoint 005 or snapshot fallback).
  let ( org_s, org_uuid_s, org_role_s, ws_uuid_s, ws_name_s ) = match ep_roles
  {
    Ok( r ) =>
    (
      r.organization_name.clone(),
      r.organization_uuid.clone(),
      r.organization_role.clone(),
      if r.workspace_uuid.is_empty() { "(none)".to_string() } else { r.workspace_uuid.clone() },
      if r.workspace_name.is_empty() { "(none)".to_string() } else { r.workspace_name.clone() },
    ),
    Err( _ ) =>
    (
      if snap.org_name.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.org_name ) },
      if snap.org_uuid.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.org_uuid ) },
      if snap.org_role.is_empty() { "N/A".to_string() } else { format!( "{} (snapshot)", snap.org_role ) },
      if snap.ws_uuid.is_empty()  { "(none)".to_string() } else { format!( "{} (snapshot)", snap.ws_uuid ) },
      if snap.ws_name.is_empty()  { "(none)".to_string() } else { format!( "{} (snapshot)", snap.ws_name ) },
    ),
  };
  out.push_str( &format!(
    "{:<17}{org_s}\n{:<17}{org_uuid_s}\n{:<17}{org_role_s}\n{:<17}{ws_uuid_s}\n{:<17}{ws_name_s}\n",
    "Org:", "Org UUID:", "Org Role:", "Workspace UUID:", "Workspace:",
  ) );

  out
}

/// Render `.account.inspect` output as compact JSON.
#[ allow( clippy::too_many_arguments, clippy::too_many_lines ) ]
fn format_inspect_json(
  name       : &str,
  status     : &str,
  expires_in : u64,
  ep_account : &Result< claude_quota::OauthAccountData, claude_quota::QuotaError >,
  ep_roles   : &Result< claude_quota::ClaudeCliRolesData, claude_quota::QuotaError >,
  ep_usage   : &Result< claude_quota::OauthUsageData, claude_quota::QuotaError >,
  snap       : &InspectSnapshot,
  data_source : &str,
) -> String
{
  let ( tagged_id, uuid, email_address, full_name, display_name ) = match ep_account
  {
    Ok( a )  => ( a.tagged_id.clone(), a.uuid.clone(), a.email_address.clone(),
                  a.full_name.clone(), a.display_name.clone() ),
    Err( _ ) => ( snap.tagged_id.clone(), snap.uuid.clone(), snap.email_address.clone(),
                  snap.full_name.clone(), snap.display_name.clone() ),
  };

  let sel_idx = ep_account.as_ref().ok()
    .filter( | a | !a.memberships.is_empty() )
    .map_or( 0, | a | claude_quota::select_membership_index( &a.memberships ) );

  let ms_json = match ep_account
  {
    Ok( a ) =>
    {
      let entries : Vec< String > = a.memberships.iter().enumerate().map( | ( i, m ) |
      {
        let caps_json = caps_to_json( &m.capabilities );
        format!(
          "{{\"index\":{i},\"billing_type\":\"{}\",\"has_max\":{},\"capabilities\":{caps_json},\"selected\":{}}}",
          json_escape( &m.billing_type ),
          if m.has_max { "true" } else { "false" },
          if i == sel_idx { "true" } else { "false" },
        )
      } ).collect();
      format!( "[{}]", entries.join( "," ) )
    }
    Err( _ ) => "[]".to_string(),
  };

  let ( billing_type, has_max, capabilities, rate_limit_tier ) = match ep_account
  {
    Ok( a ) => ( a.billing_type.clone(), a.has_max, a.capabilities.clone(), a.rate_limit_tier.clone() ),
    Err( _ ) => ( snap.billing_type.clone(), snap.has_max, vec![], String::new() ),
  };

  let caps_json = caps_to_json( &capabilities );

  let ( org_name, org_uuid, org_role, ws_uuid, ws_name ) = match ep_roles
  {
    Ok( r ) =>
      ( r.organization_name.clone(), r.organization_uuid.clone(), r.organization_role.clone(),
        r.workspace_uuid.clone(), r.workspace_name.clone() ),
    Err( _ ) =>
      ( snap.org_name.clone(), snap.org_uuid.clone(), snap.org_role.clone(),
        snap.ws_uuid.clone(), snap.ws_name.clone() ),
  };

  // Usage fields — default to 0 / empty when endpoint 001 unavailable.
  #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
  let ( s5h_pct, s5h_ts, w7d_pct, w7d_ts, son_pct, son_ts ) = match ep_usage
  {
    Ok( u ) =>
    {
      let pct = | p : &claude_quota::PeriodUsage | ( p.utilization * 100.0 ).round() as u64;
      let ts  = | p : &claude_quota::PeriodUsage |
        p.resets_at.as_deref().and_then( claude_quota::iso_to_unix_secs ).unwrap_or( 0 );
      let s5  = u.five_hour.as_ref().map_or( ( 0_u64, 0_u64 ), | p | ( pct( p ), ts( p ) ) );
      let w7  = u.seven_day.as_ref().map_or( ( 0, 0 ), | p | ( pct( p ), ts( p ) ) );
      let son = u.seven_day_sonnet.as_ref().map_or( ( 0, 0 ), | p | ( pct( p ), ts( p ) ) );
      ( s5.0, s5.1, w7.0, w7.1, son.0, son.1 )
    }
    Err( _ ) => ( 0, 0, 0, 0, 0, 0 ),
  };

  format!(
    "{{\
      \"account\":\"{}\",\
      \"status\":\"{status}\",\
      \"expires_in_secs\":{expires_in},\
      \"tagged_id\":\"{}\",\
      \"uuid\":\"{}\",\
      \"email_address\":\"{}\",\
      \"full_name\":\"{}\",\
      \"display_name\":\"{}\",\
      \"memberships\":{ms_json},\
      \"billing_type\":\"{}\",\
      \"has_max\":{},\
      \"capabilities\":{caps_json},\
      \"rate_limit_tier\":\"{}\",\
      \"session_5h_pct\":{s5h_pct},\
      \"session_5h_reset_ts\":{s5h_ts},\
      \"weekly_7d_pct\":{w7d_pct},\
      \"weekly_7d_reset_ts\":{w7d_ts},\
      \"sonnet_7d_pct\":{son_pct},\
      \"sonnet_7d_reset_ts\":{son_ts},\
      \"organization_name\":\"{}\",\
      \"organization_uuid\":\"{}\",\
      \"organization_role\":\"{}\",\
      \"workspace_uuid\":\"{}\",\
      \"workspace_name\":\"{}\",\
      \"data_source\":\"{data_source}\"\
    }}\n",
    json_escape( name ),
    json_escape( &tagged_id ),
    json_escape( &uuid ),
    json_escape( &email_address ),
    json_escape( &full_name ),
    json_escape( &display_name ),
    json_escape( &billing_type ),
    if has_max { "true" } else { "false" },
    json_escape( &rate_limit_tier ),
    json_escape( &org_name ),
    json_escape( &org_uuid ),
    json_escape( &org_role ),
    json_escape( &ws_uuid ),
    json_escape( &ws_name ),
  )
}

/// `.account.inspect` — show identity, subscription, org, and quota for one account via live endpoints.
///
/// Calls endpoints 002 (account), 005 (roles), and 001 (usage) independently.
/// Falls back to local `{name}.json` snapshot per-endpoint on failure.
/// Quota (endpoint 001) has no snapshot fallback — omitted when unavailable.
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
  if trace { eprintln!( "[trace] account.inspect  {name}  status: {tok_label}" ) }

  let mut live_token = extract_access_token( &cred_str );

  if is_expired && refresh != 0
  {
    if trace { eprintln!( "[trace] account.inspect  {name}  token expired → attempting refresh" ) }
    let paths     = require_claude_paths()?;
    let refreshed = crate::usage::attempt_expired_token_refresh(
      &name, &credential_store, &paths, trace, "auto", "auto",
    );
    if refreshed
    {
      if trace { eprintln!( "[trace] account.inspect  {name}  refresh OK — re-reading token" ) }
      live_token = std::fs::read_to_string( &cred_path ).ok()
        .and_then( | s | extract_access_token( &s ) );
    }
    else if trace
    {
      eprintln!( "[trace] account.inspect  {name}  refresh failed — proceeding with stale token" );
    }
  }

  let tok        = live_token.as_deref().unwrap_or( "" );
  let ep_account = inspect_call_account( tok, trace, &name );
  let ep_roles   = inspect_call_roles( tok, trace, &name );
  let ep_usage   = inspect_call_usage( tok, trace, &name );

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
