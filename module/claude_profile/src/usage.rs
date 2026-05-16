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
//! This row is excluded from `find_recommendation()` — it IS the current session.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use claude_quota::OauthUsageData;
use data_fmt::{ RowBuilder, TableFormatter, Format };
use crate::output::{ OutputFormat, OutputOptions, format_duration_secs, json_escape };

// ── Per-account quota result ───────────────────────────────────────────────────

struct AccountQuota
{
  name          : String,
  /// Live-token match: `accessToken` in `~/.claude/.credentials.json` equals this account's stored token.
  is_current    : bool,
  /// Active-marker match: `_active` file in the credential store names this account.
  is_active     : bool,
  expires_at_ms : u64,
  /// `Ok` = live quota fetched; `Err` = reason string (expired, network, etc.).
  result        : Result< OauthUsageData, String >,
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
fn fetch_all_quota(
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  stagger          : bool,
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
    let result = match read_token( credential_store, &acct.name )
    {
      Ok( token ) => claude_quota::fetch_oauth_usage( &token ).map_err( |e| e.to_string() ),
      Err( e )    => Err( e ),
    };
    results.push( AccountQuota
    {
      name          : acct.name.clone(),
      is_current,
      is_active     : acct.is_active,
      expires_at_ms : acct.expires_at_ms,
      result,
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
      results.insert( 0, AccountQuota
      {
        name : synthetic_name,
        is_current    : true,
        is_active     : false,
        expires_at_ms,
        result,
      } );
    }
  }

  Ok( results )
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Parse a raw numeric JSON field from a file without an external JSON parser.
///
/// Reads the file at `path`, finds `"key":` by string scan, and parses the
/// immediately following run of ASCII digits as `u64`. Returns `None` on any
/// I/O error, missing key, or non-numeric value.
fn parse_u64_field( path : &std::path::Path, key : &str ) -> Option< u64 >
{
  let s      = std::fs::read_to_string( path ).ok()?;
  let needle = format!( "\"{key}\":" );
  let start  = s.find( &needle )? + needle.len();
  let rest   = s[ start.. ].trim_start();
  let end    = rest.find( |c : char| !c.is_ascii_digit() ).unwrap_or( rest.len() );
  rest[ ..end ].parse().ok()
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

/// Shorten verbose quota error strings for display in the Status column.
///
/// `QuotaError::MissingHeader` (displays as `"rate-limit header missing: …"`) is
/// shortened to `"no header"`. All other strings pass through unchanged.
/// The caller is responsible for wrapping the result in parentheses.
fn shorten_error( reason : &str ) -> &str
{
  if reason.starts_with( "rate-limit header missing:" )
  {
    "no header"
  }
  else
  {
    reason
  }
}

/// Find the index of the recommended next account in an already-sorted slice.
///
/// Selects the non-active, non-current account with the highest `5h Left` among those
/// with valid quota data and a non-expired token (`expires_in_secs > 0`). Ties are
/// broken alphabetically — the first (alphabetically) account with equal `5h Left`
/// wins because the input is already alpha-sorted and strict-greater comparison
/// is used.
///
/// Skips `is_current` accounts (including the synthetic row) because the user is
/// already on that session — recommending it would be a no-op.
fn find_recommendation( accounts : &[ AccountQuota ], now_secs : u64 ) -> Option< usize >
{
  let mut best_idx    : Option< usize > = None;
  let mut best_5h_left : f64            = -1.0;

  for ( idx, aq ) in accounts.iter().enumerate()
  {
    if aq.is_active || aq.is_current { continue; }
    let expires_in_secs = ( aq.expires_at_ms / 1000 ).saturating_sub( now_secs );
    if expires_in_secs == 0 { continue; }
    if let Ok( data ) = &aq.result
    {
      let utilization = data.five_hour.as_ref().map_or( 0.0, |p| p.utilization );
      let left = 100.0 - utilization;
      if left > best_5h_left
      {
        best_5h_left = left;
        best_idx     = Some( idx );
      }
    }
  }

  best_idx
}

// ── Output renderers ───────────────────────────────────────────────────────────

/// Compute the 5 quota display cells for a successful OAuth usage fetch.
///
/// Returns `[5h_left, 5h_reset, 7d_left, 7d_son, 7d_reset]` as display strings.
/// Absent periods render as em-dash; absent reset timestamps render as em-dash.
fn quota_text_cells( data : &OauthUsageData, now_secs : u64 ) -> [ String; 5 ]
{
  let dash      = "\u{2014}".to_string();
  let pct_cell  = |util : Option< f64 >| -> String
  {
    util.map_or_else( || dash.clone(), |u| format!( "{:.0}%", 100.0 - u ) )
  };
  let reset_cell = |iso : Option< &str >| -> String
  {
    iso.and_then( claude_quota::iso_to_unix_secs )
      .map_or_else( || dash.clone(), |t|
        format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) )
      )
  };
  [
    pct_cell( data.five_hour.as_ref().map( |p| p.utilization ) ),
    reset_cell( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
    pct_cell( data.seven_day.as_ref().map( |p| p.utilization ) ),
    pct_cell( data.seven_day_sonnet.as_ref().map( |p| p.utilization ) ),
    reset_cell( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
  ]
}

/// Render quota results as a plain-text table using `data_fmt`.
///
/// Empty store renders `(no accounts configured)` without a table.
/// When ≥2 accounts have valid quota data and a recommendation exists, appends
/// a footer line: `Valid: X / Y   →  Next: name  (N% session left, token expires in Xh Ym)`.
fn render_text( accounts : &[ AccountQuota ] ) -> String
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

  let best_idx = find_recommendation( accounts, now_secs );

  let headers = vec![
    String::new(),
    "Account".to_string(),
    "Expires".to_string(),
    "5h Left".to_string(),
    "5h Reset".to_string(),
    "7d Left".to_string(),
    "7d(Son)".to_string(),
    "7d Reset".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for ( idx, aq ) in accounts.iter().enumerate()
  {
    // Four-level priority: ✓ (is_current) > * (is_active, not current) > → (recommendation) > blank.
    let flag_cell = if aq.is_current
    {
      "✓".to_string()
    }
    else if aq.is_active
    {
      "*".to_string()
    }
    else if best_idx == Some( idx )
    {
      "→".to_string()
    }
    else
    {
      String::new()
    };

    let expires_cell = compute_expires_cell( aq.expires_at_ms, now_secs );

    match &aq.result
    {
      Ok( data ) =>
      {
        let cells = quota_text_cells( data, now_secs );
        builder = builder.add_row( vec![
          flag_cell.into(), aq.name.clone().into(), expires_cell.into(),
          cells[ 0 ].clone().into(), cells[ 1 ].clone().into(),
          cells[ 2 ].clone().into(), cells[ 3 ].clone().into(), cells[ 4 ].clone().into(),
        ] );
      }
      Err( reason ) =>
      {
        let dash = "\u{2014}".to_string();
        builder = builder.add_row( vec![
          flag_cell.into(),
          aq.name.clone().into(),
          expires_cell.into(),
          dash.clone().into(),
          dash.clone().into(),
          dash.clone().into(),
          dash.clone().into(),
          format!( "({})", shorten_error( reason ) ).into(),
        ] );
      }
    }
  }

  let view  = builder.build_view();
  let table = Format::format( &TableFormatter::new(), &view ).unwrap_or_default();
  let body  = format!( "Quota\n\n{table}\n" );

  // Footer: shown when ≥2 valid accounts and a recommendation exists.
  let valid_count = accounts.iter().filter( |aq| aq.result.is_ok() ).count();
  if valid_count >= 2
  {
    if let Some( idx ) = best_idx
    {
      let rec = &accounts[ idx ];
      if let Ok( data ) = &rec.result
      {
        let expires_in_secs = ( rec.expires_at_ms / 1000 ).saturating_sub( now_secs );
        let expires_str     = format_duration_secs( expires_in_secs );
        let footer = format!(
          "Valid: {} / {}   →  Next: {}  ({:.0}% session left, token expires in {})\n",
          valid_count,
          accounts.len(),
          rec.name,
          data.five_hour.as_ref().map_or( 0.0, |p| 100.0 - p.utilization ),
          expires_str,
        );
        return format!( "{body}{footer}" );
      }
    }
  }

  body
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
\"session_5h_left_pct\":{session_pct},\"session_5h_resets_in_secs\":{session_reset},\
\"weekly_7d_left_pct\":{weekly_pct},\"weekly_7d_sonnet_left_pct\":{sonnet_pct},\
\"weekly_7d_resets_in_secs\":{weekly_reset}}}",
        )
      }
      Err( reason ) =>
      {
        format!(
          "{{\"account\":\"{name_esc}\",\"is_current\":{is_current_str},\"is_active\":{is_active_str},\
\"expires_in_secs\":{expires_in_secs},\"error\":\"{}\"}}",
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
/// # Parameters
///
/// - `interval_secs` — minimum seconds between full refresh cycles (≥ 30).
/// - `jitter_secs`   — maximum random seconds added to the cycle delay (0 = none).
#[ allow( unsafe_code ) ]
fn execute_live_mode(
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  interval_secs    : u64,
  jitter_secs      : u64,
) -> Result< OutputData, ErrorData >
{
  use std::os::raw::c_int;
  use core::sync::atomic::Ordering;
  use std::time::{ SystemTime, UNIX_EPOCH };
  use std::io::Write;

  type SignalFn = extern "C" fn( c_int );
  extern "C" { fn signal( signum : c_int, handler : SignalFn ) -> usize; }

  // Reset STOP_FLAG before registering the handler (safe across sequential test runs).
  STOP_FLAG.store( false, Ordering::Relaxed );
  // SAFETY: `on_sigint` is a valid C-compatible function pointer.
  unsafe { signal( 2, on_sigint ); } // 2 = SIGINT

  loop
  {
    if STOP_FLAG.load( Ordering::Relaxed ) { break; }

    // Clear terminal and move cursor to top-left on each cycle.
    print!( "\x1B[2J\x1B[H" );
    let _ = std::io::stdout().flush();

    // Fetch with per-account stagger delays (thunder-herd mitigation).
    let accounts = fetch_all_quota( credential_store, live_creds_file, true )?;

    let text = render_text( &accounts );
    print!( "{text}" );

    // Compute next-refresh wall-clock time.
    let now_secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
    let jitter_extra = if jitter_secs > 0
    {
      let nanos = u64::from( SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().subsec_nanos() );
      nanos % ( jitter_secs + 1 ) // 0..=jitter_secs seconds
    }
    else
    {
      0
    };
    let wait_secs = interval_secs + jitter_extra;
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

// ── Command handler ────────────────────────────────────────────────────────────

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
  let opts    = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let refresh = match cmd.arguments.get( "refresh" )
  {
    Some( Value::Integer( n ) ) => *n,
    _ => 0,
  };
  let live = match cmd.arguments.get( "live" )
  {
    Some( Value::Integer( n ) ) => *n,
    _ => 0_i64,
  };
  // Negative values map to 0, which is < 30 and will hit the interval guard.
  let interval = match cmd.arguments.get( "interval" )
  {
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _ => 30_u64,
  };
  let jitter = match cmd.arguments.get( "jitter" )
  {
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( 0 ),
    _ => 0_u64,
  };

  // Live-mode guards — fire BEFORE any network fetch, only when live::1 (AC-31).
  // Pitfall: placing these inside execute_live_mode() (after fetch_all_quota) would
  // require live credentials for offline guard tests it22–it24.
  if live == 1
  {
    if matches!( opts.format, OutputFormat::Json )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "live monitor mode is incompatible with format::json".to_string(),
      ) );
    }
    if interval < 30
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "interval must be >= 30".to_string(),
      ) );
    }
    if jitter > interval
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

