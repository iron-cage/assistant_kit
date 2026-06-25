//! Integration tests: `active::` marker assign/unassign — Feature 064.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Scope
//!
//! All tests are fixture-based and run entirely offline. No network access required.
//! HOME isolation via `TempDir` + `USER=testuser` + `HOSTNAME=testmachine` ensures
//! deterministic marker filenames.
//!
//! Feature 064 replaced `.accounts assign::1` + `for::` (Feature 032/037) with a single
//! `active::USER@MACHINE` param. `unclaim::1` replaced by `owner::0`. This file tests:
//!
//! - `active::USER@MACHINE name::X` — assign (write marker)
//! - `active::USER@MACHINE` (no name) — unassign (clear marker)
//! - `REMOVED_TOGGLE`: `assign::1`, `for::`, `unclaim::1` exit 1 with migration messages
//! - `active::` validation: USER@MACHINE format, empty components, sanitization
//! - `active::` isolation: does NOT modify `owner` field
//!
//! ## Test Matrix
//!
//! | ID | Test Function | FT/EC | Condition | P/N |
//! |----|---------------|-------|-----------|-----|
//! | FT-01 | `ft01_active_assign_writes_current_machine_marker` | FT-01 | `active::testuser@testmachine name::X` writes `DEFAULT_MARKER` | P |
//! | FT-01b | `ft01b_active_assign_writes_remote_marker` | FT-01 | `active::bob@laptop name::X` writes `_active_laptop_bob` | P |
//! | FT-02 | `ft02_active_unassign_clears_marker` | FT-02 | `active::testuser@testmachine` (no name) clears marker | P |
//! | FT-03 | `ft03_active_assign_dry_run` | FT-03 | `active::testuser@testmachine name::X dry::1` → no write | P |
//! | FT-04 | `ft04_active_unknown_account_exits_1` | FT-04 | `active::testuser@testmachine name::ghost` → exit 1 | N |
//! | FT-05 | `ft05_assign_removed_toggle` | FT-05 | `assign::1 name::X` → exit 1 REMOVED_TOGGLE | N |
//! | FT-06 | `ft06_assign_and_for_removed_toggles` | FT-06 | `assign::1 for::bob@laptop name::X` → exit 1 | N |
//! | FT-07 | `ft07_unclaim_removed_toggle` | FT-07 | `unclaim::1 name::X` → exit 1 REMOVED_TOGGLE | N |
//! | FT-13a | `ft13a_space_in_active_value_sanitized` | FT-13 | `active::"alice@my laptop" name::X` → `_active_my_laptop_alice` | P |
//! | FT-13b | `ft13b_dot_hyphen_in_active_value_preserved` | FT-13 | `active::user1@w003.local name::X` → `_active_w003.local_user1` | P |
//! | FT-14 | `ft14_active_does_not_modify_owner` | FT-14 | `active::...` does NOT touch `owner` field | P |
//! | FT-18 | `ft18_active_zero_rejected` | FT-18 | `active::0 name::X` exits 1 (no `@`) | N |
//! | FT-19 | `ft19_active_unassign_dry_run` | FT-19 | `active::user1@w003 dry::1` (no name) → `[dry-run]` preview | P |
//! | EC-3 | `ec3_active_no_at_exits_1` | EC-3 | `active::badvalue name::X` exits 1 | N |
//! | EC-4 | `ec4_active_empty_user_exits_1` | EC-4 | `active::@testmachine name::X` exits 1 | N |
//! | EC-5 | `ec5_active_empty_machine_exits_1` | EC-5 | `active::testuser@ name::X` exits 1 | N |
//! | EC-8 | `ec8_multiple_at_splits_on_first` | EC-8/FT-13 | `active::alice@corp.com@laptop` splits on first `@` | P |
//! | EC-10 | `ec10_active_absent_no_marker_write` | EC-10 | no `active::` param → no marker file written | P |
//! | EC-13 | `ec13_force_ignored_for_active` | EC-13 | `force::1 active::...` — force silently ignored; marker written | P |
//! | aa09 | `aa09_prefix_resolution` | — | `name::alice` prefix resolves to `alice@corp.com` | P |
//! | aa10 | `aa10_overwrite_existing_marker` | — | Second assign overwrites the marker | P |
//! | aa11 | `aa11_no_credentials_json_side_effect` | — | `active::` does NOT modify `~/.claude/.credentials.json` | P |
//! | aa12 | `aa12_dry_run_shows_marker_filename` | — | `dry::1` stdout names the target marker file | P |
//! | aa13 | `aa13_dry_run_unknown_account_exits_1` | — | `dry::1` + unknown account → exit 1 (existence check before dry-run) | N |
//! | aa15 | `aa15_ambiguous_prefix_exits_1` | — | `name::alice` when two alice-accounts → exit 1 ambiguous | N |
//! | aa16 | `aa16_exact_local_part_beats_prefix_ambiguity` | — | `name::i1` when `i1@host` + `i11@host` → resolves to `i1@host` | P |
//! | ft14b | `ft14b_active_for_does_not_modify_owner` | FT-14 | `active::bob@laptop name::X` does NOT touch `owner` field | P |
//! | aa14 | `aa14_unassign_absent_marker_is_noop` | — | unassign when marker absent → exit 0, stdout `unassigned` | P |

