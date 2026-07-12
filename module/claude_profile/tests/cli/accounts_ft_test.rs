//! Integration tests: FT — Feature 037 param unification + FT force/unclaim variants.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ft01 | `ft01_accounts_accepts_32_params` | `.accounts` accepts all 32 registered params | P |
//! | ft03 | `ft03_accounts_default_profile` | `.accounts` default output includes Owner column | P |
//! | ft07 | `ft07_accounts_unclaim_batch` | `unclaim::1` no name → clears all owned accounts | P |
//! | ft13 | `ft13_accounts_legacy_toggles_rejected` | removed toggle param → exit 1 + migration message | N |
//! | ft14 | `ft14_accounts_cols_modifier` | `cols::+display_name` → Display: line present | P |
//! | ft15 | `lim_it_ft15_accounts_refresh_live` | `refresh::1` with live token → account shown | P |
//! | ft19 | `ft19_owner_column_default_visible` | Owner: line visible by default | P |
//! | ft20 | `ft20_accounts_unclaim_force_bypasses_g8` | `unclaim::1 force::1` → clears regardless | P |
//! | ft21 | `ft21_force_no_effect_without_unclaim` | `force::1` alone → accepted, no mutation | P |
//! | it_batch_unclaim_force | `it_batch_unclaim_force_clears_non_owned` | `unclaim::1 force::1` → clears ALL | P |
//! | it_batch_unclaim_force_dry | `it_batch_unclaim_force_dry_previews_all` | `dry::1` → no writes | P |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token,
  write_account_profile_json, write_account_owner,
  write_account_renewal_json, write_account_roles_json,
  live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── FT-01 / FT-03 / FT-07 / FT-13 / FT-14 / FT-19 / FT-20 / FT-21 ──────────

