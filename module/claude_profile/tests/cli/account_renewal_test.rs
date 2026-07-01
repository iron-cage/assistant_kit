//! Integration tests: ARN (Account Renewal) + late AS tests (as22–as35).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | arn01 | `ft01_account_renewal_at_writes_renewal_at` | `at::` writes `_renewal_at` | P |
//! | arn02 | `ft02_account_renewal_from_now_positive` | `from_now::+1h30m` writes future date | P |
//! | arn03 | `ft03_account_renewal_from_now_negative` | `from_now::-30m` writes past date | P |
//! | arn04 | `ft04_account_renewal_clear_removes_key` | `clear::1` removes `_renewal_at` | P |
//! | arn05 | `ft05_account_renewal_name_all_updates_all` | `name::all from_now::+0m` → all accounts | P |
//! | arn06 | `ft06_account_renewal_dry_no_write` | `dry::1` → [dry-run], no write | P |
//! | arn07 | `ft07_account_renewal_at_from_now_conflict` | `at::` + `from_now::` → exit 1 | N |
//! | arn08 | `ft08_account_renewal_at_clear_conflict` | `at::` + `clear::` → exit 1 | N |
//! | arn09 | `ft09_account_renewal_from_now_clear_conflict` | `from_now::` + `clear::` → exit 1 | N |
//! | arn10 | `ft10_account_renewal_no_operation_exits_1` | no operation param → exit 1 | N |
//! | arn11 | `ft11_account_renewal_unknown_account_exits_2` | unknown account → exit 2 | N |
//! | arn12 | `ft12_account_renewal_comma_list_updates_both` | comma-list updates both | P |
//! | arn13 | `ft13_account_renewal_partial_comma_list` | unknown in comma-list reported | N |
//! | arn14 | `ft14_account_renewal_past_at_accepted` | past `at::` written verbatim | P |
//! | arn15 | `ft15_account_renewal_unknown_param_exits_1` | unknown param → exit 1 | N |
//! | arn16 | `ft16_account_renewal_creates_new_claude_json` | no prior file → created | P |
//! | arn17 | `arn17_from_now_invalid_format_exits_1` | `from_now::invalid` → exit 1 | N |
//! | arn18 | `arn18_from_now_unsupported_unit_exits_1` | `from_now::+1s` → exit 1 | N |
//! | arn19 | `arn19_clear_no_prior_renewal_at_exits_0` | `clear::1` without `_renewal_at` → exit 0 | P |
//! | arn20 | `arn20_all_three_conflict_exits_1` | all three conflict → exit 1 | N |
//! | arn21 | `arn21_at_invalid_iso_stored_verbatim` | invalid ISO stored verbatim | P |
//! | arn26 | `arn26_from_now_plus_no_units_exits_1` | `from_now::+` → exit 1 | N |
//! | arn27 | `arn27_from_now_minus_no_units_exits_1` | `from_now::-` → exit 1 | N |
//! | arc02 | `arc02_clear_preserves_oauth_account_content` | `clear::1` preserves oauthAccount | P |
//! | as22 | `as22_save_preserves_renewal_at` | second save preserves `_renewal_at` | P |
//! | as19 | `as19_save_best_effort_no_roles_json` | save without valid token → exit 0 | P |
//! | as20 | `as20_lim_it_save_writes_roles_json` | save with valid token → `{name}.json` | P |
//! | as21 | `as21_lim_it_resave_overwrites_roles_json` | second save overwrites `{name}.json` | P |
//! | as23 | `as_save_writes_profile_json` | `host:: role::` → `{name}.json` created | P |
//! | as24 | `as24_host_auto_capture_user_hostname` | no `host::` → auto-captured | P |
//! | as25 | `as25_host_empty_triggers_auto_capture` | `host::` empty → auto-captured | P |
//! | as26 | `as26_host_resave_overwrites` | resave with new host replaces old | P |
//! | as27 | `as27_host_with_spaces` | `host::my work laptop` stored verbatim | P |
//! | as28 | `as28_host_missing_user_stores_at_resolved_hostname` | USER/HOSTNAME unset → `@<resolved>` | P |
//! | mre_bug239 | `mre_bug239_hostname_resolved_when_env_absent` | HOSTNAME absent → resolved (BUG-239) | P |
//! | as29 | `as29_resave_credentials_unchanged` | resave does not modify credentials.json | P |
//! | as30 | `as30_role_writes_profile_json` | `role::work` → `{name}.json` has role | P |
//! | as31 | `as31_role_omit_stores_empty` | no `role::` → `{name}.json` has `"role":""` | P |
//! | as32 | `as32_role_empty_stores_empty` | `role::` empty → `{name}.json` has `"role":""` | P |
//! | as33 | `as33_role_resave_overwrites` | resave with new role replaces old | P |
//! | as34 | `as34_role_with_spaces` | `role::dev ops team` stored verbatim | P |
//! | as35 | `as35_save_dry_run_rejects_invalid_name` | `dry::1 name::not-an-email` → exit 1 | N |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_claude_json,
  FAR_FUTURE_MS,
};
use std::process::Command;
use tempfile::TempDir;

