//! Integration tests: AA (Account Assign) — `.account.assign` command.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Scope
//!
//! All tests (aa01–aa12) are fixture-based and run entirely offline.
//! No network access required. HOME isolation via `TempDir` + `USER=testuser`
//! + `HOSTNAME=testmachine` ensures deterministic marker filenames.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | FT | Condition | P/N |
//! |----|---------------|----|-----------|-----|
//! | aa01 | `aa01_current_machine_marker_written` | FT-01 | No `for::` → `_active_testmachine_testuser` created = account name | P |
//! | aa02 | `aa02_remote_machine_marker_written` | FT-02 | `for::bob@laptop` → `_active_laptop_bob` created = account name | P |
//! | aa03 | `aa03_dry_run_no_write` | FT-03 | `dry::1` → no `_active_*` file; stdout contains `[dry-run] would assign` | P |
//! | aa04 | `aa04_no_name_emits_usage_block` | FT-04 | No `name::` (active account set) → preamble + machine + active account name + `Ready to copy:` | P |
//! | aa05 | `aa05_unknown_account_exits_2` | FT-05 | Unknown account name (`@`-form) → exit 2 | N |
//! | aa06 | `aa06_for_without_at_exits_1` | FT-06 | `for::badvalue` (no `@`) → exit 1 | N |
//! | aa07 | `aa07_empty_for_component_exits_1` | FT-07 | `for::@laptop` or `for::bob@` → exit 1 | N |
//! | aa08 | `aa08_special_chars_sanitized` | FT-08 | `for::alice@my laptop` → `_active_my_laptop_alice` (space → `_`) | P |
//! | aa09 | `aa09_prefix_resolution` | FT-09 | `name::alice` prefix resolves to `alice@corp.com` | P |
//! | aa10 | `aa10_overwrite_existing_marker` | FT-10 | Overwrites existing `_active_laptop_bob` with new account name | P |
//! | aa11 | `aa11_no_credentials_json_side_effect` | FT-11 | `~/.claude/.credentials.json` content unchanged after assign | P |
//! | aa12 | `aa12_dry_run_shows_marker_filename` | FT-12 | `dry::1` + `for::bob@laptop` → stdout contains `_active_laptop_bob` | P |
//! | aa13 | `aa13_dry_run_unknown_account_exits_2` | FT-05 | `dry::1` + unknown account → exit 2 (existence validated before dry-run) | N |
//! | aa14 | `aa14_usage_block_no_active_marker_shows_none` | FT-04 | No `name::`, no marker file → `Active account: (none)`, no `Ready to copy:` | P |
//! | aa15 | `aa15_ambiguous_prefix_exits_1` | FT-05 | `name::alice` matches two accounts → exit 1 (ambiguous, not exit 2) | N |
//! | aa16 | `aa16_exact_local_part_beats_prefix_ambiguity` | FT-09 | `name::i1` when `i1@host` + `i11@host` exist → resolves to `i1@host` (exact wins) | P |
//! | aa17 | `aa17_for_only_at_both_empty_exits_1` | FT-06 | `for::@` (only `@`, both components empty) → exit 1 | N |
//! | ec7  | `ec7_dot_hyphen_in_machine_preserved` | EC-7 | `for::user1@w003.local` → `_active_w003.local_user1` (dot + hyphen kept) | P |
//! | ec8  | `ec8_multiple_at_splits_on_first` | EC-8 | `for::alice@corp.com@laptop` → split on first `@` → `_active_corp.com_laptop_alice` | P |

use crate::cli_runner::{ run_cs_with_env, stdout, stderr, assert_exit, write_account };
use tempfile::TempDir;

// ── Constants ─────────────────────────────────────────────────────────────────

/// Fixed USER value used throughout these tests for deterministic marker filenames.
const TEST_USER : &str = "testuser";

/// Fixed HOSTNAME value used throughout these tests for deterministic marker filenames.
const TEST_HOST : &str = "testmachine";

/// Expected default marker when no `for::` is provided (matches `_active_testmachine_testuser`).
const DEFAULT_MARKER : &str = "_active_testmachine_testuser";

/// Standard env block that sets HOME, USER, and HOSTNAME for deterministic behavior.
fn test_env( home : &str ) -> Vec< ( &str, &str ) >
{
  vec![ ( "HOME", home ), ( "USER", TEST_USER ), ( "HOSTNAME", TEST_HOST ) ]
}

