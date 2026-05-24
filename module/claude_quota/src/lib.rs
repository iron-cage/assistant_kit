//! `claude_quota` — Anthropic API quota HTTP transports.
//!
//! Provides rate-limit header types and OAuth usage endpoint types as library
//! types (always available), with network functions gated behind the `enabled` feature.
//!
//! # Feature Flags
//!
//! | Feature   | Adds                                                    | Extra dep |
//! |-----------|---------------------------------------------------------|-----------|
//! | (none)    | `RateLimitData`, `OauthUsageData`, `OauthAccountData`, `PeriodUsage`, `QuotaError` | — |
//! | (none)    | `parse_headers`, `parse_oauth_usage`, `parse_oauth_account`, `iso_to_unix_secs` | — |
//! | `enabled` | `fetch_rate_limits(token)`, `fetch_oauth_usage(token)`, `fetch_oauth_account(token)` | `ureq` |
//!
//! # Testability
//!
//! [`parse_headers`] accepts `Fn(&str) -> Option<String>` so unit tests pass a
//! `HashMap`-backed closure — no live network, no `ureq` in dev-dependencies.
//! [`parse_oauth_usage`] operates on a raw `&str` body for the same reason.

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
  /// The OAuth usage JSON response was absent or a required field was missing/malformed.
  /// The inner `String` names the missing or malformed field (e.g. `"utilization"`).
  ResponseParse( String ),
}

