//! Integration tests: H (Help), ACC (`.accounts` command).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### H — Help
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | h01 | `h01_dot_shows_help` | `.` → shows .accounts | P |
//! | h02 | `h02_help_lists_all_registered_commands` | `.help` → .accounts listed; .account.list/.account.status absent | P |
//! | h03 | `h03_help_hides_dot` | `.help` → bare `.` not listed | P |
//! | h04 | `h04_help_exits_0` | `.help` → exit 0 | P |
//! | h05 | `h05_no_args_shows_help` | no args → help shows .accounts | P |
//! | h06 | `h06_double_dash_help` | `--help` → exit 1 (POSIX flags not supported) | N |
//! | h07 | `h07_unknown_command_exits_1` | `.nonexistent` → exit 1 + stderr | N |
//!
//! ### ACC — `.accounts` command
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | acc01 | `acc01_lists_accounts_as_indented_blocks` | all accounts → indented key-val blocks | P |
//! | acc02 | `acc02_active_shows_yes_inactive_shows_no` | active=yes / inactive=no per account | P |
//! | acc03 | `acc03_empty_store_shows_advisory` | empty store → advisory message, exit 0 | P |
//! | acc04 | `acc04_name_scopes_to_single_block` | `name::EMAIL` → only that account's block | P |
//! | acc05 | `acc05_name_not_found_exits_2` | valid but unknown name → exit 2 | N |
//! | acc06 | `acc06_name_invalid_exits_1` | `name::a/b` (path-unsafe prefix) → exit 1 | N |
//! | acc07 | `acc07_field_presence_suppresses_lines` | `cols::-sub,-tier` → Sub/Tier absent | P |
//! | acc08 | `acc08_all_fields_off_bare_names` | all fields off → bare name list | P |
//! | acc09 | `acc09_json_format_array` | `format::json` → valid JSON array | P |
//! | acc10 | `acc10_json_ignores_field_presence` | `format::json` always includes all fields | P |
//! | acc11 | `acc11_missing_store_shows_advisory` | absent credential dir → advisory, exit 0 | P |
//! | acc12 | `acc12_sorted_alphabetically` | accounts listed in alpha order | P |
//! | acc13 | `acc13_blank_line_between_blocks` | multiple accounts → blank line between each block | P |
//! | acc14 | `acc14_nonactive_shows_own_stored_expires` | non-active uses own stored expires, not active's | P |
//! | acc15 | `acc15_missing_sub_field_shows_na` | missing subscriptionType in file → Sub: N/A | P |
//! | acc16 | `acc16_missing_tier_field_shows_na` | missing rateLimitTier in file → Tier: N/A | P |
//! | acc17 | `acc17_json_format_empty_store` | `format::json` + absent store → `[]` | P |
//! | acc18 | `acc18_single_account_no_trailing_blank` | single account text → no trailing blank line | P |
//! | acc19 | `acc19_missing_expires_at_shows_expired` | missing expiresAt in file → Expires: expired | P |
//! | acc20 | `acc20_display_name_shows_from_snapshot` | `cols::+display_name` → Display: from saved snapshot | P |
//! | acc21 | `acc21_role_billing_model_from_snapshots` | `cols::+role,+billing,+model` → 3 lines from snapshots | P |
//! | acc22 | `acc22_no_snapshot_shows_na_for_new_fields` | no snapshot → N/A for new fields when enabled | P |
//! | acc23 | `acc23_json_includes_new_fields` | `format::json` → includes `display_name`, role, billing, model | P |
//! | acc24 | `acc24_new_fields_absent_by_default` | no opt-in → Display/Role/Billing/Model absent | P |
//! | acc25 | `acc25_email_reads_from_snapshot` | Email: default-on → real email from snapshot | P |
//! | acc26 | `acc26_save_creates_snapshot_files` | `save` creates `{name}.json` when model present (BUG-222) | P |
//! | acc27 | `acc27_save_succeeds_without_claude_json` | save OK when `~/.claude.json` absent (best-effort) | P |
//! | acc28 | `acc28_save_succeeds_without_settings_json` | save OK when `settings.json` absent but `.claude.json` present | P |
//! | acc29 | `acc29_accounts_positional_bare_arg` | positional email → shows single account block | P |
//! | acc30 | `acc30_accounts_prefix_resolves` | prefix `alice` resolves to `alice@acme.com` | P |
//! | acc31 | `acc31_accounts_shows_current_yes_no` | live creds match work@acme.com → `Current: yes` on work, `Current: no` on alice | P |
//! | acc32 | `acc32_accounts_suppresses_current_when_creds_absent` | no live creds → no `Current:` line | P |
//! | acc33 | `acc33_accounts_current_param_and_json` | `cols::-current` → no `Current:`; `format::json` → `is_current` field | P |
//! | acc34 | `acc34_accounts_table_format` | `format::table` → exit 0, output contains column headers | P |
//! | acc35 | `acc35_uuid_shows_id_from_snapshot` | `cols::+uuid` → ID: line from saved snapshot | P |
//! | acc36 | `acc36_uuid_absent_by_default` | no `cols::+uuid` → ID: absent | P |
//! | acc37 | `acc37_json_includes_tagged_id` | `format::json` → `tagged_id` key always present | P |
//! | acc38 | `acc38_capabilities_shows_list_from_snapshot` | `cols::+capabilities` → Capabilities: from snapshot | P |
//! | acc39 | `acc39_capabilities_absent_by_default` | no `cols::+capabilities` → Capabilities: absent | P |
//! | acc40 | `acc40_json_includes_capabilities` | `format::json` → capabilities key always present | P |
//! | acc41 | `acc41_no_snapshot_uuid_capabilities_na` | no snapshot → uuid/capabilities show N/A | P |
//! | acc42 | `acc42_org_uuid_shows_from_roles_json` | `cols::+org_uuid` → Org ID: line from roles.json | P |
//! | acc43 | `acc43_org_uuid_absent_by_default` | no `cols::+org_uuid` → Org ID: absent | P |
//! | acc44 | `acc44_org_uuid_missing_roles_json_na` | `cols::+org_uuid`, no roles.json → Org ID: N/A | P |
//! | acc45 | `acc45_json_includes_org_uuid` | `format::json` → `organization_uuid` key always present | P |
//! | acc46 | `acc46_org_name_shows_from_roles_json` | `cols::+org_name` → Org: line from roles.json | P |
//! | acc47 | `acc47_org_name_absent_by_default` | no `cols::+org_name` → Org: absent | P |
//! | acc48 | `acc48_org_name_missing_roles_json_na` | `cols::+org_name`, no roles.json → Org: N/A | P |
//! | acc49 | `acc49_accounts_host_role_shows_profile_metadata` | `cols::+host,+role` → Host/Role from profile.json | P |
//! | acc50 | `acc50_accounts_host_no_profile_json_exits_0` | absent profile.json → no non-zero exit, Host: N/A | P |
//! | `mre_324_a` | `mre_324_role_toggle_shows_user_label` | `cols::+role` → user-defined label from `{name}.json`, not org role | P |
//! | `mre_324_b` | `mre_324_host_role_na_when_metadata_absent` | `cols::+host,+role`, no snapshot → Host/Role both N/A | P |
//! | `mre_324_c` | `mre_324_json_output_keys` | `format::json` → canonical AC-12 keys; no `profile_host`/`profile_role` | P |
//! | `mre_324_d` | `mre_324_json_owner_is_owned_values` | `format::json` → correct `owner`/`is_owned` values (AC-20) | P |
//! | `mre_324_e` | `mre_324_json_renewal_at_values` | `format::json` → correct `renewal_at` value and null (AC-21) | P |
//! | `mre_324_f` | `mre_324_json_is_owned_false_for_foreign_owner` | `format::json` → `is_owned: false` when owner is a foreign identity | P |
//! | `mre_324_g` | `mre_324_json_host_role_org_role_values` | `format::json` → correct `host`, `role`, `organization_role` VALUES | P |
//!
//! ### FT — Feature 037 Param Unification Tests
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ft01 | `ft01_accounts_accepts_32_params` | `.accounts` accepts all 32 registered params without error | P |
//! | ft03 | `ft03_accounts_default_profile` | `.accounts` default output includes Owner column | P |
//! | ft07 | `ft07_accounts_unclaim_batch` | `unclaim::1` no name → clears all owned accounts, exits 0 | P |
//! | ft11 | `ft11_account_unclaim_fully_deregistered` | `.account.unclaim` → exit 1 + generic error, no migration hint | N |
//! | ft12 | `ft12_account_assign_fully_deregistered` | `.account.assign` → exit 1 + generic error, no migration hint | N |
//! | ft13 | `ft13_accounts_legacy_toggles_rejected` | removed toggle param → exit 1 + migration message | N |
//! | ft14 | `ft14_accounts_cols_modifier` | `cols::+display_name` → Display: line present | P |
//! | ft15 | `lim_it_ft15_accounts_refresh_live` | `refresh::1` with live token → account appears in output | P |
//! | ft19 | `ft19_owner_column_default_visible` | Owner: line visible by default in text output | P |
//! | ft20 | `ft20_accounts_unclaim_force_bypasses_g8` | `unclaim::1 force::1` → clears owner regardless of owner identity | P |
//! | ft21 | `ft21_force_no_effect_without_unclaim` | `force::1` alone (no unclaim) → accepted, no mutation | P |
//! | it_batch_unclaim_force | `it_batch_unclaim_force_clears_non_owned` | `unclaim::1 force::1` no name → clears ALL accounts with non-empty owner, including non-owned | P |
//! | it_batch_unclaim_force_dry | `it_batch_unclaim_force_dry_previews_all` | `unclaim::1 force::1 dry::1` no name → [dry-run] for all non-empty-owner accounts, no writes | P |

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_credentials, write_claude_json_full, write_settings_json,
  write_account_claude_json, write_account_settings_json, write_live_credentials_with_token,
  write_account_claude_json_extended, write_account_roles_json, write_account_profile_json,
  write_account_owner, write_account_renewal_json, live_active_token, require_live_api,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── H: Help commands ──────────────────────────────────────────────────────────

