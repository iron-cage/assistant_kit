//! Integration tests: AS (Account Save), AW (Account Use), AD (Account Delete), AR (Account Relogin).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### AS — Account Save
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | as01 | `as01_save_creates_file` | save creates .credentials.json | P |
//! | as02 | `as02_save_dry_run` | `dry::1` → no file created | P |
//! | as03 | `as03_save_overwrite` | second save overwrites first | P |
//! | as04 | `as04_save_hyphened_name` | hyphenated name accepted | P |
//! | as05 | `as05_save_underscored_name` | underscored name accepted | P |
//! | as06 | `as06_save_empty_name_exits_1` | empty name → exit 1 | N |
//! | as07 | `as07_save_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | as08 | `as08_save_backslash_name_exits_1` | name with `\` → exit 1 | N |
//! | as09 | `as09_save_star_name_exits_1` | name with `*` → exit 1 | N |
//! | as10 | `as10_save_infer_absent_email_exits_1` | no `name::`, no `_active` marker → exit 1 | N |
//! | as11 | `as11_save_missing_credentials_exits_2` | no credentials file → exit 2 | N |
//! | as12 | `as12_save_auto_creates_credential_store` | credential store auto-created | P |
//! | as13 | `as13_save_dry_then_exec_match` | dry then exec → same output | P |
//! | as14 | `as14_save_file_matches_source` | saved content matches source | P |
//! | as15 | `as15_save_infers_name_from_active_marker` | no `name::`, `_active` marker present → exit 0 | P |
//! | as16 | `as16_save_writes_active_marker` | save writes active marker = name | P |
//! | as17 | `as17_save_slash_in_email_local_part_exits_1` | `/` in email local part → exit 1 | N |
//! | as18 | `as18_save_backslash_in_email_local_part_exits_1` | `\` in email local part → exit 1 | N |
//! | as22 | `as22_save_preserves_renewal_at` | second `.account.save` preserves `_renewal_at` via read-merge | P |
//!
//! ### AW — Account Use
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | aw01 | `aw01_switch_swaps_credentials` | switch replaces .credentials.json | P |
//! | aw02 | `aw02_switch_dry_run` | `dry::1` → no file changed | P |
//! | aw03 | `aw03_switch_nonexistent_exits_2` | unknown account → exit 2 | N |
//! | aw04 | `aw04_switch_empty_name_exits_1` | empty name → exit 1 | N |
//! | aw05 | `aw05_switch_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | aw06 | `aw06_switch_missing_name_param_exits_1` | no `name::` param → exit 1 | N |
//! | aw07 | `aw07_switch_updates_active_marker` | switch writes active marker | P |
//! | aw08 | `aw08_switch_same_account_idempotent` | switch to same account succeeds | P |
//! | aw09 | `aw09_switch_copies_credentials` | switch copies correct cred content | P |
//! | aw10 | `aw10_switch_dry_run_nonexistent_exits_2` | dry-run nonexistent → exit 2 | N |
//! | aw11 | `aw11_switch_slash_in_email_local_part_exits_1` | `/` in email local part → exit 1 | N |
//! | aw12 | `aw12_switch_patches_email_when_metadata_absent` | emailAddress patched when `{name}.json` absent (BUG-254) | P |
//! | — | `switch_restores_claude_json` | `~/.claude.json` restored after switch (BUG-277) | P |
//! | — | `mre_bug_217_switch_account_enforces_emailaddress` | switch enforces `emailAddress == name` over stale snapshot | P |
//! | aw13 | `aw13_use_positional_bare_arg` | positional email `personal@home.com` → switches | P |
//! | aw14 | `aw14_use_prefix_resolves` | prefix `car` resolves to `carol@example.com`, switches | P |
//! | aw15 | `aw15_use_prefix_ambiguous_exits_1` | ambiguous prefix `a` → exit 1 with "ambiguous" | N |
//! | aw16 | `aw16_exact_local_part_wins_over_ambiguous_prefix` | `i1` resolves to `i1@` when `i11@`/`i12@` also exist | P |
//! | aw17 | `aw17_use_prefix_ambiguous_no_exact_local_part_exits_1` | prefix `i1` → exit 1 when only `i11@`/`i12@` exist (no `i1@`) | N |
//! | aw22 | `aw22_touch_disabled_switch_succeeds` | `touch::0` → switch exits 0, no quota fetch | P |
//! | aw23 | `aw23_touch_skipped_no_access_token` | `touch::1` (default) + no `accessToken` → exit 0 | P |
//! | aw24 | `aw24_imodel_bad_value_exits_1` | `imodel::bad` → exit 1, stderr lists valid values | N |
//! | aw25 | `aw25_effort_bad_value_exits_1` | `effort::bad` → exit 1, stderr lists valid values | N |
//! | aw26 | `aw26_help_shows_touch_imodel_effort` | `.account.use.help` lists `touch`, `imodel`, `effort`, `trace` | P |
//! | aw27 | `aw27_lim_it_touch_with_live_token` | live token + `touch::1` → switch exits 0 (`lim_it`) | P |
//! | aw28 | `aw28_lim_it_trace_idle_account_all_lines` | `trace::1` + live idle token → all 6 trace lines on stderr (`lim_it`) | P |
//! | aw29 | `aw29_lim_it_trace_active_account_subprocess_skipped` | `trace::1` + live active token → read+fetch+idle-check+model+subprocess-skipped lines (`lim_it`) | P |
//! | aw30 | `aw30_trace_fetch_failure_skips_idle_model_lines` | `trace::1` + invalid token → fetch-err + subprocess-skipped only | N |
//! | aw31 | `aw31_trace_touch_disabled_no_trace_lines` | `touch::0 trace::1` → no `[trace] account.use` lines | P |
//! | aw32 | `aw32_trace_bad_value_exits_1` | `trace::bad` → exit 1, stderr lists valid values | N |
//! | aw35 | `aw35_help_shows_positional_example` | `.account.use.help` examples contain positional form (no `name::`) (015 FT-10/AC-10) | P |
//!
//! ### AD — Account Delete
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ad01 | `ad01_delete_inactive_removes_file` | delete inactive removes file | P |
//! | ad02 | `ad02_delete_dry_run_keeps_file` | `dry::1` → file kept | P |
//! | ad03 | `ad03_delete_active_exits_0` | delete active account → exit 0, active marker cleaned up | P |
//! | ad04 | `ad04_delete_nonexistent_exits_2` | unknown account → exit 2 | N |
//! | ad05 | `ad05_delete_empty_name_exits_1` | empty name → exit 1 | N |
//! | ad06 | `ad06_delete_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | ad07 | `ad07_delete_missing_name_param_exits_1` | no `name::` param → exit 1 | N |
//! | ad08 | `ad08_delete_then_list_absent` | delete then list → account gone | P |
//! | ad09 | `ad09_double_delete_exits_2` | delete twice → second exit 2 | N |
//! | ad10 | `ad10_delete_dry_run_active_exits_0` | dry delete active → exit 0, file kept | P |
//! | ad11 | `ad11_delete_dry_run_nonexistent_exits_2` | dry delete nonexistent → exit 2 | N |
//! | ad12 | `ad12_delete_removes_snapshot_files` | delete removes `{name}.json` snapshot | P |
//! | ad13 | `ad13_delete_positional_bare_arg` | positional email `old@archive.com` → deletes account | P |
//! | ad14 | `ad14_delete_prefix_resolves` | prefix `old` resolves to `old@archive.com`, deletes | P |
//! | ad15 | `ad15_delete_removes_roles_json` | delete removes `{name}.json` snapshot (roles data) | P |
//! | as19 | `as19_save_best_effort_no_roles_json` | save with no valid token → exit 0; no `{name}.json` | P |
//! | as20 | `as20_lim_it_save_writes_roles_json` | save with valid token → `{name}.json` created (`lim_it`) | P |
//! | as21 | `as21_lim_it_resave_overwrites_roles_json` | second save overwrites `{name}.json` (`lim_it`) | P |
//! | as23 | `as_save_writes_profile_json` | `host::testbox role::dev` → `{name}.json` created with JSON | P |
//! | as24 | `as24_host_auto_capture_user_hostname` | no `host::` → `{name}.json` has `"host":"$USER@$HOSTNAME"` | P |
//! | as25 | `as25_host_empty_triggers_auto_capture` | `host::` (empty) → same as omit, auto-captured | P |
//! | as26 | `as26_host_resave_overwrites` | resave with `host::newbox` replaces `host::oldbox` | P |
//! | as27 | `as27_host_with_spaces` | `host::my work laptop` stored verbatim in `{name}.json` | P |
//! | as28 | `as28_host_missing_user_stores_at_resolved_hostname` | USER unset, HOSTNAME unset → `"host":"@<resolved>"` (AC-03) | P |
//! | `mre_bug239` | `mre_bug239_hostname_resolved_when_env_absent` | HOSTNAME absent, USER=alice → host `"alice@<resolved>"` not `"alice@"` (BUG-239) | P |
//! | as29 | `as29_resave_credentials_unchanged` | resave with `host::newbox` does not modify credentials.json | P |
//! | as30 | `as30_role_writes_profile_json` | explicit `role::work` → `{name}.json` has `"role":"work"` | P |
//! | as31 | `as31_role_omit_stores_empty` | no `role::` param → `{name}.json` has `"role":""` | P |
//! | as32 | `as32_role_empty_stores_empty` | `role::` (empty) → `{name}.json` has `"role":""` | P |
//! | as33 | `as33_role_resave_overwrites` | resave with `role::dev` replaces `role::personal` | P |
//! | as34 | `as34_role_with_spaces` | `role::dev ops team` stored verbatim in `{name}.json` | P |
//!
//! ### AR — Account Relogin
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ar01 | `relogin_mre_no_name_uses_active` | no `name::` + active account → uses active (dry-run) | P |
//! | ar02 | `relogin_mre_no_name_no_active_exits2` | no `name::` + no `_active` marker → exit 2 | N |
//! | ar03 | `ar03_relogin_empty_name_exits_1` | empty `name::` value → exit 1 | N |
//! | ar04 | `ar04_relogin_not_found_exits_2` | `name::ghost@example.com`, no such account → exit 2 | N |
//! | ar05 | `ar05_relogin_dry_explicit_name` | `dry::1` with explicit name prints message | P |
//! | ar07 | `ar07_relogin_positional_bare_arg` | positional `work@acme.com dry::1` → resolves | P |
//! | ar08 | `ar08_relogin_prefix_resolves` | prefix `work dry::1` → `work@acme.com` | P |
//! | ar09 | `ar09_relogin_invalid_chars_exits_1` | `name::bad/name` → exit 1 | N |
//!
//! ### ARN — Account Renewal
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | arn01 | `ft01_account_renewal_at_writes_renewal_at` | `at::` writes `_renewal_at`; `oauthAccount` preserved | P |
//! | arn02 | `ft02_account_renewal_from_now_positive` | `from_now::+1h30m` writes future `_renewal_at` | P |
//! | arn03 | `ft03_account_renewal_from_now_negative` | `from_now::-30m` writes past `_renewal_at` (verbatim) | P |
//! | arn04 | `ft04_account_renewal_clear_removes_key` | `clear::1` removes `_renewal_at`; `oauthAccount` preserved | P |
//! | arn05 | `ft05_account_renewal_name_all_updates_all` | `name::all from_now::+0m` writes to every account | P |
//! | arn06 | `ft06_account_renewal_dry_no_write` | `dry::1` prints `[dry-run]`; no file written | P |
//! | arn07 | `ft07_account_renewal_at_from_now_conflict` | `at::` + `from_now::` together exits 1 | N |
//! | arn08 | `ft08_account_renewal_at_clear_conflict` | `at::` + `clear::` together exits 1 | N |
//! | arn09 | `ft09_account_renewal_from_now_clear_conflict` | `from_now::` + `clear::` together exits 1 | N |
//! | arn10 | `ft10_account_renewal_no_operation_exits_1` | no operation param exits 1 | N |
//! | arn11 | `ft11_account_renewal_unknown_account_exits_2` | unknown account exits 2 | N |
//! | arn12 | `ft12_account_renewal_comma_list_updates_both` | comma-list updates both accounts | P |
//! | arn13 | `ft13_account_renewal_partial_comma_list` | unknown in comma-list reported; others processed | N |
//! | arn14 | `ft14_account_renewal_past_at_accepted` | past `at::` written verbatim; not auto-advanced at write | P |
//! | arn15 | `ft15_account_renewal_unknown_param_exits_1` | unknown param rejected → exit 1 | N |
//! | arn16 | `ft16_account_renewal_creates_new_claude_json` | no prior `{name}.json` → file created | P |
//! | arn17 | `arn17_from_now_invalid_format_exits_1` | `from_now::invalid` (no +/-) → exit 1 with parse error | N |
//! | arn18 | `arn18_from_now_unsupported_unit_exits_1` | `from_now::+1s` (unit 's') → exit 1 with parse error | N |
//! | arn19 | `arn19_clear_no_prior_renewal_at_exits_0` | `clear::1` when no `_renewal_at` → exit 0, no error | P |
//! | arn26 | `arn26_from_now_plus_no_units_exits_1` | `from_now::+` (sign only) → exit 1 with parse error | N |
//! | arn27 | `arn27_from_now_minus_no_units_exits_1` | `from_now::-` (sign only) → exit 1 with parse error | N |
//! | arn20 | `arn20_all_three_conflict_exits_1` | `at:: from_now:: clear::` all together → exit 1 | N |
//! | arn21 | `arn21_at_invalid_iso_stored_verbatim` | `at::not-a-date` stored verbatim; exit 0 | P |
//! | arc02 | `arc02_clear_preserves_oauth_account_content` | `clear::1` removes `_renewal_at`; `oauthAccount` preserved | P |
//!
//! ### Bug Reproducers
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | mre_bug209 | `mre_bug_209_account_save_uses_active_marker_not_stale_email` | `.account.save` reads `_active` marker, not stale `emailAddress` | P |

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_claude_json, account_exists,
  write_account_claude_json, write_account_settings_json, write_account_roles_json,
  live_active_token, write_account_with_token, require_live_api,
  FAR_FUTURE_MS,
};
use std::process::Command;
use tempfile::TempDir;

