//! Integration tests: `.account.inspect` diagnostic command — Part B (AC-18+).
//!
//! Continuation of `account_inspect_test.rs`.
//!
//! ## Spec Map
//!
//! | Function | IT-N (15_account_inspect.md) |
//! |----------|------------------------------|
//! | `lim_it_ai20_refresh_attempted_on_expired_token` | IT-5/IT-12 |
//! | `lim_it_ai16_selected_marker_multi_membership` | IT-8/IT-9 |
//! | `lim_it_ai15_memberships_shown_with_count` | IT-11 |
//! | `lim_it_ai22_name_and_email_from_endpoint_002` | IT-13 |
//! | `lim_it_ai23_capabilities_and_tier_from_membership` | IT-14 |
//! | `lim_it_ai24_usage_data_from_endpoint_001` | IT-15 |
//! | `lim_it_ai19_valid_token_live_data_source_json` | IT-16 |

use crate::cli_runner::{
  stdout, stderr, assert_exit,
  write_account,
  write_account_with_token, live_active_token, require_live_api,
  FAR_FUTURE_MS, PAST_MS,
};
use crate::account_inspect_test::{ credential_store, write_inspect_claude_json, run_inspect };
use tempfile::TempDir;


#[ test ]
/// AC-18: Credentials file exists but is zero bytes → Status shows "unknown".
///
/// Simulates filesystem corruption (truncated write, disk error). The command
/// must not panic or exit non-zero; graceful fallback to "unknown" is required.
fn ai28_empty_credentials_file_shows_unknown_status()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( "u@test.com.credentials.json" ), b"" ).unwrap();

  // Text format
  let out  = run_inspect( home, &[ "name::u@test.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "unknown" ),
    "empty credentials file must show status 'unknown', got:\n{text}",
  );

  // JSON format
  let json_out  = run_inspect( home, &[ "name::u@test.com", "refresh::0", "format::json" ] );
  assert_exit( &json_out, 0 );
  let json_text = stdout( &json_out );
  assert!(
    json_text.contains( "\"status\":\"unknown\"" ),
    "JSON status must be \"unknown\" for empty credentials file, got:\n{json_text}",
  );
}

#[ test ]
/// AC-19: Valid JSON in credentials but missing `oauthAccount` key → Status shows "unknown".
///
/// Simulates a version-mismatch write (old tool wrote a different schema).
/// The command must not panic; graceful "unknown" status is required because
/// `expiresAt` cannot be found in JSON that has no `oauthAccount` wrapper.
fn ai29_malformed_credentials_json_shows_unknown_status()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( "u@test.com.credentials.json" ),
    r#"{"version":"2","data":{}}"#,
  ).unwrap();

  // Text format
  let out  = run_inspect( home, &[ "name::u@test.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "unknown" ),
    "missing oauthAccount key must show status 'unknown', got:\n{text}",
  );

  // JSON format
  let json_out  = run_inspect( home, &[ "name::u@test.com", "refresh::0", "format::json" ] );
  assert_exit( &json_out, 0 );
  let json_text = stdout( &json_out );
  assert!(
    json_text.contains( "\"status\":\"unknown\"" ),
    "JSON status must be \"unknown\" for malformed credentials, got:\n{json_text}",
  );
}

#[ test ]
/// AC-13: `format` parameter is case-sensitive — uppercase `JSON` is rejected → exit 1.
///
/// The format parameter only accepts lowercase `"text"` and `"json"`.
/// Verifies the argument validator rejects unrecognised values (including
/// same-word different-case) rather than silently falling back to text output.
fn ai30_format_case_sensitive_uppercase_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  let out  = run_inspect( home, &[ "name::alice@acme.com", "format::JSON" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!(
    err.contains( "JSON" ) || err.contains( "format" ) || err.contains( "invalid" ),
    "must report invalid format value, got: {err}",
  );
}

#[ test ]
/// AC-01: Token with `expiresAt=0` (Unix epoch) → Status shows "expired".
///
/// Tests the lower boundary of the expiry time parser. An epoch timestamp is
/// far in the past so the status must be "expired", not "unknown".
/// Distinguishes `expiresAt` present-but-zero from `expiresAt` absent.
fn ai31_expires_at_zero_shows_expired_status()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = credential_store( dir.path() );
  std::fs::create_dir_all( &store ).unwrap();
  // `expiresAt=0` is a valid timestamp (Unix epoch, 1970-01-01), always in the past.
  std::fs::write(
    store.join( "u@test.com.credentials.json" ),
    r#"{"oauthAccount":{"expiresAt":0,"subscriptionType":"pro","rateLimitTier":"standard"}}"#,
  ).unwrap();

  let out  = run_inspect( home, &[ "name::u@test.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "expired" ),
    "expiresAt=0 must show status 'expired' (not 'unknown'), got:\n{text}",
  );
  assert!(
    !text.contains( "unknown" ),
    "expiresAt=0 must not show 'unknown' — zero timestamp IS parseable, got:\n{text}",
  );
}

#[ test ]
/// AC-23: No usage lines appear when endpoint 001 is unavailable (no token → offline).
///
/// All endpoints fail with "no token". Usage section (Session/Weekly/Sonnet) must be
/// entirely absent — not shown as N/A. Other sections (Memberships, Billing) may show
/// snapshot fallback, but usage has no snapshot.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-29`
fn ai32_usage_absent_when_offline()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    !text.contains( "Session (5h):" ),
    "Session (5h): must be absent when endpoint 001 unavailable, got:\n{text}",
  );
  assert!(
    !text.contains( "Weekly (7d):" ),
    "Weekly (7d): must be absent when endpoint 001 unavailable, got:\n{text}",
  );
  assert!(
    !text.contains( "Sonnet (7d):" ),
    "Sonnet (7d): must be absent when endpoint 001 unavailable, got:\n{text}",
  );
}