#[ test ]
/// FT-01 (AC-01): `.accounts` accepts all 32 unified params; unknown param exits 1.
///
/// Structural registration test: each of the 32 unified params must not produce
/// "unknown parameter" errors. Mutation params are gated with `dry::1` to
/// prevent side-effects in the offline test environment.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-01]
fn ft01_accounts_accepts_32_params()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  // Display and filter params (offline-safe; no network, no writes).
  let out = run_cs_with_env(
    &[
      ".accounts",
      "trace::1",
      "format::text",
      "cols::+uuid,-tier",
      "sort::name",
      "desc::0",
      "no_color::1",
      "count::10",
      "offset::0",
      "only_active::0",
      "only_next::0",
      "min_5h::0",
      "min_7d::0",
      "only_valid::0",
      "exclude_exhausted::0",
      "abs::0",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // prefer/next/imodel/effort accepted (no-op when refresh::0).
  let out = run_cs_with_env(
    &[ ".accounts", "prefer::any", "next::renew", "imodel::auto", "effort::auto" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // assignee:: + name:: + dry::1 accepted (Feature 065 ownership mutation).
  let out = run_cs_with_env(
    &[ ".accounts", "assignee::testuser@testmachine", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Unknown parameter exits 1.
  let out = run_cs_with_env(
    &[ ".accounts", "unknown_foobar_xyz::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

#[ test ]
/// FT-03 (AC-03): `.accounts` default — no HTTP fetch, no subprocess, identity column set.
///
/// With `trace::1`, no ` · fetch` or ` · touch` timestamp lines should appear.
/// Owner column (default-on) must be present in output.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-03]
fn ft03_accounts_default_profile()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  let out = run_cs_with_env(
    &[ ".accounts", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let err = stderr( &out );
  assert!(
    !err.contains( " · fetch  " ),
    "FT-03: default .accounts must NOT produce fetch trace lines (no network call); got stderr:\n{err}",
  );
  assert!(
    !err.contains( " · touch  " ),
    "FT-03: default .accounts must NOT produce touch trace lines; got stderr:\n{err}",
  );

  let text = stdout( &out );
  assert!(
    text.contains( "Owner:" ),
    "FT-03: Owner column must appear by default (identity set); got:\n{text}",
  );
  assert!(
    text.contains( "testuser@testmachine" ),
    "FT-03: owner value must appear for alice; got:\n{text}",
  );
}

#[ test ]
/// FT-07 (AC-07): `.accounts unclaim::1` batch — applies to all filtered accounts; G8 per-account.
///
/// alice (owned by testuser@testmachine = current identity) → unclaimed; `alice.json` gets `"owner": ""`.
/// bob (owned by other@remote ≠ current identity) → skipped; stdout shows `"skip bob: owned by other@remote"`.
/// Overall exit 0 (best-effort batch — skips are logged, not failures).
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-07]
fn ft07_accounts_unclaim_batch()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "unclaimed alice@acme.com" ),
    "FT-07: alice must be unclaimed; got stdout:\n{text}",
  );
  assert!(
    text.contains( "skip bob@acme.com" ) || text.contains( "other@remote" ),
    "FT-07: bob must be skipped with ownership note; got stdout:\n{text}",
  );

  let store    = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let alice_meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let alice_val : serde_json::Value = serde_json::from_str( &alice_meta ).unwrap();
  assert_eq!(
    alice_val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "",
    "FT-07: alice owner must be cleared",
  );

  let bob_meta = std::fs::read_to_string( store.join( "bob@acme.com.json" ) ).unwrap();
  let bob_val : serde_json::Value = serde_json::from_str( &bob_meta ).unwrap();
  assert_eq!(
    bob_val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "other@remote",
    "FT-07: bob owner must be unchanged",
  );
}

#[ test ]
/// FT-13 (AC-13): `.accounts` rejects all 15 legacy field-toggle params with `cols::` migration message.
///
/// Each legacy param (active, current, sub, tier, expires, email, `display_name`, host, role, billing,
/// model, uuid, capabilities, `org_uuid`, `org_name`) exits 1 and the error references `cols::`.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-13]
fn ft13_accounts_legacy_toggles_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  // "active" removed from this list — Feature 065 renamed it to assignee:: (REMOVED_TOGGLE; bfs Kind::String).
  let toggles = [
    "current", "sub", "tier", "expires", "email",
    "display_name", "host", "role", "billing", "model",
    "uuid", "capabilities", "org_uuid", "org_name",
  ];
  for toggle in toggles
  {
    let param = format!( "{toggle}::1" );
    let out   = run_cs_with_env( &[ ".accounts", &param ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 1 );
    let err = stderr( &out );
    assert!(
      err.contains( "cols::" ),
      "FT-13: '{toggle}::1' must reject with a cols:: migration message; got stderr:\n{err}",
    );
  }
}

#[ test ]
/// FT-14 (AC-14): `.accounts cols::+host,-tier` adds host column, removes tier from identity set.
///
/// After applying the modifier: Host line present, Tier line absent.
/// All other default identity columns (Owner, Sub, Expires, Email) still present.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-14]
fn ft14_accounts_cols_modifier()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "alice@acme.com", Some( "work-laptop" ), None );

  let out = run_cs_with_env(
    &[ ".accounts", "cols::+host,-tier" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(  text.contains( "Host:" ),  "FT-14: Host: must appear with cols::+host; got:\n{text}"        );
  assert!( !text.contains( "Tier:" ),  "FT-14: Tier: must be absent with cols::-tier; got:\n{text}"     );
  assert!(  text.contains( "Owner:" ), "FT-14: Owner: must remain (default-on); got:\n{text}"           );
  assert!(  text.contains( "Sub:" ),   "FT-14: Sub: must remain (default-on); got:\n{text}"             );
  assert!(  text.contains( "Expires:" ), "FT-14: Expires: must remain (default-on); got:\n{text}"       );
}

#[ test ]
/// FT-15 (AC-15, `lim_it`): `.accounts refresh::1` uses same fetch algorithm as `.usage`.
///
/// Requires live API access. With a valid token, timestamped ` · fetch` lines appear in stderr.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-15]
fn lim_it_ft15_accounts_refresh_live()
{
  require_live_api( "ft15" );
  let Some( token ) = live_active_token() else { return };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );

  let out = run_cs_with_env(
    &[ ".accounts", "refresh::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "live@test.com" ),
    "FT-15: refresh::1 with live token must show account; got stdout:\n{text}",
  );
}

#[ test ]
/// FT-19 (AC-19): Owner column visible by default; shows owner from `{name}.json`; `cols::-owner` hides it.
///
/// Case A: `.accounts` — Owner: present, alice shows owner identity, bob shows em dash (—).
/// Case B: `.accounts cols::-owner` — no Owner: line.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-19]
fn ft19_owner_column_default_visible()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@acme.com", "" );

  // Case A: default — Owner column present.
  {
    let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert!(
      text.contains( "Owner:" ),
      "FT-19A: Owner: must appear by default; got:\n{text}",
    );
    assert!(
      text.contains( "testuser@testmachine" ),
      "FT-19A: alice's owner must appear; got:\n{text}",
    );
    assert!(
      text.contains( "\u{2014}" ),
      "FT-19A: bob's empty owner must show em dash (—); got:\n{text}",
    );
  }

  // Case B: cols::-owner — Owner column hidden.
  {
    let out  = run_cs_with_env( &[ ".accounts", "cols::-owner" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert!(
      !text.contains( "Owner:" ),
      "FT-19B: Owner: must be hidden with cols::-owner; got:\n{text}",
    );
  }
}

#[ test ]
/// FT-20 (AC-20): `force::1` bypasses G8 gate — unclaims even when caller ≠ stored owner.
///
/// alice is owned by "other@remote"; caller identity is "local@local" (G8 would fail without force).
/// With `force::1`: exits 0, alice.json has `"owner": ""`.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-20]
fn ft20_accounts_unclaim_force_bypasses_g8()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  // Without force: G8 blocks.
  let out_blocked = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
  );
  assert_exit( &out_blocked, 1 );

  // With force::1: G8 bypassed.
  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "name::alice@acme.com", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "local" ) ],
  );
  assert_exit( &out, 0 );

  let store    = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta     = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "",
    "FT-20: force::1 must clear owner regardless of caller identity",
  );
}