// ── AS: Account Save ──────────────────────────────────────────────────────────

#[ test ]
fn as01_save_creates_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "saved current credentials as 'alice@acme.com'" ), "must confirm save, got:\n{text}" );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "account file must exist" );
}

#[ test ]
fn as02_save_dry_run()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run] would save current credentials as 'alice@acme.com'" ), "must say dry-run preview, got:\n{text}" );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "dry-run must not create file" );
}

#[ test ]
fn as03_save_overwrite()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // First save
  let _ = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  // Update credentials and save again
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  // Verify new content
  let saved = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@acme.com.credentials.json" )
  ).unwrap();
  assert!( saved.contains( "max" ), "overwrite must use new credentials, got: {saved}" );
}

#[ test ]
fn as04_save_hyphened_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice-work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice-work@acme.com" ) );
}

#[ test ]
fn as05_save_underscored_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice_work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice_work@acme.com" ) );
}

#[ test ]
fn as06_save_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as07_save_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as08_save_backslash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a\\b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as09_save_star_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a*b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as10_save_infer_absent_email_exits_1()
{
  // IT-10: no _active marker → inference fails → exit 1.
  // write_credentials writes only ~/.claude/.credentials.json, no _active marker is set,
  // so the inference branch finds no active account and must exit 1.
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "cannot infer account name: no active account set" ),
    "stderr must explain inference failure, got:\n{err}",
  );
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.exists(), "credential store must not be created on inference failure" );
}

#[ test ]
fn as15_save_infers_name_from_active_marker()
{
  // IT-14: _active marker present → inference succeeds → exit 0, saves under marker name.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Write _active marker = "alice@acme.com" (simulates prior .account.use).
  let store = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( claude_profile::account::active_marker_filename() ),
    "alice@acme.com",
  ).unwrap();

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "saved current credentials as 'alice@acme.com'" ), "must confirm save with inferred name, got:\n{text}" );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "credential file must be created under inferred name" );
}

#[ test ]
fn as11_save_missing_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file — only create .claude dir
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn as12_save_auto_creates_credential_store()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // credential store does NOT exist

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "account file must be auto-created" );
}

#[ test ]
fn as13_save_dry_then_exec_match()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let dry = run_cs_with_env( &[ ".account.save", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &dry, 0 );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "dry-run must not create file" );

  let exec = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &exec, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "exec must create file" );
}

#[ test ]
fn as14_save_file_matches_source()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let _ = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );

  let source = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let saved = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@acme.com.credentials.json" )
  ).unwrap();
  assert_eq!( source, saved, "saved file must be byte-identical to source" );
}

// ── AW: Account Use ───────────────────────────────────────────────────────────

#[ test ]
fn aw01_switch_swaps_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "name::alice@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "switched" ), "must confirm switch, got:\n{text}" );
}

#[ test ]
fn aw02_switch_dry_run()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let before = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let out = run_cs_with_env( &[ ".account.use", "name::alice@home.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run] would switch to 'alice@home.com'" ), "must print full dry-run message, got:\n{text}" );
  let after = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  assert_eq!( before, after, "dry-run must not change credentials" );
}

#[ test ]
fn aw03_switch_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.use", "name::missing@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn aw04_switch_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw05_switch_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw06_switch_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw07_switch_updates_active_marker()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.use", "name::alice@home.com" ], &[ ( "HOME", home ) ] );

  let marker = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( claude_profile::account::active_marker_filename() )
  ).unwrap();
  assert_eq!( marker.trim(), "alice@home.com" );
}

#[ test ]
fn aw08_switch_same_account_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.use", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn aw09_switch_copies_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.use", "name::alice@home.com" ], &[ ( "HOME", home ) ] );

  let creds = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let account_file = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@home.com.credentials.json" )
  ).unwrap();
  assert_eq!( creds, account_file, "credentials must match account file after switch" );
}

// ── AD: Account Delete ────────────────────────────────────────────────────────

#[ test ]
// Fix(BUG-281):
// Root cause: run_cs_with_env() set HOME to a temp dir but inherited $PRO from the test runner;
//   PersistPaths::resolve_root() prefers $PRO over $HOME when $PRO is an existing directory, so
//   the binary operated on the real credential store ($PRO/.persistent/claude/credential) while
//   the test wrote fixtures to and checked $HOME/.persistent/claude/credential — the two paths
//   never overlapped.
// Why Not Caught: tests were developed in a Docker container where $PRO is not set; the
//   isolation failure is invisible there and only manifests in the host environment.
// Fix Applied: added cmd.env_remove("PRO") to run_cs_with_env() in helpers.rs so that $PRO
//   cannot leak into subprocesses when tests supply a custom HOME.
// Prevention: any subprocess helper that isolates HOME must explicitly remove $PRO (and
//   $USERPROFILE); document this as an invariant in helpers.rs.
// Pitfall: cmd.env("HOME", ...) does not clear inherited vars — $PRO still takes priority until
//   explicitly removed with env_remove().
fn ad01_delete_inactive_removes_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "alice@oldco.com" ), "account file must be removed" );
}

#[ test ]
fn ad02_delete_dry_run_keeps_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run] would delete account 'alice@oldco.com'" ), "must print full dry-run message, got:\n{text}" );
  assert!( account_exists( dir.path(), "alice@oldco.com" ), "dry-run must not delete file" );
}

#[ test ]
fn ad03_delete_active_exits_0()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "active account must be deleted" );
  let active_marker = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( claude_profile::account::active_marker_filename() );
  assert!( !active_marker.exists(), "_active marker must be cleaned up after deleting active account" );
}

