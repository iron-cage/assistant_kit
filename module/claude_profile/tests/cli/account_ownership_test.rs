//! Integration tests: AO (Account Ownership, Feature 036) + AU (Account Unclaim).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ao01 | `ft03_unclaim_param_placement` | `.account.save` does NOT list unclaim/owner:: | P |
//! | ao02 | `ft08_use_exits_1_when_not_owned` | non-owned `.account.use` → exit 1 | N |
//! | ao03 | `ft09_delete_exits_1_when_not_owned` | non-owned `.account.delete` → exit 1 | N |
//! | ao04 | `ft10_relogin_exits_1_when_not_owned` | non-owned `.account.relogin` → exit 1 | N |
//! | ao05 | `ft13_dry_run_does_not_skip_ownership` | `dry::1` with non-owned → exit 1 | N |
//! | ao06 | `ft01_save_does_not_stamp_owner` | `.account.save` does NOT modify owner | P |
//! | ao07 | `ft12_save_does_not_stamp_owner` | `.account.save` preserves existing owner | P |
//! | ec3 | `as_save_does_not_modify_owner` | `.account.save` preserves owner (IT-20) | P |
//! | ao08 | `ft18_use_force_bypasses_g5` | `.account.use force::1` → G5 bypassed | P |
//! | ao09 | `ft19_delete_force_bypasses_g6` | `.account.delete force::1` → G6 bypassed | P |
//! | ao10 | `ft20_relogin_force_bypasses_g7` | `.account.relogin force::1 dry::1` → bypassed | P |
//! | ao11 | `ft21_force_dry_bypasses_gate_previews` | G5/G6/G7/G8 all with force::1 dry::1 | P |
//! | ft02 | `ft02_unclaim_clears_owner` | `.accounts unclaim::1 name::X` → owner cleared | P |
//! | ft15 | `ft15_unclaim_not_on_save_or_assign` | `.account.save unclaim::1` → exit 1 | N |
//! | ft16 | `ft16_unclaim_g8_gate` | G8: non-owner exit 1; unowned exit 0 | N/P |
//! | ft17 | `ft17_unclaim_dry_run` | `dry::1` → preview, files unchanged | P |
//! | it01 | `it01_unclaim_clears_owner` | core unclaim — owner cleared | P |
//! | it02 | `it02_unclaim_credential_not_touched` | credential file content unchanged | P |
//! | it03 | `it03_unclaim_marker_not_touched` | active marker unchanged | P |
//! | it04 | `it04_unclaim_idempotent` | unowned → exit 0 | P |
//! | it05 | `it05_unclaim_g8_non_owner` | non-owner → exit 1 | N |
//! | it06 | `it06_unclaim_dry_run` | `dry::1` → [dry-run], no file change | P |
//! | it07 | `it07_unclaim_g8_before_dry` | non-owner + `dry::1` → exit 1 | N |
//! | it08 | `it08_unclaim_unknown_account` | unknown account → exit 2 | N |
//! | it09 | `it09_unclaim_missing_name` | no name → batch empty store → exit 0 | P |
//! | it10 | `it10_unclaim_unknown_param` | unknown parameter → exit 1 | N |
//! | it11 | `it11_unclaim_preserves_renewal_at` | `_renewal_at` preserved via read-merge | P |

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_owner, account_exists,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── AO: Account Ownership (Feature 036) ────────────────────────────────────

/// FT-03 (AC-03): `.account.save` does NOT list `unclaim` or `owner::`.
///
/// Structural assertion: `.account.save.help` does NOT list `unclaim` (param removed)
/// and does NOT list `owner::` (ownership is not user-specified). The former
/// `.account.assign` and `.account.unclaim` standalone commands have been removed
/// entirely (Feature 037) — real functionality lives at `assignee::USER@MACHINE`
/// and `owner::0` on `.accounts`/`.usage`.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-03]
#[ test ]
fn ft03_unclaim_param_placement()
{
  // .account.save must NOT list unclaim (param removed).
  let out_save = run_cs( &[ ".account.save.help" ] );
  assert_exit( &out_save, 0 );
  let save_text = stdout( &out_save );
  assert!(
    !save_text.contains( "unclaim" ),
    "FT-03: .account.save help must NOT list `unclaim` parameter; got:\n{save_text}",
  );
  assert!(
    !save_text.contains( "owner::" ),
    "FT-03: .account.save help must NOT list `owner::` parameter; got:\n{save_text}",
  );
}

