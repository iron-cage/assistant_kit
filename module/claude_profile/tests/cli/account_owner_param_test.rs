//! Integration tests: AP (Account Owner Param, Feature 063) — `owner::` EC tests.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## EC-N Spec Map
//!
//! Maps `ec1`..`ec9` function names to EC-01..EC-09 from
//! `tests/docs/cli/param/63_owner.md`.
//!
//! | Function | EC-N (63_owner.md) |
//! |----------|--------------------|
//! | `ec1_owner_sets_custom_identity` | EC-01 |
//! | `ec2_owner_empty_rejected` | EC-02 |
//! | `ec3_owner_and_unclaim_removed_toggle` | EC-03 |
//! | `ec4_owner_missing_name_exits_1` | EC-04 |
//! | `ec5_owner_g8_foreign_owner_blocked` | EC-05 |
//! | `ec6_owner_force_bypasses_g8` | EC-06 |
//! | `ec7_owner_dry_no_file_writes` | EC-07 |
//! | `ec8_owner_overwrite_existing` | EC-08 |
//! | `ec9_owner_idempotent_same_value` | EC-09 |
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ft01 | `ft_owner_sets_owner_field` | `owner::user1@w003 name::X` writes owner | P |
//! | ft02 | `ft_owner_requires_name` | `owner::X` without `name::` → exit 1 | N |
//! | ft03 | `ft_owner_g8_blocks_non_owner` | G8: owned by another → exit 1 | N |
//! | ft04 | `ft_owner_unowned_passes_g8` | unowned → write succeeds | P |
//! | ft05 | `ft_owner_mutual_exclusion_unclaim` | `owner:: + unclaim::1` → exit 1 | N |
//! | ft06 | `ft_owner_dry_run_preview` | `dry::1` → preview, no file writes | P |
//! | ft07 | `ft_owner_force_bypasses_g8` | `force::1` bypasses G8 | P |
//! | ft08 | `ft_owner_trace_emits_diagnostic` | `trace::1` → stderr diagnostic | P |
//! | ft09 | `ft_owner_prefix_resolution` | short name resolves to full email | P |
//! | ft10 | `ft_owner_empty_value_rejected` | empty `owner::` → exit 1 | N |
//! | ft11 | `ft_owner_gates_respect_new_value` | subsequent ops respect new owner | P |
//! | ec01 | `ec1_owner_sets_custom_identity` | `owner::alice@laptop` writes custom identity | P |
//! | ec02 | `ec2_owner_empty_rejected` | empty value → exit 1 with unclaim::1 hint | N |
//! | ec03 | `ec3_owner_and_unclaim_mutual_exclusion` | `owner:: + unclaim::1` → exit 1 | N |
//! | ec04 | `ec4_owner_missing_name_exits_1` | no `name::` → exit 1 | N |
//! | ec05 | `ec5_owner_g8_foreign_owner_blocked` | foreign owner → exit 1 | N |
//! | ec06 | `ec6_owner_force_bypasses_g8` | `force::1` bypasses G8 | P |
//! | ec07 | `ec7_owner_dry_no_file_writes` | `dry::1` no file writes | P |
//! | ec08 | `ec8_owner_overwrite_existing` | owned by caller → overwrites | P |
//! | ec09 | `ec9_owner_idempotent_same_value` | same value → idempotent exit 0 | P |
//! | ec10 | `ec10_owner_zero_clears_ownership` | `owner::0 name::X` → owner `""`, unclaimed | P |
//! | ec11 | `ec11_owner_zero_no_name_batch_clears` | `owner::0` batch → clears owned, skips unowned | P |
//! | ec12 | `ec12_owner_zero_comma_list_batch_clear` | `owner::0 name::X,Y,Z` → clears all | P |
//! | ec13 | `ec13_owner_set_comma_list_batch_set` | `owner::user name::X,Y,Z` → sets all | P |
//! | ec14 | `ec14_owner_zero_force_bypasses_g8` | `owner::0 force::1` → G8 bypassed | P |
//! | ec15 | `ec15_owner_zero_dry_run` | `owner::0 dry::1` → preview only | P |
//! | ec16 | `ec16_owner_zero_force_dry_run` | `owner::0 force::1 dry::1` → bypass + dry | P |
//! | ec17 | `ec17_owner_zero_unowned_clears_idempotent` | `owner::0 name::X` unowned → idempotent | P |
//! | ec18 | `ec18_owner_zero_g8_blocks_foreign_no_force` | `owner::0` foreign, no force → exit 1 | N |
//! | ec19 | `ec19_owner_zero_missing_json_exits_2` | `owner::0 name::X`, `.json` absent → exit 2 | N |
//! | ec20 | `ec20_owner_zero_batch_force_clears_foreign` | `owner::0 force::1` batch → clears foreign | P |

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_owner,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── AP: Account Owner Param (Feature 063) ─────────────────────────────────────

