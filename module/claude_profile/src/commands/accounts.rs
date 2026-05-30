//! `.accounts` command handler and account list renderers.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ OutputFormat, OutputOptions, json_escape, format_duration_secs };
use super::shared::{ require_credential_store, io_err_to_error_data, resolve_account_name, caps_to_json };

// ── Single-consumer helpers ───────────────────────────────────────────────────

/// Detect which saved account matches the live session token.
///
/// Reads `accessToken` from `live_creds_path` (graceful degradation: returns `None`
/// on any I/O or parse error). Compares by string equality against each saved account's
/// stored `accessToken` in `credential_store`; returns `Some(name)` on the first match,
/// `None` if no match.
fn detect_current_account(
  accounts         : &[ crate::account::Account ],
  live_creds_path  : &std::path::Path,
  credential_store : &std::path::Path,
) -> Option< String >
{
  let content    = std::fs::read_to_string( live_creds_path ).ok()?;
  let live_token = crate::account::parse_string_field( &content, "accessToken" )?;
  for acct in accounts
  {
    let path    = credential_store.join( format!( "{}.credentials.json", acct.name ) );
    let Ok( s ) = std::fs::read_to_string( &path ) else { continue };
    if let Some( token ) = crate::account::parse_string_field( &s, "accessToken" )
    {
      if token == live_token
      {
        return Some( acct.name.clone() );
      }
    }
  }
  None
}

/// Render an account list in text format with field-presence control.
///
/// Returns `"(no accounts configured)\n"` when `accounts` is empty.
/// When any field flag is `true`, each account block is followed by its fields
/// and separated from the next account by a blank line.
// Conditional rendering for 18 optional account fields; extraction into a helper
// would require passing all booleans again — no readability gain.
#[ allow( clippy::fn_params_excessive_bools, clippy::too_many_arguments, clippy::too_many_lines ) ]
#[ inline ]
fn render_accounts_text(
  accounts          : &[ &crate::account::Account ],
  show_active       : bool,
  show_current      : bool,
  current_name      : Option< &str >,
  show_sub          : bool,
  show_tier         : bool,
  show_expires      : bool,
  show_email        : bool,
  show_display_name : bool,
  show_host         : bool,
  show_role         : bool,
  show_billing      : bool,
  show_model        : bool,
  show_uuid         : bool,
  show_capabilities : bool,
  show_org_uuid     : bool,
  show_org_name     : bool,
) -> String
{
  if accounts.is_empty() { return "(no accounts configured)\n".to_string(); }
  // show_current is false when current::0 or when creds file is unreadable (current_name=None).
  let emit_current = show_current && current_name.is_some();
  let any_field = show_active || emit_current || show_sub || show_tier || show_expires || show_email
    || show_display_name || show_host || show_role || show_billing || show_model || show_uuid
    || show_capabilities || show_org_uuid || show_org_name;
  let mut out   = String::new();
  let last_idx  = accounts.len() - 1;
  for ( idx, a ) in accounts.iter().enumerate()
  {
    out.push_str( &a.name );
    out.push( '\n' );
    if any_field
    {
      if show_active
      {
        let active_str = if a.is_active { "yes" } else { "no" };
        let _ = writeln!( out, "  Active:  {active_str}" );
      }
      if emit_current
      {
        let current_str = if current_name == Some( a.name.as_str() ) { "yes" } else { "no" };
        let _ = writeln!( out, "  Current: {current_str}" );
      }
      if show_sub
      {
        let sub = if a.subscription_type.is_empty() { "N/A" } else { &a.subscription_type };
        let _ = writeln!( out, "  Sub:     {sub}" );
      }
      if show_tier
      {
        let tier = if a.rate_limit_tier.is_empty() { "N/A" } else { &a.rate_limit_tier };
        let _ = writeln!( out, "  Tier:    {tier}" );
      }
      if show_expires
      {
        let ts  = claude_profile_core::token::classify_ms( a.expires_at_ms, crate::token::WARNING_THRESHOLD_SECS );
        let exp = match &ts
        {
          crate::token::TokenStatus::Valid { expires_in }
          | crate::token::TokenStatus::ExpiringSoon { expires_in } =>
          {
            let h = expires_in.as_secs() / 3600;
            let m = ( expires_in.as_secs() % 3600 ) / 60;
            format!( "in {h}h {m}m" )
          }
          crate::token::TokenStatus::Expired => "expired".to_string(),
        };
        let _ = writeln!( out, "  Expires: {exp}" );
      }
      if show_email
      {
        let email = if a.email.is_empty() { "N/A" } else { &a.email };
        let _ = writeln!( out, "  Email:   {email}" );
      }
      if show_display_name
      {
        let dn = if a.display_name.is_empty() { "N/A" } else { &a.display_name };
        let _ = writeln!( out, "  Display: {dn}" );
      }
      if show_host
      {
        let host = if a.profile_host.is_empty() { "N/A" } else { &a.profile_host };
        let _ = writeln!( out, "  Host:    {host}" );
      }
      if show_role
      {
        let role = if a.profile_role.is_empty() { "N/A" } else { &a.profile_role };
        let _ = writeln!( out, "  Role:    {role}" );
      }
      if show_billing
      {
        let billing = if a.billing.is_empty() { "N/A" } else { &a.billing };
        let _ = writeln!( out, "  Billing: {billing}" );
      }
      if show_model
      {
        let model = if a.model.is_empty() { "N/A" } else { &a.model };
        let _ = writeln!( out, "  Model:   {model}" );
      }
      if show_uuid
      {
        let id_val = if a.tagged_id.is_empty() { "N/A" } else { &a.tagged_id };
        let _ = writeln!( out, "  ID:      {id_val}" );
      }
      if show_capabilities
      {
        let cap_val = if a.capabilities.is_empty()
        {
          "N/A".to_string()
        }
        else
        {
          a.capabilities.join( ", " )
        };
        let _ = writeln!( out, "  Capabilities: {cap_val}" );
      }
      if show_org_uuid
      {
        let val = if a.organization_uuid.is_empty() { "N/A" } else { &a.organization_uuid };
        let _ = writeln!( out, "  Org ID:  {val}" );
      }
      if show_org_name
      {
        let val = if a.organization_name.is_empty() { "N/A" } else { &a.organization_name };
        let _ = writeln!( out, "  Org:     {val}" );
      }
      if idx < last_idx { out.push( '\n' ); }
    }
  }
  out
}

