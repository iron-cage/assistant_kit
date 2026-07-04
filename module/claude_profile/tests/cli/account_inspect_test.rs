//! Integration tests: AI (Account Inspect) — `.account.inspect` command.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Scope
//!
//! Fixture-based tests (ai01–ai36) run entirely offline — credentials lack
//! `accessToken` so all three endpoint calls return "no token" immediately.
//! No network access required.
//!
//! Live tests (names contain `lim_it`, ai14–ai24) require a real Anthropic
//! OAuth access token. They are excluded from Docker CI by the nextest default
//! filter `!test(lim_it)` in `.config/nextest.toml`. They skip automatically
//! when no token is present or the API is rate-limited.
//!
//! Tests use credentials WITHOUT `accessToken` (written by `write_account`)
//! so all three endpoint calls fail immediately with "no token" — no network access.
//! Token expiry tests use `PAST_MS` + `refresh::0` to stay entirely offline.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | AC | Category | P/N | Live? | IT-N |
//! |----|---------------|----|----------|-----|-------|------|
//! | ai01 | `ai01_credential_file_absent_exits_2` | AC-16 | Error | N | no | |
//! | ai02 | `ai02_account_not_found_exits_2` | AC-12 | Error | N | no | IT-6 |
//! | ai03 | `ai03_empty_name_exits_1` | AC-12 | Error | N | no | |
//! | ai04 | `ai04_no_active_account_exits_2` | AC-12 | Error | N | no | |
//! | ai05 | `ai05_format_invalid_exits_1` | AC-13 | Error | N | no | IT-7 |
//! | ai06 | `ai06_active_marker_used_when_no_name` | AC-12 | Name | P | no | IT-1 |
//! | ai07 | `ai07_prefix_name_resolves` | AC-12 | Name | P | no | IT-2/IT-10 |
//! | ai08 | `ai08_expired_token_shows_expired_status` | AC-11 | Status | P | no | |
//! | ai09 | `ai09_snapshot_all_fields_when_no_token` | AC-07,08,09,11 | Snapshot | P | no | |
//! | ai10 | `ai10_memberships_endpoint_unavailable_message` | AC-07 | Snapshot | P | no | |
//! | ai11 | `ai11_json_all_required_fields` | AC-13 | JSON | P | no | IT-3 |
//! | ai12 | `ai12_json_data_source_snapshot_when_all_fail` | AC-13 | JSON | P | no | |
//! | ai13 | `ai13_trace_emits_lines_to_stderr` | AC-14 | Trace | P | no | IT-4 |
//! | ai22 | `ai22_credential_store_absent_exits_2` | AC-15 | Error | N | no | |
//! | ai23 | `ai23_workspace_fields_show_values_when_non_null` | AC-17 | Org Identity | P | no | |
//! | ai24 | `ai24_ambiguous_prefix_exits_1` | AC-12 | Name | N | no | |
//! | ai25 | `ai25_missing_expires_at_shows_unknown_status` | AC-01 | Status | P | no | |
//! | ai26 | `ai26_name_with_invalid_chars_exits_1` | AC-12 | Name | N | no | |
//! | ai27 | `ai27_unicode_account_name_resolves` | AC-12 | Name | P | no | |
//! | ai28 | `ai28_empty_credentials_file_shows_unknown_status` | AC-18 | Status | P | no | |
//! | ai29 | `ai29_malformed_credentials_json_shows_unknown_status` | AC-19 | Status | P | no | |
//! | ai30 | `ai30_format_case_sensitive_uppercase_exits_1` | AC-13 | Format | N | no | |
//! | ai31 | `ai31_expires_at_zero_shows_expired_status` | AC-01 | Status | P | no | |
//! | ai32 | `ai32_usage_absent_when_offline` | AC-23 | Usage | N | no | |
//! | ai33 | `ai33_name_email_from_snapshot` | AC-20 | Identity | P | no | |
//! | ai34 | `ai34_name_shows_display_name_when_different` | AC-20 | Identity | P | no | |
//! | ai35 | `ai35_no_userinfo_endpoint_reference` | AC-25 | Structural | P | no | |
//! | ai36 | `ai36_name_omitted_when_names_empty` | AC-20 | Identity | N | no | |
//! | ai14 | `lim_it_ai14_identity_fields_from_endpoint_002` | AC-01 | Identity | P | yes | |
//! | ai15 | `lim_it_ai15_memberships_shown_with_count` | AC-02 | Memberships | P | yes | |
//! | ai16 | `lim_it_ai16_selected_marker_multi_membership` | AC-03,04 | Memberships | P | yes | |
//! | ai17 | `lim_it_ai17_org_fields_from_endpoint_005` | AC-05 | Org Identity | P | yes | |
//! | ai18 | `lim_it_ai18_billing_from_selected_membership` | AC-06 | Selection | P | yes | |
//! | ai19 | `lim_it_ai19_valid_token_live_data_source_json` | AC-01,13 | JSON | P | yes | |
//! | ai20 | `lim_it_ai20_refresh_attempted_on_expired_token` | AC-10 | Refresh | P | yes | |
//! | ai21 | `lim_it_ai21_trace_endpoint_lines_on_live_account` | AC-14 | Trace | P | yes | |
//! | ai22 | `lim_it_ai22_name_and_email_from_endpoint_002` | AC-20 | Identity | P | yes | |
//! | ai23 | `lim_it_ai23_capabilities_and_tier_from_membership` | AC-21 | Subscription | P | yes | |
//! | ai24 | `lim_it_ai24_usage_data_from_endpoint_001` | AC-22 | Usage | P | yes | |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_roles_json,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── Test helpers (shared with account_inspect_test_b) ─────────────────────────