use crate::cli_runner::{ run_cs_with_env, stdout, stderr, assert_exit, write_account, write_account_owner };
use tempfile::TempDir;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Fixed USER value for deterministic marker filenames.
const TEST_USER : &str = "testuser";

/// Fixed HOSTNAME value for deterministic marker filenames.
const TEST_HOST : &str = "testmachine";

/// Expected default marker when `active::testuser@testmachine` is used.
const DEFAULT_MARKER : &str = "_active_testmachine_testuser";

/// `active::` value targeting the test machine/user.
const ACTIVE_CURRENT : &str = "testuser@testmachine";

/// Standard env block: HOME, USER, HOSTNAME for deterministic behavior.
fn test_env( home : &str ) -> Vec< ( &str, &str ) >
{
  vec![ ( "HOME", home ), ( "USER", TEST_USER ), ( "HOSTNAME", TEST_HOST ) ]
}

/// Credential store path for a given home directory.
fn credential_store( home : &std::path::Path ) -> std::path::PathBuf
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
}

/// Count `_active*` files in the credential store.
fn active_marker_count( store : &std::path::Path ) -> usize
{
  std::fs::read_dir( store )
    .map_or( 0, | entries | entries
      .filter_map( core::result::Result::ok )
      .filter( | e | e.file_name().to_string_lossy().starts_with( "_active" ) )
      .count()
    )
}

/// Read `owner` field from `{name}.json` in the credential store.
fn read_owner( home : &std::path::Path, name : &str ) -> Option< String >
{
  let meta = credential_store( home ).join( format!( "{name}.json" ) );
  std::fs::read_to_string( &meta )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .and_then( |v| v[ "owner" ].as_str().map( str::to_string ) )
}

// ── FT-01: assign writes marker ───────────────────────────────────────────────

#[ test ]
/// FT-01 (AC-01): `active::testuser@testmachine name::alice@corp.com` writes `DEFAULT_MARKER`.
fn ft01_active_assign_writes_current_machine_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "DEFAULT_MARKER must exist after active:: assign" );
  assert_eq!( content.trim(), "alice@corp.com", "marker must contain the assigned account name" );

  let out_text = stdout( &out );
  assert!( out_text.contains( "assigned" ), "stdout must confirm assignment: {out_text}" );
  assert!( out_text.contains( DEFAULT_MARKER ), "stdout must name the marker file: {out_text}" );
}

