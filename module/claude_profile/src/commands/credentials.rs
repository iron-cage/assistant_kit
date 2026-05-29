//! `.credentials.status` command handler.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::output::{ OutputFormat, OutputOptions, json_escape };
use super::shared::{ require_claude_paths, require_credential_store, derive_token_state, caps_to_json };

// ── Single-consumer helpers ───────────────────────────────────────────────────

/// Read subscription type, rate limit tier, email, display, role, and billing from live credential files.
///
/// Called by `credentials_status_routine()` to read subscription, tier, email, display, role, and billing.
/// Gracefully returns `"N/A"` for any absent or empty field.
// Fix(issue-empty-field-blank):
// Root cause: `Option::unwrap_or_else` only fires on `None`, not `Some("")`. Empty strings
//   in credential JSON (unusual but possible) produced blank output lines instead of "N/A".
// Pitfall: When adding new `parse_string_field` chains, always pair `.filter(|s| !s.is_empty())`
//   with `.unwrap_or_else(|| "N/A".to_string())` — never rely on `unwrap_or_else` alone.
fn read_live_cred_meta( paths : &crate::ClaudePaths )
  -> ( String, String, String, String, String, String )
{
  let creds   = std::fs::read_to_string( paths.credentials_file() ).unwrap_or_default();
  let sub     = crate::account::parse_string_field( &creds, "subscriptionType" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let tier    = crate::account::parse_string_field( &creds, "rateLimitTier" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  // Fix(FR-19): use claude_json_file() — ~/.claude.json lives at $HOME level, not inside ~/.claude/
  // Root cause: base().join(".claude.json") produced ~/.claude/.claude.json (one dir too deep).
  // Pitfall: ClaudePaths::base() is $HOME/.claude/, so joining there lands inside the .claude dir.
  let cj      = std::fs::read_to_string( paths.claude_json_file() ).unwrap_or_default();
  let email   = crate::account::parse_string_field( &cj, "emailAddress" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let display = crate::account::parse_string_field( &cj, "displayName" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let role    = crate::account::parse_string_field( &cj, "organizationRole" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let billing = crate::account::parse_string_field( &cj, "billingType" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  ( sub, tier, email, display, role, billing )
}

/// Read the `model` field from `~/.claude/settings.json`.
///
/// Returns `"N/A"` when the file is absent or the field is missing.
fn read_settings_model( paths : &crate::ClaudePaths ) -> String
{
  let settings = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  crate::account::parse_string_field( &settings, "model" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() )
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.credentials.status` — show live credential metadata without account store dependency.
///
/// Reads `~/.claude/.credentials.json` directly. Does not require account store setup.
/// Each output line is independently controlled by a boolean field-presence param.
/// Default-on: `account`, `sub`, `tier`, `token`, `expires`, `email`.
/// Opt-in (default off): `file`, `saved`, `display_name`, `role`, `billing`, `model`.
/// `format::json` always emits all 12 fields regardless of field-presence params.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset or `.credentials.json` is missing.
// Reading 16 field-presence flags and rendering both JSON and text formats
// in one pass; splitting would scatter tightly-coupled field reads.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn credentials_status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
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
  if trace { eprintln!( "[trace] credentials.status  reading {}", paths.credentials_file().display() ) }
  let credential_store = require_credential_store()?;

  if !paths.credentials_file().exists()
  {
    if trace { eprintln!( "[trace] credentials.status  reading: Err(not found)" ) }
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!(
        "credential file not found: {} \u{2014} run `claude auth login` to authenticate",
        paths.credentials_file().display(),
      ),
    ) );
  }
  if trace { eprintln!( "[trace] credentials.status  reading: OK" ) }

  // Per-field presence flags; None (absent param) = use default.
  // Default-on: account, sub, tier, token, expires, email.
  // Opt-in (default off): file, saved — require explicit Some(Boolean(true)).
  let show_account = matches!( cmd.arguments.get( "account" ), Some( Value::Boolean( true ) ) | None );
  let show_sub     = matches!( cmd.arguments.get( "sub"     ), Some( Value::Boolean( true ) ) | None );
  let show_tier    = matches!( cmd.arguments.get( "tier"    ), Some( Value::Boolean( true ) ) | None );
  let show_token   = matches!( cmd.arguments.get( "token"   ), Some( Value::Boolean( true ) ) | None );
  let show_expires = matches!( cmd.arguments.get( "expires" ), Some( Value::Boolean( true ) ) | None );
  let show_email   = matches!( cmd.arguments.get( "email"   ), Some( Value::Boolean( true ) ) | None );
  let show_file         = matches!( cmd.arguments.get( "file"         ), Some( Value::Boolean( true ) ) );
  let show_saved        = matches!( cmd.arguments.get( "saved"        ), Some( Value::Boolean( true ) ) );
  let show_display_name = matches!( cmd.arguments.get( "display_name" ), Some( Value::Boolean( true ) ) );
  let show_role         = matches!( cmd.arguments.get( "role"         ), Some( Value::Boolean( true ) ) );
  let show_billing      = matches!( cmd.arguments.get( "billing"      ), Some( Value::Boolean( true ) ) );
  let show_model        = matches!( cmd.arguments.get( "model"        ), Some( Value::Boolean( true ) ) );
  let show_uuid         = crate::output::parse_int_flag( &cmd, "uuid",         0 )? != 0;
  let show_capabilities = crate::output::parse_int_flag( &cmd, "capabilities", 0 )? != 0;
  let show_org_uuid     = crate::output::parse_int_flag( &cmd, "org_uuid",     0 )? != 0;
  let show_org_name     = crate::output::parse_int_flag( &cmd, "org_name",     0 )? != 0;

  let ( tok, exp, exp_secs ) = derive_token_state(
    &crate::token::status_with_threshold( crate::token::WARNING_THRESHOLD_SECS ),
  );

  let ( sub, tier, email, display, role, billing ) = read_live_cred_meta( &paths );
  let model = read_settings_model( &paths );
  // Read extended snapshot fields from live ~/.claude.json — same file, best-effort.
  let live_claude_json  = std::fs::read_to_string( paths.claude_json_file() ).unwrap_or_default();
  let tagged_id         = crate::account::parse_string_field( &live_claude_json, "taggedId" ).unwrap_or_default();
  let live_capabilities = crate::account::parse_string_array_field( &live_claude_json, "capabilities" );

  // Account: reads _active opportunistically — N/A when absent (no hard dependency).
  let active_name = std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) )
    .ok()
    .map( | s | s.trim().to_string() )
    .filter( | s | !s.is_empty() );
  let account = active_name.clone().unwrap_or_else( || "N/A".to_string() );

  // Org identity: read from {_active}.roles.json best-effort; empty when absent.
  let roles_json = active_name.as_deref()
    .and_then( | name | std::fs::read_to_string(
      credential_store.join( format!( "{name}.roles.json" ) )
    ).ok() )
    .unwrap_or_default();
  let org_uuid = crate::account::parse_string_field( &roles_json, "organization_uuid" ).unwrap_or_default();
  let org_name = crate::account::parse_string_field( &roles_json, "organization_name" ).unwrap_or_default();
  let org_role = crate::account::parse_string_field( &roles_json, "organization_role" ).unwrap_or_default();
  let ws_uuid  = crate::account::parse_string_field( &roles_json, "workspace_uuid"    ).unwrap_or_default();
  let ws_name  = crate::account::parse_string_field( &roles_json, "workspace_name"    ).unwrap_or_default();

  // Saved: count *.credentials.json files; 0 when credential_store absent.
  let saved = std::fs::read_dir( &credential_store )
    .map_or( 0, | rd | rd.filter_map( Result::ok )
      .filter( | e | e.file_name().to_string_lossy().ends_with( ".credentials.json" ) )
      .count() );

  let file_path = paths.credentials_file().display().to_string();

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      let s    = json_escape( &sub );
      let t    = json_escape( &tier );
      let tk   = json_escape( &tok );
      let em   = json_escape( &email );
      let ac   = json_escape( &account );
      let fp   = json_escape( &file_path );
      let dn   = json_escape( &display );
      let rl   = json_escape( &role );
      let bl   = json_escape( &billing );
      let md   = json_escape( &model );
      let ti   = json_escape( &tagged_id );
      let caps = caps_to_json( &live_capabilities );
      let ou   = json_escape( &org_uuid );
      let on_  = json_escape( &org_name );
      let or_  = json_escape( &org_role );
      let wu   = json_escape( &ws_uuid );
      let wn   = json_escape( &ws_name );
      format!(
        "{{\"subscription\":\"{s}\",\"tier\":\"{t}\",\"token\":\"{tk}\",\
         \"expires_in_secs\":{exp_secs},\"email\":\"{em}\",\
         \"account\":\"{ac}\",\"file\":\"{fp}\",\"saved\":{saved},\
         \"display_name\":\"{dn}\",\"role\":\"{rl}\",\"billing\":\"{bl}\",\"model\":\"{md}\",\
         \"tagged_id\":\"{ti}\",\"capabilities\":{caps},\
         \"organization_uuid\":\"{ou}\",\"organization_name\":\"{on_}\",\
         \"organization_role\":\"{or_}\",\"workspace_uuid\":\"{wu}\",\"workspace_name\":\"{wn}\"}}\n"
      )
    }
    OutputFormat::Text =>
    {
      let mut out = String::new();
      if show_account      { let _ = writeln!( out, "Account: {account}" ); }
      if show_sub          { let _ = writeln!( out, "Sub:     {sub}"     ); }
      if show_tier         { let _ = writeln!( out, "Tier:    {tier}"    ); }
      if show_token        { let _ = writeln!( out, "Token:   {tok}"     ); }
      if show_expires      { let _ = writeln!( out, "Expires: {exp}"     ); }
      if show_email        { let _ = writeln!( out, "Email:   {email}"   ); }
      if show_file         { let _ = writeln!( out, "File:    {file_path}" ); }
      if show_saved        { let _ = writeln!( out, "Saved:   {saved} account(s)" ); }
      if show_display_name { let _ = writeln!( out, "Display: {display}" ); }
      if show_role         { let _ = writeln!( out, "Role:    {role}"    ); }
      if show_billing      { let _ = writeln!( out, "Billing: {billing}" ); }
      if show_model        { let _ = writeln!( out, "Model:   {model}"   ); }
      if show_uuid
      {
        let id_val = if tagged_id.is_empty() { "N/A" } else { &tagged_id };
        let _ = writeln!( out, "ID:      {id_val}" );
      }
      if show_capabilities
      {
        let cap_val = if live_capabilities.is_empty()
        {
          "N/A".to_string()
        }
        else
        {
          live_capabilities.join( ", " )
        };
        let _ = writeln!( out, "Capabilities: {cap_val}" );
      }
      if show_org_uuid
      {
        let val = if org_uuid.is_empty() { "N/A" } else { &org_uuid };
        let _ = writeln!( out, "Org ID:  {val}" );
      }
      if show_org_name
      {
        let val = if org_name.is_empty() { "N/A" } else { &org_name };
        let _ = writeln!( out, "Org:     {val}" );
      }
      out
    }
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };
  Ok( OutputData::new( content, "text" ) )
}
