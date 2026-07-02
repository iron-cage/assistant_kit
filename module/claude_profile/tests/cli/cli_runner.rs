//! Shared test helpers for `claude_profile` integration tests.
//!
//! Provides binary runner functions, assertion helpers, credential fixtures,
//! and filesystem setup utilities used across all integration test modules.

use std::process::{ Command, Output };

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Path to the compiled `clp` binary (resolved at compile time).
pub const BIN : &str = env!( "CARGO_BIN_EXE_clp" );

/// Assert that the current process is running inside a container.
///
/// Checked on every integration test entry point to enforce Invariant 009
/// (Container-Only Test Execution). Escape hatch: `VERB_LAYER=l0` bypasses
/// this check for authorized host development via direct nextest invocation.
///
/// # Panics
///
/// Panics with a human-readable message when run outside a container without
/// the `VERB_LAYER=l0` escape hatch.
fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_profile && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// Run the binary with the given argv fragments, inheriting the real HOME.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs( args : &[ &str ] ) -> Output
{
  assert_container();
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
  assert_container();
  // Fix(BUG-281): env_remove("PRO") prevents host $PRO from overriding the test HOME.
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
  assert_container();
  // Fix(BUG-281): env_remove("PRO") prevents host $PRO from substituting for HOME.
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

/// Write `~/.claude.json` with an `OAuthAccount` profile entry.
///
/// Used to test email retrieval at `v::1` and above for the active account.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_claude_json( home : &std::path::Path, email : &str )
{
  // Fix(BUG-270): write to $HOME/.claude.json — production code reads from claude_json_file()
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

/// Merge key-value pairs into `{credential_store}/{name}.json`.
///
/// Reads the existing file (or starts with `{}`), merges `pairs` into
/// the top-level object, and writes back. Used by all `write_account_*` helpers.
fn merge_account_meta( home : &std::path::Path, name : &str, pairs : serde_json::Value )
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut val : serde_json::Value = std::fs::read_to_string( &meta_path )
    .ok()
    .and_then( | s | serde_json::from_str( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let ( Some( dst ), Some( src ) ) = ( val.as_object_mut(), pairs.as_object() )
  {
    for ( k, v ) in src { dst.insert( k.clone(), v.clone() ); }
  }
  std::fs::write( meta_path, serde_json::to_string_pretty( &val ).map( | s | s + "\n" ).unwrap() ).unwrap();
}

/// Write `oauthAccount` snapshot into `{credential_store}/{name}.json`.
///
/// Used to pre-populate `.accounts` snapshot data for `email`, `display_name`,
/// `role`, and `billing` field tests. Mirrors what `account::save()` produces.
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
  merge_account_meta( home, name, serde_json::json!({
    "oauthAccount": {
      "emailAddress": email,
      "displayName": display,
      "organizationRole": role,
      "billingType": billing,
    }
  }) );
}

/// Write extended `oauthAccount` fields into `{credential_store}/{name}.json`.
///
/// Used to test `uuid::1` and `capabilities::1` in `.accounts`.
#[ inline ]
pub fn write_account_claude_json_extended(
  home         : &std::path::Path,
  name         : &str,
  tagged_id    : &str,
  uuid         : &str,
  capabilities : &[ &str ],
)
{
  let caps : Vec< serde_json::Value > = capabilities.iter()
    .map( | c | serde_json::Value::String( (*c).to_string() ) )
    .collect();
  merge_account_meta( home, name, serde_json::json!({
    "oauthAccount": {
      "taggedId": tagged_id,
      "uuid": uuid,
      "capabilities": caps,
    }
  }) );
}

/// Write `model` field into `{credential_store}/{name}.json`.
///
/// Used to pre-populate `.accounts` snapshot data for `model` field tests.
#[ inline ]
pub fn write_account_settings_json( home : &std::path::Path, name : &str, model : &str )
{
  merge_account_meta( home, name, serde_json::json!({ "model": model }) );
}

/// Write org identity fields into `{credential_store}/{name}.json`.
///
/// Used to pre-populate `.accounts` and `.credentials.status` org field tests.
#[ inline ]
pub fn write_account_roles_json(
  home     : &std::path::Path,
  name     : &str,
  org_uuid : &str,
  org_name : &str,
  org_role : &str,
)
{
  merge_account_meta( home, name, serde_json::json!({
    "organization_uuid": org_uuid,
    "organization_name": org_name,
    "organization_role": org_role,
    "workspace_uuid": null,
    "workspace_name": null,
  }) );
}

/// Write host and role metadata into `{credential_store}/{name}.json`.
///
/// Used to pre-populate host/role fields for `.usage cols::+host` / `.usage cols::+role` tests.
/// Pass `None` to omit a field (preserves existing value via merge).
#[ inline ]
pub fn write_account_profile_json(
  home : &std::path::Path,
  name : &str,
  host : Option< &str >,
  role : Option< &str >,
)
{
  let mut pairs = serde_json::Map::new();
  if let Some( h ) = host { pairs.insert( "host".into(), serde_json::Value::String( h.into() ) ); }
  if let Some( r ) = role { pairs.insert( "role".into(), serde_json::Value::String( r.into() ) ); }
  merge_account_meta( home, name, serde_json::Value::Object( pairs ) );
}

/// Write `_renewal_at` into `{credential_store}/{name}.json`.
///
/// Used to pre-populate renewal override tests without touching `oauthAccount`.
#[ inline ]
pub fn write_account_renewal_json( home : &std::path::Path, name : &str, renewal_at_iso : &str )
{
  merge_account_meta( home, name, serde_json::json!({ "_renewal_at": renewal_at_iso }) );
}

/// Write a quota cache entry into `{credential_store}/{name}.json`.
///
/// Simulates a prior successful API fetch so the cache-fallback path in `fetch.rs`
/// returns `Ok(data)` without a network call. Used in offline rotation tests where
/// accounts have no `accessToken` — `read_token()` returns `Err("missing")` (not 401/403),
/// triggering cache fallback.
///
/// - `h5_util` : consumed 5h quota percent (0–100). Gate 4 rejects `>= 85.0`.
/// - `d7_util` : consumed 7d quota percent (0–100). Gate 6 requires `100 - d7_util > 5.0`.
/// - `d7_resets_at` : optional ISO-8601 reset timestamp for the 7d period.
///
/// The cache uses the `left_pct` field (actual stored name per `account.rs:period_json`)
/// which stores the consumed utilization percentage despite the name suggesting "left".
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_quota_cache(
  home         : &std::path::Path,
  name         : &str,
  h5_util      : f64,
  d7_util      : f64,
  d7_resets_at : Option< &str >,
)
{
  let d7_resets : serde_json::Value = match d7_resets_at
  {
    Some( s ) => serde_json::Value::String( s.to_string() ),
    None      => serde_json::Value::Null,
  };
  merge_account_meta( home, name, serde_json::json!({
    "cache": {
      "fetched_at": "2026-01-01T00:00:00Z",
      "status": "ok",
      "five_hour": { "left_pct": h5_util },
      "seven_day": { "left_pct": d7_util, "resets_at": d7_resets }
    }
  }) );
}

/// Write `owner` field into `{credential_store}/{name}.json`.
///
/// Used to pre-populate ownership metadata for G5/G6/G7 gate tests.
#[ inline ]
pub fn write_account_owner( home : &std::path::Path, name : &str, owner : &str )
{
  merge_account_meta( home, name, serde_json::json!({ "owner": owner }) );
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
  // Pre-populate quota cache from the live snapshot so clp's 120s cache-first guard
  // (fetch.rs) skips the live API call entirely.  Without this, every parallel clp
  // invocation hits /api/oauth/usage and the burst of 429 rejections contaminates the
  // test run.  With the snapshot written here, total API calls per process = 1.
  if let Some( snap ) = live_quota_snapshot()
  {
    claude_profile::account::write_quota_cache(
      &credential_store,
      name,
      snap.five_hour.as_ref().map( |( u, r )| ( *u, r.as_deref() ) ),
      snap.seven_day.as_ref().map( |( u, r )| ( *u, r.as_deref() ) ),
      snap.seven_day_sonnet.as_ref().map( |( u, r )| ( *u, r.as_deref() ) ),
    );
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

// ── Live quota snapshot ────────────────────────────────────────────────────────

/// Raw quota data fetched from `/api/oauth/usage` for cache pre-population.
struct QuotaSnapshot
{
  five_hour        : Option< ( f64, Option< String > ) >,
  seven_day        : Option< ( f64, Option< String > ) >,
  seven_day_sonnet : Option< ( f64, Option< String > ) >,
}

/// Fetch `/api/oauth/usage` exactly once per test process (OnceLock-protected).
///
/// On first call one thread performs the live fetch; all parallel callers block
/// until it completes and then share the cached result.  Returns `None` on
/// HTTP 401/403 (auth failure) or absent live token.
///
/// Transient errors (429, network failure) fall back to the HOST credential store
/// cache via `host_quota_snapshot_from_cache()` — so tests proceed even when the
/// active token is currently rate-limited.
///
/// The snapshot pre-populates the per-account quota cache in `write_account_with_token`
/// so `clp .usage` hits fetch.rs's 120-second cache-first guard and skips the live
/// endpoint entirely — keeping total `/api/oauth/usage` calls to **1** per test process.
fn live_quota_snapshot() -> Option< &'static QuotaSnapshot >
{
  static SNAPSHOT : std::sync::OnceLock< Option< QuotaSnapshot > > = std::sync::OnceLock::new();
  SNAPSHOT.get_or_init( ||
  {
    let token = live_active_token()?;
    match claude_quota::fetch_oauth_usage( &token )
    {
      Ok( data ) => Some( QuotaSnapshot
      {
        five_hour        : data.five_hour.map( |p| ( p.utilization, p.resets_at ) ),
        seven_day        : data.seven_day.map( |p| ( p.utilization, p.resets_at ) ),
        seven_day_sonnet : data.seven_day_sonnet.map( |p| ( p.utilization, p.resets_at ) ),
      } ),
      Err( e ) =>
      {
        let msg = e.to_string();
        // Auth failures mean the token is bad — no point reading stale cache.
        if msg.contains( "HTTP 401" ) || msg.contains( "HTTP 403" ) { return None; }
        // Transient error (429, network) — read from HOST credential store cache.
        host_quota_snapshot_from_cache()
      }
    }
  } ).as_ref()
}

/// Read a `QuotaSnapshot` from the HOST credential store cache.
///
/// Used as a fallback when the live `/api/oauth/usage` call fails with a
/// transient error (e.g. 429). Reads the active account's `.json` metadata
/// from the real `PersistPaths` credential store and parses the `cache`
/// section written by `account::write_quota_cache`.
///
/// Returns `None` if the credential store, active marker, metadata file,
/// or cache section is absent or the cache status is not `"ok"`.
fn host_quota_snapshot_from_cache() -> Option< QuotaSnapshot >
{
  let persist          = claude_profile::PersistPaths::new().ok()?;
  let credential_store = persist.credential_store();
  let marker           = credential_store.join( claude_profile::account::active_marker_filename() );
  let raw_name         = std::fs::read_to_string( &marker ).ok()?;
  let name             = raw_name.trim();
  if name.is_empty() { return None; }
  let meta_str = std::fs::read_to_string( credential_store.join( format!( "{name}.json" ) ) ).ok()?;
  let val      : serde_json::Value = serde_json::from_str( &meta_str ).ok()?;
  let cache    = val.get( "cache" )?;
  if cache.get( "status" ).and_then( |v| v.as_str() ) != Some( "ok" ) { return None; }
  let parse_period = | key : &str | -> Option< ( f64, Option< String > ) >
  {
    let p      = cache.get( key )?;
    let util   = p.get( "left_pct" )?.as_f64()?;
    let resets = p.get( "resets_at" ).and_then( |v| v.as_str() ).map( std::string::ToString::to_string );
    Some( ( util, resets ) )
  };
  Some( QuotaSnapshot
  {
    five_hour        : parse_period( "five_hour" ),
    seven_day        : parse_period( "seven_day" ),
    seven_day_sonnet : parse_period( "seven_day_sonnet" ),
  } )
}

/// Assert that the live Anthropic API is reachable before running a `lim_it` test.
///
/// Probes `GET /api/oauth/account` with the active OAuth token on the first call
/// in this process; all parallel test threads block on `OnceLock` until the
/// single probe completes and share the cached result.
///
/// Uses `/api/oauth/account` (not `/api/oauth/usage`) — the account endpoint has
/// a higher rate limit than usage, so this probe does not burn the quota slot that
/// the tests themselves need. `live_quota_snapshot()` handles the usage fetch for
/// cache pre-population in `write_account_with_token`.
///
/// # Panics
///
/// Panics if the API is unreachable — the test cannot produce a valid result and
/// must fail loudly rather than silently passing with Err data.
#[ inline ]
pub fn require_live_api( label : &str )
{
  static LIVE_ACCOUNT_PROBE : std::sync::OnceLock< bool > = std::sync::OnceLock::new();
  let ok = LIVE_ACCOUNT_PROBE.get_or_init( ||
  {
    let token = live_active_token().unwrap_or_default();
    claude_quota::fetch_oauth_account( &token ).is_ok()
  } );
  assert!(
    *ok,
    "{label}: API unreachable — live API required for this test",
  );
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