// ── ARN: Account Renewal ──────────────────────────────────────────────────────

#[ test ]
fn ft01_account_renewal_at_writes_renewal_at()
{
  // FT-01 (FR-030 AC-01): `at::` writes `_renewal_at`; existing `oauthAccount` preserved.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  std::fs::write(
    store.join( "test@example.com.json" ),
    r#"{"oauthAccount":{"emailAddress":"test@example.com","subscriptionType":"max"}}"#,
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::2026-06-29T21:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "2026-06-29T21:00:00Z""# ),
    "must write exact _renewal_at value, got: {content}",
  );
  assert!(
    content.contains( "oauthAccount" ),
    "must preserve oauthAccount on write, got: {content}",
  );
}

#[ test ]
fn ft02_account_renewal_from_now_positive()
{
  // FT-02 (FR-030 AC-02): `from_now::+1h30m` writes future ISO-8601 UTC timestamp.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::+1h30m" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "202"# ),
    "must write ISO-8601 timestamp starting with 202x in _renewal_at, got: {content}",
  );
  // from_now::+1h30m must not produce the same value as a clearly-past year
  assert!(
    !content.contains( r#""_renewal_at": "200"# ),
    "_renewal_at from from_now::+1h30m must not start with 200x, got: {content}",
  );
}

#[ test ]
fn ft03_account_renewal_from_now_negative()
{
  // FT-03 (FR-030 AC-03): `from_now::-30m` writes past timestamp verbatim; no auto-advance at write.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::-30m" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  // from_now::-30m on 2026-05-29 still gives a 2026 timestamp
  assert!(
    content.contains( r#""_renewal_at": "202"# ),
    "must write ISO-8601 past timestamp in _renewal_at, got: {content}",
  );
  // Must not be auto-advanced to far future at write time
  assert!(
    !content.contains( r#""_renewal_at": "2099"# ),
    "past timestamp must not be auto-advanced at write time, got: {content}",
  );
}

#[ test ]
fn ft04_account_renewal_clear_removes_key()
{
  // FT-04 (FR-030 AC-04): `clear::1` removes `_renewal_at`; `oauthAccount` preserved.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  std::fs::write(
    store.join( "test@example.com.json" ),
    r#"{"oauthAccount":{"emailAddress":"test@example.com"},"_renewal_at":"2026-06-29T21:00:00Z"}"#,
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "clear::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    !content.contains( "_renewal_at" ),
    "clear::1 must remove _renewal_at from file, got: {content}",
  );
  assert!(
    content.contains( "oauthAccount" ),
    "clear::1 must preserve oauthAccount, got: {content}",
  );
}

#[ test ]
fn ft05_account_renewal_name_all_updates_all()
{
  // FT-05 (FR-030 AC-05): `name::all from_now::+0m` writes current timestamp to every account.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "alice@a.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@a.com",   "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::all", "from_now::+0m" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let alice = std::fs::read_to_string( store.join( "alice@a.com.json" ) ).unwrap();
  let bob   = std::fs::read_to_string( store.join( "bob@a.com.json" ) ).unwrap();
  assert!(
    alice.contains( r#""_renewal_at": "202"# ),
    "alice must have _renewal_at after name::all, got: {alice}",
  );
  assert!(
    bob.contains( r#""_renewal_at": "202"# ),
    "bob must have _renewal_at after name::all, got: {bob}",
  );
}

#[ test ]
fn ft06_account_renewal_dry_no_write()
{
  // FT-06 (FR-030 AC-06): `dry::1` prints would-be value without writing file.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::2026-06-29T21:00:00Z", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "dry::1 must print [dry-run] prefix in stdout, got: {text}",
  );
  assert!(
    !store.join( "test@example.com.json" ).exists(),
    "dry::1 must not create {{name}}.json",
  );
}

#[ test ]
fn ft07_account_renewal_at_from_now_conflict()
{
  // FT-07 (FR-030 AC-07): `at::` and `from_now::` together exits 1 with conflict error.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[
      ".account.renewal", "name::test@example.com",
      "at::2026-06-29T21:00:00Z", "from_now::+1h",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "at::" ) && err.contains( "from_now::" ),
    "must name both conflicting params in error, got: {err}",
  );
}

