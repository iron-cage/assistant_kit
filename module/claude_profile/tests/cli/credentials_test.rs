//! Integration tests: cred (Credentials Status).
//!
//! Verifies that `.credentials.status` reads live credentials directly with no
//! dependency on the account store (`_active` marker or credential store).
//!
//! ## Root Cause Context
//!
//! Account-inspection commands require an `_active` marker — even on machines with valid
//! credentials but no account management initialized. These tests confirm that
//! `.credentials.status` has no such dependency.
//!
//! ## Why Tests Use `TempDir` with No Credential Store
//!
//! Each test that verifies account-store independence explicitly omits the credential
//! store from the temp HOME. This is the anti-fake check: the command succeeds without it.
//!
//! ## Fix Applied
//!
//! Introduced `.credentials.status` command that reads `~/.claude/.credentials.json`
//! and `~/.claude.json` directly — no `_active`, no credential store scan.
//!
//! ## Prevention
//!
//! Whenever a new "diagnostics" or "inspect" command is added, it must not silently
//! depend on account management state. Use these tests as the template.
//!
//! ## Pitfall
//!
//! Do NOT call `account::list()` in `credentials_status_routine`.
//! Reading `_active` is permitted for the `account::` line only; it is read
//! opportunistically and the command succeeds with `Account: N/A` when absent.
//! Fresh installations (no credential store) must still succeed.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | cred01 | `cred01_no_credential_store_succeeds` | no credential store → exit 0, Account: N/A | P |
//! | cred02 | `cred02_default_with_claude_json` | default → all 7 default-on fields shown | P |
//! | cred03 | `cred03_format_json` | format::json → JSON object with all 9 fields | P |
//! | cred04 | `cred04_missing_credentials_file_exits_nonzero` | no .credentials.json → non-zero | N |
//! | cred05 | `cred05_no_claude_json_shows_na` | no .claude.json → N/A for email/org/account | P |
//! | cred06 | `cred06_suppress_all_default_on` | all default-on suppressed → Token line only | P |
//! | cred07 | `cred07_opt_in_file_and_saved` | file::1 saved::1 → File and Saved lines | P |
//! | cred08 | `cred08_display_name_opt_in` | display_name::1 → Display: {name} shown | P |
//! | cred09 | `cred09_role_opt_in` | role::1 → Role: {role} shown | P |
//! | cred10 | `cred10_billing_opt_in` | billing::1 → Billing: {type} shown | P |
//! | cred11 | `cred11_model_opt_in` | model::1 → Model: {model} shown | P |
//! | cred12 | `cred12_json_extended_shape` | format::json → includes display_name, role, billing, model keys | P |
//! | cred13 | `cred13_new_params_absent_by_default` | all 4 new opt-in params absent in single default invocation | P |
//! | cred14 | `cred14_save_writes_active_shown_in_credentials_status` | after .account.save → Account: {name} in .credentials.status | P |
//! | cred15 | `cred15_save_infers_name_from_active_marker` | save (no name::) → reads `_active` marker → Account: shows inferred name | P |
//! | cred16 | `cred16_uuid_opt_in_shows_id_line` | `uuid::1` → ID: line with taggedId value | P |
//! | cred17 | `cred17_uuid_out_of_range_rejected` | `uuid::2` → exit 1 | N |
//! | cred18 | `cred18_uuid_string_value_rejected` | `uuid::yes` → exit 1 | N |
//! | cred19 | `cred19_uuid_absent_by_default` | no `uuid::` → ID: absent | P |
//! | cred20 | `cred20_uuid_explicit_zero_no_id_line` | `uuid::0` → ID: absent | P |
//! | cred21 | `cred21_uuid_json_always_includes_tagged_id` | `format::json uuid::0` → tagged_id key present | P |
//! | cred22 | `cred22_uuid_missing_tagged_id_shows_na` | missing taggedId → ID: N/A | P |
//! | cred23 | `cred23_capabilities_opt_in_shows_list` | `capabilities::1` → comma-separated list | P |
//! | cred24 | `cred24_capabilities_out_of_range_rejected` | `capabilities::2` → exit 1 | N |
//! | cred25 | `cred25_capabilities_string_value_rejected` | `capabilities::yes` → exit 1 | N |
//! | cred26 | `cred26_capabilities_absent_by_default` | no `capabilities::` → Capabilities: absent | P |
//! | cred27 | `cred27_capabilities_explicit_zero_absent` | `capabilities::0` → absent | P |
//! | cred28 | `cred28_capabilities_json_always_emits_key` | `format::json capabilities::0` → capabilities key present | P |
//! | cred29 | `cred29_capabilities_empty_array_shows_na` | empty array → Capabilities: N/A | P |
//! | cred30 | `cred30_capabilities_missing_field_shows_na` | missing field → Capabilities: N/A | P |
//! | cred31 | `cred31_org_uuid_shows_org_id_line` | `org_uuid::1` → Org ID: with org UUID | P |
//! | cred32 | `cred32_org_uuid_out_of_range_rejected` | `org_uuid::2` → exit 1 | N |
//! | cred33 | `cred33_org_uuid_string_value_rejected` | `org_uuid::yes` → exit 1 | N |
//! | cred34 | `cred34_org_uuid_absent_by_default` | no `org_uuid::` → Org ID: absent | P |
//! | cred35 | `cred35_org_uuid_explicit_zero_absent` | `org_uuid::0` → Org ID: absent | P |
//! | cred36 | `cred36_org_uuid_json_always_emits_key` | `format::json org_uuid::0` → organization_uuid key present | P |
//! | cred37 | `cred37_org_uuid_missing_roles_json_na` | missing roles.json → Org ID:  N/A | P |
//! | cred38 | `cred38_org_name_shows_org_line` | `org_name::1` → Org: with org name | P |
//! | cred39 | `cred39_org_name_out_of_range_rejected` | `org_name::2` → exit 1 | N |
//! | cred40 | `cred40_org_name_string_value_rejected` | `org_name::yes` → exit 1 | N |
//! | cred41 | `cred41_org_name_absent_by_default` | no `org_name::` → Org: absent | P |
//! | cred42 | `cred42_org_name_explicit_zero_absent` | `org_name::0` → Org: absent | P |
//! | cred43 | `cred43_org_name_json_always_emits_key` | `format::json org_name::0` → organization_name key present | P |
//! | cred44 | `cred44_org_name_missing_roles_json_na` | missing roles.json → Org:     N/A | P |
//! | cred45 | `cred45_ft09_format_json_includes_all_5_org_fields` | format::json includes all 5 org fields | P |
//! | cred46 | `cred46_ft11_null_workspace_fields_render_as_empty_string` | null workspace_uuid/workspace_name → `""` in JSON | P |
//! | cred47 | `cred47_absent_settings_json_model_shows_na` | absent `~/.claude/settings.json` + `model::1` → `Model: N/A` (014 FT-08/AC-08) | P |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_claude_json, write_claude_json_full, write_settings_json,
  write_claude_json_extended, write_account, write_account_roles_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── cred01 ────────────────────────────────────────────────────────────────────

