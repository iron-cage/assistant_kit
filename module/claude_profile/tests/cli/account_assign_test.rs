//! Integration tests: `assignee::` marker assign/unassign — Feature 065.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Scope
//!
//! All tests are fixture-based and run entirely offline. No network access required.
//! HOME isolation via `TempDir` + `USER=testuser` + `HOSTNAME=testmachine` ensures
//! deterministic marker filenames.
//!
//! Feature 065 renamed `active::` → `assignee::` and added `assignee::0` sentinel
//! (= current machine, expands to `$USER@$HOSTNAME`). `active::` is now a `REMOVED_TOGGLE`.
//! This file tests:
//!
//! - `assignee::USER@MACHINE name::X` — assign (write marker)
//! - `assignee::USER@MACHINE` (no name) — unassign (clear marker)
//! - `assignee::0 name::X` — sentinel: expand to `$USER@$HOSTNAME`, then assign
//! - `assignee::0` (no name) — sentinel: expand to `$USER@$HOSTNAME`, then unassign
//! - `REMOVED_TOGGLE`: `assign::1`, `for::`, `unclaim::1`, `active::` exit 1 with migration messages
//! - `assignee::` validation: USER@MACHINE format, empty components, sanitization
//! - `assignee::` isolation: does NOT modify `owner` field
//!
//! ## Test Matrix
//!
//! | ID | Test Function | FT/EC | Condition | P/N |
//! |----|---------------|-------|-----------|-----|
//! | FT-01 | `ft01_assignee_assign_writes_current_machine_marker` | FT-01 | `assignee::testuser@testmachine name::X` writes `DEFAULT_MARKER` | P |
//! | FT-01b | `ft01b_assignee_assign_writes_remote_marker` | FT-01 | `assignee::bob@laptop name::X` writes `_active_laptop_bob` | P |
//! | FT-02 | `ft02_assignee_unassign_clears_marker` | FT-03 | `assignee::user1@w003` (no name) clears marker | P |
//! | FT-03 | `ft03_assignee_assign_dry_run` | FT-05 | `assignee::testuser@testmachine name::X dry::1` → no write | P |
//! | FT-04 | `ft04_assignee_unknown_account_exits_1` | FT-08 | `assignee::testuser@testmachine name::ghost` → exit 1 | N |
//! | FT-05 | `ft05_assign_removed_toggle` | FT-10 | `assign::1 name::X` → exit 1 REMOVED_TOGGLE | N |
//! | FT-06 | `ft06_assign_and_for_removed_toggles` | FT-10 | `assign::1 for::bob@laptop name::X` → exit 1 | N |
//! | FT-07 | `ft07_unclaim_removed_toggle` | FT-10 | `unclaim::1 name::X` → exit 1 REMOVED_TOGGLE | N |
//! | FT-07b | `ft07b_assignee_unassign_dry_run` | FT-07 | `assignee::user1@w003 dry::1` (no name) → `[dry-run]` preview | P |
//! | FT-10 | `ft10_active_removed_toggle_migration_message` | FT-10 | `active::user1@w003 name::X` → exit 1 REMOVED_TOGGLE | N |
//! | FT-11 | `ft11_assignee_does_not_modify_owner` | FT-11 | `assignee::...` does NOT touch `owner` field | P |
//! | FT-11b | `ft11b_assignee_remote_does_not_modify_owner` | FT-11 | `assignee::bob@laptop name::X` does NOT touch `owner` field | P |
//! | FT-12a | `ft12a_space_in_assignee_value_sanitized` | FT-12 | `assignee::"alice@my laptop" name::X` → `_active_my_laptop_alice` | P |
//! | FT-12b | `ft12b_dot_hyphen_in_assignee_value_preserved` | FT-12 | `assignee::user1@w003.local name::X` → `_active_w003.local_user1` | P |
//! | FT-13 | `ft13_force_ignored_for_assignee` | FT-13 | `force::1 assignee::...` — force silently ignored; marker written | P |
//! | EC-2 | `ec2_assignee_zero_sentinel_assign` | EC-2 | `assignee::0 name::X` writes `DEFAULT_MARKER` via sentinel | P |
//! | EC-4 | `ec4_assignee_zero_sentinel_unassign` | EC-4 | `assignee::0` (no name) clears current machine marker | P |
//! | EC-5 | `ec5_assignee_badvalue_exits_1` | EC-5 | `assignee::badvalue name::X` exits 1 | N |
//! | EC-6 | `ec6_assignee_empty_user_exits_1` | EC-6 | `assignee::@testmachine name::X` exits 1 | N |
//! | EC-7 | `ec7_assignee_empty_machine_exits_1` | EC-7 | `assignee::testuser@ name::X` exits 1 | N |
//! | EC-8 | `ec8_multiple_at_splits_on_first` | EC-8 | `assignee::alice@corp.com@laptop` splits on first `@` | P |
//! | EC-9 | `ec9_assignee_zero_sentinel_dry_run_assign` | EC-9 | `assignee::0 name::X dry::1` → `[dry-run]` preview; no write | P |
//! | EC-10 | `ec10_assignee_zero_sentinel_dry_run_unassign` | EC-10 | `assignee::0 dry::1` (no name) → `[dry-run]` preview; no delete | P |
//! | EC-14 | `ec14_assignee_absent_no_marker_write` | EC-14 | no `assignee::` param → no marker file written | P |
//! | aa09 | `aa09_prefix_resolution` | — | `name::alice` prefix resolves to `alice@corp.com` | P |
//! | aa10 | `aa10_overwrite_existing_marker` | — | Second assign overwrites the marker | P |
//! | aa11 | `aa11_no_credentials_json_side_effect` | — | `assignee::` does NOT modify `~/.claude/.credentials.json` | P |
//! | aa12 | `aa12_dry_run_shows_marker_filename` | — | `dry::1` stdout names the target marker file | P |
//! | aa13 | `aa13_dry_run_unknown_account_exits_1` | — | `dry::1` + unknown account → exit 1 (existence check before dry-run) | N |
//! | aa14 | `aa14_unassign_absent_marker_is_noop` | — | unassign when marker absent → exit 0, stdout `unassigned` | P |
//! | aa15 | `aa15_ambiguous_prefix_exits_1` | — | `name::alice` when two alice-accounts → exit 1 ambiguous | N |
//! | aa16 | `aa16_exact_local_part_beats_prefix_ambiguity` | — | `name::i1` when `i1@host` + `i11@host` → resolves to `i1@host` | P |

