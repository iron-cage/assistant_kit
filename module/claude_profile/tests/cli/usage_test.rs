//! Integration tests: IT (Usage — live quota).
//!
//! Tests the `.usage` command which fetches live rate-limit utilization for all
//! saved accounts via `claude_quota::fetch_oauth_usage()` and renders results
//! as a `data_fmt` table with 8 columns: flag, Account, Expires, 5h Left,
//! 5h Reset, 7d Left, 7d(Son), 7d Reset.
//!
//! Live tests (names contain `lim_it`) require a real Anthropic OAuth access
//! token. They are excluded from Docker CI by the nextest default filter
//! `!test(lim_it)` in `.config/nextest.toml`. Offline tests (no `lim_it` in
//! the name) run without credentials and cover error paths and edge cases.
//!
//! ## Test Matrix
//!
//! | ID   | Test Function                                   | Condition                                                     | P/N | Live? |
//! |------|-------------------------------------------------|---------------------------------------------------------------|-----|-------|
//! | it1  | `it1_lim_it_quota_heading_and_columns`          | real token → Quota heading + new column names                 | P   | yes   |
//! | it2  | `it2_lim_it_active_account_marked`              | 2 accounts; active one has `✓` in flag column                 | P   | yes   |
//! | it3  | `it3_failed_token_shows_dash_exits_0`           | account without accessToken → `—` + "in …" Expires + exit 0  | P   | no    |
//! | it4  | `it4_lim_it_json_format_valid_array`            | real token + `format::json` → JSON with `_left_pct` fields + `weekly_7d_sonnet_left_pct` | P | yes |
//! | it5  | `it5_empty_store_shows_no_accounts`             | empty credential store → no-accounts message                  | P   | no    |
//! | it6  | `it6_unreadable_store_exits_2`                  | store dir chmod 000 → exit 2                                  | N   | no    |
//! | it7  | `it7_home_unset_exits_2`                        | HOME unset → exit 2                                           | N   | no    |
//! | it8  | `it8_lim_it_accounts_in_alpha_order`            | 3 accounts written out of order → alpha output                | P   | yes   |
//! | it9  | `it9_unreadable_credentials_shows_dash`         | credentials chmod 000 → `—` + exit 0                         | P   | no    |
//! | it10 | `it10_expired_token_shows_expired_in_expires_col` | account with PAST_MS → "EXPIRED" in Expires column           | P   | no    |
//! | it11 | `it11_lim_it_recommendation_marker_shown`       | 2 accounts with real tokens → `→` on non-active account       | P   | yes   |
//! | it12 | `it12_lim_it_footer_shows_valid_count`          | 2 accounts with real tokens → footer "Valid: 2" + "Next:"     | P   | yes   |
//! | it13 | `it13_active_divergence_shows_star`             | live creds=work, _active=alice → `✓` on work, `*` on alice    | P   | no    |
//! | it14 | `it14_creds_unreadable_no_checkmark_star_shown` | no live creds, _active=alice → no `✓`, `*` on alice           | P   | no    |
//! | it15 | `it15_current_equals_active_no_star`            | live creds=alice, _active=alice → `✓` on alice, no `*`        | P   | no    |
//! | it16 | `it16_json_is_current_is_active`                | JSON has `is_current` + `is_active`, no `active` key          | P   | no    |
//! | it17 | `it17_format_table_rejected`                    | `format::table` → exit 1 (not supported by .usage)           | N   | no    |
//! | it18 | `it18_synthetic_row_when_no_saved_match`         | live token unmatched → synthetic (current session) row with ✓ | P   | no    |
//! | it19 | `it19_refresh_disabled_param_accepted`           | `refresh::0` accepted by parser; empty store → no-accounts    | P   | no    |
//! | it20 | `it20_refresh_enabled_offline_no_retry_triggered` | `refresh::1` accepted; missing token → dash, no HTTP call   | P   | no    |
//! | it21 | `it21_lim_it_live_mode`                         | `live::1 interval::30`; real token → "Next update" in output  | P   | yes   |
//! | it22 | `it22_live_jitter_exceeds_interval`             | `live::1 interval::60 jitter::70` → exit 1 before any fetch   | N   | no    |
//! | it23 | `it23_live_interval_below_minimum`              | `live::1 interval::5` → exit 1, stderr contains "30"          | N   | no    |
//! | it24 | `it24_live_incompatible_with_json`              | `live::1 format::json` → exit 1 before any fetch              | N   | no    |
//! | it25 | `it25_synthetic_row_uses_claude_json_email`     | live token unmatched + `.claude.json` has email → row shows email, not "(current session)" | P | no |
//! | it26 | `it26_live_jitter_equals_interval_accepted`     | `live::1 interval::30 jitter::30` (boundary) → exit 2, not 1 (guard allows equal) | P | no |
//! | it27 | `it27_json_error_field_on_failed_account`       | single account without accessToken + format::json → JSON has `"error":` field | P | no |
//! | it28 | `it28_interval_jitter_ignored_when_not_live`    | `interval::5 jitter::70` without `live::1` → exit 0, guards never fire | P | no |
//! | it29 | `it29_live_default_interval_accepted`           | `live::1` alone → default interval=30, no guard error (exit 2 from store) | P | no |
//! | it30 | `it30_live_sigint_exits_0`                      | `live::1`; after 3s send SIGINT → exit 0, stdout has "Monitor stopped."  | P | no |

