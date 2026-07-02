//! Integration tests: `.credentials.status` — Part B (cred24+).
//!
//! Continuation of `credentials_test.rs`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_claude_json,
  write_claude_json_extended, write_account, write_account_roles_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

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
    err.contains( " · " ),
    "trace::1 must emit trace lines to stderr for .credentials.status, got:\n{err}",
  );
}