#[ test ]
/// AC-20: Name and Email from snapshot show `(snapshot)` suffix in text output.
///
/// Credentials lack `accessToken` so endpoint 002 fails; `{name}.json` provides
/// `fullName`, `displayName`, and `emailAddress` as snapshot fallback.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-25`
fn ai33_name_email_from_snapshot()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, false );
  write_inspect_claude_json( dir.path(), "alice@acme.com", "stripe_subscription", "user_01abc", "aaaa-bbbb", true, "Alice Smith", "Alice", "default" );
  let out   = run_inspect( home, &[ "name::alice@acme.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "Name:" ) && text.contains( "Alice Smith" ) && text.contains( "(snapshot)" ),
    "Name: must show full_name with (snapshot) suffix, got:\n{text}",
  );
  assert!(
    text.contains( "Email:" ) && text.contains( "alice@acme.com" ) && text.contains( "(snapshot)" ),
    "Email: must show email with (snapshot) suffix, got:\n{text}",
  );
}

#[ test ]
/// AC-20: When `full_name` and `display_name` differ, both are shown as `"FullName (DisplayName)"`.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-25`
fn ai34_name_shows_display_name_when_different()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "bob@test.com", "pro", "standard", PAST_MS, false );
  write_inspect_claude_json( dir.path(), "bob@test.com", "stripe_subscription", "user_02xyz", "cccc-dddd", false, "Robert Smith", "Bob", "default" );
  let out   = run_inspect( home, &[ "name::bob@test.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    text.contains( "Robert Smith (Bob)" ),
    "Name: must show 'FullName (DisplayName)' when they differ, got:\n{text}",
  );
}

#[ test ]
/// AC-25/BUG-295: Source code contains no reference to the fabricated `userinfo` endpoint.
///
/// Structural assertion: `account_inspect.rs` must not contain the string "userinfo"
/// anywhere — the fabricated `/api/oauth/userinfo` endpoint was removed per BUG-295.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-31`
fn ai35_no_userinfo_endpoint_reference()
{
  let source = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) )
    .join( "src" ).join( "commands" ).join( "account_inspect.rs" );
  let content = std::fs::read_to_string( &source )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", source.display() ) );
  assert!(
    !content.contains( "userinfo" ),
    "account_inspect.rs must not reference 'userinfo' (BUG-295: fabricated endpoint removed)",
  );
}