use crate::helpers::{
  BIN,
  run_cs_with_env, run_cs_without_home, run_cs_bytes_for_secs,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_claude_json, live_active_token,
  write_live_credentials_with_token,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── Live: heading and column names ───────────────────────────────────────────

/// Live: one account with a real token → output contains "Quota" heading and
/// the new quota column names; old combined column names are absent.
#[ test ]
fn it1_lim_it_quota_heading_and_columns()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it1: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Quota" ),    "must contain 'Quota' heading, got:\n{text}" );
  assert!( text.contains( "Expires" ),  "must contain 'Expires' column, got:\n{text}" );
  assert!( text.contains( "5h Left" ),  "must contain '5h Left' column, got:\n{text}" );
  assert!( text.contains( "5h Reset" ), "must contain '5h Reset' column, got:\n{text}" );
  assert!( text.contains( "7d Left" ),  "must contain '7d Left' column, got:\n{text}" );
  assert!( text.contains( "7d(Son)" ),  "must contain '7d(Son)' column, got:\n{text}" );
  assert!( text.contains( "7d Reset" ), "must contain '7d Reset' column, got:\n{text}" );
  assert!(
    !text.contains( "Session (5h)" ),
    "must NOT contain old 'Session (5h)' column, got:\n{text}",
  );
  assert!(
    !text.contains( "Weekly (7d)" ),
    "must NOT contain old 'Weekly (7d)' column, got:\n{text}",
  );
  assert!(
    !text.contains( "Status" ),
    "must NOT contain old 'Status' column, got:\n{text}",
  );
}

// ── Live: active account marked ──────────────────────────────────────────────

/// Live: two accounts; the active one has `✓` in the flag column on its line;
/// no line for the inactive account contains `✓`.
#[ test ]
fn it2_lim_it_active_account_marked()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it2: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a", &token, true  );
  write_account_with_token( dir.path(), "acct-b", &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let active_marked = text.lines().any( |line| line.contains( '✓' ) && line.contains( "acct-a" ) );
  assert!(
    active_marked,
    "a line must contain both ✓ and active account name 'acct-a', got:\n{text}",
  );
  let inactive_marked = text.lines().any( |line| line.contains( '✓' ) && line.contains( "acct-b" ) );
  assert!(
    !inactive_marked,
    "no line must contain both ✓ and inactive account name 'acct-b', got:\n{text}",
  );
}

// ── Offline: missing accessToken shows em-dash ───────────────────────────────