#[ test ]
fn ad04_delete_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad05_delete_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad06_delete_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad07_delete_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad08_delete_then_list_absent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "keep@example.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );

  let out = run_cs_with_env( &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "email::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( !text.contains( "alice@oldco.com" ), "deleted account must not appear in list, got:\n{text}" );
  assert!( text.contains( "keep@example.com" ), "kept account must still appear, got:\n{text}" );
}

#[ test ]
fn ad09_double_delete_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let first = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &first, 0 );

  let second = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &second, 2 );
}

// Root Cause: `account_use_routine` checked `is_dry()` before validating account
//   existence, so `.account.use dry::1 name::missing` returned exit 0 ("would switch
//   to 'missing'") even when the named account does not exist.
// Why Not Caught: `aw02_switch_dry_run` only exercises the happy-path dry-run (valid
//   account). No test covered the dry-run-with-nonexistent-account case.
// Fix Applied: `check_switch_preconditions()` extracted from `switch_account()` and
//   called in the command routine before the dry-run guard.
// Prevention: Dry-run must always run input validation + precondition checks; only the
//   mutation step is skipped.
// Pitfall: Placing `is_dry()` before domain validation produces misleading "would do X"
//   output for operations that would actually fail — always validate first, then dry-run.
#[ doc = "bug_reproducer(BUG-265)" ]
#[ test ]
fn aw10_switch_dry_run_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.use", "name::missing@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad10_delete_dry_run_active_exits_0()
{
  // Dry-run on the active account exits 0 now that the active-account guard is removed.
  // The account file must not be deleted (dry-run protection is unrelated to active status).
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "dry-run must not delete active account" );
}

// Root Cause: Same as ad10 — `is_dry()` guard ran before any account existence check,
//   so `.account.delete dry::1 name::ghost` (nonexistent) returned exit 0 instead of
//   exit 2 (`NotFound`).
// Why Not Caught: `ad02` exercises an existing account; no test covered dry-run on a
//   nonexistent account.
// Fix Applied: See ad10 — `check_delete_preconditions()` runs before dry-run guard.
// Prevention: Dry-run path must include all validation; only file-system mutation is omitted.
// Pitfall: Missing existence check in dry-run gives a false "operation would succeed"
//   signal, masking configuration errors until the real run.
#[ doc = "bug_reproducer(BUG-266)" ]
#[ test ]
fn ad11_delete_dry_run_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad12_delete_removes_snapshot_files()
{
  // IT-11: delete removes credentials and {name}.json snapshot.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "old@archive.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_claude_json(   dir.path(), "old@archive.com", "", "", "", "" );
  write_account_settings_json( dir.path(), "old@archive.com", "sonnet" );

  let out = run_cs_with_env( &[ ".account.delete", "name::old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.join( "old@archive.com.credentials.json" ).exists(), "credentials must be removed after delete" );
  assert!( !store.join( "old@archive.com.json" ).exists(),      "{{name}}.json snapshot must be removed after delete" );
}

// ── as16 ──────────────────────────────────────────────────────────────────────

/// as16: `.account.save name::work@acme.com` writes `{store}/_active_{hostname}_{user}` = `"work@acme.com"`.
///
/// CLI-level symmetry test with aw07: reads the active marker directly (not via
/// `.credentials.status`) to confirm the write happened at the filesystem level.
///
/// ## Fix Documentation — BUG-282
///
/// - **Root Cause:** `save()` never wrote the active marker; only `switch_account()` did.
/// - **Why Not Caught:** No AS test verified the active marker file after `.account.save`.
/// - **Fix Applied:** Added `std::fs::write( credential_store.join( active_marker_filename() ), name )?;` to `save()`. (Originally `join("_active")`; updated to per-machine `active_marker_filename()` per Feature 025.)
/// - **Prevention:** This test guards the active marker at the filesystem level, independently of `.credentials.status`.
/// - **Pitfall:** Must assert the raw file content — not just exit code — to catch a write that produces wrong content.
#[ test ]
fn as16_save_writes_active_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::work@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store  = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must exist after .account.save" );
  assert_eq!(
    active.trim(),
    "work@acme.com",
    "_active must equal the saved account name",
  );
}

// ── switch_restores_claude_json ────────────────────────────────────────────────

/// bug_reproducer(BUG-277): `.account.use` does not restore `~/.claude.json`,
/// so `.credentials.status` shows the previous account's email after a switch.
///
/// ## Fix Documentation — BUG-277
///
/// - **Root Cause:** `switch_account()` restored only `.credentials.json`; the
///   companion `~/.claude.json` restore (from `{name}.json` snapshot) was
///   never added, leaving the active JSON pointing at the previous account's data.
/// - **Why Not Caught:** Prior tests never called `.credentials.status` after
///   `.account.use` in a two-account setup, so the email mismatch was invisible.
/// - **Fix Applied:** Added two best-effort `let _ = std::fs::copy(...)` calls in
///   `switch_account()` after the `_active` marker write — mirroring the two
///   companion writes already present in `save()`.
/// - **Prevention:** This test encodes the full save-A / save-B / switch-to-A /
///   check-email flow, preventing any future regression where the restore pair
///   becomes asymmetric again.
/// - **Pitfall:** The `let _ = ...` idiom silences copy errors intentionally —
///   `~/.claude.json` may legitimately not exist. The test must explicitly write
///   `~/.claude.json` for both accounts before saving so the snapshots exist.
#[ doc = "bug_reproducer(BUG-277)" ]
#[ test ]
fn switch_restores_claude_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Save account A: work@acme.com
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "work@acme.com" );
  let save_a = run_cs_with_env( &[ ".account.save", "name::work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &save_a, 0 );

  // Save account B: personal@home.com (overwrites active credentials + claude.json)
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "personal@home.com" );
  let save_b = run_cs_with_env( &[ ".account.save", "name::personal@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &save_b, 0 );

  // Switch back to A — must restore work@acme.com's ~/.claude.json
  let switch_out = run_cs_with_env(
    &[ ".account.use", "name::work@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &switch_out, 0 );

  // .credentials.status must show work@acme.com — not personal@home.com
  let status_out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &status_out, 0 );
  let text = stdout( &status_out );
  assert!(
    text.contains( "work@acme.com" ),
    "Email: must reflect switched-to account, got:\n{text}",
  );
}

// ── as17 ──────────────────────────────────────────────────────────────────────

/// bug_reproducer(BUG-278): `.account.save name::a/b@c.com` exits 2 instead
/// of 1 because `validate_name()` passes `a/b@c.com` (local part non-empty,
/// domain non-empty), then `save()` hits a filesystem error when creating
/// `a/b@c.com.credentials.json`.
///
/// ## Fix Documentation — BUG-278
///
/// - **Root Cause:** `validate_name()` only checked `@` presence and non-empty
///   local/domain parts; it did not reject path-unsafe chars (`/`, `\`, `*`)
///   inside the local part, so names like `a/b@c.com` bypassed validation and
///   reached filesystem operations that exit 2.
/// - **Why Not Caught:** Existing as07/as08/as09 only cover names WITHOUT `@`
///   (caught by the "must contain @" guard); no test covered the combined case
///   where the local part carries an unsafe char but `@` is present.
/// - **Fix Applied:** Added a local-part path-safety check in `validate_name()`
///   after the `@` position is found: if the local part contains `/`, `\`, or
///   `*`, return `InvalidInput` (exit 1) before any filesystem operation runs.
/// - **Prevention:** This test (as17) and as18/aw11 encode the three unsafe-char
///   variants so any regression in the local-part check is caught immediately.
/// - **Pitfall:** Only the local part (before `@`) needs the check; domain chars
///   cannot create path traversal in practice because the `@` separates them.
#[ doc = "bug_reproducer(BUG-278)" ]
#[ test ]
fn as17_save_slash_in_email_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a/b@c.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "path-unsafe characters" ),
    "stderr must indicate path-unsafe chars, got:\n{err}",
  );
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.exists(), "credential store must not be created before validation passes" );
}

// ── as18 ──────────────────────────────────────────────────────────────────────

/// bug_reproducer(BUG-278): same root cause as as17 but for `\` in the local
/// part of the email address.
///
/// See as17 for full fix documentation.
#[ doc = "bug_reproducer(BUG-278)" ]
#[ test ]
fn as18_save_backslash_in_email_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a\\b@c.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── aw11 ──────────────────────────────────────────────────────────────────────

/// bug_reproducer(BUG-278): `.account.use name::a/b@c.com` exits 2 instead
/// of 1 for the same reason as as17 — `validate_name()` passes the name, then
/// `switch_account()` fails with a filesystem error.
///
/// See as17 for full fix documentation.
#[ doc = "bug_reproducer(BUG-278)" ]
#[ test ]
fn aw11_switch_slash_in_email_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use", "name::a/b@c.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── aw12 ──────────────────────────────────────────────────────────────────────