/// FT-01 (AC-01, Feat 063): `.accounts owner::user1@w003 name::X` writes owner field.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-01]
#[ test ]
fn ft_owner_sets_owner_field()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "user1@w003",
    "FT-01: owner must be 'user1@w003'",
  );
  assert!(
    stdout( &out ).contains( "owned alice@acme.com by user1@w003" ),
    "FT-01: stdout must confirm ownership; got:\n{}", stdout( &out ),
  );
}

/// FT-02 (AC-02, Feat 063): `.accounts owner::X` without `name::` → exit 1.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-02]
#[ test ]
fn ft_owner_requires_name()
{
  let out = run_cs( &[ ".accounts", "owner::user1@w003" ] );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "requires name" ),
    "FT-02: stderr must mention 'requires name'; got:\n{}", stderr( &out ),
  );
}

/// FT-03 (AC-03, Feat 063): G8 gate — owned by another → exit 1.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-03]
#[ test ]
fn ft_owner_g8_blocks_non_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::me@here", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "me" ), ( "HOSTNAME", "here" ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "ownership violation" ),
    "FT-03: stderr must contain 'ownership violation'; got:\n{}", stderr( &out ),
  );
}

/// FT-04 (AC-04, Feat 063): unowned account → write succeeds.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-04]
#[ test ]
fn ft_owner_unowned_passes_g8()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "user1@w003",
    "FT-04: unowned account must accept owner write",
  );
}

/// FT-05 (AC-05, Feat 063): `owner:: + unclaim::1` → exit 1 (mutual exclusion).
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-05]
#[ test ]
fn ft_owner_mutual_exclusion_unclaim()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "test@acme.com", "" );

  // Feature 064: unclaim::1 is REMOVED_TOGGLE — exits 1 with migration message.
  // (The mutual exclusion with owner:: is now moot since unclaim::1 is removed.)
  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "unclaim::1", "name::test@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "REMOVED" ) || stderr( &out ).contains( "owner::0" ),
    "FT-05: unclaim::1 is REMOVED_TOGGLE — stderr must contain migration hint; got:\n{}", stderr( &out ),
  );
}

/// FT-06 (AC-06, Feat 063): `dry::1` → preview, no file writes.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-06]
#[ test ]
fn ft_owner_dry_run_preview()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta_path = store.join( "alice@acme.com.json" );
  let before    = std::fs::read_to_string( &meta_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "FT-06: stdout must contain '[dry-run]'; got:\n{text}",
  );

  let after = std::fs::read_to_string( &meta_path ).unwrap();
  assert_eq!( before, after, "FT-06: dry-run must not modify files" );
}

/// FT-07 (AC-07, Feat 063): `force::1` bypasses G8 for other-owned account.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-07]
#[ test ]
fn ft_owner_force_bypasses_g8()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::me@here", "name::alice@acme.com", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "me" ), ( "HOSTNAME", "here" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "me@here",
    "FT-07: force::1 must bypass G8 and write owner",
  );
}

/// FT-08 (AC-08, Feat 063): `trace::1` → stderr diagnostic.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-08]
#[ test ]
fn ft_owner_trace_emits_diagnostic()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@acme.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( " · " ) && err.contains( "write_owner" ),
    "FT-08: stderr must contain trace diagnostic; got:\n{err}",
  );
}

/// FT-09 (AC-09, Feat 063): short name resolves to full email.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-09]
#[ test ]
fn ft_owner_prefix_resolution()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "user1@w003",
    "FT-09: prefix 'alice' must resolve to 'alice@acme.com'",
  );
}

/// FT-10 (AC-10, Feat 063): empty `owner::` → exit 1.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-10]
#[ test ]
fn ft_owner_empty_value_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "test@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::", "name::test@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "non-empty" ) || stderr( &out ).contains( "unclaim" ),
    "FT-10: stderr must reject empty owner; got:\n{}", stderr( &out ),
  );
}