/// Offline: credential file has no `accessToken` field (but has a future
/// `expiresAt`) → `read_token()` returns "missing accessToken" → output shows
/// em-dash for quota columns, `(missing accessToken)` in the last column, and "in …"
/// (not "EXPIRED") in the Expires column because `FAR_FUTURE_MS` is used.
#[ test ]
fn it3_failed_token_shows_dash_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account() uses credential_json() which omits accessToken.
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( '\u{2014}' ),
    "missing accessToken must render as em-dash (\u{2014}), got:\n{text}",
  );
  assert!(
    text.contains( "in " ),
    "Expires must show 'in …' (not 'EXPIRED') for FAR_FUTURE_MS token, got:\n{text}",
  );
}

// ── Live: JSON output structure ───────────────────────────────────────────────

/// Live: `format::json` → output is a JSON array where each entry has at
/// minimum `account` (string), `is_active` (boolean), and `expires_in_secs`
/// (number); successful entries use `session_5h_left_pct` (not `session_5h_pct`)
/// and include `weekly_7d_sonnet_left_pct` (number or null).
#[ test ]
fn it4_lim_it_json_format_valid_array()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it4: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let parsed : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "output must be valid JSON: {e}\ngot:\n{text}" ) );
  assert!( parsed.is_array(), "output must be a JSON array, got:\n{text}" );
  let arr = parsed.as_array().unwrap();
  assert!( !arr.is_empty(), "array must have at least one entry, got:\n{text}" );
  assert!( arr[ 0 ][ "account" ].is_string(),  "entry must have 'account' string, got:\n{text}" );
  assert!( arr[ 0 ][ "is_active" ].is_boolean(), "entry must have 'is_active' boolean, got:\n{text}" );
  assert!( arr[ 0 ][ "expires_in_secs" ].is_number(), "entry must have 'expires_in_secs' number, got:\n{text}" );
  assert!(
    arr[ 0 ][ "session_5h_left_pct" ].is_number() || arr[ 0 ][ "session_5h_left_pct" ].is_null(),
    "entry must have 'session_5h_left_pct' number or null, got:\n{text}",
  );
  let obj = arr[ 0 ].as_object().unwrap();
  assert!(
    obj.contains_key( "weekly_7d_sonnet_left_pct" ),
    "entry must have 'weekly_7d_sonnet_left_pct' field, got:\n{text}",
  );
  assert!(
    !obj.contains_key( "session_5h_pct" ),
    "entry must NOT have old 'session_5h_pct' field, got:\n{text}",
  );
  assert!(
    !obj.contains_key( "status" ),
    "entry must NOT have old 'status' field, got:\n{text}",
  );
}

// ── Offline: empty credential store ─────────────────────────────────────────

/// Offline: credential store directory exists but contains no `.credentials.json`
/// files → `account::list()` returns an empty vec → output shows the no-accounts
/// message, exit 0.
#[ test ]
fn it5_empty_store_shows_no_accounts()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "empty store must show '(no accounts configured)', got:\n{text}",
  );
}

// ── Offline: unreadable credential store → exit 2 ───────────────────────────

/// Offline: credential store directory mode 000 → `account::list()` cannot
/// enumerate it → `fetch_all_quota()` returns `ErrorData` → exit 2.
///
/// Permissions are restored before assertions so `TempDir` cleanup succeeds
/// even when a panic occurs mid-test.
#[ cfg( unix ) ]
#[ test ]
fn it6_unreadable_store_exits_2()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the directory.
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "unreadable store must produce error on stderr" );
}

// ── Offline: HOME unset → exit 2 ────────────────────────────────────────────

