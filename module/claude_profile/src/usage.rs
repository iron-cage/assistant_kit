//! `.usage` command — all-accounts live quota table.
//!
//! Fetches live rate-limit utilization for every saved account via
//! `claude_quota::fetch_rate_limits()` and renders results as a `data_fmt` table.
//! Accounts are enumerated from the credential store in alphabetical order.
//!
//! ## Synthetic Row (AC-09)
//!
//! When `~/.claude/.credentials.json` contains a token that does not match any
//! saved account's stored token (e.g. a fresh login not yet saved), `fetch_all_quota()`
//! prepends a synthetic entry with `is_current: true` and name derived from
//! `~/.claude.json` `emailAddress` (falling back to `"(current session)"`).
//! This row is excluded from `find_next_for_strategy()` recommendations — it IS the current session.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use claude_quota::OauthUsageData;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ OutputFormat, OutputOptions, format_duration_secs, json_escape };

// ── Sort and prefer strategies ─────────────────────────────────────────────────

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
enum SortStrategy { Name, Endurance, Drain, Reset }

impl SortStrategy
{
  fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "name"      => Ok( Self::Name ),
      "endurance" => Ok( Self::Endurance ),
      "drain"     => Ok( Self::Drain ),
      "reset"     => Ok( Self::Reset ),
      _           => Err( format!(
        "invalid sort:: value {s:?}: valid values are `name`, `endurance`, `drain`, `reset`",
      ) ),
    }
  }

  /// Context-sensitive default `desc` direction for each strategy.
  ///
  /// `Endurance` defaults to `true` (best on top). All others default to `false`.
  fn default_desc( self ) -> bool
  {
    matches!( self, SortStrategy::Endurance )
  }
}

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
enum PreferStrategy { Any, Opus, Sonnet }

impl PreferStrategy
{
  fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "any"    => Ok( Self::Any ),
      "opus"   => Ok( Self::Opus ),
      "sonnet" => Ok( Self::Sonnet ),
      _        => Err( format!(
        "invalid prefer:: value {s:?}: valid values are `any`, `opus`, `sonnet`",
      ) ),
    }
  }
}

#[ derive( Copy, Clone, PartialEq, Eq, Debug ) ]
enum NextStrategy { Endurance, Drain }

impl NextStrategy
{
  fn parse( s : &str ) -> Result< Self, String >
  {
    match s
    {
      "endurance" => Ok( Self::Endurance ),
      "drain"     => Ok( Self::Drain ),
      _           => Err( format!(
        "invalid next:: value {s:?}: valid values are `endurance`, `drain`",
      ) ),
    }
  }
}

/// Column visibility state for the `.usage` quota table.
///
/// `flag` (first col) and `account` (name) are structural and always visible.
/// All other columns follow the default set; `cols::` modifiers toggle each one.
#[ allow( clippy::struct_excessive_bools ) ]
struct ColsVisibility
{
  /// `●` composite status emoji column (default ON).
  status      : bool,
  /// `Expires` token TTL column (default ON).
  expires     : bool,
  /// `Sub` subscription label column (default OFF).
  sub         : bool,
  /// `~Renews` next billing date column (default ON).
  renews      : bool,
  /// `5h Left` session quota remaining (default ON).
  h5_left     : bool,
  /// `5h Reset` session reset countdown (default ON).
  h5_reset    : bool,
  /// `7d Left` weekly quota remaining (default ON).
  d7_left     : bool,
  /// `7d(Son)` Sonnet-only weekly quota remaining (default ON).
  d7_son      : bool,
  /// `7d Reset` weekly reset countdown (default ON).
  d7_reset    : bool,
  /// `7d Son Reset` Sonnet weekly reset countdown (default OFF).
  d7_son_reset : bool,
}

impl ColsVisibility
{
  fn default_set() -> Self
  {
    Self
    {
      status       : true,
      expires      : true,
      sub          : false,
      renews       : true,
      h5_left      : true,
      h5_reset     : true,
      d7_left      : true,
      d7_son       : true,
      d7_reset     : true,
      d7_son_reset : false,
    }
  }

  fn apply_modifier( &mut self, modifier : &str ) -> Result< (), String >
  {
    let ( show, id ) = if let Some( rest ) = modifier.strip_prefix( '+' )
    {
      ( true, rest )
    }
    else if let Some( rest ) = modifier.strip_prefix( '-' )
    {
      ( false, rest )
    }
    else
    {
      return Err( format!( "cols:: modifier {modifier:?} must start with `+` or `-`" ) );
    };
    match id
    {
      "status"       => self.status       = show,
      "expires"      => self.expires      = show,
      "sub"          => self.sub          = show,
      "renews"       => self.renews       = show,
      "5h_left"      => self.h5_left      = show,
      "5h_reset"     => self.h5_reset     = show,
      "7d_left"      => self.d7_left      = show,
      "7d_son"       => self.d7_son       = show,
      "7d_reset"     => self.d7_reset     = show,
      "7d_son_reset" => self.d7_son_reset = show,
      _              => return Err( format!(
        "cols:: unknown column {id:?}: valid IDs are `status`, `expires`, `sub`, `renews`, `5h_left`, `5h_reset`, `7d_left`, `7d_son`, `7d_reset`, `7d_son_reset`",
      ) ),
    }
    Ok( () )
  }

  fn parse( s : &str ) -> Result< Self, String >
  {
    let mut vis = Self::default_set();
    for modifier in s.split( ',' ).map( str::trim ).filter( |m| !m.is_empty() )
    {
      vis.apply_modifier( modifier )?;
    }
    Ok( vis )
  }
}

// ── Per-account quota result ───────────────────────────────────────────────────

struct AccountQuota
{
  name          : String,
  /// Live-token match: `accessToken` in `~/.claude/.credentials.json` equals this account's stored token.
  is_current    : bool,
  /// Active-marker match: per-machine active marker file in the credential store names this account.
  is_active     : bool,
  expires_at_ms : u64,
  /// `Ok` = live quota fetched; `Err` = reason string (expired, network, etc.).
  result        : Result< OauthUsageData, String >,
  /// Billing state from `GET /api/oauth/account`; `None` if the fetch failed.
  account       : Option< claude_quota::OauthAccountData >,
}

// ── Fetch helpers ──────────────────────────────────────────────────────────────

/// Read the OAuth access token from a named account credentials file.
///
/// Returns `Err(reason)` on I/O failure or missing `accessToken` field;
/// the reason is stored inline per-account and does not abort the full fetch.
fn read_token( credential_store : &std::path::Path, name : &str ) -> Result< String, String >
{
  let path    = credential_store.join( format!( "{name}.credentials.json" ) );
  let content = std::fs::read_to_string( &path )
    .map_err( |e| format!( "cannot read credentials: {e}" ) )?;
  crate::account::parse_string_field( &content, "accessToken" )
    .ok_or_else( || "missing accessToken".to_string() )
}

/// Enumerate all saved accounts and fetch their live quota data.
///
/// Accounts are listed in alphabetical order (delegated to `account::list()`).
/// Per-account failures are stored inline in `AccountQuota::result`; only
/// fatal errors (credential store unreadable) propagate as `ErrorData`.
///
/// `live_creds_file` is read once to extract the live `accessToken`; any failure
/// (absent file, parse error) silently sets `is_current = false` for all accounts.
///
/// If no saved account's token matches the live token, a synthetic entry is prepended
/// (AC-09): `is_current: true`, name from `~/.claude.json` email or `(current session)`.
/// Pitfall: this case is easy to miss when only testing the normal single-account path.
///
/// When `stagger` is `true`, each account fetch is preceded by a pseudo-random sleep
/// of 200–1500 ms (thunder-herd mitigation for live monitor mode).
///
/// When `trace` is `true`, one `[trace]` line is written to stderr before reading
/// each account's credentials and one after receiving the API response.
fn fetch_all_quota(
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  stagger          : bool,
  trace            : bool,
) -> Result< Vec< AccountQuota >, ErrorData >
{
  let accounts = crate::account::list( credential_store )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot read credential store: {e}" ),
    ) )?;

  // Read the live session token once (graceful degradation on any error).
  let live_token : Option< String > = std::fs::read_to_string( live_creds_file )
    .ok()
    .and_then( |s| crate::account::parse_string_field( &s, "accessToken" ) );

  let mut results = Vec::with_capacity( accounts.len() );
  for acct in &accounts
  {
    // Per-account stagger delay — prevents simultaneous API bursts in live mode.
    if stagger
    {
      let nanos = u64::from(
        std::time::SystemTime::now()
          .duration_since( std::time::UNIX_EPOCH )
          .unwrap_or_default()
          .subsec_nanos()
      );
      std::thread::sleep( core::time::Duration::from_millis( 200 + nanos % 1301 ) ); // 200..=1500 ms
    }

    // Determine whether this account's stored token matches the live session.
    let is_current = live_token.as_ref().is_some_and( |live|
    {
      read_token( credential_store, &acct.name )
        .is_ok_and( |stored| stored == *live )
    } );
    if trace
    {
      let creds_path = credential_store.join( format!( "{}.credentials.json", acct.name ) );
      eprintln!( "[trace] {}  reading {}", acct.name, creds_path.display() );
    }
    let ( result, account ) = match read_token( credential_store, &acct.name )
    {
      Ok( token ) =>
      {
        // Spawn account fetch in parallel with usage fetch — keeps latency additive-free.
        let token_for_account = token.clone();
        let account_handle = std::thread::spawn( move ||
        {
          claude_quota::fetch_oauth_account( &token_for_account )
        } );

        if trace
        {
          let prefix = if token.len() >= 20 { &token[ ..20 ] } else { &token };
          eprintln!( "[trace] {}  GET {}  token={}...  exp={}", acct.name, claude_quota::OAUTH_USAGE_URL, prefix, token_exp_label( acct.expires_at_ms ) );
        }
        let r = claude_quota::fetch_oauth_usage( &token ).map_err( |e| e.to_string() );
        if trace
        {
          match &r
          {
            Ok( _ )  => eprintln!( "[trace] {}  result: OK", acct.name ),
            Err( e ) => eprintln!( "[trace] {}  result: Err({})", acct.name, e ),
          }
        }
        let account_data = account_handle.join().ok().and_then( core::result::Result::ok );
        ( r, account_data )
      }
      Err( e )    =>
      {
        if trace { eprintln!( "[trace] {}  cannot read token: {}", acct.name, e ); }
        ( Err( e ), None )
      }
    };
    results.push( AccountQuota
    {
      name          : acct.name.clone(),
      is_current,
      is_active     : acct.is_active,
      expires_at_ms : acct.expires_at_ms,
      result,
      account,
    } );
  }

  // Synthetic row: when live creds exist but no saved account matches the live
  // token, prepend a row so the current session is still visible in the table.
  let any_current = results.iter().any( |r| r.is_current );
  if !any_current
  {
    if let Some( ref token ) = live_token
    {
      let synthetic_name = live_creds_file.parent()
        .and_then( |p| p.parent() )
        .map( |home| home.join( ".claude.json" ) )
        .and_then( |p| std::fs::read_to_string( p ).ok() )
        .and_then( |s| crate::account::parse_string_field( &s, "emailAddress" ) )
        .filter( |e| !e.is_empty() )
        .unwrap_or_else( || "(current session)".to_string() );
      let expires_at_ms = parse_u64_field( live_creds_file, "expiresAt" ).unwrap_or( 0 );
      let result        = claude_quota::fetch_oauth_usage( token ).map_err( |e| e.to_string() );
      let account       = claude_quota::fetch_oauth_account( token ).ok();
      results.insert( 0, AccountQuota
      {
        name : synthetic_name,
        is_current    : true,
        is_active     : false,
        expires_at_ms,
        result,
        account,
      } );
    }
  }

  Ok( results )
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Format token expiry as a human-readable label for trace output.
///
/// Returns `"expired(Xd Yh ago)"` or `"valid(Xd Yh left)"` using the same
/// duration format as `format_duration_secs`.
fn token_exp_label( expires_at_ms : u64 ) -> String
{
  let now_ms = u64::try_from(
    std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap_or_default()
      .as_millis()
  ).unwrap_or( u64::MAX );
  if now_ms >= expires_at_ms
  {
    format!( "expired({} ago)", format_duration_secs( ( now_ms - expires_at_ms ) / 1000 ) )
  }
  else
  {
    format!( "valid({} left)", format_duration_secs( ( expires_at_ms - now_ms ) / 1000 ) )
  }
}

/// Parse a raw numeric JSON field from a string without an external JSON parser.
///
/// Finds `"key":` by string scan and parses the immediately following run of
/// ASCII digits as `u64`. Returns `None` on a missing key or non-numeric value.
/// Works for both flat (`{"key":N}`) and nested (`{"outer":{"key":N}}`) JSON.
fn parse_u64_from_str( s : &str, key : &str ) -> Option< u64 >
{
  let needle = format!( "\"{key}\":" );
  let start  = s.find( &needle )? + needle.len();
  let rest   = s[ start.. ].trim_start();
  let end    = rest.find( |c : char| !c.is_ascii_digit() ).unwrap_or( rest.len() );
  rest[ ..end ].parse().ok()
}

/// Parse a raw numeric JSON field from a file without an external JSON parser.
///
/// Reads the file at `path` then delegates to `parse_u64_from_str`. Returns `None`
/// on any I/O error, missing key, or non-numeric value.
fn parse_u64_field( path : &std::path::Path, key : &str ) -> Option< u64 >
{
  let s = std::fs::read_to_string( path ).ok()?;
  parse_u64_from_str( &s, key )
}

fn base64url_decode( s : &str ) -> Option< Vec< u8 > >
{
  // Translate URL-safe alphabet to standard and add padding.
  let pad = match s.len() % 4 { 0 => 0, 2 => 2, 3 => 1, _ => return None };
  let b64 : String = s.chars()
    .map( |c| match c { '-' => '+', '_' => '/', c => c } )
    .chain( core::iter::repeat( '=' ).take( pad ) )
    .collect();
  // Decode groups of 4 base64 characters → 3 bytes.
  const ALPHA : &[ u8 ] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  // ALPHA has 64 entries (positions 0–63), so the position always fits in u32.
  let val = |c : u8| ALPHA.iter().position( |&a| a == c )
    .and_then( |v| u32::try_from( v ).ok() );
  let bytes = b64.as_bytes();
  let mut out = Vec::with_capacity( b64.len() / 4 * 3 );
  let mut i = 0;
  while i + 3 < bytes.len()
  {
    let v0 = val( bytes[ i ] )?;
    let v1 = val( bytes[ i + 1 ] )?;
    // `& 0xFF` makes the narrowing cast lossless — the upper bits are always zero.
    out.push( ( ( ( v0 << 2 ) | ( v1 >> 4 ) ) & 0xFF ) as u8 );
    if bytes[ i + 2 ] != b'='
    {
      let v2 = val( bytes[ i + 2 ] )?;
      out.push( ( ( ( v1 << 4 ) | ( v2 >> 2 ) ) & 0xFF ) as u8 );
    }
    if bytes[ i + 3 ] != b'='
    {
      let v2 = val( bytes[ i + 2 ] )?;
      let v3 = val( bytes[ i + 3 ] )?;
      out.push( ( ( ( v2 << 6 ) | v3 ) & 0xFF ) as u8 );
    }
    i += 4;
  }
  Some( out )
}

/// Extracts the `exp` claim from the `accessToken` JWT inside a credentials JSON string.
///
/// Returns `Some(exp_ms)` where `exp_ms = exp_secs * 1000`, or `None` if the token is
/// absent, malformed, or missing the `exp` field.  No signature verification is performed —
/// the claim is used only for display purposes.
#[ must_use ]
#[ inline ]
pub fn jwt_exp_ms( creds_json : &str ) -> Option< u64 >
{
  // Locate the accessToken string value.
  let key   = "\"accessToken\":\"";
  let start = creds_json.find( key )? + key.len();
  let rest  = &creds_json[ start.. ];
  let end   = rest.find( '"' )?;
  let token = &rest[ ..end ];
  // Split JWT into header.payload.signature — take payload (second segment).
  let mut parts   = token.splitn( 3, '.' );
  let _header     = parts.next()?;
  let payload_b64 = parts.next()?;
  // Base64url-decode and UTF-8-decode the payload.
  let payload_bytes = base64url_decode( payload_b64 )?;
  let payload       = core::str::from_utf8( &payload_bytes ).ok()?;
  // Extract the numeric `exp` field.
  let needle    = "\"exp\":";
  let after     = &payload[ payload.find( needle )? + needle.len().. ];
  let digits_end = after.find( |c : char| !c.is_ascii_digit() ).unwrap_or( after.len() );
  let exp_secs : u64 = after[ ..digits_end ].parse().ok()?;
  Some( exp_secs * 1000 )
}

/// Compute the `Expires` cell value for a given token expiry and current time.
///
/// Returns `"EXPIRED"` when `expires_at_ms / 1000 ≤ now_secs` (saturating), or
/// `"in Xh Ym"` when the token is still valid.
fn compute_expires_cell( expires_at_ms : u64, now_secs : u64 ) -> String
{
  let remaining = ( expires_at_ms / 1000 ).saturating_sub( now_secs );
  if remaining == 0
  {
    "EXPIRED".to_string()
  }
  else
  {
    format!( "in {}", format_duration_secs( remaining ) )
  }
}

