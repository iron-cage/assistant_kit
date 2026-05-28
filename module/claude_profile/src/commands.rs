//! Command handlers: one function per `claude_profile` CLI command.
//!
//! Each handler receives a `VerifiedCommand` and `ExecutionContext` and returns
//! `Result<OutputData, ErrorData>`. Handlers are registered via
//! [`register_commands()`](crate::register_commands) in `lib.rs`;
//! the binary-specific `.` handler is registered inline in `build_registry()` in `lib.rs`.
//!
//! # Note on `needless_pass_by_value`
//!
//! Every handler takes `VerifiedCommand` by value because the `CommandRoutine`
//! type alias requires ownership.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use claude_quota::RateLimitData;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ OutputFormat, OutputOptions, json_escape, format_duration_secs };

// ── Private helpers ───────────────────────────────────────────────────────────

fn require_nonempty_string_arg( cmd : &VerifiedCommand, name : &str ) -> Result< String, ErrorData >
{
  let val = match cmd.arguments.get( name )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: is required" ) ) ),
  };
  if val.is_empty()
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: value cannot be empty" ) ) );
  }
  Ok( val )
}

fn is_dry( cmd : &VerifiedCommand ) -> bool
{
  matches!( cmd.arguments.get( "dry" ), Some( Value::Boolean( true ) ) )
}

/// Parse a strict opt-in flag registered with `Kind::String`: absent or `"0"` → false, `"1"` → true.
///
/// Rejects any other value (e.g. `"yes"`, `"2"`) with an error naming the parameter.
/// Used for opt-in display flags where the framework's lenient boolean parsing
/// (`"yes"` → true) is too permissive.
fn parse_opt_bool_strict( cmd : &VerifiedCommand, name : &str ) -> Result< bool, ErrorData >
{
  match cmd.arguments.get( name )
  {
    None                       => Ok( false ),
    Some( Value::String( s ) ) => match s.as_str()
    {
      "0" => Ok( false ),
      "1" => Ok( true ),
      _   => Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "invalid value for {name}:: — expected 0 or 1, got {s:?}" ),
      ) ),
    },
    Some( _ ) => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "invalid value for {name}:: — expected 0 or 1" ),
    ) ),
  }
}

/// Classify a token from its stored `expiresAt` millisecond value.
///
/// Used for non-active named accounts where reading the live credentials file
/// would return the active account's token state, not the queried account's.
///
// Fix(issue-p2-named-account-token):
// Root cause: `status_with_threshold()` reads `~/.claude/.credentials.json`
//   which belongs to the ACTIVE account. For non-active named accounts, that
//   returns the active account's token — not the queried one's.
// Pitfall: Never call `status_with_threshold()` for non-active named accounts.
//   Always compute `TokenStatus` from the account's own stored `expiresAt`.
fn token_status_from_ms( expires_at_ms : u64 ) -> crate::token::TokenStatus
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_ms = u64::try_from(
    SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .unwrap_or_default()
      .as_millis()
  ).unwrap_or( u64::MAX );

  if now_ms >= expires_at_ms
  {
    crate::token::TokenStatus::Expired
  }
  else
  {
    let remaining = core::time::Duration::from_millis( expires_at_ms - now_ms );
    if remaining.as_secs() <= crate::token::WARNING_THRESHOLD_SECS
    {
      crate::token::TokenStatus::ExpiringSoon { expires_in : remaining }
    }
    else
    {
      crate::token::TokenStatus::Valid { expires_in : remaining }
    }
  }
}

/// Validate HOME is non-empty and return a `ClaudePaths`.
fn require_claude_paths() -> Result< crate::ClaudePaths, ErrorData >
{
  match std::env::var( "HOME" )
  {
    Ok( home ) if !home.is_empty() =>
    {
      crate::ClaudePaths::new().ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "HOME environment variable not set".to_string(),
      ) )
    }
    _ => Err( ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) ),
  }
}