/// Offline: HOME removed from process environment → `PersistPaths::new()`
/// cannot resolve the storage root → exit 2 with a non-empty error on stderr.
#[ test ]
// Fix(issue-pro-isolation):
// Root cause: run_cs_without_home() removed $HOME but not $PRO; when $PRO is set in the host
//   environment, the binary resolved the credential store via $PRO and returned a result rather
//   than failing with exit 2 as expected.
// Why Not Caught: Docker container has no $PRO set; the bug only surfaces on the host.
// Fix Applied: added .env_remove("PRO") to run_cs_without_home() in helpers.rs.
// Prevention: any "no home directory" test helper must remove all root-resolution vars, not
//   just $HOME; the full list is $PRO, $HOME, $USERPROFILE.
// Pitfall: $PRO takes priority over $HOME in PersistPaths resolution — removing only $HOME
//   leaves a silent fallback that defeats the test's isolation intent.
fn it7_home_unset_exits_2()
{
  let out = run_cs_without_home( &[ ".usage" ] );
  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "unset HOME must produce error on stderr" );
}

// ── Live: accounts appear in alphabetical order ───────────────────────────────

/// Live: three accounts written out of alphabetical order → output lists them
/// in alphabetical order (delegated to `account::list()` sort).
#[ test ]
fn it8_lim_it_accounts_in_alpha_order()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it8: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Written out of alphabetical order; output must still be alpha-sorted.
  write_account_with_token( dir.path(), "charlie", &token, false );
  write_account_with_token( dir.path(), "alpha",   &token, true  );
  write_account_with_token( dir.path(), "bravo",   &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let pos_alpha   = text.find( "alpha"   ).expect( "output must contain 'alpha'"   );
  let pos_bravo   = text.find( "bravo"   ).expect( "output must contain 'bravo'"   );
  let pos_charlie = text.find( "charlie" ).expect( "output must contain 'charlie'" );
  assert!(
    pos_alpha < pos_bravo && pos_bravo < pos_charlie,
    "accounts must appear alphabetically (alpha < bravo < charlie), got:\n{text}",
  );
}

// ── Offline: unreadable credentials file → em-dash, exit 0 ──────────────────

/// Offline: `.credentials.json` mode 000 → `account::list()` still finds the
/// account (directory is readable), but `read_token()` fails with EACCES →
/// `AccountQuota.result = Err(...)` → output shows em-dash, exit 0.
///
/// Permissions are restored before assertions so `TempDir` cleanup succeeds.
#[ cfg( unix ) ]
#[ test ]
fn it9_unreadable_credentials_shows_dash()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  write_account( dir.path(), "locked", "max", "default", FAR_FUTURE_MS, true );

  let creds = store.join( "locked.credentials.json" );
  std::fs::set_permissions( &creds, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the file.
  std::fs::set_permissions( &creds, std::fs::Permissions::from_mode( 0o644 ) ).unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( '\u{2014}' ),
    "unreadable credentials must render as em-dash (\u{2014}), got:\n{text}",
  );
}

// ── Offline: expired token shows EXPIRED in Expires column ───────────────────

/// Offline: credential file has a past `expiresAt` timestamp (`PAST_MS`) →
/// `compute_expires_cell()` returns `"EXPIRED"` → the Expires column shows
/// "EXPIRED". Exit 0 (non-fatal per-account error).
#[ test ]
fn it10_expired_token_shows_expired_in_expires_col()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "old-acct", "max", "default", PAST_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "EXPIRED" ),
    "expired token must show 'EXPIRED' in Expires column, got:\n{text}",
  );
}

// ── Live: recommendation marker shown ────────────────────────────────────────

/// Live: two accounts — one active, one non-active — both with real tokens.
/// The non-active account is the only candidate and must be marked `→`.
/// The active account must not be marked `→`.
#[ test ]
fn it11_lim_it_recommendation_marker_shown()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it11: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a", &token, true  );
  write_account_with_token( dir.path(), "acct-b", &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let rec_marked = text.lines().any( |line| line.contains( '→' ) && line.contains( "acct-b" ) );
  assert!(
    rec_marked,
    "a line must contain both → and non-active account 'acct-b', got:\n{text}",
  );
  let active_rec = text.lines().any( |line| line.contains( '→' ) && line.contains( "acct-a" ) );
  assert!(
    !active_rec,
    "active account 'acct-a' must not be marked with →, got:\n{text}",
  );
}

