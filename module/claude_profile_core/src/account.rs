//! Named credential storage and account rotation.
//!
//! # Account Store Layout
//!
//! ```text
//! ~/.claude/accounts/
//!   work.credentials.json       ← saved credential snapshot
//!   personal.credentials.json
//!   _active                     ← text: name of active account
//! ```
//!
//! # Examples
//!
//! ```no_run
//! use claude_profile_core::account;
//!
//! // List stored accounts
//! for acct in account::list().expect( "failed to list accounts" )
//! {
//!   let active = if acct.is_active { " ← active" } else { "" };
//!   println!( "{}{} ({})", acct.name, active, acct.subscription_type );
//! }
//!
//! // Save current credentials as "work"
//! account::save( "work" ).expect( "failed to save" );
//!
//! // Switch to "personal"
//! account::switch_account( "personal" ).expect( "failed to switch" );
//!
//! // Delete an old entry
//! account::delete( "old" ).expect( "failed to delete" );
//! ```

use std::path::Path;
use claude_common::ClaudePaths;

/// Metadata for a saved Claude Code account credential snapshot.
#[ derive( Debug, Clone ) ]
pub struct Account
{
  /// Account name — the filename stem in `~/.claude/accounts/`.
  pub name : String,
  /// Claude subscription type (e.g., `"max"`, `"pro"`).
  pub subscription_type : String,
  /// Rate limit tier identifier.
  pub rate_limit_tier : String,
  /// OAuth token expiry as Unix epoch milliseconds.
  pub expires_at_ms : u64,
  /// Whether this account's credentials are currently active.
  pub is_active : bool,
}

/// List all accounts in `~/.claude/accounts/`.
///
/// Returns an empty `Vec` if the account store does not exist yet — not an error.
///
/// # Errors
///
/// Returns an error if the accounts directory exists but cannot be read.
#[ inline ]
#[ must_use = "check the returned accounts list" ]
pub fn list() -> Result< Vec< Account >, std::io::Error >
{
  let paths = require_paths()?;
  let accounts_dir = paths.accounts_dir();
  if !accounts_dir.exists() { return Ok( Vec::new() ); }

  let active = read_active_marker( &accounts_dir );
  let mut accounts = Vec::new();

  for entry in std::fs::read_dir( &accounts_dir )?.flatten()
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

    accounts.push( Account { name, subscription_type, rate_limit_tier, expires_at_ms, is_active } );
  }

  accounts.sort_by( | a, b | a.name.cmp( &b.name ) );
  Ok( accounts )
}

/// Save the current `~/.claude/.credentials.json` as a named account.
///
/// Creates `~/.claude/accounts/{name}.credentials.json`. Overwrites if exists.
///
/// # Errors
///
/// Returns an error if the name is invalid, the credentials file cannot be
/// read, or the account store cannot be written.
#[ inline ]
pub fn save( name : &str ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  let paths = require_paths()?;
  let accounts_dir = paths.accounts_dir();
  std::fs::create_dir_all( &accounts_dir )?;
  let dest = accounts_dir.join( format!( "{name}.credentials.json" ) );
  std::fs::copy( paths.credentials_file(), dest )?;
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
/// Returns an I/O error if HOME is not set.
#[ inline ]
pub fn check_switch_preconditions( name : &str ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  let paths = require_paths()?;
  let src = paths.accounts_dir().join( format!( "{name}.credentials.json" ) );
  if !src.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", paths.accounts_dir().display() ),
    ) );
  }
  Ok( () )
}

/// Switch the active account by name.
///
/// Atomically overwrites `~/.claude/.credentials.json` with the named
/// account's credentials using write-then-rename, then updates
/// `~/.claude/accounts/_active`.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist, or an I/O error if
/// the switch cannot be completed.
#[ inline ]
pub fn switch_account( name : &str ) -> Result< (), std::io::Error >
{
  check_switch_preconditions( name )?;
  let paths = require_paths()?;
  let accounts_dir = paths.accounts_dir();
  let src = accounts_dir.join( format!( "{name}.credentials.json" ) );

  // Atomic write: copy to adjacent temp file, then rename into place.
  // Both files share the same ~/.claude/ directory, guaranteeing same filesystem.
  let creds = paths.credentials_file();
  let tmp = creds.with_extension( "json.tmp" );
  std::fs::copy( &src, &tmp )?;
  std::fs::rename( &tmp, &creds )?;

  // Update active marker after credentials are safely in place.
  let marker = accounts_dir.join( "_active" );
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
///
/// let switched_to = account::auto_rotate().expect( "rotation failed" );
/// println!( "rotated to: {switched_to}" );
/// ```
#[ inline ]
pub fn auto_rotate() -> Result< String, std::io::Error >
{
  let candidate = list()?
    .into_iter()
    .filter( | a | !a.is_active )
    .max_by_key( | a | a.expires_at_ms )
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "no inactive account available to rotate to",
    ) )?;

  let name = candidate.name;
  switch_account( &name )?;
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
/// Returns an I/O error if HOME is not set.
#[ inline ]
pub fn check_delete_preconditions( name : &str ) -> Result< (), std::io::Error >
{
  validate_name( name )?;
  let paths = require_paths()?;
  let accounts_dir = paths.accounts_dir();
  let active = read_active_marker( &accounts_dir );

  if active.as_deref() == Some( name )
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::PermissionDenied,
      format!( "cannot delete active account '{name}' — switch to another account first" ),
    ) );
  }

  let target = accounts_dir.join( format!( "{name}.credentials.json" ) );
  if !target.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", accounts_dir.display() ),
    ) );
  }

  Ok( () )
}

/// Delete a named account from `~/.claude/accounts/`.
///
/// # Errors
///
/// Returns `PermissionDenied` if the named account is currently active.
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn delete( name : &str ) -> Result< (), std::io::Error >
{
  check_delete_preconditions( name )?;
  let paths = require_paths()?;
  let accounts_dir = paths.accounts_dir();
  let target = accounts_dir.join( format!( "{name}.credentials.json" ) );
  std::fs::remove_file( target )?;
  Ok( () )
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn require_paths() -> Result< ClaudePaths, std::io::Error >
{
  ClaudePaths::new().ok_or_else( || std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "HOME environment variable not set",
  ) )
}

fn read_active_marker( accounts_dir : &Path ) -> Option< String >
{
  let marker = accounts_dir.join( "_active" );
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
  if name.is_empty()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "account name must not be empty",
    ) );
  }
  if name.chars().any( | c | matches!( c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' ) )
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' contains invalid characters" ),
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