/// Resolve the credential store path for a given home directory.
pub( crate ) fn credential_store( home : &std::path::Path ) -> std::path::PathBuf
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
}

/// Write `{credential_store}/{name}.json` with all inspect-relevant fields.
///
/// Combines `billingType`, `taggedId`, `uuid`, `emailAddress`, `fullName`,
/// `displayName`, `capabilities`, and `rateLimitTier` in one file, which none
/// of the standard helpers provide in combination.
#[ allow( clippy::too_many_arguments ) ]
pub( crate ) fn write_inspect_claude_json(
  home            : &std::path::Path,
  name            : &str,
  billing         : &str,
  tagged_id       : &str,
  uuid            : &str,
  has_max         : bool,
  full_name       : &str,
  display_name    : &str,
  rate_limit_tier : &str,
)
{
  let store   = credential_store( home );
  std::fs::create_dir_all( &store ).unwrap();
  let caps    = if has_max { "[\"claude_max\",\"chat\"]" } else { "[\"chat\"]" };
  let content = format!(
    "{{\"oauthAccount\":{{\"billingType\":\"{billing}\",\"taggedId\":\"{tagged_id}\",\"uuid\":\"{uuid}\",\"emailAddress\":\"{name}\",\"fullName\":\"{full_name}\",\"displayName\":\"{display_name}\",\"capabilities\":{caps},\"rateLimitTier\":\"{rate_limit_tier}\"}}}}",
  );
  std::fs::write( store.join( format!( "{name}.json" ) ), content ).unwrap();
}

/// Run `.account.inspect` with the given extra args under the isolated home directory.
pub( crate ) fn run_inspect( home : &str, extra_args : &[ &str ] ) -> std::process::Output
{
  let mut args = vec![ ".account.inspect" ];
  args.extend_from_slice( extra_args );
  run_cs_with_env( &args, &[ ( "HOME", home ) ] )
}

// ── AI: Account Inspect ───────────────────────────────────────────────────────

#[ test ]
/// AC-16: Account name supplied but credentials file is absent in the store → exit 2.
///
/// Source: `tests/docs/feature/031_account_inspect.md`
fn ai01_credential_file_absent_exits_2()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  // Store exists but no credentials file for alice@acme.com.
  let out   = run_inspect( home, &[ "name::alice@acme.com" ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "credential file not found" ),
    "must report missing credential file, got: {err}",
  );
}

