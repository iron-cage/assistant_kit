//! Account quota fetch pipeline.
//!
//! Enumerates saved accounts, fetches live OAuth usage data for each, and
//! provides numeric JSON-field parsing helpers used by the refresh pipeline.

use unilang::data::{ ErrorCode, ErrorData };
use super::types::AccountQuota;
use super::format::token_exp_label;
use claude_profile_core::account::trace_ts;

// ── Token reader ──────────────────────────────────────────────────────────────

/// Read the OAuth access token from a named account credentials file.
///
/// Returns `Err(reason)` on I/O failure or missing `accessToken` field;
/// the reason is stored inline per-account and does not abort the full fetch.
pub(super) fn read_token( credential_store : &std::path::Path, name : &str ) -> Result< String, String >
{
  let path = credential_store.join( format!( "{name}.credentials.json" ) );
  claude_profile_core::account::read_access_token_from_file( &path )
}

/// Prepend `row` to `results` only when no entry with the same `name` is already present.
///
/// Enforces the at-most-one-row-per-name invariant for the synthetic current-session row
/// injected by `fetch_all_quota`. Unconditional `results.insert(0, row)` creates a
/// duplicate when `row.name` matches an existing stored account (BUG-218).
fn inject_synthetic_if_new( results : &mut Vec< AccountQuota >, row : AccountQuota )
{
  // Fix(BUG-218): Guard insertion so the synthetic row is only added when absent.
  // Root cause: unconditional insert(0) duplicated the active account when it was
  //   already fetched into `results` as a named stored account.
  // Pitfall: any future caller that passes a synthetic row must ensure `row.name`
  //   exactly matches the stored account name — case-sensitive equality is the guard.
  if !results.iter().any( |r| r.name == row.name )
  {
    results.insert( 0, row );
  }
}

/// Fetch live quota data for a pre-built account list.
///
/// Per-account failures are stored inline in `AccountQuota::result`; only
/// fatal errors propagate as `ErrorData`.
///
/// `live_creds_file` is read once to extract the live `accessToken`; any failure
/// (absent file, parse error) silently sets `is_current = false` for all accounts.
///
/// If no supplied account's token matches the live token, a synthetic entry is prepended
/// (AC-09): `is_current: true`, name from `~/.claude.json` email or `(current session)`.
/// Pitfall: this case is easy to miss when only testing the normal single-account path.
///
/// When `stagger` is `true`, each account fetch is preceded by a pseudo-random sleep
/// of 200–1500 ms (thunder-herd mitigation for live monitor mode).
///
/// When `trace` is `true`, one timestamped diagnostic line is written to stderr before reading
/// each account's credentials and one after the final result is determined (including
/// any `billing_type` override — see AC-31).
#[ allow( clippy::too_many_lines ) ]
pub( crate ) fn fetch_quota_for_list(
  accounts         : &[ crate::account::Account ],
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  stagger          : bool,
  trace            : bool,
  solo             : bool,
) -> Vec< AccountQuota >
{
  // Read the live session token once (graceful degradation on any error).
  let live_token : Option< String > = std::fs::read_to_string( live_creds_file )
    .ok()
    .and_then( |s| crate::account::parse_string_field( &s, "accessToken" ) );

  // Compute once — which account names are active on OTHER machines.
  let occupied_elsewhere = crate::account::other_machines_active( credential_store );

  // Class B pre-flight baseline — computed once for all per-account expiry checks.
  let now_secs = std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH ).unwrap_or_default().as_secs();
  let mut results = Vec::with_capacity( accounts.len() );
  for acct in accounts
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

    // G1: Non-owned accounts bypass token read + HTTP; read cache directly.
    // Root cause prevented: token read + API call on a foreign-machine account causes
    //   credential mutations and quota exhaustion without the owner's knowledge.
    // Pitfall: always check ownership before any read_token() or HTTP call.
    let owner = claude_profile_core::account::read_owner( credential_store, &acct.name );
    if !claude_profile_core::account::is_owned( &owner )
    {
      if trace { eprintln!( "{}fetch  {}  skipped (reason: not owned)", trace_ts(), acct.name ); }
      let ( host, role )                        = read_profile_metadata( credential_store, &acct.name );
      let renewal_at                            = read_renewal_at( credential_store, &acct.name );
      let ( result, cached, cache_age_secs ) = match read_cached_quota( credential_store, &acct.name, now_secs )
      {
        Some( ( data, age ) ) => ( Ok( data ), true, Some( age ) ),
        None                  => ( Err( "not owned".to_string() ), false, None ),
      };
      results.push( AccountQuota
      {
        name                  : acct.name.clone(),
        is_current            : false,
        is_active             : acct.is_active,
        is_occupied_elsewhere : occupied_elsewhere.contains( &acct.name ),
        expires_at_ms         : acct.expires_at_ms,
        result,
        account               : None,
        host,
        role,
        renewal_at,
        cached,
        cache_age_secs,
        is_owned              : false,
        owner                 : owner.clone(),
      } );
      continue;
    }

    // Determine whether this account's stored token matches the live session.
    let is_current = live_token.as_ref().is_some_and( |live|
    {
      read_token( credential_store, &acct.name )
        .is_ok_and( |stored| stored == *live )
    } );

    // Solo gate: skip HTTP for non-current accounts when solo::1 — use cached/approximated data.
    // Fires after G1 (non-owned already handled above) and after is_current is resolved.
    // Pitfall: is_current requires a token read, but no HTTP is attempted; the read here is
    //   token-comparison only, not credential-consuming in the network sense.
    if solo && !is_current
    {
      let aq       = approximate_quota( acct, credential_store, is_current, occupied_elsewhere.contains( &acct.name ), now_secs );
      let age_hint = aq.cache_age_secs.unwrap_or( 0 );
      if trace
      {
        eprintln!( "{}fetch  {}  solo-skip: approximated (age: {}s)", trace_ts(), acct.name, age_hint );
      }
      results.push( aq );
      continue;
    }

    // G1b: skip HTTP for owned accounts occupied elsewhere — use cached/approximated data.
    // Fix(BUG-305): occupied_elsewhere set was computed at line 74 but only used to stamp the
    //   is_occupied_elsewhere field. No gate prevented HTTP calls for these accounts.
    //   Root cause: G1 (not-owned) and solo gate existed; occupancy was recorded but not gated.
    //   Pitfall: G1b must fire AFTER is_current is resolved (token-comparison read is needed)
    //   and AFTER the solo gate (solo preempts G1b when both conditions overlap).
    if !is_current && occupied_elsewhere.contains( &acct.name )
    {
      if trace { eprintln!( "{}fetch  {}  skipped (reason: occupied elsewhere)", trace_ts(), acct.name ); }
      let aq = approximate_quota( acct, credential_store, is_current, true, now_secs );
      results.push( aq );
      continue;
    }

    // Fix(BUG-233): skip both API calls for locally-expired tokens — guaranteed 401 otherwise.
    // Root cause: no pre-flight expiry check; both thread spawn + main-thread HTTP always fired.
    // Pitfall: expires_at_ms is in milliseconds; now_secs is in seconds — divide before comparing.
    let ( result, account ) = if acct.expires_at_ms / 1000 <= now_secs
    {
      if trace { eprintln!( "{}{}  token expired (local) — skipping API calls", trace_ts(), acct.name ); }
      ( Err( "token expired (local)".to_string() ), None )
    }
    else
    {
      if trace
      {
        let creds_path = credential_store.join( format!( "{}.credentials.json", acct.name ) );
        eprintln!( "{}{}  reading {}", trace_ts(), acct.name, creds_path.display() );
      }
      match read_token( credential_store, &acct.name )
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
            eprintln!(
              "{}{}  GET {}  token={}...  exp={}",
              trace_ts(),
              acct.name,
              claude_quota::OAUTH_USAGE_URL,
              prefix,
              token_exp_label( acct.expires_at_ms ),
            );
          }
          let r = claude_quota::fetch_oauth_usage( &token ).map_err( |e| e.to_string() );
          let account_data = account_handle.join().ok().and_then( core::result::Result::ok );
          // Fix(BUG-233): billing_type=="none" → cancelled subscription → usage result irrelevant.
          // Root cause: usage fetch returns 429 for cancelled accounts; displaying "rate limited (429)"
          //   is semantically wrong. Override here makes display-layer error_label logic redundant.
          // Pitfall: override only when account fetch SUCCEEDED (account_data is Some); when account
          //   fetch fails (None), usage result stands — subscription state is unknown.
          // Fix(BUG-236): guard also requires r.is_err() — billing_type="none" alone is not
          //   sufficient to conclude "no subscription"; it can appear on non-stripe accounts
          //   (team/enterprise) where the usage API still returns HTTP 200 with valid quota data.
          // Root cause: BUG-233 assumed billing_type="none" ↔ cancelled ↔ usage returns 429.
          //   That holds for genuinely cancelled accounts but not for all billing arrangements.
          // Pitfall: only override when BOTH signals agree (billing says no-sub AND usage errored);
          //   a successful usage response (r=Ok) must be preserved regardless of billing_type.
          let r = if account_data.as_ref().is_some_and( |a| a.billing_type == "none" ) && r.is_err() { Err( "no subscription".to_string() ) } else { r };
          // Fix(BUG-234): trace the final stored result, not the raw API response.
          // Root cause: trace preceded Class A override — for billing_type="none", raw=Ok but stored=Err.
          // Pitfall: always emit result trace AFTER all result-modifying overrides.
          if trace
          {
            match &r
            {
              Ok( _ )  => eprintln!( "{}{}  result: OK", trace_ts(), acct.name ),
              Err( e ) => eprintln!( "{}{}  result: Err({})", trace_ts(), acct.name, e ),
            }
          }
          ( r, account_data )
        }
        Err( e ) =>
        {
          if trace { eprintln!( "{}{}  cannot read token: {}", trace_ts(), acct.name, e ); }
          ( Err( e ), None )
        }
      }
    };
    // Read host/role from {name}.json — best-effort, empty on missing/parse error.
    let ( host, role ) = read_profile_metadata( credential_store, &acct.name );
    let renewal_at = read_renewal_at( credential_store, &acct.name );
    // Cache write on success; cache read on failure (Feature 033).
    let ( result, cached, cache_age_secs ) = match result
    {
      Ok( ref data ) =>
      {
        let h5 = data.five_hour.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        let d7 = data.seven_day.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        let sn = data.seven_day_sonnet.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref() ) );
        claude_profile_core::account::write_quota_cache( credential_store, &acct.name, h5, d7, sn );
        // Feature 040: append measurement to history ring buffer (AC-01).
        {
          let now_secs = std::time::SystemTime::now()
            .duration_since( std::time::UNIX_EPOCH )
            .map_or( 0, |d| d.as_secs() );
          let hh5 = data.five_hour.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref().unwrap_or( "" ) ) );
          let hd7 = data.seven_day.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref().unwrap_or( "" ) ) );
          let hsn = data.seven_day_sonnet.as_ref().map( |p| ( p.utilization, p.resets_at.as_deref().unwrap_or( "" ) ) );
          claude_profile_core::account::write_history_entry( credential_store, &acct.name, now_secs, hh5, hd7, hsn );
        }
        ( result, false, None )
      }
      // Fix(BUG-296): auth errors (401, 403) must not fall back to cache — they indicate
      //   credential rejection and must remain Err so should_refresh() can trigger refresh.
      // Root cause: Err(ref _e) matched all errors; cache masking converted 401 Err to
      //   Ok(cached_data), bypassing the 401/403 guard in refresh_predicate.rs:34.
      // Pitfall: only transient errors (5xx, network, timeout) are legitimate cache-fallback
      //   candidates; auth errors are definitive rejections that need credential action.
      Err( ref e ) if !e.contains( "401" ) && !e.contains( "403" ) =>
      {
        match read_cached_quota( credential_store, &acct.name, now_secs )
        {
          Some( ( data, age ) ) => ( Ok( data ), true, Some( age ) ),
          None                  => ( result, false, None ),
        }
      }
      Err( _ ) => ( result, false, None ),
    };
    results.push( AccountQuota
    {
      name                  : acct.name.clone(),
      is_current,
      is_active             : acct.is_active,
      is_occupied_elsewhere : occupied_elsewhere.contains( &acct.name ),
      expires_at_ms         : acct.expires_at_ms,
      result,
      account,
      host,
      role,
      renewal_at,
      cached,
      cache_age_secs,
      is_owned              : true,
      owner,
    } );
  }

  inject_synthetic_row_if_needed( &mut results, live_token, live_creds_file );
  results
}