/// FT-08 (AC-08): `.account.use` exits 1 when account not owned.
///
/// G5 gate: `read_owner()` returns `"other@remote"`, `is_owned()` returns
/// `false`, command exits 1 with `"ownership violation"` message.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-08]
#[ test ]
fn ft08_use_exits_1_when_not_owned()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "ownership violation" ),
    "FT-08: stderr must contain 'ownership violation'; got:\n{err}",
  );
  assert!(
    err.contains( "other@remote" ),
    "FT-08: stderr must name the owning identity; got:\n{err}",
  );
}

/// FT-09 (AC-09): `.account.delete` exits 1 when account not owned.
///
/// G6 gate: non-owned account cannot be deleted from this machine.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-09]
#[ test ]
fn ft09_delete_exits_1_when_not_owned()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "ownership violation" ),
    "FT-09: stderr must contain 'ownership violation'; got:\n{err}",
  );
  assert!(
    err.contains( "other@remote" ),
    "FT-09: stderr must name the owning identity; got:\n{err}",
  );
  assert!(
    account_exists( dir.path(), "alice@acme.com" ),
    "FT-09: credential file must NOT be deleted when ownership check fails",
  );
}

/// FT-10 (AC-10): `.account.relogin` exits 1 when account not owned.
///
/// G7 gate: non-owned account cannot be re-authenticated from this machine.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-10]
#[ test ]
fn ft10_relogin_exits_1_when_not_owned()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "ownership violation" ),
    "FT-10: stderr must contain 'ownership violation'; got:\n{err}",
  );
  assert!(
    err.contains( "other@remote" ),
    "FT-10: stderr must name the owning identity; got:\n{err}",
  );
}

/// FT-13 (AC-13): `dry::1` does NOT skip G5/G6/G7 ownership check.
///
/// Ownership guard runs BEFORE dry-run logic. All three commands exit 1
/// with the ownership violation message; the `[dry-run]` acknowledgment
/// is NOT printed.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-13]
#[ test ]
fn ft13_dry_run_does_not_skip_ownership()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  // G5: .account.use dry::1
  {
    let o = run_cs_with_env(
      &[ ".account.use", "name::alice@acme.com", "dry::1" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &o, 1 );
    let msg = stderr( &o );
    assert!( msg.contains( "ownership violation" ), "FT-13 G5: stderr must contain 'ownership violation'; got:\n{msg}" );
    assert!( !stdout( &o ).contains( "[dry-run]" ), "FT-13 G5: dry-run acknowledgment must NOT appear" );
  }

  // G6: .account.delete dry::1
  {
    let o = run_cs_with_env(
      &[ ".account.delete", "name::alice@acme.com", "dry::1" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &o, 1 );
    let msg = stderr( &o );
    assert!( msg.contains( "ownership violation" ), "FT-13 G6: stderr must contain 'ownership violation'; got:\n{msg}" );
    assert!( !stdout( &o ).contains( "[dry-run]" ), "FT-13 G6: dry-run acknowledgment must NOT appear" );
  }

  // G7: .account.relogin dry::1
  {
    let o = run_cs_with_env(
      &[ ".account.relogin", "name::alice@acme.com", "dry::1" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &o, 1 );
    let msg = stderr( &o );
    assert!( msg.contains( "ownership violation" ), "FT-13 G7: stderr must contain 'ownership violation'; got:\n{msg}" );
    assert!( !stdout( &o ).contains( "[dry-run]" ), "FT-13 G7: dry-run acknowledgment must NOT appear" );
  }
}

/// CC-9: G5 passes when account is unclaimed (empty owner).
///
/// `write_account_owner(home, name, "")` sets `owner: ""` → `is_owned()` returns
/// `true` → G5 does NOT fire → `.account.use` proceeds normally (exit 0).
/// This verifies that `unclaim::1` truly disables all enforcement gates.
#[ test ]
fn cc9_unclaimed_account_passes_use_gate()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  // `.account.use` copies credentials to `~/.claude/.credentials.json` — dir must exist.
  let dot_claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "ownership violation" ),
    "CC-9: unclaimed account (empty owner) must NOT trigger ownership violation; got:\n{err}",
  );
}