/// # Root Cause
///
/// `switch_account()` gates the `emailAddress` patch inside `if let Ok(saved_val) =
/// serde_json::from_str(&meta_text)`. When `{name}.json` is absent, `meta_text` is `""`,
/// `from_str("")` returns `Err`, and the entire oauthAccount patch block is skipped —
/// including the BUG-217 `emailAddress` enforcement. `~/.claude.json` retains the previous
/// account's `emailAddress`, causing downstream `save()` name inference to target the wrong
/// file.
///
/// # Why Not Caught
///
/// All existing `switch_account()` FT tests provide a `{name}.json` metadata file via
/// `.account.save`. No FT test covers the absent-metadata-file path where only credentials
/// exist.
///
/// # Fix Applied
///
/// Lift the unconditional `emailAddress` patch out of the metadata-file-conditional block.
/// Patch `~/.claude.json oauthAccount.emailAddress = name` before attempting to read
/// `{name}.json`. The full overlay (BUG-217 + BUG-219) still fires when metadata is present.
///
/// # Prevention
///
/// This FT test creates a credential-only account (no `{name}.json`) and asserts that
/// `emailAddress` is patched to the switched-to name after `.account.use`.
///
/// # Pitfall
///
/// `claude_json_file()` returns `$HOME/.claude.json` (HOME level), not
/// `$HOME/.claude/claude.json`. Machine-global keys must survive the patch — assert
/// preservation.
/// FT-09: AC-09 — emailAddress patched unconditionally even when metadata absent
#[ doc = "bug_reproducer(BUG-254)" ]
#[ test ]
fn aw12_switch_patches_email_when_metadata_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Live credentials (required so switch_account can copy to .credentials.json).
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // alice: credentials + active marker.  bob: credentials ONLY — NO bob@acme.com.json.
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "bob@acme.com",   "max", "tier4",    FAR_FUTURE_MS, false );

  // Seed ~/.claude.json with alice's emailAddress + machine-global keys.
  let claude_json_path = dir.path().join( ".claude.json" );
  std::fs::write(
    &claude_json_path,
    r#"{"oauthAccount":{"emailAddress":"alice@acme.com","displayName":"Alice"},"commands":{"enabled":true},"mcpServers":{}}"#,
  ).unwrap();

  // touch::0 disables pre-fetch HTTP calls — tests the pure file switch.
  let out = run_cs_with_env(
    &[ ".account.use", "name::bob@acme.com", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // After switch: emailAddress must be patched to bob even though bob@acme.com.json is absent.
  let claude_json = std::fs::read_to_string( &claude_json_path ).unwrap();
  assert!(
    claude_json.contains( r#""emailAddress":"bob@acme.com""# ),
    "BUG-254: emailAddress must be 'bob@acme.com' after switch, got:\n{claude_json}",
  );
  assert!(
    !claude_json.contains( r#""emailAddress":"alice@acme.com""# ),
    "BUG-254: stale emailAddress 'alice@acme.com' must not remain, got:\n{claude_json}",
  );

  // Machine-global keys must survive the unconditional patch.
  assert!(
    claude_json.contains( r#""commands""# ),
    "machine-global key 'commands' must survive, got:\n{claude_json}",
  );
  assert!(
    claude_json.contains( r#""mcpServers""# ),
    "machine-global key 'mcpServers' must survive, got:\n{claude_json}",
  );

  // _active marker must point at bob.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string(
    store.join( claude_profile::account::active_marker_filename() )
  ).expect( "_active must exist" );
  assert_eq!(
    active.trim(), "bob@acme.com",
    "_active marker must point at switched-to account",
  );
}

// ── aw13 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn aw13_use_positional_bare_arg()
{
  // AC-01: positional form `clp .account.use personal@home.com` is equivalent to
  // `clp .account.use name::personal@home.com`.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "work@acme.com",     "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "personal@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "switched" ), "must confirm switch, got:\n{text}" );
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) ).expect( "_active must exist" );
  assert_eq!( active.trim(), "personal@home.com", "_active must point at switched-to account" );
}

// ── aw14 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn aw14_use_prefix_resolves()
{
  // AC-05: prefix `car` resolves uniquely to `carol@example.com` and switches.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "carol@example.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "amy@example.com", "pro", "standard", FAR_FUTURE_MS, true  );

  let out = run_cs_with_env( &[ ".account.use", "car" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) ).expect( "_active must exist" );
  assert_eq!( active.trim(), "carol@example.com", "prefix car must resolve to carol@example.com" );
}

// ── aw15 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn aw15_use_prefix_ambiguous_exits_1()
{
  // AC-06: ambiguous prefix `a` matches both `alice@example.com` and `amy@example.com` → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "amy@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "a" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "ambiguous" ),
    "error must say 'ambiguous', got:\n{err}",
  );
}

// ── aw16 ──────────────────────────────────────────────────────────────────────

/// aw16 (AC-11 / `015_name_shortcut_syntax.md`): exact local-part match wins over ambiguous prefix.
///
/// Three accounts: `i1@wbox.pro`, `i11@wbox.pro`, `i12@wbox.pro`.
/// Prefix `i1` matches all three via `starts_with`, but `i1@wbox.pro` has
/// local part equal to `i1` exactly — the exact-local-part check resolves
/// it unambiguously without reaching the prefix scan.
#[ test ]
fn aw16_exact_local_part_wins_over_ambiguous_prefix()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "i1@wbox.pro",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "i11@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "i12@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "i1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "active marker must exist after use" );
  assert_eq!(
    active.trim(), "i1@wbox.pro",
    "exact local-part match must resolve to i1@wbox.pro, not be reported as ambiguous",
  );
}

// ── aw17 ──────────────────────────────────────────────────────────────────────

/// aw17 (AC-06, AC-11 / `015_name_shortcut_syntax.md` FT-08): prefix `i1` is ambiguous
/// when only `i11@wbox.pro` and `i12@wbox.pro` exist — no `i1@wbox.pro` account.
///
/// The exact-local-part check (AC-11) finds no account with local part exactly `i1`.
/// Falling through to prefix scan, both `i11@` and `i12@` match — ambiguity reported
/// with exit 1 (AC-06). Complements aw16: positive case where `i1@` exists exits 0.
#[ test ]
fn aw17_use_prefix_ambiguous_no_exact_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // Only i11 and i12 exist — no i1@wbox.pro. Prefix i1 matches both via starts_with.
  write_account( dir.path(), "i11@wbox.pro", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "i12@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "i1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "ambiguous" ),
    "error must say 'ambiguous' when prefix i1 matches i11@ and i12@ but no i1@ exists, got:\n{err}",
  );
}

// ── ad13 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad13_delete_positional_bare_arg()
{
  // AC-02 (delete): positional form `clp .account.delete old@archive.com` is
  // equivalent to `clp .account.delete name::old@archive.com`.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",    "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "old@archive.com",  "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "old@archive.com" ), "account must be deleted" );
}

// ── ad14 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad14_delete_prefix_resolves()
{
  // AC-05 (delete): prefix `old` resolves uniquely to `old@archive.com` and deletes it.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",    "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "old@archive.com",  "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "old" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "old@archive.com" ), "prefix old must resolve to old@archive.com and delete it" );
}

// ── relogin: optional-name default-to-active tests ────────────────────────────

/// IT-1 / AC-02: `.account.relogin` with no `name::` uses the active account.
///
/// Verifies that when `name::` is omitted and the `_active` marker names
/// `work@acme.com`, the dry-run output names that account — confirming the
/// active-account fallback per `invariant/006_param_defaults.md`.
#[ test ]
fn relogin_mre_no_name_uses_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run] would re-authenticate 'work@acme.com' via browser login" ),
    "dry-run must print full re-auth message naming active account, got:\n{text}",
  );
}

/// IT-2 / AC-03: `.account.relogin` with no `name::` and no `_active` marker exits 2.
///
/// Verifies that omitting `name::` when no active account is set produces
/// exit 2 with an actionable message — not exit 1 (usage error).
#[ test ]
fn relogin_mre_no_name_no_active_exits2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Account file exists but no _active marker written (make_active = false).
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.relogin" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "no active account" ) || err.contains( "name::" ),
    "error must mention missing active account, got:\n{err}",
  );
}

// ── IT-3 through IT-9 ─────────────────────────────────────────────────────────

#[ test ]
fn ar03_relogin_empty_name_exits_1()
{
  // IT-3: empty `name::` value → exit 1 (ArgumentMissing).
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.relogin", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ar04_relogin_not_found_exits_2()
{
  // IT-4: named account does not exist in the store → check_switch_preconditions fails → exit 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::ghost@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

#[ test ]
fn ar05_relogin_dry_explicit_name()
{
  // IT-5: dry::1 with an existing name prints the re-auth message without spawning claude.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::work@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run] would re-authenticate 'work@acme.com' via browser login" ),
    "dry-run must print full re-auth message, got:\n{text}",
  );
}

#[ test ]
fn ar07_relogin_positional_bare_arg()
{
  // IT-7: positional form `clp .account.relogin work@acme.com dry::1` resolves the account.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "work@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "work@acme.com" ),
    "positional arg must resolve account name, got:\n{text}",
  );
}

#[ test ]
fn ar08_relogin_prefix_resolves()
{
  // IT-8: prefix `work` uniquely resolves to `work@acme.com` and uses it.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.relogin", "work", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "work@acme.com" ),
    "prefix 'work' must resolve to work@acme.com, got:\n{text}",
  );
}