#[ test ]
fn h01_dot_shows_help()
{
  let out  = run_cs( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".accounts" ), "help must list .accounts, got:\n{text}" );
}

#[ test ]
fn h02_help_lists_all_registered_commands()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for cmd in &[
    ".accounts",
    ".account.save",
    ".account.use",
    ".account.delete",
    ".token.status",
    ".paths",
    ".usage",
    ".credentials.status",
    ".account.limits",
  ]
  {
    assert!( text.contains( cmd ), "help must list {cmd}, got:\n{text}" );
  }
  assert!( !text.contains( ".account.list" ),   "help must not list .account.list, got:\n{text}" );
  assert!( !text.contains( ".account.status" ), "help must not list .account.status, got:\n{text}" );
}

#[ test ]
fn h03_help_hides_dot()
{
  let out   = run_cs( &[ ".help" ] );
  let text  = stdout( &out );
  // `.` is registered with `hidden_from_list: true` — must not appear as a listed command.
  // `.help` IS visible (auto-registered by unilang) — that's expected.
  let lines : Vec< &str > = text.lines()
    .filter( | l | l.trim().starts_with( '.' ) )
    .collect();
  for line in &lines
  {
    let cmd = line.split_whitespace().next().unwrap_or( "" );
    assert!( cmd != ".", "listing should not include bare '.', got line: {line}" );
  }
}

#[ test ]
fn h04_help_exits_0()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn h05_no_args_shows_help()
{
  let out  = run_cs( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".accounts" ), "no-args help must list .accounts, got:\n{text}" );
}

#[ test ]
fn h06_double_dash_help()
{
  // POSIX flags (--help, -h) are not supported — use `.help` command instead.
  let out = run_cs( &[ "--help" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "unexpected flag" ), "--help must produce unexpected flag error, got:\n{err}" );
}

#[ test ]
fn h07_unknown_command_exits_1()
{
  let out = run_cs( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "unknown command must produce stderr" );
}

// ── ACC: .accounts command ────────────────────────────────────────────────────

#[ test ]
fn acc01_lists_accounts_as_indented_blocks()
{
  // IT-1: all accounts listed as indented key-val blocks.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "work@acme.com" ),     "must list work@acme.com, got:\n{text}" );
  assert!( text.contains( "personal@home.com" ), "must list personal@home.com, got:\n{text}" );
  assert!( text.contains( "Active:" ),  "must show Active: field, got:\n{text}" );
  assert!( text.contains( "Sub:" ),     "must show Sub: field, got:\n{text}" );
  assert!( text.contains( "Expires:" ), "must show Expires: field, got:\n{text}" );
  // Exactly 2 unindented name-header lines
  let name_lines : Vec< &str > = text.lines()
    .filter( | l | !l.starts_with( ' ' ) && !l.is_empty() )
    .collect();
  assert_eq!( name_lines.len(), 2, "must have exactly 2 account name lines, got:\n{text}" );
}

#[ test ]
fn acc02_active_shows_yes_inactive_shows_no()
{
  // IT-2: active account shows Active:  yes; inactive shows Active:  no.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Active:  yes" ), "active account must show Active:  yes, got:\n{text}" );
  assert!( text.contains( "Active:  no" ),  "inactive account must show Active:  no, got:\n{text}" );
}

#[ test ]
fn acc03_empty_store_shows_advisory()
{
  // IT-3: empty credential store → advisory message, exit 0.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" )
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no accounts configured" ), "empty store must say no accounts, got:\n{text}" );
}

#[ test ]
fn acc04_name_scopes_to_single_block()
{
  // IT-4: name::EMAIL shows only that account's block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts", "name::work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(  text.contains( "work@acme.com" ),     "must show named account, got:\n{text}" );
  assert!( !text.contains( "personal@home.com" ), "must not show other account, got:\n{text}" );
  assert!(  text.contains( "Active:  yes" ), "named active account must show Active:  yes, got:\n{text}" );
}

