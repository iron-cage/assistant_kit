//! Named credential storage and account rotation.
//!
//! # Account Store Layout
//!
//! ```text
//! $PRO/.persistent/claude/credential/
//!   alice@acme.com.credentials.json   ← saved credential snapshot
//!   alice@home.com.credentials.json
//!   _active                           ← text: name of active account
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use claude_profile_core::account;
//! use claude_core::ClaudePaths;
//! use std::path::Path;
//!
//! let paths = ClaudePaths::new().expect( "HOME must be set" );
//! let credential_store = Path::new( "/pro/.persistent/claude/credential" );
//!
//! // List stored accounts
//! for acct in account::list( credential_store ).expect( "failed to list accounts" )
//! {
//!   let active = if acct.is_active { " ← active" } else { "" };
//!   println!( "{}{} ({}) org={}", acct.name, active, acct.subscription_type, acct.org );
//! }
//!
//! // Save current credentials as "alice@acme.com"
//! account::save( "alice@acme.com", credential_store, &paths ).expect( "failed to save" );
//!
//! // Switch to "alice@home.com"
//! account::switch_account( "alice@home.com", credential_store, &paths ).expect( "failed to switch" );
//!
//! // Delete an old entry
//! account::delete( "alice@oldco.com", credential_store ).expect( "failed to delete" );
//! ```

use std::path::Path;
use claude_core::ClaudePaths;

/// Metadata for a saved Claude Code account credential snapshot.
#[ derive( Debug, Clone ) ]
pub struct Account
{
  /// Account name — the email address used as the credential filename stem.
  pub name : String,
  /// Claude subscription type (e.g., `"max"`, `"pro"`).
  pub subscription_type : String,
  /// Rate limit tier identifier.
  pub rate_limit_tier : String,
  /// OAuth token expiry as Unix epoch milliseconds.
  pub expires_at_ms : u64,
  /// Whether this account's credentials are currently active.
  pub is_active : bool,
  /// Organisation name from saved `{name}.claude.json` `oauthAccount.organizationName`.
  /// Empty string when snapshot absent or field missing.
  pub org : String,
  /// Display name from saved `{name}.claude.json` `oauthAccount.displayName`.
  /// Empty string when snapshot absent or field missing.
  pub display_name : String,
  /// Organisation role from saved `{name}.claude.json` `oauthAccount.organizationRole`.
  /// Empty string when snapshot absent or field missing.
  pub role : String,
  /// Billing type from saved `{name}.claude.json` `oauthAccount.billingType`.
  /// Empty string when snapshot absent or field missing.
  pub billing : String,
  /// Active model from saved `{name}.settings.json` `model` field.
  /// Empty string when snapshot absent or field missing.
  pub model : String,
}

/// List all accounts in `credential_store`.
///
/// Returns an empty `Vec` if the credential store does not exist yet — not an error.
///
/// # Errors
///
/// Returns an error if the credential store exists but cannot be read.
#[ inline ]
#[ must_use = "check the returned accounts list" ]
pub fn list( credential_store : &Path ) -> Result< Vec< Account >, std::io::Error >
{
  if !credential_store.exists() { return Ok( Vec::new() ); }

  let active = read_active_marker( credential_store );
  let mut accounts = Vec::new();

  for entry in std::fs::read_dir( credential_store )?.flatten()
  {
    let path = entry.path();
    let Some( name ) = credential_stem( &path ) else { continue };
    let content = std::fs::read_to_string( &path ).unwrap_or_default();
    let subscription_type = parse_string_field( &content, "subscriptionType" )
      .unwrap_or_default();
    let rate_limit_tier = parse_string_field( &content, "rateLimitTier" )
      .unwrap_or_default();
    let expires_at_ms = parse_u64_field( &content, "expiresAt" )
      .unwrap_or( 0 );
    let is_active = active.as_deref() == Some( name.as_str() );

    // Read per-account snapshot files written by save() — best-effort, empty when absent.
    let claude_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.claude.json" ) )
    ).unwrap_or_default();
    let settings_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.settings.json" ) )
    ).unwrap_or_default();
    let org          = parse_string_field( &claude_json, "organizationName" ).unwrap_or_default();
    let display_name = parse_string_field( &claude_json, "displayName"      ).unwrap_or_default();
    let role         = parse_string_field( &claude_json, "organizationRole" ).unwrap_or_default();
    let billing      = parse_string_field( &claude_json, "billingType"      ).unwrap_or_default();
    let model        = parse_string_field( &settings_json, "model"          ).unwrap_or_default();

    accounts.push( Account
    {
      name,
      subscription_type,
      rate_limit_tier,
      expires_at_ms,
      is_active,
      org,
      display_name,
      role,
      billing,
      model,
    } );
  }

  accounts.sort_by( | a, b | a.name.cmp( &b.name ) );
  Ok( accounts )
}

/// Save the current credentials as a named account in `credential_store`.
///
/// Creates `{credential_store}/{name}.credentials.json`. Overwrites if exists.
///
/// # Errors
///
/// Returns an error if the name is invalid, the credentials file cannot be
/// read, or the credential store cannot be written.
#[ inline ]
pub fn save( name : &str, credential_store : &Path, paths : &ClaudePaths ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  std::fs::create_dir_all( credential_store )?;
  let dest = credential_store.join( format!( "{name}.credentials.json" ) );
  std::fs::copy( paths.credentials_file(), dest )?;
  // Best-effort: snapshot ~/.claude.json and settings.json alongside credentials.
  // Missing source files are silently skipped — save() must not fail for absent optionals.
  let _ = std::fs::copy(
    paths.claude_json_file(),
    credential_store.join( format!( "{name}.claude.json" ) ),
  );
  let _ = std::fs::copy(
    paths.settings_file(),
    credential_store.join( format!( "{name}.settings.json" ) ),
  );
  Ok( () )
}