/// Resolve the credential store path for a given home directory.
fn credential_store( home : &std::path::Path ) -> std::path::PathBuf
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
}

/// Count files in the credential store whose names start with `_active`.
fn active_marker_count( store : &std::path::Path ) -> usize
{
  std::fs::read_dir( store )
    .map_or( 0, | entries | entries
      .filter_map( core::result::Result::ok )
      .filter( | e | e.file_name().to_string_lossy().starts_with( "_active" ) )
      .count()
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ test ]
/// AC-01: default (no `for::`) writes `_active_{machine}_{user}` for current machine.
fn aa01_current_machine_marker_written()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com" ], &refs );
  assert_exit( &out, 0 );

  let store    = credential_store( dir.path() );
  let content  = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "default marker must exist after assign" );
  assert_eq!( content.trim(), "alice@corp.com", "marker must contain the assigned account name" );

  let out_text = stdout( &out );
  assert!( out_text.contains( "Assigned" ), "stdout must confirm assignment: {out_text}" );
  assert!( out_text.contains( DEFAULT_MARKER ), "stdout must name the marker file: {out_text}" );
}

#[ test ]
/// AC-02: `for::bob@laptop` writes `_active_laptop_bob` (machine first, then user).
fn aa02_remote_machine_marker_written()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::bob@laptop" ], &refs );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_laptop_bob" ) )
    .expect( "_active_laptop_bob must exist after assign with for::bob@laptop" );
  assert_eq!( content.trim(), "alice@corp.com", "marker must contain the assigned account name" );
}

#[ test ]
/// AC-03: `dry::1` prints `[dry-run] would assign` and writes no marker file.
fn aa03_dry_run_no_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "dry::1" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "[dry-run] would assign" ), "stdout must contain dry-run tag: {out_text}" );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "dry-run must write no marker files" );
}

#[ test ]
/// AC-04: no `name::` with active account set → emits preamble + live usage block with machine identity,
/// active account name, and copy-paste ready examples containing that name.
///
/// ## Why pre-seeding the active marker is required
///
/// The usage block conditionally shows a `Ready to copy:` section only when `active != "(none)"`.
/// Without pre-seeding `DEFAULT_MARKER`, the command reads no marker file and shows `(none)`,
/// which omits `Ready to copy:` entirely — the primary AC-04 success path (copy-paste examples
/// with the real account name substituted) would never be exercised. Pre-seeding ensures the
/// block shows the actual account name and the `Ready to copy:` section is present.
fn aa04_no_name_emits_usage_block()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  // Write account and pre-seed the active marker so the usage block shows the real account name.
  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  let store = credential_store( dir.path() );
  std::fs::write( store.join( DEFAULT_MARKER ), "alice@corp.com" ).unwrap();

  let out = run_cs_with_env( &[ ".account.assign" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "Current machine:" ), "stdout must show current machine: {out_text}" );
  assert!( out_text.contains( "Active account:" ), "stdout must show active account: {out_text}" );
  assert!( out_text.contains( "alice@corp.com" ), "stdout must show active account name in examples: {out_text}" );
  assert!( out_text.contains( "Ready to copy:" ), "stdout must show ready-to-copy block: {out_text}" );
}

#[ test ]
/// AC-05: unknown account name → exit 2 (not found).
fn aa05_unknown_account_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::nobody@example.com" ], &refs );
  assert_exit( &out, 2 );
}

#[ test ]
/// AC-06: `for::` without `@` → exit 1 with error about USER@MACHINE format.
fn aa06_for_without_at_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::badvalue" ], &refs );
  assert_exit( &out, 1 );

  let err_text = stderr( &out );
  assert!( err_text.contains( "USER@MACHINE" ) || err_text.contains( '@' ),
    "stderr must explain the required format: {err_text}" );
}

#[ test ]
/// AC-07: empty user or machine component in `for::` → exit 1.
fn aa07_empty_for_component_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  // Empty user component (left of @).
  let out_a = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::@laptop" ], &refs );
  assert_exit( &out_a, 1 );

  // Empty machine component (right of @).
  let out_b = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::bob@" ], &refs );
  assert_exit( &out_b, 1 );
}