// ── Live: footer shows valid count and next recommendation ───────────────────

/// Live: two accounts with real tokens → at least two valid quota results →
/// footer line shows "Valid: 2" and "Next:".
#[ test ]
fn it12_lim_it_footer_shows_valid_count()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it12: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a", &token, true  );
  write_account_with_token( dir.path(), "acct-b", &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Valid: 2" ),
    "footer must contain 'Valid: 2', got:\n{text}",
  );
  assert!(
    text.contains( "Next:" ),
    "footer must contain 'Next:', got:\n{text}",
  );
}

// ── Offline: current-vs-active divergence ─────────────────────────────────────

/// it13 (IT-13): live creds match `work@acme.com`; `_active` = `alice@acme.com`.
///
/// Flag column: `work@acme.com` gets `✓` (`is_current`), `alice@acme.com` gets `*`
/// (`is_active` but not `is_current`). This demonstrates divergence: the active marker
/// and the live session point at different accounts.
#[ test ]
fn it13_active_divergence_shows_star()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is _active, work matches live creds → divergence
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let work_current = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "work@acme.com" ) );
  assert!( work_current, "work@acme.com must have ✓ (is_current), got:\n{text}" );

  let alice_active = text.lines().any( |l| l.contains( '*' ) && l.contains( "alice@acme.com" ) );
  assert!( alice_active, "alice@acme.com must have * (is_active, not current), got:\n{text}" );
}

/// it14 (IT-14): no live credentials file; `_active` = `alice@acme.com`.
///
/// With no live creds, `is_current` is false for all — no `✓` shown.
/// `alice@acme.com` is still `is_active` so `*` is still shown.
#[ test ]
fn it14_creds_unreadable_no_checkmark_star_shown()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  // Deliberately no live credentials file.

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let has_checkmark = text.lines().any( |l| l.contains( '\u{2713}' ) );
  assert!( !has_checkmark, "no ✓ when creds file absent, got:\n{text}" );

  let alice_star = text.lines().any( |l| l.contains( '*' ) && l.contains( "alice@acme.com" ) );
  assert!( alice_star, "alice@acme.com must still have * (is_active), got:\n{text}" );
}

/// it15 (IT-15): live creds match `alice@acme.com`; `_active` = `alice@acme.com`.
///
/// When `is_current` and `is_active` point at the same account, `✓` wins (priority)
/// and `*` does NOT appear on any line (no divergence).
#[ test ]
fn it15_current_equals_active_no_star()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true );
  write_live_credentials_with_token( dir.path(), "tok-alice" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let alice_current = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "alice@acme.com" ) );
  assert!( alice_current, "alice@acme.com must have ✓ when both current and active, got:\n{text}" );

  let has_star = text.lines().any( |l| l.contains( '*' ) );
  assert!( !has_star, "no * when current == active (no divergence), got:\n{text}" );
}

/// it16 (IT-16): `format::json` uses `is_current` + `is_active` field names, not `active`.
///
/// Two accounts; live creds match `work@acme.com`; `_active` = `alice@acme.com`.
/// JSON output must have `"is_current":true` on work and `"is_active":true` on alice.
/// The old `"active"` key must not appear.
#[ test ]
fn it16_json_is_current_is_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let json = stdout( &out );

  assert!( json.contains( "\"is_current\"" ), "JSON must have is_current field, got:\n{json}" );
  assert!( json.contains( "\"is_active\""  ), "JSON must have is_active field, got:\n{json}" );
  assert!( !json.contains( "\"active\""    ), "JSON must not have old 'active' field, got:\n{json}" );

  // work@acme.com: is_current=true, is_active=false
  let work_current = json.contains( "\"work@acme.com\"" ) && json.contains( "\"is_current\":true" );
  assert!( work_current, "work@acme.com must have is_current:true, got:\n{json}" );

  // alice@acme.com: is_active=true
  let alice_active = json.contains( "\"alice@acme.com\"" ) && json.contains( "\"is_active\":true" );
  assert!( alice_active, "alice@acme.com must have is_active:true, got:\n{json}" );
}

