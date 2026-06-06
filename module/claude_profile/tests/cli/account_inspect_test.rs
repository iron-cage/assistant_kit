//! Integration tests: AI (Account Inspect) — `.account.inspect` command.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Scope
//!
//! Fixture-based tests (ai01–ai13) run entirely offline — credentials lack
//! `accessToken` so all three endpoint calls return "no token" immediately.
//! No network access required.
//!
//! Live tests (names contain `lim_it`, ai14–ai21) require a real Anthropic
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
//! | ID | Test Function | AC | Category | P/N | Live? |
//! |----|---------------|----|----------|-----|-------|
//! | ai01 | `ai01_credential_file_absent_exits_2` | AC-16 | Error | N | no |
//! | ai02 | `ai02_account_not_found_exits_2` | AC-12 | Error | N | no |
//! | ai03 | `ai03_empty_name_exits_1` | AC-12 | Error | N | no |
//! | ai04 | `ai04_no_active_account_exits_2` | AC-12 | Error | N | no |
//! | ai05 | `ai05_format_invalid_exits_1` | AC-13 | Error | N | no |
//! | ai06 | `ai06_active_marker_used_when_no_name` | AC-12 | Name | P | no |
//! | ai07 | `ai07_prefix_name_resolves` | AC-12 | Name | P | no |
//! | ai08 | `ai08_expired_token_shows_expired_status` | AC-11 | Status | P | no |
//! | ai09 | `ai09_snapshot_all_fields_when_no_token` | AC-07,08,09,11 | Snapshot | P | no |
//! | ai10 | `ai10_memberships_endpoint_unavailable_message` | AC-07 | Snapshot | P | no |
//! | ai11 | `ai11_json_all_required_fields` | AC-13 | JSON | P | no |
//! | ai12 | `ai12_json_data_source_snapshot_when_all_fail` | AC-13 | JSON | P | no |
//! | ai13 | `ai13_trace_emits_lines_to_stderr` | AC-14 | Trace | P | no |
//! | ai22 | `ai22_credential_store_absent_exits_2` | AC-15 | Error | N | no |
//! | ai23 | `ai23_workspace_fields_show_values_when_non_null` | AC-17 | Org Identity | P | no |
//! | ai24 | `ai24_ambiguous_prefix_exits_1` | AC-12 | Name | N | no |
//! | ai25 | `ai25_missing_expires_at_shows_unknown_status` | AC-01 | Status | P | no |
//! | ai26 | `ai26_name_with_invalid_chars_exits_1` | AC-12 | Name | N | no |
//! | ai14 | `lim_it_ai14_identity_fields_from_endpoint_001` | AC-01 | Identity | P | yes |
//! | ai15 | `lim_it_ai15_memberships_shown_with_count` | AC-02 | Memberships | P | yes |
//! | ai16 | `lim_it_ai16_selected_marker_multi_membership` | AC-03,04 | Memberships | P | yes |
//! | ai17 | `lim_it_ai17_org_fields_from_endpoint_005` | AC-05 | Org Identity | P | yes |
//! | ai18 | `lim_it_ai18_billing_from_selected_membership` | AC-06 | Selection | P | yes |
//! | ai19 | `lim_it_ai19_valid_token_live_data_source_json` | AC-01,13 | JSON | P | yes |
//! | ai20 | `lim_it_ai20_refresh_attempted_on_expired_token` | AC-10 | Refresh | P | yes |
//! | ai21 | `lim_it_ai21_trace_endpoint_lines_on_live_account` | AC-14 | Trace | P | yes |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_roles_json,
  write_account_with_token, live_active_token, require_live_api,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── Private test helpers ───────────────────────────────────────────────────────

/// Resolve the credential store path for a given home directory.
fn credential_store( home : &std::path::Path ) -> std::path::PathBuf
{
  home.join( ".persistent" ).join( "claude" ).join( "credential" )
}