#[ test ]
/// FT-01 (remote machine): `active::bob@laptop name::alice@corp.com` writes `_active_laptop_bob`.
fn ft01b_active_assign_writes_remote_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::bob@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_laptop_bob" ) )
    .expect( "_active_laptop_bob must exist after active::bob@laptop assign" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── FT-02: unassign clears marker ─────────────────────────────────────────────

#[ test ]
/// FT-02 (AC-02): `active::user1@w003` (no `name::`) clears `_active_w003_user1`.
fn ft02_active_unassign_clears_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  let store  = credential_store( dir.path() );
  let marker = "_active_w003_user1";
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( marker ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "active::user1@w003" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "unassigned" ) && out_text.contains( marker ),
    "stdout must confirm unassign with marker name: {out_text}",
  );

  let marker_path = store.join( marker );
  let still_has_content = std::fs::read_to_string( &marker_path )
    .is_ok_and( |s| !s.trim().is_empty() );
  assert!( !still_has_content, "marker must be cleared or deleted after unassign" );
}

// ── FT-03: assign dry-run ─────────────────────────────────────────────────────

#[ test ]
/// FT-03 (AC-03): `dry::1` prints `[dry-run] would assign`; no marker written.
fn ft03_active_assign_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice@corp.com", "dry::1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "[dry-run] would assign" ), "stdout must contain dry-run tag: {out_text}" );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "dry-run must write no marker files" );
}

// ── FT-04: unknown account exits 1 ───────────────────────────────────────────

#[ test ]
/// FT-04 (AC-04): unknown account → exit 1; no marker written.
fn ft04_active_unknown_account_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::ghost@example.com" ],
    &refs,
  );
  assert_exit( &out, 1 );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "no marker file must be written for unknown account" );
}

// ── FT-05..FT-07: REMOVED_TOGGLE migration messages ─────────────────────────

#[ test ]
/// FT-05 (AC-05): `assign::1 name::X` exits 1 with REMOVED migration message.
fn ft05_assign_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assign::1", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ) && err.contains( "active::" ),
    "FT-05: stderr must contain REMOVED migration message pointing to active::; got:\n{err}",
  );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "REMOVED_TOGGLE must write no marker files" );
}

#[ test ]
/// FT-06 (AC-06): `assign::1 for::bob@laptop name::X` — both REMOVED; exits 1.
fn ft06_assign_and_for_removed_toggles()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assign::1", "name::alice@corp.com", "for::bob@laptop" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "REMOVED" ), "FT-06: must emit REMOVED migration; got:\n{}", stderr( &out ) );
}

#[ test ]
/// FT-07 (AC-07): `unclaim::1 name::X` exits 1 with REMOVED migration message.
fn ft07_unclaim_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "unclaim::1", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ) && err.contains( "owner::0" ),
    "FT-07: stderr must contain REMOVED migration pointing to owner::0; got:\n{err}",
  );
}

// ── FT-13: sanitization ────────────────────────────────────────────────────────

#[ test ]
/// FT-13a (AC-13): space in machine component → sanitized to `_`.
fn ft13a_space_in_active_value_sanitized()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::alice@my laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_my_laptop_alice" ) )
    .expect( "_active_my_laptop_alice must exist — space in machine → '_'" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// FT-13b (AC-13): dot and hyphen in machine component preserved verbatim.
fn ft13b_dot_hyphen_in_active_value_preserved()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::user1@w003.local", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_w003.local_user1" ) )
    .expect( "_active_w003.local_user1 must exist — dot/hyphen preserved in sanitization" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── FT-14: active:: does NOT modify owner ────────────────────────────────────

#[ test ]
/// FT-14 (AC-14): `active::testuser@testmachine name::X` does NOT modify `owner` field.
fn ft14_active_does_not_modify_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@machine" );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let owner = read_owner( dir.path(), "alice@corp.com" );
  assert_eq!(
    owner.as_deref(), Some( "other@machine" ),
    "FT-14: active:: must NOT modify owner; got: {owner:?}",
  );
}

