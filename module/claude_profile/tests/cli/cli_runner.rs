//! Shared test helpers for `claude_profile` integration tests.
//!
//! Provides binary runner functions, assertion helpers, credential fixtures,
//! and filesystem setup utilities used across all integration test modules.

use std::process::{ Command, Output };

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Path to the compiled `clp` binary (resolved at compile time).
pub const BIN : &str = env!( "CARGO_BIN_EXE_clp" );

/// Run the binary with the given argv fragments, inheriting the real HOME.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs( args : &[ &str ] ) -> Output
{
  Command::new( BIN )
  .args( args )
  .output()
  .expect( "failed to execute claude_profile binary" )
}

/// Run the binary with explicit environment overrides (added to inherited env).
///
/// `PRO` is always removed so the binary falls back to `HOME` for credential
/// store resolution — prevents the host `$PRO` from overriding the test HOME.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs_with_env( args : &[ &str ], env : &[ ( &str, &str ) ] ) -> Output
{
  // Fix(issue-pro-isolation): env_remove("PRO") prevents host $PRO from overriding the test HOME.
  // Root cause: PersistPaths::resolve_root() prefers $PRO over $HOME when $PRO is an existing dir;
  //   tests that only set HOME inherited $PRO from the runner, causing the binary to operate on
  //   the real production credential store instead of the test-supplied temp dir.
  // Pitfall: cmd.env("HOME", ...) alone is not enough for isolation — $PRO must also be removed.
  let mut cmd = Command::new( BIN );
  cmd.args( args );
  cmd.env_remove( "PRO" );
  for ( k, v ) in env { cmd.env( k, v ); }
  cmd.output().expect( "failed to execute claude_profile binary" )
}

/// Run the binary with HOME and PRO removed from the environment.
///
/// Removes both `HOME` and `PRO` so the binary cannot locate any credential
/// store — tests the "no home directory configured" error path.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs_without_home( args : &[ &str ] ) -> Output
{
  // Fix(issue-pro-isolation): env_remove("PRO") prevents host $PRO from substituting for HOME.
  // Root cause: removing $HOME but not $PRO left a silent fallback; the binary resolved the
  //   credential store via $PRO and succeeded instead of failing as the test expected.
  // Pitfall: Removing only $HOME is insufficient — $PRO takes priority and must also be removed.
  Command::new( BIN )
  .args( args )
  .env_remove( "HOME" )
  .env_remove( "PRO" )
  .output()
  .expect( "failed to execute claude_profile binary" )
}

/// Extract stdout as UTF-8 string.
#[ inline ]
#[ must_use ]
pub fn stdout( o : &Output ) -> String { String::from_utf8_lossy( &o.stdout ).to_string() }

/// Extract stderr as UTF-8 string.
#[ inline ]
#[ must_use ]
pub fn stderr( o : &Output ) -> String { String::from_utf8_lossy( &o.stderr ).to_string() }

/// Assert the exit code of a command output.
///
/// # Panics
///
/// Panics if the exit code does not match the expected value.
#[ inline ]
pub fn assert_exit( o : &Output, expected : i32 )
{
  let actual = o.status.code().unwrap_or( -1 );
  assert_eq!(
    actual, expected,
    "exit code: expected {expected}, got {actual}\nstdout: {}\nstderr: {}",
    stdout( o ), stderr( o ),
  );
}

/// Minimal credential JSON with configurable fields.
#[ inline ]
#[ must_use ]
pub fn credential_json( sub_type : &str, tier : &str, expires_at_ms : u64 ) -> String
{
  format!(
    r#"{{"oauthAccount":{{"subscriptionType":"{sub_type}","rateLimitTier":"{tier}"}},"expiresAt":{expires_at_ms}}}"#,
  )
}

/// Write a credentials file to `~/.claude/.credentials.json`.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_credentials( home : &std::path::Path, sub_type : &str, tier : &str, expires_at_ms : u64 )
{
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let creds = claude_dir.join( ".credentials.json" );
  std::fs::write( creds, credential_json( sub_type, tier, expires_at_ms ) ).unwrap();
}