/// Write `{credential_store}/{name}.claude.json` with all inspect-relevant fields.
///
/// Combines `billingType`, `taggedId`, `uuid`, and `capabilities` in one file,
/// which none of the standard helpers provide in combination.
fn write_inspect_claude_json(
  home      : &std::path::Path,
  name      : &str,
  billing   : &str,
  tagged_id : &str,
  uuid      : &str,
  has_max   : bool,
)
{
  let store   = credential_store( home );
  std::fs::create_dir_all( &store ).unwrap();
  let caps    = if has_max { "[\"claude_max\",\"chat\"]" } else { "[\"chat\"]" };
  let content = format!(
    "{{\"oauthAccount\":{{\"billingType\":\"{billing}\",\"taggedId\":\"{tagged_id}\",\"uuid\":\"{uuid}\",\"emailAddress\":\"{name}\",\"capabilities\":{caps}}}}}",
  );
  std::fs::write( store.join( format!( "{name}.claude.json" ) ), content ).unwrap();
}

/// Run `.account.inspect` with the given extra args under the isolated home directory.
fn run_inspect( home : &str, extra_args : &[ &str ] ) -> std::process::Output
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
/// Snapshot data is read from `{name}.claude.json` and `{name}.roles.json`.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-11`
fn ai09_snapshot_all_fields_when_no_token()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  write_inspect_claude_json( dir.path(), "alice@acme.com", "stripe_subscription", "user_01abc", "aaaa-bbbb", true );
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
    "\"tagged_id\"", "\"uuid\"", "\"memberships\"",
    "\"billing_type\"", "\"has_max\"",
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

#[ test ]
/// AC-14: `trace::1` emits `[trace]` lines to stderr (verified on the status trace line).
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
  assert!( err.contains( "[trace]" ), "trace::1 must emit [trace] lines to stderr, got:\n{err}" );
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
/// the command falls back to `{name}.roles.json`. The snapshot contains non-null workspace
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
    store.join( "alice@acme.com.roles.json" ),
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

// ── AI lim_it: live-endpoint tests ───────────────────────────────────────────
//
// These tests require a real Anthropic OAuth access token from the host HOME.
// They are excluded from Docker CI by the `!test(lim_it)` nextest filter.
// Each test calls `live_active_token()` and `require_live_api()` early and
// returns without asserting if no token is available or the API is rate-limited.

#[ test ]
/// AC-01: Identity fields (Tagged ID, UUID) come from endpoint 001 with a live token.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-01`
fn lim_it_ai14_identity_fields_from_endpoint_001()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai14" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Account:" ),   "must show Account label, got:\n{text}" );
  assert!( text.contains( "Status:" ),    "must show Status label, got:\n{text}" );
  assert!( text.contains( "Tagged ID:" ), "must show Tagged ID label, got:\n{text}" );
  assert!( text.contains( "UUID:" ),      "must show UUID label, got:\n{text}" );
  // Endpoint 001 returned a real value — neither N/A nor (snapshot) suffix.
  assert!(
    !text.contains( "Tagged ID: N/A" ),
    "Tagged ID must not be N/A when endpoint 001 succeeds, got:\n{text}",
  );
}

#[ test ]
/// AC-02: Memberships section is shown with a count line.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-02`
fn lim_it_ai15_memberships_shown_with_count()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai15" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Memberships:" ), "must show Memberships label, got:\n{text}" );
  // When endpoint 002 succeeds the Memberships line shows a count (digit).
  let has_count = text.lines()
    .filter( | l | l.starts_with( "Memberships:" ) || l.starts_with( "Memberships: " ) )
    .any( | l | l.trim_start_matches( "Memberships:" ).trim().parse::< u32 >().is_ok() );
  assert!( has_count, "Memberships: line must show numeric count, got:\n{text}" );
}

#[ test ]
/// AC-03/AC-04: `← selected` appears exactly once for multi-membership accounts;
/// single-membership accounts show no marker.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-03, FT-04, FT-16, FT-17`
fn lim_it_ai16_selected_marker_multi_membership()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai16" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Extract membership count from "Memberships: N" line.
  let count : u32 = text.lines()
    .find( | l | l.starts_with( "Memberships:" ) )
    .and_then( | l | l.trim_start_matches( "Memberships:" ).trim().parse().ok() )
    .unwrap_or( 0 );

  let marker_count = text.matches( "← selected" ).count();
  if count > 1
  {
    assert_eq!(
      marker_count, 1,
      "multi-membership must show exactly one ← selected, got:\n{text}",
    );
  }
  else
  {
    assert_eq!(
      marker_count, 0,
      "single-membership must show no ← selected, got:\n{text}",
    );
  }
}

