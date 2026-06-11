//! Integration tests: CLI command-noun contracts.
//!
//! Verifies lifecycle correctness, JSON output schema fidelity, and error code
//! contracts for the three CLI domain nouns: `account`, `token`, `credentials`.
//!
//! ## Test Matrix
//!
//! ### `noun::account` (NC-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | NC-1 | `account_nc1_full_lifecycle_roundtrip` | absent→saved→active→absent | P |
//! | NC-2 | `account_nc2_json_output_schema_valid` | `.accounts format::json` fields present | P |
//! | NC-3 | `account_nc3_error_codes_match_documented` | exit 1/2 per trigger | N |
//!
//! ### `noun::token` (NC-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | NC-1 | `token_nc1_status_is_stateless` | no files written after .token.status | P |
//! | NC-2 | `token_nc2_json_output_schema_valid` | `.token.status format::json` fields present | P |
//! | NC-3 | `token_nc3_missing_credentials_exits_2` | absent .credentials.json → exit 2 | N |
//!
//! ### `noun::credentials` (NC-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | NC-1 | `credentials_nc1_status_is_stateless` | no files written after .credentials.status | P |
//! | NC-2 | `credentials_nc2_json_output_schema_valid` | `.credentials.status format::json` fields present | P |
//! | NC-3 | `credentials_nc3_missing_credentials_exits_2` | absent .credentials.json → exit 2 | N |

use tempfile::TempDir;
use super::cli_runner::
{
  run_cs_with_env, assert_exit, stdout, stderr,
  write_credentials, write_account,
  FAR_FUTURE_MS,
};

/// Read the mtime of a path in milliseconds since epoch, or 0 if absent.
fn mtime_ms( path : &std::path::Path ) -> u64
{
  path.metadata().ok()
    .and_then( | m | m.modified().ok() )
    .and_then( | t | t.duration_since( std::time::UNIX_EPOCH ).ok() )
    .map_or( 0, | d | u64::try_from( d.as_millis() ).unwrap_or( 0 ) )
}

// ── noun::account ─────────────────────────────────────────────────────────────

// NC-1: Full lifecycle round-trip: absent → saved → active → absent
#[ test ]
fn account_nc1_full_lifecycle_roundtrip()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  // bob starts active so we can test alice going saved → active → saved
  write_account( dir.path(), "bob@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let alice_cred = cred_store.join( "alice@acme.com.credentials.json" );

  // Step 1: absent → saved
  let out1 = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );
  assert!( alice_cred.exists(), "step 1: alice credentials file must exist after save" );

  // Step 2: saved → active
  let out2 = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 0 );

  // Step 3: alice: active → saved; bob: saved → active
  let out3 = run_cs_with_env(
    &[ ".account.use", "name::bob@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out3, 0 );
  // alice file still present (still saved, just not active)
  assert!( alice_cred.exists(), "step 3: alice cred file must remain after switching to bob" );

  // Step 4: saved → absent
  let out4 = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out4, 0 );
  assert!( !alice_cred.exists(), "step 4: alice credentials file must be removed after delete" );
}

// NC-2: `.accounts format::json` output matches documented schema
#[ test ]
fn account_nc2_json_output_schema_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( &text )
    .expect( "`.accounts format::json` must produce valid JSON" );

  let array = parsed.as_array()
    .expect( "`.accounts format::json` must produce a JSON array" );
  assert!( !array.is_empty(), "array must contain at least one account" );

  let entry = &array[ 0 ];
  assert!( entry.get( "name" ).is_some(),              "each entry must have a 'name' field" );
  assert!( entry.get( "is_active" ).is_some(),         "each entry must have an 'is_active' field" );
  assert!( entry.get( "subscription_type" ).is_some(), "each entry must have a 'subscription_type' field" );
  assert!( entry.get( "rate_limit_tier" ).is_some(),   "each entry must have a 'rate_limit_tier' field" );
  assert!( entry.get( "expires_at_ms" ).is_some(),     "each entry must have an 'expires_at_ms' field" );
}