/// cred01: temp HOME with `.credentials.json` only — no credential store.
///
/// Confirms account-store independence: command exits 0 and shows sub + token.
#[ test ]
fn cred01_no_credential_store_succeeds()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Intentionally do NOT create credential store or _active
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "pro" ),
    "output must contain subscription type, got:\n{text}",
  );
  assert!(
    text.to_lowercase().contains( "valid" ) || text.to_lowercase().contains( "expir" ),
    "output must contain token state, got:\n{text}",
  );
  assert!( text.contains( "Account:" ), "Account: line must appear, got:\n{text}" );
  assert!( text.contains( "N/A" ),      "Account: N/A must appear (no _active marker), got:\n{text}" );
}

// ── cred02 ────────────────────────────────────────────────────────────────────

/// cred02: default output with both `.credentials.json` and `.claude.json`.
///
/// Confirms all 6 default-on fields shown: account, sub, tier, token, expires, email.
#[ test ]
fn cred02_default_with_claude_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "user@example.com" );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Account:" ),         "Account: line must appear, got:\n{text}" );
  assert!( text.contains( "pro" ),              "sub must appear, got:\n{text}" );
  assert!( text.contains( "standard" ),         "tier must appear, got:\n{text}" );
  assert!( text.contains( "user@example.com" ), "email must appear, got:\n{text}" );
  assert!(
    text.contains( "Expires" ) || text.contains( "expires" ),
    "Expires: line must appear in default output, got:\n{text}",
  );
}