#[ test ]
fn acc05_name_not_found_exits_2()
{
  // IT-5: valid but non-existent name → exit 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".accounts", "name::ghost@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "not found" ) || err.contains( "ghost@example.com" ),
    "must report account not found, got:\n{err}",
  );
}

#[ test ]
fn acc06_name_invalid_exits_1()
{
  // IT-6: path-unsafe prefix chars → ArgumentTypeMismatch (exit 1).
  // With resolve_account_name(): bare names (no @) are prefix queries, not email validations.
  // Path-unsafe chars (/, \, *) are still rejected with exit 1 before prefix matching runs.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn acc07_field_presence_suppresses_lines()
{
  // IT-7: cols::-sub,-tier → Sub/Tier absent; Active/Expires/Email remain.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts", "cols::-sub,-tier" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(  text.contains( "Active:" ),  "Active: must remain when cols::-sub,-tier, got:\n{text}" );
  assert!(  text.contains( "Expires:" ), "Expires: must remain when cols::-sub,-tier, got:\n{text}" );
  assert!(  text.contains( "Email:" ),   "Email: must remain when cols::-sub,-tier, got:\n{text}"  );
  assert!( !text.contains( "Sub:" ),     "Sub: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Tier:" ),    "Tier: must be suppressed, got:\n{text}" );
}

#[ test ]
fn acc08_all_fields_off_bare_names()
{
  // IT-8: all default-on fields off → bare name per line, no indented fields.
  // cols::-active,-owner,-current,-sub,-tier,-expires,-email suppresses the entire default set.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::-active,-owner,-current,-sub,-tier,-expires,-email" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 2, "all fields off must produce exactly 2 lines (names), got:\n{text}" );
  assert!( !text.contains( "Active:" ),  "Active: must be absent, got:\n{text}" );
  assert!( !text.contains( "Owner:" ),   "Owner: must be absent, got:\n{text}" );
  assert!( !text.contains( "Sub:" ),     "Sub: must be absent, got:\n{text}" );
  assert!( !text.contains( "Tier:" ),    "Tier: must be absent, got:\n{text}" );
  assert!( !text.contains( "Expires:" ), "Expires: must be absent, got:\n{text}" );
  assert!( !text.contains( "Email:" ),   "Email: must be absent, got:\n{text}" );
}

#[ test ]
fn acc09_json_format_array()
{
  // IT-9: format::json → valid JSON array with expected keys.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '[' ), "JSON must start with '[', got:\n{text}" );
  assert!( text.contains( "\"is_active\":true" ),  "active account must have is_active:true, got:\n{text}" );
  assert!( text.contains( "\"is_active\":false" ), "inactive account must have is_active:false, got:\n{text}" );
  assert!( text.contains( "\"subscription_type\"" ), "JSON must include subscription_type, got:\n{text}" );
  assert!( text.contains( "\"rate_limit_tier\"" ),   "JSON must include rate_limit_tier, got:\n{text}" );
  assert!( text.contains( "\"expires_at_ms\"" ),     "JSON must include expires_at_ms, got:\n{text}" );
}

#[ test ]
fn acc10_json_ignores_field_presence()
{
  // IT-10: format::json always includes all fields, even when field-presence params are off.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::-sub,-tier,-active", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"subscription_type\"" ), "JSON must include subscription_type despite cols::-sub, got:\n{text}" );
  assert!( text.contains( "\"rate_limit_tier\"" ),   "JSON must include rate_limit_tier despite cols::-tier, got:\n{text}" );
  assert!( text.contains( "\"is_active\"" ),          "JSON must include is_active despite cols::-active, got:\n{text}" );
}

#[ test ]
fn acc11_missing_store_shows_advisory()
{
  // IT-11: absent credential store directory → advisory, exit 0.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();
  // Deliberately do NOT create .persistent/claude/credential/

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no accounts configured" ), "absent store must say no accounts, got:\n{text}" );
  assert!( stderr( &out ).is_empty(), "absent store must not produce stderr, got:\n{}", stderr( &out ) );
}

#[ test ]
fn acc12_sorted_alphabetically()
{
  // IT-12: accounts listed in alphabetical order regardless of creation order.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "zed@acme.com",   "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "mike@acme.com",  "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::-active,-owner,-current,-sub,-tier,-expires,-email" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).map( str::trim ).collect();
  assert_eq!(
    lines,
    vec![ "alice@acme.com", "mike@acme.com", "zed@acme.com" ],
    "accounts must be sorted alphabetically, got:\n{text}",
  );
}

#[ test ]
fn acc13_blank_line_between_blocks()
{
  // IT-13: when any field is shown, a blank line separates each account block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\n\n" ),
    "multiple accounts with fields must have blank line between blocks, got:\n{text}",
  );
}

// ── acc14: P2 guard — non-active account uses its own stored expires ──────────

// Root Cause: The old `.account.status` active-account path called `status_with_threshold()`
//   which reads `~/.claude/.credentials.json` — the ACTIVE account's live credentials file.
//   For non-active accounts, a similar leak was possible. `.accounts` must always use stored
//   `expiresAt` via `token_status_from_ms()` for every account to avoid leaking the active
//   account's token state into a non-active account's Expires: line.
// Why Not Caught: Prior tests used FAR_FUTURE_MS for all accounts — no test exercised
//   a non-active account with PAST_MS while an active account had a valid token.
// Fix Applied: `accounts_routine` uses `token_status_from_ms(a.expires_at_ms)` for every
//   account in the list — the live credentials file is never read.
// Prevention: Never call `status_with_threshold()` inside `accounts_routine`; all
//   per-account data must come from the stored credential struct.
// Pitfall: Future fields that seem to require live credential reads (e.g. token validation)
//   must be refused for non-active accounts — use stored data only for consistency.
#[ doc = "bug_reproducer(BUG-276)" ]
#[ test ]
fn acc14_nonactive_shows_own_stored_expires()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Active account has a far-future (valid) token
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, true  );
  // Non-active account has an already-expired token
  write_account( dir.path(), "alice@home.com", "pro", "standard", PAST_MS,       false );

  let out  = run_cs_with_env( &[ ".accounts", "name::alice@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "expired" ),
    "non-active account must show its OWN stored expired state, not active account's valid state, got:\n{text}",
  );
  assert!(
    !text.contains( "in " ),
    "must not leak active account's valid expiry duration into non-active query, got:\n{text}",
  );
}

// ── acc15: missing subscriptionType → Sub: N/A (not blank) ───────────────────