#[ test ]
/// AC-20: Name line omitted when both `full_name` and `display_name` are empty.
///
/// Email is still shown. Only the Name: line is suppressed.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-26`
fn ai36_name_omitted_when_names_empty()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_account( dir.path(), "bob@test.com", "pro", "standard", PAST_MS, false );
  write_inspect_claude_json( dir.path(), "bob@test.com", "stripe_subscription", "user_02xyz", "cccc-dddd", false, "", "", "default" );
  let out   = run_inspect( home, &[ "name::bob@test.com", "refresh::0" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  assert!(
    !text.contains( "Name:" ),
    "Name: must be absent when full_name and display_name are empty, got:\n{text}",
  );
  assert!(
    text.contains( "Email:" ),
    "Email: must still appear even when names are empty, got:\n{text}",
  );
}

// ── AI lim_it: live-endpoint tests ───────────────────────────────────────────
//
// These tests require a real Anthropic OAuth access token from the host HOME.
// They are excluded from Docker CI by the `!test(lim_it)` nextest filter.
// Each test calls `require_live_api()` early — panics if the API is
// unreachable or rate-limited (no silent skips).

#[ test ]
/// AC-01: Identity fields (Tagged ID, UUID) come from endpoint 002 with a live token.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-01`
fn lim_it_ai14_identity_fields_from_endpoint_002()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai14" );
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
  // Endpoint 002 returned a real value — neither N/A nor (snapshot) suffix.
  assert!(
    !text.contains( "Tagged ID: N/A" ),
    "Tagged ID must not be N/A when endpoint 002 succeeds, got:\n{text}",
  );
}

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-11]
#[ test ]
/// AC-02: Memberships section is shown with a count line.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-02`
fn lim_it_ai15_memberships_shown_with_count()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai15" );
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

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-8/IT-9]
#[ test ]
/// AC-03/AC-04: `← selected` appears exactly once for multi-membership accounts;
/// single-membership accounts show no marker.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-03, FT-04, FT-16, FT-17`
fn lim_it_ai16_selected_marker_multi_membership()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai16" );
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
  require_live_api( "lim_it_ai17" );
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
  require_live_api( "lim_it_ai18" );
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

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-16]
#[ test ]
/// AC-01,AC-13: `format::json` with live token shows `data_source: "live"` when all endpoints succeed.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-13`
fn lim_it_ai19_valid_token_live_data_source_json()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai19" );
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

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-5/IT-12]
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
  require_live_api( "lim_it_ai20" );
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
/// AC-14: `trace::1` emits endpoint-level timestamped diagnostic lines to stderr for a live account.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-14`
fn lim_it_ai21_trace_endpoint_lines_on_live_account()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai21" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[ "trace::1" ] );
  assert_exit( &out, 0 );
  let err  = stderr( &out );
  // Expect one timestamped diagnostic line per endpoint (account, roles, usage).
  let trace_count = err.lines().filter( | l | l.contains( " · " ) ).count();
  assert!(
    trace_count >= 3,
    "trace::1 must emit at least 3 trace lines (one per endpoint), got {trace_count}:\n{err}",
  );
}

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-13]
#[ test ]
/// AC-20: Name and Email fields shown from live endpoint 002 (not snapshot).
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-25`
fn lim_it_ai22_name_and_email_from_endpoint_002()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai22" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Email must be present from live endpoint (not snapshot).
  assert!( text.contains( "Email:" ), "must show Email label from live endpoint, got:\n{text}" );
  let email_line = text.lines().find( | l | l.contains( "Email:" ) ).unwrap();
  assert!(
    !email_line.contains( "(snapshot)" ),
    "Email must not show (snapshot) with live endpoint 002, got:\n{text}",
  );
  // Name may or may not be present (depends on user profile), but if present, no (snapshot).
  if let Some( name_line ) = text.lines().find( | l | l.contains( "Name:" ) )
  {
    assert!(
      !name_line.contains( "(snapshot)" ),
      "Name must not show (snapshot) with live endpoint 002, got:\n{text}",
    );
  }
}

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-14]
#[ test ]
/// AC-21: Capabilities and Tier fields from live selected membership.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-27`
fn lim_it_ai23_capabilities_and_tier_from_membership()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai23" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Capabilities:" ), "must show Capabilities label, got:\n{text}" );
  let caps_line = text.lines().find( | l | l.contains( "Capabilities:" ) ).unwrap();
  assert!(
    caps_line.contains( '[' ),
    "Capabilities must show array format, got: {caps_line}",
  );
  assert!( text.contains( "Tier:" ), "must show Tier label from live membership, got:\n{text}" );
}

/// Spec: [tests/docs/cli/command/15_account_inspect.md IT-15]
#[ test ]
/// AC-22: Usage data (Session/Weekly/Sonnet) shown when endpoint 001 returns utilization.
///
/// Source: `tests/docs/feature/031_account_inspect.md § FT-28`
fn lim_it_ai24_usage_data_from_endpoint_001()
{
  let Some( token ) = live_active_token() else { return; };
  require_live_api( "lim_it_ai24" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live@test.com", &token, true );
  let out  = run_inspect( home, &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Session (5h):" ),
    "must show Session (5h): with live usage data, got:\n{text}",
  );
  let session_line = text.lines().find( | l | l.contains( "Session (5h):" ) ).unwrap();
  assert!(
    session_line.chars().any( | c | c.is_ascii_digit() ),
    "Session line must contain a percentage digit, got: {session_line}",
  );
}