// ── cred03 ────────────────────────────────────────────────────────────────────

/// cred03: `format::json` — output must be parseable JSON with all 8 required fields.
#[ test ]
fn cred03_format_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "user@example.com" );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert!( text.starts_with( '{' ) && text.ends_with( '}' ), "output must be JSON object, got:\n{text}" );
  assert!( text.contains( "\"subscription\"" ),  "JSON must have subscription field, got:\n{text}" );
  assert!( text.contains( "\"tier\"" ),          "JSON must have tier field, got:\n{text}" );
  assert!( text.contains( "\"token\"" ),         "JSON must have token field, got:\n{text}" );
  assert!( text.contains( "\"expires_in_secs\"" ), "JSON must have expires_in_secs field, got:\n{text}" );
  assert!( text.contains( "\"email\"" ),         "JSON must have email field, got:\n{text}" );
  assert!( text.contains( "\"account\"" ),       "JSON must have account field, got:\n{text}" );
  assert!( text.contains( "\"file\"" ),          "JSON must have file field, got:\n{text}" );
  assert!( text.contains( "\"saved\"" ),         "JSON must have saved field, got:\n{text}" );
}

// ── cred04 ────────────────────────────────────────────────────────────────────

/// cred04: no `.credentials.json` — must exit non-zero with actionable error.
#[ test ]
fn cred04_missing_credentials_file_exits_nonzero()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create .claude dir but NO .credentials.json
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  let code = out.status.code().unwrap_or( -1 );
  assert!( code != 0, "must exit non-zero when .credentials.json absent, got code {code}" );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "credential" ),
    "error must reference the credential file, got:\n{err}",
  );
}

// ── cred05 ────────────────────────────────────────────────────────────────────

/// cred05: default output with no `.claude.json` and no `_active` — email, account show N/A.
#[ test ]
fn cred05_no_claude_json_shows_na()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only write credentials — intentionally omit .claude.json and credential store
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // N/A must appear at least 2 times: Account:, Email:
  let na_count = text.matches( "N/A" ).count();
  assert!(
    na_count >= 2,
    "default output without .claude.json and no _active must show N/A for account, email \
     (found {na_count} N/A), got:\n{text}",
  );
}

// ── cred06 ────────────────────────────────────────────────────────────────────

/// cred06: suppress all default-on fields except token — only Token: line in output.
///
/// Confirms per-field boolean control: setting account/sub/tier/expires/email to 0
/// leaves only the Token: line in stdout.
#[ test ]
fn cred06_suppress_all_default_on()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".credentials.status", "account::0", "sub::0", "tier::0", "expires::0", "email::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.to_lowercase().contains( "valid" ) || text.to_lowercase().contains( "expir" ),
    "token state must be present, got:\n{text}",
  );
  assert!( !text.contains( "Sub:" ),     "Sub: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Tier:" ),    "Tier: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Expires:" ), "Expires: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Email:" ),   "Email: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Account:" ), "Account: must be suppressed, got:\n{text}" );
}

// ── cred07 ────────────────────────────────────────────────────────────────────

/// cred07: opt-in `file::1 saved::1` — File: and Saved: lines appended after default-on fields.
///
/// Confirms that opt-in field-presence flags add the File: and Saved: lines to output.
#[ test ]
fn cred07_opt_in_file_and_saved()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".credentials.status", "file::1", "saved::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "File:" ),  "File: line must appear, got:\n{text}" );
  assert!( text.contains( "Saved:" ), "Saved: line must appear, got:\n{text}" );
  assert!(
    text.contains( ".credentials.json" ),
    "File: line must contain credentials path, got:\n{text}",
  );
}

// ── cred08 ────────────────────────────────────────────────────────────────────

/// cred08: `display_name::1` — Display: line shows displayName from `~/.claude.json`.
///
/// Confirms opt-in field is absent by default and shown when explicitly requested.
#[ test ]
fn cred08_display_name_opt_in()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "user@example.com", "alice", "admin", "stripe_subscription" );

  // Default output must NOT contain Display: line
  let out_default = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  let text_default = stdout( &out_default );
  assert!( !text_default.contains( "Display:" ), "Display: must be absent by default, got:\n{text_default}" );

  // With display_name::1 it must appear
  let out = run_cs_with_env( &[ ".credentials.status", "display_name::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Display:" ), "Display: line must appear with display_name::1, got:\n{text}" );
  assert!( text.contains( "alice" ),    "Display: must contain displayName value, got:\n{text}" );
}