  if live == 1
  {
    return execute_live_mode( &credential_store, &live_creds_file, interval, jitter );
  }

  let mut accounts = fetch_all_quota( &credential_store, &live_creds_file, false )?;

  // Retry-once on 401/403: if refresh::1 and any account returned an HTTP auth error,
  // attempt to refresh the live token via an isolated subprocess, then re-fetch once.
  if refresh == 1
  {
    let has_auth_error = accounts.iter().any( |aq|
      matches!( aq.result, Err( ref e ) if e.contains( "401" ) || e.contains( "403" ) )
    );
    if has_auth_error
    {
      if let Ok( creds_json ) = std::fs::read_to_string( &live_creds_file )
      {
        if let Ok( isolated ) = claude_runner_core::run_isolated( &creds_json, vec![], 30 )
        {
          // Only write back and retry if the subprocess actually refreshed credentials.
          if let Some( new_creds ) = isolated.credentials
          {
            let _ = std::fs::write( &live_creds_file, &new_creds );
            if let Ok( retried ) = fetch_all_quota( &credential_store, &live_creds_file, false )
            {
              accounts = retried;
            }
          }
        }
      }
    }
  }

  let content = match opts.format
  {
    OutputFormat::Json  => render_json( &accounts ),
    OutputFormat::Text
    | OutputFormat::Table => render_text( &accounts ),
  };

  Ok( OutputData::new( content, "text" ) )
}