/// Resolve the credential store path via `PersistPaths`.
fn require_credential_store() -> Result< std::path::PathBuf, ErrorData >
{
  crate::PersistPaths::new()
    .map( | p | p.credential_store() )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "persistent storage unavailable: {e}" ),
    ) )
}

/// Map `std::io::Error` to `ErrorData` with appropriate exit code.
///
/// - `InvalidInput` → `ArgumentTypeMismatch` (exit 1)
/// - everything else → `InternalError` (exit 2)
fn io_err_to_error_data( e : &std::io::Error, context : &str ) -> ErrorData
{
  let code = match e.kind()
  {
    std::io::ErrorKind::InvalidInput => ErrorCode::ArgumentTypeMismatch,
    _                                => ErrorCode::InternalError,
  };
  ErrorData::new( code, format!( "{context}: {e}" ) )
}

/// Resolve a raw account name: full email passes through; bare prefix is resolved via saved accounts.
///
/// - Contains `@` → returned as-is (treated as full email; downstream `validate_name` catches format errors).
/// - No `@` with path-unsafe chars (`/`, `\`, `*`) → `ArgumentTypeMismatch` (exit 1).
/// - No `@` (prefix) → prefix-match all saved account names:
///   - Exactly 1 account has a local part (before `@`) equal to `raw` → resolve to that account (exact local-part match wins).
///   - Exactly 1 prefix match → return that name.
///   - 0 matches → `InternalError` (exit 2): not found.
///   - 2+ matches → `ArgumentTypeMismatch` (exit 1): ambiguous prefix.
// Fix(issue-name-shortcut):
// Root cause: bare prefix args like `alice` were passed to `validate_name()` which rejected them
//   with exit 1 ("not an email address"), masking the correct "not found" (exit 2) outcome.
// Pitfall: Prefix resolution must occur BEFORE validate_name(); calling validate_name() on a
//   bare prefix always returns exit 1, preventing the resolver from running at all.
// Fix(issue-exact-local-part):
// Root cause: `starts_with("i1")` matched `i1@wbox.pro`, `i11@wbox.pro`, `i12@wbox.pro`, all
//   reported as ambiguous even though `i1` is an exact local-part match for `i1@wbox.pro`.
// Pitfall: Always check exact-local-part match before prefix scanning; prefix scanning is
//   only meaningful when no account's local part equals the input exactly.
fn resolve_account_name( raw : &str, store : &std::path::Path ) -> Result< String, ErrorData >
{
  if raw.contains( '@' )
  {
    return Ok( raw.to_string() );
  }
  if raw.contains( '/' ) || raw.contains( '\\' ) || raw.contains( '*' )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "account name prefix '{raw}' contains invalid characters" ),
    ) );
  }
  let accounts = crate::account::list( store )
    .map_err( |e| ErrorData::new( ErrorCode::InternalError, format!( "cannot list accounts: {e}" ) ) )?;
  // Exact local-part match: if exactly one account has a local part equal to `raw`, resolve to it.
  // This prevents `i1` from being ambiguous when both `i1@host` and `i11@host` exist.
  let exact : Vec< &str > = accounts.iter()
    .filter( | a | a.name.split_once( '@' ).is_some_and( | ( local, _ ) | local == raw ) )
    .map( | a | a.name.as_str() )
    .collect();
  if exact.len() == 1
  {
    return Ok( exact[ 0 ].to_string() );
  }
  let matches : Vec< &str > = accounts.iter()
    .filter( |a| a.name.starts_with( raw ) )
    .map( |a| a.name.as_str() )
    .collect();
  match matches.len()
  {
    1 => Ok( matches[ 0 ].to_string() ),
    0 => Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "account '{raw}' not found" ),
    ) ),
    _ => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "ambiguous prefix '{raw}': matches {}", matches.join( ", " ) ),
    ) ),
  }
}

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