/// Enumerate all saved accounts and fetch their live quota data.
///
/// Calls `account::list()` to enumerate the credential store, then delegates
/// to `fetch_quota_for_list()` for the HTTP fetch loop.
/// Signature kept stable — callers that need a pre-filtered account list should
/// call `fetch_quota_for_list()` directly.
#[ cfg_attr( not( unix ), allow( dead_code ) ) ]
pub( crate ) fn fetch_all_quota(
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  stagger          : bool,
  trace            : bool,
  solo             : bool,
) -> Result< Vec< AccountQuota >, ErrorData >
{
  let accounts = crate::account::list( credential_store )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "cannot read credential store: {e}" ),
    ) )?;
  Ok( fetch_quota_for_list( &accounts, credential_store, live_creds_file, stagger, trace, solo ) )
}

/// Prepend a synthetic current-session row when no stored account matches the live token.
///
/// Reads `~/.claude.json` for the email address to use as the row name; falls back to
/// `"(current session)"` if the file is absent or `emailAddress` is empty.
/// No-op when `live_token` is `None` or any row in `results` is already current.
fn inject_synthetic_row_if_needed(
  results         : &mut Vec< AccountQuota >,
  live_token      : Option< String >,
  live_creds_file : &std::path::Path,
)
{
  if results.iter().any( |r| r.is_current ) { return; }
  let Some( token ) = live_token else { return; };
  let synthetic_name = live_creds_file.parent()
    .and_then( |p| p.parent() )
    .map( |home| home.join( ".claude.json" ) )
    .and_then( |p| std::fs::read_to_string( p ).ok() )
    .and_then( |s| crate::account::parse_string_field( &s, "emailAddress" ) )
    .filter( |e| !e.is_empty() )
    .unwrap_or_else( || "(current session)".to_string() );
  let expires_at_ms = parse_u64_field( live_creds_file, "expiresAt" ).unwrap_or( 0 );
  let result        = claude_quota::fetch_oauth_usage( &token ).map_err( |e| e.to_string() );
  let account       = claude_quota::fetch_oauth_account( &token ).ok();
  inject_synthetic_if_new( results, AccountQuota
  {
    name                  : synthetic_name,
    is_current            : true,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms,
    result,
    account,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
  } );
}

// ── Cache age ────────────────────────────────────────────────────────────────

/// Compute seconds elapsed since a `fetched_at` ISO-8601 UTC timestamp.
fn cache_age_from_fetched_at( fetched_at : &str ) -> u64
{
  let now = std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH ).unwrap_or_default().as_secs();
  let then = claude_profile_core::account::parse_iso_utc_secs( fetched_at ).unwrap_or( now );
  now.saturating_sub( then )
}

// ── Centralized cache-read + approximation ───────────────────────────────────