/// Convert a Unix timestamp (seconds) to a Gregorian `(year, month, day)` tuple.
///
/// Month is 1-based (1 = January). Day is 1-based (1 = first of month).
/// No external dependencies — hand-rolled Gregorian arithmetic.
fn unix_to_date( unix_secs : u64 ) -> ( u64, u64, u64 )
{
  let is_leap     = |y : u64| ( y % 4 == 0 && y % 100 != 0 ) || y % 400 == 0;
  let mut days    = unix_secs / 86_400;
  let mut year    = 1970_u64;
  loop
  {
    let in_year = if is_leap( year ) { 366 } else { 365 };
    if days < in_year { break; }
    days -= in_year;
    year += 1;
  }
  let feb = if is_leap( year ) { 29 } else { 28 };
  let month_days : [ u64; 12 ] = [ 31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  let mut month = 0_u64;
  for d in &month_days
  {
    if days < *d { break; }
    days -= d;
    month += 1;
  }
  ( year, month + 1, days + 1 )
}

/// Format the estimated next billing renewal as `"Mon DD"` (e.g. `"Jun  5"`).
///
/// Billing day is taken from `org_created_at` (ISO-8601 `"YYYY-MM-DD..."`).
/// Returns em-dash if parsing fails or `org_created_at` is too short.
fn next_billing_label( org_created_at : &str, now_secs : u64 ) -> String
{
  const MONTHS : [ &str; 12 ] = [ "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                                   "Jul", "Aug", "Sep", "Oct", "Nov", "Dec" ];
  if org_created_at.len() < 10 { return "\u{2014}".to_string(); }
  let billing_day : u64 = match org_created_at[ 8..10 ].parse() { Ok( d ) => d, Err( _ ) => return "\u{2014}".to_string() };
  if billing_day == 0 || billing_day > 31 { return "\u{2014}".to_string(); }
  let ( _, current_month, current_day ) = unix_to_date( now_secs );
  let renewal_month = if billing_day > current_day
  {
    current_month
  }
  else if current_month == 12
  {
    1
  }
  else
  {
    current_month + 1
  };
  #[ allow( clippy::cast_possible_truncation ) ] // renewal_month is always 1–12; cast to usize is safe
  let month_name = MONTHS[ ( renewal_month - 1 ) as usize ];
  format!( "{month_name} {billing_day:2}" )
}

/// Map account billing state to a short subscription label for the `Sub` column.
///
/// - `None`                      → `"?"` (fetch failed — state unknown)
/// - `billing_type == "none"`    → `"—"` (no active subscription)
/// - `has_max`                   → `"max"` (Claude Max plan)
/// - `"stripe_subscription"` + `!has_max` → `"pro"` (paid but not Max)
/// - anything else               → `"?"`
fn sub_label( account : Option< &claude_quota::OauthAccountData > ) -> &'static str
{
  let Some( a ) = account else { return "?"; };
  if a.billing_type == "none"                { return "\u{2014}"; }
  if a.has_max                               { return "max"; }
  if a.billing_type == "stripe_subscription" { return "pro"; }
  "?"
}

// Fix(BUG-152)
// Root cause: shorten_error had no HTTP 401 branch; the else { reason } arm returned the
//   verbose "HTTP transport error: HTTP 401" string verbatim into the 7d Reset column,
//   violating AC-03 ("shortened error reason"). HTTP 401 was added to T05 as a
//   pass-through regression guard in task 150, inadvertently documenting the wrong behaviour.
//   task/claude_profile/bug/152_shorten_error_omits_401.md
// Pitfall: shorten_error is a manual allowlist — each new HTTP error code from
//   QuotaError::HttpTransport needs an explicit branch. The else arm is NOT a shortener;
//   it is a verbatim passthrough. test_shorten_error_no_raw_http_transport_passthrough
//   enforces this invariant for known codes (401, 403, 429).
/// Shorten verbose quota error strings for display in the final table column.
///
/// `QuotaError::HttpTransport` formats errors as `"HTTP transport error: HTTP NNN"`.
/// Handled codes: `429` → `"rate limited (429)"`; `401` → `"auth expired (401)"`;
/// `403` → `"auth forbidden (403)"` (permission error returned by the usage API).
/// `QuotaError::MissingHeader` (displays as `"rate-limit header missing: …"`) is
/// shortened to `"no header"`. All other strings pass through unchanged.
/// The caller is responsible for wrapping the result in parentheses.
fn shorten_error( reason : &str ) -> &str
{
  if reason.starts_with( "HTTP transport error: HTTP 429" )
  {
    "rate limited (429)"
  }
  else if reason.starts_with( "HTTP transport error: HTTP 401" )
  {
    "auth expired (401)"
  }
  else if reason.starts_with( "HTTP transport error: HTTP 403" )
  {
    "auth forbidden (403)"
  }
  else if reason.starts_with( "rate-limit header missing:" )
  {
    "no header"
  }
  else
  {
    reason
  }
}

/// Return `5h Left` as a percentage for sorting purposes.
///
/// Returns `100.0 - five_hour.utilization` for `Ok` accounts, or `-1.0` for `Err`
/// accounts (treated as below-exhausted for drain/reset floor sinking).
fn five_hour_left( aq : &AccountQuota ) -> f64
{
  if let Ok( data ) = &aq.result
  {
    100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization )
  }
  else
  {
    -1.0
  }
}

/// Return the weekly quota left (%) for a given `prefer` strategy.
///
/// - `Opus`   → `7d Left` only.
/// - `Sonnet` → `7d(Son)` only.
/// - `Any`    → `min(7d Left, 7d(Son))` — conservative: whichever cap is more constrained.
///
/// Absent period data is treated as `0.0` left. `Err` accounts return `0.0`.
fn prefer_weekly( aq : &AccountQuota, prefer : PreferStrategy ) -> f64
{
  let Ok( data ) = &aq.result else { return 0.0; };
  let left_7d  = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
  let left_son = 100.0 - data.seven_day_sonnet.as_ref().map_or( 0.0, |p| p.utilization );
  match prefer
  {
    PreferStrategy::Opus   => left_7d,
    PreferStrategy::Sonnet => left_son,
    PreferStrategy::Any    => left_7d.min( left_son ),
  }
}

/// Return indices into `accounts` sorted by `strategy` and `desc`.
///
/// Each strategy has a canonical direction (its `default_desc()`). Passing
/// `desc = Some(!strategy.default_desc())` inverts the canonical order.
///
/// For `drain` and `reset`, exhausted accounts (`5h Left ≤ 5%`) are always
/// appended last regardless of `desc`. For `name` and `endurance`, `desc`
/// reverses the whole slice (no exhausted floor).
///
/// See `docs/feature/020_usage_sort_strategies.md` for full algorithm specs.
#[ allow( clippy::too_many_lines ) ]
fn sort_indices(
  accounts  : &[ AccountQuota ],
  strategy  : SortStrategy,
  desc      : Option< bool >,
  prefer    : PreferStrategy,
  now_secs  : u64,
) -> Vec< usize >
{
  let effective_desc = desc.unwrap_or( strategy.default_desc() );
  // `reversed`: true when effective direction deviates from the canonical direction.
  let reversed = effective_desc != strategy.default_desc();

  let all : Vec< usize > = ( 0..accounts.len() ).collect();

  match strategy
  {
    SortStrategy::Name =>
    {
      let mut v = all;
      v.sort_by( |&a, &b| accounts[ a ].name.cmp( &accounts[ b ].name ) );
      if reversed { v.reverse(); }
      v
    }

    SortStrategy::Endurance =>
    {
      let reset_secs_of = |i : usize| -> Option< u64 >
      {
        if let Ok( data ) = &accounts[ i ].result
        {
          data.five_hour.as_ref()
            .and_then( |p| p.resets_at.as_deref() )
            .and_then( claude_quota::iso_to_unix_secs )
            .map( |t| t.saturating_sub( now_secs ) )
        }
        else { None }
      };

      let ( mut qualified, mut unqualified ) : ( Vec< usize >, Vec< usize > ) =
        all.into_iter().partition( |&i|
        {
          reset_secs_of( i ).is_some_and( |r| ( 900..=3600 ).contains( &r ) )
            && prefer_weekly( &accounts[ i ], prefer ) >= 30.0
        } );

      // Qualified canonical: highest weekly first, then soonest reset.
      qualified.sort_by( |&a, &b|
      {
        let wa = prefer_weekly( &accounts[ a ], prefer );
        let wb = prefer_weekly( &accounts[ b ], prefer );
        wb.partial_cmp( &wa ).unwrap_or( core::cmp::Ordering::Equal )
          .then_with( ||
          {
            let ra = reset_secs_of( a ).unwrap_or( u64::MAX );
            let rb = reset_secs_of( b ).unwrap_or( u64::MAX );
            ra.cmp( &rb )
          } )
      } );

      // Unqualified canonical: highest 5h_left first; tiebreak highest weekly.
      unqualified.sort_by( |&a, &b|
      {
        let la = five_hour_left( &accounts[ a ] );
        let lb = five_hour_left( &accounts[ b ] );
        lb.partial_cmp( &la ).unwrap_or( core::cmp::Ordering::Equal )
          .then_with( ||
          {
            let wa = prefer_weekly( &accounts[ a ], prefer );
            let wb = prefer_weekly( &accounts[ b ], prefer );
            wb.partial_cmp( &wa ).unwrap_or( core::cmp::Ordering::Equal )
          } )
      } );

      let mut result = qualified;
      result.extend( unqualified );
      if reversed { result.reverse(); }
      result
    }

    SortStrategy::Drain =>
    {
      let ( mut non_exhausted, exhausted_vec ) : ( Vec< usize >, Vec< usize > ) =
        all.into_iter().partition( |&i| five_hour_left( &accounts[ i ] ) > 5.0 );

      // Canonical: ascending 5h_left (lowest = most drained first); tiebreak highest weekly.
      non_exhausted.sort_by( |&a, &b|
      {
        let la = five_hour_left( &accounts[ a ] );
        let lb = five_hour_left( &accounts[ b ] );
        la.partial_cmp( &lb ).unwrap_or( core::cmp::Ordering::Equal )
          .then_with( ||
          {
            let wa = prefer_weekly( &accounts[ a ], prefer );
            let wb = prefer_weekly( &accounts[ b ], prefer );
            wb.partial_cmp( &wa ).unwrap_or( core::cmp::Ordering::Equal )
          } )
      } );

      if reversed { non_exhausted.reverse(); }
      non_exhausted.extend( exhausted_vec );
      non_exhausted
    }

    SortStrategy::Reset =>
    {
      let reset_secs_of = |i : usize| -> u64
      {
        if let Ok( data ) = &accounts[ i ].result
        {
          data.five_hour.as_ref()
            .and_then( |p| p.resets_at.as_deref() )
            .and_then( claude_quota::iso_to_unix_secs )
            .map_or( u64::MAX, |t| t.saturating_sub( now_secs ) )
        }
        else { u64::MAX }
      };

      let ( mut non_exhausted, exhausted_vec ) : ( Vec< usize >, Vec< usize > ) =
        all.into_iter().partition( |&i| five_hour_left( &accounts[ i ] ) > 5.0 );

      // Canonical: ascending reset_secs (soonest first); tiebreak ascending 5h_left.
      non_exhausted.sort_by( |&a, &b|
      {
        reset_secs_of( a ).cmp( &reset_secs_of( b ) )
          .then_with( ||
          {
            let la = five_hour_left( &accounts[ a ] );
            let lb = five_hour_left( &accounts[ b ] );
            la.partial_cmp( &lb ).unwrap_or( core::cmp::Ordering::Equal )
          } )
      } );

      if reversed { non_exhausted.reverse(); }
      non_exhausted.extend( exhausted_vec );
      non_exhausted
    }
  }
}

/// Return the first eligible (non-current, non-active, non-expired, `Ok`) account
/// from a pre-sorted index slice, or `None` when no eligible account exists.
fn find_first_eligible( accounts : &[ AccountQuota ], sorted : &[ usize ], now_secs : u64 ) -> Option< usize >
{
  for &idx in sorted
  {
    let aq = &accounts[ idx ];
    if aq.is_current || aq.is_active { continue; }
    if ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs ) == 0 { continue; }
    if aq.result.is_ok() { return Some( idx ); }
  }
  None
}

/// Find the recommended next account for a specific `next` strategy.
///
/// `Endurance` and `Drain` sort via `sort_indices()` then pick the first
/// eligible (non-current, non-active, non-expired, `Ok`) account.
fn find_next_for_strategy(
  accounts  : &[ AccountQuota ],
  strategy  : NextStrategy,
  prefer    : PreferStrategy,
  now_secs  : u64,
) -> Option< usize >
{
  match strategy
  {
    NextStrategy::Endurance =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Endurance, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs )
    }
    NextStrategy::Drain =>
    {
      let sorted = sort_indices( accounts, SortStrategy::Drain, None, prefer, now_secs );
      find_first_eligible( accounts, &sorted, now_secs )
    }
  }
}

/// Format the key metric string for one strategy recommendation line.
///
/// Used in both single-strategy (`→ Next: name  (metric)`) and multi-strategy
/// (`Next by strategy:` / `  endurance  name   metric`) footers.
fn strategy_metric(
  aq       : &AccountQuota,
  strategy : NextStrategy,
  prefer   : PreferStrategy,
  now_secs : u64,
) -> String
{
  let expires_in_secs = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
  let expires_str     = format_duration_secs( expires_in_secs );
  let Ok( data ) = &aq.result else { return String::new(); };
  let session_pct = data.five_hour.as_ref().map_or( 0.0, |p| 100.0 - p.utilization );
  let reset_str   = data.five_hour.as_ref()
    .and_then( |p| p.resets_at.as_deref() )
    .and_then( claude_quota::iso_to_unix_secs )
    .map_or_else( || "\u{2014}".to_string(), |t| format_duration_secs( t.saturating_sub( now_secs ) ) );
  match strategy
  {
    NextStrategy::Endurance =>
    {
      let weekly_pct = prefer_weekly( aq, prefer );
      format!( "{session_pct:.0}% session, {weekly_pct:.0}% 7d left, expires in {expires_str}" )
    }
    NextStrategy::Drain =>
      format!( "{session_pct:.0}% session, resets in {reset_str}" ),
  }
}

// ── Output renderers ───────────────────────────────────────────────────────────

/// Compute the 5 quota display cells for a successful OAuth usage fetch.
///
/// Returns `[5h_left, 5h_reset, 7d_left, 7d_son, 7d_reset]` as display strings.
/// `5h Left` and `7d Left` cells carry a `🟢`/`🟡` prefix (same threshold as `status_emoji`).
/// Absent periods render as em-dash; absent reset timestamps render as em-dash.
fn quota_text_cells( data : &OauthUsageData, now_secs : u64 ) -> [ String; 5 ]
{
  let dash      = "\u{2014}".to_string();
  let pct_cell  = |util : Option< f64 >| -> String
  {
    util.map_or_else( || dash.clone(), |u| format!( "{:.0}%", 100.0 - u ) )
  };
  let pct_emoji = |util : Option< f64 >| -> String
  {
    util.map_or_else( || dash.clone(), |u|
    {
      let left  = 100.0 - u;
      let emoji = if left > 5.0 { "🟢" } else { "🟡" };
      format!( "{emoji} {left:.0}%" )
    } )
  };
  let reset_cell = |iso : Option< &str >| -> String
  {
    iso.and_then( claude_quota::iso_to_unix_secs )
      .map_or_else( || dash.clone(), |t|
        format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) )
      )
  };
  [
    pct_emoji( data.five_hour.as_ref().map( |p| p.utilization ) ),
    reset_cell( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
    pct_emoji( data.seven_day.as_ref().map( |p| p.utilization ) ),
    pct_cell(  data.seven_day_sonnet.as_ref().map( |p| p.utilization ) ),
    reset_cell( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
  ]
}

/// Return the single-glyph quota status emoji for an account row.
///
/// - `"🔴"` — token is invalid or missing (`result` is `Err`).
/// - `"🟡"` — token valid, but at least one of `5h Left` or `7d Left` is `≤ 5%`.
/// - `"🟢"` — token valid, BOTH `5h Left > 5%` AND `7d Left > 5%`.
///
/// Absent period data is treated as fully available (conservative, 0% utilised).
fn status_emoji( result : &Result< claude_quota::OauthUsageData, String > ) -> &'static str
{
  match result
  {
    Err( _ ) => "🔴",
    Ok( data ) =>
    {
      let h5_left = 100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization );
      let d7_left = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
      if h5_left > 5.0 && d7_left > 5.0 { "🟢" } else { "🟡" }
    }
  }
}

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// Column visibility is controlled by `cols` (structural `flag` and `account`
/// columns are always shown). Footer (TSK-184): always-visible 2-strategy block
/// when ≥2 accounts have valid quota — shows `endurance` and `drain` lines.
/// The `→` marker in the table body points to the active-strategy winner.
/// Footer is omitted when < 2 accounts have valid quota data.
#[ allow( clippy::too_many_lines ) ]
fn render_text(
  accounts : &[ AccountQuota ],
  sort     : SortStrategy,
  desc     : Option< bool >,
  prefer   : PreferStrategy,
  next     : NextStrategy,
  cols     : &ColsVisibility,
) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty()
  {
    return "Quota\n\n  (no accounts configured)\n".to_string();
  }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  // Compute the winner for the active strategy; placed as → marker in the table body.
  let best_idx       = find_next_for_strategy( accounts, next, prefer, now_secs );
  let sorted_indices = sort_indices( accounts, sort, desc, prefer, now_secs );

  // Three-tier grouping: sort order preserved within each tier (🟢 → 🟡 → 🔴).
  // Applied after the sort strategy so each tier's internal order reflects the chosen sort.
  // AC-26: within 🟡, session-exhausted (5h Left ≤ 5%) precedes weekly-exhausted.
  // Accounts where both 5h Left ≤ 5% AND 7d Left ≤ 5% fall in the session-exhausted sub-group.
  let ( mut green_indices, mut red_indices ) = ( Vec::new(), Vec::new() );
  let ( mut session_yellow, mut weekly_yellow ) = ( Vec::new(), Vec::new() );
  for idx in sorted_indices
  {
    match status_emoji( &accounts[ idx ].result )
    {
      "🟢" => green_indices.push( idx ),
      "🟡" =>
      {
        let h5_left = if let Ok( data ) = &accounts[ idx ].result
        {
          100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization )
        }
        else { 100.0 };
        if h5_left <= 5.0 { session_yellow.push( idx ); }
        else              { weekly_yellow.push( idx ); }
      }
      _    => red_indices.push( idx ),
    }
  }
  let sorted_indices : Vec< usize > = green_indices.into_iter()
    .chain( session_yellow )
    .chain( weekly_yellow )
    .chain( red_indices )
    .collect();

  // Build headers conditionally — structural cols always first and always visible.
  let mut headers = vec![ String::new() ]; // flag col
  if cols.status       { headers.push( "●".to_string() ); }
  headers.push( "Account".to_string() ); // account name — structural
  if cols.expires      { headers.push( "Expires".to_string() ); }
  if cols.sub          { headers.push( "Sub".to_string() ); }
  if cols.renews       { headers.push( "~Renews".to_string() ); }
  if cols.h5_left      { headers.push( "5h Left".to_string() ); }
  if cols.h5_reset     { headers.push( "5h Reset".to_string() ); }
  if cols.d7_left      { headers.push( "7d Left".to_string() ); }
  if cols.d7_son       { headers.push( "7d(Son)".to_string() ); }
  if cols.d7_reset     { headers.push( "7d Reset".to_string() ); }
  if cols.d7_son_reset { headers.push( "7d Son Reset".to_string() ); }

  let mut builder = RowBuilder::new( headers );
  for orig_idx in sorted_indices.iter().copied()
  {
    let aq = &accounts[ orig_idx ];
    // Four-level priority: ✓ (is_current) > * (is_active, not current) > → (active-strategy winner) > blank.
    let flag_cell = if aq.is_current
    {
      "✓".to_string()
    }
    else if aq.is_active
    {
      "*".to_string()
    }
    else if best_idx == Some( orig_idx )
    {
      "→".to_string()
    }
    else
    {
      String::new()
    };

    let expires_cell = compute_expires_cell( aq.expires_at_ms, now_secs );
    let sub_str      = sub_label( aq.account.as_ref() ).to_string();
    let renews_str   = aq.account.as_ref()
      .map_or_else( || "?".to_string(), |a| next_billing_label( &a.org_created_at, now_secs ) );

    match &aq.result
    {
      Ok( data ) =>
      {
        let cells        = quota_text_cells( data, now_secs );
        let son_reset    = data.seven_day_sonnet.as_ref()
          .and_then( |p| p.resets_at.as_deref() )
          .and_then( claude_quota::iso_to_unix_secs )
          .map_or_else(
            || "\u{2014}".to_string(),
            |t| format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) ),
          );

        let mut row : Vec< String > = vec![ flag_cell ];
        if cols.status       { row.push( status_emoji( &aq.result ).to_string() ); }
        row.push( aq.name.clone() );
        if cols.expires      { row.push( expires_cell ); }
        if cols.sub          { row.push( sub_str ); }
        if cols.renews       { row.push( renews_str ); }
        if cols.h5_left      { row.push( cells[ 0 ].clone() ); }
        if cols.h5_reset     { row.push( cells[ 1 ].clone() ); }
        if cols.d7_left      { row.push( cells[ 2 ].clone() ); }
        if cols.d7_son       { row.push( cells[ 3 ].clone() ); }
        if cols.d7_reset     { row.push( cells[ 4 ].clone() ); }
        if cols.d7_son_reset { row.push( son_reset ); }
        builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
      }
      Err( reason ) =>
      {
        let dash      = "\u{2014}".to_string();
        let error_str = format!( "({})", shorten_error( reason ) );
        // Data columns visible mask (order matches col rendering above).
        let data_vis  = [ cols.h5_left, cols.h5_reset, cols.d7_left, cols.d7_son, cols.d7_reset, cols.d7_son_reset ];
        let last_vis  = data_vis.iter().rposition( |&v| v );

        let mut row : Vec< String > = vec![ flag_cell ];
        if cols.status  { row.push( status_emoji( &aq.result ).to_string() ); }
        row.push( aq.name.clone() );
        if cols.expires { row.push( expires_cell ); }
        if cols.sub     { row.push( sub_str ); }
        if cols.renews  { row.push( renews_str ); }
        for ( i, &vis ) in data_vis.iter().enumerate()
        {
          if vis
          {
            row.push( if last_vis == Some( i ) { error_str.clone() } else { dash.clone() } );
          }
        }
        builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
      }
    }
  }

  let view  = builder.build_view();
  let table = Format::format( &TableFormatter::new(), &view ).unwrap_or_default();
  let body  = format!( "Quota\n\n{table}\n" );

  // Footer: shown only when ≥2 valid accounts (AC-09 from 023_next_account_strategies.md).
  let valid_count = accounts.iter().filter( |aq| aq.result.is_ok() ).count();
  if valid_count < 2 { return body; }

  // Responsibility(TSK-184-footer): unconditional 2-strategy footer (Endurance, Drain).
  // Both lines shown when valid_count >= 2; NOT gated on next:: value.
  // The → marker in the table body is already placed on the active-strategy winner.
  {
    use core::fmt::Write as _;
    let strategies = [ NextStrategy::Endurance, NextStrategy::Drain ];
    let names      = [ "endurance", "drain" ];
    let mut lines  = String::new();
    for ( strategy, name ) in strategies.iter().zip( names.iter() )
    {
      if let Some( idx ) = find_next_for_strategy( accounts, *strategy, prefer, now_secs )
      {
        let rec      = &accounts[ idx ];
        let metric   = strategy_metric( rec, *strategy, prefer, now_secs );
        let rec_name = &rec.name;
        let _ = writeln!( lines, "  {name:<10}{rec_name}   {metric}" );
      }
    }
    if lines.is_empty() { return body; }
    let total = accounts.len();
    format!( "{body}Valid: {valid_count} / {total}   ->  Next by strategy:\n{lines}" )
  }
}