/// Derive the token state display strings from a raw `TokenStatus` result.
///
/// Returns `( tok_label, exp_label, exp_secs )`:
/// - `tok_label` — "valid", "expiring in Nm", "expired", or "unknown"
/// - `exp_label` — "in Xh Ym", "expired", or "(unavailable)"
/// - `exp_secs`  — seconds until expiry; `0` when expired or unavailable
fn derive_token_state(
  ts : &Result< crate::token::TokenStatus, std::io::Error >,
) -> ( String, String, u64 )
{
  let tok = match ts
  {
    Ok( crate::token::TokenStatus::Valid { .. } )                => "valid".to_string(),
    Ok( crate::token::TokenStatus::ExpiringSoon { expires_in } ) =>
      format!( "expiring in {}m", expires_in.as_secs() / 60 ),
    Ok( crate::token::TokenStatus::Expired )                     => "expired".to_string(),
    Err( _ )                                                     => "unknown".to_string(),
  };
  let exp = match ts
  {
    Ok( crate::token::TokenStatus::Valid { expires_in }
      | crate::token::TokenStatus::ExpiringSoon { expires_in } ) =>
    {
      let h = expires_in.as_secs() / 3600;
      let m = ( expires_in.as_secs() % 3600 ) / 60;
      format!( "in {h}h {m}m" )
    }
    Ok( crate::token::TokenStatus::Expired ) => "expired".to_string(),
    Err( _ )                                 => "(unavailable)".to_string(),
  };
  let exp_secs = match ts
  {
    Ok( crate::token::TokenStatus::Valid { expires_in }
      | crate::token::TokenStatus::ExpiringSoon { expires_in } ) => expires_in.as_secs(),
    _ => 0,
  };
  ( tok, exp, exp_secs )
}

/// Render a `Vec<String>` capability list as a JSON array string.
///
/// Empty vec renders as `[]`. Each element is JSON-escaped.
fn caps_to_json( caps : &[ String ] ) -> String
{
  if caps.is_empty() { return "[]".to_string(); }
  let inner : Vec< String > = caps.iter()
    .map( | c | format!( "\"{}\"", json_escape( c ) ) )
    .collect();
  format!( "[{}]", inner.join( "," ) )
}

// ── Command handlers ──────────────────────────────────────────────────────────

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
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
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
  let show_uuid         = parse_opt_bool_strict( &cmd, "uuid" )?;
  let show_capabilities = parse_opt_bool_strict( &cmd, "capabilities" )?;
  let show_org_uuid     = parse_opt_bool_strict( &cmd, "org_uuid" )?;
  let show_org_name     = parse_opt_bool_strict( &cmd, "org_name" )?;

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

/// Render an account list in text format with field-presence control.
///
/// Returns `"(no accounts configured)\n"` when `accounts` is empty.
/// When any field flag is `true`, each account block is followed by its fields
/// and separated from the next account by a blank line.
// Conditional rendering for 16 optional account fields; extraction into a helper
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
    || show_display_name || show_role || show_billing || show_model || show_uuid || show_capabilities
    || show_org_uuid || show_org_name;
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
        let ts  = token_status_from_ms( a.expires_at_ms );
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
      if show_role
      {
        let role = if a.role.is_empty() { "N/A" } else { &a.role };
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

/// Render a slice of accounts as a `data_fmt` ASCII table.
///
/// Columns: flag (active/current marker), Account, Active, Sub, Tier, Expires.
/// `current_name` is matched against account names to populate the flag column;
/// `✓` = current, `*` = active-but-not-current, blank otherwise.
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
       \"organization_role\":\"{}\",\"workspace_uuid\":\"{}\",\"workspace_name\":\"{}\"}}",
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
    )
  } ).collect();
  format!( "[{}]\n", entries.join( "," ) )
}

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
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
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
  let show_role         = matches!( cmd.arguments.get( "role"         ), Some( Value::Boolean( true ) ) );
  let show_billing      = matches!( cmd.arguments.get( "billing"      ), Some( Value::Boolean( true ) ) );
  let show_model        = matches!( cmd.arguments.get( "model"        ), Some( Value::Boolean( true ) ) );
  let show_uuid         = parse_opt_bool_strict( &cmd, "uuid" )?;
  let show_capabilities = parse_opt_bool_strict( &cmd, "capabilities" )?;
  let show_org_uuid     = parse_opt_bool_strict( &cmd, "org_uuid" )?;
  let show_org_name     = parse_opt_bool_strict( &cmd, "org_name" )?;

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
        show_display_name, show_role, show_billing, show_model,
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

