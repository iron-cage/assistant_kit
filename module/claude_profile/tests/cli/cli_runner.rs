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

/// Run the binary with explicit environment overrides AND explicit removals.
///
/// Like `run_cs_with_env`, but additionally removes each name in `remove` from
/// the child's environment — used to test "env var genuinely unset" behavior
/// without relying on the test runner's own environment happening to lack it.
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs_with_env_removing( args : &[ &str ], env : &[ ( &str, &str ) ], remove : &[ &str ] ) -> Output
{
  assert_container();
  let mut cmd = Command::new( BIN );
  cmd.args( args );
  cmd.env_remove( "PRO" );
  for name in remove { cmd.env_remove( name ); }
  for ( k, v ) in env { cmd.env( k, v ); }
  cmd.output().expect( "failed to execute claude_profile binary" )
}

/// Run the binary with env overrides, explicit removals, AND an explicit working directory.
///
/// The cwd-controlling counterpart of [`run_cs_with_env_removing`]: every other helper
/// lets the child inherit the test process's own cwd, which is precisely what a test
/// asserting on cwd-relative filesystem side effects must not do (BUG-550).
///
/// # Panics
///
/// Panics if the binary cannot be executed.
#[ inline ]
#[ must_use ]
pub fn run_cs_in_dir(
  args   : &[ &str ],
  env    : &[ ( &str, &str ) ],
  remove : &[ &str ],
  cwd    : &std::path::Path,
) -> Output
{
  assert_container();
  let mut cmd = Command::new( BIN );
  cmd.args( args );
  cmd.current_dir( cwd );
  cmd.env_remove( "PRO" );
  for name in remove { cmd.env_remove( name ); }
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
/// - `d7_util` : consumed 7d quota percent (0–100). Gate 6 requires `100 - d7_util > 3.0`.
/// - `d7_resets_at` : optional ISO-8601 reset timestamp for the 7d period.
///
/// This fixture still writes the legacy `left_pct` key; since BUG-540 the writer
/// (`quota_cache.rs:period_json`) stores the same consumed percentage under `utilization`,
/// and the reader accepts both — keeping this fixture as permanent legacy-path coverage.
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

/// Write `claim_lock` field into `{credential_store}/{name}.json`.
///
/// Used to pre-populate claim-lock metadata for Gate 9 tests (Feature 070).
#[ inline ]
pub fn write_account_claim_lock( home : &std::path::Path, name : &str, claim_lock : bool )
{
  merge_account_meta( home, name, serde_json::json!({ "claim_lock": claim_lock }) );
}

/// Write `reserve` field into `{credential_store}/{name}.json`.
///
/// Used to pre-populate reserve metadata for sort-key tests (Feature 070).
#[ inline ]
pub fn write_account_reserve( home : &std::path::Path, name : &str, reserve : bool )
{
  merge_account_meta( home, name, serde_json::json!({ "reserve": reserve }) );
}

/// Write `inference_provider` field into `{credential_store}/{name}.json`.
///
/// Used to pre-populate provider metadata for `.accounts`/`.usage` read-path tests (Feature 072).
#[ inline ]
pub fn write_account_inference_provider( home : &std::path::Path, name : &str, inference_provider : &str )
{
  merge_account_meta( home, name, serde_json::json!({ "inference_provider": inference_provider }) );
}

/// Write `tags` array into `{credential_store}/{name}.json`.
///
/// Used to pre-populate tag metadata for `.account.tag`/`.tags`/`.accounts`
/// read-path tests (Feature 075).
#[ inline ]
pub fn write_account_tags( home : &std::path::Path, name : &str, tags : &[ &str ] )
{
  let arr : Vec< serde_json::Value > = tags.iter()
    .map( | t | serde_json::Value::String( (*t).to_string() ) )
    .collect();
  merge_account_meta( home, name, serde_json::json!({ "tags": arr }) );
}

/// The credential store directory under a test HOME.
///
/// For fixtures that write store-level files directly (`_filter_*`, `_active_*`)
/// or byte-compare store content around read-only commands.
#[ inline ]
#[ must_use ]
pub fn credential_store_dir( home : &std::path::Path ) -> std::path::PathBuf
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
}