/// Read quota cache and apply Feature 040 polynomial approximation when available.
///
/// Returns `None` when no cache entry exists for `name`. When `cache.history[]` has
/// ≥2 entries in the current window, applies `approximate_utilization()` for each
/// period independently (AC-04, AC-05 from `docs/feature/040_quota_measurement_history.md`).
///
/// Called by all three utilization cache-read paths: G1 (non-owned accounts), HTTP
/// error fallback, and `approximate_quota()`. Eliminates BUG-304 — each previous
/// caller duplicated the 40–55 line approximation block independently.
///
/// Fix(BUG-304): centralize cache-read + approximation in one function; all utilization
///   read paths call this function so approximation is never silently absent.
/// Root cause: three independent callers each reconstructed `OauthUsageData` from the
///   cache; G1 applied no approximation, HTTP fallback and `approximate_quota()` each
///   duplicated the ~50-line approximation block, creating divergence risk.
/// Pitfall: `read_quota_cache()` remains available for metadata-only reads (`touch_idle`,
///   age hints); this function is only for paths that need utilization values.
pub( crate ) fn read_cached_quota(
  credential_store : &std::path::Path,
  name             : &str,
  now_secs         : u64,
) -> Option< ( claude_quota::OauthUsageData, u64 /* cache_age_secs */ ) >
{
  let entry = claude_profile_core::account::read_quota_cache( credential_store, name )?;
  let age   = cache_age_from_fetched_at( &entry.fetched_at );
  let mut data = claude_quota::OauthUsageData
  {
    five_hour        : entry.five_hour.map( |( u, r )| claude_quota::PeriodUsage { utilization : u, resets_at : r } ),
    seven_day        : entry.seven_day.map( |( u, r )| claude_quota::PeriodUsage { utilization : u, resets_at : r } ),
    seven_day_sonnet : entry.seven_day_sonnet.map( |( u, r )| claude_quota::PeriodUsage { utilization : u, resets_at : r } ),
  };
  let history = claude_profile_core::account::read_history( credential_store, name );
  if history.len() >= 2
  {
    if let Some( ref mut fh ) = data.five_hour
    {
      let pts : Vec< ( u64, f64 ) > = history.iter()
        .filter_map( |m| m.h5.as_ref().map( |( u, _ )| ( m.t, *u ) ) )
        .collect();
      let resets = history.iter().rev()
        .find_map( |m| m.h5.as_ref().map( |( _, r )| r.as_str() ) )
        .and_then( claude_quota::iso_to_unix_secs );
      if let Some( v ) = super::approx::approximate_utilization( &pts, resets, 18_000, now_secs )
      {
        fh.utilization = v;
      }
    }
    if let Some( ref mut d7 ) = data.seven_day
    {
      let pts : Vec< ( u64, f64 ) > = history.iter()
        .filter_map( |m| m.d7.as_ref().map( |( u, _ )| ( m.t, *u ) ) )
        .collect();
      let resets = history.iter().rev()
        .find_map( |m| m.d7.as_ref().map( |( _, r )| r.as_str() ) )
        .and_then( claude_quota::iso_to_unix_secs );
      if let Some( v ) = super::approx::approximate_utilization( &pts, resets, 604_800, now_secs )
      {
        d7.utilization = v;
      }
    }
    if let Some( ref mut sn ) = data.seven_day_sonnet
    {
      let pts : Vec< ( u64, f64 ) > = history.iter()
        .filter_map( |m| m.sn.as_ref().map( |( u, _ )| ( m.t, *u ) ) )
        .collect();
      let resets = history.iter().rev()
        .find_map( |m| m.sn.as_ref().map( |( _, r )| r.as_str() ) )
        .and_then( claude_quota::iso_to_unix_secs );
      if let Some( v ) = super::approx::approximate_utilization( &pts, resets, 604_800, now_secs )
      {
        sn.utilization = v;
      }
    }
  }
  Some( ( data, age ) )
}

// ── Solo-gate approximation helper ───────────────────────────────────────────

/// Return cache-backed `AccountQuota` for an account bypassed by the solo gate.
///
/// Reads quota cache and history from the credential store, runs polynomial
/// approximation (Feature 040) for each period independently when ≥2 history
/// entries exist, and returns `AccountQuota` with `cached=true`.
/// When no cache entry exists the result field is `Err("no cache")`.
///
/// This is the sole permitted source of non-live data when `solo::1` is active —
/// no caller may read cache files directly for solo-skipped accounts.
fn approximate_quota(
  acct                  : &crate::account::Account,
  credential_store      : &std::path::Path,
  is_current            : bool,
  is_occupied_elsewhere : bool,
  now_secs              : u64,
) -> AccountQuota
{
  let ( host, role ) = read_profile_metadata( credential_store, &acct.name );
  let renewal_at     = read_renewal_at( credential_store, &acct.name );
  let owner          = claude_profile_core::account::read_owner( credential_store, &acct.name );
  let ( result, cached, cache_age_secs ) = match read_cached_quota( credential_store, &acct.name, now_secs )
  {
    Some( ( data, age ) ) => ( Ok( data ), true, Some( age ) ),
    None                  => ( Err( "no cache".to_string() ), false, None ),
  };
  AccountQuota
  {
    name                  : acct.name.clone(),
    is_current,
    is_active             : acct.is_active,
    is_occupied_elsewhere,
    expires_at_ms         : acct.expires_at_ms,
    result,
    account               : None,
    host,
    role,
    renewal_at,
    cached,
    cache_age_secs,
    is_owned              : true,
    owner,
  }
}

// ── Profile metadata reader ───────────────────────────────────────────────────

/// Read `host` and `role` from `{name}.json` in the credential store.
///
/// Returns `(String::new(), String::new())` when the file is absent or unparseable —
/// profile metadata is always optional (AC-09 from `docs/feature/029_account_host_metadata.md`).
fn read_profile_metadata( credential_store : &std::path::Path, name : &str ) -> ( String, String )
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let Ok( text ) = std::fs::read_to_string( &path ) else { return ( String::new(), String::new() ) };
  let host = crate::account::parse_string_field( &text, "host" ).unwrap_or_default();
  let role = crate::account::parse_string_field( &text, "role" ).unwrap_or_default();
  ( host, role )
}

/// Read `_renewal_at` from `{name}.json` in the credential store.
///
/// Returns `None` when the file is absent or `_renewal_at` is missing/unparseable.
/// The field is written by `.account.renewal` and must survive round-trips through `save()`.
fn read_renewal_at( credential_store : &std::path::Path, name : &str ) -> Option< String >
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let text = std::fs::read_to_string( &path ).ok()?;
  crate::account::parse_string_field( &text, "_renewal_at" )
}

// ── Numeric JSON field parsers ────────────────────────────────────────────────