/// `.account.use` — atomic credential rotation by name.
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the target account does not exist.
#[ inline ]
pub fn account_use_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Validate all CLI arguments before any I/O (fast-fail on bad values before filesystem access).
  // Fix(issue-switch-dry-validation): is_dry() check comes after existence validation so
  //   dry-run on nonexistent accounts correctly exits 2 (not silently succeeds).
  // Pitfall: Only the mutating step (file copy + marker write) is skipped in dry-run;
  //   all validation and precondition checks must run unconditionally.
  let raw_name   = require_nonempty_string_arg( &cmd, "name" )?;
  let touch      = crate::usage::parse_int_flag( &cmd, "touch", 1 )?;
  let trace      = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let imodel_str = match cmd.arguments.get( "imodel" )
  {
    None                       => "auto".to_string(),
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_imodel_str( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      s.clone()
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "imodel:: must be a string".to_string() ) ),
  };
  let effort_str = match cmd.arguments.get( "effort" )
  {
    None                       => "auto".to_string(),
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_effort_str( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      s.clone()
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "effort:: must be a string".to_string() ) ),
  };
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would switch to '{name}'\n" ), "text" ) );
  }

  // Pre-fetch quota before the switch while the target credential file is still readable.
  let touch_ctx = if touch != 0
  {
    crate::usage::pre_switch_touch_ctx( &name, &credential_store, trace, &imodel_str, &effort_str )
  }
  else
  {
    None
  };

  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  // Post-switch: activate idle session if quota indicated it was idle before switch.
  if let Some( ctx ) = touch_ctx
  {
    crate::usage::apply_post_switch_touch( &name, ctx, &imodel_str, &effort_str, trace );
  }

  Ok( OutputData::new( format!( "switched to '{name}'\n" ), "text" ) )
}
/// `.account.rotate` — auto-rotate to the highest-expiry inactive account.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset, the credential store cannot be read,
/// or no inactive account is available to rotate to.
#[ inline ]
pub fn account_rotate_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.rotate  reading store: {}", credential_store.display() ) }
  let paths            = require_claude_paths()?;
  if is_dry( &cmd )
  {
    let candidate = crate::account::list( &credential_store )
      .map_err( |e| io_err_to_error_data( &e, "account rotate" ) )?
      .into_iter()
      .filter( |a| !a.is_active )
      .max_by_key( |a| a.expires_at_ms )
      .ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "no inactive account available to rotate to".to_string(),
      ) )?;
    return Ok( OutputData::new( format!( "[dry-run] would rotate to '{}'\n", candidate.name ), "text" ) );
  }
  let name = crate::account::auto_rotate( &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account rotate" ) )?;
  Ok( OutputData::new( format!( "rotated to '{name}'\n" ), "text" ) )
}

pub use crate::usage::usage_routine;

// ── .account.limits helpers ──────────────────────────────────────────────────

/// Verify the active-account credentials file exists.
///
/// Returns the path to `~/.claude/.credentials.json` if present, or `Err`
/// (exit 2) with an actionable error message if no active credentials are found.
fn require_active_credentials( paths : &crate::ClaudePaths ) -> Result< std::path::PathBuf, ErrorData >
{
  let creds = paths.credentials_file();
  if !creds.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      "no active account \u{2014} run `claude auth login` to authenticate".to_string(),
    ) );
  }
  Ok( creds )
}

