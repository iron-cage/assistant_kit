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
/// - `InvalidInput` / `PermissionDenied` → `ArgumentTypeMismatch` (exit 1)
/// - Everything else → `InternalError` (exit 2)
fn io_err_to_error_data( e : &std::io::Error, context : &str ) -> ErrorData
{
  let code = match e.kind()
  {
    std::io::ErrorKind::InvalidInput | std::io::ErrorKind::PermissionDenied =>
      ErrorCode::ArgumentTypeMismatch,
    _ =>
      ErrorCode::InternalError,
  };
  ErrorData::new( code, format!( "{context}: {e}" ) )
}

/// Read subscription type, rate limit tier, email, and org from live credential files.
///
/// Called by `credentials_status_routine()` to read subscription, tier, email, and org.
/// Gracefully returns `"N/A"` for any absent or empty field.
// Fix(issue-empty-field-blank):
// Root cause: `Option::unwrap_or_else` only fires on `None`, not `Some("")`. Empty strings
//   in credential JSON (unusual but possible) produced blank output lines instead of "N/A".
// Pitfall: When adding new `parse_string_field` chains, always pair `.filter(|s| !s.is_empty())`
//   with `.unwrap_or_else(|| "N/A".to_string())` — never rely on `unwrap_or_else` alone.
fn read_live_cred_meta( paths : &crate::ClaudePaths ) -> ( String, String, String, String )
{
  let creds = std::fs::read_to_string( paths.credentials_file() ).unwrap_or_default();
  let sub   = crate::account::parse_string_field( &creds, "subscriptionType" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let tier  = crate::account::parse_string_field( &creds, "rateLimitTier" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let cj    = std::fs::read_to_string( paths.base().join( ".claude.json" ) ).unwrap_or_default();
  let email = crate::account::parse_string_field( &cj, "emailAddress" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  let org   = crate::account::parse_string_field( &cj, "organizationName" )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );
  ( sub, tier, email, org )
}

// ── Command handlers ──────────────────────────────────────────────────────────

/// `.credentials.status` — show live credential metadata without account store dependency.
///
/// Reads `~/.claude/.credentials.json` directly. Does not require account store setup.
/// Each output line is independently controlled by a boolean field-presence param.
/// Default-on: `account`, `sub`, `tier`, `token`, `expires`, `email`, `org`.
/// Opt-in (default off): `file`, `saved`.
/// `format::json` always emits all 9 fields regardless of field-presence params.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset or `.credentials.json` is missing.
#[ inline ]
pub fn credentials_status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts             = OutputOptions::from_cmd( &cmd )?;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;

  if !paths.credentials_file().exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!(
        "credential file not found: {} \u{2014} run `claude auth login` to authenticate",
        paths.credentials_file().display(),
      ),
    ) );
  }

  // Per-field presence flags; None (absent param) = use default.
  // Default-on: account, sub, tier, token, expires, email, org.
  // Opt-in (default off): file, saved — require explicit Some(Boolean(true)).
  let show_account = matches!( cmd.arguments.get( "account" ), Some( Value::Boolean( true ) ) | None );
  let show_sub     = matches!( cmd.arguments.get( "sub"     ), Some( Value::Boolean( true ) ) | None );
  let show_tier    = matches!( cmd.arguments.get( "tier"    ), Some( Value::Boolean( true ) ) | None );
  let show_token   = matches!( cmd.arguments.get( "token"   ), Some( Value::Boolean( true ) ) | None );
  let show_expires = matches!( cmd.arguments.get( "expires" ), Some( Value::Boolean( true ) ) | None );
  let show_email   = matches!( cmd.arguments.get( "email"   ), Some( Value::Boolean( true ) ) | None );
  let show_org     = matches!( cmd.arguments.get( "org"     ), Some( Value::Boolean( true ) ) | None );
  let show_file    = matches!( cmd.arguments.get( "file"    ), Some( Value::Boolean( true ) ) );
  let show_saved   = matches!( cmd.arguments.get( "saved"   ), Some( Value::Boolean( true ) ) );

  let ts  = crate::token::status_with_threshold( crate::token::WARNING_THRESHOLD_SECS );
  let tok = match &ts
  {
    Ok( crate::token::TokenStatus::Valid { .. } )                => "valid".to_string(),
    Ok( crate::token::TokenStatus::ExpiringSoon { expires_in } ) =>
      format!( "expiring in {}m", expires_in.as_secs() / 60 ),
    Ok( crate::token::TokenStatus::Expired )                     => "expired".to_string(),
    Err( _ )                                                     => "unknown".to_string(),
  };
  let exp = match &ts
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

  let ( sub, tier, email, org ) = read_live_cred_meta( &paths );

  // Account: reads _active opportunistically — N/A when absent (no hard dependency).
  let account = std::fs::read_to_string( credential_store.join( "_active" ) )
    .ok()
    .map( | s | s.trim().to_string() )
    .filter( | s | !s.is_empty() )
    .unwrap_or_else( || "N/A".to_string() );

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
      let s  = json_escape( &sub );
      let t  = json_escape( &tier );
      let tk = json_escape( &tok );
      let em = json_escape( &email );
      let or = json_escape( &org );
      let ac = json_escape( &account );
      let fp = json_escape( &file_path );
      let exp_secs = match &ts
      {
        Ok( crate::token::TokenStatus::Valid { expires_in }
          | crate::token::TokenStatus::ExpiringSoon { expires_in } ) =>
          expires_in.as_secs().to_string(),
        _ => "0".to_string(),
      };
      format!( "{{\"subscription\":\"{s}\",\"tier\":\"{t}\",\"token\":\"{tk}\",\"expires_in_secs\":{exp_secs},\"email\":\"{em}\",\"org\":\"{or}\",\"account\":\"{ac}\",\"file\":\"{fp}\",\"saved\":{saved}}}\n" )
    }
    OutputFormat::Text =>
    {
      let mut out = String::new();
      if show_account { let _ = writeln!( out, "Account: {account}" ); }
      if show_sub     { let _ = writeln!( out, "Sub:     {sub}"     ); }
      if show_tier    { let _ = writeln!( out, "Tier:    {tier}"    ); }
      if show_token   { let _ = writeln!( out, "Token:   {tok}"     ); }
      if show_expires { let _ = writeln!( out, "Expires: {exp}"     ); }
      if show_email   { let _ = writeln!( out, "Email:   {email}"   ); }
      if show_org     { let _ = writeln!( out, "Org:     {org}"     ); }
      if show_file    { let _ = writeln!( out, "File:    {file_path}" ); }
      if show_saved   { let _ = writeln!( out, "Saved:   {saved} account(s)" ); }
      out
    }
  };
  Ok( OutputData::new( content, "text" ) )
}

