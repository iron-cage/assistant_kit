//! Integration tests: ARN (Account Renewal) + late AS tests (as22–as35).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID | Test Function | Condition | P/N | IT-N |
//! |----|---------------|-----------|-----|------|
//! | arn01 | `ft01_account_renewal_at_writes_renewal_at` | `at::` writes `_renewal_at` | P | IT-1 |
//! | arn02 | `ft02_account_renewal_from_now_positive` | `from_now::+1h30m` writes future date | P | IT-2 |
//! | arn03 | `ft03_account_renewal_from_now_negative` | `from_now::-30m` writes past date | P | |
//! | arn04 | `ft04_account_renewal_clear_removes_key` | `clear::1` removes `_renewal_at` | P | IT-3 |
//! | arn05 | `ft05_account_renewal_name_all_updates_all` | `name::all from_now::+0m` → all accounts | P | IT-9 |
//! | arn06 | `ft06_account_renewal_dry_no_write` | `dry::1` → [dry-run], no write | P | IT-8 |
//! | arn07 | `ft07_account_renewal_at_from_now_conflict` | `at::` + `from_now::` → exit 1 | N | IT-4 |
//! | arn08 | `ft08_account_renewal_at_clear_conflict` | `at::` + `clear::` → exit 1 | N | IT-5 |
//! | arn09 | `ft09_account_renewal_from_now_clear_conflict` | `from_now::` + `clear::` → exit 1 | N | IT-6 |
//! | arn10 | `ft10_account_renewal_no_operation_exits_1` | no operation param → exit 1 | N | IT-7 |
//! | arn11 | `ft11_account_renewal_unknown_account_exits_2` | unknown account → exit 2 | N | IT-10 |
//! | arn12 | `ft12_account_renewal_comma_list_updates_both` | comma-list updates both | P | IT-11 |
//! | arn13 | `ft13_account_renewal_partial_comma_list` | unknown in comma-list reported | N | |
//! | arn14 | `ft14_account_renewal_past_at_accepted` | past `at::` written verbatim | P | IT-13 |
//! | arn15 | `ft15_account_renewal_unknown_param_exits_1` | unknown param → exit 1 | N | IT-14 |
//! | arn16 | `ft16_account_renewal_creates_new_claude_json` | no prior file → created | P | |
//! | arn17 | `arn17_from_now_invalid_format_exits_1` | `from_now::invalid` → exit 1 | N | |
//! | arn18 | `arn18_from_now_unsupported_unit_exits_1` | `from_now::+1s` → exit 1 | N | |
//! | arn19 | `arn19_clear_no_prior_renewal_at_exits_0` | `clear::1` without `_renewal_at` → exit 0 | P | |
//! | arn20 | `arn20_all_three_conflict_exits_1` | all three conflict → exit 1 | N | |
//! | arn21 | `arn21_at_invalid_iso_stored_verbatim` | invalid ISO stored verbatim | P | |
//! | arn26 | `arn26_from_now_plus_no_units_exits_1` | `from_now::+` → exit 1 | N | |
//! | arn27 | `arn27_from_now_minus_no_units_exits_1` | `from_now::-` → exit 1 | N | |
//! | arc02 | `arc02_clear_preserves_oauth_account_content` | `clear::1` preserves oauthAccount | P | |
//! | as22 | `as22_save_preserves_renewal_at` | second save preserves `_renewal_at` | P | |
//! | as19 | `as19_save_best_effort_no_roles_json` | save without valid token → exit 0 | P | |
//! | as20 | `as20_lim_it_save_writes_roles_json` | save with valid token → `{name}.json` | P | |
//! | as21 | `as21_lim_it_resave_overwrites_roles_json` | second save overwrites `{name}.json` | P | |
//! | as23 | `as_save_writes_profile_json` | `host:: role::` → `{name}.json` created | P | |
//! | as24 | `as24_host_auto_capture_user_hostname` | no `host::` → auto-captured | P | |
//! | as25 | `as25_host_empty_triggers_auto_capture` | `host::` empty → auto-captured | P | |
//! | as26 | `as26_host_resave_overwrites` | resave with new host replaces old | P | |
//! | as27 | `as27_host_with_spaces` | `host::my work laptop` stored verbatim | P | |
//! | as28 | `as28_host_missing_user_stores_at_resolved_hostname` | USER/HOSTNAME unset → `@<resolved>` | P | |
//! | mre_bug239 | `mre_bug239_hostname_resolved_when_env_absent` | HOSTNAME absent → resolved (BUG-239) | P | |
//! | as29 | `as29_resave_credentials_unchanged` | resave does not modify credentials.json | P | |
//! | as30 | `as30_role_writes_profile_json` | `role::work` → `{name}.json` has role | P | |
//! | as31 | `as31_role_omit_stores_empty` | no `role::` → `{name}.json` has `"role":""` | P | |
//! | as32 | `as32_role_empty_stores_empty` | `role::` empty → `{name}.json` has `"role":""` | P | |
//! | as33 | `as33_role_resave_overwrites` | resave with new role replaces old | P | |
//! | as34 | `as34_role_with_spaces` | `role::dev ops team` stored verbatim | P | |
//! | as35 | `as35_save_dry_run_rejects_invalid_name` | `dry::1 name::not-an-email` → exit 1 | N | |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_claude_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── ARN: Account Renewal ──────────────────────────────────────────────────────

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-1]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-2]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-3]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-9]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-8]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-4]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-5]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-6]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-7]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-10]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-11]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-13]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-14]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-15]
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

/// Spec: [tests/docs/cli/command/14_account_renewal.md IT-16]
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