// ── it18 ──────────────────────────────────────────────────────────────────────

/// it18 (IT-18): live token does not match any saved account → synthetic row.
///
/// `alice@acme.com` is saved with `tok-alice`; live creds use `tok-unsaved`.
/// No saved account matches the live token → `fetch_all_quota()` prepends a
/// synthetic `(current session)` row with `✓` in the flag column.
///
/// Pitfall (AC-09): this branch is easy to miss when only testing the happy path
/// where the saved account's token equals the live token. The plan explicitly
/// flagged it, and it was still omitted until a systematic AC-by-AC cross-check
/// caught the gap. Always add an explicit unmatched-token test alongside the
/// matched-token test.
#[ test ]
fn it18_synthetic_row_when_no_saved_match()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-unsaved" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "(current session)" ),
    "must show synthetic (current session) row, got:\n{text}",
  );
  let synthetic_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "(current session)" )
  );
  assert!( synthetic_current, "synthetic row must have ✓ flag, got:\n{text}" );

  let alice_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "alice@acme.com" )
  );
  assert!( !alice_current, "alice must NOT have ✓ when unsaved session is live, got:\n{text}" );
}

// ── it17 ──────────────────────────────────────────────────────────────────────

/// it17 (IT-17): `.usage format::table` exits 1 with `ArgumentTypeMismatch`.
///
/// `format::table` is only valid for `.accounts`; all other commands must reject it.
#[ test ]
fn it17_format_table_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "format::table" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it19 ──────────────────────────────────────────────────────────────────────

/// it19: `refresh::0` is accepted by the command parser; empty store exits 0.
///
/// TDD guard — fails before `refresh` is registered (unilang rejects unknown arg).
/// After registration, verifies `refresh::0` has no effect on empty-store output.
#[ test ]
fn it19_refresh_disabled_param_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "refresh::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ),
    "expected no-accounts message with refresh::0, got:\n{text}",
  );
}

// ── it20 ──────────────────────────────────────────────────────────────────────

/// it20: `refresh::1` is accepted by the parser; with a missing-token account the
/// quota call never reaches HTTP, so no 401 is triggered and no retry occurs.
///
/// TDD guard — fails before `refresh` is registered. After registration, verifies
/// `refresh::1` does not crash offline (no-HTTP) error paths.
#[ test ]
fn it20_refresh_enabled_offline_no_retry_triggered()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test-acct", "max", "default", FAR_FUTURE_MS, false );  // no accessToken → dash cells, no HTTP
  let out  = run_cs_with_env( &[ ".usage", "refresh::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "test-acct" ),
    "account name must appear in output, got:\n{text}",
  );
}

// ── it21 ──────────────────────────────────────────────────────────────────────

/// it21 (`lim_it`): `live::1 interval::30 jitter::0` with a real token.
///
/// Runs the live monitor for 10 seconds then kills the process. Within that window
/// the first fetch cycle completes and the countdown footer is written to stdout —
/// the raw byte capture must contain "Next update".
///
/// Requires one saved account with a real token. The process is killed via
/// `Child::kill()` (SIGKILL); SIGINT clean-exit is covered separately (AC-30).
#[ test ]
fn it21_lim_it_live_mode()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it21: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  // Run for 10 s — enough for one stagger (0.2–1.5 s) + network fetch + table render.
  let bytes = run_cs_bytes_for_secs(
    &[ ".usage", "live::1", "interval::30", "jitter::0" ],
    &[ ( "HOME", home ) ],
    10,
  );
  let text = String::from_utf8_lossy( &bytes );
  assert!(
    text.contains( "Next update" ),
    "live mode must emit countdown footer 'Next update ...', got:\n{text}",
  );
}