/// Render an account list in text format with field-presence control.
///
/// Returns `"(no accounts configured)\n"` when `accounts` is empty.
/// When any field flag is `true`, each account block is followed by its fields
/// and separated from the next account by a blank line.
#[ allow( clippy::fn_params_excessive_bools ) ]
#[ inline ]
fn render_accounts_text(
  accounts     : &[ &crate::account::Account ],
  show_active  : bool,
  show_sub     : bool,
  show_tier    : bool,
  show_expires : bool,
  show_org     : bool,
) -> String
{
  if accounts.is_empty() { return "(no accounts configured)\n".to_string(); }
  let any_field = show_active || show_sub || show_tier || show_expires || show_org;
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
      if show_org { let _ = writeln!( out, "  Org:     N/A" ); }
      if idx < last_idx { out.push( '\n' ); }
    }
  }
  out
}

/// `.accounts` — list all saved accounts with field-presence control.
///
/// Without `name::`: lists every account in the credential store as an indented
/// key-value block, with a blank line between accounts when any field is shown.
/// With `name::EMAIL`: shows that single account's block only.
/// Field-presence params (`active`, `sub`, `tier`, `expires`, `org`) are all default-on.
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
  let credential_store = match require_credential_store()
  {
    Ok( path ) => path,
    Err( _ )   =>
    {
      let content = match opts.format
      {
        OutputFormat::Json => "[]\n".to_string(),
        OutputFormat::Text => "(no accounts configured)\n".to_string(),
      };
      return Ok( OutputData::new( content, "text" ) );
    }
  };

  let name_arg = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };

  if !name_arg.is_empty()
  {
    crate::account::validate_name( &name_arg )
      .map_err( |e| io_err_to_error_data( &e, "accounts" ) )?;
  }

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

  let show_active  = matches!( cmd.arguments.get( "active"  ), Some( Value::Boolean( true ) ) | None );
  let show_sub     = matches!( cmd.arguments.get( "sub"     ), Some( Value::Boolean( true ) ) | None );
  let show_tier    = matches!( cmd.arguments.get( "tier"    ), Some( Value::Boolean( true ) ) | None );
  let show_expires = matches!( cmd.arguments.get( "expires" ), Some( Value::Boolean( true ) ) | None );
  let show_org     = matches!( cmd.arguments.get( "org"     ), Some( Value::Boolean( true ) ) | None );
  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      if accounts.is_empty()
      {
        "[]\n".to_string()
      }
      else
      {
        let entries : Vec< String > = accounts.iter().map( |a|
        {
          format!(
            "{{\"name\":\"{}\",\"is_active\":{},\"subscription_type\":\"{}\",\"rate_limit_tier\":\"{}\",\"expires_at_ms\":{},\"org\":\"N/A\"}}",
            json_escape( &a.name ),
            a.is_active,
            json_escape( &a.subscription_type ),
            json_escape( &a.rate_limit_tier ),
            a.expires_at_ms,
          )
        } ).collect();
        format!( "[{}]\n", entries.join( "," ) )
      }
    }
    OutputFormat::Text =>
    {
      render_accounts_text( &accounts, show_active, show_sub, show_tier, show_expires, show_org )
    }
  };
  Ok( OutputData::new( content, "text" ) )
}