use crate::cli_runner::{ run_cs_with_env, stdout, stderr, assert_exit, write_account, write_account_owner };
use tempfile::TempDir;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Fixed USER value for deterministic marker filenames.
const TEST_USER : &str = "testuser";

/// Fixed HOSTNAME value for deterministic marker filenames.
const TEST_HOST : &str = "testmachine";

/// Expected default marker when `assignee::testuser@testmachine` is used.
const DEFAULT_MARKER : &str = "_active_testmachine_testuser";

/// `assignee::` value targeting the test machine/user.
const ASSIGNEE_CURRENT : &str = "testuser@testmachine";

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
/// FT-01 (AC-01): `assignee::testuser@testmachine name::alice@corp.com` writes `DEFAULT_MARKER`.
fn ft01_assignee_assign_writes_current_machine_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "DEFAULT_MARKER must exist after assignee:: assign" );
  assert_eq!( content.trim(), "alice@corp.com", "marker must contain the assigned account name" );

  let out_text = stdout( &out );
  assert!( out_text.contains( "assigned" ), "stdout must confirm assignment: {out_text}" );
  assert!( out_text.contains( DEFAULT_MARKER ), "stdout must name the marker file: {out_text}" );
}

#[ test ]
/// FT-01 (remote machine): `assignee::bob@laptop name::alice@corp.com` writes `_active_laptop_bob`.
fn ft01b_assignee_assign_writes_remote_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::bob@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_laptop_bob" ) )
    .expect( "_active_laptop_bob must exist after assignee::bob@laptop assign" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── FT-02: unassign clears marker ─────────────────────────────────────────────

#[ test ]
/// FT-02 (AC-03): `assignee::user1@w003` (no `name::`) clears `_active_w003_user1`.
fn ft02_assignee_unassign_clears_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  let store  = credential_store( dir.path() );
  let marker = "_active_w003_user1";
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( marker ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "assignee::user1@w003" ], &refs );
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
/// FT-03 (AC-05): `dry::1` prints `[dry-run] would assign`; no marker written.
fn ft03_assignee_assign_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice@corp.com", "dry::1" ],
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
/// FT-04 (AC-08): unknown account → exit 1; no marker written.
fn ft04_assignee_unknown_account_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::ghost@example.com" ],
    &refs,
  );
  assert_exit( &out, 1 );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "no marker file must be written for unknown account" );
}

// ── FT-05..FT-07: REMOVED_TOGGLE migration messages ─────────────────────────

#[ test ]
/// FT-05 (AC-10): `assign::1 name::X` exits 1 with REMOVED migration message pointing to `assignee::`.
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
    err.contains( "REMOVED" ) && err.contains( "assignee::" ),
    "FT-05: stderr must contain REMOVED migration message pointing to assignee::; got:\n{err}",
  );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "REMOVED_TOGGLE must write no marker files" );
}