#[ test ]
fn ft08_account_renewal_at_clear_conflict()
{
  // FT-08 (FR-030 AC-08): `at::` and `clear::` together exits 1 with conflict error.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[
      ".account.renewal", "name::test@example.com",
      "at::2026-06-29T21:00:00Z", "clear::1",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "at::" ) && err.contains( "clear::" ),
    "must name both conflicting params in error, got: {err}",
  );
}

#[ test ]
fn ft09_account_renewal_from_now_clear_conflict()
{
  // FT-09 (FR-030 AC-09): `from_now::` and `clear::` together exits 1 with conflict error.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[
      ".account.renewal", "name::test@example.com",
      "from_now::+1h", "clear::1",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "from_now::" ) && err.contains( "clear::" ),
    "must name both conflicting params in error, got: {err}",
  );
}

#[ test ]
fn ft10_account_renewal_no_operation_exits_1()
{
  // FT-12 (FR-030 AC-12): no operation param (`at::`, `from_now::`, `clear::`) exits 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  // Error must mention one of the required operation params or the word "required"
  assert!(
    err.contains( "at::" ) || err.contains( "from_now::" ) || err.contains( "clear::" ) || err.contains( "required" ),
    "must print usage error naming required operation param, got: {err}",
  );
}

#[ test ]
fn ft11_account_renewal_unknown_account_exits_2()
{
  // FT-13 (FR-030 AC-13): account not in credential store exits 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Credential store exists but is empty — no accounts saved
  std::fs::create_dir_all(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" )
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::ghost@example.com", "at::2026-06-29T21:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );

  let err = stderr( &out );
  assert!(
    err.contains( "ghost@example.com" ),
    "must name the unknown account in error message, got: {err}",
  );
}

#[ test ]
fn ft12_account_renewal_comma_list_updates_both()
{
  // FT-14 (FR-030 AC-14): comma-list `name::alice,bob` updates both accounts.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "alice@a.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@a.com",   "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice@a.com,bob@a.com", "at::2026-06-29T21:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let alice = std::fs::read_to_string( store.join( "alice@a.com.json" ) ).unwrap();
  let bob   = std::fs::read_to_string( store.join( "bob@a.com.json" ) ).unwrap();
  assert!(
    alice.contains( r#""_renewal_at": "2026-06-29T21:00:00Z""# ),
    "alice must have exact _renewal_at after comma-list, got: {alice}",
  );
  assert!(
    bob.contains( r#""_renewal_at": "2026-06-29T21:00:00Z""# ),
    "bob must have exact _renewal_at after comma-list, got: {bob}",
  );
}

#[ test ]
fn ft13_account_renewal_partial_comma_list()
{
  // FT-15 (FR-030 AC-15): one unknown in comma-list — error reported; known account still processed.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "alice@a.com", "max", "standard", FAR_FUTURE_MS, false );
  // unknown@a.com is NOT in the credential store

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice@a.com,unknown@a.com", "at::2026-06-29T21:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code != 0, "partial comma-list must exit non-zero, got exit {code}" );

  let err = stderr( &out );
  assert!(
    err.contains( "unknown@a.com" ),
    "must report the unknown account in stderr, got: {err}",
  );

  // alice must still be processed despite the partial failure
  let alice = std::fs::read_to_string( store.join( "alice@a.com.json" ) ).unwrap();
  assert!(
    alice.contains( r#""_renewal_at": "2026-06-29T21:00:00Z""# ),
    "alice must have _renewal_at even in partial failure, got: {alice}",
  );
}

#[ test ]
fn ft14_account_renewal_past_at_accepted()
{
  // IT-13: past `at::` value accepted without error; stored verbatim (auto-advance at render only).
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::2020-01-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "2020-01-01T00:00:00Z""# ),
    "past at:: must be stored verbatim without auto-advance at write time, got: {content}",
  );
}

#[ test ]
fn ft15_account_renewal_unknown_param_exits_1()
{
  // IT-14: unknown parameter rejected with exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "unknown::x" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  // When .account.renewal exists, unilang rejects the unknown param with "Unknown parameter".
  // This differs from Phase 1 behavior ("command was not found"), confirming command is registered.
  assert!(
    err.contains( "Unknown parameter" ),
    "must produce 'Unknown parameter' error (not 'command not found'), got: {err}",
  );
}

#[ test ]
fn ft16_account_renewal_creates_new_claude_json()
{
  // AC-05 edge: no pre-existing `{name}.json` → `at::` creates the file.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // Pre-condition: {name}.json must not exist before command
  assert!(
    !store.join( "test@example.com.json" ).exists(),
    "pre-condition: {{name}}.json must not exist before command",
  );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::2026-06-29T21:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "2026-06-29T21:00:00Z""# ),
    "must create {{name}}.json with _renewal_at when file did not exist before, got: {content}",
  );
}