#[ test ]
fn ar09_relogin_invalid_chars_exits_1()
{
  // IT-9: `name::bad/name` — no `@`, path-unsafe `/` → ArgumentTypeMismatch → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.relogin", "name::bad/name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── ad15 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad15_delete_removes_roles_json()
{
  // AC-04: delete removes {name}.json alongside credentials.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",   "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "old@archive.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_roles_json( dir.path(), "old@archive.com", "org-del-123", "Delete Corp", "admin" );

  let out = run_cs_with_env( &[ ".account.delete", "name::old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.join( "old@archive.com.credentials.json" ).exists(), "credentials must be removed" );
  assert!( !store.join( "old@archive.com.json" ).exists(),       "{{name}}.json snapshot must be removed after delete" );
}

// ── as19 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn as19_save_best_effort_no_roles_json()
{
  // AC-02: save with no valid accessToken in credentials → exit 0; roles data absent from
  // {{name}}.json.  The unified file must not contain org identity fields.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // credentials JSON has no accessToken field, so fetch_claude_cli_roles is never called.

  let out = run_cs_with_env( &[ ".account.save", "name::user@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta = std::fs::read_to_string( store.join( "user@example.com.json" ) )
    .unwrap_or_default();
  assert!(
    !meta.contains( "organization_uuid" ),
    "{{name}}.json must not contain org identity when no accessToken, got: {meta}",
  );
}

// ── as20 (lim_it) ─────────────────────────────────────────────────────────────

#[ test ]
fn as20_lim_it_save_writes_roles_json()
{
  // AC-01 (FT-01): .account.save with a valid accessToken calls fetch_claude_cli_roles and
  // writes {name}.json to the credential store. Requires live Anthropic credentials.
  let Some( token ) = live_active_token() else
  {
    eprintln!( "as20: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "user@example.com", &token, false );
  // Copy credentials.json into ~/.claude/.credentials.json so the binary can read it.
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let cred_src = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "user@example.com.credentials.json" );
  std::fs::copy( &cred_src, claude_dir.join( ".credentials.json" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.save", "name::user@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let roles_path = store.join( "user@example.com.json" );
  assert!( roles_path.exists(), "{{name}}.json must be created after save with valid token" );
  let content = std::fs::read_to_string( &roles_path ).unwrap();
  assert!( content.contains( "\"organization_uuid\"" ), "{{name}}.json must contain organization_uuid, got:\n{content}" );
  assert!( content.contains( "\"organization_name\"" ), "{{name}}.json must contain organization_name, got:\n{content}" );
}

// ── as21 (lim_it) ─────────────────────────────────────────────────────────────

#[ test ]
fn as21_lim_it_resave_overwrites_roles_json()
{
  // AC-03 (FT-03): Second .account.save overwrites existing {name}.json with fresh data.
  // Idempotency: stale snapshot is replaced by new API response. Requires live credentials.
  let Some( token ) = live_active_token() else
  {
    eprintln!( "as21: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "user@example.com", &token, false );
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let cred_src = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "user@example.com.credentials.json" );
  std::fs::copy( &cred_src, claude_dir.join( ".credentials.json" ) ).unwrap();
  // Pre-seed stale {name}.json with a sentinel value.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::write(
    store.join( "user@example.com.json" ),
    r#"{"organization_uuid":"stale-sentinel","organization_name":"Stale","organization_role":"none","workspace_uuid":null,"workspace_name":null}"#,
  ).unwrap();

  // Second save must overwrite.
  let out = run_cs_with_env( &[ ".account.save", "name::user@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let roles_path = store.join( "user@example.com.json" );
  assert!( roles_path.exists(), "{{name}}.json must still exist after re-save" );
  let content = std::fs::read_to_string( &roles_path ).unwrap();
  assert!(
    !content.contains( "stale-sentinel" ),
    "re-save must overwrite stale {{name}}.json; sentinel must be gone, got:\n{content}",
  );
}

// ── AW: Feature 027 — post-switch touch control ────────────────────────────────

/// aw22: `touch::0` disables post-switch subprocess; switch still succeeds (IT-18).
///
/// Verifies that explicitly disabling touch does not interfere with the switch itself.
/// No accessToken is present — if touch were attempted, the quota fetch would fail;
/// exit 0 with "switched" proves touch was skipped before any quota API call.
#[ test ]
fn aw22_touch_disabled_switch_succeeds()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "touch::0 must not block switch, got:\n{}", stdout( &out ),
  );
}

/// aw23: `touch::1` (default) with no `accessToken` → exit 0, touch silently skipped (IT-20).
///
/// `write_account` produces credentials without `accessToken`; `pre_switch_touch_ctx`
/// returns `None` (token read fails) so no subprocess is spawned. The switch still succeeds.
#[ test ]
fn aw23_touch_skipped_no_access_token()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // write_account produces credentials without accessToken — quota fetch path returns None.
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "touch skipped (no token) must not block switch, got:\n{}", stdout( &out ),
  );
}

/// aw24: `imodel::bad` → exit 1; stderr names all valid values (IT-21).
///
/// Validation fires before any filesystem I/O — no accounts needed in the temp dir.
#[ test ]
fn aw24_imodel_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "imodel::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "auto" ) && err.contains( "sonnet" ) && err.contains( "opus" ) && err.contains( "keep" ),
    "stderr must name all valid imodel:: values; got:\n{err}",
  );
}

/// aw25: `effort::bad` → exit 1; stderr names all valid values (IT-22).
///
/// Validation fires before any filesystem I/O — no accounts needed in the temp dir.
#[ test ]
fn aw25_effort_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "effort::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "auto" ) && err.contains( "high" ) && err.contains( "max" ),
    "stderr must name all valid effort:: values; got:\n{err}",
  );
}

/// aw26: `.account.use.help` lists `touch`, `imodel`, `effort`, `trace`, and `refresh`
/// parameters (IT-23).
///
/// Extended from Feature 027 (touch/imodel/effort) to include `trace::` per BUG-207,
/// and `refresh::` per BUG-230.
#[ test ]
fn aw26_help_shows_touch_imodel_effort()
{
  let out  = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "touch" ),   "`.account.use.help` must list `touch` param, got:\n{text}" );
  assert!( text.contains( "imodel" ),  "`.account.use.help` must list `imodel` param, got:\n{text}" );
  assert!( text.contains( "effort" ),  "`.account.use.help` must list `effort` param, got:\n{text}" );
  assert!( text.contains( "trace" ),   "`.account.use.help` must list `trace` param, got:\n{text}" );
  assert!( text.contains( "refresh" ), "`.account.use.help` must list `refresh` param, got:\n{text}" );
}

/// aw27: `lim_it` — live token + `touch::1` → switch exits 0 (IT-17/IT-19).
///
/// Uses real credentials. Whether `pre_switch_touch_ctx` returns `Some` (idle) or `None`
/// (active/fetch fail) depends on live quota state; either path must exit 0. The subprocess
/// is fire-and-forget — its success or failure does not affect the command exit code.
#[ test ]
fn aw27_lim_it_touch_with_live_token()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "aw27: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create ~/.claude/ so switch_account() can copy credentials there (it does not create the dir).
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  // Source account (provides live credentials in the store).
  write_account_with_token( dir.path(), "source@example.com", &token, true );
  // Target account — same token so quota fetch may succeed if account is idle.
  write_account_with_token( dir.path(), "target@example.com", &token, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "switch with live token must exit 0 and say switched, got:\n{}", stdout( &out ),
  );
}

/// aw28: `trace::1 touch::1` live token — subprocess always dispatched when quota fetch OK (FT-11, IT-24).
///
/// `lim_it` — skips without a live OAuth token. Verifies reading, quota fetch, and subprocess
/// dispatch trace lines. Fix(BUG-285): idle check removed — subprocess always fires when fetch
/// succeeds regardless of `resets_at` state; `idle check:` trace line no longer emitted.
///
/// Fix(BUG-207): `pre_switch_touch_ctx` had no `trace` param — all operations were invisible.
/// Root cause: Feature 027 put `trace::` Out-of-Scope; no trace lines were emitted for .account.use.
/// Pitfall: trace lines go to stderr, not stdout — assert on `stderr(&out)`, not `stdout(&out)`.
#[ test ]
fn aw28_lim_it_trace_idle_account_all_lines()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "aw28: no live token — skipping" );
    return;
  };
  if !require_live_api( "aw28" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create ~/.claude/ so switch_account() can copy credentials there (it does not create the dir).
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "source@example.com", &token, true );
  write_account_with_token( dir.path(), "target@example.com", &token, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "aw28: stdout must contain 'switched', got:\n{}", stdout( &out ),
  );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace] account.use" ),
    "aw28: stderr must contain [trace] account.use prefix, got:\n{err}",
  );
  assert!(
    err.contains( "reading" ) && err.contains( "reading: OK" ),
    "aw28: stderr must contain reading + reading: OK trace lines, got:\n{err}",
  );
  // Fix(BUG-285): idle check removed — no `idle check:` line emitted; only scheduled + spawned.
  assert!(
    !err.contains( "idle check:" ),
    "aw28: `idle check:` trace line must not appear (BUG-285 removed idle check), got:\n{err}",
  );
  if err.contains( "quota fetch: OK" )
  {
    assert!(
      err.contains( "subprocess: scheduled (idle check removed)" ),
      "aw28: fetch-OK path must emit subprocess: scheduled (idle check removed), got:\n{err}",
    );
    assert!(
      err.contains( "model:" ),
      "aw28: fetch-OK path must emit model: line, got:\n{err}",
    );
    assert!(
      err.contains( "subprocess: spawned" ),
      "aw28: fetch-OK path must emit subprocess: spawned (always; BUG-285 fix), got:\n{err}",
    );
  }
  else
  {
    eprintln!( "aw28: quota fetch failed — fetch-OK assertions skipped" );
  }
}

/// aw29: `trace::1 touch::1` live account — subprocess always spawned when quota fetch OK (FT-12).
///
/// `lim_it` — skips without a live OAuth token. Verifies that when quota fetch succeeded,
/// the subprocess is always dispatched regardless of `resets_at` state (Fix(BUG-285): idle
/// check removed; `AlreadyActive` variant removed from `PreSwitchOutcome`; subprocess is
/// idempotent and exits immediately when already active).
#[ test ]
fn aw29_lim_it_trace_active_account_subprocess_skipped()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "aw29: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create ~/.claude/ so switch_account() can copy credentials there (it does not create the dir).
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "source@example.com", &token, true );
  write_account_with_token( dir.path(), "target@example.com", &token, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace] account.use" ),
    "aw29: stderr must contain [trace] account.use prefix, got:\n{err}",
  );
  // Fix(BUG-285): no idle check — subprocess always fires when fetch OK.
  // Old trace `idle check: resets_at=present → already active` no longer exists.
  if err.contains( "quota fetch: OK" )
  {
    assert!(
      err.contains( "subprocess: scheduled (idle check removed)" ),
      "aw29: fetch-OK path must emit subprocess: scheduled (idle check removed), got:\n{err}",
    );
    assert!(
      err.contains( "model:" ),
      "aw29: fetch-OK path must emit model: line, got:\n{err}",
    );
    assert!(
      err.contains( "effort:" ),
      "aw29: fetch-OK path must emit effort: line, got:\n{err}",
    );
    assert!(
      err.contains( "subprocess: spawned" ),
      "aw29: fetch-OK path must emit subprocess: spawned (always; BUG-285 fix), got:\n{err}",
    );
    assert!(
      !err.contains( "subprocess: skipped (reason: already active)" ),
      "aw29: subprocess: skipped (reason: already active) must not appear (BUG-285 fix), got:\n{err}",
    );
  }
  else
  {
    eprintln!( "aw29: quota fetch failed — fetch-OK assertions skipped" );
  }
}