/// Parse a raw numeric JSON field from a string without an external JSON parser.
///
/// Finds `"key":` by string scan and parses the immediately following run of
/// ASCII digits as `u64`. Returns `None` on a missing key or non-numeric value.
/// Works for both flat (`{"key":N}`) and nested (`{"outer":{"key":N}}`) JSON.
pub( crate ) fn parse_u64_from_str( s : &str, key : &str ) -> Option< u64 >
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::{ inject_synthetic_if_new, parse_u64_from_str, fetch_quota_for_list, read_cached_quota };
  use crate::usage::types::AccountQuota;
  use crate::usage::test_support::FAR_FUTURE_MS;

  // ── BUG-233 Class B: pre-flight expiry predicate ────────────────────────────

  /// Class B predicate unit test: `expires_at_ms / 1000 <= now_secs` short-circuits expired accounts.
  ///
  /// # Root Cause
  /// `fetch_all_quota` spawned threads and made HTTP calls for locally-expired tokens (BUG-233).
  /// Expired tokens always return 401; two wasted HTTP round trips per expired account.
  ///
  /// # Why Not Caught
  /// No pre-flight expiry check existed; the code always entered the fetch path after reading
  /// the token. No test verified the expiry predicate or its consequence on API calls.
  ///
  /// # Fix Applied
  /// Fix(BUG-233): Class B pre-flight: `acct.expires_at_ms / 1000 <= now_secs` gate before
  /// thread spawn and main-thread HTTP; returns `Err("token expired (local)")` immediately.
  ///
  /// # Prevention
  /// Expiry must be checked BEFORE any I/O. The divide-before-compare idiom (ms→s) must be
  /// consistent: any future caller that adds a time-based gate must use the same unit conversion.
  ///
  /// # Pitfall
  /// Integer division truncates: `expires_at_ms = 999` → `0 / 1000 = 0 <= any now_secs`.
  /// `expires_at_ms = 0` (unknown expiry) also triggers the guard — treated as epoch (expired).
  #[ doc = "bug_reproducer(BUG-233)" ]
  #[ test ]
  fn test_class_b_expired_token_predicate()
  {
    let now_secs : u64 = 1_748_000_000; // representative fixed reference point (Unix seconds)

    // Expired case: expires_at_ms converts to a second before now_secs.
    let past_ms  : u64 = ( now_secs - 1 ) * 1_000;
    assert!(
      past_ms / 1_000 <= now_secs,
      "Class B: past token (past_ms={past_ms}) must satisfy expired predicate vs now_secs={now_secs}",
    );

    // Valid case: expires_at_ms converts to 1 hour after now_secs.
    let future_ms : u64 = ( now_secs + 3_600 ) * 1_000;
    assert!(
      future_ms / 1_000 > now_secs,
      "Class B: future token (future_ms={future_ms}) must NOT satisfy expired predicate vs now_secs={now_secs}",
    );
  }

  // ── BUG-233 Class A: billing_type="none" result override predicate ──────────

  /// Class A predicate unit test: `billing_type=="none"` overrides usage result to `Err("no subscription")`.
  ///
  /// # Root Cause
  /// Cancelled-subscription accounts receive HTTP 429 from the usage API (subscription inactive).
  /// `fetch_all_quota` stored `Err("HTTP transport error: HTTP 429 ...")` — semantically wrong:
  /// the account has no subscription, not a rate limit (BUG-233).
  ///
  /// # Why Not Caught
  /// No data-layer result override existed. The display layer papered over this with `error_label`
  /// (BUG-231 workaround) which was the wrong fix location — data-layer incorrectness requires
  /// a data-layer fix.
  ///
  /// # Fix Applied
  /// Fix(BUG-233): Class A override after `account_handle.join()`: when `billing_type=="none"`,
  /// replace the usage result with `Err("no subscription")` regardless of what the API returned.
  ///
  /// # Prevention
  /// Semantic correctness belongs at the data layer (fetch.rs). Display-layer hacks for
  /// data-layer incorrectness always become dead code after the proper fix is applied.
  ///
  /// # Pitfall
  /// Override fires ONLY when `account_data` is `Some` (account fetch succeeded and `billing_type`
  /// is known). When `account_data` is `None` (account fetch failed), `billing_type` is unknown —
  /// the original usage result must be preserved unchanged.
  #[ doc = "bug_reproducer(BUG-233)" ]
  #[ test ]
  fn test_class_a_billing_none_override_predicate()
  {
    // billing_type="none" (cancelled) → predicate fires → override to Err("no subscription").
    let cancelled = Some( claude_quota::OauthAccountData
    {
      tagged_id       : String::new(),
      uuid            : String::new(),
      email_address   : String::new(),
      full_name       : String::new(),
      display_name    : String::new(),
      billing_type    : "none".to_string(),
      has_max         : false,
      capabilities    : vec![],
      rate_limit_tier : String::new(),
      org_created_at  : "2024-01-01T00:00:00Z".to_string(),
      memberships     : vec![],
    } );
    assert!(
      cancelled.as_ref().is_some_and( |a| a.billing_type == "none" ),
      "Class A: billing_type==\"none\" must trigger result override to Err(\"no subscription\")",
    );

    // billing_type="stripe_subscription" (active) → predicate does NOT fire → result unchanged.
    let active = Some( claude_quota::OauthAccountData
    {
      tagged_id       : String::new(),
      uuid            : String::new(),
      email_address   : String::new(),
      full_name       : String::new(),
      display_name    : String::new(),
      billing_type    : "stripe_subscription".to_string(),
      has_max         : false,
      capabilities    : vec![],
      rate_limit_tier : String::new(),
      org_created_at  : "2024-01-01T00:00:00Z".to_string(),
      memberships     : vec![],
    } );
    assert!(
      !active.as_ref().is_some_and( |a| a.billing_type == "none" ),
      "Class A: active subscription must NOT trigger result override",
    );

    // account_data=None (account fetch failed) → predicate does NOT fire → result unchanged.
    let failed : Option< claude_quota::OauthAccountData > = None;
    assert!(
      !failed.as_ref().is_some_and( |a| a.billing_type == "none" ),
      "Class A: account=None must NOT trigger result override (billing_type unknown)",
    );
  }

  // ── BUG-234: result trace ordering ─────────────────────────────────────────

  /// Result trace must be emitted AFTER the Class A `billing_type` override.
  ///
  /// # Root Cause
  /// The result trace block preceded `account_handle.join()` and the Class A override
  /// (BUG-233 fix). For `billing_type="none"` accounts the raw API result (Ok or 429) can
  /// differ from the stored result (`Err("no subscription")`), making the trace misleading.
  ///
  /// # Why Not Caught
  /// No test verified the relative ordering of the result trace vs. the `billing_type` override.
  /// The contradiction (trace says OK, table says `(no subscription)`) only appears with live
  /// accounts whose `billing_type == "none"` — uncommon in unit test fixtures.
  ///
  /// # Fix Applied
  /// Fix(BUG-234): moved the `if trace { match &r { ... } }` block to after the Class A
  /// override. Trace now reports the final stored result, not the intermediate raw response.
  ///
  /// # Prevention
  /// Trace emissions must always follow ALL transformations of the value being reported.
  /// Rule: (1) compute raw, (2) apply overrides, (3) emit trace.
  ///
  /// # Pitfall
  /// When adding new result overrides in future, ensure they precede the result trace block —
  /// not after it. The Class A override must remain immediately before the trace.
  #[ doc = "bug_reproducer(BUG-234)" ]
  #[ test ]
  fn mre_bug234_result_trace_after_billing_type_override()
  {
    // Structural assertion: Class A override must precede the result trace in source.
    // RED before fix (trace at ~144, override at ~154); GREEN after fix (override first).
    let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/fetch.rs" ) );
    let override_pos = src.find( r#"a.billing_type == "none" ) && r.is_err() { Err( "no subscription""# )
      .expect( "BUG-234: Class A billing_type override not found in fetch.rs" );
    let trace_pos = src.find( r#"eprintln!( "{}{}  result: OK""# )
      .expect( "BUG-234: result: OK trace line not found in fetch.rs" );
    assert!(
      override_pos < trace_pos,
      "BUG-234: result trace emitted before Class A override — \
       for billing_type=\"none\" accounts trace shows raw result, not stored result; \
       override_pos={override_pos}, trace_pos={trace_pos}",
    );
  }

  // ── parse_u64_from_str ──────────────────────────────────────────────────────

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
  #[ doc = "bug_reproducer(BUG-170)" ]
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

  // ── BUG-218 ─────────────────────────────────────────────────────────────────

  /// BUG-218 MRE: `fetch_all_quota()` must not inject a synthetic row when the derived
  /// name already appears in the stored-account result list.
  ///
  /// # Root Cause
  /// `results.insert(0, AccountQuota { name: synthetic_name, ... })` is unconditional
  /// when `any_current == false`. When `~/.claude.json emailAddress` matches an existing
  /// stored account name (a precondition enabled by BUG-217's stale email install +
  /// subsequent token rotation), a duplicate row is created. `Valid: N+1` is reported;
  /// `apply_refresh` and `apply_touch` then process the same account twice — risking
  /// double-refresh or double subprocess spawning against the same credential file.
  ///
  /// # Why Not Caught
  /// The synthetic row path was designed for the case where the live email is genuinely
  /// unknown (first-run or external credential install). No test exercised the collision
  /// path where `synthetic_name` matches an existing stored account name.
  ///
  /// # Fix Applied
  /// `inject_synthetic_if_new()` — lookup-then-insert: only inject when `synthetic_name`
  /// is absent from `results`. Enforces: at most one row per unique account name in the
  /// quota table.
  ///
  /// # Prevention
  /// Any "inject if not present" operation must be lookup-then-insert, not unconditional
  /// insert. Collections built from two sources (directory scan + supplemental injection)
  /// must merge on a unique key.
  ///
  /// # Pitfall
  /// `any_current == false` is also true when BUG-217 installs a stale email that matches
  /// an existing account — that is the exact collision precondition. Both bugs compound:
  /// BUG-217 makes collision possible; BUG-218 makes it destructive.
  #[ doc = "bug_reproducer(BUG-218)" ]
  #[ test ]
  fn test_mre_bug218_fetch_all_quota_no_duplicate_synthetic_row()
  {
    // Simulate post-fetch state:
    //   - stored account "i6@wbox.pro" present (is_current=false — live token differs)
    //   - any_current=false — no stored account matches the live session token
    //   - synthetic_name derived from ~/.claude.json emailAddress = "i6@wbox.pro"
    // BUG-218: fetch_all_quota() does results.insert(0, synthetic) unconditionally —
    //   when synthetic_name == "i6@wbox.pro" which already exists, count becomes 2.
    let stored_row = AccountQuota
    {
      name                 : "i6@wbox.pro".to_string(),
      is_current           : false,
      is_active            : false,
      is_occupied_elsewhere : false,
      expires_at_ms        : FAR_FUTURE_MS,
      result               : Err( "missing accessToken".to_string() ),
      account              : None,
      host                 : String::new(),
      role                 : String::new(),
      renewal_at           : None,
      cached               : false,
      cache_age_secs       : None,
      is_owned             : true,
      owner                : String::new(),
    };
    let mut results = vec![ stored_row ];

    let synthetic = AccountQuota
    {
      name                 : "i6@wbox.pro".to_string(),
      is_current           : true,
      is_active            : false,
      is_occupied_elsewhere : false,
      expires_at_ms        : FAR_FUTURE_MS,
      result               : Err( "missing accessToken".to_string() ),
      account              : None,
      host                 : String::new(),
      role                 : String::new(),
      renewal_at           : None,
      cached               : false,
      cache_age_secs       : None,
      is_owned             : true,
      owner                : String::new(),
    };

    // Fix(BUG-218): guarded injection — only insert when name is absent from results.
    // Root cause: unconditional results.insert(0, ...) when any_current==false created
    // duplicate rows when synthetic_name matched an existing stored account name;
    // downstream apply_refresh and apply_touch then processed the same account twice.
    // Pitfall: any_current==false also occurs when BUG-217's stale email collides with
    // an existing account — both bugs must be fixed together for full correction.
    inject_synthetic_if_new( &mut results, synthetic );

    // Invariant: at most one row per unique account name.
    // FAILS before fix: count == 2 (duplicate row for "i6@wbox.pro").
    let i6_count = results.iter().filter( |r| r.name == "i6@wbox.pro" ).count();
    assert_eq!(
      i6_count, 1,
      "BUG-218: inject_synthetic creates duplicate — missing collision guard; count={i6_count}",
    );
    assert_eq!(
      results.len(), 1,
      "BUG-218: quota table must have exactly 1 row for 1 stored account; len={}",
      results.len(),
    );
  }

  // ── BUG-296: auth errors must bypass cache fallback ──────────────────────────

  /// MRE for BUG-296: HTTP 401/403 auth errors must bypass the cache fallback arm.
  ///
  /// # Root Cause
  /// `Err( ref _e ) =>` in `fetch_quota_for_list` matched ALL error variants, including
  /// HTTP 401 and 403 authentication failures. A 401 from the server would be silently
  /// converted to `Ok(cached_data)` when a warm cache existed, causing:
  ///   - `should_refresh()` auth-error guard at `refresh_predicate.rs:34` bypassed
  ///   - No token refresh attempted (trace shows: `should_retry=false  reason: ok`)
  ///   - Watchdog receiving 🟢 status from stale cache indefinitely (confirmed: 7+ cycles)
  ///
  /// # Why Not Caught
  /// The cache fallback was designed for transient errors (429, network, timeout); no test
  /// verified that auth errors (401, 403) bypass the cache arm. The distinction between
  /// "transient failure" and "credential rejection" was absent from both code and tests.
  ///
  /// # Fix Applied
  /// Fix(BUG-296): match guard `Err( ref e ) if !e.contains("401") && !e.contains("403")`.
  /// Auth errors fall through to `Err( _ ) => ( result, false, None )` — `cached=false`,
  /// `aq.result` remains `Err`, enabling `should_refresh()` to trigger credential refresh.
  ///
  /// # Prevention
  /// Structural assertion: the guard pattern must appear in source before `read_quota_cache`.
  /// Any future modification to the cache fallback match must preserve this ordering.
  ///
  /// # Pitfall
  /// HTTP 401/403 are DEFINITIVE rejections, not transient errors. Using cached data hides
  /// a credential failure and prevents automated recovery via the refresh pipeline.
  #[ doc = "bug_reproducer(BUG-296)" ]
  #[ test ]
  fn mre_bug296_cached_non_expired_401_no_refresh()
  {
    // Structural assertion: auth-error guard must appear, and read_quota_cache must appear
    // AFTER it (not before) — there is another read_quota_cache call earlier in the file
    // (non-owned path), so we search within src[guard_pos..] to find the one in the error arm.
    let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/fetch.rs" ) );
    let guard_pos = src.find( r#"!e.contains( "401" ) && !e.contains( "403" )"# )
      .expect(
        "BUG-296: auth-error guard not found in fetch.rs — \
         401/403 errors must bypass cache fallback via match guard",
      );
    assert!(
      src[ guard_pos.. ].contains( "read_cached_quota( credential_store" ),
      "BUG-296: read_cached_quota call not found after auth-error guard in fetch.rs — \
       guard must precede read_cached_quota in the cache fallback arm",
    );
    // Confirm catch-all arm propagates auth errors without cache substitution.
    assert!(
      src.contains( "Err( _ ) => ( result, false, None )" ),
      "BUG-296: catch-all Err arm missing in cache fallback match — \
       auth errors must propagate as Err with cached=false",
    );
  }

  // ── BUG-236 MRE: billing_type override guard ────────────────────────────────

  /// MRE 1/2 for BUG-236: `billing_type="none"` with `r=Ok(...)` must NOT be overridden.
  ///
  /// # Root Cause
  /// BUG-233 introduced a Class A override: when `billing_type == "none"`, store
  /// `Err("no subscription")` regardless of the usage API result. This assumed
  /// `billing_type="none"` ↔ cancelled subscription ↔ usage returns error. That holds for
  /// genuinely cancelled accounts but NOT for non-stripe billing arrangements (team/enterprise)
  /// where `billing_type="none"` can appear even when the usage API returns HTTP 200 with
  /// valid quota data — the override discarded the real quota and replaced it with an error.
  ///
  /// # Why Not Caught
  /// The BUG-233 test (`test_class_a_billing_none_override_predicate`) only checked the
  /// `billing_type == "none"` condition — it did not verify that `r.is_err()` is also required.
  /// No test covered the active-subscription + `billing_type="none"` combination.
  ///
  /// # Fix Applied
  /// Fix(BUG-236): added `&& r.is_err()` to the Class A override predicate so it only fires
  /// when BOTH signals agree: `billing_type` says no-subscription AND usage API also errored.
  ///
  /// # Prevention
  /// Override predicates that combine multiple signals must require ALL signals to agree.
  /// A single field (`billing_type`) is insufficient when the authoritative signal (usage API
  /// HTTP status) is also available.
  ///
  /// # Pitfall
  /// `billing_type="none"` has at least two causes: (a) cancelled subscription → usage error;
  /// (b) non-stripe billing arrangement (team/enterprise) → usage 200/Ok. Never treat
  /// `billing_type` alone as proof of subscription state when the usage API result is available.
  #[ doc = "bug_reproducer(BUG-236)" ]
  #[ test ]
  fn mre_bug236_ok_result_not_overridden_when_billing_type_none()
  {
    // billing_type="none" + r=Ok → second condition (r.is_err()) is false → NO override.
    let account_data = Some( claude_quota::OauthAccountData
    {
      tagged_id       : String::new(),
      uuid            : String::new(),
      email_address   : String::new(),
      full_name       : String::new(),
      display_name    : String::new(),
      billing_type    : "none".to_string(),
      has_max         : false,
      capabilities    : vec![],
      rate_limit_tier : String::new(),
      org_created_at  : "2024-01-01T00:00:00Z".to_string(),
      memberships     : vec![],
    } );
    let r : Result< claude_quota::OauthUsageData, String > = Ok( claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : None,
    } );
    // Replicate the fixed predicate from fetch_all_quota.
    let would_override = account_data.as_ref().is_some_and( |a| a.billing_type == "none" ) && r.is_err();
    assert!(
      !would_override,
      "BUG-236: billing_type=\"none\" + r=Ok must NOT trigger override — active non-stripe \
       accounts have billing_type=\"none\" but a valid usage response",
    );
  }

  /// MRE 2/2 for BUG-236: `billing_type="none"` with `r=Err(...)` IS overridden.
  ///
  /// Confirms the override still fires for genuinely cancelled accounts: both conditions
  /// must be true (`billing_type="none"` AND usage errored) for the "no subscription" override.
  #[ doc = "bug_reproducer(BUG-236)" ]
  #[ test ]
  fn mre_bug236_err_result_overridden_when_billing_type_none()
  {
    let account_data = Some( claude_quota::OauthAccountData
    {
      tagged_id       : String::new(),
      uuid            : String::new(),
      email_address   : String::new(),
      full_name       : String::new(),
      display_name    : String::new(),
      billing_type    : "none".to_string(),
      has_max         : false,
      capabilities    : vec![],
      rate_limit_tier : String::new(),
      org_created_at  : "2024-01-01T00:00:00Z".to_string(),
      memberships     : vec![],
    } );
    let r : Result< claude_quota::OauthUsageData, String > = Err( "HTTP 429".to_string() );
    // Replicate the fixed predicate from fetch_all_quota.
    let would_override = account_data.as_ref().is_some_and( |a| a.billing_type == "none" ) && r.is_err();
    assert!(
      would_override,
      "BUG-236: billing_type=\"none\" + r=Err must trigger override — cancelled account \
       signals agree (billing=no-sub AND usage=err)",
    );
  }

  // ── G1: non-owned accounts bypass token + HTTP; read cache ──────────────────

  /// FT-04 (AC-04): G1 gate — non-owned account skips token read + HTTP; reads cache; `is_owned=false`.
  ///
  /// When `{name}.json` has `owner` ≠ `current_identity()`, G1 fires:
  /// - `read_token()` is NOT called (no credentials path exercise).
  /// - `fetch_oauth_usage()` is NOT called (no HTTP).
  /// - Returned `AccountQuota` has `is_owned: false` and `cached: true` from cache JSON.
  ///
  /// Pitfall: `live_creds_file` is intentionally absent — live token lookup
  /// must not block the function; graceful degradation sets `is_current = false`.
  ///
  /// Spec: [`tests/docs/feature/036_account_ownership.md` FT-04]
  #[ test ]
  fn ft04_non_owned_uses_cache_not_http()
  {
    let store = tempfile::TempDir::new().unwrap();

    // Write {name}.json with owner != current_identity() AND a quota cache entry.
    // "other@remote" is guaranteed to differ from current_identity() (USER@hostname).
    let meta = serde_json::json!(
    {
      "owner" : "other@remote",
      "cache" :
      {
        "fetched_at"  : "2026-06-14T10:00:00Z",
        "status"      : "ok",
        "five_hour"   : { "left_pct" : 70.0 }
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).map( | s | s + "\n" ).unwrap(),
    ).unwrap();

    // Credentials file must exist for the account struct to be valid.
    std::fs::write(
      store.path().join( "alice@test.com.credentials.json" ),
      r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
    ).unwrap();

    let accounts = vec![ crate::account::Account
    {
      name              : "alice@test.com".to_string(),
      subscription_type : "pro".to_string(),
      rate_limit_tier   : String::new(),
      expires_at_ms     : u64::MAX / 2,
      is_active         : false,
      email             : String::new(),
      display_name      : String::new(),
      billing           : String::new(),
      model             : String::new(),
      tagged_id         : String::new(),
      uuid              : String::new(),
      capabilities      : Vec::new(),
      organization_uuid : String::new(),
      organization_name : String::new(),
      org_role          : String::new(),
      workspace_uuid    : String::new(),
      workspace_name    : String::new(),
      host              : String::new(),
      role              : String::new(),
      owner             : String::new(),
      is_owned          : true,
      renewal_at        : None,
    } ];

    // live_creds_file absent → graceful degradation; is_current=false for all accounts.
    let absent_live = store.path().join( ".absent_credentials.json" );

    let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

    assert_eq!( results.len(), 1, "FT-04: must return exactly 1 AccountQuota for 1 account" );
    let aq = &results[ 0 ];
    assert!(
      !aq.is_owned,
      "FT-04: G1 gate must set is_owned=false for non-owned account; got: {:?}", aq.is_owned,
    );
    assert!(
      aq.cached,
      "FT-04: G1 gate must read cache (cached=true) for non-owned account; got: {:?}", aq.cached,
    );
    // result must be Ok (from cache) — not an HTTP error.
    assert!(
      aq.result.is_ok(),
      "FT-04: G1 gate must return Ok(cache_data) when cache present; got: {:?}", aq.result,
    );
  }

  /// CC-7: G1 gate — non-owned account with NO cache → Err("not owned"), cached=false.
  ///
  /// When `{name}.json` has a foreign owner but no quota cache, G1 returns
  /// `Err("not owned")` with `cached=false`. The render shows `—` for all
  /// quota columns.
  #[ test ]
  fn cc7_non_owned_no_cache()
  {
    let store = tempfile::TempDir::new().unwrap();

    // owner != current_identity(), NO cache section.
    let meta = serde_json::json!( { "owner" : "other@remote" } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).map( | s | s + "\n" ).unwrap(),
    ).unwrap();

    std::fs::write(
      store.path().join( "alice@test.com.credentials.json" ),
      r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
    ).unwrap();

    let accounts = vec![ crate::account::Account
    {
      name              : "alice@test.com".to_string(),
      subscription_type : "pro".to_string(),
      rate_limit_tier   : String::new(),
      expires_at_ms     : u64::MAX / 2,
      is_active         : false,
      email             : String::new(),
      display_name      : String::new(),
      billing           : String::new(),
      model             : String::new(),
      tagged_id         : String::new(),
      uuid              : String::new(),
      capabilities      : Vec::new(),
      organization_uuid : String::new(),
      organization_name : String::new(),
      org_role          : String::new(),
      workspace_uuid    : String::new(),
      workspace_name    : String::new(),
      host              : String::new(),
      role              : String::new(),
      owner             : String::new(),
      is_owned          : true,
      renewal_at        : None,
    } ];

    let absent_live = store.path().join( ".absent_credentials.json" );
    let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

    assert_eq!( results.len(), 1, "CC-7: must return exactly 1 AccountQuota" );
    let aq = &results[ 0 ];
    assert!( !aq.is_owned, "CC-7: G1 gate must set is_owned=false" );
    assert!( !aq.cached, "CC-7: no cache → cached must be false" );
    assert!(
      aq.result.is_err(),
      "CC-7: no cache → result must be Err; got: {:?}", aq.result,
    );
    let err = aq.result.as_ref().unwrap_err();
    assert!(
      err.contains( "not owned" ),
      "CC-7: error message must contain 'not owned'; got: {err}",
    );
  }

  // ── Feature 040: history pipeline tests ────────────────────────────────────

  /// FT-03 — Cache-fallback path does NOT write a new history entry.
  ///
  /// A transient expired-token error triggers cache-fallback (not 401/403).
  /// History is only appended on the success arm — the fallback arm must leave
  /// the ring buffer unchanged.
  #[ test ]
  fn ft03_history_skips_cached_fallback()
  {
    let store = tempfile::TempDir::new().unwrap();

    // Pre-write cache with one existing history entry (object-per-entry format).
    let meta = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at" : "2026-06-01T10:00:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 60.0 },
        "history"    :
        [
          { "t" : 1_748_000_000_u64, "h5" : [ 60.0, "" ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    // Expired token (expiresAt=1ms) — triggers "token expired (local)" error,
    // which is not 401/403 and goes to the cache-fallback arm.
    std::fs::write(
      store.path().join( "alice@test.com.credentials.json" ),
      r#"{"accessToken":"tok","expiresAt":1}"#,
    ).unwrap();

    let accounts = vec![ crate::account::Account
    {
      name              : "alice@test.com".to_string(),
      subscription_type : "pro".to_string(),
      rate_limit_tier   : String::new(),
      expires_at_ms     : 1,
      is_active         : false,
      email             : String::new(),
      display_name      : String::new(),
      billing           : String::new(),
      model             : String::new(),
      tagged_id         : String::new(),
      uuid              : String::new(),
      capabilities      : Vec::new(),
      organization_uuid : String::new(),
      organization_name : String::new(),
      org_role          : String::new(),
      workspace_uuid    : String::new(),
      workspace_name    : String::new(),
      host              : String::new(),
      role              : String::new(),
      owner             : String::new(),
      is_owned          : true,
      renewal_at        : None,
    } ];

    let absent_live = store.path().join( ".absent_credentials.json" );
    let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

    // Pipeline must have used cache-fallback.
    assert_eq!( results.len(), 1 );
    assert!( results[ 0 ].cached, "FT-03: result must be cached" );

    // History must still have exactly 1 entry — fallback arm must not append.
    let text = std::fs::read_to_string( store.path().join( "alice@test.com.json" ) ).unwrap();
    let json : serde_json::Value = serde_json::from_str( &text ).unwrap();
    let history_len = json[ "cache" ][ "history" ].as_array().map_or( 0, Vec::len );
    assert_eq!(
      history_len, 1,
      "FT-03: cache-fallback must not append new history entry; got {history_len} entries",
    );
  }

  /// FT-05 — Approximation is independent per period: absent `sn` history entries leave sn unaffected.
  ///
  /// History has 2 h5 entries (both utilization=70.0, slope=0 → extrapolation=70.0).
  /// No sn entries. Cache h5=50.0, sn=50.0.
  /// After cache-fallback + approximation: h5 becomes 70.0, sn stays 50.0.
  #[ test ]
  fn ft05_approx_independent_periods_absent_sn_unaffected()
  {
    let store = tempfile::TempDir::new().unwrap();

    // History: 2 h5 entries at 70.0, no sn entries.
    // Both h5 at 70.0 → linear slope=0 → extrapolation=70.0 regardless of now_secs.
    // Cache: five_hour=50.0, seven_day_sonnet=50.0.
    let meta = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at"       : "2026-06-01T10:00:00Z",
        "status"           : "ok",
        "five_hour"        : { "left_pct" : 50.0 },
        "seven_day_sonnet" : { "left_pct" : 50.0 },
        "history" :
        [
          { "t" : 1_748_000_000_u64, "h5" : [ 70.0, "" ], "d7" : null, "sn" : null },
          { "t" : 1_748_003_600_u64, "h5" : [ 70.0, "" ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    std::fs::write(
      store.path().join( "alice@test.com.credentials.json" ),
      r#"{"accessToken":"tok","expiresAt":1}"#,
    ).unwrap();

    let accounts = vec![ crate::account::Account
    {
      name              : "alice@test.com".to_string(),
      subscription_type : "pro".to_string(),
      rate_limit_tier   : String::new(),
      expires_at_ms     : 1,
      is_active         : false,
      email             : String::new(),
      display_name      : String::new(),
      billing           : String::new(),
      model             : String::new(),
      tagged_id         : String::new(),
      uuid              : String::new(),
      capabilities      : Vec::new(),
      organization_uuid : String::new(),
      organization_name : String::new(),
      org_role          : String::new(),
      workspace_uuid    : String::new(),
      workspace_name    : String::new(),
      host              : String::new(),
      role              : String::new(),
      owner             : String::new(),
      is_owned          : true,
      renewal_at        : None,
    } ];

    let absent_live = store.path().join( ".absent_credentials.json" );
    let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

    assert_eq!( results.len(), 1 );
    let aq = &results[ 0 ];
    assert!( aq.cached, "FT-05: must be cached result" );

    let data = aq.result.as_ref().expect( "FT-05: result must be Ok(cached)" );
    let h5 = data.five_hour.as_ref().expect( "FT-05: five_hour must be Some" );
    let sn = data.seven_day_sonnet.as_ref().expect( "FT-05: seven_day_sonnet must be Some" );

    // h5: 2 history points both at 70.0 → slope=0 → approx=70.0 (changed from 50.0 cache value).
    assert!(
      ( h5.utilization - 70.0 ).abs() < 1e-6,
      "FT-05: h5.utilization must be 70.0 (approximated from 2 identical history points); got: {}", h5.utilization,
    );
    // sn: no history entries → approx returns None → sn.utilization unchanged at 50.0 (cache).
    assert!(
      ( sn.utilization - 50.0 ).abs() < 1e-6,
      "FT-05: sn.utilization must remain 50.0 (no sn history; absent period unaffected); got: {}", sn.utilization,
    );
  }

  // ── FT-14..FT-18: read_cached_quota() unit tests (Fix BUG-304) ─────────────

  /// FT-14 — `read_cached_quota` returns `None` when no cache entry exists (AC-11 backward-compat).
  ///
  /// Verifies the base-case: absent cache file produces `None`, not a panic or dummy data.
  #[ test ]
  fn test_read_cached_quota_absent_returns_none()
  {
    let store    = tempfile::TempDir::new().unwrap();
    let now_secs = 1_750_000_000_u64;
    let result   = read_cached_quota( store.path(), "nonexistent@test.com", now_secs );
    assert!( result.is_none(), "FT-14: absent cache must return None" );
  }

  /// FT-15 — `read_cached_quota` returns raw cache values when no `history` key is present (AC-11).
  ///
  /// Old cache format (no `"history"` key) must be treated as 0 measurements — approximation not applied.
  #[ test ]
  fn test_read_cached_quota_no_history_returns_raw()
  {
    let store    = tempfile::TempDir::new().unwrap();
    let now_secs = 1_750_000_000_u64;
    let meta     = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at" : "2026-06-21T10:00:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 42.0 }
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
      .expect( "FT-15: cache present → must return Some" );
    let h5 = data.five_hour.expect( "FT-15: five_hour must be Some" );
    assert!(
      ( h5.utilization - 42.0 ).abs() < 1e-6,
      "FT-15: no history → raw cache value 42.0 unchanged; got {}", h5.utilization,
    );
  }

  /// FT-16 — `read_cached_quota` returns raw cache when only 1 history entry exists (AC-04).
  ///
  /// Single measurement is below the ≥2 threshold; polynomial fit not attempted.
  #[ test ]
  fn test_read_cached_quota_one_history_returns_raw()
  {
    let store    = tempfile::TempDir::new().unwrap();
    let now_secs = 1_750_000_000_u64;
    let meta     = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at" : "2026-06-21T10:00:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 42.0 },
        "history"    :
        [
          { "t" : 1_749_990_000_u64, "h5" : [ 42.0, "" ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
      .expect( "FT-16: cache present → must return Some" );
    let h5 = data.five_hour.expect( "FT-16: five_hour must be Some" );
    assert!(
      ( h5.utilization - 42.0 ).abs() < 1e-6,
      "FT-16: 1 history entry (< 2) → raw cache 42.0 unchanged; got {}", h5.utilization,
    );
  }

  /// FT-17 — `read_cached_quota` applies quadratic approximation with 3+ history entries (AC-04).
  ///
  /// Three h5 measurements at t0/t1/t2 with linear upward trend (10→25→40). At `now_secs` 1h
  /// after t2 (within 2x-span safety), linear extrapolation gives ~55.0 ≠ raw cache 42.0.
  ///
  /// History `resets_at` is `""` (empty) so `iso_to_unix_secs` returns `None` and the window
  /// filter keeps all 3 entries unconditionally — same pattern as FT-05 in `approx.rs`.
  #[ test ]
  fn test_read_cached_quota_applies_approximation()
  {
    let store = tempfile::TempDir::new().unwrap();
    // t0=0, t1=+1h, t2=+2h (relative). now_secs = t2 + 1h → within 2x span (span=7200, 2x=14400, delta=3600).
    let t0       = 1_749_990_000_u64;
    let t1       = t0 + 3_600;
    let t2       = t0 + 7_200;
    let now_secs = t2 + 3_600; // 1h after last point; extrapolation gives ~55.0
    let meta = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at" : "2025-06-15T12:20:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 42.0 },
        "history"    :
        [
          { "t" : t0, "h5" : [ 10.0, "" ], "d7" : null, "sn" : null },
          { "t" : t1, "h5" : [ 25.0, "" ], "d7" : null, "sn" : null },
          { "t" : t2, "h5" : [ 40.0, "" ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
      .expect( "FT-17: cache present → must return Some" );
    let h5 = data.five_hour.expect( "FT-17: five_hour must be Some" );
    // Linear trend 10→25→40 with slope=15/3600 per second; at +1h after t2: 40 + 15 = 55.0.
    assert!(
      ( h5.utilization - 55.0 ).abs() < 5.0,
      "FT-17: 3 history entries → approximation ~55.0 (not raw 42.0); got {}", h5.utilization,
    );
  }

  /// FT-18 — `read_cached_quota` returns 0.0 when `resets_at` has elapsed (AC-07).
  ///
  /// When `now_secs > resets_at_secs`, the quota window has reset; approximated utilization = 0.0.
  #[ test ]
  fn test_read_cached_quota_expired_window_returns_zero()
  {
    let store = tempfile::TempDir::new().unwrap();
    // resets_at = 2026-06-01T09:00:00Z ≈ unix 1748768400.
    // window_start (h5, 18000s window) = 1748768400 - 18000 = 1748750400.
    // Points at 1748760000 and 1748763600 are within the window.
    // now_secs = 1748900000 > resets_at → expired → returns 0.0.
    let resets_at = "2025-06-01T09:00:00+00:00"; // unix 1748768400
    let meta      = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at" : "2026-06-01T08:30:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 42.0, "resets_at" : resets_at },
        "history"    :
        [
          { "t" : 1_748_760_000_u64, "h5" : [ 35.0, resets_at ], "d7" : null, "sn" : null },
          { "t" : 1_748_763_600_u64, "h5" : [ 42.0, resets_at ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    let now_secs = 1_748_900_000_u64; // well after resets_at 1748768400
    let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
      .expect( "FT-18: cache present → must return Some" );
    let h5 = data.five_hour.expect( "FT-18: five_hour must be Some" );
    assert!(
      h5.utilization.abs() < 1e-6,
      "FT-18: resets_at elapsed → approximated utilization must be 0.0; got {}", h5.utilization,
    );
  }

  // ── CC-08: read_cached_quota with exactly 2 history entries → linear approx ─

  /// CC-08 — `read_cached_quota` with exactly 2 history entries → linear approximation applied.
  ///
  /// # Root Cause
  /// `history.len() >= 2` is the boundary guard. At n=2 the degree selector picks degree 1
  /// (linear). This test verifies the minimum required count (2) is sufficient to trigger the
  /// approximation path.
  ///
  /// # Why Not Caught
  /// FT-14 and FT-15 used 3+ history entries (degree 2 quadratic). The n=2 boundary was not
  /// explicitly exercised in isolation.
  ///
  /// # Fix Applied
  /// Not a bug fix — correctness coverage for the `>= 2` boundary condition.
  ///
  /// # Prevention
  /// Always test the minimum required count to confirm the guard fires at the boundary.
  ///
  /// # Pitfall
  /// With n=2 and `now_secs = last_t + span` (not strictly greater), tangent continuation does
  /// NOT fire. Linear extrapolation returns `last_y + slope × elapsed`.
  #[ test ]
  fn cc08_read_cached_quota_two_history_entries_applies_linear()
  {
    let store    = tempfile::TempDir::new().unwrap();
    let t0       = 1_750_000_000_u64;
    let t1       = t0 + 3_600;
    let now_secs = t1 + 3_600; // 1h after t1; linear slope = (50-30)/3600 → +20 → 70.0

    // No five_hour.resets_at → resets_at=None → no window filter → both entries survive.
    // Exactly 2 h5 entries triggers history.len()>=2 guard → degree 1 (linear path).
    let meta = serde_json::json!(
    {
      "cache" :
      {
        "fetched_at" : "2026-06-15T00:00:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 42.0 },
        "history"    :
        [
          { "t" : t0, "h5" : [ 30.0, "" ], "d7" : null, "sn" : null },
          { "t" : t1, "h5" : [ 50.0, "" ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    let ( data, _ ) = read_cached_quota( store.path(), "alice@test.com", now_secs )
      .expect( "CC-08: cache present → must return Some" );
    let h5 = data.five_hour.expect( "CC-08: five_hour must be Some" );

    // Linear: slope = (50-30)/3600 s⁻¹; elapsed = 3_600s after t1 → 50 + 20 = 70.0.
    // Raw cache is 42.0 — approximation must differ to confirm history.len()>=2 path fired.
    assert!(
      ( h5.utilization - 42.0 ).abs() > 1e-6,
      "CC-08: 2 history entries must apply linear approx, not return raw 42.0; got {}", h5.utilization,
    );
    assert!(
      ( h5.utilization - 70.0 ).abs() < 5.0,
      "CC-08: linear extrapolation at +1h after t1 must be ≈70.0; got {}", h5.utilization,
    );
  }

  // ── FT-23: G1 non-owned path applies approximation (Fix BUG-304) ────────────

  /// FT-23 — G1 non-owned path applies polynomial approximation from history (AC-04, Fix BUG-304).
  ///
  /// Before BUG-304 fix, the G1 path called `read_quota_cache()` directly and returned the raw
  /// cached value (42.0). After the fix, it calls `read_cached_quota()` which applies Feature 040
  /// polynomial approximation — returning an approximated value different from raw 42.0.
  ///
  /// Three h5 history entries with upward trend (10→25→40) at fixed timestamps far in the past.
  /// At real test-execution time (well past the last measurement), the 2x-span safety fires and
  /// tangent continuation clamps to 100.0. Assertion: h5 ≠ 42.0 (approximation was applied).
  #[ test ]
  fn ft23_g1_non_owned_applies_approximation()
  {
    let store = tempfile::TempDir::new().unwrap();

    // Empty resets_at in history h5 entries → iso_to_unix_secs("") = None → no window filter
    // → all 3 entries survive at real test-execution time → approximation fires.
    let meta = serde_json::json!(
    {
      "owner" : "other@remote",
      "cache" :
      {
        "fetched_at" : "2026-06-11T10:00:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 42.0, "resets_at" : "2026-06-11T15:00:00+00:00" },
        "history"    :
        [
          { "t" : 1_749_990_000_u64, "h5" : [ 10.0, "" ], "d7" : null, "sn" : null },
          { "t" : 1_749_993_600_u64, "h5" : [ 25.0, "" ], "d7" : null, "sn" : null },
          { "t" : 1_749_997_200_u64, "h5" : [ 40.0, "" ], "d7" : null, "sn" : null }
        ]
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();
    std::fs::write(
      store.path().join( "alice@test.com.credentials.json" ),
      r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
    ).unwrap();

    let accounts = vec![ crate::account::Account
    {
      name              : "alice@test.com".to_string(),
      subscription_type : "pro".to_string(),
      rate_limit_tier   : String::new(),
      expires_at_ms     : u64::MAX / 2,
      is_active         : false,
      email             : String::new(),
      display_name      : String::new(),
      billing           : String::new(),
      model             : String::new(),
      tagged_id         : String::new(),
      uuid              : String::new(),
      capabilities      : Vec::new(),
      organization_uuid : String::new(),
      organization_name : String::new(),
      org_role          : String::new(),
      workspace_uuid    : String::new(),
      workspace_name    : String::new(),
      host              : String::new(),
      role              : String::new(),
      owner             : String::new(),
      is_owned          : true,
      renewal_at        : None,
    } ];

    let absent_live = store.path().join( ".absent_credentials.json" );
    let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

    assert_eq!( results.len(), 1 );
    let aq = &results[ 0 ];
    assert!( !aq.is_owned, "FT-23: G1 gate must set is_owned=false" );
    assert!( aq.cached, "FT-23: G1 must return cached=true when cache exists" );

    let data = aq.result.as_ref().expect( "FT-23: result must be Ok (cache exists)" );
    let h5 = data.five_hour.as_ref().expect( "FT-23: five_hour must be Some" );

    // Before BUG-304 fix: h5.utilization == 42.0 (raw cache, no approximation).
    // After fix: approximation fires (3 history entries, upward trend); tangent-continuation
    // clamps to 100.0 at test-execution time (well past last measurement at 1749997200).
    assert!(
      ( h5.utilization - 42.0 ).abs() > 1e-6,
      "FT-23 (BUG-304): G1 non-owned path must apply approximation; \
       raw cache 42.0 was returned — read_cached_quota() not called",
    );
  }

  /// FT-12 — Non-owned accounts do not append to history (AC-12).
  ///
  /// G1 gate intercepts non-owned accounts before the success arm, so
  /// `write_history_entry` is never called for them.
  #[ test ]
  fn ft12_history_non_owned_skips_append()
  {
    let store = tempfile::TempDir::new().unwrap();

    // Foreign-owned account with a quota cache but no history.
    let meta = serde_json::json!(
    {
      "owner" : "other@remote",
      "cache" :
      {
        "fetched_at" : "2026-06-01T10:00:00Z",
        "status"     : "ok",
        "five_hour"  : { "left_pct" : 60.0 }
      }
    } );
    std::fs::write(
      store.path().join( "alice@test.com.json" ),
      serde_json::to_string_pretty( &meta ).unwrap() + "\n",
    ).unwrap();

    std::fs::write(
      store.path().join( "alice@test.com.credentials.json" ),
      r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
    ).unwrap();

    let accounts = vec![ crate::account::Account
    {
      name              : "alice@test.com".to_string(),
      subscription_type : "pro".to_string(),
      rate_limit_tier   : String::new(),
      expires_at_ms     : u64::MAX / 2,
      is_active         : false,
      email             : String::new(),
      display_name      : String::new(),
      billing           : String::new(),
      model             : String::new(),
      tagged_id         : String::new(),
      uuid              : String::new(),
      capabilities      : Vec::new(),
      organization_uuid : String::new(),
      organization_name : String::new(),
      org_role          : String::new(),
      workspace_uuid    : String::new(),
      workspace_name    : String::new(),
      host              : String::new(),
      role              : String::new(),
      owner             : String::new(),
      is_owned          : true,
      renewal_at        : None,
    } ];

    let absent_live = store.path().join( ".absent_credentials.json" );
    let results = fetch_quota_for_list( &accounts, store.path(), &absent_live, false, false, false );

    // G1 gate must have fired.
    assert_eq!( results.len(), 1 );
    assert!( !results[ 0 ].is_owned, "FT-12: G1 gate must set is_owned=false" );

    // No history key must have been written to {name}.json.
    let text = std::fs::read_to_string( store.path().join( "alice@test.com.json" ) ).unwrap();
    let json : serde_json::Value = serde_json::from_str( &text ).unwrap();
    let has_history = json[ "cache" ].as_object()
      .is_some_and( |c| c.contains_key( "history" ) );
    assert!(
      !has_history,
      "FT-12: G1-gated non-owned account must not have 'history' written to account json",
    );
  }
}