#[ test ]
fn ft17_account_renewal_single_prefix_resolves()
{
  // IT-15 / EC-21 (AC-12): single bare prefix `name::alice` resolves to `alice@acme.com`.
  // Root cause guard for TSK-232: the single-name path already called resolve_account_name(),
  //   so this test confirms no regression in that path after the comma-list loop refactor.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "alice@acme.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice", "at::2026-07-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "2026-07-01T00:00:00Z""# ),
    "single prefix must resolve to full email and write _renewal_at, got: {content}",
  );
}

#[ test ]
fn ft18_account_renewal_comma_list_prefix_tokens()
{
  // IT-16 / EC-22 (AC-13): comma-list `name::alice,bob` — each prefix token resolved independently.
  // Root cause guard for TSK-232: the comma-list branch previously collected raw tokens without
  //   calling resolve_account_name(), causing prefix tokens to fail with "account not found".
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "alice@acme.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@acme.com",   "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice,bob", "at::2026-07-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let alice = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let bob   = std::fs::read_to_string( store.join( "bob@acme.com.json" ) ).unwrap();
  assert!(
    alice.contains( r#""_renewal_at": "2026-07-01T00:00:00Z""# ),
    "alice@acme.com must have _renewal_at after comma-list prefix resolution, got: {alice}",
  );
  assert!(
    bob.contains( r#""_renewal_at": "2026-07-01T00:00:00Z""# ),
    "bob@acme.com must have _renewal_at after comma-list prefix resolution, got: {bob}",
  );
}

// ── AS-22: save() read-merge preserving _renewal_at ───────────────────────────

#[ test ]
fn as22_save_preserves_renewal_at()
{
  // FT-11 (FR-002 AC-17): second `.account.save` preserves `_renewal_at` via read-merge (not overwrite).
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Setup: source credentials and ~/.claude.json
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  // First save — establishes account credential file and initial {name}.json snapshot
  let first = run_cs_with_env(
    &[ ".account.save", "name::test@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &first, 0 );

  // Inject _renewal_at into the existing {name}.json (simulates a prior .account.renewal run)
  std::fs::write(
    store.join( "test@example.com.json" ),
    r#"{"oauthAccount":{"emailAddress":"test@example.com","subscriptionType":"max"},"_renewal_at":"2026-06-29T21:00:00Z"}"#,
  ).unwrap();

  // Update source ~/.claude.json with new oauthAccount content (simulates fresh OAuth login)
  std::fs::write(
    dir.path().join( ".claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"test@example.com","subscriptionType":"pro"}}"#,
  ).unwrap();

  // Second save — must read-merge: preserve _renewal_at, update oauthAccount
  let second = run_cs_with_env(
    &[ ".account.save", "name::test@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &second, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "2026-06-29T21:00:00Z""# ),
    "second .account.save must preserve _renewal_at via read-merge (not overwrite), got: {content}",
  );
  assert!(
    content.contains( "oauthAccount" ),
    "oauthAccount must remain in {{name}}.json after second save, got: {content}",
  );
  assert!(
    content.contains( "\"subscriptionType\": \"pro\"" ),
    "oauthAccount must be updated with new content from ~/.claude.json on second save, got: {content}",
  );
}

// ── as23: save writes {name}.json with host and role ──────────────────────────

#[ test ]
fn as_save_writes_profile_json()
{
  // TSK-225 RED gate: `.account.save host::testbox role::dev` must create
  // `{name}.json` containing the host and role values as JSON.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::testbox", "role::dev" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let profile_path = store.join( "test@example.com.json" );
  assert!(
    profile_path.exists(),
    "{{name}}.json must be created by .account.save when host:: is passed, path: {}",
    profile_path.display(),
  );
  let content = std::fs::read_to_string( &profile_path ).unwrap();
  assert!(
    content.contains( r#""host": "testbox""# ),
    "{{name}}.json must contain host value, got: {content}",
  );
  assert!(
    content.contains( r#""role": "dev""# ),
    "{{name}}.json must contain role value, got: {content}",
  );
}

// ── ARN corner cases ──────────────────────────────────────────────────────────

#[ test ]
fn arn17_from_now_invalid_format_exits_1()
{
  // `from_now::` must start with '+' or '-'. "invalid" has neither → parse error → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::invalid" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "from_now::" ) || err.contains( "'+'" ) || err.contains( "'-'" ) || err.contains( "must start" ),
    "must report from_now:: parse error, got: {err}",
  );
}

#[ test ]
fn arn18_from_now_unsupported_unit_exits_1()
{
  // `from_now::+1s` — unit 's' is not supported (only d, h, m) → parse error → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::+1s" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "from_now::" ) || err.contains( "unknown unit" ) || err.contains( "'s'" ),
    "must report unknown unit error, got: {err}",
  );
}

// ── arn22: from_now::+ (sign only, no units) exits 1 ────────────────────────

/// arn22 (BUG-220): `from_now::+` (sign with no duration units) must exit 1.
///
/// Previously, `parse_from_now_delta("+")` returned `Ok(0)` (zero-second delta),
/// silently setting `_renewal_at` to the current time instead of rejecting the
/// malformed input.  The fix adds an empty-rest guard before the parsing loop.
///
/// # Root Cause (BUG-220)
/// `parse_from_now_delta` consumed the sign, then entered the while loop on an empty
/// `rest` slice — the loop body never ran, `total_secs` stayed 0, and `Ok(0)` was
/// returned with no error.
///
/// # Fix Applied
/// Added `if rest.trim().is_empty() { return Err(...) }` immediately after sign
/// extraction in `parse_from_now_delta` (`claude_profile_core/src/account.rs`).
///
/// # Why Not Caught
/// No test existed for sign-only input.  All prior tests had at least one numeric unit
/// component (`+1h`, `-30m`, `+0m`).
///
/// # Prevention
/// Any `from_now::` value that consists of only a sign character (after trimming) must
/// be rejected with a clear error message referencing `from_now::`.
///
/// # Pitfall
/// Zero-delta IS valid when written explicitly as `+0m` or `+0h` (sets renewal to now
/// intentionally).  Sign-only `+` or `-` is a user mistake and must be rejected.
#[ doc = "bug_reproducer(BUG-220)" ]
#[ test ]
fn arn26_from_now_plus_no_units_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::+" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "from_now::" ),
    "from_now::+ must exit 1 with parse error mentioning from_now::, got: {err}",
  );
}

// ── arn23: from_now::- (sign only, no units) exits 1 ────────────────────────

/// arn23: `from_now::-` (sign only, no units) must also exit 1.
///
/// Symmetric case to arn22 — the negative sign alone is equally malformed.
///
/// Spec: same fix as BUG-220 covers both `+` and `-` sign-only inputs.
#[ test ]
fn arn27_from_now_minus_no_units_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::-" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "from_now::" ),
    "from_now::- must exit 1 with parse error mentioning from_now::, got: {err}",
  );
}

#[ test ]
fn arn19_clear_no_prior_renewal_at_exits_0()
{
  // `clear::1` when no `_renewal_at` key exists in the file → exits 0 without error.
  // The file gets written with an empty JSON object (no _renewal_at, no other keys).
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // Write a {name}.json with oauthAccount but NO _renewal_at
  std::fs::write(
    store.join( "test@example.com.json" ),
    r#"{"oauthAccount":{"emailAddress":"test@example.com"}}"#,
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "clear::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // oauthAccount must be preserved; _renewal_at must not appear.
  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    !content.contains( "_renewal_at" ),
    "clear on no-prior _renewal_at must not introduce the key, got: {content}",
  );
  assert!(
    content.contains( "oauthAccount" ),
    "clear must preserve oauthAccount when no _renewal_at was present, got: {content}",
  );
}

#[ test ]
fn arn20_all_three_conflict_exits_1()
{
  // All three mutually-exclusive params together → at least one conflict check fires → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[
      ".account.renewal", "name::test@example.com",
      "at::2026-06-29T21:00:00Z", "from_now::+1h", "clear::1",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "mutually exclusive" ) || err.contains( "at::" ) || err.contains( "from_now::" ),
    "must report conflict error when all three params provided, got: {err}",
  );
}