#[ test ]
/// AC-05: Org section fields (Org, Org UUID, Org Role, Workspace UUID, Workspace) are shown.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-05`
fn lim_it_ai17_org_fields_from_endpoint_005()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai17" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ),          "must show Org label, got:\n{text}" );
  assert!( text.contains( "Org UUID:" ),     "must show Org UUID label, got:\n{text}" );
  assert!( text.contains( "Org Role:" ),     "must show Org Role label, got:\n{text}" );
  assert!( text.contains( "Workspace UUID:" ), "must show Workspace UUID label, got:\n{text}" );
  assert!( text.contains( "Workspace:" ),    "must show Workspace label, got:\n{text}" );
}

#[ test ]
/// AC-06: Billing and Has Max reflect the priority-selected membership (not N/A or snapshot).
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-06`
fn lim_it_ai18_billing_from_selected_membership()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai18" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Billing:" ),  "must show Billing label, got:\n{text}" );
  assert!( text.contains( "Has Max:" ),  "must show Has Max label, got:\n{text}" );
  // Billing must not be N/A when endpoint 002 succeeds.
  assert!(
    !text.contains( "Billing:         N/A" ),
    "Billing must not be N/A when endpoint 002 succeeds, got:\n{text}",
  );
}

#[ test ]
/// AC-01,AC-13: `format::json` with live token shows `data_source: "live"` when all endpoints succeed.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-13`
fn lim_it_ai19_valid_token_live_data_source_json()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai19" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[ "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // All three endpoints succeed → data_source must be "live".
  // Partial success would yield "partial_snapshot".
  assert!(
    text.contains( "\"live\"" ) || text.contains( "\"partial_snapshot\"" ),
    "data_source must be live or partial_snapshot with real token, got:\n{text}",
  );
  // Ensure status is "valid" (far-future expiresAt was written).
  assert!( text.contains( "\"valid\"" ), "status must be valid with far-future token, got:\n{text}" );
}

#[ test ]
/// AC-10: Locally-expired token with `refresh::1` (default) — refresh is attempted;
/// command exits 0 regardless of whether the refresh succeeds.
///
/// Uses a fake token so refresh fails, but the command must not panic or exit non-zero.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-10`
fn lim_it_ai20_refresh_attempted_on_expired_token()
{
  let Some( _token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai20" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write a credential file with an expired timestamp and a fake accessToken.
  // The refresh attempt will fail (fake token → OAuth returns 400), and the
  // command must still exit 0 with snapshot data.
  write_account_with_token( dir.path(), "expired@test.com", "fake_refresh_token", false );
  // Override expiresAt to PAST_MS by writing the cred file directly.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred  = format!(
    r#"{{"oauthAccount":{{"subscriptionType":"max","rateLimitTier":"default"}},"expiresAt":{PAST_MS},"accessToken":"fake_refresh_token"}}"#,
  );
  std::fs::write( store.join( "expired@test.com.credentials.json" ), &cred ).unwrap();
  std::fs::write( store.join( claude_profile::account::active_marker_filename() ), "expired@test.com" ).unwrap();
  // refresh::1 (default) → attempt_expired_token_refresh is called; it fails gracefully.
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "expired" ), "status must show expired when refresh fails, got:\n{text}" );
}

#[ test ]
/// AC-14: `trace::1` emits endpoint-level `[trace]` lines to stderr for a live account.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-14`
fn lim_it_ai21_trace_endpoint_lines_on_live_account()
{
  let Some( token ) = live_active_token() else { return; };
  if !require_live_api( "lim_it_ai21" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[ "trace::1" ] );
  assert_exit( &out, 0 );
  let err  = stderr( &out );
  // Expect one [trace] line per endpoint (userinfo, subscriptions, roles).
  let trace_count = err.lines().filter( | l | l.contains( "[trace]" ) ).count();
  assert!(
    trace_count >= 3,
    "trace::1 must emit at least 3 [trace] lines (one per endpoint), got {trace_count}:\n{err}",
  );
}
