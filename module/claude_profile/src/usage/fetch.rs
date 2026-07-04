// Items are pub for test_bridge re-export; lints suppressed — internal API.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate ) ]
//! Account quota fetch pipeline.
//!
//! Enumerates saved accounts, fetches live OAuth usage data for each, and
//! provides numeric JSON-field parsing helpers used by the refresh pipeline.

use unilang::data::{ ErrorCode, ErrorData };
use super::types::AccountQuota;
use super::format::token_exp_label;
use super::fetch_cache::read_cached_quota;
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
pub fn inject_synthetic_if_new( results : &mut Vec< AccountQuota >, row : AccountQuota )
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
pub fn fetch_quota_for_list(
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

    // Cache-first guard: use recently-fetched cache without hitting the live API.
    // Prevents API flooding from rapid-succession .usage invocations — test suites and
    // polling scripts commonly invoke .usage every few seconds. Window tightened to 30s
    // for fresher data — note this permits live calls ~4x more often than the prior 120s
    // setting, which had been calibrated against the observed /oauth/usage burst-rate limit.
    // Pitfall: cache-first fires AFTER the G1/G1b/solo gates, so non-owned and
    // occupied-elsewhere accounts are already handled above; `is_current` is resolved.
    const CACHE_FRESH_SECS : u64 = 30;
    if let Some( ( data, age ) ) = read_cached_quota( credential_store, &acct.name, now_secs )
      .filter( |( _, age )| *age <= CACHE_FRESH_SECS )
    {
      if trace { eprintln!( "{}{}  cache-first ({}s old, skipping API)", trace_ts(), acct.name, age ); }
      let ( host, role ) = read_profile_metadata( credential_store, &acct.name );
      let renewal_at     = read_renewal_at( credential_store, &acct.name );
      results.push( AccountQuota
      {
        name                  : acct.name.clone(),
        is_current,
        is_active             : acct.is_active,
        is_occupied_elsewhere : occupied_elsewhere.contains( &acct.name ),
        expires_at_ms         : acct.expires_at_ms,
        result                : Ok( data ),
        account               : None,
        host,
        role,
        renewal_at,
        cached                : true,
        cache_age_secs        : Some( age ),
        is_owned              : true,
        owner,
      } );
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
pub fn parse_u64_from_str( s : &str, key : &str ) -> Option< u64 >
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