/// Spec: [`tests/docs/cli/command/15_account_inspect.md` IT-6]
#[ test ]
/// AC-12: Prefix name resolves to nothing → exit 2 with account not found.
///
/// Source: `tests/docs/feature/031_account_inspect.md`
fn ai02_account_not_found_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  // "nobody" prefix matches no saved account.
  let out  = run_inspect( home, &[ "name::nobody" ] );
  assert_exit( &out, 2 );
  let err  = stderr( &out );
  assert!( err.contains( "not found" ), "must say account not found, got: {err}" );
}

#[ test ]
/// AC-12: Empty `name::` value → exit 1.
fn ai03_empty_name_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  let out  = run_inspect( home, &[ "name::" ] );
  assert_exit( &out, 1 );
}

#[ test ]
/// AC-12: No `name::` supplied and no active account marker → exit 2.
fn ai04_no_active_account_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Account file exists but make_active=false → no _active marker.
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 2 );
  let err  = stderr( &out );
  assert!( err.contains( "no active account" ), "must report no active account, got: {err}" );
}

/// Spec: [`tests/docs/cli/command/15_account_inspect.md` IT-7]
#[ test ]
/// AC-13: Invalid `format::` value → exit 1.
fn ai05_format_invalid_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  let out  = run_inspect( home, &[ "name::alice@acme.com", "format::csv" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "format" ), "must mention format error, got: {err}" );
}

/// Spec: [`tests/docs/cli/command/15_account_inspect.md` IT-1]
#[ test ]
/// AC-12: When `name::` is omitted the per-machine active marker file determines the account.
fn ai06_active_marker_used_when_no_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // make_active=true writes the _active marker for alice@acme.com.
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, true );
  // No name:: — active marker resolves to alice@acme.com.
  let out  = run_inspect( home, &[ "refresh::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "alice@acme.com" ),
    "must show active account name, got:\n{text}",
  );
}

/// Spec: [`tests/docs/cli/command/15_account_inspect.md` IT-2/IT-10]
#[ test ]
/// AC-12: Bare prefix `name::alice` resolves to `alice@acme.com` when unambiguous.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-12`
fn ai07_prefix_name_resolves()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out  = run_inspect( home, &[ "name::alice", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "alice@acme.com" ),
    "prefix must resolve to full name, got:\n{text}",
  );
}

#[ test ]
/// AC-11: Locally-expired token with `refresh::0` — `Status:` line shows "expired".
///
/// No network access: credentials lack `accessToken` so endpoint calls return
/// "no token" immediately. `expiresAt` is `PAST_MS` (year 2001) → expired.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-11`
fn ai08_expired_token_shows_expired_status()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out  = run_inspect( home, &[ "name::alice@acme.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Status:" ), "output must have Status: label, got:\n{text}" );
  assert!( text.contains( "expired" ), "status must say expired, got:\n{text}" );
}

#[ test ]
/// AC-07, AC-08, AC-09, AC-11: Snapshot fields shown with `(snapshot)` suffix when all endpoints fail.
///
/// All endpoints fail with "no token" (no `accessToken` in credentials).
/// Snapshot data is read from `{name}.json`.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-11`
fn ai09_snapshot_all_fields_when_no_token()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  write_inspect_claude_json( dir.path(), "alice@acme.com", "stripe_subscription", "user_01abc", "aaaa-bbbb", true, "Alice", "Alice", "default" );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-uuid-1", "alice's Org", "admin" );

  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );

  assert!( text.contains( "user_01abc (snapshot)" ), "tagged_id must show (snapshot), got:\n{text}" );
  assert!( text.contains( "aaaa-bbbb (snapshot)" ), "uuid must show (snapshot), got:\n{text}" );
  assert!( text.contains( "stripe_subscription (snapshot)" ), "billing_type must show (snapshot), got:\n{text}" );
  assert!( text.contains( "alice's Org (snapshot)" ), "org name must show (snapshot), got:\n{text}" );
  assert!( text.contains( "org-uuid-1 (snapshot)" ), "org uuid must show (snapshot), got:\n{text}" );
  assert!( text.contains( "admin (snapshot)" ), "org role must show (snapshot), got:\n{text}" );
}