/// Read the OAuth access token from a credentials file.
///
/// Searches for `accessToken` in the credential JSON using `parse_string_field`.
/// Works for both the active credentials file and saved named account files
/// because the field search is position-independent.
fn read_auth_token( creds_path : &std::path::Path ) -> Result< String, ErrorData >
{
  let content = std::fs::read_to_string( creds_path )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot read credentials: {e}" ),
    ) )?;
  crate::account::parse_string_field( &content, "accessToken" )
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "credentials missing 'accessToken' \u{2014} re-authenticate with `claude auth login`".to_string(),
    ) )
}

/// Format rate-limit data as human-readable text: labelled with reset durations.
fn format_rate_limits_text( data : &RateLimitData ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  let pct_session       = format!( "{:.0}", data.utilization_5h * 100.0 );
  let pct_weekly        = format!( "{:.0}", data.utilization_7d * 100.0 );
  let reset_session_str = format_duration_secs( data.reset_5h.saturating_sub( now_secs ) );
  let reset_weekly_str  = format_duration_secs( data.reset_7d.saturating_sub( now_secs ) );
  let status            = &data.status;
  format!( "Session (5h):  {pct_session}% consumed, resets in {reset_session_str}\nWeekly (7d):   {pct_weekly}% consumed, resets in {reset_weekly_str}\nStatus:        {status}\n" )
}

/// Format rate-limit data as a JSON object.
fn format_rate_limits_json( data : &RateLimitData ) -> String
{
  let pct_session  = format!( "{:.0}", data.utilization_5h * 100.0 );
  let pct_weekly   = format!( "{:.0}", data.utilization_7d * 100.0 );
  let ts_session   = data.reset_5h;
  let ts_weekly    = data.reset_7d;
  let status_esc   = json_escape( &data.status );
  format!( "{{\n  \"session_5h_pct\": {pct_session},\n  \"session_5h_reset_ts\": {ts_session},\n  \"weekly_7d_pct\": {pct_weekly},\n  \"weekly_7d_reset_ts\": {ts_weekly},\n  \"status\": \"{status_esc}\"\n}}\n" )
}

/// `.account.limits` — show rate-limit utilization for the selected account (FR-18).
///
/// Makes a lightweight `POST /v1/messages` to fetch `anthropic-ratelimit-unified-*`
/// response headers; outputs session (5h) and weekly (7d) utilization percentages.
///
/// # Errors
///
/// Returns `ErrorData` if:
/// - HOME is unset (exit 2)
/// - `name::` contains invalid characters (exit 1)
/// - Named account does not exist (exit 2)
/// - No active credentials are configured (exit 2)
/// - Credentials missing `accessToken` (exit 2)
/// - HTTP transport fails or rate-limit headers absent (exit 2)
#[ inline ]
pub fn account_limits_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts             = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.limits  store: {}", credential_store.display() ) }

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };

  let creds_path = if raw_name.is_empty()
  {
    require_active_credentials( &paths )?
  }
  else
  {
    let name_arg = resolve_account_name( &raw_name, &credential_store )?;
    crate::account::validate_name( &name_arg )
      .map_err( | e | io_err_to_error_data( &e, "account limits" ) )?;
    let path = credential_store.join( format!( "{name_arg}.credentials.json" ) );
    if !path.exists()
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "account '{name_arg}' not found" ),
      ) );
    }
    path
  };

  let token = read_auth_token( &creds_path )?;
  let data  = claude_quota::fetch_rate_limits( &token )
    .map_err( |e| ErrorData::new( ErrorCode::InternalError, e.to_string() ) )?;
  let text = match opts.format
  {
    OutputFormat::Json  => format_rate_limits_json( &data ),
    OutputFormat::Text  => format_rate_limits_text( &data ),
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };
  Ok( OutputData::new( text, "text" ) )
}

/// `.` handler — registered in the command registry as a hidden fallback.
///
/// The adapter intercepts `.` before it reaches the registry and redirects it
/// to `.help`, so this routine is never invoked in normal operation. It is kept
/// registered to satisfy the `hidden_from_list` registry entry and to prevent
/// "unknown command" errors if the adapter path is ever bypassed.
///
/// # Errors
///
/// Never returns an error — always succeeds with empty output.
#[ inline ]
pub fn dot_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  Ok( OutputData::new( String::new(), "text" ) )
}