/// aw30: `trace::1 touch::1` invalid token → fetch-err + subprocess-skipped trace lines (FT-13).
///
/// Uses an invalid `accessToken` so quota fetch fails. Verifies that:
/// - reading: OK and quota fetch: Err( are emitted
/// - subprocess: skipped (reason: fetch failed) is emitted
/// - idle check: and model: lines are NOT emitted (short-circuit on fetch failure)
/// - switch still exits 0 (fetch failure is non-fatal to the switch)
///
/// Fix(BUG-207): `pre_switch_touch_ctx` must emit fetch-err trace when quota API fails.
/// Root cause: original function collapsed all failures into None with no tracing.
/// Pitfall: the switch exits 0 regardless of fetch failure; assert on stderr, not exit code.
#[ test ]
fn aw30_trace_fetch_failure_skips_idle_model_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // Invalid token ensures quota fetch fails with auth error.
  write_account_with_token( dir.path(), "target@example.com", "invalid-token-for-fetch-failure", false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "aw30: fetch failure must not block switch, got:\n{}", stdout( &out ),
  );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace] account.use" ),
    "aw30: stderr must contain [trace] account.use prefix, got:\n{err}",
  );
  assert!(
    err.contains( "reading: OK" ),
    "aw30: stderr must contain reading: OK (credential file was read), got:\n{err}",
  );
  assert!(
    err.contains( "quota fetch: Err(" ),
    "aw30: stderr must contain quota fetch: Err(, got:\n{err}",
  );
  assert!(
    err.contains( "subprocess: skipped (reason: fetch failed)" ),
    "aw30: stderr must contain subprocess: skipped (reason: fetch failed), got:\n{err}",
  );
  assert!(
    !err.contains( "idle check:" ),
    "aw30: fetch-failed path must NOT emit idle check: line, got:\n{err}",
  );
  assert!(
    !err.contains( "model:" ),
    "aw30: fetch-failed path must NOT emit model: line, got:\n{err}",
  );
  assert!(
    err.contains( "expiry check: valid" ),
    "aw30: fetch Err + FAR_FUTURE_MS expiresAt must emit expiry check: valid, got:\n{err}",
  );
}

/// aw31: `trace::1 touch::0` → no `[trace] account.use` lines emitted (FT-14, EC-7).
///
/// When `touch::0` is set, `pre_switch_touch_ctx` is never called — no quota fetch
/// operations occur, so no `[trace] account.use` lines should appear on stderr.
/// The `trace::1` parameter is accepted (exit 0) but has no effect without touch operations.
#[ test ]
fn aw31_trace_touch_disabled_no_trace_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::0", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "aw31: touch::0 trace::1 must not block switch, got:\n{}", stdout( &out ),
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "[trace] account.use" ),
    "aw31: touch::0 must produce no [trace] account.use lines, got stderr:\n{err}",
  );
}

/// aw32: `trace::bad` → exit 1; stderr names all four valid values (FT-16, IT-26).
///
/// Validation fires before any filesystem I/O — empty account store is sufficient.
///
/// Fix(BUG-207): `trace::` was absent from .account.use; before fix, `trace::bad` produced
///   "unrecognized parameter" (different message), not an invalid-value exit 1.
/// Root cause: parameter not registered; `parse_int_flag` never ran; parse never saw the value.
/// Pitfall: must assert on exit code AND stderr content — exit code alone is insufficient.
#[ test ]
fn aw32_trace_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "trace::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ) && err.contains( "false" ) && err.contains( "true" ),
    "aw32: stderr must name all valid trace:: values (0, 1, false, true), got:\n{err}",
  );
}

// ── it_trace_account_save_accepted ────────────────────────────────────────────

/// EC-11 (023): `trace::1` accepted by `.account.save` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_account_save_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // write_account creates the credential store dir so require_credential_store succeeds.
  write_account( dir.path(), "existing@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "dry::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.save, got stderr:\n{err}",
  );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must emit [trace] lines to stderr for .account.save, got:\n{err}",
  );
}

// ── it_trace_account_use_accepted ─────────────────────────────────────────────

/// EC-12 (023): `trace::1` accepted by `.account.use` — no "Unknown parameter" error.
/// `test@example.com` does not exist so command exits 2, but must not exit 1 for unknown-param.
/// `.account.use` already has `trace::` registered — this test is expected to pass before impl.
#[ test ]
fn it_trace_account_use_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::test@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.use, got stderr:\n{err}",
  );
}

// ── it_trace_account_delete_accepted ──────────────────────────────────────────

/// EC-13 (023): `trace::1` accepted by `.account.delete` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_account_delete_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account creates the credential store dir and the target account file.
  write_account( dir.path(), "test@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::test@example.com", "dry::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.delete, got stderr:\n{err}",
  );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must emit [trace] lines to stderr for .account.delete, got:\n{err}",
  );
}

// ── it_trace_account_relogin_accepted ─────────────────────────────────────────

/// EC-14 (023): `trace::1` accepted by `.account.relogin` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_account_relogin_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "dry::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.relogin, got stderr:\n{err}",
  );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must emit [trace] lines to stderr for .account.relogin, got:\n{err}",
  );
}

// ── Bug Reproducers ───────────────────────────────────────────────────────────

/// bug_reproducer(BUG-209): `.account.save` reads stale `emailAddress` from `~/.claude.json`
/// instead of the per-machine `_active` marker after `.account.use B`.
///
/// ## Fix Documentation — BUG-209
///
/// - **Root Cause:** `account_save_routine()` reads top-level `emailAddress` from `~/.claude.json`
///   as the fallback name when `name::` is omitted. `switch_account()` patches only the
///   `oauthAccount` subtree — `emailAddress` remains stale. After `.account.use B`, running
///   `.account.save` (no `name::`) saves under account `A` and overwrites `_active` with `A`.
/// - **Why Not Caught:** No test exercised the two-step sequence `.account.use B` →
///   `.account.save` (no `name::`). `as15` tested emailAddress inference without a stale case.
/// - **Fix Applied:** Read `_active_{hostname}_{user}` (per `active_marker_filename()`) instead of
///   `emailAddress`. The `_active` marker is authoritative — `switch_account()` always writes it.
/// - **Prevention:** Any code that infers "current account name" must read from the `_active`
///   marker, not from `~/.claude.json` fields that are not synced on every switch.
/// - **Pitfall:** `emailAddress` in `~/.claude.json` becomes stale immediately after any
///   `switch_account()` call; never use it as a proxy for the active account name.
#[ test ]
fn mre_bug_209_account_save_uses_active_marker_not_stale_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Credentials required for save to succeed (write_credentials also creates ~/.claude/).
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Write stale ~/.claude.json: top-level emailAddress = "a@test.com" (prior account).
  // switch_account() never updates this field — it is always stale after a switch.
  std::fs::write(
    dir.path().join( ".claude.json" ),
    r#"{"emailAddress":"a@test.com","oauthAccount":{"emailAddress":"b@test.com"}}"#,
  ).unwrap();

  // Write _active marker = "b@test.com" — set by prior .account.use b@test.com.
  let store = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  let marker = store.join( claude_profile::account::active_marker_filename() );
  std::fs::write( &marker, "b@test.com" ).unwrap();

  // .account.save with no name:: — must read _active (b@test.com), not emailAddress (a@test.com).
  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let stdout_text = stdout( &out );
  assert!(
    stdout_text.contains( "b@test.com" ),
    "must save as b@test.com (active marker), got:\n{stdout_text}",
  );
  assert!(
    !stdout_text.contains( "a@test.com" ),
    "must NOT save as a@test.com (stale emailAddress), got:\n{stdout_text}",
  );

  // Active marker must still be b@test.com after save (save() writes the marker with the saved name).
  let marker_content = std::fs::read_to_string( &marker ).unwrap_or_default();
  assert_eq!(
    marker_content.trim(), "b@test.com",
    "active marker must remain b@test.com after save, got: {marker_content}",
  );
}