/// Validate that a named account can be switched to (name valid + file exists).
///
/// Called by both `switch_account` and the CLI dry-run path so that dry-run
/// reports the same errors as a live switch.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn check_switch_preconditions( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  let src = credential_store.join( format!( "{name}.credentials.json" ) );
  if !src.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }
  Ok( () )
}

/// Switch the active account by name.
///
/// Atomically overwrites the credentials file with the named account's
/// credentials using write-then-rename, then updates `{credential_store}/_active`.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist, or an I/O error if
/// the switch cannot be completed.
#[ inline ]
pub fn switch_account( name : &str, credential_store : &Path, paths : &ClaudePaths ) -> Result< (), std::io::Error >
{
  check_switch_preconditions( name, credential_store )?;
  let src = credential_store.join( format!( "{name}.credentials.json" ) );

  // Atomic write: copy to adjacent temp file, then rename into place.
  let creds = paths.credentials_file();
  let tmp = creds.with_extension( "json.tmp" );
  std::fs::copy( &src, &tmp )?;
  std::fs::rename( &tmp, &creds )?;

  // Update active marker after credentials are safely in place.
  let marker = credential_store.join( "_active" );
  std::fs::write( marker, name )?;

  Ok( () )
}

/// Automatically rotate to the best available inactive account.
///
/// Selects the inactive account with the highest `expires_at_ms` and switches
/// to it. Consolidates the pick-best-account-and-switch pattern so callers
/// need a single call instead of duplicating the selection loop.
///
/// Returns the name of the account switched to.
///
/// # Errors
///
/// Returns `NotFound` if no accounts are configured or if no inactive account
/// is available (all accounts are the currently active one).
///
/// # Examples
///
/// ```no_run
/// use claude_profile_core::account;
/// use claude_core::ClaudePaths;
/// use std::path::Path;
///
/// let credential_store = Path::new( "/pro/.persistent/claude/credential" );
/// let paths = ClaudePaths::new().expect( "HOME must be set" );
/// let switched_to = account::auto_rotate( credential_store, &paths ).expect( "rotation failed" );
/// println!( "rotated to: {switched_to}" );
/// ```
#[ inline ]
pub fn auto_rotate( credential_store : &Path, paths : &ClaudePaths ) -> Result< String, std::io::Error >
{
  let candidate = list( credential_store )?
    .into_iter()
    .filter( | a | !a.is_active )
    .max_by_key( | a | a.expires_at_ms )
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "no inactive account available to rotate to",
    ) )?;

  let name = candidate.name;
  switch_account( &name, credential_store, paths )?;
  Ok( name )
}

/// Validate that a named account can be deleted (name valid + not active + file exists).
///
/// Called by both `delete` and the CLI dry-run path so that dry-run
/// reports the same errors as a live delete.
///
/// # Errors
///
/// Returns `PermissionDenied` if the account is currently active.
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn check_delete_preconditions( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  let active = read_active_marker( credential_store );

  if active.as_deref() == Some( name )
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::PermissionDenied,
      format!( "cannot delete active account '{name}' — switch to another account first" ),
    ) );
  }

  let target = credential_store.join( format!( "{name}.credentials.json" ) );
  if !target.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }

  Ok( () )
}

/// Delete a named account from `credential_store`.
///
/// # Errors
///
/// Returns `PermissionDenied` if the named account is currently active.
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn delete( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  check_delete_preconditions( name, credential_store )?;
  let target = credential_store.join( format!( "{name}.credentials.json" ) );
  std::fs::remove_file( target )?;
  Ok( () )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn read_active_marker( credential_store : &Path ) -> Option< String >
{
  let marker = credential_store.join( "_active" );
  std::fs::read_to_string( marker )
    .ok()
    .map( | s | s.trim().to_string() )
}

/// Extract the account name from a `{name}.credentials.json` path.
///
/// Returns `None` for anything that is not a `*.credentials.json` file
/// (e.g. the `_active` marker or unrelated files).
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn credential_stem( path : &Path ) -> Option< String >
{
  let filename = path.file_name()?.to_str()?;
  filename
    .strip_suffix( ".credentials.json" )
    .map( std::string::ToString::to_string )
}

#[ doc( hidden ) ]
#[ inline ]
pub fn validate_name( name : &str ) -> Result< (), std::io::Error >
{
  // Account names must be valid email addresses (local@domain) so they can be
  // used as filenames and unambiguously identify the Claude account owner.
  let at = name.find( '@' ).ok_or_else( || std::io::Error::new(
    std::io::ErrorKind::InvalidInput,
    format!( "account name '{name}' is not a valid email address: must contain '@'" ),
  ) )?;
  if at == 0
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' is not a valid email address: local part must not be empty" ),
    ) );
  }
  if name[ at + 1.. ].is_empty()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' is not a valid email address: domain must not be empty" ),
    ) );
  }
  Ok( () )
}

/// Extract a quoted string field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon: both `"key":"val"` and
/// `"key": "val"` forms.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_string_field( json : &str, key : &str ) -> Option< String >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  if !rest.starts_with( '"' ) { return None; }
  let inner = &rest[ 1.. ];
  let end = inner.find( '"' )?;
  Some( inner[ ..end ].to_string() )
}

/// Extract an unsigned integer field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_u64_field( json : &str, key : &str ) -> Option< u64 >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  let end = rest
    .find( | c : char | !c.is_ascii_digit() )
    .unwrap_or( rest.len() );
  if end == 0 { return None; }
  rest[ ..end ].parse().ok()
}