// Root Cause: `account::list()` uses `unwrap_or_default()` for missing JSON fields,
//   yielding `""` when `subscriptionType` is absent from the credential file. Without
//   the `.is_empty()` guard, the empty string produces a blank "Sub:     " line rather
//   than "Sub:     N/A".
// Why Not Caught: All prior tests used `write_account()` which always writes non-empty
//   sub/tier values. No test used a raw credential file with a missing field.
// Fix Applied: `accounts_routine` guards with `if a.subscription_type.is_empty() { "N/A" }`
//   before formatting the Sub: line — same pattern as `credentials_status_routine`.
// Prevention: Every field read from `account::list()` for display must guard with
//   `.is_empty()` because `account::list()` returns "" for absent JSON fields.
// Pitfall: `account::list()` returns "" (not None) for missing fields; Option-based
//   patterns like `.unwrap_or("N/A")` will NOT catch it — check `.is_empty()`.
#[ doc = "bug_reproducer(BUG-269)" ]
#[ test ]
fn acc15_missing_sub_field_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  // Credential file with NO subscriptionType field — account::list() yields "" for it
  std::fs::write(
    credential_store.join( "alice@home.com.credentials.json" ),
    r#"{"oauthAccount":{"rateLimitTier":"standard"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub:     N/A" ),
    "missing subscriptionType must display 'Sub:     N/A', got:\n{text}",
  );
}

// ── acc16: missing rateLimitTier → Tier: N/A (not blank) ─────────────────────

#[ test ]
fn acc16_missing_tier_field_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  // Credential file with NO rateLimitTier field — account::list() yields "" for it
  std::fs::write(
    credential_store.join( "alice@home.com.credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"pro"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Tier:    N/A" ),
    "missing rateLimitTier must display 'Tier:    N/A', got:\n{text}",
  );
}

// ── acc17: format::json + absent store → [] ───────────────────────────────────

/// acc17: `format::json` with no credential store directory → returns `[]`.
///
/// Root Cause: The `Json` branch has an explicit `if accounts.is_empty()` guard
///   that returns `"[]\n"` — this code path was not directly tested.
/// Why Not Caught: acc09 (json format) requires accounts to be present; acc11
///   (absent store) uses text format only. The intersection was untested.
/// Fix Applied: No fix needed — the guard was already correct. Test confirms it.
/// Prevention: For every format × store-state combination (json+empty, text+empty)
///   add an explicit test — do not assume format handling is symmetric.
/// Pitfall: An empty JSON array `[]` and the text advisory `(no accounts configured)`
///   are NOT interchangeable — callers of `format::json` must parse `[]`, not text.
#[ test ]
fn acc17_json_format_empty_store()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Deliberately do NOT create .persistent/claude/credential/ — account::list returns []
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '[' ),
    "json format must start with '[', got:\n{text}",
  );
  assert!(
    text.contains( "[]" ),
    "json format with absent store must return empty array '[]', got:\n{text}",
  );
  assert!(
    !text.contains( "no accounts" ),
    "json format must not return text advisory, got:\n{text}",
  );
}

// ── acc18: single account text → no trailing blank line ───────────────────────

/// acc18: A single account in text mode produces no trailing blank line.
///
/// Root Cause: `render_accounts_text` adds a blank separator only between blocks
///   (`if idx < last_idx`). For a single account (`idx=0, last_idx=0`) the condition
///   is false — no blank line is appended after the final block.
/// Why Not Caught: acc13 confirms a blank line EXISTS between two accounts, but
///   never asserts the last block has no trailing blank. acc04 confirms single-block
///   content but does not check for absence of trailing blank.
/// Fix Applied: No fix needed — the guard was already correct. Test confirms it.
/// Prevention: When testing separator logic, test both the presence case (multiple
///   blocks) and the absence case (single block) explicitly.
/// Pitfall: A trailing blank line in text output breaks scripts that read the last
///   line of output — always verify last-block is not followed by a blank.
#[ test ]
fn acc18_single_account_no_trailing_blank()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "solo@example.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "solo@example.com" ),
    "must list the sole account, got:\n{text}",
  );
  assert!(
    !text.ends_with( "\n\n" ),
    "single account must not have a trailing blank line, got:\n{text:?}",
  );
}

// ── acc19: missing expiresAt → Expires: expired ───────────────────────────────

/// acc19: A credential file missing `expiresAt` is displayed as expired.
///
/// Root Cause: `account::list()` calls `parse_u64_field(&content, "expiresAt")`
///   which returns `None` for a missing field, then `unwrap_or(0)` yields 0 ms
///   (Unix epoch). Any current time is >> 0 ms → `token_status_from_ms(0)` →
///   `TokenStatus::Expired` → `Expires: expired`.
/// Why Not Caught: All prior tests use `write_account()` which always writes a
///   non-zero `expiresAt`. No test used a raw credential file with the field absent.
/// Fix Applied: No fix needed — `unwrap_or(0)` correctly maps missing → expired.
///   Test documents the contract and prevents future regressions.
/// Prevention: When adding or changing `parse_u64_field` call sites, verify
///   the fallback for a missing field produces the expected sentinel behaviour.
/// Pitfall: A missing `expiresAt` silently renders as expired — do not mistake
///   this for a credential-read error; the account IS listed, just marked expired.
#[ test ]
fn acc19_missing_expires_at_shows_expired()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  // No `expiresAt` field → parse_u64_field returns None → unwrap_or(0) → epoch → expired
  std::fs::write(
    credential_store.join( "ghost@example.com.credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"pro","rateLimitTier":"standard"}}"#,
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "ghost@example.com" ),
    "must list the account, got:\n{text}",
  );
  assert!(
    text.contains( "Expires: expired" ),
    "missing expiresAt must display 'Expires: expired', got:\n{text}",
  );
}

// ── acc20–acc28: Rich account metadata (FR-20, feature/014) ──────────────────

/// acc20 (T01): `display_name::1` renders `Display:` line from saved snapshot.
///
/// Root Cause (before fix): `Account` struct lacked `display_name` field; `list()` never
///   read snapshot files; `render_accounts_text()` did not accept `show_display_name` param.
/// Why Not Caught: All prior tests used only the 5 original Account fields.
/// Fix Applied: `Account` gains `display_name`; `list()` reads `{name}.json`;
///   `render_accounts_text()` renders `Display:` when `show_display_name` is true.
/// Prevention: Whenever adding opt-in fields, write a snapshot-present test (acc20)
///   and a snapshot-absent test (acc22) to cover both code paths.
/// Pitfall: `parse_string_field()` searches the flat string — it finds nested keys like
///   `"displayName"` regardless of JSON depth. Do NOT add custom JSON parsing.
#[ test ]
fn acc20_display_name_shows_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe" );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+display_name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Display: Alice K" ),
    "cols::+display_name must render Display: line from snapshot, got:\n{text}",
  );
}