// ── cred09 ────────────────────────────────────────────────────────────────────

/// cred09: `role::1` — Role: line shows organizationRole from `~/.claude.json`.
///
/// Confirms opt-in field is absent by default and shown when explicitly requested.
#[ test ]
fn cred09_role_opt_in()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "user@example.com", "alice", "admin", "stripe_subscription" );

  // Default output must NOT contain Role: line
  let out_default = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  let text_default = stdout( &out_default );
  assert!( !text_default.contains( "Role:" ), "Role: must be absent by default, got:\n{text_default}" );

  // With role::1 it must appear
  let out = run_cs_with_env( &[ ".credentials.status", "role::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Role:" ),  "Role: line must appear with role::1, got:\n{text}" );
  assert!( text.contains( "admin" ), "Role: must contain organizationRole value, got:\n{text}" );
}

// ── cred10 ────────────────────────────────────────────────────────────────────

/// cred10: `billing::1` — Billing: line shows billingType from `~/.claude.json`.
///
/// Confirms opt-in field is absent by default and shown when explicitly requested.
#[ test ]
fn cred10_billing_opt_in()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "user@example.com", "alice", "admin", "stripe_subscription" );

  // Default output must NOT contain Billing: line
  let out_default = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  let text_default = stdout( &out_default );
  assert!( !text_default.contains( "Billing:" ), "Billing: must be absent by default, got:\n{text_default}" );

  // With billing::1 it must appear
  let out = run_cs_with_env( &[ ".credentials.status", "billing::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Billing:" ),              "Billing: line must appear with billing::1, got:\n{text}" );
  assert!( text.contains( "stripe_subscription" ),   "Billing: must contain billingType value, got:\n{text}" );
}

// ── cred11 ────────────────────────────────────────────────────────────────────

/// cred11: `model::1` — Model: line shows model from `~/.claude/settings.json`.
///
/// Confirms opt-in field is absent by default and shown when explicitly requested.
#[ test ]
fn cred11_model_opt_in()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_settings_json( dir.path(), "sonnet" );

  // Default output must NOT contain Model: line
  let out_default = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  let text_default = stdout( &out_default );
  assert!( !text_default.contains( "Model:" ), "Model: must be absent by default, got:\n{text_default}" );

  // With model::1 it must appear
  let out = run_cs_with_env( &[ ".credentials.status", "model::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Model:" ),  "Model: line must appear with model::1, got:\n{text}" );
  assert!( text.contains( "sonnet" ), "Model: must contain model value from settings.json, got:\n{text}" );
}

// ── cred12 ────────────────────────────────────────────────────────────────────

/// cred12: `format::json` — JSON output includes `display_name`, `role`, `billing`, `model` keys.
///
/// Confirms JSON shape is extended with all 4 new fields regardless of field-presence params.
#[ test ]
fn cred12_json_extended_shape()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "user@example.com", "alice", "admin", "stripe_subscription" );
  write_settings_json( dir.path(), "sonnet" );

  let out = run_cs_with_env( &[ ".credentials.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert!( text.starts_with( '{' ) && text.ends_with( '}' ), "output must be JSON object, got:\n{text}" );
  assert!( text.contains( "\"display_name\"" ), "JSON must have display_name key, got:\n{text}" );
  assert!( text.contains( "\"role\"" ),         "JSON must have role key, got:\n{text}" );
  assert!( text.contains( "\"billing\"" ),      "JSON must have billing key, got:\n{text}" );
  assert!( text.contains( "\"model\"" ),        "JSON must have model key, got:\n{text}" );
  assert!( text.contains( "alice" ),            "display_name value must be present, got:\n{text}" );
  assert!( text.contains( "admin" ),            "role value must be present, got:\n{text}" );
  assert!( text.contains( "stripe_subscription" ), "billing value must be present, got:\n{text}" );
  assert!( text.contains( "sonnet" ),           "model value must be present, got:\n{text}" );
}

// ── cred13 ────────────────────────────────────────────────────────────────────

/// cred13: consolidated default-off check — all 4 new opt-in params absent in a single invocation.
///
/// Verifies that `display_name`, `role`, `billing`, and `model` are ALL absent from the
/// default output when all relevant fixture data is present. A single invocation covers
/// all four so the default-off guarantee is tested as a group, not split across tests.
#[ test ]
fn cred13_new_params_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "user@example.com", "alice", "admin", "stripe_subscription" );
  write_settings_json( dir.path(), "sonnet" );

  let out  = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Display:" ), "Display: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Role:" ),    "Role: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Billing:" ), "Billing: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Model:" ),   "Model: must be absent by default, got:\n{text}" );
}