#[ test ]
/// FT-21 (AC-21): `force::1` without `unclaim::1` is silently ignored — no error.
///
/// Case A: `.accounts force::1` (no mutation) → exits 0, lists accounts normally.
/// Case B: `.accounts force::1 assign::1 name::alice` → exits 0, marker written; force is a no-op on assign.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-21]
fn ft21_force_no_effect_without_unclaim()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  // Case A: force alone → normal list.
  {
    let out = run_cs_with_env(
      &[ ".accounts", "force::1" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert!(
      text.contains( "alice@acme.com" ),
      "FT-21A: force::1 alone must not suppress output; got:\n{text}",
    );
  }

  // Case B: force + assignee:: → marker written, no error (force is silently ignored on assignee::).
  {
    let out = run_cs_with_env(
      &[ ".accounts", "force::1", "assignee::testuser@testmachine", "name::alice@acme.com" ],
      &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
    );
    assert_exit( &out, 0 );

    let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
    let marker = std::fs::read_to_string( store.join( "_active_testmachine_testuser" ) )
      .expect( "FT-21B: marker must be written with force::1 + assignee::testuser@testmachine" );
    assert_eq!( marker.trim(), "alice@acme.com", "FT-21B: marker must contain alice@acme.com" );
  }
}

#[ test ]
/// IT: `unclaim::1 force::1` batch (no `name::`) clears ALL accounts with a non-empty owner,
/// including those owned by a different identity.
///
/// `force::1` bypasses the G8 per-account skip logic in the batch loop:
/// ```
/// if !force && !is_owned(&owner) { skip; continue; }
/// ```
/// With `force::1` the condition short-circuits — non-owned accounts are NOT skipped.
///
/// Setup: alice (owned by current = `testuser@testmachine`), bob (owned by `other@remote`),
/// carol (unowned, empty owner — skipped because `owner.is_empty()`).
///
/// Expected: alice + bob both unclaimed (`owner: ""`); carol unchanged; exit 0.
fn it_batch_unclaim_force_clears_non_owned()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@acme.com", "other@remote" );

  write_account( dir.path(), "carol@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  // carol: no owner written → empty owner → not touched by unclaim

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "force::1" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "unclaimed alice@acme.com" ),
    "it_batch_unclaim_force: alice (self-owned) must be unclaimed; got:\n{text}",
  );
  assert!(
    text.contains( "unclaimed bob@acme.com" ),
    "it_batch_unclaim_force: bob (other-owned) must be unclaimed when force::1; got:\n{text}",
  );
  assert!(
    !text.contains( "carol" ),
    "it_batch_unclaim_force: carol (no .json) must not appear in output; got:\n{text}",
  );

  let store    = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let alice_meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let alice_val : serde_json::Value = serde_json::from_str( &alice_meta ).unwrap();
  assert_eq!(
    alice_val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "",
    "it_batch_unclaim_force: alice.json owner must be cleared",
  );

  let bob_meta = std::fs::read_to_string( store.join( "bob@acme.com.json" ) ).unwrap();
  let bob_val : serde_json::Value = serde_json::from_str( &bob_meta ).unwrap();
  assert_eq!(
    bob_val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "",
    "it_batch_unclaim_force: bob.json owner must be cleared (force bypasses G8)",
  );
}