/// Render quota results as a JSON array (one object per account).
///
/// Every row includes `expires_in_secs`. Successful accounts include quota
/// fields using `_left_pct` naming (remaining, not consumed); failed accounts
/// include `error`.
fn render_json( accounts : &[ AccountQuota ] ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };

  if accounts.is_empty()
  {
    return "[]\n".to_string();
  }

  let now_secs = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  let mut parts = Vec::with_capacity( accounts.len() );
  for aq in accounts
  {
    let name_esc         = json_escape( &aq.name );
    let is_current_str   = if aq.is_current { "true" } else { "false" };
    let is_active_str    = if aq.is_active  { "true" } else { "false" };
    let expires_in_secs  = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
    let billing_type_str = aq.account.as_ref()
      .map_or_else( || "null".to_string(), |a| format!( "\"{}\"", json_escape( &a.billing_type ) ) );
    let has_max_str      = aq.account.as_ref()
      .map_or( "null", |a| if a.has_max { "true" } else { "false" } );
    let next_renewal_str = aq.account.as_ref()
      .map_or_else( || "null".to_string(), |a| format!( "\"{}\"", json_escape( &next_billing_label( &a.org_created_at, now_secs ) ) ) );
    let entry = match &aq.result
    {
      Ok( data ) =>
      {
        // Helpers: Option<f64> utilization → "{:.0}" percent or "null";
        //          Option<&str> ISO reset  → seconds-until-reset or "null".
        let pct_str   = |util : Option< f64 >| -> String
        {
          util.map_or_else( || "null".to_string(), |u| format!( "{:.0}", 100.0 - u ) )
        };
        let reset_str = |iso : Option< &str >| -> String
        {
          iso.and_then( claude_quota::iso_to_unix_secs )
            .map_or_else( || "null".to_string(), |t| t.saturating_sub( now_secs ).to_string() )
        };
        let session_pct   = pct_str( data.five_hour.as_ref().map( |p| p.utilization ) );
        let session_reset = reset_str( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) );
        let weekly_pct    = pct_str( data.seven_day.as_ref().map( |p| p.utilization ) );
        let sonnet_pct    = pct_str( data.seven_day_sonnet.as_ref().map( |p| p.utilization ) );
        let weekly_reset  = reset_str( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) );
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\"next_renewal_est\":{next_renewal_str},\
\"session_5h_left_pct\":{session_pct},\"session_5h_resets_in_secs\":{session_reset},\
\"weekly_7d_left_pct\":{weekly_pct},\"weekly_7d_sonnet_left_pct\":{sonnet_pct},\
\"weekly_7d_resets_in_secs\":{weekly_reset}}}",
        )
      }
      Err( reason ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\
\"billing_type\":{billing_type_str},\"has_max\":{has_max_str},\"next_renewal_est\":{next_renewal_str},\
\"error\":\"{}\"}}",
          json_escape( reason ),
        )
      }
    };
    parts.push( entry );
  }

  format!( "[\n  {}\n]\n", parts.join( ",\n  " ) )
}

// ── Live monitor mode ──────────────────────────────────────────────────────────

/// Shared quit flag — set to `true` by `on_sigint` on SIGINT; polled each second.
static STOP_FLAG : core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new( false );

/// SIGINT handler: sets `STOP_FLAG` so the countdown loop exits cleanly.
extern "C" fn on_sigint( _ : std::os::raw::c_int )
{
  STOP_FLAG.store( true, core::sync::atomic::Ordering::Relaxed );
}

/// Format a Unix timestamp as `HH:MM:SS` in UTC (no external dep).
fn secs_to_hms_utc( unix_secs : u64 ) -> String
{
  let sod = unix_secs % 86400;
  let h   = sod / 3600;
  let m   = ( sod % 3600 ) / 60;
  let s   = sod % 60;
  format!( "{h:02}:{m:02}:{s:02}" )
}

/// Continuous quota monitor loop.
///
/// Clears the screen, fetches all accounts with per-account stagger delays,
/// renders the table, displays a countdown footer rewritten in-place each second,
/// and repeats until Ctrl-C (SIGINT) sets `STOP_FLAG`.
///
/// Timing is governed by `params.interval` (minimum seconds between cycles, ≥ 30)
/// and `params.jitter` (maximum random seconds added per cycle, 0 = none).
/// When `params.trace` is `true`, per-account `[trace]` lines are emitted to stderr.
#[ allow( unsafe_code ) ]
fn execute_live_mode(
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  params           : &UsageParams,
) -> Result< OutputData, ErrorData >
{
  use std::os::raw::{ c_int, c_void };
  use core::sync::atomic::Ordering;
  use std::time::{ SystemTime, UNIX_EPOCH };
  use std::io::Write;

  type SignalFn = extern "C" fn( c_int );
  extern "C"
  {
    fn signal     ( signum : c_int, handler : SignalFn ) -> usize;
    fn sigprocmask( how : c_int, set : *const c_void, oldset : *mut c_void ) -> c_int;
    fn sigemptyset( set : *mut c_void ) -> c_int;
    fn sigaddset  ( set : *mut c_void, signum : c_int ) -> c_int;
  }

  // Reset STOP_FLAG before registering the handler (safe across sequential test runs).
  STOP_FLAG.store( false, Ordering::Relaxed );
  // Unblock SIGINT: test runners (nextest) block SIGINT in their own mask; child processes
  // inherit this blocked mask.  A blocked signal is never delivered even with a registered
  // handler, so the STOP_FLAG is never set and the monitor loops forever.
  // Fix: explicitly unblock SIGINT before registering the handler.
  // sigset_t on Linux = 128 bytes, represented as [u64; 16].
  let mut sigset = [ 0u64; 16 ];
  // SAFETY: `on_sigint` is a valid C-compatible function pointer.
  //         `sigset` is zero-initialised and large enough for sigset_t on Linux.
  unsafe
  {
    sigemptyset( sigset.as_mut_ptr().cast::< c_void >() );
    sigaddset  ( sigset.as_mut_ptr().cast::< c_void >(), 2 );  // 2 = SIGINT
    sigprocmask( 1, sigset.as_ptr().cast::< c_void >(), core::ptr::null_mut() ); // 1 = SIG_UNBLOCK
    signal( 2, on_sigint );
  }

  loop
  {
    if STOP_FLAG.load( Ordering::Relaxed ) { break; }

    // Clear terminal and move cursor to top-left on each cycle.
    print!( "\x1B[2J\x1B[H" );
    let _ = std::io::stdout().flush();

    // Fetch with per-account stagger delays (thunder-herd mitigation).
    let accounts = fetch_all_quota( credential_store, live_creds_file, true, params.trace )?;

    let text = render_text( &accounts, params.sort, params.desc, params.prefer, params.next, &params.cols );
    print!( "{text}" );

    // Compute next-refresh wall-clock time.
    let now_secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
    let jitter_extra = if params.jitter > 0
    {
      let nanos = u64::from( SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().subsec_nanos() );
      nanos % ( params.jitter + 1 ) // 0..=jitter seconds
    }
    else
    {
      0
    };
    let wait_secs = params.interval + jitter_extra;
    let next_at   = now_secs + wait_secs;

    // Countdown footer — rewritten in-place each second via \r.
    let mut remaining = wait_secs;
    loop
    {
      if STOP_FLAG.load( Ordering::Relaxed ) { break; }
      let next_hms = secs_to_hms_utc( next_at );
      let m        = remaining / 60;
      let s        = remaining % 60;
      let line     = format!( "  Next update in {m}:{s:02} (at {next_hms} UTC)  [Ctrl-C to exit]" );
      // Right-pad to 80+ chars to erase leftover characters from a previous longer line.
      print!( "\r{line:<80}" );
      let _ = std::io::stdout().flush();
      if remaining == 0 { break; }
      remaining -= 1;
      std::thread::sleep( core::time::Duration::from_secs( 1 ) );
    }
    println!();

    if STOP_FLAG.load( Ordering::Relaxed ) { break; }
  }

  println!( "\nMonitor stopped." );
  Ok( OutputData::new( String::new(), "text" ) )
}

// ── Refresh helper ─────────────────────────────────────────────────────────────

/// Return `true` when `apply_refresh` should attempt a token refresh for `aq`.
///
/// Triggers on:
/// - 401 or 403 — authentication failure; token rejected by the server.
/// - 429 AND locally expired (`expires_at_ms / 1000 ≤ now_secs`) — the per-account
///   credentials file may be stale (Claude Code updated the live session file but not
///   the saved per-account copy). Refreshing updates both the token and `expiresAt`.
///
/// Returns `false` for 429 with a non-expired local token: the token is valid;
/// refreshing would add a 30-second subprocess wait with no benefit.
fn should_refresh( aq : &AccountQuota, now_secs : u64 ) -> bool
{
  if matches!( aq.result, Err( ref e ) if e.contains( "401" ) || e.contains( "403" ) )
  {
    return true;
  }
  // Fix(issue-156): also refresh when rate-limited AND locally expired.
  // Root cause: 429+expired accounts were unconditionally excluded; the guard
  //   assumed "429 = valid token" but a past `expiresAt` indicates the per-account
  //   file may be stale — the token may need refreshing despite the 429 response.
  // Pitfall: don't refresh ALL 429 accounts (as task 142 did) — that adds a
  //   pointless 30-second wait for valid-but-rate-limited accounts.
  matches!( aq.result, Err( ref e ) if e.contains( "429" ) )
    && ( aq.expires_at_ms / 1000 ) <= now_secs
}

/// Retry quota fetch for accounts that need token refresh (401/403 auth errors,
/// or 429 rate-limit with locally-expired credentials).
///
/// Uses the account lifecycle when `claude_paths` is available: `switch_account` copies
/// the named account's credentials to the live session, the isolated subprocess refreshes
/// the token via an API call side-effect, then `save` propagates the updated credentials
/// back to the persistent store and all companion files.  Falls back to direct persistent-
/// store reads/writes when `claude_paths` is `None`.  Mutates `accounts` in place.
///
/// Fix(issue-150) — HTTP 429 removed from unconditional retry guard.
/// Root cause: HTTP 429 is a rate-limit response, not an authentication failure.
/// Pitfall: Task 142 added 429 unconditionally; task 150 removed it. The correct
/// behaviour (issue-156) is to refresh only when 429 AND locally expired.
fn apply_refresh(
  accounts         : &mut [ AccountQuota ],
  credential_store : &std::path::Path,
  claude_paths     : Option< &crate::ClaudePaths >,
  trace            : bool,
)
{
  let now_secs = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();

  // Snapshot active account to restore after cycling through per-account refreshes.
  let original_active = std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) ).ok();

  for aq in accounts
  {
    let should_retry = should_refresh( aq, now_secs );
    if trace
    {
      let reason = aq.result.as_ref().err().map_or( "ok", String::as_str );
      eprintln!( "[trace] refresh  {}  should_retry={} (reason: {})", aq.name, should_retry, reason );
    }
    if !should_retry { continue; }

    if trace { eprintln!( "[trace] refresh  {}  attempting token refresh", aq.name ); }
    let Some( new_creds ) = crate::account::refresh_account_token(
      &aq.name, credential_store, claude_paths, trace,
    )
    else
    {
      if trace
      {
        eprintln!( "[trace] refresh  {}  refresh returned None — skipping retry", aq.name );
      }
      continue;
    };

    // Fix(issue-162): derive expiry from JWT exp claim — subprocess does not update expiresAt.
    // Root cause: the isolated subprocess writes refreshed accessToken/refreshToken but leaves
    //   expiresAt at the original expired timestamp; re-reading from file gives stale value.
    // Pitfall: expiresAt is a server-issued claim the subprocess cannot update; always derive
    //   post-refresh expiry from jwt_exp_ms(), never by re-reading the credentials file.
    if let Some( exp_ms ) = jwt_exp_ms( &new_creds )
    {
      aq.expires_at_ms = exp_ms;
    }
    // Fix(BUG-170): fallback to expiresAt field for opaque sk-ant-oat01-* tokens.
    // Root cause: jwt_exp_ms returns None for tokens with no '.' separator (not a JWT);
    //   the if-let above never fires, leaving aq.expires_at_ms at the stale pre-refresh value.
    // Pitfall: use else-if (not a second if-let) — only update from expiresAt when JWT decode
    //   fails; a separate if-let would run even on JWT success and silently overwrite with the
    //   expiresAt field value, which may differ from the JWT exp claim by clock skew.
    else if let Some( exp_ms ) = parse_u64_from_str( &new_creds, "expiresAt" )
    {
      aq.expires_at_ms = exp_ms;
    }

    // Re-read the refreshed token and retry only this account's quota.
    if trace { eprintln!( "[trace] refresh  {}  token refreshed, retrying quota fetch", aq.name ); }
    let Ok( token ) = read_token( credential_store, &aq.name ) else { continue; };
    match claude_quota::fetch_oauth_usage( &token )
    {
      Ok( retried ) =>
      {
        if trace { eprintln!( "[trace] refresh  {}  retry OK", aq.name ); }
        aq.result = Ok( retried );
        // Fix(BUG-171): account must be re-fetched after refresh; initial fetch used
        //   the expired token; quota fetch path and account fetch path diverged.
        // Root cause: fetch_oauth_account was added to fetch_all_quota later than apply_refresh;
        //   the refresh retry path never had a corresponding account re-fetch.
        // Pitfall: use if-let, not unconditional .ok() assignment — preserve existing value
        //   on network failure; aq.account = fetch_oauth_account(...).ok() silently destroys
        //   previously-populated account data on transient errors.
        if let Ok( acct ) = claude_quota::fetch_oauth_account( &token )
        {
          aq.account = Some( acct );
        }
      }
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] refresh  {}  retry Err({})", aq.name, e ); }
        // Fix(issue-156): propagate the retry error to show the current post-refresh status.
        // Root cause: on retry failure the original error (e.g. "401 expired") was kept,
        //   hiding the actual post-refresh state (e.g. "429 rate-limited after refresh").
        // Pitfall: ignoring the retry error masks the true current state after refresh.
        aq.result = Err( e.to_string() );
      }
    }
  }

  // Restore original active account after cycling through per-account refreshes.
  if let ( Some( original ), Some( paths ) ) = ( original_active.as_deref(), claude_paths )
  {
    let name = original.trim();
    if !name.is_empty()
    {
      let _ = crate::account::switch_account( name, credential_store, paths );
    }
  }
}

// ── Touch helper ───────────────────────────────────────────────────────────────

/// Activate an idle 5h session window for `aq` by spawning an isolated subprocess.
///
/// The trigger requires both conditions:
/// - `aq.result.is_ok()` — account must have valid quota data (not an auth error).
/// - `five_hour.resets_at.is_none()` — 5h window not yet started (rendered as `—`).
///
/// After a successful touch, quota is re-fetched so the table shows the concrete
/// `5h Reset` value. If the subprocess or re-fetch fails the account row is unchanged
/// (touch failure is non-aborting — other accounts and the render continue normally).
///
/// The original active account is restored unconditionally inside this call before
/// using the new credentials. This prevents a stale active marker if the process is
/// interrupted between touches.
fn apply_touch(
  aq               : &mut AccountQuota,
  credential_store : &std::path::Path,
  claude_paths     : Option< &crate::ClaudePaths >,
  trace            : bool,
)
{
  // Guard: errored accounts are never touched; trigger requires valid quota data.
  let Ok( ref data ) = aq.result else { return; };

  // Guard: accounts with an active 5h window are not idle — skip.
  let is_idle = data.five_hour.as_ref()
    .and_then( |p| p.resets_at.as_deref() )
    .is_none();
  if !is_idle { return; }

  // Save active account before switching for the subprocess lifecycle.
  let original_active = std::fs::read_to_string( credential_store.join( crate::account::active_marker_filename() ) ).ok();

  let new_creds = crate::account::refresh_account_token(
    &aq.name, credential_store, claude_paths, trace,
  );

  // CRITICAL: restore active marker unconditionally before using new_creds (Fix(BUG-170) pattern).
  // If restoration is deferred past the return points below, an interrupted touch leaves
  // the active marker pointing at the touched account instead of the original.
  if let ( Some( original ), Some( paths ) ) = ( original_active.as_deref(), claude_paths )
  {
    let name = original.trim();
    if !name.is_empty()
    {
      let _ = crate::account::switch_account( name, credential_store, paths );
    }
  }

  let Some( creds ) = new_creds else { return; };

  // Update expiry using JWT exp field with expiresAt fallback (same as apply_refresh).
  if let Some( exp_ms ) = jwt_exp_ms( &creds )
  {
    aq.expires_at_ms = exp_ms;
  }
  else if let Some( exp_ms ) = parse_u64_from_str( &creds, "expiresAt" )
  {
    aq.expires_at_ms = exp_ms;
  }

  // Re-read token AFTER subprocess — the pre-subprocess token is stale.
  let Ok( token ) = read_token( credential_store, &aq.name ) else { return; };
  if let Ok( new_data ) = claude_quota::fetch_oauth_usage( &token )
  {
    aq.result = Ok( new_data );
    if let Ok( acct ) = claude_quota::fetch_oauth_account( &token )
    {
      aq.account = Some( acct );
    }
  }
}

// ── Command handler ────────────────────────────────────────────────────────────

/// Parsed `.usage` parameters extracted from a `VerifiedCommand`.
struct UsageParams
{
  /// 1 = auto-refresh expired tokens (default); 0 = show errors as-is.
  refresh  : i64,
  /// 1 = continuous live-monitor loop; 0 = single fetch (default).
  live     : i64,
  /// Seconds between live-loop cycles (default 30; only validated when live=1).
  interval : u64,
  /// Max random seconds added to each cycle (default 0; only validated when live=1).
  jitter   : u64,
  /// true = emit `[trace]` diagnostic lines to stderr.
  trace    : bool,
  /// Row ordering strategy for the text table.
  sort     : SortStrategy,
  /// Sort direction override; `None` = use strategy's context-sensitive default.
  desc     : Option< bool >,
  /// Weekly quota column selector for strategies that reference weekly availability.
  prefer   : PreferStrategy,
  /// Recommendation strategy controlling `→` marker and footer format.
  next     : NextStrategy,
  /// Column visibility modifiers applied to the text table.
  cols     : ColsVisibility,
  /// 1 = activate idle 5h session windows via subprocess; 0 = off (default).
  touch    : i64,
}

/// Parse and validate the five `.usage`-specific parameters.
///
/// # Errors
///
/// Returns `ErrorData` (exit 1 / `ArgumentTypeMismatch`) for any out-of-range
/// or wrong-type value. `interval` and `jitter` constraint validation is deferred
/// to `usage_routine` because it only applies when `live = 1`.
///
/// Fix(issue-155): `refresh` default is 1 (enabled). Omitting the param ≠
/// "user wants disabled" — auto-refresh is the safer default.
/// Fix(issue-157): strict 0/1 range guard added for `refresh`, `live`, `trace`.
/// Pitfall: bool-typed params (e.g. `touch::`) use `Kind::String` registration so
/// `"true"`/`"false"` pass through; `parse_int_flag` is the sole normalisation point.
/// Parse an integer `0`-or-`1` flag from `cmd.arguments` with a configurable default.
///
/// Returns `default` when absent; rejects non-`Value::Integer` values or integers outside
/// `{0, 1}` with `ArgumentTypeMismatch`.
///
/// Pitfall: params registered as `Kind::String` (e.g. `touch::`) deliver all values
/// including `"0"` and `"1"` as `Value::String` — the integer arms handle `Kind::Integer` params.
fn parse_int_flag( cmd : &VerifiedCommand, name : &str, default : i64 ) -> Result< i64, ErrorData >
{
  match cmd.arguments.get( name )
  {
    None                                       => Ok( default ),
    Some( Value::Integer( 0 ) )                => Ok( 0 ),
    Some( Value::Integer( 1 ) )                => Ok( 1 ),
    Some( Value::String( s ) ) if s == "0"     => Ok( 0 ),
    Some( Value::String( s ) ) if s == "1"     => Ok( 1 ),
    Some( Value::String( s ) ) if s == "true"  => Ok( 1 ),
    Some( Value::String( s ) ) if s == "false" => Ok( 0 ),
    _ => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "{name}:: must be 0 or 1" ),
    ) ),
  }
}