#[ test ]
/// FT-06 (AC-10): `assign::1 for::bob@laptop name::X` — both REMOVED; exits 1.
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
/// FT-07 (AC-10): `unclaim::1 name::X` exits 1 with REMOVED migration message.
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

// ── FT-07b: unassign dry-run ──────────────────────────────────────────────────

#[ test ]
/// FT-07b (AC-07): `assignee::user1@w003 dry::1` (no `name::`) prints `[dry-run]`; no file deleted.
fn ft07b_assignee_unassign_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  let store  = credential_store( dir.path() );
  let marker = "_active_w003_user1";
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( marker ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "assignee::user1@w003", "dry::1" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "[dry-run]" ) && out_text.contains( "unassign" ),
    "FT-07b: stdout must contain [dry-run] unassign preview: {out_text}",
  );

  let content = std::fs::read_to_string( store.join( marker ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com", "FT-07b: dry-run must NOT delete or clear the marker" );
}

// ── FT-10: active:: REMOVED_TOGGLE ───────────────────────────────────────────

#[ test ]
/// FT-10 (AC-10): `active::USER@MACHINE name::X` exits 1 — `active::` is a `REMOVED_TOGGLE` (Feature 065).
fn ft10_active_removed_toggle_migration_message()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "active::user1@w003", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ) && err.contains( "assignee::" ),
    "FT-10: stderr must contain REMOVED migration message pointing to assignee::; got:\n{err}",
  );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "REMOVED_TOGGLE must write no marker files" );
}

// ── FT-11: assignee:: does NOT modify owner ───────────────────────────────────

#[ test ]
/// FT-11 (AC-11): `assignee::testuser@testmachine name::X` does NOT modify `owner` field.
fn ft11_assignee_does_not_modify_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@machine" );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let owner = read_owner( dir.path(), "alice@corp.com" );
  assert_eq!(
    owner.as_deref(), Some( "other@machine" ),
    "FT-11: assignee:: must NOT modify owner; got: {owner:?}",
  );
}

#[ test ]
/// FT-11 (remote variant): `assignee::bob@laptop name::X` does NOT modify `owner` field.
fn ft11b_assignee_remote_does_not_modify_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@machine" );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::bob@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let owner = read_owner( dir.path(), "alice@corp.com" );
  assert_eq!(
    owner.as_deref(), Some( "other@machine" ),
    "FT-11b: assignee::bob@laptop must NOT modify owner; got: {owner:?}",
  );
}

// ── FT-12: sanitization ────────────────────────────────────────────────────────

#[ test ]
/// FT-12a (AC-12): space in machine component → sanitized to `_`.
fn ft12a_space_in_assignee_value_sanitized()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::alice@my laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_my_laptop_alice" ) )
    .expect( "_active_my_laptop_alice must exist — space in machine → '_'" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// FT-12b (AC-12): dot and hyphen in machine component preserved verbatim.
fn ft12b_dot_hyphen_in_assignee_value_preserved()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::user1@w003.local", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_w003.local_user1" ) )
    .expect( "_active_w003.local_user1 must exist — dot/hyphen preserved in sanitization" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── FT-13: force::1 silently ignored ─────────────────────────────────────────

#[ test ]
/// FT-13 (AC-13): `force::1 assignee::testuser@testmachine name::X` — `force::1` silently ignored;
/// marker is written normally (`assignee::` has no ownership gate).
fn ft13_force_ignored_for_assignee()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice@corp.com", "force::1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "DEFAULT_MARKER must exist — force::1 is silently ignored for assignee::" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── EC-2: sentinel assign ─────────────────────────────────────────────────────

#[ test ]
/// EC-2 (AC-02): `assignee::0 name::X` expands sentinel to `$USER@$HOSTNAME`; writes `DEFAULT_MARKER`.
fn ec2_assignee_zero_sentinel_assign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::0", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "DEFAULT_MARKER must exist — assignee::0 sentinel expands to testuser@testmachine" );
  assert_eq!( content.trim(), "alice@corp.com", "sentinel must write correct account name" );

  // Stdout must show the expanded identity, not the literal "0".
  let out_text = stdout( &out );
  assert!(
    out_text.contains( "testuser" ) && out_text.contains( "testmachine" ),
    "EC-2: stdout must show expanded identity (not literal '0'): {out_text}",
  );
}

// ── EC-4: sentinel unassign ───────────────────────────────────────────────────

#[ test ]
/// EC-4 (AC-04): `assignee::0` (no `name::`) expands sentinel; clears current machine marker.
fn ec4_assignee_zero_sentinel_unassign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( DEFAULT_MARKER ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "assignee::0" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "unassigned" ),
    "EC-4: stdout must confirm unassign: {out_text}",
  );

  let still_has_content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .is_ok_and( |s| !s.trim().is_empty() );
  assert!( !still_has_content, "EC-4: marker must be cleared or deleted after sentinel unassign" );
}