/// acc21 (T02+T03+T04): `role::1 billing::1 model::1` renders three lines.
///
/// After TSK-225: `role::1` shows the profile.json role label (`profile_role` field), not
/// the OAuth snapshot org role. The test now writes `profile.json` with `role: "admin"`
/// so the assertion "Role: admin" still holds — the source is profile.json, not
/// the claude.json snapshot. Snapshot org role lives in `a.role` and is still surfaced
/// via the JSON format output.
///
/// Root Cause (before fix): `Account` struct lacked `role`, `billing`, `model` fields;
///   `list()` did not read snapshot files; rendering did not handle these params.
/// Why Not Caught: Only original 5 fields were tested.
/// Fix Applied: `Account` gains `role`, `billing`, `model`; `list()` reads both snapshot
///   files; `render_accounts_text()` renders the three new lines when enabled.
/// Prevention: Test all three in one function to catch the common mistake of reading
///   one snapshot file but forgetting the other.
/// Pitfall: `model` comes from `{name}.json`, not `{name}.json`. A single
///   snapshot read call is insufficient — both files must be read independently.
#[ test ]
fn acc21_role_billing_model_from_snapshots()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe_sub" );
  write_account_settings_json( dir.path(), "alice@acme.com", "claude-sonnet" );
  // role::1 now shows profile.json role label (TSK-225); write profile.json so the
  // "Role: admin" assertion holds with the new profile-based semantics.
  write_account_profile_json( dir.path(), "alice@acme.com", None, Some( "admin" ) );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+role,+billing,+model" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Role:    admin"         ), "cols::+role must render Role: from profile.json, got:\n{text}"  );
  assert!( text.contains( "Billing: stripe_sub"    ), "cols::+billing must render Billing: from snapshot, got:\n{text}" );
  assert!( text.contains( "Model:   claude-sonnet" ), "cols::+model must render Model: from snapshot, got:\n{text}"  );
}

/// acc22 (T05): when no snapshot files exist, opt-in fields show `N/A`.
///
/// Root Cause (before fix): Without snapshot files, `list()` would yield empty strings
///   for all new fields, but `render_accounts_text()` lacked the empty-string → N/A guard.
/// Why Not Caught: Implementation gap: snapshot reading not yet coded.
/// Fix Applied: `list()` uses `unwrap_or_default()` → empty string; `render_accounts_text()`
///   guards each new field with `if field.is_empty() { "N/A" }`.
/// Prevention: Always pair a snapshot-absent test with each snapshot-present test so both
///   the reading path and the fallback path are verified.
/// Pitfall: `unwrap_or_default()` on a missing file yields `""` — callers must guard
///   against empty string, not `None`. Pattern: `if s.is_empty() { "N/A" } else { s }`.
#[ test ]
fn acc22_no_snapshot_shows_na_for_new_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No snapshot files written — all new fields must fall back to N/A.

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+display_name,+role,+billing,+model" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Display: N/A" ), "absent snapshot must show Display: N/A, got:\n{text}" );
  assert!( text.contains( "Role:    N/A" ), "absent snapshot must show Role:    N/A, got:\n{text}" );
  assert!( text.contains( "Billing: N/A" ), "absent snapshot must show Billing: N/A, got:\n{text}" );
  assert!( text.contains( "Model:   N/A" ), "absent snapshot must show Model:   N/A, got:\n{text}" );
}

/// acc23 (T06): `format::json` always includes the four new field keys.
///
/// Root Cause (before fix): JSON format string in `accounts_routine()` hardcoded only
///   legacy fields; no `display_name`, `role`, `billing`, `model` keys were emitted.
/// Why Not Caught: acc10 (`json_ignores_field_presence`) only checked original fields.
/// Fix Applied: Extend the JSON format string with all four new fields using
///   `json_escape(&a.display_name)` etc. — matches the `.credentials.status` JSON pattern.
/// Prevention: When adding struct fields, always extend BOTH text rendering AND JSON output
///   in the same phase to avoid silent key omissions.
/// Pitfall: `format::json` emits all fields unconditionally — do NOT gate new JSON keys
///   on the field-presence booleans (`show_display_name` etc.); those control text only.
#[ test ]
fn acc23_json_includes_new_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe_sub" );
  write_account_settings_json( dir.path(), "alice@acme.com", "claude-sonnet" );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"email\""         ), "JSON must include email key, got:\n{text}"        );
  assert!( text.contains( "\"display_name\"" ), "JSON must include display_name key, got:\n{text}" );
  assert!( text.contains( "\"role\""         ), "JSON must include role key, got:\n{text}"         );
  assert!( text.contains( "\"billing\""      ), "JSON must include billing key, got:\n{text}"      );
  assert!( text.contains( "\"model\""        ), "JSON must include model key, got:\n{text}"        );
  assert!( text.contains( "alice@acme.com"   ), "JSON email must contain actual value, got:\n{text}"        );
  assert!( text.contains( "Alice K"          ), "JSON display_name must contain actual value, got:\n{text}" );
  assert!( text.contains( "claude-sonnet"    ), "JSON model must contain actual value, got:\n{text}"        );
}

/// acc24 (T07): new opt-in fields absent from output by default.
///
/// Root Cause (invariant guard): Opt-in fields use `Some(Value::Boolean(true))` without
///   `| None` fallback, so absence of the param = field hidden. No `None` in the match
///   is the ONLY difference from default-on params.
/// Why Not Caught: No test verified that the new params are truly off by default.
/// Fix Applied: Invariant confirmed by test; `accounts_routine()` reads each new param
///   with `matches!(..., Some(Value::Boolean(true)))` (no `None`).
/// Prevention: For every opt-in param, pair an opt-in-enabled test (acc20) with an
///   opt-in-absent test (acc24) so regressions to default-on are caught immediately.
/// Pitfall: Adding `| None` to an opt-in param silently makes it default-on — a
///   runtime-invisible change that only a test like this one catches.
#[ test ]
fn acc24_new_fields_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe" );
  write_account_settings_json( dir.path(), "alice@acme.com", "claude-sonnet" );

  // Default .accounts call — no opt-in params.
  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Display:" ), "Display: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Role:"    ), "Role: must be absent by default, got:\n{text}"    );
  assert!( !text.contains( "Billing:" ), "Billing: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Model:"   ), "Model: must be absent by default, got:\n{text}"   );
}

/// acc25 (T08): `Email:` default-on renders real email from saved snapshot.
///
/// Root Cause (before fix): `list()` read `organizationName` for `Account.org`; the
///   `emailAddress` field was never read from the per-account snapshot file.
/// Why Not Caught: No test verified that Email: shows the actual stored emailAddress value.
/// Fix Applied: `list()` reads `{name}.json` → `emailAddress` and populates
///   `Account.email`; `render_accounts_text()` uses `a.email` with empty-string → N/A guard.
/// Prevention: When a display value is derived from a data source, write a test that
///   verifies the ACTUAL VALUE appears — not just the label line.
/// Pitfall: An empty-string fallback silently shows "N/A" — always add a snapshot-present
///   test like this one so the read path is exercised, not just the fallback.
#[ test ]
fn acc25_email_reads_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe" );

  // Email is default-on — no toggle needed.
  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Email:   alice@acme.com" ),
    "Email: must show real email from snapshot, got:\n{text}",
  );
  assert!(
    !text.contains( "Email:   N/A" ),
    "Email: must not show N/A when snapshot has emailAddress, got:\n{text}",
  );
}