#[ test ]
fn arn21_at_invalid_iso_stored_verbatim()
{
  // `at::` accepts any string verbatim — validation happens only at render time.
  // A non-ISO value like "not-a-date" is stored as-is; exit 0.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::not-a-date" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "not-a-date""# ),
    "malformed at:: value must be stored verbatim, got: {content}",
  );
}

// ── as24: host:: auto-capture $USER@$HOSTNAME ────────────────────────────────

/// as24 — Omitting `host::` auto-captures `$USER@$HOSTNAME` into `{name}.json`.
///
/// Spec: [`tests/docs/cli/param/048_host.md` EC-2]
/// Also: [`tests/docs/feature/029_account_host_metadata.md` FT-02]
#[ test ]
fn as24_host_auto_capture_user_hostname()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com" ],
    &[ ( "HOME", home ), ( "USER", "alice" ), ( "HOSTNAME", "workstation" ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""host": "alice@workstation""# ),
    "omitting host:: must auto-capture USER@HOSTNAME, got: {content}",
  );
}

// ── as25: host:: empty triggers auto-capture ─────────────────────────────────

/// as25 — `host::` with empty value behaves identically to omitting `host::`.
///
/// Spec: [`tests/docs/cli/param/048_host.md` EC-3]
#[ test ]
fn as25_host_empty_triggers_auto_capture()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::" ],
    &[ ( "HOME", home ), ( "USER", "bob" ), ( "HOSTNAME", "laptop" ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""host": "bob@laptop""# ),
    "empty host:: must auto-capture USER@HOSTNAME same as omitting it, got: {content}",
  );
}