// ── cred14 ────────────────────────────────────────────────────────────────────

/// cred14: after `.account.save`, `.credentials.status` shows `Account: {name}`.
///
/// Regression guard for the bug where `save()` did not write the active marker,
/// leaving `.credentials.status Account:` as `N/A` after every save.
///
/// ## Fix Documentation — issue-active-marker
///
/// - **Root Cause:** `save()` never wrote the active marker; only `switch_account()` did.
/// - **Why Not Caught:** No cross-command test verified `Account:` output immediately after `.account.save`.
/// - **Fix Applied:** Added `std::fs::write( credential_store.join( active_marker_filename() ), name )?;` to `save()`. (Originally `join("_active")`; updated to per-machine `active_marker_filename()` per Feature 025.)
/// - **Prevention:** This test will catch any regression that drops the write.
/// - **Pitfall:** Active marker write must be non-best-effort (`?`) — a silent drop leaves `.credentials.status` showing `Account: N/A`.
#[ test ]
fn cred14_save_writes_active_shown_in_credentials_status()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );

  // Save current credentials as test@example.com — must write _active.
  let save_out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &save_out, 0 );

  // .credentials.status must immediately show Account: test@example.com.
  let status_out = run_cs_with_env(
    &[ ".credentials.status" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &status_out, 0 );
  let text = stdout( &status_out );
  assert!(
    text.contains( "Account: test@example.com" ),
    "Account: must show saved name after .account.save, got:\n{text}",
  );
}

// ── cred15 ────────────────────────────────────────────────────────────────────

/// cred15: `.account.save` with no `name::` reads the `_active` marker for the inferred name.
///
/// Confirms the full inferred-name path: `save()` reads the `_active` marker, writes `_active`
/// (re-affirms the same name), and `.credentials.status` shows that email as `Account:`.
///
/// ## Fix Documentation — issue-inferred-name-save
///
/// - **Root Cause:** No test covered the save-without-name path end-to-end.
/// - **Why Not Caught:** cred14 only tested the explicit-name save path.
/// - **Fix Applied:** Added cred15 to guard the inferred-name → _active → Account: path.
/// - **Prevention:** This test will fail if _active marker read or _active write breaks.
/// - **Pitfall:** The `_active` marker must be written before `.account.save` — it is the source.
#[ test ]
fn cred15_save_infers_name_from_active_marker()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );

  // Write _active marker = "inferred@example.com" (simulates prior .account.use).
  let store = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( claude_profile::account::active_marker_filename() ),
    "inferred@example.com",
  ).unwrap();

  // Save with no name:: — must read _active marker.
  let save_out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &save_out, 0 );

  // .credentials.status must show Account: inferred@example.com.
  let status_out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &status_out, 0 );
  let text = stdout( &status_out );
  assert!(
    text.contains( "Account: inferred@example.com" ),
    "Account: must show inferred name after nameless .account.save, got:\n{text}",
  );
}

// ── cred16–cred22: uuid:: parameter (028_uuid.md EC-1 through EC-7) ──────────

