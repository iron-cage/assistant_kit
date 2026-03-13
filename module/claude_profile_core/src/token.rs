//! Active OAuth token expiry status detection.
//!
//! Reads `expiresAt` from `~/.claude/.credentials.json` and classifies the
//! token as [`TokenStatus::Valid`], [`TokenStatus::ExpiringSoon`], or
//! [`TokenStatus::Expired`].
//!
//! # Token vs Subscription Window
//!
//! `expiresAt` reflects the **OAuth access token** expiry — typically refreshed
//! automatically by Claude Code. It does **not** reflect the server-side 5-hour
//! subscription usage window, which is not locally observable.
//!
//! # Examples
//!
//! ```no_run
//! use claude_profile_core::token;
//!
//! match token::status().expect( "failed to read credentials" )
//! {
//!   token::TokenStatus::Valid { expires_in } =>
//!     println!( "valid — {}m remaining", expires_in.as_secs() / 60 ),
//!   token::TokenStatus::ExpiringSoon { expires_in } =>
//!     eprintln!( "expires in {}m — consider switching accounts", expires_in.as_secs() / 60 ),
//!   token::TokenStatus::Expired =>
//!     eprintln!( "token expired — run: claude auth login" ),
//! }
//! ```

use core::time::Duration;
use std::time::{ SystemTime, UNIX_EPOCH };
use claude_common::ClaudePaths;

/// Default warning threshold in seconds before expiry to report
/// [`TokenStatus::ExpiringSoon`] instead of [`TokenStatus::Valid`].
pub const WARNING_THRESHOLD_SECS : u64 = 3600; // 60 minutes

/// Classification of the active OAuth access token.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum TokenStatus
{
  /// Token is valid with more than the warning threshold remaining.
  Valid
  {
    /// Time until token expiry.
    expires_in : Duration,
  },
  /// Token is valid but within the warning threshold of expiry.
  ExpiringSoon
  {
    /// Time until token expiry.
    expires_in : Duration,
  },
  /// Token has expired.
  Expired,
}

/// Read the active token status from `~/.claude/.credentials.json`.
///
/// Uses [`WARNING_THRESHOLD_SECS`] (60 minutes) as the `ExpiringSoon` threshold.
///
/// # Errors
///
/// Returns an error if `HOME` is not set, the credentials file is missing,
/// unreadable, or does not contain a parseable `expiresAt` field.
///
/// # Examples
///
/// ```no_run
/// use claude_profile_core::token;
///
/// let status = token::status().expect( "failed to read credentials" );
/// println!( "{status:?}" );
/// ```
#[ inline ]
pub fn status() -> Result< TokenStatus, std::io::Error >
{
  status_with_threshold( WARNING_THRESHOLD_SECS )
}

/// Read token status with a custom warning threshold (seconds before expiry).
///
/// # Errors
///
/// Returns an error if credentials cannot be read or `expiresAt` cannot
/// be parsed from the credential file.
#[ inline ]
pub fn status_with_threshold( warning_secs : u64 ) -> Result< TokenStatus, std::io::Error >
{
  let paths = ClaudePaths::new()
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "HOME environment variable not set",
    ) )?;

  let content = std::fs::read_to_string( paths.credentials_file() )
    .map_err( | e | std::io::Error::new(
      e.kind(),
      format!( "failed to read credentials file: {e}" ),
    ) )?;

  let expires_at_ms = parse_expires_at( &content )
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      "credentials file missing or unparseable 'expiresAt' field",
    ) )?;

  let now_ms = u64::try_from(
    SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .unwrap_or_default()
      .as_millis()
  ).unwrap_or( u64::MAX );

  if now_ms >= expires_at_ms
  {
    return Ok( TokenStatus::Expired );
  }

  let remaining = Duration::from_millis( expires_at_ms - now_ms );
  if remaining.as_secs() <= warning_secs
  {
    Ok( TokenStatus::ExpiringSoon { expires_in : remaining } )
  }
  else
  {
    Ok( TokenStatus::Valid { expires_in : remaining } )
  }
}

/// Extract the `expiresAt` integer value from credentials JSON.
///
/// Zero-dependency parser: locates the `"expiresAt":` key and reads the
/// following digit sequence. Handles optional whitespace after the colon.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_expires_at( json : &str ) -> Option< u64 >
{
  let key = "\"expiresAt\":";
  let colon_end = json.find( key )? + key.len();
  let rest = json[ colon_end.. ].trim_start();
  let end = rest
    .find( | c : char | !c.is_ascii_digit() )
    .unwrap_or( rest.len() );
  if end == 0 { return None; }
  rest[ ..end ].parse().ok()
}
