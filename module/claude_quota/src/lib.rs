//! `claude_quota` — Anthropic API quota HTTP transports.
//!
//! Provides rate-limit header types and OAuth usage endpoint types as library
//! types (always available), with network functions gated behind the `enabled` feature.
//!
//! # Feature Flags
//!
//! | Feature   | Adds                                                    | Extra dep |
//! |-----------|---------------------------------------------------------|-----------|
//! | (none)    | `RateLimitData`, `OauthUsageData`, `OauthAccountData`, `MembershipRaw`, `UserinfoData`, `ClaudeCliRolesData`, `PeriodUsage`, `QuotaError` | — |
//! | (none)    | `parse_headers`, `parse_oauth_usage`, `parse_oauth_account`, `parse_userinfo`, `parse_subscriptions`, `parse_claude_cli_roles`, `select_membership_index`, `iso_to_unix_secs` | — |
//! | `enabled` | `fetch_rate_limits(token)`, `fetch_oauth_usage(token)`, `fetch_oauth_account(token)`, `fetch_userinfo(token)`, `fetch_subscriptions(token)`, `fetch_claude_cli_roles(token)` | `ureq` |
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
#[ inline ]
#[ allow( clippy::similar_names ) ]
pub fn parse_headers< F >( get : F ) -> Result< RateLimitData, QuotaError >
where
  F : Fn( &str ) -> Option< String >,
{
  let require = |name : &str| -> Result< String, QuotaError >
  {
    get( name ).ok_or_else( || QuotaError::MissingHeader( name.to_string() ) )
  };

  let utilization_5h = require( "anthropic-ratelimit-unified-5h-utilization" )?
    .parse::< f64 >().map_err( |e|
      QuotaError::MalformedHeader( format!( "5h-utilization: {e}" ) )
    )?;
  let reset_5h = require( "anthropic-ratelimit-unified-5h-reset" )?
    .parse::< u64 >().map_err( |e|
      QuotaError::MalformedHeader( format!( "5h-reset: {e}" ) )
    )?;
  let utilization_7d = require( "anthropic-ratelimit-unified-7d-utilization" )?
    .parse::< f64 >().map_err( |e|
      QuotaError::MalformedHeader( format!( "7d-utilization: {e}" ) )
    )?;
  let reset_7d = require( "anthropic-ratelimit-unified-7d-reset" )?
    .parse::< u64 >().map_err( |e|
      QuotaError::MalformedHeader( format!( "7d-reset: {e}" ) )
    )?;
  let status = require( "anthropic-ratelimit-unified-status" )?;

  Ok( RateLimitData
  {
    utilization_5h,
    reset_5h,
    utilization_7d,
    reset_7d,
    status,
  } )
}

// ── http_agent ───────────────────────────────────────────────────────────────

/// Build an HTTP agent with explicit read and connect timeouts.
///
/// # Fix(BUG-172)
///
/// Root cause: bare ureq convenience functions use the global agent whose
/// `timeout_recv_body` defaults to `None` (indefinite), causing ~75–99s hangs when
/// a server TCP-connects but stalls the response body.
/// Pitfall: all new HTTP call sites must use this helper, not bare ureq calls.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
fn http_agent() -> ureq::Agent
{
  let config = ureq::Agent::config_builder()
    .timeout_recv_body( Some( core::time::Duration::from_secs( 10 ) ) )
    .timeout_connect( Some( core::time::Duration::from_secs( 5 ) ) )
    .http_status_as_error( false )
    .build();
  ureq::Agent::new_with_config( config )
}

// ── fetch_rate_limits ─────────────────────────────────────────────────────────