#[ test ]
/// AC-07: When endpoint 002 is unavailable the `Memberships:` line shows "endpoint unavailable".
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-07`
fn ai10_memberships_endpoint_unavailable_message()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "endpoint unavailable" ),
    "Memberships: must say endpoint unavailable, got:\n{text}",
  );
}

/// Spec: [`tests/docs/cli/command/15_account_inspect.md` IT-3]
#[ test ]
/// AC-13: `format::json` output includes all required top-level fields.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-13`
fn ai11_json_all_required_fields()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0", "format::json" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );

  for field in &[
    "\"account\"", "\"status\"", "\"expires_in_secs\"",
    "\"tagged_id\"", "\"uuid\"",
    "\"email_address\"", "\"full_name\"", "\"display_name\"",
    "\"memberships\"",
    "\"billing_type\"", "\"has_max\"",
    "\"capabilities\"", "\"rate_limit_tier\"",
    "\"session_5h_pct\"", "\"session_5h_reset_ts\"",
    "\"weekly_7d_pct\"", "\"weekly_7d_reset_ts\"",
    "\"sonnet_7d_pct\"", "\"sonnet_7d_reset_ts\"",
    "\"organization_name\"", "\"organization_uuid\"",
    "\"organization_role\"", "\"workspace_uuid\"",
    "\"workspace_name\"", "\"data_source\"",
  ]
  {
    assert!( text.contains( field ), "JSON must contain {field}, got:\n{text}" );
  }

  assert!(
    text.contains( "\"alice@acme.com\"" ),
    "account field must show alice@acme.com, got:\n{text}",
  );
}

#[ test ]
/// AC-13: `data_source` is `"snapshot"` when all three endpoints fail (no token).
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-13`
fn ai12_json_data_source_snapshot_when_all_fail()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0", "format::json" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "\"snapshot\"" ),
    "data_source must be \"snapshot\" when all endpoints fail, got:\n{text}",
  );
}

/// Spec: [`tests/docs/cli/command/15_account_inspect.md` IT-4]
#[ test ]
/// AC-14: `trace::1` emits timestamped diagnostic lines to stderr (verified on the status trace line).
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-14`
fn ai13_trace_emits_lines_to_stderr()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0", "trace::1" ] );
  assert_exit( &out, 0 );
  let err   = stderr( &out );
  assert!( err.contains( " · " ), "trace::1 must emit trace lines to stderr, got:\n{err}" );
  assert!( err.contains( "status" ), "trace must include status line, got:\n{err}" );
}

#[ test ]
/// AC-15: No credential store directory → exit 2 with `credential file not found`.
///
/// The store directory is never created. With a full email `name::`, the account resolver
/// short-circuits (contains `@`) and proceeds directly to the credential file existence
/// check — producing the same error as AC-16. No distinct "store not found" branch exists.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-15`
fn ai22_credential_store_absent_exits_2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Store directory NOT created — neither store dir nor credential file exists.
  let out  = run_inspect( home, &[ "name::alice@acme.com" ] );
  assert_exit( &out, 2 );
  let err  = stderr( &out );
  assert!(
    err.contains( "credential file not found" ),
    "absent store must produce credential-file-not-found error, got: {err}",
  );
}

#[ test ]
/// AC-17: Non-null `workspace_uuid` and `workspace_name` in the roles snapshot render as
/// actual strings — not as `(none)` — in both text and `format::json` output.
///
/// Uses the snapshot path: credentials have no `accessToken`, so endpoint 005 fails and
/// the command falls back to `{name}.json`. The snapshot contains non-null workspace
/// values, which must appear as actual strings in text output (with `(snapshot)` suffix)
/// and as raw string values in `format::json`.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-19`
fn ai23_workspace_fields_show_values_when_non_null()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  // Write roles snapshot with non-null workspace fields (endpoint 005 fails → snapshot).
  let store = credential_store( dir.path() );
  std::fs::write(
    store.join( "alice@acme.com.json" ),
    r#"{"organization_uuid":"org-uuid-1","organization_name":"alice's Org","organization_role":"admin","workspace_uuid":"ws-uuid-123","workspace_name":"alice's Workspace"}"#,
  ).unwrap();

  // ── Text format: workspace values appear as actual strings, not "(none)" ──
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "ws-uuid-123" ),
    "Workspace UUID must show actual value when non-null, got:\n{text}",
  );
  assert!(
    text.contains( "alice's Workspace" ),
    "Workspace must show actual name when non-null, got:\n{text}",
  );

  // ── JSON format: workspace fields contain raw string values, not null/empty ─
  let json_out  = run_inspect( home, &[ "name::alice@acme.com", "refresh::0", "format::json" ] );
  assert_exit( &json_out, 0 );
  let json_text = stdout( &json_out );
  assert!(
    json_text.contains( "\"ws-uuid-123\"" ),
    "workspace_uuid must be actual string in JSON output, got:\n{json_text}",
  );
  assert!(
    json_text.contains( "\"alice's Workspace\"" ),
    "workspace_name must be actual string in JSON output, got:\n{json_text}",
  );
}