/// Render a slice of accounts as a JSON array string.
fn render_accounts_json( accounts : &[ &crate::account::Account ], current_name : Option< &str > ) -> String
{
  if accounts.is_empty() { return "[]\n".to_string(); }
  let entries : Vec< String > = accounts.iter().map( |a|
  {
    let is_current = current_name == Some( a.name.as_str() );
    format!(
      "{{\"name\":\"{}\",\"is_active\":{},\"is_current\":{},\"subscription_type\":\"{}\",\
       \"rate_limit_tier\":\"{}\",\"expires_at_ms\":{},\"email\":\"{}\",\
       \"display_name\":\"{}\",\"role\":\"{}\",\"billing\":\"{}\",\"model\":\"{}\",\
       \"tagged_id\":\"{}\",\"capabilities\":{},\
       \"organization_uuid\":\"{}\",\"organization_name\":\"{}\",\
       \"organization_role\":\"{}\",\"workspace_uuid\":\"{}\",\"workspace_name\":\"{}\",\
       \"profile_host\":\"{}\",\"profile_role\":\"{}\"}}",
      json_escape( &a.name ),
      a.is_active,
      is_current,
      json_escape( &a.subscription_type ),
      json_escape( &a.rate_limit_tier ),
      a.expires_at_ms,
      json_escape( &a.email ),
      json_escape( &a.display_name ),
      json_escape( &a.role ),
      json_escape( &a.billing ),
      json_escape( &a.model ),
      json_escape( &a.tagged_id ),
      caps_to_json( &a.capabilities ),
      json_escape( &a.organization_uuid ),
      json_escape( &a.organization_name ),
      json_escape( &a.organization_role ),
      json_escape( &a.workspace_uuid ),
      json_escape( &a.workspace_name ),
      json_escape( &a.profile_host ),
      json_escape( &a.profile_role ),
    )
  } ).collect();
  format!( "[{}]\n", entries.join( "," ) )
}

