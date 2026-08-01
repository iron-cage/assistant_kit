//! Integration tests: TS (Token Status regression coverage, retargeted to `.credentials.status`), P (Paths).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### TS — Token Status (retargeted to `.credentials.status` after `.token.status` removal)
//!
//! ts01/02/03 (text valid/expiring/expired), ts08 (missing creds exit 2), and ts12 (HOME unset
//! exit 2) removed — fully superseded by `.credentials.status` coverage already present in
//! `user_story_test.rs` (UA-2), `command_noun_test.rs` (NC-3), and `cross_cutting_test.rs` (e02).
//! `it_trace_token_status_accepted` (EC-16) removed — superseded by `it_trace_credentials_status_accepted`
//! (EC-8) in `credentials_test_b.rs`. Remaining rows retarget to `.credentials.status` with vocabulary
//! and exit-code expectations corrected against verified source ground truth (`derive_token_state()` in
//! `src/commands/cmd_context.rs`, `credentials_status_routine()` in `src/commands/credentials.rs`):
//! JSON key is `token` (not `status`); a malformed-but-present credentials file degrades gracefully to
//! `Token: unknown` at exit 0 (only a missing file exits 2) — this is a genuine behavior delta from the
//! retired `.token.status`, which hard-failed at exit 2 for the same malformed-file case.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ts06 | `ts06_credentials_valid_json` | valid token, `format::json` → `"token":"valid"` | P |
//! | ts07 | `ts07_credentials_expired_json` | expired token, `format::json` → `"token":"expired"` | P |
//! | ts09 | `ts09_credentials_malformed_creds_shows_unknown` | malformed JSON, file present → `Token: unknown`, exit 0 | P |
//! | ts10 | `ts10_credentials_threshold_0_always_valid` | `threshold::0` + near-future → Valid | P |
//! | ts11 | `ts11_credentials_threshold_86400_expiring_soon` | `threshold::86400` + 2h expiry → `ExpiringSoon` | P |
//! | ts13 | `ts13_credentials_empty_creds_shows_unknown` | empty credentials file, file present → `Token: unknown`, exit 0 | P |
//! | ts14 | `ts14_credentials_expiring_soon_json` | near-expiry token, `format::json` → `"token":"expiring in...` | P |
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
//! | p09 | `p09_paths_field_returns_single_value` | field::credential_store → 1-line raw path; exit 0 | P |
//! | p10 | `p10_paths_field_unknown_exits_1` | field::nonexistent → exit 1; stderr names the bad field | N |

use crate::cli_runner::{
  run_cs_with_env, run_cs_without_home,
  stdout, stderr, assert_exit,
  write_credentials,
  FAR_FUTURE_MS, PAST_MS, near_future_ms,
};
use tempfile::TempDir;

// ── TS: Token Status (retargeted to `.credentials.status`) ─────────────────────

#[ test ]
fn ts06_credentials_valid_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"token\":\"valid\"" ), "JSON must contain token valid, got:\n{text}" );
  assert!( text.contains( "\"expires_in_secs\":" ), "JSON must contain expires_in_secs, got:\n{text}" );
}

#[ test ]
fn ts07_credentials_expired_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", PAST_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"token\":\"expired\"" ), "JSON must contain token expired, got:\n{text}" );
}

#[ test ]
// `.credentials.status` degrades gracefully for a malformed-but-present credentials file:
// `derive_token_state()` maps the Err from `status_with_threshold()` to `Token: unknown` at
// exit 0 — only a missing FILE (checked separately via `.exists()`) exits 2. This is a real
// behavior delta from the retired `.token.status`, which hard-failed at exit 2 here.
fn ts09_credentials_malformed_creds_shows_unknown()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( ".credentials.json" ), "{\"foo\":\"bar\"}" ).unwrap();

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Token:   unknown" ),
    "malformed-but-present credentials file must show Token: unknown, got:\n{text}",
  );
  assert!(
    text.contains( "Expires: (unavailable)" ),
    "malformed-but-present credentials file must show Expires: (unavailable), got:\n{text}",
  );
}