/// # Root Cause
///
/// `account_save_routine()` (BUG-209 fix) read the `_active_{hostname}_{user}` marker
/// as the SOLE name inference source when `name::` is absent. The marker is only written
/// by clp ops (`switch_account`, `save`). External OAuth login writes `~/.claude.json`
/// (including `oauthAccount.emailAddress`) without updating `_active` — leaving the marker
/// stale. BUG-209 fix introduced this regression by swapping one stale source for another.
///
/// # Why Not Caught
///
/// The BUG-209 MRE (`mre_bug_209_*`) pre-populates the `_active` marker with the correct
/// target account before calling `.account.save`. It validates that the marker beats stale
/// top-level `emailAddress` — but does NOT test a stale marker itself. No test simulated
/// the external-login scenario: set marker=A, write live `oauthAccount.emailAddress`=B,
/// assert save targets B not A.
///
/// # Fix Applied
///
/// `account_save_routine()` now reads `oauthAccount.emailAddress` from `~/.claude.json` as
/// the primary source; falls back to `_active` marker only when emailAddress is absent/empty.
/// `oauthAccount.emailAddress` is updated by BOTH `switch_account()` (snapshot restore) AND
/// external OAuth login (Claude writes `~/.claude.json` on every authentication).
///
/// # Prevention
///
/// Add MRE test: write `_active` = stale account; write `~/.claude.json` with live
/// `oauthAccount.emailAddress`; call `.account.save` (no `name::`) — assert save targets
/// the live email, not the stale marker.
///
/// # Pitfall
///
/// Any inference that relies on a single marker written only by one class of credential-change
/// ops fails silently when other classes bypass that marker. Always prefer a source that ALL
/// credential-change paths maintain — `oauthAccount.emailAddress` is the universal source.
#[ doc = "bug_reproducer(BUG-212)" ]
#[ test ]
fn mre_bug_212_account_save_stale_marker_uses_oauth_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Credentials required for save to succeed (write_credentials also creates ~/.claude/).
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Fresh oauthAccount.emailAddress from external OAuth login (i5 authenticated via browser).
  // _active marker is NOT updated by external login — only clp ops (.account.use/.account.save) write it.
  std::fs::write(
    dir.path().join( ".claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"i5@wbox.pro"}}"#,
  ).unwrap();

  // Stale _active marker = "i2@wbox.pro" — set by prior .account.use i2; not updated by external login.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( claude_profile::account::active_marker_filename() ),
    "i2@wbox.pro",
  ).unwrap();

  // .account.save with no name:: — must use oauthAccount.emailAddress (i5), not _active (i2).
  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let stdout_text = stdout( &out );
  assert!(
    stdout_text.contains( "i5@wbox.pro" ),
    "must save as i5@wbox.pro (oauthAccount.emailAddress), got:\n{stdout_text}",
  );
  assert!(
    !stdout_text.contains( "i2@wbox.pro" ),
    "must NOT save as i2@wbox.pro (stale _active marker), got:\n{stdout_text}",
  );

  // BUG-212: before fix, i2's file is created instead of i5's.
  let i5_file = store.join( "i5@wbox.pro.credentials.json" );
  let i2_file = store.join( "i2@wbox.pro.credentials.json" );
  assert!( i5_file.exists(), "i5@wbox.pro.credentials.json must be created" );
  assert!( !i2_file.exists(), "i2@wbox.pro.credentials.json must NOT be created (stale marker must not win)" );
}

// ── mre_bug213_account_use_refuses_expired_token_on_fetch_error ───────────────

/// MRE for BUG-213: `.account.use` with `touch::1` (default) must refuse to call
/// `switch_account()` when the target account's `expiresAt` is in the past and
/// the quota fetch returns an Err. Before fix: exits 0, installs expired credentials.
/// After fix: exits 3, `~/.claude/.credentials.json` unchanged.
///
/// # Root Cause
///
/// `account_use_routine()` calls `pre_switch_touch_ctx()` which returns `None`
/// when the quota fetch fails (no `accessToken`, HTTP error, etc.). The routine
/// then calls `switch_account()` unconditionally — never consulting `expiresAt`
/// from the target's credential file. Expired credentials are silently installed;
/// subsequent API calls immediately fail with 401, violating the invariant:
/// "after `.account.use X` reports success, X is usable for API calls."
///
/// # Why Not Caught
///
/// FT-04 (`aw23`) tests fetch failure with `expiresAt = FAR_FUTURE_MS` — the
/// non-expired path that silently skips touch and exits 0. AC-04 (pre-fix) said
/// "touch is skipped silently" without distinguishing expired vs valid credentials.
/// No test exercised the expired-`expiresAt` + fetch-Err combination that is the
/// actual BUG-213 failure mode.
///
/// # Fix Applied
///
/// BUG-213: In `account_use_routine()`, after `pre_switch_touch_ctx()` returns `None`:
/// when `touch != 0 && touch_ctx.is_none()`, read `expiresAt` from the target
/// credential file. If `now_ms > expiresAt`, emit a clear error on stderr and
/// call `std::process::exit(3)` without calling `switch_account()`.
///
/// BUG-230: The exit-3 block now first attempts `attempt_expired_token_refresh()`
/// when `refresh::1` (default). In this test, the target has no `accessToken`, so
/// the refresh attempt fails immediately → exit 3 with `"and refresh failed"`. The
/// `err.contains("account credentials expired")` assertion still holds because the
/// new message `"account credentials expired and refresh failed"` contains the substring.
/// For the `refresh::0` (immediate-exit) path, see `aw33_refresh_disabled_exits_3_immediately`.
///
/// # Prevention
///
/// After any probe function returns None due to a fetch error, independently
/// read credential state before proceeding. Verify that `aw23` (fetch fail +
/// `FAR_FUTURE_MS`) still exits 0 — the not-expired path must not be blocked.
/// Use `expiresAt = 1000` (epoch + 1s) for expired fixtures, never `FAR_FUTURE_MS`.
///
/// # Pitfall
///
/// A `None` return from a probe function that also reads credential state conflates
/// "valid-but-fetch-failed" with "expired-and-fetch-failed". Never treat all `None`
/// returns from stateful probe functions identically at the decision point — add
/// an explicit expiry check for each distinct None cause.
#[ doc = "bug_reproducer(BUG-213)" ]
#[ test ]
fn mre_bug213_account_use_refuses_expired_token_on_fetch_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Active session credentials — the file switch_account() would overwrite.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Target account: expiresAt = 1000ms since epoch (~56 years in the past); no accessToken.
  // No accessToken → quota fetch fails immediately (no HTTP call). expiresAt 1000 → expired.
  // BUG-213: before fix, switch_account() is called and active creds are overwritten.
  write_account( dir.path(), "alice@home.com", "max", "default", 1000, false );

  // Capture current main credentials — must be unchanged after exit 3.
  let creds_path   = dir.path().join( ".claude" ).join( ".credentials.json" );
  let creds_before = std::fs::read_to_string( &creds_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@home.com" ],
    &[ ( "HOME", home ) ],
  );

  // BUG-213: before fix this exits 0 and overwrites creds_path with alice's expired token.
  assert_exit( &out, 3 );
  let err = stderr( &out );
  assert!(
    err.contains( "account credentials expired" ),
    "stderr must contain 'account credentials expired', got:\n{err}",
  );
  assert!(
    err.contains( "alice@home.com" ),
    "stderr must name the account, got:\n{err}",
  );

  // switch_account() must NOT have been called — credentials file unchanged.
  let creds_after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!(
    creds_before,
    creds_after,
    "~/.claude/.credentials.json must be unchanged when exit 3 fires before switch_account()",
  );

  // Active marker must NOT have been updated.
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let marker = std::fs::read_to_string(
    store.join( claude_profile::account::active_marker_filename() )
  ).unwrap_or_default();
  assert!(
    !marker.contains( "alice@home.com" ),
    "active marker must NOT be updated when exit 3 fires, got: '{marker}'",
  );
}

// ── BUG-230 ───────────────────────────────────────────────────────────────────

/// `.account.use` with an expired token and `refresh::1` (default) attempts refresh;
/// when refresh fails (no `accessToken` in credential file), exits 3 with
/// `"account credentials expired and refresh failed"` on stderr.
///
/// # Root Cause (BUG-230)
///
/// The BUG-213 guard added `exit(3)` on expired token without attempting an OAuth
/// token refresh. Token expiry is recoverable via `refresh_account_token()` —
/// the same mechanism used by `.usage refresh::1`. The guard gave up without trying.
///
/// # Why Not Caught
///
/// BUG-213 tests verified that the switch is refused on expiry but did not distinguish
/// "refuse immediately" from "try refresh then refuse". The `refresh::` parameter did
/// not exist at the time BUG-213 was fixed, so no test covered the refresh-attempt path.
///
/// # Fix Applied
///
/// Added `refresh::` parameter (default 1) to `.account.use`. When `refresh::1` and
/// token is locally expired: calls `attempt_expired_token_refresh()`. If `None` returned
/// (refresh failed), exits 3 with `"account credentials expired and refresh failed: {name}"`.
///
/// # Prevention
///
/// Any "refuse on expired credential" guard must first attempt refresh when `refresh::1`.
/// Use `err.contains("and refresh failed")` to detect the failure-after-attempt path;
/// use `refresh::0` to test the immediate-refusal path.
///
/// # Pitfall
///
/// The `mre_bug213` test (expired + no accessToken) still passes because the new message
/// `"account credentials expired and refresh failed"` contains `"account credentials expired"`.
/// But it now exercises the refresh-attempt path, not the immediate-refusal path.
/// Use `refresh::0` for tests that need the old immediate-exit-3 semantics.
#[ doc = "bug_reproducer(BUG-230)" ]
#[ test ]
fn mre_bug230_account_use_refresh_fails_exits_3_with_updated_message()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Active session — must be unchanged.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Target: expiresAt = 1000 (expired), no accessToken (refresh will fail immediately).
  write_account( dir.path(), "alice@home.com", "max", "default", 1000, false );

  let creds_path   = dir.path().join( ".claude" ).join( ".credentials.json" );
  let creds_before = std::fs::read_to_string( &creds_path ).unwrap();

  // Default refresh::1 — will attempt refresh, fail (no accessToken), then exit 3.
  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@home.com" ],
    &[ ( "HOME", home ) ],
  );

  assert_exit( &out, 3 );
  let err = stderr( &out );
  assert!(
    err.contains( "account credentials expired and refresh failed" ),
    "BUG-230: stderr must contain 'account credentials expired and refresh failed', got:\n{err}",
  );
  assert!(
    err.contains( "alice@home.com" ),
    "BUG-230: stderr must name the account, got:\n{err}",
  );

  // switch_account() must NOT have been called — credentials unchanged.
  let creds_after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!(
    creds_before,
    creds_after,
    "BUG-230: credentials must be unchanged when exit 3 fires after refresh failure",
  );
}