/// FT-11 (AC-11, Feat 063): subsequent ops respect new owner.
///
/// After setting owner, a subsequent unclaim by a different identity is blocked by G8.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-11]
#[ test ]
fn ft_owner_gates_respect_new_value()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  // Step 1: claim ownership.
  let out = run_cs_with_env(
    &[ ".accounts", "owner::alice@laptop", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Step 2: clear ownership by different identity → G8 must block.
  let out2 = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "bob" ), ( "HOSTNAME", "desktop" ) ],
  );
  assert_exit( &out2, 1 );
  assert!(
    stderr( &out2 ).contains( "ownership violation" ),
    "FT-11: subsequent owner::0 by different identity must be blocked; got:\n{}", stderr( &out2 ),
  );
}

// ── EC: owner:: edge cases (Param 062) ────────────────────────────────────────

/// EC-01 (Param 062): `owner::alice@laptop name::X` writes `"owner": "alice@laptop"`.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-01]
#[ test ]
fn ec1_owner_sets_custom_identity()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::alice@laptop", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "alice@laptop",
    "EC-01: owner must be 'alice@laptop'",
  );
}

/// EC-02 (Param 062): empty `owner::` → exit 1 with `owner::0` hint (Feature 064).
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-02]
#[ test ]
fn ec2_owner_empty_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "test@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::", "name::test@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "owner::0" ),
    "EC-02: stderr must mention 'owner::0' as the correct way to clear ownership (Feature 064); got:\n{}", stderr( &out ),
  );
}

/// EC-03 (Param 062): `owner::user1@w003 unclaim::1 name::X` → exit 1 — `unclaim::1`
/// is `REMOVED_TOGGLE` (Feature 064); migration message directs user to `owner::0`.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-03]
#[ test ]
fn ec3_owner_and_unclaim_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "test@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "unclaim::1", "name::test@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "REMOVED" ),
    "EC-03: stderr must contain REMOVED_TOGGLE migration message for unclaim::1 (Feature 064); got:\n{}", stderr( &out ),
  );
}

/// EC-04 (Param 062): `owner::user1@w003` (no `name::`) → exit 1.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-04]
#[ test ]
fn ec4_owner_missing_name_exits_1()
{
  let out = run_cs( &[ ".accounts", "owner::user1@w003" ] );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "requires name" ),
    "EC-04: stderr must mention 'requires name'; got:\n{}", stderr( &out ),
  );
}

/// EC-05 (Param 062): G8 — account owned by another → exit 1 ownership violation.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-05]
#[ test ]
fn ec5_owner_g8_foreign_owner_blocked()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@host" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::me@local", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "me" ), ( "HOSTNAME", "local" ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "ownership violation" ),
    "EC-05: stderr must contain 'ownership violation'; got:\n{}", stderr( &out ),
  );
}

/// EC-06 (Param 062): same as EC-05 + `force::1` → write succeeds, exit 0.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-06]
#[ test ]
fn ec6_owner_force_bypasses_g8()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@host" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::me@local", "name::alice@acme.com", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "me" ), ( "HOSTNAME", "local" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "me@local",
    "EC-06: force::1 must bypass G8",
  );
}

/// EC-07 (Param 062): `owner::user1@w003 name::X dry::1` → `[dry-run]`, no file writes.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-07]
#[ test ]
fn ec7_owner_dry_no_file_writes()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta_path = store.join( "alice@acme.com.json" );
  let before    = std::fs::read_to_string( &meta_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "[dry-run]" ), "EC-07: stdout must contain [dry-run]" );

  let after = std::fs::read_to_string( &meta_path ).unwrap();
  assert_eq!( before, after, "EC-07: dry-run must not modify files" );
}

/// EC-08 (Param 062): account owned by caller → `owner::new@identity` → overwrites.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-08]
#[ test ]
fn ec8_owner_overwrite_existing()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@w003" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::new@identity", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "new@identity",
    "EC-08: owner must be overwritten to 'new@identity'",
  );
}

/// EC-09 (Param 062): idempotent — same `owner::user1@w003` when already owned by same → exit 0.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-09]
#[ test ]
fn ec9_owner_idempotent_same_value()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@w003" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "user1@w003",
    "EC-09: idempotent write must preserve owner",
  );
}