/// Render a slice of accounts as a `data_fmt` ASCII table.
///
/// Columns: flag (active/current marker), Account, Active, Sub, Tier, Expires.
/// `current_name` is matched against account names to populate the flag column;
/// `✓` = current, `*` = active-but-not-current, blank otherwise.
fn render_accounts_table(
  accounts     : &[ &crate::account::Account ],
  current_name : Option< &str >,
) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty() { return "(no accounts configured)\n".to_string(); }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  let headers = vec![
    String::new(),
    "Account".to_string(),
    "Active".to_string(),
    "Sub".to_string(),
    "Tier".to_string(),
    "Expires".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for acct in accounts
  {
    let is_current = current_name == Some( acct.name.as_str() );
    let flag_cell  = if is_current { "✓".to_string() }
      else if acct.is_active { "*".to_string() }
      else { String::new() };

    let remaining    = ( acct.expires_at_ms / 1000 ).saturating_sub( now_secs );
    let expires_cell = if remaining == 0
    {
      "EXPIRED".to_string()
    }
    else
    {
      format!( "in {}", format_duration_secs( remaining ) )
    };

    builder = builder.add_row( vec![
      flag_cell.into(),
      acct.name.clone().into(),
      if acct.is_active { "yes" } else { "no" }.into(),
      acct.subscription_type.clone().into(),
      acct.rate_limit_tier.clone().into(),
      expires_cell.into(),
    ] );
  }

  let view  = builder.build_view();
  Format::format( &TableFormatter::new(), &view ).unwrap_or_default()
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.accounts` — list all saved accounts with field-presence control.
///
/// Without `name::`: lists every account in the credential store as an indented
/// key-value block, with a blank line between accounts when any field is shown.
/// With `name::EMAIL`: shows that single account's block only.
/// Field-presence params (`active`, `sub`, `tier`, `expires`, `email`) are all default-on.
/// `format::json` always includes all fields regardless of presence params.
///
/// # Errors
///
/// Returns `ErrorData` if `name::` is invalid (exit 1),
/// the named account is not found (exit 2), or the credential store is unreadable.
///
/// Storage unavailable (HOME/PRO unset) returns advisory "(no accounts configured)"
/// with exit 0 — same graceful behavior as an empty credential store.
// Fix(issue-accounts-home-unset):
// Root cause: require_credential_store()?; propagated Err (exit 2) when HOME and PRO are
//   both unset. .accounts is a graceful-read command; storage unavailability means the same
//   thing as an empty store — show advisory, not an error.
// Pitfall: require_credential_store() failing is NOT the same as list() returning [] —
//   they are different code paths. The graceful fallback must be at require_credential_store()
//   level, not at list() level.
#[ inline ]
pub fn accounts_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts             = OutputOptions::from_cmd( &cmd )?;
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let Ok( credential_store ) = require_credential_store() else
  {
    if trace { eprintln!( "[trace] accounts  credential store: not found" ) }
    let content = match opts.format
    {
      OutputFormat::Json  => "[]\n".to_string(),
      OutputFormat::Text
      | OutputFormat::Table => "(no accounts configured)\n".to_string(),
    };
    return Ok( OutputData::new( content, "text" ) );
  };
  if trace { eprintln!( "[trace] accounts  reading store: {}", credential_store.display() ) }

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };
  let name_arg = if raw_name.is_empty()
  {
    raw_name
  }
  else
  {
    resolve_account_name( &raw_name, &credential_store )?
  };

  let all_accounts = crate::account::list( &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "accounts" ) )?;

  let accounts : Vec< _ > = if name_arg.is_empty()
  {
    all_accounts.iter().collect()
  }
  else
  {
    let found : Vec< _ > = all_accounts.iter().filter( |a| a.name == name_arg ).collect();
    if found.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "account '{name_arg}' not found" ),
      ) );
    }
    found
  };

  let show_active       = matches!( cmd.arguments.get( "active"       ), Some( Value::Boolean( true ) ) | None );
  let show_current      = matches!( cmd.arguments.get( "current"      ), Some( Value::Boolean( true ) ) | None );
  let show_sub          = matches!( cmd.arguments.get( "sub"          ), Some( Value::Boolean( true ) ) | None );
  let show_tier         = matches!( cmd.arguments.get( "tier"         ), Some( Value::Boolean( true ) ) | None );
  let show_expires      = matches!( cmd.arguments.get( "expires"      ), Some( Value::Boolean( true ) ) | None );
  let show_email        = matches!( cmd.arguments.get( "email"        ), Some( Value::Boolean( true ) ) | None );
  let show_display_name = matches!( cmd.arguments.get( "display_name" ), Some( Value::Boolean( true ) ) );
  let show_host         = crate::output::parse_int_flag( &cmd, "host",         0 )? != 0;
  let show_role         = matches!( cmd.arguments.get( "role"         ), Some( Value::Boolean( true ) ) );
  let show_billing      = matches!( cmd.arguments.get( "billing"      ), Some( Value::Boolean( true ) ) );
  let show_model        = matches!( cmd.arguments.get( "model"        ), Some( Value::Boolean( true ) ) );
  let show_uuid         = crate::output::parse_int_flag( &cmd, "uuid",         0 )? != 0;
  let show_capabilities = crate::output::parse_int_flag( &cmd, "capabilities", 0 )? != 0;
  let show_org_uuid     = crate::output::parse_int_flag( &cmd, "org_uuid",     0 )? != 0;
  let show_org_name     = crate::output::parse_int_flag( &cmd, "org_name",     0 )? != 0;

  // Detect which account matches the live session token (graceful: None when creds absent).
  let live_creds = crate::ClaudePaths::new()
    .map_or_else( || std::path::PathBuf::from( "/dev/null" ), |p| p.credentials_file() );
  let current_name = detect_current_account( &all_accounts, &live_creds, &credential_store );

  let content = match opts.format
  {
    OutputFormat::Json => render_accounts_json( &accounts, current_name.as_deref() ),
    OutputFormat::Text =>
    {
      render_accounts_text(
        &accounts,
        show_active, show_current, current_name.as_deref(),
        show_sub, show_tier, show_expires, show_email,
        show_display_name, show_host, show_role, show_billing, show_model,
        show_uuid, show_capabilities,
        show_org_uuid, show_org_name,
      )
    }
    OutputFormat::Table =>
    {
      render_accounts_table( &accounts, current_name.as_deref() )
    }
  };
  Ok( OutputData::new( content, "text" ) )
}
