//! Formatting helpers for `.account.inspect`.
//!
//! Owns the `InspectSnapshot` data type and all render functions.
//! Data-gathering and HTTP calls live in `account_inspect.rs`.

use crate::output::json_escape;
use super::shared::caps_to_json;

// ── Snapshot type ─────────────────────────────────────────────────────────────

/// Snapshot data read from `{name}.json` for per-endpoint fallback.
pub( crate ) struct InspectSnapshot
{
  pub( crate ) tagged_id     : String,
  pub( crate ) uuid          : String,
  pub( crate ) email_address : String,
  pub( crate ) full_name     : String,
  pub( crate ) display_name  : String,
  pub( crate ) billing_type  : String,
  pub( crate ) has_max       : bool,
  pub( crate ) org_name      : String,
  pub( crate ) org_uuid      : String,
  pub( crate ) org_role      : String,
  pub( crate ) ws_uuid       : String,
  pub( crate ) ws_name       : String,
}

// ── Token status ──────────────────────────────────────────────────────────────

/// Derive display label, bare status word, seconds-until-expiry, and expired flag from credentials JSON.
pub( crate ) fn inspect_derive_status( cred_str : &str ) -> ( String, &'static str, u64, bool )
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

// ── Token extraction ──────────────────────────────────────────────────────────

/// Extract the raw `accessToken` string value from a credentials JSON string.
pub( crate ) fn extract_access_token( cred_str : &str ) -> Option< String >
{
  let pos  = cred_str.find( "\"accessToken\":" )?;
  let rest = cred_str[ pos + "\"accessToken\":".len().. ].trim_start();
  if !rest.starts_with( '"' ) { return None; }
  let inner = &rest[ 1.. ];
  let end   = inner.find( '"' )?;
  Some( inner[ ..end ].to_string() )
}

// ── Snapshot builder ──────────────────────────────────────────────────────────

/// Build per-endpoint snapshot data from `{name}.json`.
pub( crate ) fn build_inspect_snapshot( claude_json : &str, roles_json : &str ) -> InspectSnapshot
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

// ── Reset countdown ───────────────────────────────────────────────────────────

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

// ── Text renderer ─────────────────────────────────────────────────────────────

/// Render `.account.inspect` output as human-readable text.
#[ allow( clippy::too_many_lines ) ]
#[ allow( clippy::format_push_string ) ]
pub( crate ) fn format_inspect_text(
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

// ── JSON renderer ─────────────────────────────────────────────────────────────

/// Render `.account.inspect` output as compact JSON.
#[ allow( clippy::too_many_arguments, clippy::too_many_lines ) ]
pub( crate ) fn format_inspect_json(
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