#[ test ]
/// AC-12: Prefix matches two accounts with the same local part but different domains → exit 1.
///
/// `name::alice` is not an exact local-part match (two accounts share local part "alice"),
/// so the resolver falls through to prefix scanning which also finds both — ambiguous.
///
/// Source: `resolve_account_name` ambiguous branch (exit 1 / `ArgumentTypeMismatch`)
fn ai24_ambiguous_prefix_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@corp.com", "pro", "standard", FAR_FUTURE_MS, false );
  let out  = run_inspect( home, &[ "name::alice", "refresh::0" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "ambiguous" ), "must report ambiguous prefix, got: {err}" );
  assert!( err.contains( "alice@acme.com" ), "must list matching accounts, got: {err}" );
  assert!( err.contains( "alice@corp.com" ), "must list matching accounts, got: {err}" );
}

#[ test ]
/// AC-01: Credentials without `expiresAt` field produce `Status: unknown` in text and
/// `"status":"unknown"` in JSON — the command still exits 0.
///
/// Real credential files always have `expiresAt`; this tests graceful handling of malformed
/// or stripped credential files that omit it.
fn ai25_missing_expires_at_shows_unknown_status()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  // Credential file with no `expiresAt` field.
  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( "u@test.com.credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"max","rateLimitTier":"default"}}"#,
  ).unwrap();

  // ── Text format ────────────────────────────────────────────────────────────
  let out   = run_inspect( home, &[ "name::u@test.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "Status:" ),
    "must show Status: label, got:\n{text}",
  );
  assert!(
    text.contains( "unknown" ),
    "status must be 'unknown' when expiresAt is absent, got:\n{text}",
  );

  // ── JSON format ────────────────────────────────────────────────────────────
  let json_out  = run_inspect( home, &[ "name::u@test.com", "refresh::0", "format::json" ] );
  assert_exit( &json_out, 0 );
  let json_text = stdout( &json_out );
  assert!(
    json_text.contains( "\"status\":\"unknown\"" ),
    "JSON status must be \"unknown\" when expiresAt is absent, got:\n{json_text}",
  );
}

#[ test ]
/// AC-12: `name::` value containing `/`, `\`, or `*` is rejected immediately → exit 1.
///
/// These characters are invalid in account name prefixes as they could be used
/// for path traversal or glob expansion against the credential store.
fn ai26_name_with_invalid_chars_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  for bad_name in &[ "foo/bar", "foo*bar", r"foo\bar" ]
  {
    let out = run_inspect( home, &[ &format!( "name::{bad_name}" ) ] );
    assert_exit( &out, 1 );
    let err = stderr( &out );
    assert!(
      err.contains( "invalid characters" ) || err.contains( "invalid" ),
      "name '{bad_name}' must report invalid characters, got: {err}",
    );
  }
}

#[ test ]
/// AC-12: Account name containing Unicode characters (IDN email) resolves by
/// full email lookup.
///
/// Verifies that UTF-8 account names like `alice@münchen.de` are written and
/// read back correctly — the credential filename `alice@münchen.de.credentials.json`
/// must survive the round-trip on Linux UTF-8 filesystems.
fn ai27_unicode_account_name_resolves()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@münchen.de", "pro", "standard", FAR_FUTURE_MS, true );
  let out   = run_inspect( home, &[ "name::alice@münchen.de", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "alice@münchen.de" ),
    "output must contain the Unicode account name, got:\n{text}",
  );
}
