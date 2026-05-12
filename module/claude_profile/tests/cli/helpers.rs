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
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs_with_env( args : &[ &str ], env : &[ ( &str, &str ) ] ) -> Output
{
  let mut cmd = Command::new( BIN );
  cmd.args( args );
  for ( k, v ) in env { cmd.env( k, v ); }
  cmd.output().expect( "failed to execute claude_profile binary" )
}

/// Run the binary with HOME removed entirely from the environment.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs_without_home( args : &[ &str ] ) -> Output
{
  Command::new( BIN )
  .args( args )
  .env_remove( "HOME" )
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
    std::fs::write( credential_store.join( "_active" ), name ).unwrap();
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
    std::fs::write( credential_store.join( "_active" ), name ).unwrap();
  }
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