/// Write a saved account credential file into `{home}/.persistent/claude/credential/{name}.credentials.json`
/// and optionally mark it active.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account( home : &std::path::Path, name : &str, sub_type : &str, tier : &str, expires_at_ms : u64, make_active : bool )
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let dest = credential_store.join( format!( "{name}.credentials.json" ) );
  std::fs::write( dest, credential_json( sub_type, tier, expires_at_ms ) ).unwrap();
  if make_active
  {
    std::fs::write( credential_store.join( claude_profile::account::active_marker_filename() ), name ).unwrap();
  }
}

/// Write `~/.claude/.claude.json` with an `OAuthAccount` profile entry.
///
/// Used to test email retrieval at `v::1` and above for the active account.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_claude_json( home : &std::path::Path, email : &str )
{
  // Fix(FR-19): write to $HOME/.claude.json — production code reads from claude_json_file()
  // Root cause: was writing to $HOME/.claude/.claude.json (one dir too deep), matching old bug.
  // Pitfall: fixture write path must equal production read path (Fixture–Production Path Alignment).
  let content = format!(
    r#"{{"oauthAccount":{{"emailAddress":"{email}"}}}}"#,
  );
  std::fs::write( home.join( ".claude.json" ), content ).unwrap();
}

/// Write `~/.claude.json` with a full `oauthAccount` profile (email, displayName, role, billing).
///
/// Used to test all four oauthAccount fields in `.credentials.status`.
///
/// # Panics
///
/// Panics if the file cannot be written.
#[ inline ]
pub fn write_claude_json_full(
  home    : &std::path::Path,
  email   : &str,
  display : &str,
  role    : &str,
  billing : &str,
)
{
  let content = format!(
    r#"{{"oauthAccount":{{"emailAddress":"{email}","displayName":"{display}","organizationRole":"{role}","billingType":"{billing}"}}}}"#,
  );
  std::fs::write( home.join( ".claude.json" ), content ).unwrap();
}

/// Write `~/.claude.json` with extended fields: `taggedId`, `uuid`, and `capabilities`.
///
/// Used to test `uuid::1` and `capabilities::1` in `.credentials.status`.
///
/// # Panics
///
/// Panics if the file cannot be written.
#[ inline ]
pub fn write_claude_json_extended(
  home         : &std::path::Path,
  tagged_id    : &str,
  uuid         : &str,
  capabilities : &[ &str ],
)
{
  let caps = capabilities.iter()
    .map( | c | format!( "\"{c}\"" ) )
    .collect::< Vec< _ > >()
    .join( "," );
  let content = format!(
    r#"{{"oauthAccount":{{"taggedId":"{tagged_id}","uuid":"{uuid}","capabilities":[{caps}]}}}}"#,
  );
  std::fs::write( home.join( ".claude.json" ), content ).unwrap();
}

/// Write `~/.claude/settings.json` with the given model value.
///
/// Used to test `model::1` field in `.credentials.status`.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_settings_json( home : &std::path::Path, model : &str )
{
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let content = format!( r#"{{"model":"{model}"}}"# );
  std::fs::write( claude_dir.join( "settings.json" ), content ).unwrap();
}

/// Write `{credential_store}/{name}.claude.json` with an `oauthAccount` snapshot.
///
/// Used to pre-populate `.accounts` snapshot data for `email`, `display_name`,
/// `role`, and `billing` field tests. Mirrors what `account::save()` produces.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_claude_json(
  home    : &std::path::Path,
  name    : &str,
  email   : &str,
  display : &str,
  role    : &str,
  billing : &str,
)
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let content = format!(
    r#"{{"oauthAccount":{{"emailAddress":"{email}","displayName":"{display}","organizationRole":"{role}","billingType":"{billing}"}}}}"#,
  );
  std::fs::write( credential_store.join( format!( "{name}.claude.json" ) ), content ).unwrap();
}