/// Write a `_filter_{slug}` Identity tag-filter file into the credential store.
///
/// `slug` is the pre-sanitized `{hostname}_{user}` suffix — the caller controls
/// the child's `HOSTNAME`/`USER` env so the binary resolves the same slug
/// (Feature 076, `docs/schema/009_identity_filter_json.md`).
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_filter_file( home : &std::path::Path, slug : &str, include : &[ &str ], exclude : &[ &str ] )
{
  let store = credential_store_dir( home );
  std::fs::create_dir_all( &store ).unwrap();
  let json = serde_json::json!( { "include": include, "exclude": exclude } );
  std::fs::write(
    store.join( format!( "_filter_{slug}" ) ),
    serde_json::to_string_pretty( &json ).unwrap() + "\n",
  ).unwrap();
}

/// Write an `_active_{slug}` marker naming `name` as that Identity's active account.
///
/// Unlike [`write_account`]'s `make_active` (which derives the slug from the
/// test process's own env), the slug here is explicit — for tests whose child
/// env (`HOSTNAME`/`USER`) differs from the test runner's.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_active_marker( home : &std::path::Path, slug : &str, name : &str )
{
  let store = credential_store_dir( home );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( format!( "_active_{slug}" ) ), name ).unwrap();
}

/// Split output into lines with runs of whitespace collapsed to single spaces.
///
/// For column-layout assertions (`.tags`, `.identities`, table renders) that
/// must not depend on padding widths.
#[ inline ]
#[ must_use ]
pub fn normalized_lines( s : &str ) -> Vec< String >
{
  s.lines()
    .map( | l | l.split_whitespace().collect::< Vec< _ > >().join( " " ) )
    .collect()
}

/// Check whether an account credential file exists.
#[ inline ]
#[ must_use ]
pub fn account_exists( home : &std::path::Path, name : &str ) -> bool
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( format!( "{name}.credentials.json" ) ).exists()
}

/// Read and parse `{credential_store}/{name}.json` as a [`serde_json::Value`].
///
/// Returns `serde_json::json!({})` when the file is absent (unset fields default `false`).
///
/// # Panics
///
/// Panics if the file exists but is not valid JSON.
#[ inline ]
#[ must_use ]
pub fn read_account_meta( home : &std::path::Path, name : &str ) -> serde_json::Value
{
  let meta_path = home.join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( format!( "{name}.json" ) );
  std::fs::read_to_string( &meta_path )
    .ok()
    .map_or_else( || serde_json::json!( {} ), | s | serde_json::from_str( &s ).expect( "account meta must be valid JSON" ) )
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
  // Pre-populate quota cache from the live snapshot so clp's 30s cache-first guard
  // (fetch.rs) skips the live API call entirely.  Without this, every parallel clp
  // invocation hits /api/oauth/usage and the burst of 429 rejections contaminates the
  // test run.  With the snapshot written here (file-cached across test processes),
  // total API calls ≈ 1 per RUN — not per process; see live_quota_snapshot().
  let snap = live_quota_snapshot();
  claude_profile::account::write_quota_cache(
    &credential_store,
    name,
    snap.five_hour.as_ref().map( |( u, r )| ( *u, r.as_deref() ) ),
    snap.seven_day.as_ref().map( |( u, r )| ( *u, r.as_deref() ) ),
    snap.seven_day_sonnet.as_ref().map( |( u, r )| ( *u, r.as_deref() ) ),
  );
}