/// cred16 (EC-1): `uuid::1` shows `ID:` line with taggedId value.
#[ test ]
fn cred16_uuid_opt_in_shows_id_line()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "user_abc123", "some-uuid", &[ "claude_code" ] );

  let out = run_cs_with_env( &[ ".credentials.status", "uuid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "ID:" ),       "uuid::1 must emit ID: line, got:\n{text}" );
  assert!( text.contains( "user_abc123" ), "ID: line must show taggedId value, got:\n{text}" );
}

/// cred17 (EC-2): `uuid::2` rejected — out of range for boolean param.
#[ test ]
fn cred17_uuid_out_of_range_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "uuid::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// cred18 (EC-3): `uuid::yes` rejected — type validation.
#[ test ]
fn cred18_uuid_string_value_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "uuid::yes" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// cred19 (EC-4): Default — `ID:` absent when `uuid::` not specified.
#[ test ]
fn cred19_uuid_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "user_abc123", "some-uuid", &[ "claude_code" ] );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "ID:" ), "ID: must be absent by default, got:\n{text}" );
}

/// cred20 (EC-5): `uuid::0` explicit disable — `ID:` absent.
#[ test ]
fn cred20_uuid_explicit_zero_no_id_line()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "user_abc123", "some-uuid", &[ "claude_code" ] );

  let out = run_cs_with_env( &[ ".credentials.status", "uuid::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "ID:" ), "ID: must be absent with uuid::0, got:\n{text}" );
}

/// cred21 (EC-6): `format::json uuid::0` — `tagged_id` always in JSON output.
#[ test ]
fn cred21_uuid_json_always_includes_tagged_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "user_abc123", "some-uuid", &[] );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json", "uuid::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"tagged_id\"" ), "format::json must emit tagged_id key, got:\n{text}" );
  assert!( text.contains( "user_abc123" ),   "tagged_id must contain taggedId value, got:\n{text}" );
}

/// cred22 (EC-7): Missing `taggedId` in `~/.claude.json` → `ID: N/A`.
#[ test ]
fn cred22_uuid_missing_tagged_id_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  // Write claude.json without taggedId
  write_claude_json( dir.path(), "user@example.com" );

  let out = run_cs_with_env( &[ ".credentials.status", "uuid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "ID:" ),    "ID: line must appear with uuid::1, got:\n{text}" );
  assert!( text.contains( "N/A" ),    "missing taggedId must show N/A, got:\n{text}" );
}

// ── cred23–cred30: capabilities:: parameter (029_capabilities.md EC-1 through EC-8) ─

/// cred23 (EC-1): `capabilities::1` shows `Capabilities:` line as comma-separated list.
#[ test ]
fn cred23_capabilities_opt_in_shows_list()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "", "", &[ "claude_code", "pro" ] );

  let out = run_cs_with_env( &[ ".credentials.status", "capabilities::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Capabilities:" ), "capabilities::1 must emit Capabilities: line, got:\n{text}" );
  assert!( text.contains( "claude_code" ),   "Capabilities: must list claude_code, got:\n{text}" );
  assert!( text.contains( "pro" ),           "Capabilities: must list pro, got:\n{text}" );
}

/// cred24 (EC-2): `capabilities::2` rejected — out of range.
#[ test ]
fn cred24_capabilities_out_of_range_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "capabilities::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// cred25 (EC-3): `capabilities::yes` rejected — type validation.
#[ test ]
fn cred25_capabilities_string_value_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "capabilities::yes" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// cred26 (EC-4): Default — `Capabilities:` absent when `capabilities::` not specified.
#[ test ]
fn cred26_capabilities_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "", "", &[ "claude_code" ] );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Capabilities:" ), "Capabilities: must be absent by default, got:\n{text}" );
}

/// cred27 (EC-5): `capabilities::0` explicit disable — `Capabilities:` absent.
#[ test ]
fn cred27_capabilities_explicit_zero_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "", "", &[ "claude_code" ] );

  let out = run_cs_with_env( &[ ".credentials.status", "capabilities::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Capabilities:" ), "Capabilities: must be absent with capabilities::0, got:\n{text}" );
}

/// cred28 (EC-6): `format::json capabilities::0` — `capabilities` always in JSON output.
#[ test ]
fn cred28_capabilities_json_always_emits_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "", "", &[ "claude_code" ] );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json", "capabilities::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"capabilities\"" ), "format::json must emit capabilities key, got:\n{text}" );
  assert!( text.contains( "claude_code" ),      "capabilities array must contain the value, got:\n{text}" );
}

