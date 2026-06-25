//! `.accounts` command handler and account list renderers.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ OutputFormat, OutputOptions, json_escape, format_duration_secs };
use super::shared::{ require_credential_store, io_err_to_error_data, resolve_account_name, caps_to_json };

// ── Column visibility ─────────────────────────────────────────────────────────

/// Column visibility set for `.accounts` text/table output.
///
/// Default set (default-on): account, owner, active, current, sub, tier, expires, email.
/// Opt-in: `display_name`, host, role, billing, model, uuid, capabilities, `org_uuid`, `org_name`.
///
/// Constructed via [`IdentityCols::default_set()`] or parsed from a `cols::` modifier string
/// (comma-separated `+col_id` / `-col_id` tokens) via [`IdentityCols::parse()`].
// IdentityCols is a pure column-visibility bitfield; all 17 flags are intentional.
#[ allow( clippy::struct_excessive_bools ) ]
#[ derive( Clone, Debug ) ]
struct IdentityCols
{
  account      : bool,
  owner        : bool,
  active       : bool,
  current      : bool,
  sub          : bool,
  tier         : bool,
  expires      : bool,
  email        : bool,
  display_name : bool,
  host         : bool,
  role         : bool,
  billing      : bool,
  model        : bool,
  uuid         : bool,
  capabilities : bool,
  org_uuid     : bool,
  org_name     : bool,
}

impl IdentityCols
{
  fn default_set() -> Self
  {
    Self
    {
      account      : true,
      owner        : true,
      active       : true,
      current      : true,
      sub          : true,
      tier         : true,
      expires      : true,
      email        : true,
      display_name : false,
      host         : false,
      role         : false,
      billing      : false,
      model        : false,
      uuid         : false,
      capabilities : false,
      org_uuid     : false,
      org_name     : false,
    }
  }

  /// Parse a `cols::` modifier string into an `IdentityCols`.
  ///
  /// Starts from [`default_set()`] and applies each `+col_id` / `-col_id` token.
  /// Returns `Err` on unknown col IDs or tokens missing `+`/`-` prefix.
  fn parse( s : &str ) -> Result< Self, unilang::data::ErrorData >
  {
    let mut cols = Self::default_set();
    for token in s.split( ',' ).map( str::trim ).filter( |t| !t.is_empty() )
    {
      let ( flag, name ) = if let Some( n ) = token.strip_prefix( '+' )
      {
        ( true, n )
      }
      else if let Some( n ) = token.strip_prefix( '-' )
      {
        ( false, n )
      }
      else
      {
        return Err( unilang::data::ErrorData::new(
          unilang::data::ErrorCode::ArgumentTypeMismatch,
          format!( "cols:: token '{token}' must start with '+' or '-'" ),
        ) );
      };
      match name
      {
        "account"      => cols.account      = flag,
        "owner"        => cols.owner        = flag,
        "active"       => cols.active       = flag,
        "current"      => cols.current      = flag,
        "sub"          => cols.sub          = flag,
        "tier"         => cols.tier         = flag,
        "expires"      => cols.expires      = flag,
        "email"        => cols.email        = flag,
        "display_name" => cols.display_name = flag,
        "host"         => cols.host         = flag,
        "role"         => cols.role         = flag,
        "billing"      => cols.billing      = flag,
        "model"        => cols.model        = flag,
        "uuid"         => cols.uuid         = flag,
        "capabilities" => cols.capabilities = flag,
        "org_uuid"     => cols.org_uuid     = flag,
        "org_name"     => cols.org_name     = flag,
        _ => return Err( unilang::data::ErrorData::new(
          unilang::data::ErrorCode::ArgumentTypeMismatch,
          format!( "unknown cols:: column id '{name}'; valid: account, owner, active, current, sub, tier, expires, email, display_name, host, role, billing, model, uuid, capabilities, org_uuid, org_name" ),
        ) ),
      }
    }
    Ok( cols )
  }
}

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