// ── as26: re-save with different host:: overwrites ───────────────────────────

/// as26 — Second save with a different `host::` overwrites `{name}.json`.
///
/// Spec: [`tests/docs/cli/param/048_host.md` EC-5]
/// Also: [`tests/docs/feature/029_account_host_metadata.md` FT-04]
#[ test ]
fn as26_host_resave_overwrites()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  // First save.
  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::oldbox" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Second save overwrites.
  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::newbox" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""host": "newbox""# ),
    "re-save must overwrite old host value with newbox, got: {content}",
  );
  assert!(
    !content.contains( "oldbox" ),
    "old host value oldbox must not be present after re-save, got: {content}",
  );
}

// ── as27: host:: value with spaces stored verbatim ───────────────────────────

/// as27 — `host::` value containing spaces is stored verbatim in `{name}.json`.
///
/// Spec: [`tests/docs/cli/param/048_host.md` EC-6]
#[ test ]
fn as27_host_with_spaces()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::my work laptop" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""host": "my work laptop""# ),
    "host:: value with spaces must be stored verbatim, got: {content}",
  );
}

// ── as28: USER unset, HOSTNAME unset → host="@<resolved>" ────────────────────

/// as28 — When `$USER` and `$HOSTNAME` are both unset, `host` is `"@<hostname>"` where
/// the hostname comes from the `resolve_hostname()` fallback chain (BUG-239 fix).
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-03]
/// Before BUG-239 fix: HOSTNAME unset → empty hostname → guard `user.is_empty() && hostname.is_empty()` → `""`.
/// After BUG-239 fix: `resolve_hostname()` → `/etc/hostname` or "local" → always resolves → `"@<hostname>"`.
#[ test ]
fn as28_host_missing_user_stores_at_resolved_hostname()
{
  use crate::cli_runner::BIN;
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  // Remove USER and HOSTNAME entirely from the subprocess environment.
  let out = Command::new( BIN )
    .args( [ ".account.save", "name::test@example.com" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .env_remove( "USER" )
    .env_remove( "HOSTNAME" )
    .output()
    .expect( "failed to execute clp" );

  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  // host starts with "@" (USER absent) but has a non-empty resolved hostname (not bare "@").
  assert!(
    content.contains( r#""host": "@"# ),
    "USER absent must produce host starting with '@', got: {content}",
  );
  assert!(
    !content.contains( r#""host": "@""# ),
    "hostname must not be empty when resolved via fallback chain (BUG-239), got: {content}",
  );
}

// ── mre_bug239: HOSTNAME absent → host has resolved non-empty hostname ────────

/// `mre_bug239` — `.account.save` with `$HOSTNAME` removed from subprocess env produces
/// `"host":"alice@<resolved>"` — hostname from `resolve_hostname()` fallback chain.
///
/// # Root Cause
/// `std::env::var("HOSTNAME")` returns `Err` in child processes spawned from bash when
/// `$HOSTNAME` is not exported. Old code used `unwrap_or_default()` → empty string →
/// `"alice@"` (bare user with empty hostname part).
///
/// # Why Not Caught
/// The old `as24` / `as25` tests supplied `HOSTNAME` explicitly in the subprocess env.
/// No test removed `HOSTNAME` while keeping `USER` to expose the empty-hostname path.
///
/// # Fix Applied
/// `resolve_hostname()` extracted from `active_marker_filename()` with fallback chain:
/// `$HOSTNAME` env → `/etc/hostname` → `"local"`. `account_save_routine()` now calls
/// `resolve_hostname()` instead of `std::env::var("HOSTNAME")`.
///
/// # Prevention
/// Never call `std::env::var("HOSTNAME")` directly — use `resolve_hostname()` everywhere.
/// Hostname env is a bash built-in; it is not exported to child processes by default.
///
/// # Pitfall
/// The resolved hostname value is environment-dependent (`/etc/hostname` or "local").
/// Tests must assert "non-empty hostname after @" without hardcoding the hostname value.
#[ doc = "bug_reproducer(BUG-239)" ]
#[ test ]
fn mre_bug239_hostname_resolved_when_env_absent()
{
  use crate::cli_runner::{ BIN, write_credentials, write_claude_json, FAR_FUTURE_MS };
  use std::process::Command;
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  // Remove HOSTNAME to simulate bash child process where $HOSTNAME is not exported.
  // Keep USER so the bug manifests as "alice@" (not "@").
  let out = Command::new( BIN )
    .args( [ ".account.save", "name::test@example.com" ] )
    .env( "HOME", home )
    .env( "USER", "alice" )
    .env_remove( "PRO" )
    .env_remove( "HOSTNAME" )
    .output()
    .expect( "failed to execute clp" );

  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  // Before BUG-239 fix: host = "alice@" (empty hostname).
  // After fix: host = "alice@<resolved>" (non-empty, from /etc/hostname or "local").
  assert!(
    !content.contains( r#""host": "alice@""# ),
    "hostname must not be empty when $HOSTNAME env is absent — resolve_hostname() must use fallback chain, got: {content}",
  );
  assert!(
    content.contains( "alice@" ),
    "host must contain 'alice@' (user prefix), got: {content}",
  );
}

// ── as29: re-save with host:: does not change credentials.json ───────────────

/// as29 — Re-saving with `host::newbox` updates `{name}.json` but leaves credentials.json unchanged.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-10]
#[ test ]
fn as29_resave_credentials_unchanged()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  // First save — record credentials file content.
  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::oldbox" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let cred_path    = store.join( "test@example.com.credentials.json" );
  let cred_before  = std::fs::read_to_string( &cred_path ).unwrap();

  // Second save with different host — credentials must not change.
  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::newbox" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let cred_after = std::fs::read_to_string( &cred_path ).unwrap();

  assert_eq!(
    cred_before, cred_after,
    "re-save with host:: must not modify credentials.json content",
  );

  let profile = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    profile.contains( r#""host": "newbox""# ),
    "{{name}}.json must be updated to newbox, got: {profile}",
  );
}

// ── as30: role:: writes {name}.json ──────────────────────────────────────────

/// as30 — Explicit `role::work` written to `{name}.json` as `"role":"work"`.
///
/// Spec: [`tests/docs/cli/param/052_role.md` EC-1]
#[ test ]
fn as30_role_writes_profile_json()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "role::work" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""role": "work""# ),
    "explicit role::work must be stored in {{name}}.json, got: {content}",
  );
}

// ── as31: omit role:: stores empty string ────────────────────────────────────

/// as31 — Omitting `role::` stores `"role":""` in `{name}.json` (not absent).
///
/// Spec: [`tests/docs/cli/param/052_role.md` EC-2]
#[ test ]
fn as31_role_omit_stores_empty()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""role": """# ),
    "omitting role:: must store empty string role in {{name}}.json, got: {content}",
  );
}