#[ test ]
/// IT: `unclaim::1 force::1 dry::1` batch (no `name::`) previews without writing.
///
/// Same 3-account setup as `it_batch_unclaim_force_clears_non_owned`.
/// With `dry::1`, the unclaim loop prints `[dry-run] would unclaim <name>` for each
/// non-empty-owner account (alice + bob) and exits 0 — no writes occur.
fn it_batch_unclaim_force_dry_previews_all()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@acme.com", "other@remote" );

  write_account( dir.path(), "carol@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "owner::0", "force::1", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ) && text.contains( "alice@acme.com" ),
    "it_batch_unclaim_force_dry: alice must appear in dry-run output; got:\n{text}",
  );
  assert!(
    text.contains( "[dry-run]" ) && text.contains( "bob@acme.com" ),
    "it_batch_unclaim_force_dry: bob must appear in dry-run output (force bypasses G8); got:\n{text}",
  );

  // Verify no writes occurred — both owners must be unchanged.
  let store    = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let alice_meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let alice_val : serde_json::Value = serde_json::from_str( &alice_meta ).unwrap();
  assert_eq!(
    alice_val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "testuser@testmachine",
    "it_batch_unclaim_force_dry: alice.json owner must NOT be cleared in dry mode",
  );

  let bob_meta = std::fs::read_to_string( store.join( "bob@acme.com.json" ) ).unwrap();
  let bob_val : serde_json::Value = serde_json::from_str( &bob_meta ).unwrap();
  assert_eq!(
    bob_val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "other@remote",
    "it_batch_unclaim_force_dry: bob.json owner must NOT be cleared in dry mode",
  );
}

// ── mre_324: Account struct field alignment (TSK-324) ────────────────────────

/// `mre_324_a` — `cols::+role` shows user-defined label from `{name}.json` `role` field.
///
/// After TSK-324, `Account.role` holds the user-defined label from `{name}.json`
/// top-level `"role"` key (previously `profile_role`). `Account.org_role` holds
/// the Roles API value. `cols::+role` must show the user label, not the org role.
///
/// Spec: [`docs/feature/003_account_list.md` AC-10]
#[ test ]
fn mre_324_role_toggle_shows_user_label()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "test@example.com", None, Some( "work" ) );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+role" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Role:    work" ),
    "cols::+role must show user-defined label from {{name}}.json role field, got:\n{text}",
  );
}

// ── mre_324_b ─────────────────────────────────────────────────────────────────

/// `mre_324_b` — `cols::+host,+role` both show `N/A` when no `{name}.json` exists.
///
/// When no metadata snapshot is present, `Account.host` and `Account.role` are
/// both empty strings; the text renderer falls back to `N/A` for each.
///
/// Spec: [`docs/feature/003_account_list.md` AC-11]
#[ test ]
fn mre_324_host_role_na_when_metadata_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // No {name}.json written — host and role must degrade gracefully.

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+host,+role" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host:    N/A" ),
    "absent profile.json must show Host: N/A, got:\n{text}",
  );
  assert!(
    text.contains( "Role:    N/A" ),
    "absent profile.json must show Role: N/A, got:\n{text}",
  );
}

// ── mre_324_c ─────────────────────────────────────────────────────────────────