/// Render an account list in text format controlled by [`IdentityCols`].
///
/// Returns `"(no accounts configured)\n"` when `accounts` is empty.
/// When any field in `cols` is enabled, each account block is followed by its
/// field lines and separated from the next account by a blank line.
/// `owners` must be parallel to `accounts` (one owner string per account);
/// pass an empty slice when `cols.owner` is false.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
fn render_accounts_text(
  accounts     : &[ &crate::account::Account ],
  owners       : &[ String ],
  cols         : &IdentityCols,
  current_name : Option< &str >,
) -> String
{
  if accounts.is_empty() { return "(no accounts configured)\n".to_string(); }
  // emit_current is false when cols.current is false or when current_name is None.
  let emit_current = cols.current && current_name.is_some();
  let any_field = cols.owner || cols.active || emit_current || cols.sub || cols.tier
    || cols.expires || cols.email || cols.display_name || cols.host || cols.role
    || cols.billing || cols.model || cols.uuid || cols.capabilities || cols.org_uuid
    || cols.org_name;
  let mut out  = String::new();
  let last_idx = accounts.len() - 1;
  for ( idx, a ) in accounts.iter().enumerate()
  {
    out.push_str( &a.name );
    out.push( '\n' );
    if any_field
    {
      if cols.owner
      {
        let owner_raw = owners.get( idx ).map_or( "", String::as_str );
        let owner_val = if owner_raw.is_empty() { "\u{2014}" } else { owner_raw };
        let _ = writeln!( out, "  Owner:   {owner_val}" );
      }
      if cols.active
      {
        let active_str = if a.is_active { "yes" } else { "no" };
        let _ = writeln!( out, "  Active:  {active_str}" );
      }
      if emit_current
      {
        let current_str = if current_name == Some( a.name.as_str() ) { "yes" } else { "no" };
        let _ = writeln!( out, "  Current: {current_str}" );
      }
      if cols.sub
      {
        let sub = if a.subscription_type.is_empty() { "N/A" } else { &a.subscription_type };
        let _ = writeln!( out, "  Sub:     {sub}" );
      }
      if cols.tier
      {
        let tier = if a.rate_limit_tier.is_empty() { "N/A" } else { &a.rate_limit_tier };
        let _ = writeln!( out, "  Tier:    {tier}" );
      }
      if cols.expires
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
      if cols.email
      {
        let email = if a.email.is_empty() { "N/A" } else { &a.email };
        let _ = writeln!( out, "  Email:   {email}" );
      }
      if cols.display_name
      {
        let dn = if a.display_name.is_empty() { "N/A" } else { &a.display_name };
        let _ = writeln!( out, "  Display: {dn}" );
      }
      if cols.host
      {
        let host = if a.host.is_empty() { "N/A" } else { &a.host };
        let _ = writeln!( out, "  Host:    {host}" );
      }
      if cols.role
      {
        let role = if a.role.is_empty() { "N/A" } else { &a.role };
        let _ = writeln!( out, "  Role:    {role}" );
      }
      if cols.billing
      {
        let billing = if a.billing.is_empty() { "N/A" } else { &a.billing };
        let _ = writeln!( out, "  Billing: {billing}" );
      }
      if cols.model
      {
        let model = if a.model.is_empty() { "N/A" } else { &a.model };
        let _ = writeln!( out, "  Model:   {model}" );
      }
      if cols.uuid
      {
        let id_val = if a.tagged_id.is_empty() { "N/A" } else { &a.tagged_id };
        let _ = writeln!( out, "  ID:      {id_val}" );
      }
      if cols.capabilities
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
      if cols.org_uuid
      {
        let val = if a.organization_uuid.is_empty() { "N/A" } else { &a.organization_uuid };
        let _ = writeln!( out, "  Org ID:  {val}" );
      }
      if cols.org_name
      {
        let val = if a.organization_name.is_empty() { "N/A" } else { &a.organization_name };
        let _ = writeln!( out, "  Org:     {val}" );
      }
      if idx < last_idx { out.push( '\n' ); }
    }
  }
  out
}

/// Serialise an optional renewal timestamp as a JSON value (`null` or a quoted string).
fn renewal_at_json( v : Option< &str > ) -> String
{
  match v
  {
    None    => "null".to_string(),
    Some(s) => format!( "\"{}\"", json_escape( s ) ),
  }
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
       \"host\":\"{}\",\"owner\":\"{}\",\"is_owned\":{},\"renewal_at\":{}}}",
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
      json_escape( &a.org_role ),
      json_escape( &a.workspace_uuid ),
      json_escape( &a.workspace_name ),
      json_escape( &a.host ),
      json_escape( &a.owner ),
      a.is_owned,
      renewal_at_json( a.renewal_at.as_deref() ),
    )
  } ).collect();
  format!( "[{}]\n", entries.join( "," ) )
}

