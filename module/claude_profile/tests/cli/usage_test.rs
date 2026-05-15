//! Integration tests: IT (Usage — live quota).
//!
//! Tests the `.usage` command which fetches live rate-limit utilization for all
//! saved accounts via `claude_quota::fetch_rate_limits()` and renders results
//! as a `data_fmt` table with 8 columns: flag, Account, Expires, 5h Left,
//! 5h Reset, 7d Left, 7d Reset, Status.
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
//! | it4  | `it4_lim_it_json_format_valid_array`            | real token + `format::json` → JSON with `_left_pct` fields    | P   | yes   |
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

use crate::helpers::{
  run_cs_with_env, run_cs_without_home,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, live_active_token, write_live_credentials_with_token,
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
  assert!( text.contains( "7d Reset" ), "must contain '7d Reset' column, got:\n{text}" );
  assert!(
    !text.contains( "Session (5h)" ),
    "must NOT contain old 'Session (5h)' column, got:\n{text}",
  );
  assert!(
    !text.contains( "Weekly (7d)" ),
    "must NOT contain old 'Weekly (7d)' column, got:\n{text}",
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
/// em-dash for quota columns, `(missing accessToken)` in Status, and "in …"
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
/// minimum `account` (string), `active` (boolean), and `expires_in_secs`
/// (number); successful entries use `session_5h_left_pct` (not `session_5h_pct`).
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
  assert!( arr[ 0 ][ "active" ].is_boolean(),   "entry must have 'active' boolean, got:\n{text}" );
  assert!( arr[ 0 ][ "expires_in_secs" ].is_number(), "entry must have 'expires_in_secs' number, got:\n{text}" );
  assert!(
    arr[ 0 ][ "session_5h_left_pct" ].is_number(),
    "entry must have 'session_5h_left_pct' number, got:\n{text}",
  );
  let obj = arr[ 0 ].as_object().unwrap();
  assert!(
    !obj.contains_key( "session_5h_pct" ),
    "entry must NOT have old 'session_5h_pct' field, got:\n{text}",
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