/// `.account.save` — save current credentials as a named account profile.
///
/// # Errors
///
/// Returns `ErrorData` if the name cannot be resolved (explicit empty value or
/// `_active` marker absent from the credential store), HOME is unset,
/// or the credential copy fails.
#[ inline ]
pub fn account_save_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let paths            = require_claude_paths()?;
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let name             = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    Some( Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentMissing, "name:: value cannot be empty".to_string() ) ),
    _ =>
    {
      // Fix(BUG-212): read oauthAccount.emailAddress from ~/.claude.json as primary inference source;
      //   fall back to _active marker only when emailAddress is absent or empty.
      // Root cause: BUG-209 fix replaced stale top-level emailAddress with _active marker, but the marker
      //   is only written by clp ops (switch_account, save). External OAuth login writes ~/.claude.json
      //   (including oauthAccount.emailAddress) without updating _active — leaving the marker stale.
      // Pitfall: any single-source inference fails when other credential-change paths bypass that source.
      //   oauthAccount.emailAddress is updated by BOTH clp switches (snapshot restore) AND external OAuth
      //   login (Claude writes ~/.claude.json on every auth). _active is clp-only.
      let cs          = require_credential_store()?;
      let cj          = std::fs::read_to_string( paths.claude_json_file() ).unwrap_or_default();
      // Extract emailAddress nested inside oauthAccount {…}: locate "oauthAccount": first,
      // then apply parse_string_field on the suffix so only the nested key is found.
      let oauth_email = cj
        .find( "\"oauthAccount\":" )
        .and_then( | pos | crate::account::parse_string_field( &cj[ pos.. ], "emailAddress" ) )
        .filter( | s | !s.is_empty() );
      if let Some( email ) = oauth_email
      {
        email
      }
      else
      {
        let marker_path = cs.join( crate::account::active_marker_filename() );
        std::fs::read_to_string( &marker_path )
          .ok()
          .map( | s | s.trim().to_string() )
          .filter( | s | !s.is_empty() )
          .ok_or_else( || ErrorData::new(
            ErrorCode::ArgumentMissing,
            "cannot infer account name: no active account set — pass name:: explicitly".to_string(),
          ) )?
      }
    }
  };
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.save  reading {}", paths.credentials_file().display() ) }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would save current credentials as '{name}'\n" ), "text" ) );
  }

  crate::account::save( &name, &credential_store, &paths, true )
    .map_err( |e| io_err_to_error_data( &e, "account save" ) )?;
  if trace { eprintln!( "[trace] account.save  write: OK" ) }
  Ok( OutputData::new( format!( "saved current credentials as '{name}'\n" ), "text" ) )
}

/// `.account.delete` — delete a saved account (guard: refuses active).
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the account does not exist.
#[ inline ]
pub fn account_delete_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Fix(issue-delete-dry-validation):
  // Root cause: is_dry() was checked before existence check,
  //   so dry-run bypassed NotFound (missing account).
  // Pitfall: precondition checks must run before the dry-run shortcut.
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let raw_name         = require_nonempty_string_arg( &cmd, "name" )?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.delete  store: {}", credential_store.display() ) }
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_delete_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would delete account '{name}'\n" ), "text" ) );
  }

  crate::account::delete( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;
  Ok( OutputData::new( format!( "deleted account '{name}'\n" ), "text" ) )
}