fn parse_usage_params( cmd : &VerifiedCommand ) -> Result< UsageParams, ErrorData >
{
  // refresh default is 1 (enabled); live/trace/touch default is 0 (disabled).
  let refresh = parse_int_flag( cmd, "refresh", 1 )?;
  let live    = parse_int_flag( cmd, "live",    0 )?;
  let trace   = parse_int_flag( cmd, "trace",   0 )? != 0;
  let touch   = parse_int_flag( cmd, "touch",   0 )?;
  // Negative values map to 0, which is < 30 and will hit the interval guard.
  let interval = match cmd.arguments.get( "interval" )
  {
    None                        => 30_u64,
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "interval:: must be a non-negative integer".to_string(),
    ) ),
  };
  let jitter = match cmd.arguments.get( "jitter" )
  {
    None                        => 0_u64,
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "jitter:: must be a non-negative integer".to_string(),
    ) ),
  };
  let sort = match cmd.arguments.get( "sort" )
  {
    None                         => SortStrategy::Reset,
    Some( Value::String( s ) )   => SortStrategy::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "sort:: must be a string".to_string(),
    ) ),
  };
  let desc_param = match cmd.arguments.get( "desc" )
  {
    None                        => None,
    Some( Value::Integer( 0 ) ) => Some( false ),
    Some( Value::Integer( 1 ) ) => Some( true ),
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "desc:: must be 0 or 1".to_string(),
    ) ),
  };
  let prefer = match cmd.arguments.get( "prefer" )
  {
    None                         => PreferStrategy::Any,
    Some( Value::String( s ) )   => PreferStrategy::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "prefer:: must be a string".to_string(),
    ) ),
  };
  let next = match cmd.arguments.get( "next" )
  {
    None                         => NextStrategy::Endurance,
    Some( Value::String( s ) )   => NextStrategy::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "next:: must be a string".to_string(),
    ) ),
  };
  let cols = match cmd.arguments.get( "cols" )
  {
    None                         => ColsVisibility::default_set(),
    Some( Value::String( s ) )   => ColsVisibility::parse( s ).map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?,
    _ => return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "cols:: must be a string".to_string(),
    ) ),
  };
  Ok( UsageParams { refresh, live, interval, jitter, trace, sort, desc : desc_param, prefer, next, cols, touch } )
}