/// Write `{credential_store}/{name}.claude.json` with `taggedId`, `uuid`, and `capabilities`.
///
/// Used to test `uuid::1` and `capabilities::1` in `.accounts`.
/// Mirrors what `account::save()` produces (extended snapshot format).
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_claude_json_extended(
  home         : &std::path::Path,
  name         : &str,
  tagged_id    : &str,
  uuid         : &str,
  capabilities : &[ &str ],
)
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let caps = capabilities.iter()
    .map( | c | format!( "\"{c}\"" ) )
    .collect::< Vec< _ > >()
    .join( "," );
  let content = format!(
    r#"{{"oauthAccount":{{"taggedId":"{tagged_id}","uuid":"{uuid}","capabilities":[{caps}]}}}}"#,
  );
  std::fs::write( credential_store.join( format!( "{name}.claude.json" ) ), content ).unwrap();
}

/// Write `{credential_store}/{name}.settings.json` with a `model` field.
///
/// Used to pre-populate `.accounts` snapshot data for `model` field tests.
/// Mirrors what `account::save()` produces.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_settings_json( home : &std::path::Path, name : &str, model : &str )
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let content = format!( r#"{{"model":"{model}"}}"# );
  std::fs::write( credential_store.join( format!( "{name}.settings.json" ) ), content ).unwrap();
}

/// Write `{credential_store}/{name}.roles.json` with org identity fields.
///
/// Used to pre-populate `.accounts` and `.credentials.status` org field tests.
/// Mirrors the format written by `account::save()` when `fetch_claude_cli_roles` succeeds.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_roles_json(
  home     : &std::path::Path,
  name     : &str,
  org_uuid : &str,
  org_name : &str,
  org_role : &str,
)
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let content = format!(
    r#"{{"organization_uuid":"{org_uuid}","organization_name":"{org_name}","organization_role":"{org_role}","workspace_uuid":null,"workspace_name":null}}"#,
  );
  std::fs::write( credential_store.join( format!( "{name}.roles.json" ) ), content ).unwrap();
}