/// `.account.relogin` — force browser re-authentication for a named account with dead refreshToken.
///
/// Switches to the named account, spawns `claude` with inherited TTY so the user can
/// complete browser login, then saves the refreshed credentials back into the account store
/// and restores the original active account.
///
/// # Errors
///
/// - Exit 1: `name::` value is empty or contains invalid characters.
/// - Exit 2: `name::` omitted and no active account; account not found; HOME unset;
///   `claude` binary cannot be spawned; or save fails.
/// - Exit 3 (via `process::exit`): `claude` exited without updating `~/.claude/.credentials.json`
///   (login abandoned or timed out).
#[ inline ]
pub fn account_relogin_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace            = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.relogin  store: {}", credential_store.display() ) }
  let raw_name         = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    Some( Value::String( _ ) )                  =>
      return Err( ErrorData::new(
        ErrorCode::ArgumentMissing,
        "name:: value cannot be empty".to_string(),
      ) ),
    _ =>
      std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) )
        .ok()
        .map( | s | s.trim().to_string() )
        .filter( | s | !s.is_empty() )
        .ok_or_else( || ErrorData::new(
          ErrorCode::InternalError,
          "name:: omitted and no active account — set an active account via .account.use or pass name:: explicitly".to_string(),
        ) )?,
  };
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account relogin" ) )?;

  // Snapshot original active — best-effort; None when marker absent.
  let original_active = std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) )
    .ok()
    .map( | s | s.trim().to_string() )
    .filter( | s | !s.is_empty() );

  if is_dry( &cmd )
  {
    return Ok( OutputData::new(
      format!( "[dry-run] would re-authenticate '{name}' via browser login\n" ),
      "text",
    ) );
  }

  // Make the named account the live session so `claude` picks up its refreshToken.
  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account relogin: switch" ) )?;

  // Snapshot credentials content before spawning.
  let creds_path   = paths.credentials_file();
  let before_creds = std::fs::read_to_string( &creds_path ).unwrap_or_default();

  // Spawn `claude` with inherited TTY — NOT run_isolated — so the user sees the browser login flow.
  // Delegates to claude_runner_core::ClaudeCommand::execute_interactive() to respect the Single
  // Execution Point Rule: all process spawning goes through claude_runner_core.
  let spawn_result = claude_runner_core::ClaudeCommand::new()
    .execute_interactive();

  if let Err( e ) = spawn_result
  {
    // Restore original before returning — switch already happened above.
    if let Some( original ) = &original_active
    {
      if original != &name
      {
        let _ = crate::account::switch_account( original, &credential_store, &paths );
      }
    }
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot spawn claude: {e}" ),
    ) );
  }

  // Detect whether credentials were refreshed by comparing file content.
  let after_creds = std::fs::read_to_string( &creds_path ).unwrap_or_default();
  let updated     = after_creds != before_creds;

  if updated
  {
    // Persist the refreshed credentials into the account store.
    crate::account::save( &name, &credential_store, &paths, true )
      .map_err( |e| io_err_to_error_data( &e, "account relogin: save" ) )?;
  }

  // Restore the original active account (best-effort — failure is non-fatal).
  if let Some( original ) = &original_active
  {
    if original != &name
    {
      let _ = crate::account::switch_account( original, &credential_store, &paths );
    }
  }

  if !updated
  {
    // Fix(BUG-183): bare exit(3) produced no user-visible output.
    // Root cause: all other paths return OutputData, but this branch bypassed the dispatcher.
    // Pitfall: process::exit bypasses return-based output — always add eprintln before it.
    eprintln!( "relogin abandoned \u{2014} credentials unchanged for '{name}'" );
    std::process::exit( 3 );
  }

  Ok( OutputData::new( format!( "re-authenticated '{name}' — credentials saved\n" ), "text" ) )
}