/// EC-10 (Param 062): `owner::0 name::alice@corp.com` → writes `owner: ""`, exits 0, stdout `unclaimed`.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-10]
#[ test ]
fn ec10_owner_zero_clears_ownership()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "user1@w003" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "unclaimed alice@corp.com" ),
    "EC-10: stdout must contain 'unclaimed alice@corp.com'; got:\n{}", stdout( &out ),
  );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@corp.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "",
    "EC-10: owner must be cleared to empty string",
  );
}

/// EC-11 (Param 062): `owner::0` alone (no `name::`) — batch-clears owned accounts, skips unowned with message.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-11]
#[ test ]
fn ec11_owner_zero_no_name_batch_clears()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // A: owned by caller → cleared
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@w003" );
  // B: unowned → skipped with "skip" message
  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@acme.com", "" );
  // C: owned by caller → cleared
  write_account( dir.path(), "carol@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "carol@acme.com", "user1@w003" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "unclaimed alice@acme.com" ),
    "EC-11: stdout must contain 'unclaimed alice@acme.com'; got:\n{out_text}",
  );
  assert!(
    out_text.contains( "skip bob@acme.com" ),
    "EC-11: stdout must contain 'skip bob@acme.com' for unowned account; got:\n{out_text}",
  );
  assert!(
    out_text.contains( "unclaimed carol@acme.com" ),
    "EC-11: stdout must contain 'unclaimed carol@acme.com'; got:\n{out_text}",
  );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  let meta_a : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap()
  ).unwrap();
  assert_eq!( meta_a[ "owner" ].as_str().unwrap_or( "MISSING" ), "", "EC-11: alice owner must be cleared" );

  let meta_c : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( store.join( "carol@acme.com.json" ) ).unwrap()
  ).unwrap();
  assert_eq!( meta_c[ "owner" ].as_str().unwrap_or( "MISSING" ), "", "EC-11: carol owner must be cleared" );
}

/// EC-12 (Param 062): `owner::0 name::X,Y,Z` — batch-clear via comma-list; G8 per account; exits 0.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-12]
#[ test ]
fn ec12_owner_zero_comma_list_batch_clear()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "user1@w003" );
  write_account( dir.path(), "bob@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@corp.com", "user1@w003" );
  write_account( dir.path(), "charlie@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "charlie@corp.com", "user1@w003" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com,bob@corp.com,charlie@corp.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "unclaimed alice@corp.com" ),   "EC-12: must unclaim alice" );
  assert!( out_text.contains( "unclaimed bob@corp.com" ),     "EC-12: must unclaim bob" );
  assert!( out_text.contains( "unclaimed charlie@corp.com" ), "EC-12: must unclaim charlie" );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  for name in &[ "alice@corp.com", "bob@corp.com", "charlie@corp.com" ]
  {
    let meta : serde_json::Value = serde_json::from_str(
      &std::fs::read_to_string( store.join( format!( "{name}.json" ) ) ).unwrap()
    ).unwrap();
    assert_eq!(
      meta[ "owner" ].as_str().unwrap_or( "MISSING" ), "",
      "EC-12: {name} owner must be cleared",
    );
  }
}

/// EC-13 (Param 062): `owner::user1@w003 name::X,Y,Z` — batch-set via comma-list; exits 0.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-13]
#[ test ]
fn ec13_owner_set_comma_list_batch_set()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "" );
  write_account( dir.path(), "bob@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@corp.com", "" );
  write_account( dir.path(), "charlie@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "charlie@corp.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::user1@w003", "name::alice@corp.com,bob@corp.com,charlie@corp.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "owned alice@corp.com by user1@w003" ),   "EC-13: must own alice" );
  assert!( out_text.contains( "owned bob@corp.com by user1@w003" ),     "EC-13: must own bob" );
  assert!( out_text.contains( "owned charlie@corp.com by user1@w003" ), "EC-13: must own charlie" );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  for name in &[ "alice@corp.com", "bob@corp.com", "charlie@corp.com" ]
  {
    let meta : serde_json::Value = serde_json::from_str(
      &std::fs::read_to_string( store.join( format!( "{name}.json" ) ) ).unwrap()
    ).unwrap();
    assert_eq!(
      meta[ "owner" ].as_str().unwrap_or( "MISSING" ), "user1@w003",
      "EC-13: {name} owner must be set to user1@w003",
    );
  }
}