/// acc26 (T09 — save with claude.json + settings.json): `account::save` writes credential
/// and `.json` snapshot files when all sources are present (BUG-222 fix).
///
/// Root Cause (before fix): `save()` only called `std::fs::copy(paths.credentials_file(), dest)`.
///   The `oauthAccount` data from `~/.claude.json` was never persisted to the credential store.
/// Why Not Caught: No save test verified the presence of snapshot files after save.
/// Fix Applied: `save()` surgically extracts the `oauthAccount` subtree from `~/.claude.json`
///   and writes it to `{name}.json`. BUG-222 additionally captures `model` from
///   `~/.claude/settings.json` into `{name}.json`, enabling model preference restore
///   on `switch_account()`. When settings.json source is absent, no snapshot is written (acc28).
/// Prevention: After any `save()` implementation change, verify ALL expected output files exist.
///   `settings.json` snapshot must be created when source model is present; absent when source
///   is absent (see acc28 for the no-source case).
/// Pitfall: The `oauthAccount` extraction silently skips if the key is absent — this is
///   intentional best-effort, but means a wrong source path would silently produce no output.
///   This test catches that by asserting `{name}.json` EXISTS after save.
#[ test ]
fn acc26_save_creates_snapshot_files()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Source files that save() will copy.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "alice@acme.com", "Alice K", "admin", "stripe" );
  write_settings_json( dir.path(), "claude-sonnet" );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must create credentials snapshot, store: {}", store.display(),
  );
  assert!(
    store.join( "alice@acme.com.json" ).exists(),
    "save must create .json snapshot, store: {}", store.display(),
  );
  // Fix(BUG-222): save() captures model from ~/.claude/settings.json into {name}.json.
  //   write_settings_json above created the source with "claude-sonnet" → snapshot must exist.
  let settings_snap = store.join( "alice@acme.com.json" );
  assert!(
    settings_snap.exists(),
    "save must create settings.json snapshot when source model present (BUG-222), store: {}", store.display(),
  );
  let settings_content = std::fs::read_to_string( &settings_snap ).unwrap();
  assert!(
    settings_content.contains( "claude-sonnet" ),
    "settings.json snapshot must contain the source model value, got: {settings_content}",
  );
}

/// acc27 (T09 — save without `~/.claude.json`): save succeeds even when source is absent.
///
/// Root Cause (before fix): `save()` only copied credentials; no `.json` snapshot
///   was ever created. After BUG-174, oauthAccount extraction was added but must silently
///   skip if `~/.claude.json` is absent or unparseable.
/// Why Not Caught: All prior save tests relied on a credentials file being present;
///   no test verified the best-effort behaviour for the optional `.json` source.
/// Fix Applied: `save()` wraps the oauthAccount extraction in `if let Ok(text) = read_to_string(...)` —
///   absent or malformed `~/.claude.json` silently skips; credential copy still uses `?` (required).
/// Prevention: For every best-effort file operation, add a test where the source is absent
///   to confirm the operation succeeds and no partial output is written.
/// Pitfall: Silently-discarded read errors mean a wrong path never fails — always add an
///   absent-source test to confirm no snapshot is created when source is missing.
#[ test ]
fn acc27_save_succeeds_without_claude_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only credentials file — no ~/.claude.json, no settings.json.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must still create credential file when snapshots absent",
  );
  // Unified {{name}}.json is always created but must not contain oauthAccount (no source).
  let meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) )
    .unwrap_or_default();
  assert!(
    !meta.contains( "oauthAccount" ),
    "no ~/.claude.json → {{name}}.json must not contain oauthAccount, got: {meta}",
  );
}

/// acc28 (T09 — save with `.claude.json` but without `settings.json`): confirms oauthAccount
/// extraction succeeds when `settings.json` source is absent; `{name}.json` is created
/// with oauthAccount but no `model` field when `settings.json` has no model.
///
/// Root Cause (before fix): After the initial snapshot feature was added, `save()` tried
///   to read `settings.json`; a missing file could interfere. BUG-222 made model capture
///   best-effort: `save()` reads `model` from `~/.claude/settings.json` when present.
/// Why Not Caught: No test verified that `settings.json` absence did not affect the
///   `{name}.json` snapshot creation.
/// Fix Applied (BUG-222): `save()` merges `model` from `~/.claude/settings.json` into
///   `{name}.json` when present. When source is absent, `{name}.json` is still written
///   with whatever other data is available (e.g. oauthAccount).
/// Prevention: Verify `{name}.json` contains oauthAccount but no model when source absent.
/// Pitfall: `save()` only captures the `model` field, not the entire `settings.json`;
///   machine-global keys (commands.*, mcpServers) are never stored per-account.
#[ test ]
fn acc28_save_succeeds_without_settings_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Credentials + .claude.json present; settings.json absent.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "alice@acme.com", "Alice K", "admin", "stripe" );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must create credential snapshot",
  );
  let meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) )
    .expect( "save must create {{name}}.json" );
  assert!(
    meta.contains( "oauthAccount" ),
    "{{name}}.json must contain oauthAccount from ~/.claude.json, got: {meta}",
  );
  assert!(
    !meta.contains( "\"model\"" ),
    "no settings.json source → {{name}}.json must not contain model, got: {meta}",
  );
}

// ── acc29 ─────────────────────────────────────────────────────────────────────

#[ test ]
fn acc29_accounts_positional_bare_arg()
{
  // AC-03: positional form `clp .accounts alice@acme.com` is equivalent to
  // `clp .accounts name::alice@acme.com` — shows only that account block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".accounts", "alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show alice@acme.com, got:\n{text}" );
  assert!( !text.contains( "work@acme.com" ), "must not show work@acme.com, got:\n{text}" );
}

// ── acc30 ─────────────────────────────────────────────────────────────────────

#[ test ]
fn acc30_accounts_prefix_resolves()
{
  // AC-05 (accounts): prefix `alice` resolves to `alice@acme.com` — shows only that block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );

  let out = run_cs_with_env( &[ ".accounts", "alice" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "prefix alice must resolve to alice@acme.com, got:\n{text}" );
  assert!( !text.contains( "work@acme.com" ), "must not show work@acme.com, got:\n{text}" );
}

// ── acc31 ─────────────────────────────────────────────────────────────────────

/// acc31 (IT-26): live creds `accessToken` matches `work@acme.com` → `Current: yes` on that
/// account and `Current: no` on `alice@acme.com`.
///
/// Both saved accounts have `accessToken` fields (via `write_account_with_token`) so
/// `detect_current_account()` can compare them against the live creds.
#[ test ]
fn acc31_accounts_shows_current_yes_no()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Each account block: find work@ and alice@ sections and verify Current: line.
  let lines : Vec< &str > = text.lines().collect();
  let work_idx  = lines.iter().position( |l| l.contains( "work@acme.com"  ) );
  let alice_idx = lines.iter().position( |l| l.contains( "alice@acme.com" ) );

  let work_idx  = work_idx.expect( "work@acme.com not found in output" );
  let alice_idx = alice_idx.expect( "alice@acme.com not found in output" );

  // Find Current: line near each account header (within the next 10 lines).
  let work_block  = &lines[ work_idx  ..( work_idx  + 10 ).min( lines.len() ) ];
  let alice_block = &lines[ alice_idx ..( alice_idx + 10 ).min( lines.len() ) ];

  assert!(
    work_block.iter().any( |l| l.contains( "Current:" ) && l.contains( "yes" ) ),
    "work@acme.com block must have 'Current: yes', got block:\n{}", work_block.join( "\n" ),
  );
  assert!(
    alice_block.iter().any( |l| l.contains( "Current:" ) && l.contains( "no" ) ),
    "alice@acme.com block must have 'Current: no', got block:\n{}", alice_block.join( "\n" ),
  );
}