/// `.account.switch` — atomic credential rotation by name.
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the target account does not exist.
#[ inline ]
pub fn account_switch_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Fix(issue-switch-dry-validation):
  // Root cause: is_dry() was checked before existence validation, so dry-run silently
  //   succeeded for non-existent accounts instead of reporting NotFound (exit 2).
  // Pitfall: Always run input validation + precondition checks before the dry-run guard;
  //   only the mutating operation (file copy + marker write) is skipped in dry-run.
  let name             = require_nonempty_string_arg( &cmd, "name" )?;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account switch" ) )?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would switch to '{name}'\n" ), "text" ) );
  }

  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account switch" ) )?;
  Ok( OutputData::new( format!( "switched to '{name}'\n" ), "text" ) )
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

/// Format rate-limit data as compact text (`v::0`): bare percentages, no labels or reset times.
fn format_rate_limits_compact( data : &RateLimitData ) -> String
{
  let pct_session = format!( "{:.0}", data.utilization_5h * 100.0 );
  let pct_weekly  = format!( "{:.0}", data.utilization_7d * 100.0 );
  let status      = &data.status;
  format!( "{pct_session}%\n{pct_weekly}%\n{status}\n" )
}

/// Format rate-limit data as human-readable text (`v::1` default): labelled with reset durations.
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