/// EC-14 (Param 062): `owner::0 name::X force::1` — bypasses G8 for foreign-owned account.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-14]
#[ test ]
fn ec14_owner_zero_force_bypasses_g8()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "unclaimed alice@corp.com" ),
    "EC-14: stdout must contain 'unclaimed alice@corp.com'; got:\n{}", stdout( &out ),
  );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@corp.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "",
    "EC-14: force::1 must bypass G8 and clear owner",
  );
}

/// EC-15 (Param 062): `owner::0 name::X dry::1` — `[dry-run]` message, no file written.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-15]
#[ test ]
fn ec15_owner_zero_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "user1@w003" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta_path = store.join( "alice@corp.com.json" );
  let before    = std::fs::read_to_string( &meta_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "[dry-run]" ),
    "EC-15: stdout must contain '[dry-run]'; got:\n{}", stdout( &out ),
  );

  let after = std::fs::read_to_string( &meta_path ).unwrap();
  assert_eq!( before, after, "EC-15: dry-run must not modify files" );
}

/// EC-16 (Param 062): `owner::0 name::X force::1 dry::1` — bypass G8 + dry-run; no files written.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-16]
#[ test ]
fn ec16_owner_zero_force_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@remote" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta_path = store.join( "alice@corp.com.json" );
  let before    = std::fs::read_to_string( &meta_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com", "force::1", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "[dry-run]" ),
    "EC-16: stdout must contain '[dry-run]'; got:\n{}", stdout( &out ),
  );

  let after = std::fs::read_to_string( &meta_path ).unwrap();
  assert_eq!( before, after, "EC-16: dry-run must not modify files even with force::1" );
}

/// EC-17 (Param 062): `owner::0 name::X` on already-unowned account — idempotent, exit 0.
///
/// G8 passes for unowned accounts (`is_owned("") = true`); write is applied and
/// stdout contains `unclaimed`.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-17]
#[ test ]
fn ec17_owner_zero_unowned_clears_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  // Unowned — owner field is ""
  write_account_owner( dir.path(), "alice@corp.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "unclaimed alice@corp.com" ),
    "EC-17: stdout must contain 'unclaimed alice@corp.com'; got:\n{}", stdout( &out ),
  );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@corp.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "",
    "EC-17: owner must remain empty for idempotent clear",
  );
}

/// EC-18 (Param 062): `owner::0 name::X` — foreign owner, no `force::1` → G8 blocks, exit 1.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-18]
#[ test ]
fn ec18_owner_zero_g8_blocks_foreign_no_force()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "ownership violation" ),
    "EC-18: stderr must contain 'ownership violation'; got:\n{}", stderr( &out ),
  );

  // File must not have been modified.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@corp.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "other@remote",
    "EC-18: owner must not be modified when G8 blocks",
  );
}

/// EC-19 (Param 062): `owner::0 name::X` when `{name}.json` is absent → exit 2.
///
/// Only `.credentials.json` exists; the metadata file is absent.  The named
/// dispatch path requires `.json` and exits with `InternalError` (exit 2).
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-19]
#[ test ]
fn ec19_owner_zero_missing_json_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account creates .credentials.json but NOT .json
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@corp.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 2 );
  assert!(
    stderr( &out ).contains( "account not found" ),
    "EC-19: stderr must contain 'account not found'; got:\n{}", stderr( &out ),
  );
}

/// EC-20 (Param 062): `owner::0` batch + `force::1` — clears accounts owned by a foreign identity.
///
/// Without `force::1` the batch path skips foreign-owned accounts.  With
/// `force::1` it bypasses G8 and clears all.
///
/// Spec: [`tests/docs/cli/param/63_owner.md` EC-20]
#[ test ]
fn ec20_owner_zero_batch_force_clears_foreign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Two accounts owned by a different identity.
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@corp.com", "other@remote" );
  write_account( dir.path(), "bob@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@corp.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "w003" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "unclaimed alice@corp.com" ),
    "EC-20: stdout must contain 'unclaimed alice@corp.com'; got:\n{text}",
  );
  assert!(
    text.contains( "unclaimed bob@corp.com" ),
    "EC-20: stdout must contain 'unclaimed bob@corp.com'; got:\n{text}",
  );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  for name in &[ "alice@corp.com", "bob@corp.com" ]
  {
    let meta = std::fs::read_to_string( store.join( format!( "{name}.json" ) ) ).unwrap();
    let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
    assert_eq!(
      val[ "owner" ].as_str().unwrap_or( "MISSING" ), "",
      "EC-20: owner of {name} must be cleared to empty",
    );
  }
}