/// FT-12 (AC-19, Feat 036): `.account.save` does NOT modify the `owner` field — ownership-neutral.
///
/// Pre-seed `{name}.json` with `"owner": "old@host"`. Run `.account.save`; assert
/// `owner` is UNCHANGED — `account_save_routine()` passes `owner: None` to `save()`,
/// preserving the existing value via read-merge.
///
/// Spec: [`tests/docs/feature/02_account_save.md` FT-12]
#[ test ]
fn ft12_save_does_not_stamp_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_owner( dir.path(), "alice@acme.com", "old@host" );

  // .account.save reads from ~/.claude/.credentials.json — must exist.
  let dot_claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  let owner = val[ "owner" ].as_str().unwrap_or( "MISSING" );
  assert_eq!(
    owner, "old@host",
    "FT-12: .account.save must NOT modify owner — existing value must be preserved; got: {owner:?}",
  );
}

/// FT-01 (AC-01, Feat 036): `.account.save` does NOT modify the `owner` field — ownership-neutral.
///
/// Pre-seed `{name}.json` with `"owner": "old@host"`. After `.account.save`, the
/// `owner` field must remain `"old@host"` — `account_save_routine()` passes `owner: None`
/// to `save()`, preserving the existing value via read-merge.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-01]
#[ test ]
fn ft01_save_does_not_stamp_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_owner( dir.path(), "alice@acme.com", "old@host" );

  let dot_claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  let owner = val[ "owner" ].as_str().unwrap_or( "MISSING" );
  assert_eq!(
    owner, "old@host",
    "FT-01: .account.save must NOT modify owner — existing value must be preserved; got: {owner:?}",
  );
}


/// IT-20 (AC-19, Feat 002): `.account.save` does NOT modify the `owner` field.
///
/// Pre-seed `{name}.json` with `"owner": "user1@host1"`. After `.account.save`,
/// `owner` must remain `"user1@host1"` — `account_save_routine()` passes `owner: None`
/// to `save()`, preserving the existing value via read-merge.
///
/// Spec: [`tests/docs/cli/command/04_account_save.md` IT-20]
#[ test ]
fn as_save_does_not_modify_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( "alice@acme.com.json" ), r#"{"owner":"user1@host1"}"# ).unwrap();
  std::fs::write( store.join( "alice@acme.com.credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let dot_claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "other" ), ( "HOSTNAME", "machine2" ) ],
  );
  assert_exit( &out, 0 );

  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  let owner = val[ "owner" ].as_str().unwrap_or( "MISSING" );
  assert_eq!(
    owner, "user1@host1",
    "IT-20: .account.save must NOT modify owner — existing value must be preserved; got: {owner:?}",
  );
}

// ── AU: Account Unclaim (Feature 036, Command 17) ───────────────────────────

/// FT-02 (AC-02, Feat 036/037): `.accounts owner::0 name::X` writes `owner: ""` — credential file NOT touched.
///
/// Pre-seed `{name}.json` with `"owner": "user1@host1"`. Run `.accounts unclaim::1 name::alice`.
/// Owner must be `""`. Credential file mtime must be unchanged (pure metadata operation).
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-02]
#[ test ]
fn ft02_unclaim_clears_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@host1" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred_path = store.join( "alice@acme.com.credentials.json" );
  let cred_mtime_before = std::fs::metadata( &cred_path ).unwrap().modified().unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );

  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  let owner = val[ "owner" ].as_str().unwrap_or( "MISSING" );
  assert_eq!( owner, "", "FT-02: .accounts owner::0 must clear owner to \"\"; got: {owner:?}" );

  let cred_mtime_after = std::fs::metadata( &cred_path ).unwrap().modified().unwrap();
  assert_eq!(
    cred_mtime_before, cred_mtime_after,
    "FT-02: credential file must NOT be touched by .accounts owner::0",
  );
}