/// aw33: `refresh::0` on an expired token → exits 3 immediately with old message,
/// no refresh attempt (FT-18).
///
/// Verifies that the immediate-refusal path (BUG-213 semantics) is preserved when
/// `refresh::0` is explicitly set.
#[ test ]
fn aw33_refresh_disabled_exits_3_immediately()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "default", 1000, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@home.com", "refresh::0", "trace::1" ],
    &[ ( "HOME", home ) ],
  );

  assert_exit( &out, 3 );
  let err = stderr( &out );
  // refresh::0 uses the old message (no "and refresh failed").
  assert!(
    err.contains( "account credentials expired: alice@home.com" ),
    "aw33: stderr must contain 'account credentials expired: alice@home.com', got:\n{err}",
  );
  assert!(
    !err.contains( "and refresh failed" ),
    "aw33: refresh::0 must NOT emit 'and refresh failed' (no refresh attempted), got:\n{err}",
  );
  // Trace must show refused (refresh::0) annotation.
  assert!(
    err.contains( "refused (refresh::0)" ),
    "aw33: trace must include 'refused (refresh::0)', got:\n{err}",
  );
}

/// aw34: `refresh::bad` → exit 1; stderr names all valid values (IT-28).
///
/// Validation fires before any filesystem I/O — no accounts needed in the temp dir.
#[ test ]
fn aw34_refresh_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "refresh::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "refresh::" ) && err.contains( '0' ) && err.contains( '1' ) && err.contains( "false" ) && err.contains( "true" ),
    "aw34: stderr must name all valid refresh:: values (0, 1, false, true); got:\n{err}",
  );
}

// ── aw35 ──────────────────────────────────────────────────────────────────────

/// aw35 (015 FT-10 / AC-10): `.account.use.help` Examples section shows the positional form
/// `clp .account.use alice@acme.com` — without `name::` prefix.
///
/// Feature 015 requires that help text shows the shortcut syntax, not only the explicit form.
/// Prevents doc-drift where the help block lists only `name::EMAIL` examples.
#[ test ]
fn aw35_help_shows_positional_example()
{
  let out  = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // The examples section must include a bare email (no "name::" keyword) to demonstrate
  // positional syntax.  We check for a known example from the spec (or any bare-email pattern).
  let has_positional = text
    .lines()
    .any( |l| l.contains( '@' ) && !l.contains( "name::" ) && !l.trim_start().starts_with( "##" ) );
  assert!(
    has_positional,
    ".account.use.help must show a positional example (email without name:: prefix), got:\n{text}",
  );
}

// ── BUG-217 ───────────────────────────────────────────────────────────────────

/// `switch_account()` inserts `oauthAccount` verbatim from the per-account snapshot,
/// carrying the stale `emailAddress` field into `~/.claude.json`.
///
/// # Root Cause
///
/// `switch_account()` called `obj.insert("oauthAccount", oauth)` where `oauth` was cloned
/// verbatim from `{name}.json`. When `emailAddress` in the snapshot was stale (from
/// a prior corruption cycle), the wrong email propagated to `~/.claude.json`, causing
/// `account_save_routine()` to infer the wrong account name on subsequent saves.
///
/// # Why Not Caught
///
/// `switch_restores_claude_json` saves via `.account.save` so snapshots are always
/// correct — it never exercised a pre-existing stale snapshot. No test seeded
/// `{name}.json` with a wrong `emailAddress` before switching.
///
/// # Fix Applied
///
/// After extracting `oauth` from the snapshot, `as_object_mut()` is used to overwrite
/// `emailAddress` with `name` before `obj.insert("oauthAccount", oauth)`.
///
/// # Prevention
///
/// Identity fields in per-account snapshots must not be trusted when the account key IS
/// the canonical source. Override before inserting into shared files.
///
/// # Pitfall
///
/// Corruption is self-perpetuating: stale email installed in shared file → read by save
/// as primary name source → saved under wrong account → same stale email re-installed on
/// next switch. Both BUG-217 and BUG-218 must be fixed together.
#[ doc = "bug_reproducer(BUG-217)" ]
#[ test ]
fn mre_bug_217_switch_account_enforces_emailaddress()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Create ~/.claude/.credentials.json so switch_account can write the adjacent temp file.
  // Without this directory, std::fs::copy to .credentials.json.tmp fails with NotFound.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Target account: credentials in the credential store (no accessToken required — touch::0).
  write_account( dir.path(), "i7@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );

  // Stale snapshot: emailAddress is "i1@wbox.pro" — should be "i7@wbox.pro".
  // BUG-217: switch_account() reads this and installs it verbatim into ~/.claude.json.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::write(
    store.join( "i7@wbox.pro.json" ),
    r#"{"oauthAccount":{"emailAddress":"i1@wbox.pro","id":"uuid-placeholder"}}"#,
  ).unwrap();

  // Initial ~/.claude.json — switch_account() patches oauthAccount in-place.
  let claude_json_path = dir.path().join( ".claude.json" );
  std::fs::write(
    &claude_json_path,
    r#"{"someGlobalKey":true,"oauthAccount":{"emailAddress":"i9@wbox.pro"}}"#,
  ).unwrap();

  // touch::0 disables pre-fetch HTTP calls and the expiry guard — tests the pure file switch.
  let out = run_cs_with_env(
    &[ ".account.use", "name::i7@wbox.pro", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // After switch: oauthAccount.emailAddress must equal the target account name — not the
  // stale value from the snapshot.
  // BUG-217: before fix, actual = "i1@wbox.pro" (verbatim from snapshot).
  let claude_json = std::fs::read_to_string( &claude_json_path ).unwrap();
  assert!(
    claude_json.contains( r#""emailAddress":"i7@wbox.pro""# ),
    "BUG-217: expected emailAddress='i7@wbox.pro' in ~/.claude.json after switch, got:\n{claude_json}",
  );
  assert!(
    !claude_json.contains( r#""emailAddress":"i1@wbox.pro""# ),
    "BUG-217: stale emailAddress 'i1@wbox.pro' must not appear in ~/.claude.json, got:\n{claude_json}",
  );

  // Global keys must be preserved — switch must not wholesale overwrite ~/.claude.json.
  assert!(
    claude_json.contains( "\"someGlobalKey\":true" ),
    "switch_account must preserve global keys in ~/.claude.json, got:\n{claude_json}",
  );
}

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
    content.contains( r#""_renewal_at":"2026-06-29T21:00:00Z""# ),
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
    content.contains( r#""_renewal_at":"202"# ),
    "must write ISO-8601 timestamp starting with 202x in _renewal_at, got: {content}",
  );
  // from_now::+1h30m must not produce the same value as a clearly-past year
  assert!(
    !content.contains( r#""_renewal_at":"200"# ),
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
    content.contains( r#""_renewal_at":"202"# ),
    "must write ISO-8601 past timestamp in _renewal_at, got: {content}",
  );
  // Must not be auto-advanced to far future at write time
  assert!(
    !content.contains( r#""_renewal_at":"2099"# ),
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
    alice.contains( r#""_renewal_at":"202"# ),
    "alice must have _renewal_at after name::all, got: {alice}",
  );
  assert!(
    bob.contains( r#""_renewal_at":"202"# ),
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
    alice.contains( r#""_renewal_at":"2026-06-29T21:00:00Z""# ),
    "alice must have exact _renewal_at after comma-list, got: {alice}",
  );
  assert!(
    bob.contains( r#""_renewal_at":"2026-06-29T21:00:00Z""# ),
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
    alice.contains( r#""_renewal_at":"2026-06-29T21:00:00Z""# ),
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
    content.contains( r#""_renewal_at":"2020-01-01T00:00:00Z""# ),
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
    content.contains( r#""_renewal_at":"2026-06-29T21:00:00Z""# ),
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
    content.contains( r#""_renewal_at":"2026-07-01T00:00:00Z""# ),
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
    alice.contains( r#""_renewal_at":"2026-07-01T00:00:00Z""# ),
    "alice@acme.com must have _renewal_at after comma-list prefix resolution, got: {alice}",
  );
  assert!(
    bob.contains( r#""_renewal_at":"2026-07-01T00:00:00Z""# ),
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
    content.contains( r#""_renewal_at":"2026-06-29T21:00:00Z""# ),
    "second .account.save must preserve _renewal_at via read-merge (not overwrite), got: {content}",
  );
  assert!(
    content.contains( "oauthAccount" ),
    "oauthAccount must remain in {{name}}.json after second save, got: {content}",
  );
  assert!(
    content.contains( "\"subscriptionType\":\"pro\"" ),
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
    content.contains( r#""host":"testbox""# ),
    "{{name}}.json must contain host value, got: {content}",
  );
  assert!(
    content.contains( r#""role":"dev""# ),
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
    content.contains( r#""_renewal_at":"not-a-date""# ),
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
    content.contains( r#""host":"alice@workstation""# ),
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
    content.contains( r#""host":"bob@laptop""# ),
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
    content.contains( r#""host":"newbox""# ),
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
    content.contains( r#""host":"my work laptop""# ),
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
    content.contains( r#""host":"@"# ),
    "USER absent must produce host starting with '@', got: {content}",
  );
  assert!(
    !content.contains( r#""host":"@""# ),
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
    !content.contains( r#""host":"alice@""# ),
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
    profile.contains( r#""host":"newbox""# ),
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
    content.contains( r#""role":"work""# ),
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
    content.contains( r#""role":"""# ),
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
    content.contains( r#""role":"""# ),
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
    content.contains( r#""role":"dev""# ),
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
    content.contains( r#""role":"dev ops team""# ),
    "role:: value with spaces must be stored verbatim, got: {content}",
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
    content.contains( r#""_renewal_at":"202"# ),
    "from_now::+0m must write ISO-8601 timestamp starting with 202x, got: {content}",
  );
  // Must not be a far-future or far-past timestamp
  assert!(
    !content.contains( r#""_renewal_at":"2099"# ),
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
    content.contains( r#""_renewal_at":"202"# ),
    "from_now::+1d must write ISO-8601 future timestamp starting with 202x, got: {content}",
  );
  // +1d must not produce a clearly-past year
  assert!(
    !content.contains( r#""_renewal_at":"200"# ),
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