#[ test ]
/// FT-14 (remote variant): `active::bob@laptop name::X` does NOT modify `owner` field.
fn ft14b_active_for_does_not_modify_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@machine" );

  let out = run_cs_with_env(
    &[ ".accounts", "active::bob@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let owner = read_owner( dir.path(), "alice@corp.com" );
  assert_eq!(
    owner.as_deref(), Some( "other@machine" ),
    "FT-14b: active::bob@laptop must NOT modify owner; got: {owner:?}",
  );
}

// ── FT-18: active::0 rejected ─────────────────────────────────────────────────

#[ test ]
/// FT-18 (AC-18): `active::0 name::X` exits 1 — value `"0"` contains no `@`.
fn ft18_active_zero_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::0", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "USER@MACHINE" ) || err.contains( "'@'" ),
    "FT-18: stderr must explain USER@MACHINE format requirement; got:\n{err}",
  );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "active::0 must write no marker file" );
}

// ── FT-19: unassign dry-run ───────────────────────────────────────────────────

#[ test ]
/// FT-19 (AC-19): `active::user1@w003 dry::1` (no `name::`) prints `[dry-run]`; no file deleted.
fn ft19_active_unassign_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  let store  = credential_store( dir.path() );
  let marker = "_active_w003_user1";
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( marker ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "active::user1@w003", "dry::1" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "[dry-run]" ) && out_text.contains( "unassign" ),
    "FT-19: stdout must contain [dry-run] unassign preview: {out_text}",
  );

  let content = std::fs::read_to_string( store.join( marker ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com", "FT-19: dry-run must NOT delete or clear the marker" );
}

// ── EC-3..EC-5: active:: format validation ────────────────────────────────────

#[ test ]
/// EC-3: `active::badvalue` (no `@`) exits 1 with USER@MACHINE format error.
fn ec3_active_no_at_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::badvalue", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "USER@MACHINE" ) || stderr( &out ).contains( "'@'" ),
    "EC-3: stderr must explain USER@MACHINE format; got:\n{}", stderr( &out ),
  );
}

#[ test ]
/// EC-4: `active::@testmachine` (empty user component) exits 1.
fn ec4_active_empty_user_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::@testmachine", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "user" ) || stderr( &out ).contains( "empty" ),
    "EC-4: stderr must mention empty user component; got:\n{}", stderr( &out ),
  );
}

#[ test ]
/// EC-5: `active::testuser@` (empty machine component) exits 1.
fn ec5_active_empty_machine_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::testuser@", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "machine" ) || stderr( &out ).contains( "empty" ),
    "EC-5: stderr must mention empty machine component; got:\n{}", stderr( &out ),
  );
}

// ── EC-8: multiple @ splits on first ─────────────────────────────────────────

#[ test ]
/// EC-8: `active::alice@corp.com@laptop` — split on first `@` → marker `_active_corp.com_laptop_alice`.
fn ec8_multiple_at_splits_on_first()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::alice@corp.com@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_corp.com_laptop_alice" ) )
    .expect( "_active_corp.com_laptop_alice must exist — split on first @ only" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── EC-10: active:: absent → no marker ────────────────────────────────────────

#[ test ]
/// EC-10: no `active::` param → `.accounts` runs normally; no `_active_*` file written.
fn ec10_active_absent_no_marker_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "EC-10: no active:: → no marker file must be written" );
}

// ── EC-13: force::1 silently ignored for active:: ────────────────────────────

#[ test ]
/// EC-13: `force::1 active::testuser@testmachine name::X` — `force::1` silently ignored;
/// marker is written normally (`active::` has no ownership gate).
fn ec13_force_ignored_for_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice@corp.com", "force::1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "DEFAULT_MARKER must exist — force::1 is silently ignored for active::" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── Regression / preserved tests ─────────────────────────────────────────────