#[ test ]
/// AC-08: space in machine component is sanitized to `_`; marker filename uses sanitized form.
fn aa08_special_chars_sanitized()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  // "alice@my laptop" — space in machine component must sanitize to `_`.
  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::alice@my laptop" ], &refs );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_my_laptop_alice" ) )
    .expect( "_active_my_laptop_alice must exist after assign with space in machine component" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// AC-09: bare prefix `alice` resolves to `alice@corp.com` when that is the only match.
fn aa09_prefix_resolution()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice" ], &refs );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "default marker must exist after assign with prefix name" );
  assert_eq!( content.trim(), "alice@corp.com", "resolved name must be written to marker" );
}

#[ test ]
/// AC-10: second assign to the same marker path overwrites the previous account name.
fn aa10_overwrite_existing_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );
  write_account( dir.path(), "bob@corp.com", "max", "tier4", 9_999_999_999_999, false );

  // Write initial marker.
  let store = credential_store( dir.path() );
  std::fs::write( store.join( "_active_laptop_bob" ), "old@account.com" ).unwrap();

  // Overwrite with alice@corp.com.
  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::bob@laptop" ], &refs );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "_active_laptop_bob" ) ).unwrap();
  assert_eq!( content.trim(), "alice@corp.com", "marker must be overwritten with new account" );
}

#[ test ]
/// AC-11: `.account.assign` must not modify `~/.claude/.credentials.json`.
fn aa11_no_credentials_json_side_effect()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  // Write a sentinel .credentials.json and capture its content.
  let claude_dir  = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let creds_path  = claude_dir.join( ".credentials.json" );
  let before      = r#"{"sentinel":"must-not-change"}"#;
  std::fs::write( &creds_path, before ).unwrap();

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com" ], &refs );
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!( after, before, "~/.claude/.credentials.json must be unchanged after .account.assign" );
}

// ── AA13 — Dry-run with unknown @-account exits 2 ─────────────────────────────

#[ test ]
/// AC-05 / BUG-247: `dry::1` with an unknown @-containing account name must still exit 2.
///
/// ## Root Cause (BUG-247)
///
/// `resolve_account_name` short-circuits on `@` and returns the raw name without
/// validating it against the credential store. The existence check was not performed
/// before the dry-run branch, so `dry::1` silently succeeded (exit 0) even when the
/// named account did not exist — contradicting the spec execution order (validate
/// existence → then check dry-run flag).
///
/// ## Why Not Caught
///
/// `aa05` tests the non-dry case; no prior test exercised the `dry::1` path with an
/// @-containing name that is absent from the credential store.
///
/// ## Fix Applied
///
/// Added existence guard before the dry-run branch in `account_assign_routine()`:
/// `check_account_exists()` now runs unconditionally — the dry-run flag only suppresses
/// the write step, never the precondition checks.
///
/// ## Prevention
///
/// For every command that accepts a `dry::` flag, pair a dry-unknown-account test with
/// the normal-path test. Validate-then-dry is the canonical order for all mutation commands.
///
/// ## Pitfall
///
/// `resolve_account_name` `@`-fast-path bypasses store validation intentionally (full email
/// → no prefix expansion needed), but callers must still call `check_account_exists()` after
/// resolution. Never assume a resolved name is a valid stored account.
fn aa13_dry_run_unknown_account_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::ghost@example.com", "dry::1" ], &refs );
  assert_exit( &out, 2 );
  let out_text = stdout( &out );
  assert!(
    !out_text.contains( "[dry-run] would assign" ),
    "dry-run output must not be emitted for a non-existent account: {out_text}",
  );
}

// ── AA14 — Usage block shows (none) when no active marker ─────────────────────

#[ test ]
/// AC-04 (none-branch): when `name::` is absent AND no `_active_{machine}_{user}` file
/// exists, the usage block shows `Active account: (none)` and omits `Ready to copy:`.
fn aa14_usage_block_no_active_marker_shows_none()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  // No account written; no _active_* marker file created.
  // Create the credential store directory so the command doesn't fail on store setup.
  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".account.assign" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "Active account:   (none)" ) || out_text.contains( "Active account: (none)" ),
    "usage block must show (none) when no marker file exists: {out_text}",
  );
  assert!(
    !out_text.contains( "Ready to copy:" ),
    "Ready to copy: section must be absent when active account is (none): {out_text}",
  );
}

// ── AA15 — Ambiguous prefix exits 1 ───────────────────────────────────────────