/// FT-15 (AC-15, Feat 036): `.account.save unclaim::1` exits 1 — `unclaim::` not registered.
///
/// `unclaim::` is NOT registered on `.account.save`. The former `.account.assign` command
/// is fully removed (Feature 037) — `unclaim::` never existed on it either.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-15]
#[ test ]
fn ft15_unclaim_not_on_save_or_assign()
{
  // .account.save unclaim::1 → exit 1 (unknown param)
  let out_save = run_cs( &[ ".account.save", "unclaim::1" ] );
  assert_exit( &out_save, 1 );
}

/// FT-16 (AC-16, Feat 036): G8 ownership gate on `.account.unclaim`.
///
/// Case A: non-owner → exit 1 with "ownership violation".
/// Case B: unowned (empty owner) → exit 0 (gate passes, idempotent unclaim).
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-16]
#[ test ]
fn ft16_unclaim_g8_gate()
{
  // Case A: non-owner → exit 1
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
    write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

    let out = run_cs_with_env(
      &[ ".accounts", "owner::0", "name::alice@acme.com" ],
      &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
    );
    assert_exit( &out, 1 );
    let err = stderr( &out );
    assert!(
      err.contains( "ownership violation" ),
      "FT-16A: stderr must contain 'ownership violation'; got:\n{err}",
    );
    assert!(
      err.contains( "other@remote" ),
      "FT-16A: stderr must name the owning identity; got:\n{err}",
    );
  }

  // Case B: unowned (empty owner) → exit 0 (idempotent)
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
    write_account_owner( dir.path(), "alice@acme.com", "" );

    let out = run_cs_with_env(
      &[ ".accounts", "owner::0", "name::alice@acme.com" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &out, 0 );
  }
}

/// FT-17 (AC-17, Feat 036/037): `.accounts unclaim::1 name::X dry::1` prints preview; files unchanged.
///
/// Pre-seed `{name}.json` with `"owner": "user1@host1"`. Run `.accounts unclaim::1 name::X dry::1`.
/// Owner must remain `"user1@host1"` — dry-run must not write.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-17]
#[ test ]
fn ft17_unclaim_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@host1" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta_path = store.join( "alice@acme.com.json" );
  let content_before = std::fs::read_to_string( &meta_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ) && text.contains( "clear owner" ),
    "FT-17: stdout must contain dry-run preview; got:\n{text}",
  );

  let content_after = std::fs::read_to_string( &meta_path ).unwrap();
  assert_eq!(
    content_before, content_after,
    "FT-17: dry::1 must not write; file content must be unchanged",
  );
}

/// FT-18 (AC-18, Feat 036): `.account.use name::X force::1` bypasses G5 ownership gate.
///
/// Account owned by `"other@remote"`; current identity = `"testuser@testmachine"`.
/// Without `force::1` → exit 1 (ownership violation). With `force::1` → gate
/// bypassed, `switch_account()` executes, exits 0.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-18]
#[ test ]
fn ft18_use_force_bypasses_g5()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  // `.account.use` copies credentials to `~/.claude/.credentials.json` — directory must exist.
  let dot_claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "ownership violation" ),
    "FT-18: force::1 must bypass G5 — no ownership violation message; got:\n{err}",
  );
}

/// FT-19 (AC-19, Feat 036): `.account.delete name::X force::1` bypasses G6 ownership gate.
///
/// Account owned by `"other@remote"`; current identity = `"testuser@testmachine"`.
/// Without `force::1` → exit 1 and file kept. With `force::1` → gate bypassed,
/// deletion proceeds, exits 0, credential file removed.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-19]
#[ test ]
fn ft19_delete_force_bypasses_g6()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    !account_exists( dir.path(), "alice@acme.com" ),
    "FT-19: force::1 must bypass G6 — credential file must be deleted",
  );
}

/// FT-20 (AC-20, Feat 036): `.account.relogin name::X force::1` bypasses G7 ownership gate.
///
/// Account owned by `"other@remote"`; current identity = `"testuser@testmachine"`.
/// Without `force::1` → exit 1 (ownership violation). With `force::1 dry::1` →
/// G7 bypassed, dry-run preview printed, exits 0.
///
/// `dry::1` is used here to prevent spawning the browser-login subprocess in CI.
/// The key assertion is that G7 does not fire (no ownership violation message).
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-20]
#[ test ]
fn ft20_relogin_force_bypasses_g7()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::alice@acme.com", "force::1", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "ownership violation" ),
    "FT-20: force::1 must bypass G7 — no ownership violation message; got:\n{err}",
  );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "FT-20: dry::1 must produce dry-run preview after gate bypass; got:\n{text}",
  );
}