/// Write `{credential_store}/{name}.profile.json` with host and role metadata.
///
/// Used to pre-populate host/role fields for `.usage cols::+host` / `.usage cols::+role` tests
/// and for `.accounts host::1 role::1` display tests.
/// Mirrors the format written by `account::save()` when `host::` / `role::` params are supplied.
///
/// Pass `None` to omit a field entirely from the JSON.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_profile_json(
  home : &std::path::Path,
  name : &str,
  host : Option< &str >,
  role : Option< &str >,
)
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let mut fields = Vec::new();
  if let Some( h ) = host { fields.push( format!( r#""host":"{h}""# ) ); }
  if let Some( r ) = role { fields.push( format!( r#""role":"{r}""# ) ); }
  let content = format!( "{{{}}}", fields.join( "," ) );
  std::fs::write( credential_store.join( format!( "{name}.profile.json" ) ), content ).unwrap();
}

/// Write `{credential_store}/{name}.claude.json` containing only `_renewal_at`.
///
/// Used to pre-populate renewal override tests without touching `oauthAccount`.
/// The file is created as `{"_renewal_at":"<iso_ts>"}`.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_renewal_json( home : &std::path::Path, name : &str, renewal_at_iso : &str )
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let content = format!( r#"{{"_renewal_at":"{renewal_at_iso}"}}"# );
  std::fs::write( credential_store.join( format!( "{name}.claude.json" ) ), content ).unwrap();
}

/// Check whether an account credential file exists.
#[ inline ]
#[ must_use ]
pub fn account_exists( home : &std::path::Path, name : &str ) -> bool
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( format!( "{name}.credentials.json" ) ).exists()
}

/// Far-future timestamp (year ~2286) for "valid" tokens.
pub const FAR_FUTURE_MS : u64 = 9_999_999_999_000;

/// Timestamp 30 minutes from "now" — within default 3600s threshold.
///
/// # Panics
///
/// Panics if the system clock is before the Unix epoch.
#[ inline ]
#[ must_use ]
pub fn near_future_ms() -> u64
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  #[ allow( clippy::cast_possible_truncation ) ]
  let now_ms = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap().as_millis() as u64;
  now_ms + 30 * 60 * 1000 // +30 minutes
}

/// Timestamp in the past.
pub const PAST_MS : u64 = 1_000_000_000;

// ── Stats-cache helpers ───────────────────────────────────────────────────────

/// A single day entry for `dailyModelTokens` test fixtures.
#[ derive( Debug ) ]
pub struct DayEntry
{
  /// ISO date string (e.g. "2026-03-07").
  pub date   : &'static str,
  /// Per-model token counts: `(model_name, total_tokens)`.
  pub models : Vec< ( &'static str, u64 ) >,
}

/// Write a `stats-cache.json` with given `lastComputedDate` and daily entries.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_stats_cache(
  home              : &std::path::Path,
  last_computed     : Option< &str >,
  daily             : &[ DayEntry ],
)
{
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();

  let lcd = match last_computed
  {
    Some( d ) => format!( "\"lastComputedDate\":\"{d}\"," ),
    None      => String::new(),
  };

  let mut entries = Vec::new();
  for day in daily
  {
    let mut model_pairs = Vec::new();
    for ( model, tokens ) in &day.models
    {
      model_pairs.push( format!( "\"{model}\":{tokens}" ) );
    }
    entries.push( format!(
      "{{\"date\":\"{}\",\"tokensByModel\":{{{}}}}}",
      day.date,
      model_pairs.join( "," ),
    ) );
  }

  let json = format!(
    "{{{lcd}\"dailyModelTokens\":[{}]}}",
    entries.join( "," ),
  );

  std::fs::write( claude_dir.join( "stats-cache.json" ), json ).unwrap();
}

/// Write a raw string as `stats-cache.json`.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_stats_cache_raw( home : &std::path::Path, content : &str )
{
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "stats-cache.json" ), content ).unwrap();
}

// ── Live-quota helpers ────────────────────────────────────────────────────────

/// Credential JSON including an `accessToken` field.
///
/// Used by `write_account_with_token()` to create credential files that pass
/// through `read_token()` successfully, enabling `fetch_rate_limits()` to be called.
#[ inline ]
#[ must_use ]
pub fn credential_json_with_token( token : &str ) -> String
{
  format!(
    r#"{{"oauthAccount":{{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}},"expiresAt":{FAR_FUTURE_MS},"accessToken":"{token}"}}"#,
  )
}

/// Write a saved account credential file WITH an `accessToken` field.
///
/// Unlike `write_account()`, the resulting file contains `accessToken` so
/// `read_token()` will succeed and `fetch_rate_limits()` can be called.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_with_token(
  home        : &std::path::Path,
  name        : &str,
  token       : &str,
  make_active : bool,
)
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let dest = credential_store.join( format!( "{name}.credentials.json" ) );
  std::fs::write( dest, credential_json_with_token( token ) ).unwrap();
  if make_active
  {
    std::fs::write( credential_store.join( claude_profile::account::active_marker_filename() ), name ).unwrap();
  }
}

/// Write `~/.claude/.credentials.json` with an `accessToken` field.
///
/// Used to simulate a live authenticated session for `detect_current_account()` tests.
/// The credential JSON includes `accessToken` so the detection algorithm can match it
/// against saved account credential files.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_live_credentials_with_token( home : &std::path::Path, token : &str )
{
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let content = format!(
    r#"{{"accessToken":"{token}","oauthAccount":{{"subscriptionType":"max","rateLimitTier":"default"}},"expiresAt":{FAR_FUTURE_MS}}}"#,
  );
  std::fs::write( claude_dir.join( ".credentials.json" ), content ).unwrap();
}

/// Read the active OAuth access token from the real HOME credentials file.
///
/// Returns `None` if HOME is unset, the credentials file is absent, or
/// `accessToken` is not present. Used exclusively in `lim_it` tests that
/// require a real Anthropic API token.
#[ inline ]
#[ must_use ]
pub fn live_active_token() -> Option< String >
{
  let home    = std::env::var( "HOME" ).ok()?;
  let content = std::fs::read_to_string(
    std::path::Path::new( &home ).join( ".claude" ).join( ".credentials.json" ),
  ).ok()?;
  claude_profile::account::parse_string_field( &content, "accessToken" )
}