#[ test ]
fn ts10_credentials_threshold_0_always_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Token expiring in 30 minutes — normally "expiring soon" with default threshold
  write_credentials( dir.path(), "pro", "standard", near_future_ms() );

  let out = run_cs_with_env( &[ ".credentials.status", "threshold::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // status_with_threshold(0): remaining > 0 and remaining > threshold(0) → Valid
  assert!(
    text.contains( "Token:   valid" ),
    "threshold::0 with non-expired token should be valid, got:\n{text}",
  );
}

#[ test ]
fn ts11_credentials_threshold_86400_expiring_soon()
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

  let out = run_cs_with_env( &[ ".credentials.status", "threshold::86400" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Token:   expiring in" ),
    "2h remaining with 86400s threshold should be expiring_soon, got:\n{text}",
  );
}

#[ test ]
// `.credentials.status` degrades gracefully for a malformed-but-present credentials file — see
// ts09's comment. An empty file is likewise present-but-unparseable, not missing.
fn ts13_credentials_empty_creds_shows_unknown()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( ".credentials.json" ), "" ).unwrap();

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Token:   unknown" ),
    "empty-but-present credentials file must show Token: unknown, got:\n{text}",
  );
}

#[ test ]
fn ts14_credentials_expiring_soon_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", near_future_ms() );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"token\":\"expiring in" ), "JSON must show expiring in Xm, got:\n{text}" );
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

/// p09 — `field::credential_store` outputs exactly one raw path line with no label prefix.
///
/// # Root Cause
/// No `field::` parameter existed; scripts required `jq` to extract a single path value from
/// `clp .paths format::json` output, creating an external tool dependency.
///
/// # Why Not Caught
/// New feature, not a regression.
///
/// # Fix Applied
/// Added `field::` parameter with early-return in `paths_routine()` that outputs the named
/// field value followed by `\n`, bypassing format selection entirely.
///
/// # Prevention
/// p09 verifies single-line raw output; p10 verifies the unknown-field error path.
///
/// # Pitfall
/// `field::` names match JSON key form (underscore), not text-label form: use `session_env`
/// not `session-env`. An empty `field::` falls through to the full listing unchanged.
#[ test ]
fn p09_paths_field_returns_single_value()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths", "field::credential_store" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 1, "expected exactly 1 line, got: {lines:?}" );
  assert!( !lines[ 0 ].is_empty(), "expected non-empty path value" );
  assert!(
    !lines[ 0 ].contains( "credential_store:" ),
    "must not contain label prefix, got: {}",
    lines[ 0 ]
  );
}

/// p10 — unknown `field::` name exits 1 and names the bad field in stderr.
///
/// # Root Cause
/// No `field::` parameter existed; scripts required `jq` to extract a single path value from
/// `clp .paths format::json` output, creating an external tool dependency.
///
/// # Why Not Caught
/// New feature, not a regression.
///
/// # Fix Applied
/// Added `field::` parameter that returns `ErrorCode::ArgumentTypeMismatch` with a message
/// enumerating all 8 valid field names when an unknown name is supplied.
///
/// # Prevention
/// p10 locks in the exit-1 contract for unknown field names so callers get clear diagnostics.
///
/// # Pitfall
/// The error message must list all valid names — silent rejection would leave the caller with
/// no guidance on what values are accepted.
#[ test ]
fn p10_paths_field_unknown_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths", "field::nonexistent" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "nonexistent" ),
    "stderr must mention the unknown field name, got:\n{err}"
  );
}

// ── it_trace_token_status_accepted ────────────────────────────────────────────
//
// EC-16 removed along with `.token.status` — `it_trace_credentials_status_accepted` (EC-8) in
// credentials_test_b.rs already covers this exact scenario for the surviving `.credentials.status`.

// ── it_trace_paths_accepted ───────────────────────────────────────────────────

/// EC-17 (023): `trace::1` accepted by `.paths` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_paths_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".paths", "trace::1" ], &[ ( "HOME", home ) ] );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .paths, got stderr:\n{err}",
  );
  assert_exit( &out, 0 );
  assert!(
    err.contains( " · " ),
    "trace::1 must emit trace lines to stderr for .paths, got:\n{err}",
  );
}