// ── it22 ──────────────────────────────────────────────────────────────────────

/// it22: `live::1 interval::60 jitter::70` — jitter exceeds interval → exit 1.
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-27: `jitter > interval` is rejected.
#[ test ]
fn it22_live_jitter_exceeds_interval()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::60", "jitter::70" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "jitter > interval must produce error on stderr",
  );
}

// ── it23 ──────────────────────────────────────────────────────────────────────

/// it23: `live::1 interval::5` — interval below minimum → exit 1, message contains "30".
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-26: `interval < 30` is rejected; error message cites the minimum (30).
#[ test ]
fn it23_live_interval_below_minimum()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::5", "jitter::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "30" ),
    "interval-too-small error must mention the minimum (30), got:\n{err}",
  );
}

// ── it24 ──────────────────────────────────────────────────────────────────────

/// it24: `live::1 format::json` — JSON format rejected in live mode → exit 1.
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-25: `live::1 format::json` is incompatible.
#[ test ]
fn it24_live_incompatible_with_json()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live + json must produce error on stderr",
  );
}

// ── it25 ──────────────────────────────────────────────────────────────────────

/// it25: live token unmatched + `~/.claude.json` has `emailAddress` →
/// synthetic row shows the email, NOT the `"(current session)"` fallback.
///
/// Pitfall (AC-09): the synthetic row resolution has TWO paths:
///   • `.claude.json` present with non-empty `emailAddress` → use it (this test)
///   • `.claude.json` absent or empty `emailAddress` → `"(current session)"` (it18)
/// it18 covers the fallback; this test covers the happy path that it18 cannot.
#[ test ]
fn it25_synthetic_row_uses_claude_json_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is saved; live creds use a different token → no saved match → synthetic row.
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-unsaved" );
  // .claude.json supplies the email for the synthetic row.
  write_claude_json( dir.path(), "unsaved@example.com" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "unsaved@example.com" ),
    "synthetic row must use emailAddress from .claude.json, got:\n{text}",
  );
  assert!(
    !text.contains( "(current session)" ),
    "must NOT fall back to '(current session)' when .claude.json has emailAddress, got:\n{text}",
  );
  let synthetic_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "unsaved@example.com" )
  );
  assert!( synthetic_current, "synthetic row must carry ✓ flag, got:\n{text}" );
}

// ── it26 ──────────────────────────────────────────────────────────────────────

/// it26: `live::1 interval::30 jitter::30` — jitter EQUAL to interval is accepted.
///
/// The guard is `jitter > interval` (strict greater-than).  Equal values must not
/// trigger the error.  Exit 2 (store unreadable) proves the guards were passed and
/// `execute_live_mode()` was entered before failing on the unreadable store.
/// Exit 1 would indicate a guard fired, which would be a bug.
#[ cfg( unix ) ]
#[ test ]
fn it26_live_jitter_equals_interval_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::30", "jitter::30" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered, store unreadable (guards passed).
  // Exit 1 = a guard fired — that would be a bug (equal is allowed).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "jitter == interval must not trigger the guard, stderr:\n{err}",
  );
}

// ── it27 ──────────────────────────────────────────────────────────────────────

/// it27: `format::json` for an account whose quota fetch fails → JSON has `"error"` field.
///
/// `write_account()` produces a credential file without `accessToken`, so `read_token()`
/// returns `Err("missing accessToken")` → `AccountQuota.result = Err(...)` →
/// `render_json()` emits `{"account":…,"error":"…"}` instead of quota fields.
///
/// Root cause of gap: it4 and it16 verify JSON structure for successful fetches;
/// neither explicitly asserts the `error` key is present on a failed account.
#[ test ]
fn it27_json_error_field_on_failed_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No accessToken → read_token() fails → result is Err.
  write_account( dir.path(), "no-token@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let json = stdout( &out );

  assert!(
    json.contains( "\"error\":" ),
    "failed account must produce JSON with 'error' key, got:\n{json}",
  );
  assert!(
    !json.contains( "session_5h_left_pct" ),
    "failed account must NOT have quota fields, got:\n{json}",
  );
  // Mandatory base fields must still be present.
  assert!( json.contains( "\"is_current\""     ), "must have is_current, got:\n{json}" );
  assert!( json.contains( "\"is_active\""      ), "must have is_active, got:\n{json}" );
  assert!( json.contains( "\"expires_in_secs\"" ), "must have expires_in_secs, got:\n{json}" );
}