/// Check whether the live Anthropic API is accessible (not rate-limited).
///
/// Makes a single `curl` probe on first call, then caches the result in a
/// temp file for 60 seconds so all test processes within the same nextest
/// run share one probe.  Returns `false` when the API returns HTTP 429.
///
/// `lim_it` tests that require **successful** API responses should call
/// this guard.  Tests that handle error responses gracefully (e.g. testing
/// the "fetch failed" trace path) do NOT need this guard.
#[ inline ]
#[ must_use ]
pub fn require_live_api( label : &str ) -> bool
{
  let probe_path = std::env::temp_dir().join( ".lim_it_api_probe" );

  // Reuse a recent probe result (same nextest run — typically < 30s).
  if let Ok( meta ) = std::fs::metadata( &probe_path )
  {
    if meta.modified().ok()
      .and_then( |m| m.elapsed().ok() )
      .is_some_and( |age| age.as_secs() < 60 )
    {
      let cached = std::fs::read_to_string( &probe_path )
        .is_ok_and( |s| s.trim() == "1" );
      if !cached
      {
        eprintln!( "{label}: API rate-limited (cached probe) — skipping" );
      }
      return cached;
    }
  }

  // First probe in this nextest run: hit the API WITH the live token.
  // Per-token 429 means this session is rate-limited even if the global
  // endpoint would accept unauthenticated requests.
  let token = live_active_token().unwrap_or_default();
  let http_code = std::process::Command::new( "curl" )
    .args([
      "-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "5",
      "-H", &format!( "Authorization: Bearer {token}" ),
      "https://api.claude.ai/api/oauth/usage",
    ])
    .output()
    .ok()
    .and_then( |o| String::from_utf8( o.stdout ).ok() )
    .and_then( |s| s.trim().parse::< u16 >().ok() )
    .unwrap_or( 0 );
  let ok = http_code != 0 && http_code != 429;
  let _ = std::fs::write( &probe_path, if ok { "1" } else { "0" } );
  if !ok
  {
    eprintln!( "{label}: API rate-limited (HTTP {http_code}) — skipping" );
  }
  ok
}

/// Spawn the binary, wait `secs` seconds, kill it, and return all bytes written to stdout.
///
/// Reads from the piped stdout using a background thread so bytes accumulate even
/// while the main thread sleeps. After killing the child process the write-end of the
/// pipe is closed, causing `read_to_end` to return immediately with all buffered bytes.
///
/// Used by `lim_it` tests that need to observe live-monitor output before the process exits.
///
/// # Panics
///
/// Panics if the binary cannot be spawned.
#[ must_use ]
#[ inline ]
pub fn run_cs_bytes_for_secs( args : &[ &str ], env : &[ ( &str, &str ) ], secs : u64 ) -> Vec< u8 >
{
  use std::process::Stdio;
  use std::io::Read;
  use std::sync::{ Arc, Mutex };

  let mut cmd = std::process::Command::new( BIN );
  cmd.args( args ).env_remove( "PRO" );
  for ( k, v ) in env { cmd.env( k, v ); }
  cmd.stdout( Stdio::piped() );

  let mut child  = cmd.spawn().expect( "failed to spawn binary" );
  let mut stdout = child.stdout.take().unwrap();

  // Reader thread accumulates bytes so the pipe buffer does not fill and block the child.
  let collected : Arc< Mutex< Vec< u8 > > > = Arc::new( Mutex::new( Vec::new() ) );
  let collected2 = collected.clone();
  let reader = std::thread::spawn( move ||
  {
    let mut buf = [ 0u8; 4096 ];
    loop
    {
      match stdout.read( &mut buf )
      {
        Ok( 0 ) | Err( _ ) => break,
        Ok( n ) => collected2.lock().unwrap().extend_from_slice( &buf[ ..n ] ),
      }
    }
  } );

  std::thread::sleep( core::time::Duration::from_secs( secs ) );
  let _ = child.kill();
  let _ = reader.join();
  let _ = child.wait();

  let guard = collected.lock().unwrap();
  guard.clone()
}