// NC-3: Documented error codes produced for documented trigger conditions
#[ test ]
fn account_nc3_error_codes_match_documented()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  // (a) Invalid name format (slash in local part is path-unsafe) → exit 1
  let out_a = run_cs_with_env(
    &[ ".account.use", "name::alice/work@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_a, 1 );

  // (b) Account not found → exit 2
  let out_b = run_cs_with_env(
    &[ ".account.use", "name::nobody@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_b, 2 );

  // (c) Missing required operation param → exit 1
  let out_c = run_cs_with_env(
    &[ ".account.renewal", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_c, 1 );
}

// ── noun::token ───────────────────────────────────────────────────────────────

// NC-1: Token status is stateless — no persistent state written
#[ test ]
fn token_nc1_status_is_stateless()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let creds_path = dir.path().join( ".claude" ).join( ".credentials.json" );
  let mtime_before = mtime_ms( &creds_path );

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  assert_eq!(
    mtime_before, mtime_ms( &creds_path ),
    ".token.status must not modify ~/.claude/.credentials.json",
  );

  // No new files in credential store
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let file_count = std::fs::read_dir( &store ).map( core::iter::Iterator::count ).unwrap_or( 0 );
  assert_eq!( file_count, 0, ".token.status must not create files in credential store" );
}

// NC-2: `.token.status format::json` output matches documented schema
#[ test ]
fn token_nc2_json_output_schema_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".token.status", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( &text )
    .expect( "`.token.status format::json` must produce valid JSON" );

  let obj = parsed.as_object()
    .expect( "`.token.status format::json` must produce a JSON object" );
  assert!( obj.contains_key( "status" ), "output must have 'status' field" );
  assert!( obj.contains_key( "expires_in_secs" ), "output must have 'expires_in_secs' field" );

  let status_val = obj[ "status" ].as_str().expect( "status field must be a string" );
  assert!(
    [ "valid", "expiring_soon", "expired" ].contains( &status_val ),
    "status must be one of: valid, expiring_soon, expired; got: {status_val}",
  );
}

// NC-3: Missing credentials file exits 2
#[ test ]
fn token_nc3_missing_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file
  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err_text = stderr( &out );
  assert!( !err_text.is_empty(), "error message must be non-empty on missing credentials" );
}

// ── noun::credentials ─────────────────────────────────────────────────────────

// NC-1: Credentials status is stateless — no persistent state written
#[ test ]
fn credentials_nc1_status_is_stateless()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let creds_path = dir.path().join( ".claude" ).join( ".credentials.json" );
  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let alice_file = cred_store.join( "alice@acme.com.credentials.json" );
  let mtime_live   = mtime_ms( &creds_path );
  let mtime_alice  = mtime_ms( &alice_file );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  assert_eq!(
    mtime_live, mtime_ms( &creds_path ),
    ".credentials.status must not modify ~/.claude/.credentials.json",
  );
  assert_eq!(
    mtime_alice, mtime_ms( &alice_file ),
    ".credentials.status must not modify stored account credential file",
  );
}

// NC-2: `.credentials.status format::json` output matches documented schema
#[ test ]
fn credentials_nc2_json_output_schema_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( &text )
    .expect( "`.credentials.status format::json` must produce valid JSON" );

  let obj = parsed.as_object()
    .expect( "`.credentials.status format::json` must produce a JSON object" );
  // Required fields per documented schema
  assert!( obj.contains_key( "subscription" ), "output must have 'subscription' field" );
  assert!( obj.contains_key( "tier" ),         "output must have 'tier' field" );
  assert!( obj.contains_key( "token" ),        "output must have 'token' field" );
  assert!( obj.contains_key( "expires_in_secs" ), "output must have 'expires_in_secs' field" );
  assert!( obj.contains_key( "file" ),         "output must have 'file' field" );
}

// NC-3: Missing `~/.claude/.credentials.json` exits 2
#[ test ]
fn credentials_nc3_missing_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file
  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}
