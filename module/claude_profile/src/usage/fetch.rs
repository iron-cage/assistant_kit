//! Account quota fetch pipeline.
//!
//! Enumerates saved accounts, fetches live OAuth usage data for each, and
//! provides numeric JSON-field parsing helpers used by the refresh pipeline.

use unilang::data::{ ErrorCode, ErrorData };
use super::types::AccountQuota;
use super::format::token_exp_label;

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
pub( crate ) fn fetch_all_quota(
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
          eprintln!(
            "[trace] {}  GET {}  token={}...  exp={}",
            acct.name,
            claude_quota::OAUTH_USAGE_URL,
            prefix,
            token_exp_label( acct.expires_at_ms ),
          );
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
      Err( e ) =>
      {
        if trace { eprintln!( "[trace] {}  cannot read token: {}", acct.name, e ); }
        ( Err( e ), None )
      }
    };
    // Read host/role from {name}.profile.json — best-effort, empty on missing/parse error.
    let ( host, role ) = read_profile_metadata( credential_store, &acct.name );
    let renewal_at = read_renewal_at( credential_store, &acct.name );
    results.push( AccountQuota
    {
      name          : acct.name.clone(),
      is_current,
      is_active     : acct.is_active,
      expires_at_ms : acct.expires_at_ms,
      result,
      account,
      host,
      role,
      renewal_at,
    } );
  }

  inject_synthetic_row_if_needed( &mut results, live_token, live_creds_file );
  Ok( results )
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
    name : synthetic_name,
    is_current    : true,
    is_active     : false,
    expires_at_ms,
    result,
    account,
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
  } );
}

// ── Profile metadata reader ───────────────────────────────────────────────────

/// Read `host` and `role` from `{name}.profile.json` in the credential store.
///
/// Returns `(String::new(), String::new())` when the file is absent or unparseable —
/// profile metadata is always optional (AC-09 from `docs/feature/029_account_host_metadata.md`).
fn read_profile_metadata( credential_store : &std::path::Path, name : &str ) -> ( String, String )
{
  let path = credential_store.join( format!( "{name}.profile.json" ) );
  let Ok( text ) = std::fs::read_to_string( &path ) else { return ( String::new(), String::new() ) };
  let host = crate::account::parse_string_field( &text, "host" ).unwrap_or_default();
  let role = crate::account::parse_string_field( &text, "role" ).unwrap_or_default();
  ( host, role )
}

/// Read `_renewal_at` from `{name}.claude.json` in the credential store.
///
/// Returns `None` when the file is absent or `_renewal_at` is missing/unparseable.
/// The field is written by `.account.renewal` and must survive round-trips through `save()`.
fn read_renewal_at( credential_store : &std::path::Path, name : &str ) -> Option< String >
{
  let path = credential_store.join( format!( "{name}.claude.json" ) );
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
  use super::{ inject_synthetic_if_new, parse_u64_from_str };
  use crate::usage::types::AccountQuota;
  use crate::usage::test_support::FAR_FUTURE_MS;

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
      name          : "i6@wbox.pro".to_string(),
      is_current    : false,
      is_active     : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "missing accessToken".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
    };
    let mut results = vec![ stored_row ];

    let synthetic = AccountQuota
    {
      name          : "i6@wbox.pro".to_string(),
      is_current    : true,
      is_active     : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "missing accessToken".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at    : None,
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
}