/// `.token.status` — show active OAuth token expiry classification.
///
/// **CRITICAL:** Uses `status_with_threshold()`, NEVER bare function that
/// matches the responsibility test grep pattern. See P1 in the plan.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset, credentials are missing,
/// or the `expiresAt` field is unparseable.
#[ inline ]
pub fn token_status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let trace = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let paths = require_claude_paths()?;
  if trace { eprintln!( "[trace] token.status  reading {}", paths.credentials_file().display() ) }

  let threshold_secs = match cmd.arguments.get( "threshold" )
  {
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( crate::token::WARNING_THRESHOLD_SECS ),
    _ => crate::token::WARNING_THRESHOLD_SECS,
  };

  let token_result = crate::token::status_with_threshold( threshold_secs )
    .map_err( |e| io_err_to_error_data( &e, "token status" ) )?;

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      match &token_result
      {
        crate::token::TokenStatus::Valid { expires_in } =>
          format!( "{{\"status\":\"valid\",\"expires_in_secs\":{}}}\n", expires_in.as_secs() ),
        crate::token::TokenStatus::ExpiringSoon { expires_in } =>
          format!( "{{\"status\":\"expiring_soon\",\"expires_in_secs\":{}}}\n", expires_in.as_secs() ),
        crate::token::TokenStatus::Expired =>
          "{\"status\":\"expired\"}\n".to_string(),
      }
    }
    OutputFormat::Text =>
    {
      match &token_result
      {
        crate::token::TokenStatus::Valid { expires_in } =>
          format!( "valid — {}m remaining\n", expires_in.as_secs() / 60 ),
        crate::token::TokenStatus::ExpiringSoon { expires_in } =>
          format!( "expiring soon — {}m remaining\n", expires_in.as_secs() / 60 ),
        crate::token::TokenStatus::Expired =>
          "expired\n".to_string(),
      }
    }
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.paths` — show all resolved `~/.claude/` canonical file paths.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset or empty.
#[ inline ]
pub fn paths_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace = crate::usage::parse_int_flag( &cmd, "trace", 0 )? != 0;
  if let Some( Value::String( field ) ) = cmd.arguments.get( "field" )
  {
    if !field.is_empty()
    {
      let paths            = require_claude_paths()?;
      let credential_store = require_credential_store()?;
      let raw = match field.as_str()
      {
        "base"             => paths.base().display().to_string(),
        "credentials"      => paths.credentials_file().display().to_string(),
        "credential_store" => credential_store.display().to_string(),
        "projects"         => paths.projects_dir().display().to_string(),
        "stats"            => paths.stats_file().display().to_string(),
        "settings"         => paths.settings_file().display().to_string(),
        "session_env"      => paths.session_env_dir().display().to_string(),
        "sessions"         => paths.sessions_dir().display().to_string(),
        other =>
        {
          return Err( ErrorData::new(
            ErrorCode::ArgumentTypeMismatch,
            format!( "unknown field '{other}'; valid: base, credentials, credential_store, projects, stats, settings, session_env, sessions" ),
          ) );
        }
      };
      return Ok( OutputData::new( format!( "{raw}\n" ), "text" ) );
    }
  }
  let opts             = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let paths            = require_claude_paths()?;
  if trace { eprintln!( "[trace] paths  base: {}", paths.base().display() ) }
  let credential_store = require_credential_store()?;

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      format!(
        concat!(
          "{{\"base\":\"{}\",",
          "\"credentials\":\"{}\",",
          "\"credential_store\":\"{}\",",
          "\"projects\":\"{}\",",
          "\"stats\":\"{}\",",
          "\"settings\":\"{}\",",
          "\"session_env\":\"{}\",",
          "\"sessions\":\"{}\"}}\n",
        ),
        json_escape( &paths.base().display().to_string() ),
        json_escape( &paths.credentials_file().display().to_string() ),
        json_escape( &credential_store.display().to_string() ),
        json_escape( &paths.projects_dir().display().to_string() ),
        json_escape( &paths.stats_file().display().to_string() ),
        json_escape( &paths.settings_file().display().to_string() ),
        json_escape( &paths.session_env_dir().display().to_string() ),
        json_escape( &paths.sessions_dir().display().to_string() ),
      )
    }
    OutputFormat::Text =>
    {
      format!(
        "credentials:      {}\ncredential_store: {}\nprojects:         {}\nstats:            {}\nsettings:         {}\nsession-env:      {}\nsessions:         {}\n",
        paths.credentials_file().display(),
        credential_store.display(),
        paths.projects_dir().display(),
        paths.stats_file().display(),
        paths.settings_file().display(),
        paths.session_env_dir().display(),
        paths.sessions_dir().display(),
      )
    }
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };

  Ok( OutputData::new( content, "text" ) )
}