/// cred29 (EC-7): Empty capabilities array → `Capabilities: N/A`.
#[ test ]
fn cred29_capabilities_empty_array_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_extended( dir.path(), "", "", &[] );

  let out = run_cs_with_env( &[ ".credentials.status", "capabilities::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Capabilities:" ), "Capabilities: line must appear, got:\n{text}" );
  assert!( text.contains( "N/A" ),           "empty capabilities must show N/A, got:\n{text}" );
}

/// cred30 (EC-8): Missing capabilities field in `~/.claude.json` → `Capabilities: N/A`.
#[ test ]
fn cred30_capabilities_missing_field_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  // Write claude.json without capabilities field
  write_claude_json( dir.path(), "user@example.com" );

  let out = run_cs_with_env( &[ ".credentials.status", "capabilities::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Capabilities:" ), "Capabilities: line must appear, got:\n{text}" );
  assert!( text.contains( "N/A" ),           "missing capabilities must show N/A, got:\n{text}" );
}

// ── cred31–cred44: org_uuid:: and org_name:: parameters (030/031 EC-1 through EC-7) ─

/// cred31 (EC-1): `org_uuid::1` shows `Org ID:` line with `organization_uuid` from active account's roles.json.
#[ test ]
fn cred31_org_uuid_shows_org_id_line()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env( &[ ".credentials.status", "org_uuid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org ID:" ),     "org_uuid::1 must emit Org ID: line, got:\n{text}" );
  assert!( text.contains( "org-xyz-789" ), "Org ID: must show organization_uuid, got:\n{text}" );
}

/// cred32 (EC-2): `org_uuid::2` rejected — out of range.
#[ test ]
fn cred32_org_uuid_out_of_range_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "org_uuid::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "org_uuid" ), "error must reference org_uuid::, got:\n{err}" );
}

/// cred33 (EC-3): `org_uuid::yes` rejected — type validation.
#[ test ]
fn cred33_org_uuid_string_value_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "org_uuid::yes" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// cred34 (EC-4): Default — `Org ID:` absent when `org_uuid::` not specified.
#[ test ]
fn cred34_org_uuid_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org ID:" ), "Org ID: must be absent by default, got:\n{text}" );
}

/// cred35 (EC-5): `org_uuid::0` explicit disable — `Org ID:` absent.
#[ test ]
fn cred35_org_uuid_explicit_zero_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env( &[ ".credentials.status", "org_uuid::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org ID:" ), "Org ID: must be absent with org_uuid::0, got:\n{text}" );
}

/// cred36 (EC-6): `format::json org_uuid::0` — `organization_uuid` always in JSON output.
#[ test ]
fn cred36_org_uuid_json_always_emits_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz", "Acme Corp", "admin" );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json", "org_uuid::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"organization_uuid\"" ), "format::json must emit organization_uuid key, got:\n{text}" );
  assert!( text.contains( "org-xyz" ),               "organization_uuid must contain the value, got:\n{text}" );
}

/// cred37 (EC-7): Missing roles.json → `Org ID:  N/A` when `org_uuid::1`.
#[ test ]
fn cred37_org_uuid_missing_roles_json_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No roles.json written.

  let out = run_cs_with_env( &[ ".credentials.status", "org_uuid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org ID:" ), "Org ID: line must appear with org_uuid::1, got:\n{text}" );
  assert!( text.contains( "N/A" ),     "missing roles.json must show N/A, got:\n{text}" );
}

/// cred38 (EC-1): `org_name::1` shows `Org:` line with `organization_name` from active account's roles.json.
#[ test ]
fn cred38_org_name_shows_org_line()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env( &[ ".credentials.status", "org_name::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ),      "org_name::1 must emit Org: line, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ), "Org: must show organization_name, got:\n{text}" );
}

/// cred39 (EC-2): `org_name::2` rejected — out of range.
#[ test ]
fn cred39_org_name_out_of_range_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "org_name::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "org_name" ), "error must reference org_name::, got:\n{err}" );
}

/// cred40 (EC-3): `org_name::yes` rejected — type validation.
#[ test ]
fn cred40_org_name_string_value_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "org_name::yes" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// cred41 (EC-4): Default — `Org:` absent when `org_name::` not specified.
#[ test ]
fn cred41_org_name_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org:" ), "Org: must be absent by default, got:\n{text}" );
}