/// Format rate-limit data as verbose text (`v::2`): all fields including raw floats and timestamps.
fn format_rate_limits_verbose( data : &RateLimitData ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  let reset_session_str = format_duration_secs( data.reset_5h.saturating_sub( now_secs ) );
  let reset_weekly_str  = format_duration_secs( data.reset_7d.saturating_sub( now_secs ) );
  let pct_session       = format!( "{:.0}", data.utilization_5h * 100.0 );
  let pct_weekly        = format!( "{:.0}", data.utilization_7d * 100.0 );
  let raw_session       = data.utilization_5h;
  let raw_weekly        = data.utilization_7d;
  let ts_session        = data.reset_5h;
  let ts_weekly         = data.reset_7d;
  let status            = &data.status;
  format!(
    "Session (5h):  {pct_session}% consumed, resets in {reset_session_str}\n  raw: {raw_session:.6}, reset_ts: {ts_session}\nWeekly (7d):   {pct_weekly}% consumed, resets in {reset_weekly_str}\n  raw: {raw_weekly:.6}, reset_ts: {ts_weekly}\nStatus:        {status}\n"
  )
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
/// Output format uses a two-level dispatch: outer on `opts.format` (`json` vs `text`),
/// inner on `opts.verbosity` (only within `text`): `0`=compact, `2`=verbose, `_`=default.
///
/// # Pitfall
///
/// The inner verbosity match (`0`/`2`/`_`) is SEPARATE from the outer format match.
/// If only the outer match exists, all text verbosity levels silently fall back to `v::1`
/// output. Both dispatches are required; `v::0` and `v::2` have automated live tests
/// (`lim_it2`, `lim_it5`) to catch regressions.
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
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;

  let name_arg = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };

  let creds_path = if name_arg.is_empty()
  {
    require_active_credentials( &paths )?
  }
  else
  {
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
    OutputFormat::Json => format_rate_limits_json( &data ),
    OutputFormat::Text => match opts.verbosity
    {
      0 => format_rate_limits_compact( &data ),
      2 => format_rate_limits_verbose( &data ),
      _ => format_rate_limits_text( &data ),
    },
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
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the credential copy fails.
#[ inline ]
pub fn account_save_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let name             = require_nonempty_string_arg( &cmd, "name" )?;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would save current credentials as '{name}'\n" ), "text" ) );
  }

  crate::account::save( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account save" ) )?;
  Ok( OutputData::new( format!( "saved current credentials as '{name}'\n" ), "text" ) )
}

/// `.account.delete` — delete a saved account (guard: refuses active).
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// the account is active, or the account does not exist.
#[ inline ]
pub fn account_delete_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Fix(issue-delete-dry-validation):
  // Root cause: is_dry() was checked before active-account guard and existence check,
  //   so dry-run bypassed PermissionDenied (active account) and NotFound (missing account).
  // Pitfall: The active-account safety invariant must hold even in dry-run — reporting
  //   "would delete active account" without error is a misleading no-op.
  let name             = require_nonempty_string_arg( &cmd, "name" )?;
  let credential_store = require_credential_store()?;
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
  let opts = OutputOptions::from_cmd( &cmd )?;
  require_claude_paths()?;

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
      match ( &token_result, opts.verbosity )
      {
        ( crate::token::TokenStatus::Valid { .. }, 0 ) =>
          "valid\n".to_string(),
        ( crate::token::TokenStatus::Valid { expires_in }, 1 ) =>
          format!( "valid — {}m remaining\n", expires_in.as_secs() / 60 ),
        ( crate::token::TokenStatus::Valid { expires_in }, _ ) =>
          format!( "valid — {}s remaining (threshold={}s)\n", expires_in.as_secs(), threshold_secs ),
        ( crate::token::TokenStatus::ExpiringSoon { .. }, 0 ) =>
          "expiring soon\n".to_string(),
        ( crate::token::TokenStatus::ExpiringSoon { expires_in }, 1 ) =>
          format!( "expiring soon — {}m remaining\n", expires_in.as_secs() / 60 ),
        ( crate::token::TokenStatus::ExpiringSoon { expires_in }, _ ) =>
          format!( "expiring soon — {}s remaining (threshold={}s)\n", expires_in.as_secs(), threshold_secs ),
        ( crate::token::TokenStatus::Expired, _ ) =>
          "expired\n".to_string(),
      }
    }
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
  let opts             = OutputOptions::from_cmd( &cmd )?;
  let paths            = require_claude_paths()?;
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
      match opts.verbosity
      {
        0 =>
        {
          format!( "{}\n", paths.base().display() )
        }
        1 =>
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
        _ =>
        {
          let entries : Vec< ( &str, std::path::PathBuf ) > = vec![
            ( "credentials",      paths.credentials_file() ),
            ( "credential_store", credential_store ),
            ( "projects",         paths.projects_dir() ),
            ( "stats",            paths.stats_file() ),
            ( "settings",         paths.settings_file() ),
            ( "session-env",      paths.session_env_dir() ),
            ( "sessions",         paths.sessions_dir() ),
          ];
          let mut out = String::new();
          for ( label, path ) in entries
          {
            let exists = if path.exists() { "exists" } else { "absent" };
            let _ = writeln!( out, "{label}: {} [{exists}]", path.display() );
          }
          out
        }
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}