/// Write a saved account credential file WITH `accessToken` and WITHOUT any quota cache.
///
/// Seam-test counterpart of [`write_account_with_token`]: that helper pre-seeds the
/// quota cache from the LIVE API (live-token lane); this one leaves the cache empty
/// so `fetch.rs` takes the HTTP path — required by `CLAUDE_QUOTA_BASE_URL` seam tests
/// that assert which requests the pipeline actually makes against a local server.
///
/// # Panics
///
/// Panics if the directory or file cannot be created.
#[ inline ]
pub fn write_account_with_token_uncached(
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

/// Write a deterministic quota cache for `name` with chosen utilization values.
///
/// Offline counterpart of the live-snapshot seeding in `write_account_with_token()`:
/// unowned fixture accounts (no owner file) render straight from this cache via the
/// G1 not-owned gate in `fetch.rs` — no token, no HTTP — so a test controls the
/// exact `5h/7d Left` percentages that filters and display see (`Left = 100 - utilization`).
///
/// `None` omits that quota window entirely (absent data); `seven_day_sonnet` and
/// `resets_at` are always absent — no current consumer needs them controlled.
///
/// # Panics
///
/// Panics if the credential-store directory cannot be created — test fixtures
/// fail loudly on setup errors rather than letting a test run against missing state.
#[ inline ]
pub fn seed_quota_cache(
  home      : &std::path::Path,
  name      : &str,
  five_hour : Option< f64 >,
  seven_day : Option< f64 >,
)
{
  let credential_store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  claude_profile::account::write_quota_cache(
    &credential_store,
    name,
    five_hour.map( | u | ( u, None ) ),
    seven_day.map( | u | ( u, None ) ),
    None,
  );
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

/// Location of the cross-process snapshot cache: the shared compilation cache
/// (`CARGO_TARGET_DIR` — the runbox working volume in-container) so every test
/// process in a run resolves the same file; temp dir as bare-host fallback.
fn snapshot_cache_path() -> std::path::PathBuf
{
  let root = std::env::var( "CARGO_TARGET_DIR" )
    .map_or_else( |_| std::env::temp_dir(), std::path::PathBuf::from );
  root.join( "-live_quota_snapshot.tsv" )
}

/// True when the cache file exists and is younger than the sharing window.
///
/// 300s covers one full suite run plus an immediate re-run. Staleness is
/// harmless for correctness — the snapshot only pre-seeds quota caches whose
/// displayed values tests treat as opaque live data — the TTL merely bounds
/// how old that data can get.
fn snapshot_cache_fresh( path : &std::path::Path ) -> bool
{
  const TTL : core::time::Duration = core::time::Duration::from_secs( 300 );
  std::fs::metadata( path )
    .and_then( |m| m.modified() )
    .ok()
    .and_then( |t| t.elapsed().ok() )
    .is_some_and( |age| age < TTL )
}

/// Parse the cache file (one `window\tutilization\tresets_at` line per present
/// window, `-` for absent `resets_at`). Any anomaly — empty file, unknown key,
/// malformed field — returns `None` so the caller falls back to a live fetch;
/// the cache can only save requests, never substitute bad data.
fn read_snapshot_cache( path : &std::path::Path ) -> Option< QuotaSnapshot >
{
  let content = std::fs::read_to_string( path ).ok()?;
  if content.trim().is_empty()
  {
    return None;
  }
  let mut snap = QuotaSnapshot { five_hour : None, seven_day : None, seven_day_sonnet : None };
  for line in content.lines()
  {
    let mut parts = line.split( '\t' );
    let key       = parts.next()?;
    let util : f64 = parts.next()?.parse().ok()?;
    let resets = match parts.next()?
    {
      "-" => None,
      s   => Some( s.to_string() ),
    };
    match key
    {
      "five_hour"        => snap.five_hour        = Some( ( util, resets ) ),
      "seven_day"        => snap.seven_day        = Some( ( util, resets ) ),
      "seven_day_sonnet" => snap.seven_day_sonnet = Some( ( util, resets ) ),
      _                  => return None,
    }
  }
  Some( snap )
}

/// Write the cache best-effort (tmp file + atomic rename; a failed write just
/// means the next process fetches live).
fn write_snapshot_cache( path : &std::path::Path, snap : &QuotaSnapshot )
{
  use core::fmt::Write as _;
  let mut out = String::new();
  for ( key, val ) in
  [
    ( "five_hour", &snap.five_hour ),
    ( "seven_day", &snap.seven_day ),
    ( "seven_day_sonnet", &snap.seven_day_sonnet ),
  ]
  {
    if let Some( ( util, resets ) ) = val
    {
      let _ = writeln!( out, "{key}\t{util}\t{}", resets.as_deref().unwrap_or( "-" ) );
    }
  }
  let tmp = path.with_extension( format!( "tmp.{}", std::process::id() ) );
  if std::fs::write( &tmp, out ).is_ok()
  {
    let _ = std::fs::rename( &tmp, path );
  }
}

/// Fetch `/api/oauth/usage` once per test RUN — not once per test process.
///
/// Two cache layers. The `OnceLock` dedups threads within one process; the file
/// cache under `snapshot_cache_path()` dedups across processes. The second layer
/// exists because nextest runs one PROCESS per test: ~100 live-seeded tests per
/// suite would otherwise fire ~100 usage fetches per run, and the endpoint's
/// rolling budget rejects that volume (HTTP 429) no matter how the requests are
/// spaced — serialization and retries (.config/nextest.toml) handle burst and
/// transient windows, this layer removes the volume itself.
///
/// Panics on any failure — missing token, auth failure (401/403), rate limit (429),
/// or network error. The live API is required for this test; no cached or synthetic
/// data is substituted (no silent skips): the file layer only ever reuses a
/// successful fetch's data, never masks a failed fetch.
///
/// The snapshot pre-populates the per-account quota cache in `write_account_with_token`
/// so `clp .usage` hits fetch.rs's 30-second cache-first guard and skips the live
/// endpoint entirely — keeping total `/api/oauth/usage` calls to **~1 per run**.
fn live_quota_snapshot() -> &'static QuotaSnapshot
{
  static SNAPSHOT : std::sync::OnceLock< QuotaSnapshot > = std::sync::OnceLock::new();
  SNAPSHOT.get_or_init( ||
  {
    let cache = snapshot_cache_path();
    if snapshot_cache_fresh( &cache )
    {
      if let Some( snap ) = read_snapshot_cache( &cache )
      {
        return snap;
      }
    }
    let token = live_active_token().expect( "live_quota_snapshot: live API token required — no ~/.claude/.credentials.json" );
    let data  = claude_quota::fetch_oauth_usage( &token )
      .expect( "live_quota_snapshot: /api/oauth/usage unreachable — live API required for this test" );
    let snap = QuotaSnapshot
    {
      five_hour        : data.five_hour.map( |p| ( p.utilization, p.resets_at ) ),
      seven_day        : data.seven_day.map( |p| ( p.utilization, p.resets_at ) ),
      seven_day_sonnet : data.seven_day_sonnet.map( |p| ( p.utilization, p.resets_at ) ),
    };
    write_snapshot_cache( &cache, &snap );
    snap
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

/// Whether the account whose name starts with `account_prefix` clears both
/// rotation-eligibility quota floors right now, in the fixture HOME `home`.
///
/// For live-lane footer tests whose sole `Next`-eligible candidate is seeded by
/// `write_account_with_token` — its quota cache mirrors the host's LIVE snapshot,
/// so footer presence tracks the operator's real-time consumption. Callers branch:
/// floors clear → footer must recommend the candidate; floors not clear → footer
/// must be SUPPRESSED (BUG-292/BUG-324) — both live states assert the real contract.
///
/// Fix(audit-live-footer-fragile)
/// Root cause: it102/it103/it104 asserted the footer recommendation unconditionally,
///   but `find_first_eligible` (`sort_next.rs`) excludes h-exhausted (`5h ≤ 15%`) and
///   weekly-exhausted (`7d ≤ 3%`) candidates BY DESIGN (BUG-292/BUG-324) — and the
///   fixture caches mirror the host's LIVE account snapshot, so the tests' verdicts
///   tracked the operator's real-time quota consumption: green at 4% weekly left,
///   red at 3%, with no code change in between. Hoisted here from `usage_touch_test.rs`
///   when the same audit found `usage_core_test.rs` it011/it012 carrying the identical
///   unconditional assertions the original fix missed.
/// Pitfall: parse the floors from the rendered TSV cells, never re-derive them from
///   raw utilization — the displayed rounded value IS what eligibility compares
///   (round-before-compare doctrine, BUG-331/BUG-336); a private re-computation here
///   would reintroduce the exact drift audit-h-exhaustion-drift fixed in `sort_next.rs`.
///
/// # Panics
///
/// Panics if the `.usage` TSV probe fails, lacks the expected columns, has no row
/// for `account_prefix`, or a quota cell is unparseable — a broken probe must fail
/// loudly, never default to either branch.
#[ must_use ]
#[ inline ]
pub fn clears_rotation_floors( home : &str, account_prefix : &str ) -> bool
{
  use claude_profile::usage::test_bridge::types::{ H_EXHAUSTED_THRESHOLD, WEEKLY_EXHAUSTION_THRESHOLD };
  let tsv = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &tsv, 0 );
  let text = stdout( &tsv );
  let mut lines = text.lines();
  let header : Vec< &str > = lines.next().expect( "floors probe: TSV must have a header line" ).split( '\t' ).collect();
  let account_idx = header.iter().position( |h| *h == "account" ).expect( "floors probe: account column missing" );
  let h5_idx      = header.iter().position( |h| *h == "5h_left" ).expect( "floors probe: 5h_left column missing" );
  let d7_idx      = header.iter().position( |h| *h == "7d_left" ).expect( "floors probe: 7d_left column missing" );
  let row : Vec< &str > = lines
    .map( |l| l.split( '\t' ).collect::< Vec< _ > >() )
    .find( |cells| cells.get( account_idx ).is_some_and( |n| n.starts_with( account_prefix ) ) )
    .unwrap_or_else( || panic!( "floors probe: no TSV row for {account_prefix}" ) );
  let pct = | cell : &str | -> f64
  {
    // Fix(BUG-553): TSV quota cells gained the cache-staleness `~` prefix once BUG-553
    //   routed this surface through `quota_cells_for`; before that only the text table
    //   carried it, so this probe parsed a bare `NN%` and panicked on `~44%`.
    //   Root cause: the probe encoded "TSV percentages are bare" — an assumption that held
    //   only because TSV was missing a rule the text table already applied.
    //   Pitfall: never map a `~` cell to the 100.0 absent-data default — that would silently
    //   promote a cache-stale exhausted account into one that clears the floors. The strip runs
    //   ahead of the dash check for robustness only; `prefix_tilde` currently exempts a bare
    //   `—` (staleness qualifies a value, and `—` is the absence of one), so `~—` does not
    //   occur today — this ordering just means the probe would not care if that ever changed.
    let cell = cell.trim_start_matches( '~' );
    // `—` = absent window data on an Ok row → 100 in the canonical accessors
    // (five_hour_left/seven_day_left: absent data ≠ exhausted).
    if cell == "\u{2014}" { return 100.0; }
    cell.strip_suffix( '%' )
      .and_then( |n| n.parse::< f64 >().ok() )
      .unwrap_or_else( || panic!( "floors probe: unparseable quota cell {cell:?}" ) )
  };
  pct( row[ h5_idx ] ) > H_EXHAUSTED_THRESHOLD && pct( row[ d7_idx ] ) > WEEKLY_EXHAUSTION_THRESHOLD
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