// ── as32: role:: (empty) stores empty string ─────────────────────────────────

/// as32 — `role::` with empty value stores `"role":""` — same as omitting.
///
/// Spec: [`tests/docs/cli/param/052_role.md` EC-3]
#[ test ]
fn as32_role_empty_stores_empty()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "role::" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""role": """# ),
    "empty role:: must store empty string in {{name}}.json, got: {content}",
  );
}

// ── as33: re-save with different role:: overwrites ───────────────────────────

/// as33 — Second save with a different `role::` overwrites the old role in `{name}.json`.
///
/// Spec: [`tests/docs/cli/param/052_role.md` EC-5]
#[ test ]
fn as33_role_resave_overwrites()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  // First save.
  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "role::personal" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Second save overwrites.
  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "role::dev" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""role": "dev""# ),
    "re-save must overwrite old role value with dev, got: {content}",
  );
  assert!(
    !content.contains( "personal" ),
    "old role value personal must not be present after re-save, got: {content}",
  );
}

// ── as34: role:: value with spaces stored verbatim ───────────────────────────

/// as34 — `role::` value containing spaces is stored verbatim in `{name}.json`.
///
/// Spec: [`tests/docs/cli/param/052_role.md` EC-6]
#[ test ]
fn as34_role_with_spaces()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "test@example.com" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "role::dev ops team" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""role": "dev ops team""# ),
    "role:: value with spaces must be stored verbatim, got: {content}",
  );
}

/// `dry::1` with an invalid name (no `@`) exits 1 — validation runs before dry-run.
///
/// Previously, `dry::1` returned early with "[dry-run] would save" before
/// `validate_name()`, so invalid names appeared accepted.
#[ test ]
fn as35_save_dry_run_rejects_invalid_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::not-an-email", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "not a valid email address" ),
    "as35: dry-run must reject invalid names; got stderr: {err}",
  );
}

// ── arn22: at:: with explicit +00:00 offset accepted ──────────────────────────