/// Render a slice of accounts as a `data_fmt` ASCII table.
///
/// Columns respect `cols`: flag (active/current marker), Account, Owner (when `cols.owner`),
/// Active (when `cols.active`), Sub, Tier, Expires. `current_name` populates the flag
/// column (`✓` = current, `*` = active-but-not-current, blank otherwise).
/// `owners` must be parallel to `accounts`; pass an empty slice when `cols.owner` is false.
fn render_accounts_table(
  accounts     : &[ &crate::account::Account ],
  owners       : &[ String ],
  cols         : &IdentityCols,
  current_name : Option< &str >,
) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty() { return "(no accounts configured)\n".to_string(); }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  let mut headers = vec![ String::new(), "Account".to_string() ];
  if cols.owner  { headers.push( "Owner".to_string()  ); }
  if cols.active { headers.push( "Active".to_string() ); }
  headers.push( "Sub".to_string() );
  headers.push( "Tier".to_string() );
  headers.push( "Expires".to_string() );

  let mut builder = RowBuilder::new( headers );
  for ( idx, acct ) in accounts.iter().enumerate()
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

    let mut row = vec![ flag_cell.into(), acct.name.clone().into() ];
    if cols.owner
    {
      let owner_raw = owners.get( idx ).map_or( "", String::as_str );
      let owner_val = if owner_raw.is_empty() { "\u{2014}".to_string() } else { owner_raw.to_string() };
      row.push( owner_val.into() );
    }
    if cols.active { row.push( if acct.is_active { "yes" } else { "no" }.into() ); }
    row.push( acct.subscription_type.clone().into() );
    row.push( acct.rate_limit_tier.clone().into() );
    row.push( expires_cell.into() );

    builder = builder.add_row( row );
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
// Fix(BUG-268):
// Root cause: require_credential_store()?; propagated Err (exit 2) when HOME and PRO are
//   both unset. .accounts is a graceful-read command; storage unavailability means the same
//   thing as an empty store — show advisory, not an error.
// Pitfall: require_credential_store() failing is NOT the same as list() returning [] —
//   they are different code paths. The graceful fallback must be at require_credential_store()
//   level, not at list() level.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
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
    raw_name.clone()
  }
  else if raw_name.contains( ',' )
  {
    // Comma-list for batch owner:: ops — defer per-component resolution to dispatch.
    raw_name.clone()
  }
  else
  {
    resolve_account_name( &raw_name, &credential_store )?
  };

  let all_accounts = crate::account::list( &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "accounts" ) )?;

  let accounts : Vec< _ > = if name_arg.is_empty() || name_arg.contains( ',' ) || cmd.arguments.contains_key( "active" )
  {
    // Comma-list and active:: dispatch handle their own account filtering/validation.
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

  // ── Mutation dispatch ──────────────────────────────────────────────────────
  use super::shared::is_dry;

  // REMOVED_TOGGLE checks: assign, unclaim, for → migration messages (Feature 064).
  if cmd.arguments.contains_key( "assign" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "assign:: REMOVED — use active::USER@MACHINE name::X instead".to_string(),
    ) );
  }
  if cmd.arguments.contains_key( "unclaim" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "unclaim:: REMOVED — use owner::0 name::X instead (or owner::0 alone to batch-clear)".to_string(),
    ) );
  }
  if cmd.arguments.contains_key( "for" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "for:: REMOVED — functionality absorbed into active:: value: active::USER@MACHINE name::X".to_string(),
    ) );
  }

  // ── active:: dispatch (Feature 064) ────────────────────────────────────────
  if let Some( Value::String( av ) ) = cmd.arguments.get( "active" )
  {
    let av  = av.clone();
    let san = | s : &str | -> String
    {
      s.chars().map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } ).collect()
    };
    let ( usr_raw, mch_raw ) = av.split_once( '@' ).ok_or_else( || ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "active:: must be USER@MACHINE format (no '@' found)".to_string(),
    ) )?;
    if usr_raw.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "active:: user component (left of '@') must not be empty".to_string(),
      ) );
    }
    if mch_raw.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "active:: machine component (right of '@') must not be empty".to_string(),
      ) );
    }
    let su      = san( usr_raw );
    let sm      = san( mch_raw );
    let marker  = format!( "_active_{sm}_{su}" );
    let display = format!( "{su}@{sm}" );
    if !name_arg.is_empty()
    {
      // Assign: write marker pointing to name_arg.
      let cred_path = credential_store.join( format!( "{name_arg}.credentials.json" ) );
      if !cred_path.exists()
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          format!( "account '{name_arg}' not found in credential store" ),
        ) );
      }
      if is_dry( &cmd )
      {
        return Ok( OutputData::new(
          format!( "[dry-run] would assign {name_arg} for {display}  \u{2192}  {marker}\n" ),
          "text",
        ) );
      }
      std::fs::write( credential_store.join( &marker ), name_arg.as_bytes() )
        .map_err( | e | io_err_to_error_data( &e, "accounts active" ) )?;
      if trace { eprintln!( "[trace] accounts active  write marker: {marker}  →  {name_arg}" ) }
      return Ok( OutputData::new(
        format!( "assigned {name_arg} for {display}  \u{2192}  {marker}\n" ),
        "text",
      ) );
    }
    // Unassign: clear the marker file.
    if is_dry( &cmd )
    {
      return Ok( OutputData::new(
        format!( "[dry-run] would unassign {display}  \u{2192}  {marker} cleared\n" ),
        "text",
      ) );
    }
    let marker_path = credential_store.join( &marker );
    if marker_path.exists()
    {
      std::fs::remove_file( &marker_path )
        .map_err( | e | io_err_to_error_data( &e, "accounts active unassign" ) )?;
    }
    if trace { eprintln!( "[trace] accounts active  cleared marker: {marker}" ) }
    return Ok( OutputData::new(
      format!( "unassigned {display}  \u{2192}  {marker} cleared\n" ),
      "text",
    ) );
  }

  // owner:: param — explicit ownership assignment (Feature 063 + 064).
  let owner_value = match cmd.arguments.get( "owner" )
  {
    Some( unilang::types::Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    Some( unilang::types::Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch,
        "owner:: value must be non-empty — use owner::0 to clear ownership".into() ) ),
    _ => None,
  };

  // ── owner:: explicit ownership assignment/release (Feature 063 + 064) ────────
  if let Some( ref ov ) = owner_value
  {
    let is_sentinel = ov.as_str() == "0";
    let force       = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
    let is_dry_run  = is_dry( &cmd );

    if raw_name.is_empty()
    {
      // No name:: → batch-clear (owner::0 only; owner::VALUE requires name::).
      if !is_sentinel
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          "owner::USER@MACHINE requires name:: to specify the target account".to_string(),
        ) );
      }
      // Batch-clear all accounts currently owned by this identity.
      // Unowned and foreign-owned accounts are skipped with a "skip" message (AC-09).
      let mut out = String::new();
      for acct in &all_accounts
      {
        let json_path = credential_store.join( format!( "{}.json", acct.name ) );
        // No metadata file → silently skip (no ownership info to act on).
        if !json_path.exists() { continue; }
        let acct_owner = crate::account::read_owner( &credential_store, &acct.name );
        if acct_owner.is_empty()
        {
          // Unowned — nothing to clear; skip with message (AC-09).
          writeln!( out, "skip {}", acct.name ).unwrap();
          continue;
        }
        if !force && !crate::account::is_owned( &acct_owner )
        {
          // Owned by another identity — skip with message (AC-09).
          if trace { eprintln!( "[trace] accounts owner  batch-skip (foreign owner): {}  owner={acct_owner}", acct.name ) }
          writeln!( out, "skip {}", acct.name ).unwrap();
          continue;
        }
        if is_dry_run
        {
          writeln!( out, "[dry-run] would clear owner of {}", acct.name ).unwrap();
          continue;
        }
        crate::account::write_owner( &acct.name, &credential_store, "" )
          .map_err( |e| io_err_to_error_data( &e, "accounts owner batch-clear" ) )?;
        if trace { eprintln!( "[trace] accounts owner  cleared: {}  was={acct_owner}", acct.name ) }
        writeln!( out, "unclaimed {}", acct.name ).unwrap();
      }
      return Ok( OutputData::new( out, "text" ) );
    }

    // name:: present — resolve each component (comma-list supported for owner:: ops).
    let target_names : Vec< String > = if raw_name.contains( ',' )
    {
      raw_name.split( ',' )
        .map( | part | resolve_account_name( part.trim(), &credential_store ) )
        .collect::< Result< Vec< _ >, _ > >()?
    }
    else
    {
      vec![ name_arg.clone() ]
    };

    let mut out = String::new();
    for name in &target_names
    {
      let json_path = credential_store.join( format!( "{name}.json" ) );
      if !json_path.exists()
      {
        return Err( ErrorData::new(
          ErrorCode::InternalError,
          format!( "account not found: {name}" ),
        ) );
      }
      // G8 ownership gate — evaluated per account, even in dry-run (AC-16/AC-17).
      let acct_owner = crate::account::read_owner( &credential_store, name );
      if !force && !crate::account::is_owned( &acct_owner )
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          format!( "ownership violation: {name} is owned by {acct_owner}" ),
        ) );
      }
      if is_dry_run
      {
        if is_sentinel
        {
          writeln!( out, "[dry-run] would clear owner of {name}" ).unwrap();
        }
        else
        {
          writeln!( out, "[dry-run] would set owner of {name} to {ov}" ).unwrap();
        }
        continue;
      }
      let new_owner = if is_sentinel { "" } else { ov.as_str() };
      crate::account::write_owner( name, &credential_store, new_owner )
        .map_err( |e| io_err_to_error_data( &e, "accounts owner" ) )?;
      if trace
      {
        eprintln!( "[trace] accounts owner  write_owner: OK  name={name} identity={}", if is_sentinel { "(cleared)" } else { ov } );
      }
      if is_sentinel
      {
        writeln!( out, "unclaimed {name}" ).unwrap();
      }
      else
      {
        writeln!( out, "owned {name} by {ov}" ).unwrap();
      }
    }
    return Ok( OutputData::new( out, "text" ) );
  }

  // ── Legacy field-toggle rejection (Feature 037 — removed; use cols:: instead) ─
  // Params remain registered so the framework routes to this routine; the routine
  // rejects them explicitly to provide a helpful migration message.
  const REMOVED_TOGGLES : &[ ( &str, &str ) ] = &[
    ( "current",      "-current" ),
    ( "sub",          "-sub" ),
    ( "tier",         "-tier" ),
    ( "expires",      "-expires" ),
    ( "email",        "-email" ),
    ( "display_name", "+display_name" ),
    ( "host",         "+host" ),
    ( "role",         "+role" ),
    ( "billing",      "+billing" ),
    ( "model",        "+model" ),
    ( "uuid",         "+uuid" ),
    ( "capabilities", "+capabilities" ),
    ( "org_uuid",     "+org_uuid" ),
    ( "org_name",     "+org_name" ),
  ];
  for ( param, suggestion ) in REMOVED_TOGGLES
  {
    if cmd.arguments.contains_key( *param )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "parameter '{param}' removed — use 'cols::{suggestion}' instead" ),
      ) );
    }
  }

  // ── Parse cols:: modifier string into IdentityCols ────────────────────────────
  let cols_raw = match cmd.arguments.get( "cols" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    _ => String::new(),
  };
  let cols = if cols_raw.is_empty()
  {
    IdentityCols::default_set()
  }
  else
  {
    IdentityCols::parse( &cols_raw )?
  };

  // Compute owner strings for display (skipped when cols.owner is false to avoid I/O).
  let owners : Vec< String > = if cols.owner
  {
    accounts.iter().map( |a| crate::account::read_owner( &credential_store, &a.name ) ).collect()
  }
  else
  {
    Vec::new()
  };

  // Detect which account matches the live session token (graceful: None when creds absent).
  let live_creds = crate::ClaudePaths::new()
    .map_or_else( || std::path::PathBuf::from( "/dev/null" ), |p| p.credentials_file() );
  let current_name = detect_current_account( &all_accounts, &live_creds, &credential_store );

  let content = match opts.format
  {
    OutputFormat::Json => render_accounts_json( &accounts, current_name.as_deref() ),
    OutputFormat::Text =>
    {
      render_accounts_text( &accounts, &owners, &cols, current_name.as_deref() )
    }
    OutputFormat::Table =>
    {
      render_accounts_table( &accounts, &owners, &cols, current_name.as_deref() )
    }
  };
  Ok( OutputData::new( content, "text" ) )
}
