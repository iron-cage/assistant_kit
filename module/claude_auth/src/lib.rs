//! `claude_auth` — Anthropic OAuth token refresh transport.
//!
//! Layer `*` standalone primitive: zero workspace dependencies.
//!
//! ## Feature Flags
//!
//! | Feature   | Adds                          | Extra dep  |
//! |-----------|-------------------------------|------------|
//! | (none)    | `TokenRefreshResult`, `AuthError`, `parse_response` | — |
//! | `enabled` | `refresh_token`               | `ureq ~2`  |

use std::fmt;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Anthropic OAuth token refresh endpoint.
pub const TOKEN_URL : &str = "https://platform.claude.com/v1/oauth/token";

/// Public OAuth client ID for the Claude desktop application.
pub const CLIENT_ID : &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

// ── Types ─────────────────────────────────────────────────────────────────────

/// Result of a successful OAuth token refresh.
#[ derive( Debug ) ]
pub struct TokenRefreshResult
{
  /// Fresh Bearer access token.
  pub access_token  : String,
  /// New refresh token (rotated on every refresh).
  pub refresh_token : String,
  /// Absolute expiry in milliseconds since Unix epoch.
  /// Computed as `now_ms + expires_in_secs * 1000`.
  pub expires_at_ms : u64,
}

/// Errors returned by `claude_auth` operations.
#[ derive( Debug ) ]
pub enum AuthError
{
  /// HTTP-level transport failure (connection refused, TLS error, etc.).
  HttpTransport( String ),
  /// Response JSON was valid UTF-8 but a required field was absent or malformed.
  /// The inner `String` names the field (e.g. `"access_token"`).
  ResponseParse( String ),
  /// The server returned HTTP 429 — caller should back off before retrying.
  RateLimited,
}

impl fmt::Display for AuthError
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::HttpTransport( msg ) => write!( f, "HTTP transport error: {msg}" ),
      Self::ResponseParse( field ) =>
        write!( f, "response parse error: missing or malformed field '{field}'" ),
      Self::RateLimited => write!( f, "rate limited (429): back off before retrying" ),
    }
  }
}

impl std::error::Error for AuthError {}

// ── Parsing ───────────────────────────────────────────────────────────────────

/// Parse an OAuth token-refresh JSON response body.
///
/// `now_ms` is the current time in milliseconds since Unix epoch; it is used
/// to compute the absolute `expires_at_ms` value.
///
/// Does **not** require `serde` or any external parser — operates on raw string
/// needles so it is always available regardless of feature flags.
///
/// # Errors
///
/// Returns [`AuthError::ResponseParse`] if any required field is absent or
/// cannot be interpreted as the expected type.
#[ inline ]
pub fn parse_response( body : &str, now_ms : u64 ) -> Result< TokenRefreshResult, AuthError >
{
  let access_token  = parse_string_field( body, "access_token" )?;
  let refresh_token = parse_string_field( body, "refresh_token" )?;
  let expires_in    = parse_u64_field( body, "expires_in" )?;
  Ok
  (
    TokenRefreshResult
    {
      access_token,
      refresh_token,
      expires_at_ms : now_ms + expires_in * 1000,
    }
  )
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Extract a JSON string value for `key` from a flat JSON object body.
///
/// Uses needle `"\"key\":"` (colon included) to avoid prefix-collision between
/// keys like `"token"` and `"access_token"`.
fn parse_string_field( body : &str, key : &str ) -> Result< String, AuthError >
{
  let needle     = format!( "\"{}\":", key );
  let after_key  = body
    .find( needle.as_str() )
    .map( | pos | &body[ pos + needle.len() .. ] )
    .ok_or_else( || AuthError::ResponseParse( key.to_string() ) )?;

  let after_colon = after_key.trim_start();

  if !after_colon.starts_with( '"' )
  {
    return Err( AuthError::ResponseParse( key.to_string() ) );
  }

  let inner = &after_colon[ 1 .. ];
  let end   = inner
    .find( '"' )
    .ok_or_else( || AuthError::ResponseParse( key.to_string() ) )?;

  Ok( inner[ .. end ].to_string() )
}

/// Extract a JSON unsigned-integer value for `key` from a flat JSON object body.
///
/// Returns `Err(ResponseParse)` if the value starts with `"` (string, not
/// integer) or if the digit sequence cannot be parsed as [`u64`].
fn parse_u64_field( body : &str, key : &str ) -> Result< u64, AuthError >
{
  let needle     = format!( "\"{}\":", key );
  let after_key  = body
    .find( needle.as_str() )
    .map( | pos | &body[ pos + needle.len() .. ] )
    .ok_or_else( || AuthError::ResponseParse( key.to_string() ) )?;

  let after_colon = after_key.trim_start();

  // Reject JSON strings (value starts with `"`)
  if after_colon.starts_with( '"' )
  {
    return Err( AuthError::ResponseParse( key.to_string() ) );
  }

  // Collect leading ASCII digits
  let digits : &str = after_colon
    .find( | c : char | !c.is_ascii_digit() )
    .map( | end | &after_colon[ .. end ] )
    .unwrap_or( after_colon );

  digits
    .parse::< u64 >()
    .map_err( | _ | AuthError::ResponseParse( key.to_string() ) )
}

// ── Network transport (feature = "enabled") ───────────────────────────────────

/// Exchange a refresh token for a new access token via Anthropic's OAuth endpoint.
///
/// Performs a blocking HTTP POST to [`TOKEN_URL`].
///
/// # Errors
///
/// * [`AuthError::RateLimited`] — server returned HTTP 429.
/// * [`AuthError::HttpTransport`] — network or TLS failure.
/// * [`AuthError::ResponseParse`] — unexpected response body.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn refresh_token( refresh_tok : &str, scope : &str ) -> Result< TokenRefreshResult, AuthError >
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  let body = format!(
    r#"{{"grant_type":"refresh_token","refresh_token":"{refresh_tok}","client_id":"{CLIENT_ID}","scope":"{scope}"}}"#
  );

  let response = ureq::post( TOKEN_URL )
    .set( "Content-Type", "application/json" )
    .send_string( &body );

  match response
  {
    Err( ureq::Error::Status( 429, _ ) ) => Err( AuthError::RateLimited ),
    Err( ureq::Error::Status( _, resp ) ) =>
    {
      let status_text = resp.status_text().to_string();
      Err( AuthError::HttpTransport( status_text ) )
    },
    Err( ureq::Error::Transport( t ) ) =>
      Err( AuthError::HttpTransport( t.to_string() ) ),
    Ok( resp ) =>
    {
      let text = resp
        .into_string()
        .map_err( | e | AuthError::HttpTransport( e.to_string() ) )?;
      let now_ms = SystemTime::now()
        .duration_since( UNIX_EPOCH )
        .map( | d | d.as_millis() as u64 )
        .unwrap_or( 0 );
      parse_response( &text, now_ms )
    },
  }
}