#[ test ]
/// Prefix `alice` resolves to `alice@corp.com` (one unambiguous match).
fn aa09_prefix_resolution()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// Second `active::bob@laptop` assign overwrites the existing `_active_laptop_bob` marker.
fn aa10_overwrite_existing_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  write_account( dir.path(), "bob@corp.com",   "max", "tier4", 9_999_999_999_999, false );

  let store = credential_store( dir.path() );
  std::fs::write( store.join( "_active_laptop_bob" ), "old@account.com" ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "active::bob@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "_active_laptop_bob" ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// `active::` must not modify `~/.claude/.credentials.json`.
fn aa11_no_credentials_json_side_effect()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let creds_path = claude_dir.join( ".credentials.json" );
  let before     = r#"{"sentinel":"must-not-change"}"#;
  std::fs::write( &creds_path, before ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!( after, before, "~/.claude/.credentials.json must be unchanged after active:: assign" );
}

#[ test ]
/// `dry::1` stdout names the target marker file (`_active_laptop_bob`).
fn aa12_dry_run_shows_marker_filename()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::bob@laptop", "name::alice@corp.com", "dry::1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "_active_laptop_bob" ),
    "dry-run stdout must name the target marker file: {out_text}",
  );
}

#[ test ]
/// `dry::1` + unknown account → exit 1 (existence validated before dry-run).
///
/// ## Root Cause (BUG-247)
///
/// `resolve_account_name` short-circuits on `@`; the cred-file existence check
/// must run before the dry-run branch — not after.
///
/// ## Fix Applied
///
/// Existence guard placed unconditionally before dry-run check in the `active::` dispatch.
///
/// ## Pitfall
///
/// `@`-form names bypass prefix resolution but NOT existence validation; callers
/// must check existence after resolution.
fn aa13_dry_run_unknown_account_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::ghost@example.com", "dry::1" ],
    &refs,
  );
  assert_exit( &out, 1 );
  let out_text = stdout( &out );
  assert!(
    !out_text.contains( "[dry-run] would assign" ),
    "dry-run output must not be emitted for a non-existent account: {out_text}",
  );
}

#[ test ]
/// Ambiguous prefix `alice` (two alice-accounts) → exit 1 (not 2 — argument error, not missing).
fn aa15_ambiguous_prefix_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com",  "max", "tier4", 9_999_999_999_999, false );
  write_account( dir.path(), "alice@other.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::alice" ],
    &refs,
  );
  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "ambiguous" ), "stderr must explain ambiguity: {}", stderr( &out ) );
}

#[ test ]
/// `name::i1` when both `i1@host` and `i11@host` exist → exact local-part match wins → `i1@host`.
fn aa16_exact_local_part_beats_prefix_ambiguity()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "i1@host",  "max", "tier4", 9_999_999_999_999, false );
  write_account( dir.path(), "i11@host", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ), "name::i1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) ).unwrap();
  assert_eq!( content.trim(), "i1@host", "exact local-part must resolve to i1@host, not i11@host" );
}

#[ test ]
/// aa14: `active::USER@MACHINE` (no `name::`) when the marker file does not
/// exist — treated as a no-op; exits 0; stdout confirms `unassigned`.
///
/// The unassign path skips `remove_file` when the marker is absent and still
/// emits the standard confirmation message.
fn aa14_unassign_absent_marker_is_noop()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  // Credential store exists but contains NO marker files.
  std::fs::create_dir_all( credential_store( dir.path() ) ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", &format!( "active::{ACTIVE_CURRENT}" ) ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "unassigned" ),
    "aa14: stdout must contain 'unassigned' even when marker was absent; got:\n{out_text}",
  );
  // Marker must still be absent (not created).
  assert!(
    !credential_store( dir.path() ).join( DEFAULT_MARKER ).exists(),
    "aa14: unassign of absent marker must not create the marker file",
  );
}