/// FT-21 (AC-21, Feat 036): `force::1 dry::1` on G5–G8 commands bypasses gate and previews.
///
/// For each of the four commands (G5: `.account.use`, G6: `.account.delete`,
/// G7: `.account.relogin`, G8: `.accounts unclaim::1`): account owned by
/// `"other@remote"`, current identity `"local@local"`. `force::1 dry::1` →
/// ownership gate bypassed (no exit 1), `[dry-run]` message printed, exits 0.
/// File writes are suppressed by `dry::1`.
///
/// Spec: [`tests/docs/feature/36_account_ownership.md` FT-21]
#[ test ]
fn ft21_force_dry_bypasses_gate_previews()
{
  // G5: .account.use force::1 dry::1
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
    write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

    let o = run_cs_with_env(
      &[ ".account.use", "name::alice@acme.com", "force::1", "dry::1" ],
      &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
    );
    assert_exit( &o, 0 );
    assert!( !stderr( &o ).contains( "ownership violation" ), "FT-21 G5: must not exit with ownership violation" );
    assert!( stdout( &o ).contains( "[dry-run]" ), "FT-21 G5: [dry-run] message required in stdout" );
  }

  // G6: .account.delete force::1 dry::1
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
    write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

    let o = run_cs_with_env(
      &[ ".account.delete", "name::alice@acme.com", "force::1", "dry::1" ],
      &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
    );
    assert_exit( &o, 0 );
    assert!( !stderr( &o ).contains( "ownership violation" ), "FT-21 G6: must not exit with ownership violation" );
    assert!( stdout( &o ).contains( "[dry-run]" ), "FT-21 G6: [dry-run] message required in stdout" );
    assert!( account_exists( dir.path(), "alice@acme.com" ), "FT-21 G6: dry::1 must not delete credential file" );
  }

  // G7: .account.relogin force::1 dry::1
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
    write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

    let o = run_cs_with_env(
      &[ ".account.relogin", "name::alice@acme.com", "force::1", "dry::1" ],
      &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
    );
    assert_exit( &o, 0 );
    assert!( !stderr( &o ).contains( "ownership violation" ), "FT-21 G7: must not exit with ownership violation" );
    assert!( stdout( &o ).contains( "[dry-run]" ), "FT-21 G7: [dry-run] message required in stdout" );
  }

  // G8: .accounts unclaim::1 force::1 dry::1
  {
    let dir  = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
    write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

    let store        = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
    let meta_path    = store.join( "alice@acme.com.json" );
    let meta_before  = std::fs::read_to_string( &meta_path ).unwrap();

    let o = run_cs_with_env(
      &[ ".accounts", "owner::0", "name::alice@acme.com", "force::1", "dry::1" ],
      &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
    );
    assert_exit( &o, 0 );
    assert!( !stderr( &o ).contains( "ownership violation" ), "FT-21 G8: must not exit with ownership violation" );
    assert!( stdout( &o ).contains( "[dry-run]" ), "FT-21 G8: [dry-run] message required in stdout" );
    let meta_after = std::fs::read_to_string( &meta_path ).unwrap();
    assert_eq!( meta_before, meta_after, "FT-21 G8: dry::1 must not write owner field" );
  }
}

/// IT-1: core unclaim — owner match clears owner to `""`.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-1]
#[ test ]
fn it01_unclaim_clears_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@host1" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!( val[ "owner" ].as_str().unwrap_or( "MISSING" ), "", "IT-1: owner must be \"\"" );
}

/// IT-2: credential file NOT touched.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-2]
#[ test ]
fn it02_unclaim_credential_not_touched()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@host1" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred_path = store.join( "alice@acme.com.credentials.json" );
  let before    = std::fs::read_to_string( &cred_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );

  let after = std::fs::read_to_string( &cred_path ).unwrap();
  assert_eq!( before, after, "IT-2: credential file content must be unchanged" );
}