#[ test ]
/// Ambiguous bare prefix: `name::alice` when both `alice@corp.com` and `alice@other.com`
/// exist exits 1 (not exit 2), because the input is syntactically valid but the
/// resolution is ambiguous — a user error, not a "not found".
fn aa15_ambiguous_prefix_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com",  "max", "tier4", 9_999_999_999_999, false );
  write_account( dir.path(), "alice@other.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice" ], &refs );
  assert_exit( &out, 1 );

  let err_text = stderr( &out );
  assert!(
    err_text.contains( "ambiguous" ),
    "stderr must explain the ambiguity: {err_text}",
  );
}

// ── AA16 — Exact local-part beats prefix ambiguity ────────────────────────────

#[ test ]
/// Exact local-part match: when `i1@host` and `i11@host` both exist, `name::i1`
/// resolves to `i1@host` (exact local-part match wins over prefix scan).
///
/// Without this rule, `i1` would be ambiguous (it is a prefix of both `i1@host`
/// and `i11@host`). The exact-match shortcut ensures `i1` unambiguously selects `i1@host`.
fn aa16_exact_local_part_beats_prefix_ambiguity()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "i1@host",  "max", "tier4", 9_999_999_999_999, false );
  write_account( dir.path(), "i11@host", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::i1" ], &refs );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( DEFAULT_MARKER ) )
    .expect( "marker must exist after exact-local-part resolution" );
  assert_eq!( content.trim(), "i1@host", "exact local-part match must resolve to i1@host, not i11@host" );
}

// ── AA17 — for::@ (only @, both components empty) exits 1 ─────────────────────

#[ test ]
/// `for::@` — when the `for::` value is exactly `@`, both split components are empty.
/// The empty-user check fires first → exit 1.
fn aa17_for_only_at_both_empty_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::@" ], &refs );
  assert_exit( &out, 1 );

  let store = credential_store( dir.path() );
  assert_eq!( active_marker_count( &store ), 0, "no marker file must be written when for:: is invalid: for::@" );
}

// ── EC-7 ──────────────────────────────────────────────────────────────────────

#[ test ]
/// EC-7: dot and hyphen in machine component are preserved verbatim (kept in sanitization).
///
/// `for::user1@w003.local` — both `.` and `-` are in the allowed charset (alphanumeric, `-`, `.`).
/// The marker filename must be `_active_w003.local_user1`, not `_active_w003_local_user1`.
fn ec7_dot_hyphen_in_machine_preserved()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::user1@w003.local" ], &refs );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_w003.local_user1" ) )
    .expect( "_active_w003.local_user1 must exist — dot and hyphen must be preserved in sanitization" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

// ── EC-8 ──────────────────────────────────────────────────────────────────────

#[ test ]
/// EC-8: multiple `@` in `for::` value — split on the **first** `@` only.
///
/// `for::alice@corp.com@laptop` splits into:
/// - user component: `alice`
/// - machine component: `corp.com@laptop` (sanitized: `@` → `_` → `corp.com_laptop`)
///
/// Written filename: `_active_corp.com_laptop_alice`.
fn ec8_multiple_at_splits_on_first()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env(
    &[ ".account.assign", "name::alice@corp.com", "for::alice@corp.com@laptop" ],
    &refs,
  );
  assert_exit( &out, 0 );

  let store   = credential_store( dir.path() );
  let content = std::fs::read_to_string( store.join( "_active_corp.com_laptop_alice" ) )
    .expect( "_active_corp.com_laptop_alice must exist — split on first @ only, rest becomes machine" );
  assert_eq!( content.trim(), "alice@corp.com" );
}

#[ test ]
/// AC-12: `dry::1` with `for::bob@laptop` → stdout contains the target marker filename.
fn aa12_dry_run_shows_marker_filename()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let env  = test_env( home );
  let refs : Vec< ( &str, &str ) > = env.iter().map( | ( k, v ) | ( *k, *v ) ).collect();

  write_account( dir.path(), "alice@corp.com", "max", "tier4", 9_999_999_999_999, false );

  let out = run_cs_with_env( &[ ".account.assign", "name::alice@corp.com", "for::bob@laptop", "dry::1" ], &refs );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "_active_laptop_bob" ),
    "dry-run stdout must name the target marker file: {out_text}" );
}
