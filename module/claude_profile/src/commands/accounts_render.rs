//! Account list renderers and column-visibility type for `.accounts`.

use core::fmt::Write as _;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use unilang::data::{ ErrorCode, ErrorData };
use crate::output::{ json_escape, format_duration_secs };
use super::shared::caps_to_json;

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
pub( crate ) struct IdentityCols
{
  pub( crate ) account      : bool,
  pub( crate ) owner        : bool,
  pub( crate ) active       : bool,
  pub( crate ) current      : bool,
  pub( crate ) sub          : bool,
  pub( crate ) tier         : bool,
  pub( crate ) expires      : bool,
  pub( crate ) email        : bool,
  pub( crate ) display_name : bool,
  pub( crate ) host         : bool,
  pub( crate ) role         : bool,
  pub( crate ) billing      : bool,
  pub( crate ) model        : bool,
  pub( crate ) uuid         : bool,
  pub( crate ) capabilities : bool,
  pub( crate ) org_uuid     : bool,
  pub( crate ) org_name     : bool,
}

impl IdentityCols
{
  pub( crate ) fn default_set() -> Self
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
  pub( crate ) fn parse( s : &str ) -> Result< Self, ErrorData >
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
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
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
        _ => return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          format!( "unknown cols:: column id '{name}'; valid: account, owner, active, current, sub, tier, expires, email, display_name, host, role, billing, model, uuid, capabilities, org_uuid, org_name" ),
        ) ),
      }
    }
    Ok( cols )
  }
}

// ── Renderers ─────────────────────────────────────────────────────────────────

/// Render an account list in text format controlled by [`IdentityCols`].
///
/// Returns `"(no accounts configured)\n"` when `accounts` is empty.
/// When any field in `cols` is enabled, each account block is followed by its
/// field lines and separated from the next account by a blank line.
/// `owners` must be parallel to `accounts` (one owner string per account);
/// pass an empty slice when `cols.owner` is false.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub( crate ) fn render_accounts_text(
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
pub( crate ) fn render_accounts_json( accounts : &[ &crate::account::Account ], current_name : Option< &str > ) -> String
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
pub( crate ) fn render_accounts_table(
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