/// IT-3: active marker NOT touched.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-3]
#[ test ]
fn it03_unclaim_marker_not_touched()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account with make_active=true writes _active_{hostname}_{user} marker.
  // We set USER/HOSTNAME to known values so we can predict the marker filename.
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account_owner( dir.path(), "alice@acme.com", "user1@host1" );

  // The marker file written by write_account uses the process's current env for hostname,
  // so just find any _active* file in the store.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let marker_path = std::fs::read_dir( &store ).unwrap()
    .filter_map( Result::ok )
    .map( |e| e.path() )
    .find( |p| p.file_name().unwrap_or_default().to_string_lossy().starts_with( "_active" ) )
    .expect( "IT-3 setup: _active marker must exist" );
  let marker_before = std::fs::read_to_string( &marker_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );

  let marker_after = std::fs::read_to_string( &marker_path ).unwrap();
  assert_eq!( marker_before, marker_after, "IT-3: active marker must be unchanged" );
}

/// IT-4: idempotent — unclaiming an already-unowned account exits 0.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-4]
#[ test ]
fn it04_unclaim_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

/// IT-5: G8 gate — non-owner exits 1.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-5]
#[ test ]
fn it05_unclaim_g8_non_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
  );
  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "ownership violation" ), "IT-5: must report ownership violation" );
}

/// IT-6: dry-run — `dry::1` prints preview, files unchanged.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-6]
#[ test ]
fn it06_unclaim_dry_run()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "user1@host1" );

  let store     = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta_path = store.join( "alice@acme.com.json" );
  let before    = std::fs::read_to_string( &meta_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "[dry-run]" ), "IT-6: stdout must contain [dry-run]" );

  let after = std::fs::read_to_string( &meta_path ).unwrap();
  assert_eq!( before, after, "IT-6: dry-run must not modify files" );
}

/// IT-7: G8 gate fires before dry-run — non-owner + `dry::1` → exit 1.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-7]
#[ test ]
fn it07_unclaim_g8_before_dry()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
  );
  assert_exit( &out, 1 );
  assert!( stderr( &out ).contains( "ownership violation" ), "IT-7: G8 must fire before dry-run" );
  assert!( !stdout( &out ).contains( "[dry-run]" ), "IT-7: dry-run msg must NOT appear" );
}

/// IT-8: unknown account → exit 2.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-8]
#[ test ]
fn it08_unclaim_unknown_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::ghost@nowhere.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  assert!( stderr( &out ).contains( "not found" ), "IT-8: stderr must mention 'not found'" );
}

/// IT-9: no `name::` → batch unclaim all owned accounts; empty store → exit 0.
///
/// Feature 037 moved unclaim into `.accounts unclaim::1`. Without `name::`, the
/// batch path runs: iterates all accounts, finds none owned → exits 0 with message.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-9]
#[ test ]
fn it09_unclaim_missing_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // owner::0 with no name:: → batch-clear on empty store → exits 0 (no accounts to iterate).
  let out = run_cs_with_env( &[ ".accounts", "owner::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// IT-10: unknown parameter → exit 1.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-10]
#[ test ]
fn it10_unclaim_unknown_param()
{
  // bogus:: is an unregistered param → framework exits 1 before dispatch.
  let out = run_cs( &[ ".accounts", "owner::0", "name::test", "bogus::1" ] );
  assert_exit( &out, 1 );
}

/// IT-11: read-merge preserves `_renewal_at` across unclaim.
///
/// Spec: [`tests/docs/cli/command/18_account_unclaim.md` IT-11]
#[ test ]
fn it11_unclaim_preserves_renewal_at()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( "alice@acme.com.json" ),
    r#"{"_renewal_at":"2026-06-29T21:00:00Z","owner":"user1@host1"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "alice@acme.com.credentials.json" ),
    r#"{"accessToken":"tok"}"#,
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "user1" ), ( "HOSTNAME", "host1" ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &content ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "",
    "IT-11: owner must be cleared",
  );
  assert_eq!(
    val[ "_renewal_at" ].as_str().unwrap_or( "MISSING" ), "2026-06-29T21:00:00Z",
    "IT-11: _renewal_at must be preserved via read-merge in write_owner()",
  );
}

