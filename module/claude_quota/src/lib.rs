//! `claude_quota` — Anthropic API rate-limit HTTP transport.
//!
//! Provides [`RateLimitData`] and [`QuotaError`] as library types (always available),
//! and [`fetch_rate_limits`] gated behind the `enabled` feature.
//!
//! # Feature Flags
//!
//! | Feature   | Adds                         | Extra dep |
//! |-----------|------------------------------|-----------|
//! | (none)    | `RateLimitData`, `QuotaError` | —         |
//! | `enabled` | `fetch_rate_limits(token)`   | `ureq`    |
//!
//! # Testability
//!
//! [`parse_headers`] accepts `Fn(&str) -> Option<&str>` so unit tests pass a
//! `HashMap`-backed closure — no live network, no `ureq` in dev-dependencies.

use std::fmt;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Anthropic messages endpoint used for quota checks.
pub const API_URL : &str = "https://api.anthropic.com/v1/messages";

/// OAuth beta header value — must match the Claude binary's OAuth implementation.
///
/// # Pitfall
///
/// This string is not documented in public Anthropic API docs; it was discovered
/// via `strings $(which claude)`. If live tests fail with "OAuth authentication is
/// currently not supported", the Claude binary was updated. Re-run
/// `strings $(which claude) | grep oauth` to find the new value.
pub const ANTHROPIC_BETA    : &str = "oauth-2025-04-20";

/// Anthropic API version header value.
pub const ANTHROPIC_VERSION : &str = "2023-06-01";

// ── RateLimitData ─────────────────────────────────────────────────────────────

/// Rate-limit utilization data parsed from Anthropic API response headers.
#[ derive( Debug ) ]
pub struct RateLimitData
{
  /// 5-hour session window utilization (0.0–1.0).
  pub utilization_5h : f64,
  /// 5-hour session window reset time (Unix timestamp, seconds).
  pub reset_5h       : u64,
  /// 7-day all-model utilization (0.0–1.0).
  pub utilization_7d : f64,
  /// 7-day all-model reset time (Unix timestamp, seconds).
  pub reset_7d       : u64,
  /// Rate-limit status: `allowed`, `allowed_warning`, or `rejected`.
  pub status         : String,
}

// ── QuotaError ────────────────────────────────────────────────────────────────

/// Errors produced by the quota HTTP transport.
#[ derive( Debug ) ]
pub enum QuotaError
{
  /// HTTP transport failure (network error, TLS error, etc.).
  HttpTransport( String ),
  /// A required rate-limit header was absent from the API response.
  MissingHeader( String ),
  /// A required rate-limit header was present but could not be parsed.
  MalformedHeader( String ),
}

impl fmt::Display for QuotaError
{
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::HttpTransport( msg ) =>
        write!( f, "HTTP transport error: {msg}" ),
      Self::MissingHeader( name ) =>
        write!( f, "rate-limit header missing: {name}" ),
      Self::MalformedHeader( ctx ) =>
        write!( f, "rate-limit header malformed: {ctx}" ),
    }
  }
}

impl std::error::Error for QuotaError {}

// ── parse_headers ─────────────────────────────────────────────────────────────

/// Parse rate-limit utilization headers using a closure-based header accessor.
///
/// Accepts `Fn(&str) -> Option<String>` (owned) so callers can pass either a live
/// `|name| resp.header(name).map(str::to_string)` or a test `HashMap`-backed
/// closure — no network access required for unit tests.
///
/// The owned-return design avoids lifetime coupling between the header-accessor
/// return value and either the header-name input or the live HTTP response object.
///
/// # Errors
///
/// Returns [`QuotaError::MissingHeader`] if a required header is absent, or
/// [`QuotaError::MalformedHeader`] if a present header cannot be parsed.
pub fn parse_headers< F >( get : F ) -> Result< RateLimitData, QuotaError >
where
  F : Fn( &str ) -> Option< String >,
{
  let require = |name : &str| -> Result< String, QuotaError >
  {
    get( name ).ok_or_else( || QuotaError::MissingHeader( name.to_string() ) )
  };

  let s_5h_util   = require( "anthropic-ratelimit-unified-5h-utilization" )?;
  let s_5h_reset  = require( "anthropic-ratelimit-unified-5h-reset" )?;
  let s_7d_util   = require( "anthropic-ratelimit-unified-7d-utilization" )?;
  let s_7d_reset  = require( "anthropic-ratelimit-unified-7d-reset" )?;
  let status      = require( "anthropic-ratelimit-unified-status" )?;

  let utilization_5h = s_5h_util.parse::< f64 >().map_err( |e|
    QuotaError::MalformedHeader( format!( "5h-utilization: {e}" ) )
  )?;
  let reset_5h = s_5h_reset.parse::< u64 >().map_err( |e|
    QuotaError::MalformedHeader( format!( "5h-reset: {e}" ) )
  )?;
  let utilization_7d = s_7d_util.parse::< f64 >().map_err( |e|
    QuotaError::MalformedHeader( format!( "7d-utilization: {e}" ) )
  )?;
  let reset_7d = s_7d_reset.parse::< u64 >().map_err( |e|
    QuotaError::MalformedHeader( format!( "7d-reset: {e}" ) )
  )?;

  Ok( RateLimitData
  {
    utilization_5h,
    reset_5h,
    utilization_7d,
    reset_7d,
    status : status.to_string(),
  } )
}

// ── fetch_rate_limits ─────────────────────────────────────────────────────────

/// Fetch rate-limit utilization data from the Anthropic API.
///
/// Makes a lightweight `POST /v1/messages` (`max_tokens: 1`) using the provided
/// OAuth access token. Rate-limit headers are returned on **all** responses,
/// including HTTP error codes — the `Ok(r) | Err(ureq::Error::Status(_, r))`
/// pattern extracts the response body from both success and error variants.
///
/// # Fix(issue-oauth-beta-stale)
///
/// Root cause: the `anthropic-beta` value `oauth-2023-09-22` was stale; the API
/// rejected it with 401 ("OAuth authentication is currently not supported"), so
/// rate-limit headers were never returned.
/// Pitfall: the beta string is not in public docs — confirm via
/// `strings $(which claude) | grep oauth` whenever Claude Code updates.
///
/// # Errors
///
/// Returns [`QuotaError::HttpTransport`] on network failure, or parsing errors
/// from [`parse_headers`] if required headers are absent or malformed.
#[ cfg( feature = "enabled" ) ]
pub fn fetch_rate_limits( token : &str ) -> Result< RateLimitData, QuotaError >
{
  let body = r#"{"model":"claude-haiku-4-5-20251001","max_tokens":1,"messages":[{"role":"user","content":"quota"}]}"#;

  let req_result = ureq::post( API_URL )
    .set( "Authorization",    &format!( "Bearer {token}" ) )
    .set( "anthropic-beta",   ANTHROPIC_BETA )
    .set( "anthropic-version", ANTHROPIC_VERSION )
    .set( "Content-Type",     "application/json" )
    .send_string( body );

  // Rate-limit headers are present on ALL responses, including HTTP error codes.
  // The Ok(r) | Err(ureq::Error::Status(_, r)) pattern extracts the response
  // body from both success (2xx) and HTTP-error (4xx/5xx) variants.
  let resp = match req_result
  {
    Ok( r ) | Err( ureq::Error::Status( _, r ) ) => r,
    Err( e ) => return Err( QuotaError::HttpTransport( e.to_string() ) ),
  };

  parse_headers( |name| resp.header( name ).map( str::to_string ) )
}