// ── acc32 ─────────────────────────────────────────────────────────────────────

/// acc32 (IT-27): no live credentials file → `Current:` line is suppressed entirely.
///
/// When `~/.claude/.credentials.json` is absent, the detection algorithm cannot match
/// any account and the `Current:` line must not appear at all (not even `Current: no`).
#[ test ]
fn acc32_accounts_suppresses_current_when_creds_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );
  // Deliberately do NOT write live credentials file.

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Current:" ),
    "Current: line must be absent when creds file is missing, got:\n{text}",
  );
}

// ── acc33 ─────────────────────────────────────────────────────────────────────

/// acc33 (IT-28): two sub-tests for the `current::` parameter and JSON `is_current` field.
///
/// (a) `current::0` suppresses the `Current:` line even when live creds are present.
/// (b) `format::json` output includes an `is_current` boolean field per account object.
#[ test ]
fn acc33_accounts_current_param_and_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-alice" );

  // (a) cols::-current must suppress the Current: line.
  let out_off = run_cs_with_env( &[ ".accounts", "cols::-current" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_off, 0 );
  let text_off = stdout( &out_off );
  assert!(
    !text_off.contains( "Current:" ),
    "cols::-current must suppress Current: line, got:\n{text_off}",
  );

  // (b) format::json must include is_current boolean field.
  let out_json = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_json, 0 );
  let json = stdout( &out_json );
  assert!(
    json.contains( "\"is_current\"" ),
    "JSON output must include is_current field, got:\n{json}",
  );
}

// ── acc34 ─────────────────────────────────────────────────────────────────────

/// acc34 (IT-34): `format::table` renders a `data_fmt` ASCII table with column headers.
///
/// # Root Cause
/// Task 131 adds `OutputFormat::Table` to `.accounts`. Before implementation this
/// exits 1 with `"unknown format 'table'"`.
///
/// # Why Not Caught
/// New feature; no prior test existed.
///
/// # Fix Applied
/// Added `OutputFormat::Table` variant to `output.rs`, `"table"` parse arm in
/// `from_cmd()`, and `render_accounts_table()` in `commands.rs`.
///
/// # Prevention
/// Covered by this test: two accounts saved; `format::table` asserted to exit 0
/// and contain `Account` header (column header from `data_fmt` table).
///
/// # Pitfall
/// `format::table` for non-`.accounts` commands must exit 1. Only `.accounts`
/// accepts this format; all other routines reject with `ArgumentTypeMismatch`.
#[ test ]
fn acc34_accounts_table_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "work@acme.com",  "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts", "format::table" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Account" ),
    "format::table must include 'Account' column header, got:\n{text}",
  );
  assert!(
    text.contains( "alice@acme.com" ),
    "format::table must include alice@acme.com in output, got:\n{text}",
  );
  assert!(
    text.contains( "work@acme.com" ),
    "format::table must include work@acme.com in output, got:\n{text}",
  );
}

// ── acc35–acc41: uuid:: and capabilities:: on .accounts (FR-21) ──────────────

/// acc35: `uuid::1` shows `ID:` line from saved snapshot.
#[ test ]
fn acc35_uuid_shows_id_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "user_abc123", "some-uuid", &[ "claude_code" ] );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+uuid" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "ID:" ),         "cols::+uuid must emit ID: line, got:\n{text}" );
  assert!( text.contains( "user_abc123" ), "ID: must show taggedId from snapshot, got:\n{text}" );
}

/// acc36: Default — `ID:` absent when `uuid::` not specified.
#[ test ]
fn acc36_uuid_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "user_abc123", "some-uuid", &[] );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "ID:" ), "ID: must be absent by default, got:\n{text}" );
}

/// acc37: `format::json` always includes `tagged_id` key regardless of `uuid::`.
#[ test ]
fn acc37_json_includes_tagged_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "user_abc123", "some-uuid", &[] );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"tagged_id\"" ), "format::json must include tagged_id key, got:\n{text}" );
  assert!( text.contains( "user_abc123" ),   "tagged_id must contain the snapshot value, got:\n{text}" );
}

/// acc38: `capabilities::1` shows `Capabilities:` line from saved snapshot.
#[ test ]
fn acc38_capabilities_shows_list_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "", "", &[ "claude_max", "chat" ] );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+capabilities" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Capabilities:" ), "cols::+capabilities must emit Capabilities: line, got:\n{text}" );
  assert!( text.contains( "claude_max" ),    "Capabilities: must list claude_max, got:\n{text}" );
  assert!( text.contains( "chat" ),          "Capabilities: must list chat, got:\n{text}" );
}

/// acc39: Default — `Capabilities:` absent when `capabilities::` not specified.
#[ test ]
fn acc39_capabilities_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "", "", &[ "claude_max" ] );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Capabilities:" ), "Capabilities: must be absent by default, got:\n{text}" );
}

/// acc40: `format::json` always includes `capabilities` key.
#[ test ]
fn acc40_json_includes_capabilities()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "", "", &[ "claude_max" ] );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"capabilities\"" ), "format::json must include capabilities key, got:\n{text}" );
  assert!( text.contains( "claude_max" ),       "capabilities must contain the snapshot value, got:\n{text}" );
}

/// acc41: No snapshot → `ID: N/A` and `Capabilities: N/A` when opted in.
#[ test ]
fn acc41_no_snapshot_uuid_capabilities_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No snapshot files written.

  let out  = run_cs_with_env( &[ ".accounts", "cols::+uuid,+capabilities" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "ID:" ),           "ID: line must appear with cols::+uuid, got:\n{text}" );
  assert!( text.contains( "Capabilities:" ), "Capabilities: line must appear, got:\n{text}" );
  assert!( text.contains( "N/A" ),           "absent snapshot must show N/A for new fields, got:\n{text}" );
}

// ── acc42 ─────────────────────────────────────────────────────────────────────

/// acc42 (EC-1): `org_uuid::1` shows `Org ID:` line with value from `{name}.json`.
#[ test ]
fn acc42_org_uuid_shows_from_roles_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_uuid" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org ID:" ),    "cols::+org_uuid must emit Org ID: line, got:\n{text}" );
  assert!( text.contains( "org-xyz-789" ), "Org ID: must show organization_uuid from roles.json, got:\n{text}" );
}

// ── acc43 ─────────────────────────────────────────────────────────────────────

/// acc43 (EC-4): Default — `Org ID:` absent when `org_uuid::` not specified.
#[ test ]
fn acc43_org_uuid_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org ID:" ), "Org ID: must be absent by default, got:\n{text}" );
}