/// Fetch rate-limit utilization data from the Anthropic API.
///
/// Makes a lightweight `POST /v1/messages` (`max_tokens: 1`) using the provided
/// OAuth access token. Rate-limit headers are returned on **all** responses,
/// including HTTP error codes — `http_status_as_error(false)` on the agent
/// ensures 4xx/5xx responses return `Ok(resp)` so headers are always accessible.
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
#[ inline ]
pub fn fetch_rate_limits( token : &str ) -> Result< RateLimitData, QuotaError >
{
  let body = r#"{"model":"claude-haiku-4-5-20251001","max_tokens":1,"messages":[{"role":"user","content":"quota"}]}"#;

  let resp = http_agent()
    .post( API_URL )
    .header( "Authorization",     &format!( "Bearer {token}" ) )
    .header( "anthropic-beta",    ANTHROPIC_BETA )
    .header( "anthropic-version", ANTHROPIC_VERSION )
    .header( "Content-Type",      "application/json" )
    .send( body )
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  // Rate-limit headers are present on ALL responses, including HTTP error codes.
  // http_status_as_error(false) on the agent ensures 4xx/5xx responses return
  // Ok(resp) so headers are always accessible regardless of HTTP status.
  parse_headers( |name|
    resp.headers().get( name )
      .and_then( |v| v.to_str().ok() )
      .map( str::to_string )
  )
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
#[ must_use ]
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
  let is_leap = |y : u64| ( y % 4 == 0 && y % 100 != 0 ) || y % 400 == 0;
  let mut days : u64 = 0;
  for y in 1970..year
  {
    days += if is_leap( y ) { 366 } else { 365 };
  }

  // Days for completed months in this year
  let days_in_month = [ 31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  for m in 1..month
  {
    let extra = u64::from( m == 2 && is_leap( year ) );
    days += days_in_month[ usize::try_from( m - 1 ).unwrap_or( 0 ) ] + extra;
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
        if depth == 0 { return Some( &s[ ..=i ] ); }
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
  let needle = format!( "\"{key}\":" );
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
  let needle     = format!( "\"{key}\":" );
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
  let needle    = format!( "\"{key}\":" );
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
  let mut resp = http_agent()
    .get( OAUTH_USAGE_URL )
    .header( "Authorization", &format!( "Bearer {token}" ) )
    .call()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  let status = resp.status().as_u16();
  if status >= 400
  {
    return Err( QuotaError::HttpTransport( format!( "HTTP {status}" ) ) );
  }

  let body = resp
    .body_mut()
    .read_to_string()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  parse_oauth_usage( &body )
}

// ── OauthAccountData ──────────────────────────────────────────────────────────

/// OAuth account URL — GET endpoint returning account identity and org membership.
pub const OAUTH_ACCOUNT_URL : &str = "https://api.anthropic.com/api/oauth/account";

/// Account identity and subscription state parsed from `GET /api/oauth/account`.
///
/// Fields are populated from the priority-selected membership (see [`select_membership_index`]).
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

/// One membership entry parsed from `GET /api/oauth/account` or `GET /api/oauth/claude_cli/subscriptions`.
///
/// Used by [`select_membership_index`] to pick the billing-relevant membership,
/// and by `.account.inspect` to display all memberships with a selection indicator.
#[ derive( Debug ) ]
pub struct MembershipRaw
{
  /// Zero-based position in the `memberships` array.
  pub index          : usize,
  /// Billing status: `"stripe_subscription"` or `"none"`.
  pub billing_type   : String,
  /// Whether `"claude_max"` appears in this membership's org `capabilities` array.
  pub has_max        : bool,
  /// Raw capability strings from this membership's org `capabilities` array.
  pub capabilities   : Vec< String >,
  /// ISO-8601 UTC org creation timestamp (billing cycle anchor), or empty string if absent.
  pub org_created_at : String,
}

/// User identity parsed from `GET /api/oauth/userinfo`.
#[ derive( Debug ) ]
pub struct UserinfoData
{
  /// Stable user identifier (e.g. `"user_01ABCDEFGhijklmnopqrstuvwx"`).
  pub tagged_id     : String,
  /// User UUID.
  pub uuid          : String,
  /// Primary email address.
  pub email_address : String,
}

/// Return the index of the highest-priority membership in `memberships`.
///
/// Priority (descending):
/// 1. `billing_type == "stripe_subscription"` **and** `has_max == true`
/// 2. `billing_type == "stripe_subscription"` (any)
/// 3. `0` (index fallback — always valid because memberships is non-empty)
///
/// # Panics
///
/// Does not panic; returns `0` for an empty slice.
#[ inline ]
#[ must_use ]
pub fn select_membership_index( memberships : &[ MembershipRaw ] ) -> usize
{
  // Priority 1: stripe + max
  if let Some( m ) = memberships.iter().find( |m| m.billing_type == "stripe_subscription" && m.has_max )
  {
    return m.index;
  }
  // Priority 2: stripe any
  if let Some( m ) = memberships.iter().find( |m| m.billing_type == "stripe_subscription" )
  {
    return m.index;
  }
  // Priority 3: fallback to first
  0
}

/// Extract all string elements from a JSON array field inside a block.
///
/// Finds `"key": [...]` inside `block` and returns each quoted string token.
/// Returns an empty `Vec` if the key is absent or the array is empty.
fn parse_string_array( block : &str, key : &str ) -> Vec< String >
{
  let needle = format!( "\"{key}\":" );
  let Some( pos ) = block.find( needle.as_str() ) else { return vec![]; };
  let rest = block[ pos + needle.len() .. ].trim_start();
  let Some( arr_start ) = rest.find( '[' ) else { return vec![]; };
  let inner = &rest[ arr_start + 1 .. ];
  let Some( arr_end ) = inner.find( ']' ) else { return vec![]; };
  let array_content = &inner[ ..arr_end ];
  let mut caps = Vec::new();
  let mut scan = array_content;
  while let Some( start ) = scan.find( '"' )
  {
    scan = &scan[ start + 1 .. ];
    let Some( end ) = scan.find( '"' ) else { break; };
    let token = &scan[ ..end ];
    if !token.is_empty() { caps.push( token.to_string() ); }
    scan = &scan[ end + 1 .. ];
  }
  caps
}

/// Parse all membership objects from the `"memberships"` array of a JSON body.
///
/// Iterates membership `{...}` objects using brace-balanced scanning, extracts the
/// nested `"organization":` block from each, and returns one [`MembershipRaw`] per entry.
///
/// Returns `Err(ResponseParse)` if the `"memberships":` key or its opening `[` is absent,
/// or if no membership objects with an `"organization"` block are found.
fn parse_membership_list( body : &str ) -> Result< Vec< MembershipRaw >, QuotaError >
{
  let mem_label = "\"memberships\":";
  let mem_pos   = body
    .find( mem_label )
    .ok_or_else( || QuotaError::ResponseParse( "memberships".to_string() ) )?;
  let after_label = &body[ mem_pos + mem_label.len() .. ];
  let arr_offset  = after_label
    .find( '[' )
    .ok_or_else( || QuotaError::ResponseParse( "memberships: no array".to_string() ) )?;
  let array_body = &after_label[ arr_offset + 1 .. ];

  let mut memberships = Vec::new();
  let mut pos = 0_usize;
  let mut idx = 0_usize;

  loop
  {
    let rest = &array_body[ pos .. ];
    // Find next '{' or stop at ']' (end of memberships array)
    let close_pos = rest.find( ']' ).unwrap_or( rest.len() );
    let obj_pos   = match rest.find( '{' )
    {
      Some( p ) if p < close_pos => p,
      _                          => break,
    };
    let obj_slice     = &rest[ obj_pos .. ];
    let Some( membership_block ) = extract_object_block( obj_slice ) else { break };

    // Extract the nested "organization": { ... } block
    if let Some( org_label_pos ) = membership_block.find( "\"organization\":" )
    {
      let org_label   = "\"organization\":";
      let after_org   = membership_block[ org_label_pos + org_label.len() .. ].trim_start();
      if let Some( org_block ) = extract_object_block( after_org )
      {
        let billing_type   = parse_optional_string_in_block( org_block, "billing_type" )
          .unwrap_or_default();
        let has_max        = org_block.contains( "\"claude_max\"" );
        let org_created_at = parse_optional_string_in_block( org_block, "created_at" )
          .unwrap_or_default();
        let capabilities   = parse_string_array( org_block, "capabilities" );
        memberships.push( MembershipRaw { index: idx, billing_type, has_max, capabilities, org_created_at } );
      }
    }

    pos += obj_pos + membership_block.len();
    idx += 1;
  }

  if memberships.is_empty()
  {
    return Err( QuotaError::ResponseParse( "memberships: empty or missing organization".to_string() ) );
  }
  Ok( memberships )
}

/// Parse the body of `GET /api/oauth/account` into [`OauthAccountData`].
///
/// Iterates ALL membership objects using brace-balanced scanning, applies
/// [`select_membership_index`] to pick the billing-relevant entry, then extracts
/// `billing_type`, `has_max`, and `created_at` from the selected membership's
/// `organization` block.
///
/// # Errors
///
/// Returns [`QuotaError::ResponseParse`] if `memberships`, a valid `organization` block,
/// or `billing_type` are absent.
///
/// # Fix(BUG-237)
///
/// Previously used `str::find("\"organization\":")` on the full memberships string,
/// which always resolved to `memberships[0]`'s organization regardless of which
/// membership held the active subscription.
#[ inline ]
pub fn parse_oauth_account( body : &str ) -> Result< OauthAccountData, QuotaError >
{
  let memberships = parse_membership_list( body )?;
  let sel         = select_membership_index( &memberships );
  let m           = &memberships[ sel ];
  if m.billing_type.is_empty()
  {
    return Err( QuotaError::ResponseParse( "organization.billing_type".to_string() ) );
  }
  Ok( OauthAccountData
  {
    billing_type   : m.billing_type.clone(),
    has_max        : m.has_max,
    org_created_at : m.org_created_at.clone(),
  } )
}

/// Userinfo endpoint URL — returns the authenticated user's stable identity fields.
pub const OAUTH_USERINFO_URL : &str = "https://api.anthropic.com/api/oauth/userinfo";

/// Subscriptions endpoint URL — returns all org memberships with billing state.
pub const CLAUDE_CLI_SUBSCRIPTIONS_URL : &str = "https://api.anthropic.com/api/oauth/claude_cli/subscriptions";

/// Parse the body of `GET /api/oauth/userinfo` into [`UserinfoData`].
///
/// Uses string-needle scanning so it is available without feature flags.
///
/// # Errors
///
/// Returns [`QuotaError::ResponseParse`] if `tagged_id`, `uuid`, or `email_address`
/// are absent from the response body.
#[ inline ]
pub fn parse_userinfo( body : &str ) -> Result< UserinfoData, QuotaError >
{
  let tagged_id = parse_optional_string_in_block( body, "tagged_id" )
    .or_else( || parse_optional_string_in_block( body, "taggedId" ) )
    .ok_or_else( || QuotaError::ResponseParse( "tagged_id".to_string() ) )?;
  let uuid = parse_optional_string_in_block( body, "uuid" )
    .ok_or_else( || QuotaError::ResponseParse( "uuid".to_string() ) )?;
  let email_address = parse_optional_string_in_block( body, "email_address" )
    .or_else( || parse_optional_string_in_block( body, "emailAddress" ) )
    .ok_or_else( || QuotaError::ResponseParse( "email_address".to_string() ) )?;
  Ok( UserinfoData { tagged_id, uuid, email_address } )
}

/// Parse the body of `GET /api/oauth/claude_cli/subscriptions` into a membership list.
///
/// Delegates to [`parse_membership_list`] — the subscriptions endpoint uses the same
/// `memberships: [...]` structure as `/api/oauth/account`.
///
/// # Errors
///
/// Returns [`QuotaError::ResponseParse`] if the response body cannot be parsed.
#[ inline ]
pub fn parse_subscriptions( body : &str ) -> Result< Vec< MembershipRaw >, QuotaError >
{
  parse_membership_list( body )
}

/// Fetch user identity from the Anthropic OAuth userinfo endpoint.
///
/// # Errors
///
/// Returns [`QuotaError::HttpTransport`] on network failure, or
/// [`QuotaError::ResponseParse`] if the response body cannot be parsed.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn fetch_userinfo( token : &str ) -> Result< UserinfoData, QuotaError >
{
  let mut resp = http_agent()
    .get( OAUTH_USERINFO_URL )
    .header( "Authorization",     &format!( "Bearer {token}" ) )
    .header( "anthropic-version", ANTHROPIC_VERSION )
    .call()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  let status = resp.status().as_u16();
  if status >= 400
  {
    return Err( QuotaError::HttpTransport( format!( "HTTP {status}" ) ) );
  }

  let body = resp
    .body_mut()
    .read_to_string()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  parse_userinfo( &body )
}

/// Fetch all org memberships from the Anthropic subscriptions endpoint.
///
/// # Errors
///
/// Returns [`QuotaError::HttpTransport`] on network failure, or
/// [`QuotaError::ResponseParse`] if the response body cannot be parsed.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn fetch_subscriptions( token : &str ) -> Result< Vec< MembershipRaw >, QuotaError >
{
  let mut resp = http_agent()
    .get( CLAUDE_CLI_SUBSCRIPTIONS_URL )
    .header( "Authorization",     &format!( "Bearer {token}" ) )
    .header( "anthropic-version", ANTHROPIC_VERSION )
    .call()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  let status = resp.status().as_u16();
  if status >= 400
  {
    return Err( QuotaError::HttpTransport( format!( "HTTP {status}" ) ) );
  }

  let body = resp
    .body_mut()
    .read_to_string()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  parse_subscriptions( &body )
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
#[ inline ]
pub fn fetch_oauth_account( token : &str ) -> Result< OauthAccountData, QuotaError >
{
  let mut resp = http_agent()
    .get( OAUTH_ACCOUNT_URL )
    .header( "Authorization",     &format!( "Bearer {token}" ) )
    .header( "anthropic-version", ANTHROPIC_VERSION )
    .call()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  let status = resp.status().as_u16();
  if status >= 400
  {
    return Err( QuotaError::HttpTransport( format!( "HTTP {status}" ) ) );
  }

  let body = resp
    .body_mut()
    .read_to_string()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

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
#[ inline ]
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
#[ inline ]
pub fn fetch_claude_cli_roles( token : &str ) -> Result< ClaudeCliRolesData, QuotaError >
{
  let mut resp = http_agent()
    .get( CLAUDE_CLI_ROLES_URL )
    .header( "Authorization",     &format!( "Bearer {token}" ) )
    .header( "anthropic-version", ANTHROPIC_VERSION )
    .call()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  let status = resp.status().as_u16();
  if status >= 400
  {
    return Err( QuotaError::HttpTransport( format!( "HTTP {status}" ) ) );
  }

  let body = resp
    .body_mut()
    .read_to_string()
    .map_err( |e| QuotaError::HttpTransport( e.to_string() ) )?;

  parse_claude_cli_roles( &body )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::*;

  // ── BUG-237 MRE: multi-membership selection ─────────────────────────────────

  #[ test ]
  #[ doc = "`bug_reproducer(237)`" ]
  /// `parse_oauth_account` selects the stripe+max membership over a none-billing entry.
  ///
  /// # Root Cause
  /// `str::find("\"organization\":")` always resolves to `memberships[0]`'s organization
  /// block. Accounts with a paid subscription at index > 0 were silently misclassified as
  /// having no subscription.
  ///
  /// # Why Not Caught
  /// All test fixtures used single-membership bodies. Multi-membership accounts require
  /// separate Anthropic org entities — uncommon in CI fixtures.
  ///
  /// # Fix Applied
  /// `parse_oauth_account` now calls `parse_membership_list` which iterates ALL membership
  /// objects using brace-balanced scanning, then `select_membership_index` picks the
  /// highest-priority entry.
  ///
  /// # Prevention
  /// This test must FAIL before the fix (memberships[0] is "none") and PASS after.
  ///
  /// # Pitfall
  /// Always use brace-balanced extraction when iterating JSON arrays containing nested
  /// objects — `str::find` on a needle will collide with nested occurrences of the same key.
  fn mre_bug237_multi_membership_selects_stripe_max_over_none()
  {
    let body = r#"{
      "tagged_id": "user_01ABC",
      "memberships": [
        { "role": "member", "organization": { "billing_type": "none", "capabilities": ["chat"], "created_at": "2024-01-01T00:00:00Z" } },
        { "role": "admin",  "organization": { "billing_type": "stripe_subscription", "capabilities": ["claude_max","chat"], "created_at": "2024-02-01T00:00:00Z" } }
      ]
    }"#;
    let result = parse_oauth_account( body ).expect( "should parse" );
    assert_eq!( result.billing_type, "stripe_subscription", "must select membership[1] (stripe+max), not membership[0] (none)" );
    assert!( result.has_max, "membership[1] has claude_max capability" );
    assert_eq!( result.org_created_at, "2024-02-01T00:00:00Z" );
  }

  #[ test ]
  #[ doc = "`bug_reproducer(237)`" ]
  /// `parse_oauth_account` selects stripe (no max) over none when no max tier is present.
  fn mre_bug237_multi_membership_selects_stripe_over_none_no_max()
  {
    let body = r#"{
      "tagged_id": "user_01ABC",
      "memberships": [
        { "role": "member", "organization": { "billing_type": "none", "capabilities": ["chat"], "created_at": "2024-01-01T00:00:00Z" } },
        { "role": "admin",  "organization": { "billing_type": "stripe_subscription", "capabilities": ["chat"], "created_at": "2024-03-01T00:00:00Z" } }
      ]
    }"#;
    let result = parse_oauth_account( body ).expect( "should parse" );
    assert_eq!( result.billing_type, "stripe_subscription" );
    assert!( !result.has_max, "no claude_max in membership[1]" );
    assert_eq!( result.org_created_at, "2024-03-01T00:00:00Z" );
  }

  #[ test ]
  #[ doc = "`bug_reproducer(237)`" ]
  /// Single-membership body: index 0 is always selected (Priority 3 fallback unchanged).
  fn mre_bug237_single_membership_fallback_unchanged()
  {
    let body = r#"{
      "tagged_id": "user_01ABC",
      "memberships": [
        { "role": "member", "organization": { "billing_type": "none", "capabilities": ["chat"], "created_at": "2024-01-01T00:00:00Z" } }
      ]
    }"#;
    let result = parse_oauth_account( body ).expect( "should parse" );
    assert_eq!( result.billing_type, "none", "single membership always selected via Priority 3" );
    assert!( !result.has_max );
  }
}