/// cred42 (EC-5): `org_name::0` explicit disable — `Org:` absent.
#[ test ]
fn cred42_org_name_explicit_zero_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env( &[ ".credentials.status", "org_name::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org:" ), "Org: must be absent with org_name::0, got:\n{text}" );
}

/// cred43 (EC-6): `format::json org_name::0` — `organization_name` always in JSON output.
#[ test ]
fn cred43_org_name_json_always_emits_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "org-xyz-789", "Acme Corp", "admin" );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json", "org_name::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"organization_name\"" ), "format::json must emit organization_name key, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ),             "organization_name must contain the value, got:\n{text}" );
}

/// cred44 (EC-7): Missing roles.json → `Org:     N/A` when `org_name::1`.
#[ test ]
fn cred44_org_name_missing_roles_json_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No roles.json written.

  let out = run_cs_with_env( &[ ".credentials.status", "org_name::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ), "Org: line must appear with org_name::1, got:\n{text}" );
  assert!( text.contains( "N/A" ),  "missing roles.json must show N/A, got:\n{text}" );
}

// ── cred45–cred46: FR-22 FT-09 and FT-11 org JSON completeness ────────────────

/// cred45 (FT-09): `format::json` always includes all 5 org fields regardless of opt-in params.
///
/// Verifies `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`,
/// and `workspace_name` are all present in JSON output even without `org_uuid::` or `org_name::` params.
#[ test ]
fn cred45_ft09_format_json_includes_all_5_org_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "user@example.com", "uuid-org-001", "Test Org", "member" );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"organization_uuid\""  ), "format::json must emit organization_uuid key, got:\n{text}" );
  assert!( text.contains( "\"organization_name\""  ), "format::json must emit organization_name key, got:\n{text}" );
  assert!( text.contains( "\"organization_role\""  ), "format::json must emit organization_role key, got:\n{text}" );
  assert!( text.contains( "\"workspace_uuid\""     ), "format::json must emit workspace_uuid key, got:\n{text}" );
  assert!( text.contains( "\"workspace_name\""     ), "format::json must emit workspace_name key, got:\n{text}" );
}

/// cred46 (FT-11): Null `workspace_uuid` and `workspace_name` in `roles.json` render as `""` in JSON.
///
/// The roles.json fixture always writes `workspace_uuid:null` and `workspace_name:null` (personal
/// account / no workspace membership). The CLI normalises null to empty string so the JSON field
/// value is `""`, not `null`.
#[ test ]
fn cred46_ft11_null_workspace_fields_render_as_empty_string()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_account( dir.path(), "user@example.com", "max", "tier4", FAR_FUTURE_MS, true );
  // write_account_roles_json always writes workspace_uuid:null and workspace_name:null.
  write_account_roles_json( dir.path(), "user@example.com", "uuid-org-002", "Personal Org", "owner" );

  let out = run_cs_with_env(
    &[ ".credentials.status", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"workspace_uuid\":\"\"" ), "null workspace_uuid must render as empty string in JSON, got:\n{text}" );
  assert!( text.contains( "\"workspace_name\":\"\"" ), "null workspace_name must render as empty string in JSON, got:\n{text}" );
}

// ── cred47 ────────────────────────────────────────────────────────────────────

/// cred47 (014 FT-08 / AC-08): absent `~/.claude/settings.json` → `Model:` shows `N/A`.
///
/// When `settings.json` does not exist, `model::1` must still succeed (exit 0) and
/// print `Model: N/A` rather than omitting the line or erroring.
#[ test ]
fn cred47_absent_settings_json_model_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write credentials but do NOT write settings.json.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out  = run_cs_with_env( &[ ".credentials.status", "model::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Model:" ),
    "Model: line must appear even when settings.json is absent, got:\n{text}",
  );
  assert!(
    text.contains( "N/A" ),
    "Model: value must be N/A when settings.json is absent, got:\n{text}",
  );
}

// ── it_trace_credentials_status_accepted ──────────────────────────────────────

/// EC-8 (023): `trace::1` accepted by `.credentials.status` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_credentials_status_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status", "trace::1" ], &[ ( "HOME", home ) ] );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .credentials.status, got stderr:\n{err}",
  );
  assert_exit( &out, 0 );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must emit [trace] lines to stderr for .credentials.status, got:\n{err}",
  );
}