// ── acc44 ─────────────────────────────────────────────────────────────────────

/// acc44 (EC-7): Missing roles.json → `Org ID:  N/A` when `org_uuid::1`.
#[ test ]
fn acc44_org_uuid_missing_roles_json_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No roles.json written.

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_uuid" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org ID:" ), "Org ID: line must appear with cols::+org_uuid, got:\n{text}" );
  assert!( text.contains( "N/A" ),     "absent roles.json must show N/A, got:\n{text}" );
}

// ── acc45 ─────────────────────────────────────────────────────────────────────

/// acc45 (EC-6): `format::json` always includes `organization_uuid` key.
#[ test ]
fn acc45_json_includes_org_uuid()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"organization_uuid\"" ), "JSON must include organization_uuid key, got:\n{text}" );
  assert!( text.contains( "org-xyz-789" ),           "organization_uuid must contain the snapshot value, got:\n{text}" );
}

// ── acc46 ─────────────────────────────────────────────────────────────────────

/// acc46 (EC-1): `org_name::1` shows `Org:` line with value from `{name}.json`.
#[ test ]
fn acc46_org_name_shows_from_roles_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ),      "cols::+org_name must emit Org: line, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ), "Org: must show organization_name from roles.json, got:\n{text}" );
}

// ── acc47 ─────────────────────────────────────────────────────────────────────

/// acc47 (EC-4): Default — `Org:` absent when `org_name::` not specified.
#[ test ]
fn acc47_org_name_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org:" ), "Org: must be absent by default, got:\n{text}" );
}

// ── acc48 ─────────────────────────────────────────────────────────────────────

/// acc48 (EC-7): Missing roles.json → `Org:     N/A` when `org_name::1`.
#[ test ]
fn acc48_org_name_missing_roles_json_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No roles.json written.

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ), "Org: line must appear with cols::+org_name, got:\n{text}" );
  assert!( text.contains( "N/A" ),  "absent roles.json must show N/A, got:\n{text}" );
}

// ── it_trace_accounts_accepted ─────────────────────────────────────────────────

/// EC-9 (023): `trace::1` accepted by `.accounts` on empty store — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_accounts_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".accounts", "trace::1" ], &[ ( "HOME", home ) ] );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .accounts, got stderr:\n{err}",
  );
  assert_exit( &out, 0 );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must emit [trace] lines to stderr for .accounts, got:\n{err}",
  );
}

// ── acc49: host::1 role::1 shows profile metadata ────────────────────────────

/// acc49 — `.accounts host::1 role::1` shows Host and Role from profile.json.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-08]
#[ test ]
fn acc49_accounts_host_role_shows_profile_metadata()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "test@example.com", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+host,+role" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host:    mybox" ),
    "cols::+host must show Host: from profile.json, got:\n{text}",
  );
  assert!(
    text.contains( "Role:    work" ),
    "cols::+role must show Role: from profile.json, got:\n{text}",
  );
}

// ── acc50: absent profile.json — host::1 exits 0, shows N/A ──────────────────

/// acc50 — absent `profile.json` must not cause any command to exit non-zero.
///
/// When `host::1` is given but no `{name}.json` exists, the Host field
/// shows `N/A` (empty → fallback) and exit is 0. Resilience spec.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-09]
#[ test ]
fn acc50_accounts_host_no_profile_json_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // No profile.json written — must be treated as optional metadata.

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+host" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host:    N/A" ),
    "absent profile.json must show Host: N/A (not error), got:\n{text}",
  );
}

// ── acc51 ─────────────────────────────────────────────────────────────────────

#[ test ]
fn acc51_accounts_positional_after_key_value()
{
  // BUG-294: reversed arg order `clp .accounts format::json alice@acme.com` — key::value
  // before bare name — must rewrite positional arg regardless of argv position.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json", "alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "reversed-order positional must show alice@acme.com, got:\n{text}" );
  assert!( !text.contains( "work@acme.com" ), "must not show work@acme.com, got:\n{text}" );
}


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

  // active:: + name:: + dry::1 accepted (Feature 064 ownership mutation).
  let out = run_cs_with_env(
    &[ ".accounts", "active::testuser@testmachine", "name::alice@acme.com", "dry::1" ],
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
/// With `trace::1`, no `[trace] fetch` or `[trace] touch` lines should appear.
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
    !err.contains( "[trace] fetch" ),
    "FT-03: default .accounts must NOT produce [trace] fetch lines (no network call); got stderr:\n{err}",
  );
  assert!(
    !err.contains( "[trace] touch" ),
    "FT-03: default .accounts must NOT produce [trace] touch lines; got stderr:\n{err}",
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
/// FT-11 (AC-11): `.account.unclaim` is fully deregistered — generic "unknown command" error, no migration hint.
///
/// `.account.unclaim` was removed in Feature 037 with no redirect stub.
/// Calling it is indistinguishable from calling any other unregistered command:
/// exits 1 with a generic error; stderr must NOT contain `"unclaim::1"` or `"moved to"`.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-11]
fn ft11_account_unclaim_fully_deregistered()
{
  let out = run_cs( &[ ".account.unclaim", "name::alice@acme.com" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "FT-11: .account.unclaim must produce a non-empty error on stderr",
  );
  assert!(
    !err.contains( "unclaim::1" ) && !err.contains( "moved to" ),
    "FT-11: error must be generic (no migration hint to .accounts unclaim::1); got:\n{err}",
  );
}

#[ test ]
/// FT-12 (AC-12): `.account.assign` is fully deregistered — generic "unknown command" error, no migration hint.
///
/// `.account.assign` was removed in Feature 037 with no redirect stub.
/// Calling it is indistinguishable from calling any other unregistered command:
/// exits 1 with a generic error; stderr must NOT contain `"assign::1"` or `"moved to"`.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-12]
fn ft12_account_assign_fully_deregistered()
{
  let out = run_cs( &[ ".account.assign", "name::alice@acme.com" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "FT-12: .account.assign must produce a non-empty error on stderr",
  );
  assert!(
    !err.contains( "assign::1" ) && !err.contains( "moved to" ),
    "FT-12: error must be generic (no migration hint to .accounts assign::1); got:\n{err}",
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

  // "active" removed from this list — Feature 064 repurposed it as Kind::String (active::USER@MACHINE).
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
/// Requires live API access. With a valid token, `[trace] fetch` lines appear in stderr.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-15]
fn lim_it_ft15_accounts_refresh_live()
{
  if !require_live_api( "ft15" ) { return; }
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

  // Case B: force + active:: → marker written, no error (force is silently ignored on active::).
  {
    let out = run_cs_with_env(
      &[ ".accounts", "force::1", "active::testuser@testmachine", "name::alice@acme.com" ],
      &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
    );
    assert_exit( &out, 0 );

    let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
    let marker = std::fs::read_to_string( store.join( "_active_testmachine_testuser" ) )
      .expect( "FT-21B: marker must be written with force::1 + active::testuser@testmachine" );
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