// ── it28 ──────────────────────────────────────────────────────────────────────

/// it28: `interval::5 jitter::70` without `live::1` → no guard fires, exit 0.
///
/// Live-mode guards (interval minimum, jitter ceiling) only activate when
/// `live == 1`.  Specifying invalid interval/jitter in non-live mode must be
/// silently ignored — the params are undefined outside live mode.
#[ test ]
fn it28_interval_jitter_ignored_when_not_live()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  // interval::5 would fail the live-mode guard if live::1 were set.
  // jitter::70 > interval::5 would also fail. Neither should fire here.
  let out = run_cs_with_env(
    &[ ".usage", "interval::5", "jitter::70" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ),
    "non-live mode must ignore interval/jitter and show no-accounts message, got:\n{text}",
  );
}

// ── it30 ──────────────────────────────────────────────────────────────────────

/// it30: `live::1` with a no-token account — SIGINT after 3s → exit 0, "Monitor stopped." in stdout.
///
/// Verifies AC-30: Ctrl-C (SIGINT) causes a clean exit (code 0) without error output.
/// Uses an account with no `accessToken` so the per-account fetch fails instantly (no HTTP call),
/// the binary renders the error table, starts the countdown, then receives SIGINT.
/// `kill -INT` is used as a subprocess to avoid a `libc` dev-dependency.
#[ cfg( unix ) ]
#[ test ]
fn it30_live_sigint_exits_0()
{
  use std::process::Stdio;

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No accessToken → read_token() fails instantly (no HTTP call); render error row; countdown starts.
  write_account( dir.path(), "myaccount", "max", "default", FAR_FUTURE_MS, true );

  let mut child = std::process::Command::new( BIN )
    .args( &[ ".usage", "live::1", "interval::30", "jitter::0" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clp binary" );

  // Wait for the cycle to complete: stagger (200–1500 ms) + instant fail + render + countdown start.
  std::thread::sleep( std::time::Duration::from_secs( 3 ) );

  // Send SIGINT via the system `kill` utility — no libc dep needed.
  let _ = std::process::Command::new( "kill" )
    .args( &[ "-INT", &child.id().to_string() ] )
    .status();

  let out = child.wait_with_output().expect( "failed to wait on clp binary" );
  let text = String::from_utf8_lossy( &out.stdout );

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "SIGINT must cause clean exit 0, got: {:?}\nstdout: {text}\nstderr: {}",
    out.status,
    String::from_utf8_lossy( &out.stderr ),
  );
  assert!(
    text.contains( "Monitor stopped." ),
    "clean SIGINT exit must print 'Monitor stopped.', got:\n{text}",
  );
}

// ── it29 ──────────────────────────────────────────────────────────────────────

/// it29: `live::1` alone — default `interval=30` satisfies the `>= 30` guard.
///
/// When neither `interval::` nor `jitter::` are specified, the binary applies
/// defaults: `interval=30`, `jitter=0`.  `30 < 30` is false so the interval
/// guard does not fire.  Exit 2 (unreadable store) proves `execute_live_mode()`
/// was entered; exit 1 would mean a guard incorrectly fired.
#[ cfg( unix ) ]
#[ test ]
fn it29_live_default_interval_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = guards passed with default interval; exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "interval" ),
    "default interval (30) must not trigger the interval guard, stderr:\n{err}",
  );
}