/// `.usage` — show live quota utilization for all saved accounts.
///
/// Enumerates `{credential_store}/*.credentials.json`, fetches rate-limit
/// headers per account, and renders a `data_fmt` table (or JSON array with
/// `format::json`).
///
/// # Errors
///
/// Returns `ErrorData` (exit 2) if HOME/PRO is unset or the credential store
/// exists but cannot be read. Per-account API errors are displayed inline.
#[ inline ]
pub fn usage_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts   = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let params = parse_usage_params( &cmd )?;

  // Live-mode guards — fire BEFORE any network fetch, only when live::1 (AC-31).
  // Pitfall: placing these inside execute_live_mode() (after fetch_all_quota) would
  // require live credentials for offline guard tests it22–it24.
  if params.live == 1
  {
    if matches!( opts.format, OutputFormat::Json )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "live monitor mode is incompatible with format::json".to_string(),
      ) );
    }
    if params.interval < 30
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "interval must be >= 30".to_string(),
      ) );
    }
    if params.jitter > params.interval
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "jitter must not exceed interval".to_string(),
      ) );
    }
  }

  let persist_paths    = crate::PersistPaths::new()
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot resolve storage root: {e}" ),
    ) )?;
  let credential_store = persist_paths.credential_store();
  let live_creds_file  = crate::ClaudePaths::new()
    .map_or_else( || std::path::PathBuf::from( "/dev/null" ), |p| p.credentials_file() );

  if params.live == 1
  {
    return execute_live_mode( &credential_store, &live_creds_file, &params );
  }

  let mut accounts = fetch_all_quota( &credential_store, &live_creds_file, false, params.trace )?;

  // Retry-once per account on 401/403 auth errors or 429+locally-expired: if
  // refresh::1 and any account's quota fetch failed with an auth error OR a
  // rate-limit response while its local `expiresAt` is past, refresh that token
  // via an isolated subprocess, then re-fetch only that account's quota.
  // Pure 429 with a non-expired local token is not retried — the token is valid.
  if params.refresh == 1
  {
    let claude_paths = crate::ClaudePaths::new();
    apply_refresh( &mut accounts, &credential_store, claude_paths.as_ref(), params.trace );
  }

  // touch::1: activate idle 5h windows — runs after refresh so post-refresh results
  // are touched (an account that was refreshed and now has valid quota with no resets_at
  // will be touched; an account that still errors after refresh is skipped by apply_touch).
  if params.touch == 1
  {
    let claude_paths = crate::ClaudePaths::new();
    for aq in &mut accounts
    {
      apply_touch( aq, &credential_store, claude_paths.as_ref(), params.trace );
    }
  }

  let content = match opts.format
  {
    OutputFormat::Json  => render_json( &accounts ),
    OutputFormat::Text
    | OutputFormat::Table => render_text( &accounts, params.sort, params.desc, params.prefer, params.next, &params.cols ),
  };

  Ok( OutputData::new( content, "text" ) )
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use tempfile::TempDir;

  // ── shorten_error ──────────────────────────────────────────────────────────

  /// T04 — `shorten_error` maps HTTP 429 transport string to the compact label.
  ///
  /// # Root Cause
  /// `apply_refresh` had HTTP 429 in its retry guard condition. HTTP 429 is a
  /// rate-limit response, not an auth failure; the token is still valid. Task 142
  /// added the 429 code to the guard by mistake; task 143 removes it and adds a
  /// `shorten_error` branch so the table shows a compact label instead of the
  /// verbose transport string.
  ///
  /// # Why Not Caught
  /// No existing test covered this string — `shorten_error` only had a single
  /// branch for `"rate-limit header missing:"`.
  ///
  /// # Fix Applied
  /// Added `"HTTP transport error: HTTP 429"` → `"rate limited (429)"` branch to
  /// `shorten_error()` before the pass-through else.
  ///
  /// # Prevention
  /// This test acts as a regression guard: if the branch is removed, the function
  /// returns the verbose 40-character string and this assertion fails.
  ///
  /// # Pitfall
  /// The match is an exact prefix check — `starts_with` — so partial or differently
  /// formatted 429 strings would still pass through. Only
  /// `claude_quota::QuotaError::HttpTransport` formats as `"HTTP transport error: HTTP N"`.
  // test_kind: bug_reproducer(issue-150)
  #[ test ]
  fn test_shorten_error_429_returns_rate_limited()
  {
    assert_eq!(
      shorten_error( "HTTP transport error: HTTP 429" ),
      "rate limited (429)",
    );
  }

  /// T05 — `shorten_error` must return `"auth expired (401)"` for HTTP 401 transport strings.
  ///
  /// # Root Cause
  /// `shorten_error` is an explicit allowlist. When task 150 added the HTTP 429 branch, it
  /// also added an HTTP 401 case to T05 as a regression guard — but as a pass-through check,
  /// documenting the wrong (non-AC-03) behaviour: HTTP 401 was not shortened.
  /// AC-03 (`docs/feature/009_token_usage.md:116`) requires "a shortened error reason" in the
  /// final column for ALL error cases, not only 429.
  ///
  /// # Why Not Caught
  /// T05 was written to assert the pass-through (current) behaviour, not the AC-03 requirement.
  /// No test verified the AC-03 invariant holistically — that ALL HTTP transport codes are
  /// shortened before reaching the table column.
  ///
  /// # Fix Applied
  /// Added `else if reason.starts_with( "HTTP transport error: HTTP 401" ) { "auth expired (401)" }`
  /// branch in `shorten_error()` between the 429 branch and the `"rate-limit header missing:"`
  /// branch. Fix(BUG-152).
  ///
  /// # Prevention
  /// `test_shorten_error_no_raw_http_transport_passthrough` asserts that no `"HTTP transport
  /// error:"` string passes through `shorten_error` unchanged. This test will fail for any
  /// future unshortened HTTP code, catching the gap early.
  ///
  /// # Pitfall
  /// `shorten_error` is a manual allowlist — each new HTTP error code from
  /// `QuotaError::HttpTransport` needs an explicit branch. The `else { reason }` arm is NOT
  /// a shortener; it is a verbatim passthrough. A new auth-failure code (e.g., 403) that the
  /// quota API might return in the future would silently appear in full in the table.
  // test_kind: bug_reproducer(issue-152)
  #[ test ]
  fn test_shorten_error_mre_401_shortened()
  {
    assert_eq!(
      shorten_error( "HTTP transport error: HTTP 401" ),
      "auth expired (401)",
      "HTTP 401 transport string must be shortened per AC-03 (BUG-152)",
    );
  }

  /// T06 — `shorten_error` maps HTTP 403 transport string to compact label.
  ///
  /// HTTP 403 (Forbidden) is returned by the usage API as a permission error and is handled
  /// by `apply_refresh` as an auth-failure trigger. Without `refresh::1`, a 403 error would
  /// previously appear verbatim as "(HTTP transport error: HTTP 403)" in the table column,
  /// violating AC-03 ("shortened error reason"). This branch shortens it to "auth forbidden (403)".
  // test_kind: regression_guard
  #[ test ]
  fn test_shorten_error_403_returns_auth_forbidden()
  {
    assert_eq!(
      shorten_error( "HTTP transport error: HTTP 403" ),
      "auth forbidden (403)",
      "HTTP 403 transport string must be shortened per AC-03",
    );
  }

  /// Invariant — `shorten_error` must never return a raw `"HTTP transport error:"` string
  /// for HTTP error codes that appear in the current shortening allowlist.
  ///
  /// When adding a new HTTP error code to `claude_quota` fetch paths AND to `shorten_error`,
  /// add it to `shortened_codes` here too.
  #[ test ]
  fn test_shorten_error_no_raw_http_transport_passthrough()
  {
    // All codes with explicit branches in shorten_error are listed here.
    let shortened_codes = &[
      "HTTP transport error: HTTP 401",  // Fix(BUG-152): "auth expired (401)"
      "HTTP transport error: HTTP 403",  // "auth forbidden (403)" — usage API permission error
      "HTTP transport error: HTTP 429",  // task 150: "rate limited (429)"
    ];
    for &e in shortened_codes
    {
      let shortened = shorten_error( e );
      assert!(
        !shortened.starts_with( "HTTP transport error:" ),
        "shorten_error must shorten {e:?}; got verbatim passthrough {shortened:?}",
      );
    }
  }

  /// C6 regression — existing `"rate-limit header missing:"` branch still works.
  #[ test ]
  fn test_shorten_error_no_header_preserved()
  {
    assert_eq!( shorten_error( "rate-limit header missing: X-RateLimit-Remaining" ), "no header" );
  }

  /// A5 — empty string passes through `shorten_error` unchanged.
  #[ test ]
  fn test_shorten_error_empty_passthrough()
  {
    assert_eq!( shorten_error( "" ), "" );
  }

  /// A6 — arbitrary non-matching string passes through `shorten_error` unchanged.
  #[ test ]
  fn test_shorten_error_arbitrary_passthrough()
  {
    assert_eq!( shorten_error( "network timeout" ), "network timeout" );
  }

  // ── apply_refresh ──────────────────────────────────────────────────────────

  /// T01 — `apply_refresh` leaves a 429 error result unchanged (no retry path).
  ///
  /// # Root Cause
  /// In task 142, `apply_refresh`'s retry guard included `e.contains("429")` alongside
  /// `"401"` and `"403"`. HTTP 429 is a rate-limit response (token is still valid); retrying
  /// on 429 triggers an unnecessary token refresh. Task 143 removed 429 from the guard at
  /// `usage.rs` line 634, leaving only auth-failure codes (401, 403) as retry triggers.
  ///
  /// # Why Not Caught
  /// No test existed for `apply_refresh` behavior with 429 errors before task 143; the guard
  /// was added in task 142 without a companion test proving 429 is passed through unchanged.
  ///
  /// # Fix Applied
  /// Removed `e.contains("429")` from the retry guard; guard is now
  /// `Err(ref e) if e.contains("401") || e.contains("403")` only.
  ///
  /// # Prevention
  /// This test verifies the result string is identical after `apply_refresh`, acting as a
  /// regression guard against re-adding 429 to the retry trigger conditions.
  ///
  /// # Pitfall
  /// Without a credential file in the store, the retry body is unreachable regardless of the
  /// guard — `apply_refresh` cannot attempt a refresh and leaves the result unchanged either
  /// way. This test validates the guard does not corrupt the result, but is NOT a full guard
  /// against re-adding 429: even with the bug restored, this test would still pass (no creds).
  /// The `shorten_error` test (T04) provides the stronger behavioral invariant.
  // test_kind: bug_reproducer(issue-150)
  #[ test ]
  fn test_apply_refresh_429_not_retried()
  {
    let store = TempDir::new().unwrap();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "test-acct".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), None, false );

    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e == "HTTP transport error: HTTP 429" ),
      "429 error must be unchanged after apply_refresh; result: {:?}", accounts[ 0 ].result,
    );
  }

  /// B2 — `apply_refresh` does not corrupt a successful Ok result.
  ///
  /// An account with a valid quota result must remain Ok after `apply_refresh`;
  /// the guard only fires on Err results containing "401" or "403".
  #[ test ]
  fn test_apply_refresh_ok_result_unchanged()
  {
    let store = TempDir::new().unwrap();
    let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![
      AccountQuota
      {
        name          : "ok-acct".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Ok( quota ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), None, false );
    assert!( accounts[ 0 ].result.is_ok(), "Ok result must not be changed by apply_refresh" );
  }

  /// B3 — `apply_refresh` leaves a generic network error unchanged (not an auth error).
  ///
  /// Only "401" and "403" substrings trigger the retry guard; unrelated error
  /// strings pass through without entering the retry path.
  #[ test ]
  fn test_apply_refresh_generic_error_unchanged()
  {
    let store   = TempDir::new().unwrap();
    let err_msg = "network timeout after 30s".to_string();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "net-acct".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( err_msg.clone() ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), None, false );
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e == &err_msg ),
      "generic error must be unchanged; result: {:?}", accounts[ 0 ].result,
    );
  }

  // ── apply_refresh: corner cases ─────────────────────────────────────────────

  /// C1 — `apply_refresh` on an empty accounts slice is a no-op.
  #[ test ]
  fn test_apply_refresh_empty_accounts()
  {
    let store = TempDir::new().unwrap();
    let mut accounts : Vec< AccountQuota > = vec![];
    apply_refresh( &mut accounts, store.path(), None, false );
    assert!( accounts.is_empty(), "empty slice must remain empty" );
  }

  /// C2 / FT-14 — `apply_refresh` `None`-paths: 401 + no credential file → result unchanged.
  ///
  /// `should_refresh` fires (`should_retry=true`); `crate::account::refresh_account_token`
  /// is called with `paths=None`; internally it reads `{store}/{name}.credentials.json`
  /// which is absent, so it returns `None`; `apply_refresh` skips the account via
  /// `continue` without modifying the result.
  #[ test ]
  fn test_apply_refresh_401_no_cred_file()
  {
    let store = TempDir::new().unwrap();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "ghost@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), None, false );
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "401" ) ),
      "401 with no cred file must be unchanged; result: {:?}", accounts[ 0 ].result,
    );
  }

  /// C3 — `apply_refresh` with 403 error but no credential file on disk.
  ///
  /// Same as C2 but with HTTP 403. Both 401 and 403 are auth-error triggers,
  /// but without a credential file the retry body is unreachable.
  #[ test ]
  fn test_apply_refresh_403_no_cred_file()
  {
    let store = TempDir::new().unwrap();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "ghost@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 403".to_string() ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), None, false );
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "403" ) ),
      "403 with no cred file must be unchanged; result: {:?}", accounts[ 0 ].result,
    );
  }

  /// C4 / FT-07 — `apply_refresh` with mixed results: refresh failure does not affect siblings.
  ///
  /// Four accounts: Ok, 429+expired (`expires_at_ms=0`), 401, generic error.
  /// After `apply_refresh`, the 401 and the 429+expired accounts enter the retry guard
  /// but stay unchanged (no credential file → `refresh_account_token` returns `None`
  /// → `continue`).  Ok and generic error are untouched (Ok never retries; generic
  /// error has no auth/429 signal).  Implements FT-07: refresh failure in one account
  /// does not corrupt any sibling's result.
  #[ test ]
  fn test_apply_refresh_mixed_accounts()
  {
    let store = TempDir::new().unwrap();
    let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let mut accounts = vec![
      AccountQuota
      {
        name          : "a@ok.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Ok( quota ),
        account       : None,
      },
      AccountQuota
      {
        name          : "b@ratelimited.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
      },
      AccountQuota
      {
        name          : "c@expired.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
      AccountQuota
      {
        name          : "d@network.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "connection refused".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), None, false );

    assert!( accounts[ 0 ].result.is_ok(), "Ok account must remain Ok" );
    assert!(
      matches!( accounts[ 1 ].result, Err( ref e ) if e.contains( "429" ) ),
      "429+expired with no credential file must be unchanged (retry attempted, no cred file → continue)",
    );
    assert!(
      matches!( accounts[ 2 ].result, Err( ref e ) if e.contains( "401" ) ),
      "401 stays unchanged when no cred file exists",
    );
    assert!(
      matches!( accounts[ 3 ].result, Err( ref e ) if e == "connection refused" ),
      "generic error must be unchanged",
    );
  }

  /// C5 — `apply_refresh` with trace=true does not panic.
  ///
  /// Verifies the trace code path executes without crashing, even when the
  /// credential file is absent and the retry path short-circuits.
  #[ test ]
  fn test_apply_refresh_trace_does_not_panic()
  {
    let store = TempDir::new().unwrap();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "trace@test.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), None, true );
  }

  // ── apply_refresh: lifecycle (Some(paths)) ──────────────────────────────────

  /// L1 — `apply_refresh` skips lifecycle path when `switch_account` fails (no cred file).
  ///
  /// # Root Cause
  /// Before BUG-165, `apply_refresh` bypassed `switch_account` entirely, writing credentials
  /// directly to the persistent store while leaving the live session stale. After the fix,
  /// `apply_refresh` calls `switch_account` first when `claude_paths` is `Some`; if it fails
  /// (account not found in store), the account is skipped and its error result is left unchanged.
  ///
  /// # Why Not Caught
  /// All prior inline tests passed `apply_refresh(..., None, ...)`, exercising only the `None`
  /// (fallback/test) branch. Zero tests exercised `Some(paths)` (lifecycle/production branch).
  ///
  /// # Fix Applied
  /// BUG-165 / issue-165: extracted `refresh_account_token` (full lifecycle: switch → refresh →
  /// save); `apply_refresh` delegates via `crate::account::refresh_account_token`; skips the
  /// account with `continue` if `refresh_account_token` returns `None`.
  ///
  /// # Prevention
  /// This test guards the `Some(paths)` early-exit: when the credential file is absent,
  /// `refresh_account_token` returns `None` and `apply_refresh` must `continue` without
  /// corrupting the account result.
  ///
  /// # Pitfall
  /// Tests where the credential file exists will reach `refresh_account_token`, which internally
  /// spawns the `claude` binary and blocks for up to 35 s. Only test scenarios where the
  /// credential file is absent (causing `None` early-exit) to avoid subprocess blocking.
  // test_kind: regression(issue-165)
  #[ test ]
  fn test_apply_refresh_lifecycle_switch_fails_result_unchanged()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    // No alice@example.com.credentials.json in store — switch_account returns NotFound.
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );

    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "401" ) ),
      "lifecycle path: 401 result must be unchanged when switch_account fails; result: {:?}",
      accounts[ 0 ].result,
    );
  }

  /// L2 — `apply_refresh` restores the original active account after the refresh cycle.
  ///
  /// # Root Cause
  /// `apply_refresh` snapshots `original_active` before iterating accounts, then restores it
  /// with `switch_account` after the loop. Without this restore, the active account would
  /// change permanently to whichever account was processed last — breaking the user's session.
  ///
  /// # Why Not Caught
  /// All prior inline tests passed `None` for `claude_paths`. The `None` branch never calls
  /// `switch_account`, so the restore code at `usage.rs:897-904` had zero unit test coverage.
  ///
  /// # Fix Applied
  /// BUG-165 / issue-165: added `original_active` snapshot before the loop and
  /// `switch_account(original_active, store, paths)` restore after the loop.
  ///
  /// # Prevention
  /// This test guards the restore: after a refresh cycle where bob's `switch_account` fails,
  /// the restore runs `switch_account("alice@example.com", ...)` which succeeds (alice has a
  /// cred file), writing alice's creds to the live file and "alice@example.com" to the active marker.
  ///
  /// # Pitfall
  /// The `{fake_home}/.claude/` directory MUST exist before `apply_refresh` is called.
  /// `switch_account` calls `fs::copy(src, tmp)` where `tmp` is inside `{fake_home}/.claude/`;
  /// if the directory is absent, `copy` fails and the restore silently does nothing —
  /// the active marker remains unchanged but for the wrong reason (silent failure, not correct restore).
  // test_kind: regression(issue-165)
  #[ test ]
  fn test_apply_refresh_lifecycle_original_active_restored()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    // Alice's credential file in store — needed for restore switch_account to succeed.
    let alice_creds = r#"{"accessToken":"alice-token"}"#;
    std::fs::write(
      store.path().join( "alice@example.com.credentials.json" ),
      alice_creds,
    ).unwrap();

    // Set active account to alice before the loop.
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), "alice@example.com" ).unwrap();

    // Create {fake_home}/.claude/ so switch_account can write the live credentials file.
    std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();

    let paths = crate::ClaudePaths::with_home( fake_home.path() );

    // Bob has 401 but no credential file — switch_account fails, loop continues to next account.
    let mut accounts = vec![
      AccountQuota
      {
        name          : "bob@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );

    // Restore ran: switch_account("alice@example.com", ...) wrote active marker and live creds.
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
    assert_eq!(
      active, "alice@example.com",
      "per-machine active marker must be restored to original account after refresh cycle",
    );

    let live_creds = std::fs::read_to_string( paths.credentials_file() ).unwrap();
    assert_eq!(
      live_creds, alice_creds,
      "live credentials file must contain alice's creds after restore",
    );
  }

  /// L3 — `apply_refresh` lifecycle: 429+expired + `Some(paths)` + no cred file → skipped.
  ///
  /// 429 with an expired local token meets `should_refresh` but `switch_account` fails
  /// (no cred file in the persistent store), so the account is skipped and the result
  /// is left unchanged — same guarantee as L1 but for the 429+expired trigger path.
  #[ test ]
  fn test_apply_refresh_lifecycle_429_expired_switch_fails_unchanged()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,  // expired: 0/1000=0 <= now_secs
        result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "429" ) ),
      "lifecycle: 429+expired result must be unchanged when switch_account fails; result: {:?}",
      accounts[ 0 ].result,
    );
  }

  /// FT-03 — `apply_refresh` lifecycle: 403 + `Some(paths)` + no cred file → result unchanged.
  ///
  /// 403 meets `should_refresh` (authentication failure, identical to 401) but
  /// `switch_account` fails (no credential file in store), so `refresh_account_token`
  /// returns `None` and `apply_refresh` skips the account via `continue`.  The 403
  /// result is left unchanged — confirms 403 enters the refresh path, not the
  /// non-trigger `continue` guard.
  #[ test ]
  fn test_apply_refresh_lifecycle_ft3_403_no_cred_result_unchanged()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    // No alice@example.com.credentials.json — switch_account returns NotFound.
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : FAR_FUTURE_MS,  // non-expired; 403 triggers regardless of expiry
        result        : Err( "HTTP transport error: HTTP 403".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );

    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "403" ) ),
      "lifecycle: 403 result must be unchanged when switch_account fails; result: {:?}",
      accounts[ 0 ].result,
    );
  }

  /// L4 — `apply_refresh` lifecycle: cred file exists but `{home}/.claude/` dir missing
  /// → `fs::copy` fails inside `switch_account` → account is skipped, result unchanged.
  ///
  /// `switch_account` copies the credential to a temp file inside `{home}/.claude/`.
  /// If that directory does not exist, `fs::copy` returns an `Err`, causing `apply_refresh`
  /// to `continue` without modifying the account result.
  #[ test ]
  fn test_apply_refresh_lifecycle_copy_fails_no_dot_claude_dir()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    // Cred file exists — check_switch_preconditions passes.
    std::fs::write(
      store.path().join( "alice@example.com.credentials.json" ),
      r#"{"accessToken":"tok"}"#,
    ).unwrap();
    // {fake_home}/.claude/ deliberately NOT created → fs::copy target parent missing.
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "401" ) ),
      "lifecycle: 401 result must be unchanged when fs::copy fails (no .claude/ dir); result: {:?}",
      accounts[ 0 ].result,
    );
  }

  /// L5 — `apply_refresh` lifecycle: no active marker file → `original_active = None` → no restore.
  ///
  /// `read_to_string` on the absent active marker file returns `Err`; `.ok()` maps that to `None`.
  /// The restore block requires `Some(original)`, so it is skipped entirely.
  #[ test ]
  fn test_apply_refresh_lifecycle_no_active_file_no_restore()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → no loop body
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );
    assert!(
      !store.path().join( crate::account::active_marker_filename() ).exists(),
      "per-machine active marker must not be created when it was absent before apply_refresh",
    );
  }

  /// L6 — `apply_refresh` lifecycle with `trace=true` and `switch_account` failure does not panic.
  ///
  /// Exercises the trace code path in the `Some(paths)` branch: logs the switch attempt
  /// and the skip message, then returns without crashing.
  #[ test ]
  fn test_apply_refresh_lifecycle_trace_switch_fails_no_panic()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts = vec![
      AccountQuota
      {
        name          : "trace@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];
    // Must not panic — switch_account fails (no cred file), trace logs to stderr.
    apply_refresh( &mut accounts, store.path(), Some( &paths ), true );
  }

  /// L7 — active marker file with trailing newline: `trim()` strips whitespace → correct restore.
  ///
  /// `read_to_string` returns `"alice@example.com\n"`.  `original.trim()` strips the newline,
  /// yielding the valid name used in `switch_account` → restore succeeds.
  #[ test ]
  fn test_apply_refresh_lifecycle_active_newline_trimmed_restore()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    let alice_creds = r#"{"accessToken":"alice-tok"}"#;
    std::fs::write(
      store.path().join( "alice@example.com.credentials.json" ),
      alice_creds,
    ).unwrap();
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), "alice@example.com\n" ).unwrap();
    std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → restore path only
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
    assert_eq!(
      active, "alice@example.com",
      "trailing-newline active marker must be trimmed before restore; active marker after = {active:?}",
    );
  }

  /// L8 — active marker file containing only whitespace: `trim().is_empty()` → restore skipped.
  ///
  /// An active marker file with content `"   \n  "` trims to `""`.  `is_empty()` is `true`,
  /// so `switch_account` is never called and the file content is not modified.
  #[ test ]
  fn test_apply_refresh_lifecycle_active_whitespace_only_no_restore()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    let ws = "   \n  ";
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), ws ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts : Vec< AccountQuota > = vec![];
    apply_refresh( &mut accounts, store.path(), Some( &paths ), false );
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
    assert_eq!(
      active, ws,
      "whitespace-only active marker must not trigger restore; content must be unchanged",
    );
  }

  /// L9 — `claude_paths = None`: restore guard `if let (Some(original), Some(paths))`
  /// short-circuits on `paths = None` → active marker is never modified by restore.
  ///
  /// Verifies the `None` branch guard: an existing active marker file must be unchanged
  /// after `apply_refresh` using the fallback (non-lifecycle) path.
  #[ test ]
  fn test_apply_refresh_none_paths_active_unchanged()
  {
    let store = TempDir::new().unwrap();
    std::fs::write( store.path().join( crate::account::active_marker_filename() ), "alice@example.com" ).unwrap();
    let mut accounts : Vec< AccountQuota > = vec![];  // no accounts → no loop body
    apply_refresh( &mut accounts, store.path(), None, false );
    let active = std::fs::read_to_string( store.path().join( crate::account::active_marker_filename() ) ).unwrap();
    assert_eq!(
      active, "alice@example.com",
      "per-machine active marker must be unchanged when claude_paths=None (no restore possible)",
    );
  }

  /// L10 / FT-15 — `apply_refresh` lifecycle with `trace=true` reaching `run_isolated` invocation.
  ///
  /// `switch_account` succeeds (cred file in store, `.claude/` dir in `fake_home`).
  /// `run_isolated` is invoked but fails fast (no valid claude binary or fake token) →
  /// trace emits `[trace] … run_isolated: Err(…)` or `OK credentials=None` →
  /// `refresh_account_token` returns `None` → account skipped → no panic.
  ///
  /// # Root Cause
  /// Before BUG-166, `refresh_account_token` had no `trace` parameter. The `apply_refresh`
  /// `trace` arg was accepted but never forwarded, making the lifecycle completely opaque:
  /// all failure paths returned `None` silently. Running `clp .usage refresh::1 trace::1`
  /// showed only "refresh returned None — skipping retry" with no step-level detail.
  ///
  /// # Why Not Caught
  /// The trace parameter existed in `apply_refresh` but there were no tests verifying
  /// it actually reached `refresh_account_token`. Silent pass-through was undetectable.
  ///
  /// # Fix Applied
  /// BUG-166: added `trace: bool` as a 4th parameter to `refresh_account_token`;
  /// replaced all bare `?` operators with explicit `match` + `if trace { eprintln!(...) }` blocks.
  ///
  /// # Prevention
  /// This test guards the full call chain: `apply_refresh(trace=true)` →
  /// `refresh_account_token(trace=true)` → `run_isolated` invocation. If the trace
  /// parameter is ever dropped between layers, this test still passes (no panic),
  /// but the trace output would be missing. The `account_refresh_test::art_some_paths_run_isolated_invoked_trace_no_panic`
  /// test covers the `refresh_account_token` function directly.
  ///
  /// # Pitfall
  /// Tests using "does not panic" cannot assert stderr content — nextest does not
  /// capture `eprintln!` output for unit test assertions. This is the correct pattern
  /// for trace tests.
  // test_kind: regression(issue-166)
  #[ test ]
  fn test_apply_refresh_lifecycle_l10_trace_run_isolated_invoked_no_panic()
  {
    let store     = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    // Cred file in store AND .claude/ dir present — switch_account succeeds.
    std::fs::write(
      store.path().join( "alice@example.com.credentials.json" ),
      r#"{"accessToken":"fake-tok","expiresAt":9999999999999}"#,
    ).unwrap();
    std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
    let paths = crate::ClaudePaths::with_home( fake_home.path() );
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];
    // Must not panic — switch_account succeeds; run_isolated invoked; fails fast (fake creds).
    apply_refresh( &mut accounts, store.path(), Some( &paths ), true );
  }

  /// FT-04 — `apply_refresh`: 429 + non-expired local token → NOT retried, result unchanged.
  ///
  /// `should_refresh` returns false when 429+non-expired (`expires_at_ms / 1000 > now_secs`):
  /// the local token is valid; the 429 is a genuine rate-limit, not a stale-credential
  /// condition.  `apply_refresh` skips `refresh_account_token` entirely (early `continue`).
  /// The 429 result is left unchanged.
  #[ test ]
  fn test_apply_refresh_ft4_429_valid_token_not_retried()
  {
    let store = TempDir::new().unwrap();
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : FAR_FUTURE_MS,  // non-expired → 429 is genuine rate-limit
        result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), None, false );

    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "429" ) ),
      "429 with valid (non-expired) token must NOT be retried; result: {:?}",
      accounts[ 0 ].result,
    );
  }

  /// FT-05 — `apply_refresh` `None`-paths: 429 + expired local token → refresh path
  /// entered, but no credential file in store → `refresh_account_token` returns `None`
  /// → account skipped via `continue` → result unchanged.
  ///
  /// Contrasts with FT-04 (`test_apply_refresh_ft4_429_valid_token_not_retried`):
  ///   FT-04: 429 + non-expired → `should_refresh` returns `false` → refresh path NEVER entered.
  ///   FT-05: 429 + expired    → `should_refresh` returns `true`  → refresh path IS entered,
  ///          but gracefully exits when no per-account credential file exists in the store.
  #[ test ]
  fn test_apply_refresh_ft5_429_expired_refresh_path_entered_no_cred()
  {
    let store = TempDir::new().unwrap();
    // expires_at_ms=0 → 0/1000=0 ≤ now_secs → locally expired → should_refresh=true for 429.
    let mut accounts = vec![
      AccountQuota
      {
        name          : "alice@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
      },
    ];

    apply_refresh( &mut accounts, store.path(), None, false );

    // No credential file → refresh_account_token returns None → continue → result unchanged.
    assert!(
      matches!( accounts[ 0 ].result, Err( ref e ) if e.contains( "429" ) ),
      "429+expired: result must be unchanged when no cred file (refresh path entered but gracefully skipped); result: {:?}",
      accounts[ 0 ].result,
    );
  }

  // ── BUG-170 MRE: jwt_exp_ms + parse_u64_from_str ────────────────────────────

  /// MRE 1/2 for BUG-170: `jwt_exp_ms` returns `None` for opaque `sk-ant-oat01-*` tokens.
  ///
  /// # Root Cause
  /// `jwt_exp_ms` splits `accessToken` on `.` via `splitn(3, '.')`. Opaque `sk-ant-oat01-*`
  /// tokens have no `.` separator — the second `parts.next()?` returns `None` and
  /// `jwt_exp_ms` returns `None`. The `if let Some` guard at `usage.rs:803-806` never fires,
  /// leaving `aq.expires_at_ms` at its stale pre-refresh expired timestamp.
  ///
  /// # Why Not Caught
  /// BUG-162 tests used synthetic JWT-format tokens. No test verified `jwt_exp_ms` behavior
  /// for opaque `sk-ant-oat01-*` tokens, nor that `expires_at_ms` is correct post-refresh
  /// when `jwt_exp_ms` returns `None`.
  ///
  /// # Fix Applied
  /// Fix(BUG-170): `parse_u64_from_str` fallback added after `jwt_exp_ms` in `apply_refresh`.
  /// This test guards the precondition: `jwt_exp_ms` correctly returns `None` for opaque tokens.
  ///
  /// # Prevention
  /// `jwt_exp_ms` returns `None` for any non-JWT token; this is by design. Never "fix"
  /// `jwt_exp_ms` to handle opaque tokens — the correct fix is a separate `expiresAt` fallback.
  ///
  /// # Pitfall
  /// If `jwt_exp_ms` is modified to handle opaque tokens directly (wrong fix), this test
  /// fails, alerting that the `parse_u64_from_str` fallback may be redundant. Preserve the
  /// two-step fallback design regardless — opaque tokens will never have a parseable JWT payload.
  // test_kind: bug_reproducer(BUG-170)
  #[ test ]
  fn test_jwt_exp_ms_mre_bug170_opaque_returns_none()
  {
    // Opaque sk-ant-oat01-* token: no '.' separator — splitn(3, '.') yields one part.
    let opaque_creds = r#"{"accessToken":"sk-ant-oat01-XXXXXXXXXXXX","expiresAt":9999999999999}"#;
    assert!(
      jwt_exp_ms( opaque_creds ).is_none(),
      "jwt_exp_ms must return None for opaque sk-ant-oat01 token (no JWT structure); \
       if this fails, jwt_exp_ms was changed to handle opaque tokens — review BUG-170 fix",
    );
  }

  /// MRE 2/2 for BUG-170: `parse_u64_from_str` extracts `expiresAt` from credentials JSON.
  ///
  /// # Root Cause
  /// `parse_u64_field` takes `&Path` and cannot be used with the in-memory `new_creds: String`
  /// directly. BUG-170 is that there is no string-based fallback for extracting `expiresAt`
  /// from `new_creds` when `jwt_exp_ms` returns `None`, leaving `aq.expires_at_ms` stale.
  ///
  /// # Why Not Caught
  /// TSK-163 replaced `parse_u64_field` (stale file) with `jwt_exp_ms` (new token) but added
  /// no fallback for the case where `jwt_exp_ms` returns `None`. No test verified that the
  /// `expiresAt` field in `new_creds` is readable and used when JWT decoding fails.
  ///
  /// # Fix Applied
  /// Fix(BUG-170): extracted `parse_u64_from_str(s: &str, key: &str) -> Option<u64>` from
  /// `parse_u64_field`; added as `else if` fallback in `apply_refresh` at lines 803-810.
  ///
  /// # Prevention
  /// When adding an expiry-extraction strategy, always provide a string-based fallback for
  /// credentials JSON already in memory; never assume all access tokens are JWTs.
  ///
  /// # Pitfall
  /// `parse_u64_from_str` scans for `"key":digits` — works for both flat JSON
  /// (`{"expiresAt":N}`) and nested JSON (`{"claudeAiOauth":{"expiresAt":N}}`); the plain
  /// string scan finds the first occurrence of the key regardless of nesting depth.
  // test_kind: bug_reproducer(BUG-170)
  #[ test ]
  fn test_parse_u64_from_str_mre_bug170_extracts_expires_at()
  {
    // Flat credentials JSON (common in test fixtures).
    let flat = r#"{"accessToken":"sk-ant-oat01-XXXX","expiresAt":9999999999999}"#;
    assert_eq!(
      parse_u64_from_str( flat, "expiresAt" ),
      Some( 9_999_999_999_999_u64 ),
      "parse_u64_from_str must extract expiresAt from flat credentials JSON",
    );

    // Nested credentials JSON (claudeAiOauth wrapper present in production credentials).
    let nested =
      r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat01-XXXX","expiresAt":1779487948931}}"#;
    assert_eq!(
      parse_u64_from_str( nested, "expiresAt" ),
      Some( 1_779_487_948_931_u64 ),
      "parse_u64_from_str must extract expiresAt from nested claudeAiOauth credentials JSON",
    );

    // Missing key — must return None, not panic.
    let no_key = r#"{"accessToken":"sk-ant-oat01-XXXX"}"#;
    assert!(
      parse_u64_from_str( no_key, "expiresAt" ).is_none(),
      "parse_u64_from_str must return None when expiresAt key is absent",
    );
  }

  // ── should_refresh ──────────────────────────────────────────────────────────

  /// SR-1 — 401 triggers refresh regardless of `expires_at_ms` (far-future token).
  #[ test ]
  fn test_should_refresh_401_triggers()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
    };
    assert!( should_refresh( &aq, 0 ), "401 must trigger refresh" );
  }

  /// SR-2 — 403 triggers refresh regardless of `expires_at_ms`.
  #[ test ]
  fn test_should_refresh_403_triggers()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "HTTP transport error: HTTP 403".to_string() ),
        account       : None,
    };
    assert!( should_refresh( &aq, 0 ), "403 must trigger refresh" );
  }

  /// SR-3 — 429 + locally expired (`expires_at_ms=0`, `now_secs=9999`) triggers refresh.
  ///
  /// Verifies BUG-156 fix: a rate-limited account with a stale (past) `expiresAt`
  /// must enter the refresh path so the credentials file gets updated.
  // test_kind: bug_reproducer(issue-156)
  #[ test ]
  fn test_should_refresh_mre_bug156_429_expired_triggers()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : 0, // locally expired
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
    };
    assert!(
      should_refresh( &aq, 9_999 ),
      "429+expired must trigger refresh (BUG-156), expires=0 now=9999",
    );
  }

  /// SR-4 — 429 with non-expired token must NOT trigger refresh.
  ///
  /// When the local `expiresAt` is in the future, 429 means the token is valid but
  /// the account is rate-limited. Refreshing would add a 30-second wait with no benefit.
  #[ test ]
  fn test_should_refresh_429_valid_token_no_trigger()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : FAR_FUTURE_MS, // not expired
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
    };
    assert!(
      !should_refresh( &aq, 0 ),
      "429 with valid (non-expired) token must NOT trigger refresh",
    );
  }

  /// SR-5 — 429 with `expires_at_ms` exactly equal to `now_secs * 1000` → triggers refresh.
  ///
  /// The guard uses `(expires_at_ms / 1000) <= now_secs`.  When `expires_at_ms = 5000`
  /// and `now_secs = 5`, `5000/1000 = 5 <= 5` is `true` — the token is treated as expired.
  #[ test ]
  fn test_should_refresh_429_exact_boundary_expired_triggers()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : 5_000,
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
    };
    assert!(
      should_refresh( &aq, 5 ),
      "429 with expires_at_ms=5000, now_secs=5 → 5000/1000=5<=5 → must trigger refresh",
    );
  }

  /// SR-6 — 429 with `expires_at_ms` one second in the future → no refresh triggered.
  ///
  /// When `expires_at_ms = 6000` and `now_secs = 5`, `6000/1000 = 6 <= 5` is `false` —
  /// the token is still valid; no refresh triggered.
  #[ test ]
  fn test_should_refresh_429_one_sec_future_no_trigger()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : 6_000,  // one second ahead of now_secs=5
      result        : Err( "HTTP transport error: HTTP 429".to_string() ),
        account       : None,
    };
    assert!(
      !should_refresh( &aq, 5 ),
      "429 with expires_at_ms=6000, now_secs=5 → 6000/1000=6<=5 false → must not trigger refresh",
    );
  }

  /// SR-7 — Ok result never triggers refresh.
  #[ test ]
  fn test_should_refresh_ok_no_trigger()
  {
    let quota = claude_quota::OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : 0,
      result        : Ok( quota ),
        account       : None,
    };
    assert!( !should_refresh( &aq, 9_999 ), "Ok result must not trigger refresh" );
  }

  /// SR-8 — Generic (non-HTTP) error does not trigger refresh.
  #[ test ]
  fn test_should_refresh_generic_error_no_trigger()
  {
    let aq = AccountQuota
    {
      name          : "a@test.com".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : 0,
      result        : Err( "connection refused".to_string() ),
        account       : None,
    };
    assert!( !should_refresh( &aq, 9_999 ), "generic error must not trigger refresh" );
  }

  // ── compute_expires_cell ────────────────────────────────────────────────────

  /// C6 — Both zero: `expires_at_ms=0, now_secs=0` → "EXPIRED".
  #[ test ]
  fn test_compute_expires_cell_both_zero()
  {
    assert_eq!( compute_expires_cell( 0, 0 ), "EXPIRED" );
  }

  /// C7 — Sub-second truncation: `expires_at_ms=999` rounds down to 0 seconds → "EXPIRED".
  #[ test ]
  fn test_compute_expires_cell_subsecond_truncation()
  {
    assert_eq!( compute_expires_cell( 999, 0 ), "EXPIRED" );
  }

  /// C8 — Exactly 1 second remaining → "in ..." (not "EXPIRED").
  #[ test ]
  fn test_compute_expires_cell_one_second_remaining()
  {
    let result = compute_expires_cell( 1000, 0 );
    assert!( result.starts_with( "in " ), "1 second remaining must start with 'in ', got: {result}" );
  }

  /// C9 — Saturating subtraction: now exceeds expires → "EXPIRED", no underflow.
  #[ test ]
  fn test_compute_expires_cell_now_exceeds_expires()
  {
    assert_eq!( compute_expires_cell( 1000, 9999 ), "EXPIRED" );
  }

  const FAR_FUTURE_MS : u64 = 9_999_999_999_000;

  // ── secs_to_hms_utc ────────────────────────────────────────────────────────

  /// C15 — Zero seconds → "00:00:00".
  #[ test ]
  fn test_secs_to_hms_utc_zero()
  {
    assert_eq!( secs_to_hms_utc( 0 ), "00:00:00" );
  }

  /// C16 — End of day → "23:59:59".
  #[ test ]
  fn test_secs_to_hms_utc_end_of_day()
  {
    assert_eq!( secs_to_hms_utc( 86399 ), "23:59:59" );
  }

  /// C17 — Exactly one day wraps to "00:00:00".
  #[ test ]
  fn test_secs_to_hms_utc_day_wrap()
  {
    assert_eq!( secs_to_hms_utc( 86400 ), "00:00:00" );
  }

  /// C18 — Mid-day timestamp.
  #[ test ]
  fn test_secs_to_hms_utc_midday()
  {
    assert_eq!( secs_to_hms_utc( 45045 ), "12:30:45" );
  }

  // ── status_emoji ────────────────────────────────────────────────────────────

  fn mk_aq_ok( utilization : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name : "test@example.com".to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS, result : Ok( data ), account : None,
    }
  }

  fn mk_aq_err() -> AccountQuota
  {
    AccountQuota
    {
      name : "bad@example.com".to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS, result : Err( "missing accessToken".to_string() ),
      account : None,
    }
  }

  /// SE-1 — Err result → 🔴.
  #[ test ]
  fn test_status_emoji_red()
  {
    let aq = mk_aq_err();
    let output = render_text( &[ aq ], SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    assert!( output.contains( "🔴" ), "Err account must show 🔴. Got:\n{output}" );
  }

  /// SE-2 — Ok, `5h_left` = 90% (util=10.0) → 🟢.
  #[ test ]
  fn test_status_emoji_green()
  {
    let aq = mk_aq_ok( 10.0 );
    let output = render_text( &[ aq ], SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    assert!( output.contains( "🟢" ), "90% left must show 🟢. Got:\n{output}" );
  }

  /// SE-3 — Ok, `5h_left` = 3% (util=97.0) → 🟡.
  #[ test ]
  fn test_status_emoji_yellow()
  {
    let aq = mk_aq_ok( 97.0 );
    let output = render_text( &[ aq ], SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    assert!( output.contains( "🟡" ), "3% left must show 🟡. Got:\n{output}" );
  }

  /// SE-4 — Boundary: 5% exactly (util=95.0) → 🟡 (inclusive at 5%).
  /// SE-4b — Boundary: 5.1% (util=94.9) → 🟢.
  #[ test ]
  fn test_status_emoji_boundary()
  {
    let aq_5pct   = mk_aq_ok( 95.0 );
    let aq_5_1pct = mk_aq_ok( 94.9 );
    let out_5    = render_text( &[ aq_5pct ],   SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    let out_5_1  = render_text( &[ aq_5_1pct ], SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    assert!( out_5.contains( "🟡" ),   "exactly 5% left must show 🟡. Got:\n{out_5}" );
    assert!( out_5_1.contains( "🟢" ), "5.1% left must show 🟢. Got:\n{out_5_1}" );
  }

  /// SE-5 — Synthetic current-session row (`is_current=true`) shows correct emoji.
  #[ test ]
  fn test_status_emoji_on_synthetic_row()
  {
    let mut aq = mk_aq_ok( 20.0 );
    aq.is_current = true;
    aq.name = "(current session)".to_string();
    let output = render_text( &[ aq ], SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    assert!( output.contains( "🟢" ), "80% left synthetic row must show 🟢. Got:\n{output}" );
  }

  /// SE-6 — JSON output must NOT contain emoji (AC-20 no JSON equivalent).
  #[ test ]
  fn test_status_emoji_absent_in_json()
  {
    let aq = mk_aq_ok( 50.0 );
    let json = render_json( &[ aq ] );
    assert!( !json.contains( "🔴" ) && !json.contains( "🟡" ) && !json.contains( "🟢" ),
      "JSON must not contain status emoji. Got:\n{json}" );
  }

  // ── render_text ─────────────────────────────────────────────────────────────

  /// C19 — Empty accounts → "(no accounts configured)".
  #[ test ]
  fn test_render_text_empty()
  {
    let result = render_text( &[], SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &ColsVisibility::default_set() );
    assert!( result.contains( "no accounts configured" ), "empty must say no accounts, got: {result}" );
  }

  // ── render_json ─────────────────────────────────────────────────────────────

  /// C20 — Empty accounts → "[]".
  #[ test ]
  fn test_render_json_empty()
  {
    let result = render_json( &[] );
    assert_eq!( result.trim(), "[]" );
  }

  /// C21 — Err account → JSON contains "error" field.
  #[ test ]
  fn test_render_json_error_account()
  {
    let accounts = vec![
      AccountQuota
      {
        name : "fail@test.com".to_string(), is_current : false, is_active : false,
        expires_at_ms : 0, result : Err( "auth failed".to_string() ),
        account       : None,
      },
    ];
    let result = render_json( &accounts );
    assert!( result.contains( "\"error\":" ), "Err account must have error field, got: {result}" );
    assert!( result.contains( "auth failed" ), "error message must be preserved, got: {result}" );
  }

  /// C22 — Account name with quotes is JSON-escaped.
  #[ test ]
  fn test_render_json_escapes_quotes_in_name()
  {
    let accounts = vec![
      AccountQuota
      {
        name : "test\"@evil.com".to_string(), is_current : false, is_active : false,
        expires_at_ms : 0, result : Err( "fail".to_string() ),
        account       : None,
      },
    ];
    let result = render_json( &accounts );
    assert!(
      result.contains( r#"test\"@evil.com"# ),
      "quotes in name must be escaped, got: {result}",
    );
  }

  /// FT-08 — Mixed Ok and Err accounts both appear in `format::json` output.
  ///
  /// After `apply_refresh` runs on a mixed list (one Ok account, one Err account
  /// with no credential file to refresh), `render_json` must produce a valid JSON
  /// array where the Ok account carries quota fields and the Err account carries
  /// an `error` field — both rows present and correctly structured.
  #[ test ]
  fn test_render_json_ft8_mixed_ok_and_err_both_present()
  {
    let store = TempDir::new().unwrap();
    let quota = claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let mut accounts = vec![
      AccountQuota
      {
        name          : "ok@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : FAR_FUTURE_MS,
        result        : Ok( quota ),
        account       : None,
      },
      AccountQuota
      {
        name          : "err@example.com".to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
      },
    ];

    // No cred files in store → apply_refresh skips both; results unchanged.
    apply_refresh( &mut accounts, store.path(), None, false );

    let json = render_json( &accounts );

    // Both accounts must be present in the output.
    assert!( json.contains( "ok@example.com" ),  "Ok account must appear in JSON; got: {json}" );
    assert!( json.contains( "err@example.com" ), "Err account must appear in JSON; got: {json}" );

    // Err account must carry an "error" field; Ok account must carry quota fields.
    assert!( json.contains( "\"error\":" ),                 "Err account must have error field; got: {json}" );
    assert!( json.contains( "\"session_5h_left_pct\":" ),   "Ok account must have quota fields; got: {json}" );

    // Output must be a non-empty JSON array.
    let trimmed = json.trim();
    assert!( trimmed.starts_with( '[' ), "JSON must start with '['; got: {json}" );
    assert!( trimmed.ends_with(   ']' ), "JSON must end with ']'; got: {json}" );
  }

  // ── token_exp_label ────────────────────────────────────────────────────────

  /// EC-1 — epoch timestamp (ms=0) is always in the past → `expired(... ago)`.
  ///
  /// # Root Cause
  /// `token_exp_label` is a private helper used in the `[trace]` GET line.
  /// It branches on `now_ms >= expires_at_ms`. Epoch zero is always ≤ now,
  /// so the expired branch must fire for any realistic system clock.
  ///
  /// # Why Not Caught
  /// New function added in BUG-169 trace enhancement; no tests existed.
  ///
  /// # Fix Applied
  /// Added unit test with deterministic input (ms=0 is always past).
  ///
  /// # Prevention
  /// Cover both branches of `token_exp_label` with deterministic inputs that
  /// are guaranteed past (0) and guaranteed future (`u64::MAX`).
  ///
  /// # Pitfall
  /// `token_exp_label` calls `SystemTime::now()` internally — cannot be mocked.
  /// Use extreme boundary values (0 and `u64::MAX`) to guarantee branch coverage
  /// regardless of wall-clock time.
  #[ test ]
  fn tel_epoch_zero_is_expired()
  {
    let label = token_exp_label( 0 );
    assert!( label.starts_with( "expired(" ), "expected expired prefix; got: {label}" );
    assert!( label.ends_with( " ago)" ),      "expected ' ago)' suffix; got: {label}" );
  }

  /// EC-2 — far-future timestamp (`u64::MAX` ms) is always in the future → `valid(... left)`.
  ///
  /// # Root Cause
  /// See `tel_epoch_zero_is_expired` — covers the `valid` branch of `token_exp_label`.
  ///
  /// # Why Not Caught
  /// New function; no tests existed.
  ///
  /// # Fix Applied
  /// Added unit test with `u64::MAX` as the expiry — always future for any real clock.
  ///
  /// # Prevention
  /// Use `u64::MAX` to guarantee the `valid` branch fires without mocking `SystemTime`.
  ///
  /// # Pitfall
  /// `u64::MAX` milliseconds is ~584 million years from epoch — safe for all foreseeable use.
  #[ test ]
  fn tel_far_future_is_valid()
  {
    let label = token_exp_label( u64::MAX );
    assert!( label.starts_with( "valid(" ), "expected valid prefix; got: {label}" );
    assert!( label.ends_with( " left)" ),   "expected ' left)' suffix; got: {label}" );
  }

  // ── SortStrategy / PreferStrategy enum parsing ──────────────────────────────

  /// AC-09 — `SortStrategy::parse` rejects unknown values with descriptive error.
  #[ test ]
  fn test_sort_strategy_parse_invalid_rejected()
  {
    let err = SortStrategy::parse( "bogus" ).unwrap_err();
    assert!( err.contains( "bogus" ),     "error must name the bad value; got: {err}" );
    assert!( err.contains( "name" ),      "error must name valid values; got: {err}" );
    assert!( err.contains( "endurance" ), "error must name valid values; got: {err}" );
    assert!( err.contains( "drain" ),     "error must name valid values; got: {err}" );
    assert!( err.contains( "reset" ),     "error must name valid values; got: {err}" );
  }

  /// AC-10 — `PreferStrategy::parse` rejects unknown values with descriptive error.
  #[ test ]
  fn test_prefer_strategy_parse_invalid_rejected()
  {
    let err = PreferStrategy::parse( "bogus" ).unwrap_err();
    assert!( err.contains( "bogus" ),   "error must name the bad value; got: {err}" );
    assert!( err.contains( "any" ),     "error must name valid values; got: {err}" );
    assert!( err.contains( "opus" ),    "error must name valid values; got: {err}" );
    assert!( err.contains( "sonnet" ),  "error must name valid values; got: {err}" );
  }

  // ── sort_indices / sort strategies ────────────────────────────────────────────

  // Helper: build AccountQuota with controlled 5h_left and name.
  fn mk_aq_sort( name : &str, five_hour_util : f64, expires_at_ms : u64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name : name.to_string(), is_current : false, is_active : false,
      expires_at_ms, result : Ok( data ), account : None,
    }
  }

  fn mk_aq_sort_weekly( name : &str, five_hour_util : f64, seven_day_util : f64, seven_day_sonnet_util : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : seven_day_util, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : seven_day_sonnet_util, resets_at : None } ),
    };
    AccountQuota
    {
      name : name.to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS, result : Ok( data ), account : None,
    }
  }

  // Helper: build ISO-8601 reset string at `now_secs + offset_secs` for sort::endurance / sort::reset tests.
  fn reset_iso_at( now_secs : u64, offset_secs : u64 ) -> String
  {
    let ts = now_secs + offset_secs;
    // Format as minimal ISO-8601 accepted by iso_to_unix_secs: "YYYY-MM-DDTHH:MM:SSZ".
    let ( y, mo, d ) = unix_to_date( ts );
    let sod  = ts % 86400;
    let h    = sod / 3600;
    let mi   = ( sod % 3600 ) / 60;
    let s    = sod % 60;
    format!( "{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z" )
  }

  fn mk_aq_with_reset( name : &str, five_hour_util : f64, now_secs : u64, reset_offset_secs : u64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage
      {
        utilization : five_hour_util,
        resets_at   : Some( reset_iso_at( now_secs, reset_offset_secs ) ),
      } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name : name.to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS, result : Ok( data ), account : None,
    }
  }

  /// AC-01 — `sort::name` (default) produces alphabetical order; `render_text` names appear A→Z.
  #[ test ]
  fn test_sort_name_alphabetical()
  {
    let accounts = vec![
      mk_aq_sort( "zzz@test.com", 50.0, FAR_FUTURE_MS ),
      mk_aq_sort( "aaa@test.com", 50.0, FAR_FUTURE_MS ),
      mk_aq_sort( "mmm@test.com", 50.0, FAR_FUTURE_MS ),
    ];
    let indices = sort_indices( &accounts, SortStrategy::Name, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "aaa@test.com" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "mmm@test.com" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "zzz@test.com" );
  }

  /// AC-01 / AC-05 — `sort::name desc::1` produces Z→A.
  #[ test ]
  fn test_sort_name_desc_reverses()
  {
    let accounts = vec![
      mk_aq_sort( "aaa@test.com", 50.0, FAR_FUTURE_MS ),
      mk_aq_sort( "zzz@test.com", 50.0, FAR_FUTURE_MS ),
    ];
    let indices = sort_indices( &accounts, SortStrategy::Name, Some( true ), PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "zzz@test.com", "desc::1 must reverse name order" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "aaa@test.com" );
  }

  /// AC-03 — `sort::drain` places exhausted (≤5% `5h_left`) accounts last.
  /// Non-exhausted sorted by `5h_left` ascending (lowest first = drain targets first).
  #[ test ]
  fn test_sort_drain_exhausted_sunk_rest_ascending()
  {
    // util: 99% → 1% left (exhausted), 75% → 25% left, 30% → 70% left
    let accounts = vec![
      mk_aq_sort( "exhausted@test.com", 99.0, FAR_FUTURE_MS ),  // 1% left → exhausted
      mk_aq_sort( "low@test.com",       75.0, FAR_FUTURE_MS ),  // 25% left
      mk_aq_sort( "high@test.com",      30.0, FAR_FUTURE_MS ),  // 70% left
    ];
    let indices = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "low@test.com",      "lowest non-exhausted must be first" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "high@test.com",     "next lowest non-exhausted second" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "exhausted@test.com","exhausted must be last" );
  }

  /// AC-03 + AC-05 — `sort::drain desc::1` reverses non-exhausted; exhausted stays last.
  #[ test ]
  fn test_sort_drain_desc_reverses_non_exhausted_only()
  {
    let accounts = vec![
      mk_aq_sort( "exhausted@test.com", 99.0, FAR_FUTURE_MS ),  // ≤5% — sunk
      mk_aq_sort( "low@test.com",       75.0, FAR_FUTURE_MS ),  // 25% left
      mk_aq_sort( "high@test.com",      30.0, FAR_FUTURE_MS ),  // 70% left
    ];
    let indices = sort_indices( &accounts, SortStrategy::Drain, Some( true ), PreferStrategy::Any, 0 );
    assert_eq!( accounts[ indices[ 0 ] ].name, "high@test.com",     "desc::1 drain: highest non-exhausted first" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "low@test.com",      "desc::1 drain: second" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "exhausted@test.com","exhausted must still be last" );
  }

  /// AC-04 — `sort::reset` places exhausted accounts last; non-exhausted sorted by soonest reset.
  #[ test ]
  fn test_sort_reset_soonest_first_exhausted_last()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "late@test.com",      30.0, now, 7200  ),  // 70% left, 2h reset
      mk_aq_with_reset( "exhausted@test.com", 99.0, now, 600   ),  // ≤5% left — exhausted
      mk_aq_with_reset( "soon@test.com",      30.0, now, 600   ),  // 70% left, 10min reset
    ];
    let indices = sort_indices( &accounts, SortStrategy::Reset, None, PreferStrategy::Any, now );
    assert_eq!( accounts[ indices[ 0 ] ].name, "soon@test.com",      "soonest reset must be first" );
    assert_eq!( accounts[ indices[ 1 ] ].name, "late@test.com",      "later reset second" );
    assert_eq!( accounts[ indices[ 2 ] ].name, "exhausted@test.com", "exhausted must be last" );
  }

  /// AC-06 — `sort::endurance` without explicit `desc::` equals `desc::1` (qualified first).
  #[ test ]
  fn test_sort_endurance_default_equals_desc1()
  {
    let now : u64 = 1_000_000;
    // One qualified: reset in 30min, weekly=50%; one unqualified: reset in 5h, weekly=10%.
    let accounts = vec![
      mk_aq_with_reset( "unqualified@test.com", 50.0, now, 18000 ), // 5h reset — too far
      mk_aq_with_reset( "qualified@test.com",   50.0, now, 1800  ), // 30min reset ✓
    ];
    // Add weekly data to qualified account.
    let mut accounts = accounts;
    if let Ok( ref mut data ) = accounts[ 1 ].result
    {
      data.seven_day = Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } );
    }

    let idx_default = sort_indices( &accounts, SortStrategy::Endurance, None,         PreferStrategy::Any, now );
    let idx_desc1   = sort_indices( &accounts, SortStrategy::Endurance, Some( true ), PreferStrategy::Any, now );
    assert_eq!( idx_default, idx_desc1, "endurance default must equal desc::1" );
    assert_eq!( accounts[ idx_default[ 0 ] ].name, "qualified@test.com", "qualified must be first with default" );
  }

  /// AC-06 — `sort::drain` without explicit `desc::` equals `desc::0` (lowest first).
  #[ test ]
  fn test_sort_drain_default_equals_desc0()
  {
    let accounts = vec![
      mk_aq_sort( "high@test.com", 30.0, FAR_FUTURE_MS ),  // 70% left
      mk_aq_sort( "low@test.com",  75.0, FAR_FUTURE_MS ),  // 25% left
    ];
    let idx_default = sort_indices( &accounts, SortStrategy::Drain, None,          PreferStrategy::Any, 0 );
    let idx_desc0   = sort_indices( &accounts, SortStrategy::Drain, Some( false ), PreferStrategy::Any, 0 );
    assert_eq!( idx_default, idx_desc0, "drain default must equal desc::0" );
    assert_eq!( accounts[ idx_default[ 0 ] ].name, "low@test.com", "lowest first with default drain" );
  }

  /// AC-07 — `prefer::sonnet` uses `7d(Son)` for endurance qualification.
  /// `prefer::any` uses min(7d Left, 7d(Son)).
  ///
  /// Account with 7d(Son)=35% but 7d Left=10% is qualified with `prefer::sonnet`, not with `prefer::any`.
  #[ test ]
  fn test_prefer_sonnet_qualifies_by_sonnet_quota()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "target@test.com", 50.0, now, 1800 ), // 30min reset
    ];
    let mut accounts = accounts;
    // 7d(Son)=35% left (util=65%), 7d Left=10% left (util=90%).
    if let Ok( ref mut data ) = accounts[ 0 ].result
    {
      data.seven_day        = Some( claude_quota::PeriodUsage { utilization : 90.0, resets_at : None } );
      data.seven_day_sonnet = Some( claude_quota::PeriodUsage { utilization : 65.0, resets_at : None } );
    }

    // prefer::any → min(10%, 35%) = 10% < 30% → NOT qualified.
    let idx_any    = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Any,    now );
    // prefer::sonnet → 35% ≥ 30% → qualified.
    let idx_sonnet = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Sonnet, now );
    // prefer::opus → 10% < 30% → NOT qualified.
    let idx_opus   = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Opus,   now );

    // Qualification affects position within endurance groups (qualified vs unqualified).
    // We check via five_hour_left — qualified vs unqualified doesn't change order for single account,
    // but we can verify prefer_weekly returns the expected value.
    assert!(
      prefer_weekly( &accounts[ 0 ], PreferStrategy::Sonnet ) >= 30.0,
      "prefer::sonnet must return ≥30% for this account",
    );
    assert!(
      prefer_weekly( &accounts[ 0 ], PreferStrategy::Any ) < 30.0,
      "prefer::any must return <30% (constrained by 7d Left=10%)",
    );
    assert!(
      prefer_weekly( &accounts[ 0 ], PreferStrategy::Opus ) < 30.0,
      "prefer::opus must return <30% (7d Left=10%)",
    );
    // Indices should still cover all accounts.
    assert_eq!( idx_any.len(), 1 );
    assert_eq!( idx_sonnet.len(), 1 );
    assert_eq!( idx_opus.len(), 1 );
  }

  /// AC-08 — `prefer::` affects drain tiebreak when two accounts have identical `5h_left`.
  #[ test ]
  fn test_prefer_opus_tiebreak_in_drain()
  {
    // Two accounts, same 5h_left (50% = util 50.0).
    // Account A: 7d Left=20% (util 80.0), 7d(Son)=80% — prefer::opus uses 7d Left=20%.
    // Account B: 7d Left=80% (util 20.0), 7d(Son)=20% — prefer::opus uses 7d Left=80%.
    let accounts = vec![
      mk_aq_sort_weekly( "low7d@test.com",  50.0, 80.0, 20.0 ),  // 7d Left=20%
      mk_aq_sort_weekly( "high7d@test.com", 50.0, 20.0, 80.0 ),  // 7d Left=80%
    ];
    // With prefer::opus: tiebreak by 7d Left descending → high7d (80%) ranks first in non-exhausted
    // ascending group... wait: ascending 5h_left is the PRIMARY sort key, then weekly tiebreak.
    // Both have same 5h_left (50%), so weekly tiebreak applies.
    // Drain canonical: lowest 5h_left first; tiebreak: highest weekly (prefer) first.
    // Here both have same 5h_left, so tiebreak by prefer_weekly desc.
    // prefer::opus: low7d has 20%, high7d has 80%. High weekly = high7d → it's first (index 1 in input = index 0 in output).
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Opus, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "high7d@test.com",
      "prefer::opus drain tiebreak: higher 7d Left must be first; got: {:?}", accounts[ idx[ 0 ] ].name,
    );
  }

  /// AC-13 — `render_json` output is NOT sorted by `sort::` strategy (stays alphabetical).
  #[ test ]
  fn test_json_unaffected_by_sort()
  {
    let accounts = vec![
      mk_aq_sort( "zzz@test.com", 30.0, FAR_FUTURE_MS ),  // 70% left
      mk_aq_sort( "aaa@test.com", 80.0, FAR_FUTURE_MS ),  // 20% left
    ];
    let json = render_json( &accounts );
    // JSON array preserves input order (alphabetical from fetch_all_quota).
    let zzz_pos = json.find( "zzz@test.com" ).unwrap_or( 0 );
    let aaa_pos = json.find( "aaa@test.com" ).unwrap_or( usize::MAX );
    assert!(
      zzz_pos < aaa_pos,
      "render_json must preserve input order (not sort:: strategy order); zzz first in input must appear first in JSON",
    );
  }

  /// AC-11 — `sort::drain` display order does not affect `→ Next` recommendation footer.
  ///
  /// `a@x.com` (`5h_left`=80%) and `b@x.com` (`5h_left`=25%) are both non-active.
  /// `sort::drain` places `b@x.com` first in display order (lowest `5h_left` first).
  /// The recommendation must still point to `a@x.com` because `find_recommendation`
  /// always runs on the original alphabetical accounts slice, not on the display-sorted order.
  #[ test ]
  fn test_sort_recommendation_unaffected_by_sort_strategy()
  {
    let accounts = vec![
      // Input order: alphabetical (a before b). Both non-active, both non-current, valid tokens.
      mk_aq_sort( "a@x.com", 20.0, FAR_FUTURE_MS ),  // 80% left — best recommendation
      mk_aq_sort( "b@x.com", 75.0, FAR_FUTURE_MS ),  // 25% left — drain target, first in drain order
    ];

    // sort::drain would place b@x.com (25% left) first in display order.
    // next::endurance picks the account with the most quota remaining (a@x.com, 80% left).
    // Use next::endurance so that → appears in the table body on a@x.com.
    let output = render_text(
      &accounts, SortStrategy::Drain, None, PreferStrategy::Any,
      NextStrategy::Endurance, &ColsVisibility::default_set(),
    );

    // Footer should recommend a@x.com (highest 5h_left = 80%), not b@x.com.
    assert!(
      output.contains( "a@x.com" ),
      "output must contain a@x.com; got:\n{output}",
    );
    // The → marker must appear on a@x.com's line, not b@x.com's line.
    let arrow_line = output.lines()
      .find( |l| l.contains( '→' ) );
    if let Some( line ) = arrow_line
    {
      assert!(
        line.contains( "a@x.com" ),
        "→ recommendation must be a@x.com (highest 5h_left), not b@x.com (AC-11); line: {line}",
      );
    }
    // Footer must show the endurance recommendation pointing to a@x.com
    // (TSK-184: footer is now 2-strategy unconditional; format: "  endurance   name   metric").
    let endurance_line = output.lines().find( |l| l.contains( "endurance" ) );
    assert!(
      endurance_line.is_some_and( |l| l.contains( "a@x.com" ) ),
      "footer endurance line must recommend a@x.com regardless of sort::drain display order (AC-11); got:\n{output}",
    );
  }

  /// CC-012 — `sort::reset desc::1` reverses non-exhausted tier; exhausted floor unchanged.
  #[ test ]
  fn test_sort_reset_desc1_reverses_non_exhausted_only()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "soon@test.com",      30.0, now, 600  ),  // 70% left, 10min reset
      mk_aq_with_reset( "late@test.com",      30.0, now, 7200 ),  // 70% left, 2h reset
      mk_aq_with_reset( "exhausted@test.com", 99.0, now, 600  ),  // ≤5% left — sunk
    ];
    // desc::1 reverses non-exhausted: latest reset first, soonest second; exhausted still last.
    let idx = sort_indices( &accounts, SortStrategy::Reset, Some( true ), PreferStrategy::Any, now );
    assert_eq!( accounts[ idx[ 0 ] ].name, "late@test.com",      "desc::1 reset: latest reset first" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "soon@test.com",      "desc::1 reset: soonest second" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "exhausted@test.com", "exhausted must still be last" );
  }

  /// CC-026 — `sort::drain prefer::sonnet` tiebreaks by `7d(Son)` descending when `5h_left` tied.
  #[ test ]
  fn test_sort_drain_prefer_sonnet_tiebreak()
  {
    // Both accounts have 5h_left=50%.
    // "low_son":  7d(Son)=20% left (son_util=80). "high_son": 7d(Son)=80% left (son_util=20).
    // Drain tiebreak: higher prefer_weekly first → high_son must be first.
    let accounts = vec![
      mk_aq_sort_weekly( "low_son@test.com",  50.0, 0.0, 80.0 ),
      mk_aq_sort_weekly( "high_son@test.com", 50.0, 0.0, 20.0 ),
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Sonnet, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "high_son@test.com",
      "prefer::sonnet drain tiebreak: higher 7d(Son) left must be first",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "low_son@test.com",
      "prefer::sonnet drain tiebreak: lower 7d(Son) left must be second",
    );
  }

  /// CC-027 — `sort::drain prefer::any` tiebreaks by `min(7d Left, 7d(Son))` descending.
  #[ test ]
  fn test_sort_drain_prefer_any_tiebreak()
  {
    // Both accounts have 5h_left=50%.
    // "high_any": 7d_util=30→7d_left=70%, son_util=40→son_left=60% → any=min(70,60)=60%.
    // "low_any":  7d_util=70→7d_left=30%, son_util=60→son_left=40% → any=min(30,40)=30%.
    // Drain tiebreak: higher prefer_weekly first → high_any must be first.
    let accounts = vec![
      mk_aq_sort_weekly( "high_any@test.com", 50.0, 30.0, 40.0 ),
      mk_aq_sort_weekly( "low_any@test.com",  50.0, 70.0, 60.0 ),
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "high_any@test.com",
      "prefer::any drain tiebreak: higher min(7d,Son) left must be first",
    );
    assert_eq!(
      accounts[ idx[ 1 ] ].name, "low_any@test.com",
      "prefer::any drain tiebreak: lower min(7d,Son) left must be second",
    );
  }

  /// CC-044 — `sort::drain` with all accounts exhausted preserves input order.
  #[ test ]
  fn test_sort_drain_all_exhausted_preserves_input_order()
  {
    // All three accounts have ≤5% 5h_left — all exhausted.
    // No non-exhausted tier to sort; all land in the exhausted floor in input order.
    let accounts = vec![
      mk_aq_sort( "first@test.com",  99.0, FAR_FUTURE_MS ),  // 1% left — exhausted
      mk_aq_sort( "second@test.com", 97.0, FAR_FUTURE_MS ),  // 3% left — exhausted
      mk_aq_sort( "third@test.com",  95.0, FAR_FUTURE_MS ),  // 5% left — exhausted (boundary)
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ idx[ 0 ] ].name, "first@test.com",  "all-exhausted drain: input order preserved" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "second@test.com", "all-exhausted drain: input order preserved" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "third@test.com",  "all-exhausted drain: input order preserved" );
  }

  /// CC-045 — `sort::reset` with all accounts exhausted preserves input order.
  #[ test ]
  fn test_sort_reset_all_exhausted_preserves_input_order()
  {
    let now : u64 = 1_000_000;
    let accounts = vec![
      mk_aq_with_reset( "first@test.com",  99.0, now, 600  ),  // 1% left — exhausted
      mk_aq_with_reset( "second@test.com", 97.0, now, 7200 ),  // 3% left — exhausted
      mk_aq_with_reset( "third@test.com",  95.0, now, 3600 ),  // 5% left — exhausted (boundary)
    ];
    let idx = sort_indices( &accounts, SortStrategy::Reset, None, PreferStrategy::Any, now );
    assert_eq!( accounts[ idx[ 0 ] ].name, "first@test.com",  "all-exhausted reset: input order preserved" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "second@test.com", "all-exhausted reset: input order preserved" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "third@test.com",  "all-exhausted reset: input order preserved" );
  }

  /// CC-058 — Account with `five_hour: None` is treated as non-exhausted (conservative 100% left).
  ///
  /// Why not caught: `five_hour_left` uses `map_or(0.0, ...)` — None → 0% util → 100% left.
  /// This is intentional conservative behaviour but must be pinned so a silent change
  /// cannot accidentally sink no-data accounts into the exhausted floor.
  #[ test ]
  fn test_sort_drain_none_five_hour_treated_as_non_exhausted()
  {
    let mk_no_fh = |name : &str| -> AccountQuota
    {
      AccountQuota
      {
        name          : name.to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : FAR_FUTURE_MS,
        result        : Ok( OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None } ),
        account       : None,
      }
    };
    let accounts = vec![
      mk_aq_sort( "low@test.com",       75.0, FAR_FUTURE_MS ),  // 25% left
      mk_no_fh(   "no_fh@test.com"                          ),  // None → 100% assumed
      mk_aq_sort( "exhausted@test.com", 99.0, FAR_FUTURE_MS ),  // 1% left — sunk
    ];
    // Drain canonical: ascending 5h_left → low(25%), no_fh(100%); exhausted sunk last.
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Any, 0 );
    assert_eq!( accounts[ idx[ 0 ] ].name, "low@test.com",       "25% left drains first" );
    assert_eq!( accounts[ idx[ 1 ] ].name, "no_fh@test.com",     "None five_hour = 100% left: last among non-exhausted" );
    assert_eq!( accounts[ idx[ 2 ] ].name, "exhausted@test.com", "exhausted always sunk to bottom" );
  }

  // ── status_emoji AND logic (T01–T04) ──────────────────────────────────────

  fn mk_aq_ok_both( h5_util : f64, d7_util : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name : "test@example.com".to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS, result : Ok( data ), account : None,
    }
  }

  /// SE-AND-T01: `5h_left`=50%, `7d_left`=50% → 🟢 (both > 5%).
  #[ test ]
  fn test_status_emoji_and_both_ample_green()
  {
    let aq = mk_aq_ok_both( 50.0, 50.0 );
    assert_eq!( status_emoji( &aq.result ), "🟢", "both > 5% → 🟢" );
  }

  /// SE-AND-T02: `5h_left`=50%, `7d_left`=3% (`d7_util`=97) → 🟡 (7d ≤ 5%).
  #[ test ]
  fn test_status_emoji_and_7d_low_yellow()
  {
    let aq = mk_aq_ok_both( 50.0, 97.0 );
    assert_eq!( status_emoji( &aq.result ), "🟡", "7d ≤ 5% despite 5h ample → 🟡" );
  }

  /// SE-AND-T03: `5h_left`=3% (`h5_util`=97), `7d_left`=50% → 🟡 (5h ≤ 5%).
  #[ test ]
  fn test_status_emoji_and_5h_low_yellow()
  {
    let aq = mk_aq_ok_both( 97.0, 50.0 );
    assert_eq!( status_emoji( &aq.result ), "🟡", "5h ≤ 5% despite 7d ample → 🟡" );
  }

  /// SE-AND-T04: `5h_left`=5%, `7d_left`=5% → 🟡 (exclusive > 5% boundary).
  #[ test ]
  fn test_status_emoji_and_both_at_threshold_yellow()
  {
    let aq = mk_aq_ok_both( 95.0, 95.0 );
    assert_eq!( status_emoji( &aq.result ), "🟡", "both exactly 5% left → 🟡 (not > 5%)" );
  }

  // ── quota_text_cells emoji prefix (T05–T06) ────────────────────────────────

  /// QT-T05: `5h_left`=86% (util=14.0) → cells[0] = "🟢 86%".
  #[ test ]
  fn test_quota_text_cells_5h_emoji_green()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 14.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let cells = quota_text_cells( &data, 0 );
    assert_eq!( cells[ 0 ], "🟢 86%", "86% 5h left → 🟢 86%" );
  }

  /// QT-T06: `5h_left`=3% (util=97.0) → cells[0] = "🟡 3%".
  #[ test ]
  fn test_quota_text_cells_5h_emoji_yellow()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 97.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let cells = quota_text_cells( &data, 0 );
    assert_eq!( cells[ 0 ], "🟡 3%", "3% 5h left → 🟡 3%" );
  }

  // ── Three-tier grouping (T07–T08) ─────────────────────────────────────────

  fn mk_named_aq( name : &str, h5_util : f64, d7_util : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name : name.to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS, result : Ok( data ), account : None,
    }
  }

  fn mk_named_aq_err( name : &str ) -> AccountQuota
  {
    AccountQuota
    {
      name : name.to_string(), is_current : false, is_active : false,
      expires_at_ms : FAR_FUTURE_MS,
      result : Err( "missing accessToken".to_string() ),
      account : None,
    }
  }

  /// TT-T07/T08 — three-tier grouping: 🟢 → 🟡 → 🔴 overrides sort order.
  ///
  /// `sort::name` gives alpha order a→b→c, but tier order yields b(🟢)→a(🟡)→c(🔴).
  #[ test ]
  fn test_three_tier_grouping_green_before_yellow_before_red()
  {
    let a = mk_named_aq(     "a@x.com", 97.0, 0.0  ); // 5h=3% → 🟡
    let b = mk_named_aq(     "b@x.com", 10.0, 10.0 ); // 5h=90%, 7d=90% → 🟢
    let c = mk_named_aq_err( "c@x.com"             ); // Err → 🔴
    let accounts = vec![ a, b, c ];
    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      NextStrategy::Endurance, &ColsVisibility::default_set(),
    );
    let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear in output" );
    let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear in output" );
    let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear in output" );
    assert!( pos_b < pos_a, "🟢(b) must appear before 🟡(a). Got:\n{output}" );
    assert!( pos_a < pos_c, "🟡(a) must appear before 🔴(c). Got:\n{output}" );
  }

  /// CC-059/CC-060 — `prefer_weekly` with absent period data treats account as fully available (100% left).
  ///
  /// None `seven_day` → 100% `7d_left` for `prefer::opus`.
  /// None `seven_day_sonnet` → 100% `sonnet_left` for `prefer::sonnet`.
  /// Verified via drain tiebreak: no-data account outranks explicit-low-data account when `5h_left` tied.
  #[ test ]
  fn test_prefer_weekly_none_periods_treated_as_full()
  {
    // prefer::opus: "no_data" has seven_day=None → prefer_weekly=100%.
    // "has_data" has seven_day_util=60 → 7d_left=40%.
    // Same 5h_left (50%) → tiebreak by prefer_weekly desc → no_data (100%) comes first.
    let accounts = vec![
      mk_aq_sort_weekly( "has_data@test.com", 50.0, 60.0, 60.0 ),  // 7d_left=40%
      mk_aq_sort(        "no_data@test.com",  50.0, FAR_FUTURE_MS ), // seven_day=None → 100%
    ];
    let idx = sort_indices( &accounts, SortStrategy::Drain, None, PreferStrategy::Opus, 0 );
    assert_eq!(
      accounts[ idx[ 0 ] ].name, "no_data@test.com",
      "None seven_day treated as 100% left; must rank first in drain prefer::opus tiebreak",
    );
  }

  // ── Per-column emoji formatting unit tests (FT-11/009) ───────────────────

  /// FT-11 of feature/009 — per-column emoji prefix in `5h Left` cell values.
  ///
  /// `quota_text_cells` must attach `🟢` prefix when `5h_left` > 5%, `🟡` when ≤ 5%.
  /// The boundary (exactly 5.0%) is inclusive for `🟡`.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-11]
  #[ test ]
  fn test_ft11_009_per_column_emoji_prefix_three_cases()
  {
    let mk_5h = |util : f64| -> claude_quota::OauthUsageData
    {
      claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage { utilization : util, resets_at : None } ),
        seven_day        : None,
        seven_day_sonnet : None,
      }
    };

    // Pct A: util=10.0 → 90% left (> 5%) → 🟢
    let cells_a = quota_text_cells( &mk_5h( 10.0 ), 0 );
    assert_eq!( cells_a[ 0 ], "🟢 90%", "Pct A (90% left) must have 🟢 prefix (FT-11/009)" );

    // Pct B: util=97.0 → 3% left (< 5%) → 🟡
    let cells_b = quota_text_cells( &mk_5h( 97.0 ), 0 );
    assert_eq!( cells_b[ 0 ], "🟡 3%", "Pct B (3% left) must have 🟡 prefix (FT-11/009)" );

    // Pct C: util=95.0 → exactly 5% left (≤ 5%) → 🟡 (boundary inclusive)
    let cells_c = quota_text_cells( &mk_5h( 95.0 ), 0 );
    assert_eq!( cells_c[ 0 ], "🟡 5%", "Pct C (exactly 5% left) must have 🟡 prefix — boundary inclusive (FT-11/009)" );
  }

  // ── Yellow-tier session-before-weekly sub-grouping (FT-16/009, AC-26) ────

  /// FT-16 of feature/009 — within 🟡 tier, session-exhausted appears before weekly-exhausted.
  ///
  /// Three-tier grouping splits 🟡 into two sub-groups (AC-26):
  ///
  /// - `session_yellow`: `5h Left ≤ 5%`  — appears first within 🟡
  /// - `weekly_yellow`:  `5h Left > 5%` AND `7d Left ≤ 5%` — appears after `session_yellow`
  ///
  /// Accounts with BOTH ≤ 5% fall in `session_yellow` (by `5h Left` check).
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-16]
  ///       [`docs/feature/009_token_usage.md` AC-26]
  #[ test ]
  fn test_ft16_009_yellow_tier_session_before_weekly()
  {
    // a@x.com: 5h=90% left, 7d=2% left → 🟡 WEEKLY-exhausted
    // b@x.com: 5h=1% left, 7d=70% left → 🟡 SESSION-exhausted
    // c@x.com: 5h=3% left, 7d=50% left → 🟡 SESSION-exhausted
    // d@x.com: 5h=90% left, 7d=90% left → 🟢
    //
    // With SortStrategy::Name: alpha order a,b,c,d.
    // Three-tier + AC-26: d (🟢), b (session 🟡), c (session 🟡), a (weekly 🟡).
    let a = mk_named_aq( "a@x.com", 10.0, 98.0 );  // 5h=90%, 7d=2% → weekly-exhausted
    let b = mk_named_aq( "b@x.com", 99.0, 30.0 );  // 5h=1%, 7d=70% → session-exhausted
    let c = mk_named_aq( "c@x.com", 97.0, 50.0 );  // 5h=3%, 7d=50% → session-exhausted
    let d = mk_named_aq( "d@x.com", 10.0, 10.0 );  // 5h=90%, 7d=90% → 🟢
    let accounts = vec![ a, b, c, d ];

    let output = render_text(
      &accounts,
      SortStrategy::Name,
      None,
      PreferStrategy::Any,
      NextStrategy::Endurance,
      &ColsVisibility::default_set(),
    );

    let pos_d = output.find( "d@x.com" ).expect( "d@x.com must appear" );
    let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear" );
    let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear" );
    let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear" );

    assert!( pos_d < pos_b, "🟢(d) must appear before session-yellow(b) (FT-16/009 AC-26);\n{output}" );
    assert!( pos_b < pos_a, "session-exhausted(b) must appear before weekly-exhausted(a) (FT-16/009 AC-26);\n{output}" );
    assert!( pos_c < pos_a, "session-exhausted(c) must appear before weekly-exhausted(a) (FT-16/009 AC-26);\n{output}" );
    // Within session_yellow: b comes before c (alpha order preserved within sub-group).
    assert!( pos_b < pos_c, "within session-yellow sub-group, alpha order must be preserved: b before c (FT-16/009 AC-26);\n{output}" );
  }

  /// FT-15 of feature/020 — `desc::1` reverses within each 🟡 sub-group but does NOT swap sub-group order.
  ///
  /// `z@x.com` is weekly-exhausted and alphabetically last; `desc::1` + `sort::name` would place it first
  /// among yellows without the sub-partition. With sub-partition, it stays after session-yellows.
  ///
  /// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-15]
  ///       [`docs/feature/020_usage_sort_strategies.md` AC-14]
  #[ test ]
  fn test_ft15_020_yellow_sub_grouping_not_reversed_by_desc()
  {
    // a@x.com: 5h=1% left, 7d=70% left → 🟡 SESSION-exhausted (alphabetically first)
    // b@x.com: 5h=3% left, 7d=50% left → 🟡 SESSION-exhausted
    // c@x.com: 5h=90% left, 7d=90% left → 🟢
    // z@x.com: 5h=90% left, 7d=2% left → 🟡 WEEKLY-exhausted (alphabetically last)
    //
    // With SortStrategy::Name + desc::1: sorted order = z, c, b, a (reverse alpha).
    // Without sub-partition: z(weekly) appears first among yellows → WRONG.
    // With sub-partition: session[b,a] before weekly[z] → b,a,z order among yellows.
    let a = mk_named_aq( "a@x.com", 99.0, 30.0 );  // 5h=1%, 7d=70% → session-exhausted
    let b = mk_named_aq( "b@x.com", 97.0, 50.0 );  // 5h=3%, 7d=50% → session-exhausted
    let c = mk_named_aq( "c@x.com", 10.0, 10.0 );  // 5h=90%, 7d=90% → 🟢
    let z = mk_named_aq( "z@x.com", 10.0, 98.0 );  // 5h=90%, 7d=2% → weekly-exhausted

    let accounts = vec![ a, b, c, z ];

    let output = render_text(
      &accounts,
      SortStrategy::Name,
      Some( true ), // desc::1
      PreferStrategy::Any,
      NextStrategy::Endurance,
      &ColsVisibility::default_set(),
    );

    let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear" );
    let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear" );
    let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear" );
    let pos_z = output.find( "z@x.com" ).expect( "z@x.com must appear" );

    // Sub-grouping is not reversed by desc:: — session-yellow still before weekly-yellow.
    assert!( pos_b < pos_z, "session-exhausted(b) must appear before weekly-exhausted(z) even with desc::1 (FT-15/020 AC-14);\n{output}" );
    assert!( pos_a < pos_z, "session-exhausted(a) must appear before weekly-exhausted(z) even with desc::1 (FT-15/020 AC-14);\n{output}" );
    // Green tier still leads.
    assert!( pos_c < pos_b, "🟢(c) must appear before session-yellow(b) (FT-15/020 AC-14);\n{output}" );
    // Within session-yellow sub-group, desc::1 reverses alpha → b before a (b > a alphabetically).
    assert!( pos_b < pos_a, "within session-yellow, desc::1 puts b before a (FT-15/020 AC-14);\n{output}" );
  }

  // ── find_next_for_strategy unit tests (FT-02/023, FT-06/009) ──────────────

  /// FT-02 of feature/023 — `find_next_for_strategy` places `→` on winner; None when no eligible.
  ///
  /// When-A: with B and C eligible (`is_current=false`, `result=Ok`), returns `Some(winner_idx)`.
  /// When-B: all accounts are `is_current=true` → returns `None` (no eligible candidate → no `→`).
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-02]
  #[ test ]
  fn test_ft02_023_find_next_for_strategy_some_when_eligible_none_when_all_current()
  {
    let now = 0u64;
    // When-A: A is current (ineligible), B and C are eligible.
    // Endurance strategy picks highest 5h_left first → B (70% left) over C (40% left).
    let mut a = mk_aq_sort( "a@test.com", 20.0, FAR_FUTURE_MS );
    a.is_current = true;
    let b = mk_aq_sort( "b@test.com", 30.0, FAR_FUTURE_MS );  // 70% left
    let c = mk_aq_sort( "c@test.com", 60.0, FAR_FUTURE_MS );  // 40% left
    let accounts = vec![ a, b, c ];

    let winner_a = find_next_for_strategy( &accounts, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!(
      winner_a.is_some(),
      "find_next_for_strategy must return Some when eligible candidates exist (FT-02/023 When-A)",
    );
    let winner_idx = winner_a.unwrap();
    assert_eq!(
      accounts[ winner_idx ].name, "b@test.com",
      "endurance winner must be b@test.com (highest `5h_left`); got index {winner_idx}",
    );

    // When-B: all accounts are is_current=true → no eligible candidate → None.
    let mut a2 = mk_aq_sort( "a@test.com", 20.0, FAR_FUTURE_MS );
    let mut b2 = mk_aq_sort( "b@test.com", 30.0, FAR_FUTURE_MS );
    let mut c2 = mk_aq_sort( "c@test.com", 60.0, FAR_FUTURE_MS );
    a2.is_current = true;
    b2.is_current = true;
    c2.is_current = true;
    let all_current = vec![ a2, b2, c2 ];

    let winner_b = find_next_for_strategy( &all_current, NextStrategy::Endurance, PreferStrategy::Any, now );
    assert!(
      winner_b.is_none(),
      "find_next_for_strategy must return None when all accounts are is_current=true (FT-02/023 When-B)",
    );
  }

  /// FT-06 of feature/009 — endurance tiebreaker: higher expiry wins when `5h Left` is tied.
  ///
  /// When two accounts have identical `five_hour.utilization`, the tiebreaker is
  /// `expires_at_ms` descending — the account whose token expires later wins.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-06]
  #[ test ]
  fn test_ft06_009_endurance_tiebreaker_higher_expiry_wins()
  {
    let now_ms   = 1_700_000_000_000u64;  // arbitrary fixed reference
    let now_secs = now_ms / 1000;

    // Both have identical 5h_left (50%).  "a" expires later (now+7200s), "b" sooner (now+3600s).
    let a = mk_aq_sort( "a@x.com", 50.0, now_ms + 7_200_000 );  // 2h expiry
    let b = mk_aq_sort( "b@x.com", 50.0, now_ms + 3_600_000 );  // 1h expiry
    let accounts = vec![ a, b ];

    let idx = find_next_for_strategy( &accounts, NextStrategy::Endurance, PreferStrategy::Any, now_secs );
    assert_eq!(
      idx, Some( 0 ),
      "endurance tiebreaker must pick a@x.com (higher expiry) when 5h_left tied (FT-06/009)",
    );
    assert_eq!(
      accounts[ idx.unwrap() ].name, "a@x.com",
      "winner must be a@x.com",
    );
  }

  // ── footer rendering unit tests (FT-08/023) ───────────────────────────────

  /// FT-08 of feature/023 — footer omits both strategy lines when no eligible candidate exists.
  ///
  /// When all accounts are `is_current=true` (ineligible for recommendation), neither the
  /// "endurance" nor the "drain" strategy line appears in `render_text` footer output.
  /// `find_next_for_strategy` returns None for both → `lines` is empty → footer body only.
  ///
  /// Spec: [`tests/docs/feature/023_next_account_strategies.md` FT-08]
  #[ test ]
  fn test_ft08_023_footer_omits_strategy_lines_when_no_eligible_candidate()
  {
    // Two valid accounts (result=Ok → valid_count=2, footer threshold passed),
    // but both is_current=true → no eligible candidate for either strategy.
    let mut a = mk_aq_sort( "a@test.com", 30.0, FAR_FUTURE_MS );
    let mut b = mk_aq_sort( "b@test.com", 60.0, FAR_FUTURE_MS );
    a.is_current = true;
    b.is_current = true;
    let accounts = vec![ a, b ];

    let output = render_text(
      &accounts,
      SortStrategy::Name,
      None,
      PreferStrategy::Any,
      NextStrategy::Endurance,
      &ColsVisibility::default_set(),
    );

    assert!(
      !output.contains( "endurance" ),
      "footer must omit endurance line when no eligible candidate (FT-08/023), got:\n{output}",
    );
    assert!(
      !output.contains( "drain" ),
      "footer must omit drain line when no eligible candidate (FT-08/023), got:\n{output}",
    );
    assert!(
      !output.contains( "Next by strategy:" ),
      "footer must not show 'Next by strategy:' when lines is empty (FT-08/023), got:\n{output}",
    );
  }

  // ── find_next_for_strategy::Drain unit tests ──────────────────────────────

  /// FT-04/023 unit A — drain picks lowest non-exhausted (> 5% left) account first.
  ///
  /// Two non-exhausted accounts: `a` has 6% left, `b` has 80% left.
  /// Drain canonical: ascending `5h_left` → `a` before `b`.
  /// `find_next_for_strategy` with Drain must return index of `a`.
  #[ test ]
  fn test_find_next_drain_picks_lowest_nonexhausted()
  {
    let now    = 0u64;
    let a = mk_aq_sort( "a@test.com", 94.0, FAR_FUTURE_MS );  // 6% left — non-exhausted
    let b = mk_aq_sort( "b@test.com", 20.0, FAR_FUTURE_MS );  // 80% left — non-exhausted
    let accounts = vec![ b, a ];  // intentionally reversed to confirm sort, not input order

    let idx = find_next_for_strategy( &accounts, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!(
      idx.is_some(),
      "drain must find a winner among two non-exhausted accounts",
    );
    assert_eq!(
      accounts[ idx.unwrap() ].name, "a@test.com",
      "drain must pick a@test.com (6% left, lowest non-exhausted); got index {idx:?}",
    );
  }

  /// FT-04/023 unit B — drain puts exhausted accounts (≤ 5% left) after non-exhausted.
  ///
  /// `exhausted` has 3% left (≤ 5%) and `healthy` has 80% left (> 5%).
  /// Even though `exhausted` has lower `5h_left`, drain picks `healthy` first.
  #[ test ]
  fn test_find_next_drain_prefers_nonexhausted_over_exhausted()
  {
    let now       = 0u64;
    let exhausted = mk_aq_sort( "exhausted@test.com", 97.0, FAR_FUTURE_MS );  // 3% left — exhausted
    let healthy   = mk_aq_sort( "healthy@test.com",   20.0, FAR_FUTURE_MS );  // 80% left — non-exhausted
    let accounts  = vec![ exhausted, healthy ];  // exhausted first in input order

    let idx = find_next_for_strategy( &accounts, NextStrategy::Drain, PreferStrategy::Any, now );
    assert!(
      idx.is_some(),
      "drain must find a winner when at least one non-exhausted account exists",
    );
    assert_eq!(
      accounts[ idx.unwrap() ].name, "healthy@test.com",
      "drain must pick healthy (80% left, non-exhausted) before exhausted (3% left); got index {idx:?}",
    );
  }

  // ── status_emoji with absent period data ─────────────────────────────────

  /// SE-7 — `five_hour=None` treated as 100% left → 🟢 (conservative, 0% utilised).
  ///
  /// Doc comment: "Absent period data is treated as fully available (conservative, 0% utilised)."
  /// `five_hour`=None → `map_or`(0.0) → `h5_left`=100% > 5% → 🟢 (given `seven_day` also absent → 100%).
  #[ test ]
  fn test_status_emoji_five_hour_none_is_green()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour : None, seven_day : None, seven_day_sonnet : None,
    };
    let result : Result< claude_quota::OauthUsageData, String > = Ok( data );
    assert_eq!(
      status_emoji( &result ), "🟢",
      "five_hour=None must yield 🟢 (conservative 100% left)",
    );
  }

  // ── quota_text_cells with absent period data ──────────────────────────────

  /// QT-T07 — `five_hour=None` in `quota_text_cells` → cells[0] = "—" (em dash).
  ///
  /// `pct_emoji(None)` → `util.map_or_else(|| dash.clone(), ...)` → "—".
  /// The absence of period data is displayed as em dash, not as a percentage.
  /// This is semantically distinct from `status_emoji` which treats None as 100% left.
  #[ test ]
  fn test_quota_text_cells_five_hour_none_shows_dash()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour : None, seven_day : None, seven_day_sonnet : None,
    };
    let cells = quota_text_cells( &data, 0 );
    assert_eq!(
      cells[ 0 ], "\u{2014}",
      "five_hour=None must produce em-dash in cells[0], not a percentage",
    );
  }

  // ── sort_indices: endurance tiebreaker ────────────────────────────────────

  /// bug_reproducer(BUG-173): endurance unqualified sort must tiebreak by
  /// highest weekly when `5h Left` values are equal.
  ///
  /// # Root Cause
  ///
  /// `unqualified.sort_by` compared only `five_hour_left` — when multiple
  /// accounts had identical 5h utilization, insertion order silently selected
  /// the wrong account, ignoring weekly quota.
  ///
  /// # Why Not Caught
  ///
  /// Existing sort tests used accounts with distinct `five_hour_left` values,
  /// so the tiebreaker path was never exercised.
  ///
  /// # Fix Applied
  ///
  /// Added `.then_with(prefer_weekly)` to the `unqualified.sort_by` closure,
  /// mirroring the drain strategy tiebreaker.
  ///
  /// # Prevention
  ///
  /// This test constructs 3 accounts with identical `five_hour.utilization`
  /// but varying `seven_day.utilization`, asserting deterministic sort order.
  ///
  /// # Pitfall
  ///
  /// The tiebreaker uses `prefer_weekly(prefer)` — the `prefer` parameter
  /// must be forwarded, not hardcoded. Changing the prefer strategy changes
  /// which weekly field is used for the tiebreaker.
  #[ test ]
  fn test_bug173_mre_endurance_unqualified_prefers_highest_weekly()
  {
    let make_account = |name : &str, five_h_util : f64, seven_d_util : f64| -> AccountQuota
    {
      AccountQuota
      {
        name          : name.to_string(),
        is_current    : false,
        is_active     : false,
        expires_at_ms : u64::MAX,
        result        : Ok( claude_quota::OauthUsageData
        {
          five_hour : Some( claude_quota::PeriodUsage
          {
            utilization : five_h_util,
            resets_at   : None,
          } ),
          seven_day : Some( claude_quota::PeriodUsage
          {
            utilization : seven_d_util,
            resets_at   : None,
          } ),
          seven_day_sonnet : None,
        } ),
        account : None,
      }
    };

    // All three have five_hour.utilization = 50.0 (i.e. 50% left).
    // Weekly utilization differs: 98%, 0%, 27% → weekly LEFT = 2%, 100%, 73%.
    let accounts = vec![
      make_account( "acct_a", 50.0, 98.0 ),  // 2% weekly left
      make_account( "acct_b", 50.0,  0.0 ),  // 100% weekly left
      make_account( "acct_c", 50.0, 27.0 ),  // 73% weekly left
    ];

    // No resets_at → all unqualified.  prefer=Any → left_7d.min(left_son);
    // seven_day_sonnet=None → left_son=100.0 → prefer_weekly = left_7d.
    let sorted = sort_indices( &accounts, SortStrategy::Endurance, None, PreferStrategy::Any, 0 );

    // Expected canonical: highest weekly left first → B(100%), C(73%), A(2%).
    assert_eq!(
      sorted, vec![ 1, 2, 0 ],
      "BUG-173: endurance unqualified sort must tiebreak by weekly; \
       expected [B=1,C=2,A=0], got {sorted:?}",
    );
  }
}
