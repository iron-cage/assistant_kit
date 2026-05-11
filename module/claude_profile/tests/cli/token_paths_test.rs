//! Integration tests: TS (Token Status), P (Paths).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### TS — Token Status
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ts01 | `ts01_token_valid_text_v1` | valid token → "valid — Nm remaining" output | P |
//! | ts02 | `ts02_token_expiring_soon_text_v1` | near-expiry token → "expiring soon — Nm remaining" | P |
//! | ts03 | `ts03_token_expired_text_v1` | expired token → "expired" | P |
//! | ts06 | `ts06_token_valid_json` | valid token, `format::json` → JSON object | P |
//! | ts07 | `ts07_token_expired_json` | expired token, `format::json` → JSON with status | P |
//! | ts08 | `ts08_token_missing_creds_exits_2` | no credentials file → exit 2 | N |
//! | ts09 | `ts09_token_malformed_creds_exits_2` | malformed JSON → exit 2 | N |
//! | ts10 | `ts10_token_threshold_0_always_valid` | `threshold::0` + far-future → Valid | P |
//! | ts11 | `ts11_token_threshold_86400_expiring_soon` | `threshold::86400` + 1h expiry → `ExpiringSoon` | P |
//! | ts12 | `ts12_token_home_unset_exits_2` | HOME unset → exit 2 | N |
//! | ts13 | `ts13_token_empty_creds_exits_2` | empty credentials file → exit 2 | N |
//! | ts14 | `ts14_token_expiring_soon_json` | near-expiry token, `format::json` → JSON with status | P |
//!
//! ### P — Paths
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | p02 | `p02_paths_text_v1_labeled` | default → 7 labeled paths | P |
//! | p04 | `p04_paths_json` | format::json → JSON object | P |
//! | p05 | `p05_paths_home_unset_exits_2` | HOME unset → exit 2 | N |
//! | p06 | `p06_paths_contain_home_value` | HOME set → output contains HOME value | P |
//! | p07 | `p07_paths_home_with_spaces` | HOME path with spaces → works | P |
//! | p08 | `p08_paths_home_empty_exits_2` | HOME="" → exit 2 | N |

use crate::helpers::{
  run_cs_with_env, run_cs_without_home,
  stdout, assert_exit,
  write_credentials,
  FAR_FUTURE_MS, PAST_MS, near_future_ms,
};
use tempfile::TempDir;

// ── TS: Token Status ──────────────────────────────────────────────────────────

#[ test ]
fn ts01_token_valid_text_v1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( "valid" ), "far-future token must be valid, got:\n{text}" );
  assert!( text.contains( "remaining" ), "v::1 must show remaining time, got:\n{text}" );
}

#[ test ]
fn ts02_token_expiring_soon_text_v1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", near_future_ms() );

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( "expiring soon" ), "near-future token must be expiring_soon, got:\n{text}" );
}

#[ test ]
fn ts03_token_expired_text_v1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", PAST_MS );

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( "expired" ), "past token must be expired, got:\n{text}" );
}

#[ test ]
fn ts06_token_valid_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".token.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"status\":\"valid\"" ), "JSON must contain status valid, got:\n{text}" );
  assert!( text.contains( "\"expires_in_secs\":" ), "JSON must contain expires_in_secs, got:\n{text}" );
}

#[ test ]
fn ts07_token_expired_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", PAST_MS );

  let out = run_cs_with_env( &[ ".token.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"status\":\"expired\"" ), "JSON must contain status expired, got:\n{text}" );
}

#[ test ]
fn ts08_token_missing_creds_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ts09_token_malformed_creds_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( ".credentials.json" ), "{\"foo\":\"bar\"}" ).unwrap();

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ts10_token_threshold_0_always_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Token expiring in 30 minutes — normally "expiring soon" with default threshold
  write_credentials( dir.path(), "pro", "standard", near_future_ms() );

  let out = run_cs_with_env( &[ ".token.status", "threshold::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // With threshold 0, any non-expired token is "expiring soon" (0 < remaining)
  // Actually with threshold 0: remaining > 0 means valid, not expiring_soon
  // Wait — status_with_threshold(0): if remaining > 0 and remaining > threshold(0) → valid
  assert!( text.starts_with( "valid" ), "threshold::0 with non-expired token should be valid, got:\n{text}" );
}

#[ test ]
fn ts11_token_threshold_86400_expiring_soon()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Token expiring in ~265,000 years (FAR_FUTURE_MS)... that's way beyond 86400s
  // Use a token that expires in 2 hours — within 86400s threshold
  use std::time::{ SystemTime, UNIX_EPOCH };
  #[ allow( clippy::cast_possible_truncation ) ]
  let two_hours_ms = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap().as_millis() as u64
    + 2 * 3600 * 1000;
  write_credentials( dir.path(), "pro", "standard", two_hours_ms );

  let out = run_cs_with_env( &[ ".token.status", "threshold::86400" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( "expiring soon" ), "2h remaining with 86400s threshold should be expiring_soon, got:\n{text}" );
}

#[ test ]
fn ts12_token_home_unset_exits_2()
{
  let out = run_cs_without_home( &[ ".token.status" ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ts13_token_empty_creds_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( ".credentials.json" ), "" ).unwrap();

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ts14_token_expiring_soon_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", near_future_ms() );

  let out = run_cs_with_env( &[ ".token.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"status\":\"expiring_soon\"" ), "JSON must show expiring_soon, got:\n{text}" );
}

// ── P: Paths ──────────────────────────────────────────────────────────────────

#[ test ]
fn p02_paths_text_v1_labeled()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "credentials:" ), "v::1 must have credentials label, got:\n{text}" );
  assert!( text.contains( "credential_store:" ), "v::1 must have credential_store label, got:\n{text}" );
  assert!( text.contains( "sessions:" ), "v::1 must have sessions label, got:\n{text}" );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 7, "v::1 must have 7 labeled lines, got {}", lines.len() );
}

#[ test ]
fn p04_paths_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( '{' ), "JSON must start with '{{', got:\n{text}" );
  assert!( text.contains( "\"base\":" ), "JSON must contain base key, got:\n{text}" );
  assert!( text.contains( "\"credentials\":" ), "JSON must contain credentials key, got:\n{text}" );
  assert!( text.contains( "\"sessions\":" ), "JSON must contain sessions key, got:\n{text}" );
}

#[ test ]
fn p05_paths_home_unset_exits_2()
{
  let out = run_cs_without_home( &[ ".paths" ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn p06_paths_contain_home_value()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for line in text.lines()
  {
    assert!( line.contains( home ), "each line must contain HOME={home}, got: {line}" );
  }
}

#[ test ]
fn p07_paths_home_with_spaces()
{
  let dir = TempDir::new().unwrap();
  let space_path = dir.path().join( "path with spaces" );
  std::fs::create_dir_all( &space_path ).unwrap();
  let home = space_path.to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "path with spaces" ), "must handle spaces in path, got:\n{text}" );
}

#[ test ]
fn p08_paths_home_empty_exits_2()
{
  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}