/// `mre_324_c` — `format::json` emits AC-12 canonical key set; no legacy keys.
///
/// After TSK-324, JSON output must include `"organization_role"`, `"host"`,
/// `"owner"`, `"is_owned"`, `"renewal_at"` and must NOT include the removed
/// `"profile_host"` or `"profile_role"` keys.
///
/// Spec: [`docs/feature/003_account_list.md` AC-12]
#[ test ]
fn mre_324_json_output_keys()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "test@example.com", Some( "mybox" ), Some( "work" ) );
  write_account_owner( dir.path(), "test@example.com", "testuser@testmachine" );
  write_account_renewal_json( dir.path(), "test@example.com", "2026-08-01T00:00:00Z" );

  let out  = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"organization_role\"" ), "JSON must include organization_role key, got:\n{text}" );
  assert!( text.contains( "\"host\""              ), "JSON must include host key, got:\n{text}"              );
  assert!( text.contains( "\"owner\""             ), "JSON must include owner key, got:\n{text}"             );
  assert!( text.contains( "\"is_owned\""          ), "JSON must include is_owned key, got:\n{text}"          );
  assert!( text.contains( "\"renewal_at\""        ), "JSON must include renewal_at key, got:\n{text}"        );
  assert!( !text.contains( "\"profile_host\""     ), "JSON must NOT include profile_host key, got:\n{text}"  );
  assert!( !text.contains( "\"profile_role\""     ), "JSON must NOT include profile_role key, got:\n{text}"  );
}

// ── mre_324_d ─────────────────────────────────────────────────────────────────