// ── EC-5..EC-7: assignee:: format validation ─────────────────────────────────

#[ test ]
/// EC-5 (AC-09): `assignee::badvalue` (no `@`, not `"0"`) exits 1 with USER@MACHINE format error.
fn ec5_assignee_badvalue_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::badvalue", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "USER@MACHINE" ) || stderr( &out ).contains( "'@'" ),
    "EC-5: stderr must explain USER@MACHINE format; got:\n{}", stderr( &out ),
  );
}

#[ test ]
/// EC-6 (AC-09): `assignee::@testmachine` (empty user component) exits 1.
fn ec6_assignee_empty_user_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::@testmachine", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "user" ) || stderr( &out ).contains( "empty" ),
    "EC-6: stderr must mention empty user component; got:\n{}", stderr( &out ),
  );
}

#[ test ]
/// EC-7 (AC-09): `assignee::testuser@` (empty machine component) exits 1.
fn ec7_assignee_empty_machine_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::testuser@", "name::alice@corp.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "machine" ) || stderr( &out ).contains( "empty" ),
    "EC-7: stderr must mention empty machine component; got:\n{}", stderr( &out ),
  );
}

// ── EC-8: multiple @ splits on first ─────────────────────────────────────────

#[ test ]
/// EC-8: `assignee::alice@corp.com@laptop` — split on first `@` → marker `_active_corp.com_laptop_alice`.
fn ec8_multiple_at_splits_on_first()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::alice@corp.com@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_corp.com_laptop_alice" ) )
    .expect( "_active_corp.com_laptop_alice must exist — split on first @ only" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── EC-9: sentinel dry-run assign ────────────────────────────────────────────

#[ test ]
/// EC-9 (AC-06): `assignee::0 name::X dry::1` expands sentinel; prints `[dry-run]`; no write.
fn ec9_assignee_zero_sentinel_dry_run_assign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::0", "name::alice@corp.com", "dry::1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "[dry-run] would assign" ),
    "EC-9: stdout must contain [dry-run] would assign: {out_text}",
  );
  // Sentinel must be expanded in the dry-run output (not literal "0").
  assert!(
    out_text.contains( "testuser" ) || out_text.contains( "testmachine" ),
    "EC-9: dry-run output must show expanded identity, not literal '0': {out_text}",
  );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "EC-9: dry-run must write no marker files" );
}

// ── EC-10: sentinel dry-run unassign ─────────────────────────────────────────

#[ test ]
/// EC-10 (AC-07): `assignee::0 dry::1` (no `name::`) expands sentinel; prints `[dry-run]`; no delete.
fn ec10_assignee_zero_sentinel_dry_run_unassign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( DEFAULT_MARKER ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "assignee::0", "dry::1" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "[dry-run]" ) && out_text.contains( "unassign" ),
    "EC-10: stdout must contain [dry-run] unassign preview: {out_text}",
  );

  // Marker must still exist and be unchanged.
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com", "EC-10: dry-run must NOT modify the marker" );
}

// ── EC-14: assignee:: absent → no marker ─────────────────────────────────────

#[ test ]
/// EC-14: no `assignee::` param → `.accounts` runs normally; no `_active_*` file written.
fn ec14_assignee_absent_no_marker_write()
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
  assert_eq!( active_marker_count( &store ), 0, "EC-14: no assignee:: → no marker file must be written" );
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
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// Second `assignee::bob@laptop` assign overwrites the existing `_active_laptop_bob` marker.
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
    &[ ".accounts", "assignee::bob@laptop", "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "_active_laptop_bob" ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// `assignee::` must not modify `~/.claude/.credentials.json`.
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
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice@corp.com" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!( after, before, "~/.claude/.credentials.json must be unchanged after assignee:: assign" );
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
    &[ ".accounts", "assignee::bob@laptop", "name::alice@corp.com", "dry::1" ],
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
/// Existence guard placed unconditionally before dry-run check in the `assignee::` dispatch.
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
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::ghost@example.com", "dry::1" ],
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
/// aa14: `assignee::USER@MACHINE` (no `name::`) when the marker file does not
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

  let out = run_cs_with_env( &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ) ], &refs );
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
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::alice" ],
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
    &[ ".accounts", &format!( "assignee::{ASSIGNEE_CURRENT}" ), "name::i1" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) ).unwrap();
  assert_eq!( content.trim(), "i1@host", "exact local-part must resolve to i1@host, not i11@host" );
}