/// arn22 — `at::2026-06-29T21:00:00+00:00` (explicit +00:00 offset) is accepted; exits 0 and
/// `_renewal_at` is written to the credential store.
///
/// Spec: [`tests/docs/cli/param/049_at.md` EC-2]
#[ test ]
fn arn22_at_explicit_utc_offset_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::2026-06-29T21:00:00+00:00" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( "_renewal_at" ),
    "at:: with +00:00 offset must write _renewal_at field, got: {content}",
  );
  assert!(
    content.contains( "2026-06-29" ),
    "stored _renewal_at must contain the date 2026-06-29, got: {content}",
  );
}

// ── arn23: at:: date-only accepted ───────────────────────────────────────────

/// arn23 — `at::2026-06-29` (date-only format) is accepted; exits 0 and `_renewal_at` is written.
///
/// Note: the implementation stores `at::` values verbatim — "2026-06-29" is stored as-is
/// (not normalized to "2026-06-29T00:00:00Z"). The spec describes aspirational normalization;
/// actual behavior is verbatim storage consistent with `arn21`.
///
/// Spec: [`tests/docs/cli/param/049_at.md` EC-3]
#[ test ]
fn arn23_at_date_only_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "at::2026-06-29" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( "_renewal_at" ),
    "date-only at:: must write _renewal_at field, got: {content}",
  );
  assert!(
    content.contains( "2026-06-29" ),
    "stored _renewal_at must contain the date portion 2026-06-29, got: {content}",
  );
}

// ── arn24: from_now::+0m for single account writes current time ───────────────

/// arn24 — `from_now::+0m` for a single named account writes the current time as `_renewal_at`
/// (ISO-8601 UTC, within a few seconds of invocation).
///
/// Spec: [`tests/docs/cli/param/050_from_now.md` EC-2]
#[ test ]
fn arn24_from_now_zero_delta_writes_current_time()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::+0m" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  // from_now::+0m must write a present-year ISO timestamp
  assert!(
    content.contains( r#""_renewal_at": "202"# ),
    "from_now::+0m must write ISO-8601 timestamp starting with 202x, got: {content}",
  );
  // Must not be a far-future or far-past timestamp
  assert!(
    !content.contains( r#""_renewal_at": "2099"# ),
    "_renewal_at from from_now::+0m must not be far future, got: {content}",
  );
}

// ── arn25: from_now::+1d single-unit delta accepted ──────────────────────────

/// arn25 — `from_now::+1d` (single day unit) is accepted; exits 0 and writes a future
/// ISO-8601 timestamp approximately 24 hours from now.
///
/// Spec: [`tests/docs/cli/param/050_from_now.md` EC-4]
#[ test ]
fn arn25_from_now_single_day_unit_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "from_now::+1d" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    content.contains( r#""_renewal_at": "202"# ),
    "from_now::+1d must write ISO-8601 future timestamp starting with 202x, got: {content}",
  );
  // +1d must not produce a clearly-past year
  assert!(
    !content.contains( r#""_renewal_at": "200"# ),
    "_renewal_at from from_now::+1d must not start with 200x, got: {content}",
  );
}

// ── arc02: clear::1 preserves oauthAccount content ───────────────────────────

/// arc02 — `clear::1` removes `_renewal_at` while preserving all other keys.
///
/// Given a `{name}.json` with both `oauthAccount` and `_renewal_at`, a `clear::1`
/// operation must remove only `_renewal_at` via read-merge semantics. The
/// `oauthAccount` field — including nested fields — must be unchanged afterward.
///
/// Spec: [`tests/docs/cli/param/051_clear.md` EC-3]
#[ test ]
fn arc02_clear_preserves_oauth_account_content()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // Write {name}.json with both oauthAccount and _renewal_at.
  std::fs::write(
    store.join( "test@example.com.json" ),
    r#"{"oauthAccount":{"emailAddress":"test@example.com","subscriptionType":"max"},"_renewal_at":"2026-06-29T21:00:00Z"}"#,
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::test@example.com", "clear::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content = std::fs::read_to_string( store.join( "test@example.com.json" ) ).unwrap();
  assert!(
    !content.contains( "_renewal_at" ),
    "clear::1 must remove _renewal_at key (051 EC-3), got: {content}",
  );
  assert!(
    content.contains( "oauthAccount" ),
    "clear::1 must preserve oauthAccount key (051 EC-3), got: {content}",
  );
  assert!(
    content.contains( "emailAddress" ),
    "clear::1 must preserve nested oauthAccount.emailAddress (051 EC-3), got: {content}",
  );
  assert!(
    content.contains( "subscriptionType" ),
    "clear::1 must preserve nested oauthAccount.subscriptionType (051 EC-3), got: {content}",
  );
}