/// `mre_324_d` — `format::json` emits correct `owner` and `is_owned` VALUES per account.
///
/// AC-20: Account A with `owner` matching current identity → `is_owned: true`;
/// Account B with no owner field → `owner: ""` and `is_owned: true` (unowned = all-owned).
///
/// Spec: [`tests/docs/feature/03_account_list.md` FT-20]
#[ test ]
fn mre_324_json_owner_is_owned_values()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Account A: owned by testuser@testmachine (matches identity set via env vars below)
  write_account( dir.path(), "alice@acme.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  // Account B: no owner field → unowned = owned by all
  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let arr : serde_json::Value = serde_json::from_str( &text )
    .expect( "format::json must produce valid JSON" );
  let arr = arr.as_array().expect( "JSON root must be an array" );

  let alice = arr.iter()
    .find( |v| v[ "name" ].as_str() == Some( "alice@acme.com" ) )
    .expect( "alice@acme.com must appear in JSON output" );
  assert_eq!(
    alice[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "testuser@testmachine",
    "FT-20: alice owner value must be 'testuser@testmachine'; got:\n{text}",
  );
  assert!(
    alice[ "is_owned" ].as_bool().unwrap_or( false ),
    "FT-20: alice is_owned must be true (owner matches identity); got:\n{text}",
  );

  let bob = arr.iter()
    .find( |v| v[ "name" ].as_str() == Some( "bob@acme.com" ) )
    .expect( "bob@acme.com must appear in JSON output" );
  assert_eq!(
    bob[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "",
    "FT-20: bob owner must be empty string (no owner field); got:\n{text}",
  );
  assert!(
    bob[ "is_owned" ].as_bool().unwrap_or( false ),
    "FT-20: bob is_owned must be true (unowned = owned by all); got:\n{text}",
  );
}

// ── mre_324_e ─────────────────────────────────────────────────────────────────

/// `mre_324_e` — `format::json` emits correct `renewal_at` VALUE; `null` when absent.
///
/// AC-21: Account A with `_renewal_at` set → `renewal_at: "<iso>"`;
/// Account B with no `_renewal_at` → `renewal_at: null`.
///
/// Spec: [`tests/docs/feature/03_account_list.md` FT-21]
#[ test ]
fn mre_324_json_renewal_at_values()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Account A: has a renewal override date
  write_account( dir.path(), "alice@acme.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_renewal_json( dir.path(), "alice@acme.com", "2025-08-01T00:00:00Z" );

  // Account B: no _renewal_at field → renewal_at must be null in JSON output
  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let arr : serde_json::Value = serde_json::from_str( &text )
    .expect( "format::json must produce valid JSON" );
  let arr = arr.as_array().expect( "JSON root must be an array" );

  let alice = arr.iter()
    .find( |v| v[ "name" ].as_str() == Some( "alice@acme.com" ) )
    .expect( "alice@acme.com must appear in JSON output" );
  assert_eq!(
    alice[ "renewal_at" ].as_str().unwrap_or( "MISSING" ),
    "2025-08-01T00:00:00Z",
    "FT-21: alice renewal_at must be '2025-08-01T00:00:00Z'; got:\n{text}",
  );

  let bob = arr.iter()
    .find( |v| v[ "name" ].as_str() == Some( "bob@acme.com" ) )
    .expect( "bob@acme.com must appear in JSON output" );
  assert!(
    bob[ "renewal_at" ].is_null(),
    "FT-21: bob renewal_at must be null when _renewal_at absent; got:\n{text}",
  );
}

// ── mre_324_f ─────────────────────────────────────────────────────────────────

/// `mre_324_f` — `format::json` emits `is_owned: false` when owner is a foreign identity.
///
/// AC-20 covers three states: empty owner (all-owned), matching owner (this machine owns),
/// and foreign owner (different machine owns). Only the third yields `is_owned: false`.
/// This test exercises that branch by writing owner "other@remote" and running as a
/// different identity "local@localmachine".
///
/// Spec: [`docs/feature/003_account_list.md` AC-20]
#[ test ]
fn mre_324_json_is_owned_false_for_foreign_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Account owned by "other@remote"; we run as USER=local, HOSTNAME=localmachine → mismatch.
  write_account( dir.path(), "alice@acme.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ), ( "USER", "local" ), ( "HOSTNAME", "localmachine" ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let arr : serde_json::Value = serde_json::from_str( &text )
    .expect( "format::json must produce valid JSON" );
  let arr = arr.as_array().expect( "JSON root must be an array" );

  let alice = arr.iter()
    .find( |v| v[ "name" ].as_str() == Some( "alice@acme.com" ) )
    .expect( "alice@acme.com must appear in JSON output" );
  assert_eq!(
    alice[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "other@remote",
    "mre_324_f: owner field must be 'other@remote'; got:\n{text}",
  );
  assert!(
    !alice[ "is_owned" ].as_bool().unwrap_or( true ),
    "mre_324_f: is_owned must be false when owner is a foreign identity; got:\n{text}",
  );
}

// ── mre_324_g ─────────────────────────────────────────────────────────────────

/// `mre_324_g` — `format::json` emits correct VALUES for `host`, `role`, `organization_role`.
///
/// `mre_324_c` verified key presence; this test verifies that each field carries
/// the correct value from its data source:
/// - `"role"` → user-defined label from `{name}.json` `"role"` key (via `write_account_profile_json`)
/// - `"host"` → host label from `{name}.json` `"host"` key (via `write_account_profile_json`)
/// - `"organization_role"` → org role from `{name}.json` `"organization_role"` key (via `write_account_roles_json`)
///
/// Spec: [`docs/feature/003_account_list.md` AC-12]
#[ test ]
fn mre_324_json_host_role_org_role_values()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // User-defined role label and host from profile fields:
  write_account_profile_json( dir.path(), "test@example.com", Some( "work-laptop" ), Some( "developer" ) );
  // Org role from roles.json (organization_role key, distinct from the user role label):
  write_account_roles_json( dir.path(), "test@example.com", "uuid-123", "Acme Corp", "admin" );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let arr : serde_json::Value = serde_json::from_str( &text )
    .expect( "format::json must produce valid JSON" );
  let arr = arr.as_array().expect( "JSON root must be an array" );

  let acct = arr.iter()
    .find( |v| v[ "name" ].as_str() == Some( "test@example.com" ) )
    .expect( "test@example.com must appear in JSON output" );

  assert_eq!(
    acct[ "host" ].as_str().unwrap_or( "MISSING" ),
    "work-laptop",
    "mre_324_g: host must be 'work-laptop' from profile host field; got:\n{text}",
  );
  assert_eq!(
    acct[ "role" ].as_str().unwrap_or( "MISSING" ),
    "developer",
    "mre_324_g: role must be 'developer' (user-defined label from role field); got:\n{text}",
  );
  assert_eq!(
    acct[ "organization_role" ].as_str().unwrap_or( "MISSING" ),
    "admin",
    "mre_324_g: organization_role must be 'admin' from organization_role field; got:\n{text}",
  );
}