impl fmt::Display for QuotaError
{
  #[ inline ]
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
      Self::ResponseParse( field ) =>
        write!( f, "OAuth usage response parse error: missing or malformed field '{field}'" ),
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

// ── http_agent ───────────────────────────────────────────────────────────────

/// Build an HTTP agent with explicit read and connect timeouts.
///
/// # Fix(BUG-172)
///
/// Root cause: bare ureq convenience functions use the global agent whose
/// `timeout_read` defaults to `None` (indefinite), causing ~75–99s hangs when
/// a server TCP-connects but stalls the response body.
/// Pitfall: all new HTTP call sites must use this helper, not bare ureq calls.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
fn http_agent() -> ureq::Agent
{
  ureq::AgentBuilder::new()
    .timeout_read( std::time::Duration::from_secs( 10 ) )
    .timeout_connect( std::time::Duration::from_secs( 5 ) )
    .build()
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

  let req_result = http_agent().post( API_URL )
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

// ── OauthUsageData / PeriodUsage ──────────────────────────────────────────────

/// OAuth usage URL — GET endpoint returning per-period quota buckets.
pub const OAUTH_USAGE_URL : &str = "https://api.anthropic.com/api/oauth/usage";

/// Per-period quota bucket from the OAuth usage endpoint.
///
/// `utilization` is 0.0–100.0 (consumed percent).
/// `resets_at` is an ISO-8601 UTC string (may be `None` if the server returns `null`).
#[ derive( Debug ) ]
pub struct PeriodUsage
{
  /// Consumed quota as a percentage (0.0–100.0).
  pub utilization : f64,
  /// ISO-8601 UTC reset timestamp, e.g. `"2026-05-20T04:00:00+00:00"`. `None` when server returns `null`.
  pub resets_at   : Option< String >,
}

/// Response from `GET /api/oauth/usage` — three period buckets.
///
/// Each field is `None` when the server returns `null` (e.g. for non-subscription accounts).
#[ derive( Debug ) ]
pub struct OauthUsageData
{
  /// 5-hour session quota bucket.
  pub five_hour        : Option< PeriodUsage >,
  /// 7-day all-model quota bucket.
  pub seven_day        : Option< PeriodUsage >,
  /// 7-day Sonnet-only quota bucket.
  pub seven_day_sonnet : Option< PeriodUsage >,
}

// ── iso_to_unix_secs ──────────────────────────────────────────────────────────

/// Convert an ISO-8601 UTC timestamp to Unix seconds.
///
/// Parses `"YYYY-MM-DDTHH:MM:SS[.ffffff][+HH:MM|Z]"` using hand-rolled Gregorian
/// calendar arithmetic — no external dependencies.
///
/// Only the date part (`YYYY-MM-DD`) and the time part (`HH:MM:SS`) are used;
/// fractional seconds and UTC offset are ignored (offset is assumed to be `+00:00`
/// for quota-reset purposes — all Anthropic timestamps are UTC).
///
/// Returns `None` on any parse failure (wrong format, non-numeric fields, etc.).
#[ inline ]
pub fn iso_to_unix_secs( s : &str ) -> Option< u64 >
{
  // Require at least "YYYY-MM-DDTHH:MM:SS" (19 chars)
  if s.len() < 19 { return None; }

  // Split on 'T'
  let t_pos = s.find( 'T' )?;
  let date_part = &s[ ..t_pos ];
  let time_part = &s[ t_pos + 1 .. ];

  // Parse date: "YYYY-MM-DD"
  if date_part.len() < 10 { return None; }
  let year  = date_part[ 0..4 ].parse::< u64 >().ok()?;
  let month = date_part[ 5..7 ].parse::< u64 >().ok()?;
  let day   = date_part[ 8..10 ].parse::< u64 >().ok()?;
  if !( 1..=12 ).contains( &month ) || !( 1..=31 ).contains( &day ) { return None; }

  // Parse time: "HH:MM:SS" — ignore fractional seconds and timezone
  if time_part.len() < 8 { return None; }
  let hour = time_part[ 0..2 ].parse::< u64 >().ok()?;
  let min  = time_part[ 3..5 ].parse::< u64 >().ok()?;
  let sec  = time_part[ 6..8 ].parse::< u64 >().ok()?;
  if hour > 23 || min > 59 || sec > 59 { return None; }

  // Days from 1970-01-01 to YYYY-01-01
  let is_leap = |y : u64| ( y.is_multiple_of( 4 ) && !y.is_multiple_of( 100 ) ) || y.is_multiple_of( 400 );
  let mut days : u64 = 0;
  for y in 1970..year
  {
    days += if is_leap( y ) { 366 } else { 365 };
  }

  // Days for completed months in this year
  let days_in_month = [ 31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  for m in 1..month
  {
    let extra = if m == 2 && is_leap( year ) { 1 } else { 0 };
    days += days_in_month[ ( m - 1 ) as usize ] + extra;
  }

  days += day - 1;

  Some( days * 86_400 + hour * 3_600 + min * 60 + sec )
}

// ── parse_oauth_usage ─────────────────────────────────────────────────────────

/// Parse the body of `GET /api/oauth/usage` into [`OauthUsageData`].
///
/// Uses string-needle scanning (no `serde_json`) so it is always available
/// regardless of feature flags.
///
/// # Errors
///
/// Returns [`QuotaError::ResponseParse`] if the body does not contain the
/// expected top-level keys, or a present period object is missing the required
/// `"utilization"` field or contains a non-numeric value.
#[ inline ]
pub fn parse_oauth_usage( body : &str ) -> Result< OauthUsageData, QuotaError >
{
  // Body must contain at least one of the three period keys.
  // Invalid JSON (e.g. "not json") will fail to find any needle → ResponseParse.
  if !body.contains( "\"five_hour\"" )
    && !body.contains( "\"seven_day\"" )
    && !body.contains( "\"seven_day_sonnet\"" )
  {
    return Err( QuotaError::ResponseParse( "five_hour/seven_day/seven_day_sonnet".to_string() ) );
  }

  let five_hour        = parse_period( body, "five_hour" )?;
  let seven_day        = parse_period( body, "seven_day" )?;
  let seven_day_sonnet = parse_period( body, "seven_day_sonnet" )?;

  Ok( OauthUsageData { five_hour, seven_day, seven_day_sonnet } )
}

/// Extract a `{...}` object block from the start of `s` using brace counting.
///
/// `s` must start with `'{'`. Returns the slice `s[..end]` including both braces,
/// or `None` if the input doesn't start with `'{'` or has unmatched braces.
fn extract_object_block( s : &str ) -> Option< &str >
{
  if !s.starts_with( '{' ) { return None; }
  let mut depth = 0_i32;
  for ( i, c ) in s.char_indices()
  {
    match c
    {
      '{' => depth += 1,
      '}' =>
      {
        depth -= 1;
        if depth == 0 { return Some( &s[ ..i + 1 ] ); }
      }
      _ => {}
    }
  }
  None
}

/// Extract a single period bucket from the usage JSON body.
///
/// Finds `"key":` needle, inspects the value:
/// - `null` → `None`
/// - `{...}` block → parse `utilization` (required) and `resets_at` (optional)
///
/// Returns `Err(ResponseParse)` if `utilization` is missing or non-numeric,
/// or if the JSON structure is unexpected.
fn parse_period( body : &str, key : &str ) -> Result< Option< PeriodUsage >, QuotaError >
{
  let needle = format!( "\"{}\":", key );
  let after_key = body
    .find( needle.as_str() )
    .map( |pos| &body[ pos + needle.len() .. ] )
    .ok_or_else( || QuotaError::ResponseParse( key.to_string() ) )?;

  let value_start = after_key.trim_start();

  // null → None
  if value_start.starts_with( "null" )
  {
    return Ok( None );
  }

  // Must be an object starting with '{'
  if !value_start.starts_with( '{' )
  {
    return Err( QuotaError::ResponseParse( format!( "{key}: expected object or null" ) ) );
  }

  let block = extract_object_block( value_start )
    .ok_or_else( || QuotaError::ResponseParse( format!( "{key}: unclosed object" ) ) )?;

  // Parse `utilization` (required f64)
  let utilization = parse_f64_in_block( block, "utilization" )
    .ok_or_else( || QuotaError::ResponseParse( format!( "{key}.utilization" ) ) )?;

  // Parse `resets_at` (optional string; may be null)
  let resets_at = parse_optional_string_in_block( block, "resets_at" );

  Ok( Some( PeriodUsage { utilization, resets_at } ) )
}

/// Find and parse a `f64` value for `"key":` inside a JSON object fragment.
///
/// Returns `None` if the key is absent or the value is not a valid `f64`.
fn parse_f64_in_block( block : &str, key : &str ) -> Option< f64 >
{
  let needle     = format!( "\"{}\":", key );
  let after_key  = block.find( needle.as_str() ).map( |p| &block[ p + needle.len() .. ] )?;
  let value      = after_key.trim_start();

  // Reject string values (start with '"')
  if value.starts_with( '"' ) { return None; }

  // Collect leading numeric characters: digits, '.', '-', 'e', 'E', '+'
  let end = value
    .find( |c : char| !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+' )
    .unwrap_or( value.len() );
  value[ ..end ].parse::< f64 >().ok()
}

/// Find and parse an optional string value for `"key":` inside a JSON object fragment.
///
/// Returns `None` if the key is absent, the value is `null`, or parsing fails.
fn parse_optional_string_in_block( block : &str, key : &str ) -> Option< String >
{
  let needle    = format!( "\"{}\":", key );
  let after_key = block.find( needle.as_str() ).map( |p| &block[ p + needle.len() .. ] )?;
  let value     = after_key.trim_start();

  // null → None
  if value.starts_with( "null" ) { return None; }

  // String value — extract until closing '"' (simple, no escape handling needed for timestamps)
  if let Some( inner ) = value.strip_prefix( '"' )
  {
    let end = inner.find( '"' )?;
    return Some( inner[ ..end ].to_string() );
  }

  None
}

// ── fetch_oauth_usage ─────────────────────────────────────────────────────────

/// Fetch OAuth usage data from the Anthropic API.
///
/// Makes a `GET /api/oauth/usage` request using the provided OAuth access token.
///
/// # Errors
///
/// Returns [`QuotaError::HttpTransport`] on network failure, or
/// [`QuotaError::ResponseParse`] if the response body cannot be parsed.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn fetch_oauth_usage( token : &str ) -> Result< OauthUsageData, QuotaError >
{
  let resp = http_agent().get( OAUTH_USAGE_URL )
    .set( "Authorization", &format!( "Bearer {token}" ) )
    .call();

  let body = match resp
  {
    Ok( r ) => r.into_string().map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?,
    Err( ureq::Error::Status( _, r ) ) =>
    {
      return Err( QuotaError::HttpTransport( format!( "HTTP {}", r.status() ) ) );
    }
    Err( ureq::Error::Transport( t ) ) =>
    {
      return Err( QuotaError::HttpTransport( t.to_string() ) );
    }
  };

  parse_oauth_usage( &body )
}

// ── OauthAccountData ──────────────────────────────────────────────────────────

/// OAuth account URL — GET endpoint returning account identity and org membership.
pub const OAUTH_ACCOUNT_URL : &str = "https://api.anthropic.com/api/oauth/account";

/// Account identity and subscription state parsed from `GET /api/oauth/account`.
///
/// Only the billing-relevant fields from `memberships[0].organization` are captured.
/// The full response also contains user identity fields and large `settings` objects
/// not needed for quota display.
///
/// # Pitfall
///
/// `credentials.json` `subscriptionType` is written at OAuth-token-creation time and
/// goes stale after subscription changes. `billing_type` from this endpoint is the
/// authoritative current state — prefer it over the cached credential field.
#[ derive( Debug ) ]
pub struct OauthAccountData
{
  /// Current subscription status: `"stripe_subscription"` = active, `"none"` = cancelled.
  pub billing_type   : String,
  /// Whether the account has Claude Max capability (`"claude_max"` in org capabilities array).
  pub has_max        : bool,
  /// ISO-8601 UTC org creation timestamp — Stripe billing cycle anchor date.
  pub org_created_at : String,
}

/// Parse the body of `GET /api/oauth/account` into [`OauthAccountData`].
///
/// Locates `memberships[0].organization` by string-needle scanning and extracts
/// `billing_type`, `capabilities`, and `created_at`.
///
/// # Errors
///
/// Returns [`QuotaError::ResponseParse`] if `memberships`, `organization`,
/// `billing_type`, or `created_at` are absent or the object block is malformed.
pub fn parse_oauth_account( body : &str ) -> Result< OauthAccountData, QuotaError >
{
  let memberships_pos = body
    .find( "\"memberships\":" )
    .ok_or_else( || QuotaError::ResponseParse( "memberships".to_string() ) )?;

  let after_memberships = &body[ memberships_pos + "\"memberships\":".len() .. ];

  let org_pos = after_memberships
    .find( "\"organization\":" )
    .ok_or_else( || QuotaError::ResponseParse( "organization".to_string() ) )?;

  let after_org = after_memberships[ org_pos + "\"organization\":".len() .. ].trim_start();
  let org_block = extract_object_block( after_org )
    .ok_or_else( || QuotaError::ResponseParse( "organization: unclosed object".to_string() ) )?;

  let billing_type = parse_optional_string_in_block( org_block, "billing_type" )
    .ok_or_else( || QuotaError::ResponseParse( "organization.billing_type".to_string() ) )?;

  let org_created_at = parse_optional_string_in_block( org_block, "created_at" )
    .ok_or_else( || QuotaError::ResponseParse( "organization.created_at".to_string() ) )?;

  // capabilities is a JSON array — check for the literal string token
  let has_max = org_block.contains( "\"claude_max\"" );

  Ok( OauthAccountData { billing_type, has_max, org_created_at } )
}

/// Fetch account identity and subscription state from the Anthropic OAuth account endpoint.
///
/// Makes a `GET /api/oauth/account` request using the provided OAuth access token.
///
/// # Errors
///
/// Returns [`QuotaError::HttpTransport`] on network failure, or
/// [`QuotaError::ResponseParse`] if the response body cannot be parsed.
#[ cfg( feature = "enabled" ) ]
pub fn fetch_oauth_account( token : &str ) -> Result< OauthAccountData, QuotaError >
{
  let resp = http_agent().get( OAUTH_ACCOUNT_URL )
    .set( "Authorization",     &format!( "Bearer {token}" ) )
    .set( "anthropic-version", ANTHROPIC_VERSION )
    .call();

  let body = match resp
  {
    Ok( r ) => r.into_string().map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?,
    Err( ureq::Error::Status( _, r ) ) =>
    {
      return Err( QuotaError::HttpTransport( format!( "HTTP {}", r.status() ) ) );
    }
    Err( ureq::Error::Transport( t ) ) =>
    {
      return Err( QuotaError::HttpTransport( t.to_string() ) );
    }
  };

  parse_oauth_account( &body )
}

// ── ClaudeCliRolesData ────────────────────────────────────────────────────────

/// Claude CLI roles URL — GET endpoint returning org and workspace identity.
pub const CLAUDE_CLI_ROLES_URL : &str = "https://api.anthropic.com/api/oauth/claude_cli/roles";

/// Org identity snapshot parsed from `GET /api/oauth/claude_cli/roles`.
///
/// Personal accounts have empty `workspace_uuid` and `workspace_name` (API returns `null`).
/// Enterprise accounts have non-null workspace fields.
#[ derive( Debug ) ]
pub struct ClaudeCliRolesData
{
  /// Organisation UUID.
  pub organization_uuid : String,
  /// Organisation display name.
  pub organization_name : String,
  /// User's role within the organisation (e.g., `"admin"`, `"member"`).
  pub organization_role : String,
  /// Workspace UUID — empty string for personal accounts (API returns `null`).
  pub workspace_uuid    : String,
  /// Workspace display name — empty string for personal accounts (API returns `null`).
  pub workspace_name    : String,
}

/// Parse the body of `GET /api/oauth/claude_cli/roles` into [`ClaudeCliRolesData`].
///
/// Uses string-needle scanning (no `serde_json`) so it is always available
/// regardless of feature flags.
///
/// Nullable fields (`workspace_uuid`, `workspace_name`) become empty strings when
/// the server returns `null`.
///
/// # Errors
///
/// Returns [`QuotaError::ResponseParse`] if `organization_uuid` or `organization_name`
/// are absent or the body is not valid JSON.
pub fn parse_claude_cli_roles( body : &str ) -> Result< ClaudeCliRolesData, QuotaError >
{
  let organization_uuid = parse_optional_string_in_block( body, "organization_uuid" )
    .ok_or_else( || QuotaError::ResponseParse( "organization_uuid".to_string() ) )?;
  let organization_name = parse_optional_string_in_block( body, "organization_name" )
    .ok_or_else( || QuotaError::ResponseParse( "organization_name".to_string() ) )?;
  let organization_role = parse_optional_string_in_block( body, "organization_role" )
    .unwrap_or_default();
  let workspace_uuid    = parse_optional_string_in_block( body, "workspace_uuid" )
    .unwrap_or_default();
  let workspace_name    = parse_optional_string_in_block( body, "workspace_name" )
    .unwrap_or_default();

  Ok( ClaudeCliRolesData
  {
    organization_uuid,
    organization_name,
    organization_role,
    workspace_uuid,
    workspace_name,
  } )
}

/// Fetch org identity from the Claude CLI roles endpoint.
///
/// Makes a `GET /api/oauth/claude_cli/roles` request using the provided OAuth access token.
///
/// # Errors
///
/// Returns [`QuotaError::HttpTransport`] on network failure, or
/// [`QuotaError::ResponseParse`] if the response body cannot be parsed.
#[ cfg( feature = "enabled" ) ]
pub fn fetch_claude_cli_roles( token : &str ) -> Result< ClaudeCliRolesData, QuotaError >
{
  let resp = http_agent().get( CLAUDE_CLI_ROLES_URL )
    .set( "Authorization",     &format!( "Bearer {token}" ) )
    .set( "anthropic-version", ANTHROPIC_VERSION )
    .call();

  let body = match resp
  {
    Ok( r ) => r.into_string().map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?,
    Err( ureq::Error::Status( _, r ) ) =>
    {
      return Err( QuotaError::HttpTransport( format!( "HTTP {}", r.status() ) ) );
    }
    Err( ureq::Error::Transport( t ) ) =>
    {
      return Err( QuotaError::HttpTransport( t.to_string() ) );
    }
  };

  parse_claude_cli_roles( &body )
}
