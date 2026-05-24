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
//! | it11 | `it11_lim_it_recommendation_marker_shown`       | 2 accounts + `next::session` → `→` on non-active account      | P   | yes   |
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
//! | it31 | `it31_usage_help_shows_live_params`             | `.usage.help` → exit 0, stdout contains `live`, `interval`, `jitter`     | P | no |
//! | it32 | `it32_lim_it_refresh_per_account`               | real token + `refresh::1` → exit 0, account name visible (AC-19)         | P | yes |
//! | it33 | `it33_mre_refresh_help_excludes_429`            | `.usage.help` refresh says 401/403 not 401/403/429 (issue-refresh-help-429) | P | no |
//! | it34 | `it34_trace_param_writes_to_stderr`             | `trace::1` with no-token account → stderr contains `[trace]` lines         | P | no |
//! | it35 | `it35_empty_store_json_format`                  | empty store + `format::json` → output is `[]`                              | P | no |
//! | it36 | `it36_no_footer_when_no_valid_accounts`         | single failed account → no "Valid:" footer line                            | P | no |
//! | it37 | `it37_mre_bug155_refresh_defaults_to_1`         | `.usage.help` shows "1 = enabled, default" for refresh (BUG-155)           | P | no |
//! | it38 | `it38_mre_bug156_refresh_help_mentions_429_expired` | `.usage.help` refresh mentions 429+locally-expired case (BUG-156)      | P | no |
//! | it39 | `it39_refresh_2_rejected`                           | `refresh::2` out of range → exit 1 (EC-3)                | N | no |
//! | it40 | `it40_refresh_yes_rejected`                         | `refresh::yes` type error → exit 1 (EC-4)                | N | no |
//! | it41 | `it41_live_0_single_fetch_exits_0`                  | `live::0` explicit → exit 0, no countdown footer (EC-2)     | P | no |
//! | it42 | `it42_live_2_rejected`                              | `live::2` out of range → exit 1 (EC-4)                      | N | no |
//! | it43 | `it43_live_yes_rejected`                            | `live::yes` type error → exit 1 (EC-5)                      | N | no |
//! | it44 | `it44_interval_abc_rejected`                        | `interval::abc` type error → exit 1 (EC-6)              | N | no |
//! | it45 | `it45_interval_60_live_accepted`                    | `live::1 interval::60` guard passes (exit 2, not 1) (EC-3) | P | no |
//! | it46 | `it46_jitter_0_explicit_live_accepted`              | `live::1 jitter::0` explicit zero guard passes (EC-1)     | P | no |
//! | it47 | `it47_jitter_10_live_accepted`                      | `live::1 interval::30 jitter::10` guard passes (EC-2)     | P | no |
//! | it48 | `it48_jitter_abc_rejected`                          | `jitter::abc` type error → exit 1 (EC-7)                  | N | no |
//! | it49 | `it49_trace_0_no_trace_on_stderr`                   | `trace::0` explicit → no [trace] on stderr (EC-2)          | P | no |
//! | it50 | `it50_trace_2_rejected`                             | `trace::2` out of range → exit 1 (EC-3)                    | N | no |
//! | it51 | `it51_trace_yes_rejected`                           | `trace::yes` type error → exit 1 (EC-4)                    | N | no |
//! | it52 | `it52_trace_default_off`                            | no `trace::` → no [trace] lines on stderr (EC-5)           | P | no |
//! | it043 | `it043_sort_name_accepted`                         | `sort::name` + empty store → exit 0 (IT-44/AC-01)          | P | no |
//! | it044 | `it044_sort_endurance_accepted`                     | `sort::endurance` + empty store → exit 0 (IT-45/AC-02)     | P | no |
//! | it045 | `it045_sort_drain_accepted`                         | `sort::drain` + empty store → exit 0 (IT-46/AC-03)         | P | no |
//! | it046 | `it046_sort_reset_accepted`                         | `sort::reset` + empty store → exit 0 (IT-47/AC-04)         | P | no |
//! | it047 | `it047_sort_invalid_value_exit_1`                   | `sort::bogus` → exit 1, stderr names valid values (IT-48/AC-09) | N | no |
//! | it048 | `it048_prefer_invalid_value_exit_1`                 | `prefer::bogus` → exit 1, stderr names valid values (IT-49/AC-10) | N | no |
//! | it049 | `it049_usage_help_shows_sort_params`                | `.usage.help` lists `sort`, `desc`, `prefer` (IT-50)       | P | no |
//! | it050 | `it050_desc_0_accepted`                             | `desc::0` + empty store → exit 0 (026_desc EC-1)           | P | no |
//! | it051 | `it051_desc_1_accepted`                             | `desc::1` + empty store → exit 0 (026_desc EC-2)           | P | no |
//! | it052_desc_2_rejected | `it052_desc_2_rejected`            | `desc::2` out of range → exit 1 (026_desc EC-3)            | N | no |
//! | it053 | `it053_sort_name_desc_0_identical_to_sort_name`     | `sort::name desc::0` same order as `sort::name` (CC-1)     | P | no |
//! | it054 | `it054_sort_name_desc_1_reverses_order`             | `sort::name desc::1` shows z before a (CC-2)               | P | no |
//! | it055 | `it055_prefer_any_accepted`                         | `prefer::any` + empty store → exit 0 (027_prefer EC-1)     | P | no |
//! | it056 | `it056_prefer_opus_accepted`                        | `prefer::opus` + empty store → exit 0 (027_prefer EC-2)    | P | no |
//! | it057 | `it057_prefer_sonnet_accepted`                      | `prefer::sonnet` + empty store → exit 0 (027_prefer EC-3)  | P | no |
//! | it058 | `it058_sort_json_unaffected_by_sort_strategy`       | JSON alphabetical regardless of `sort::` strategy (025_sort CC-1) | P | no |
//! | it059 | `it059_sort_uppercase_rejected`                     | `sort::Name` (uppercase) → exit 1 (case-sensitive)         | N | no |
//! | it060 | `it060_prefer_uppercase_rejected`                   | `prefer::Opus` (uppercase) → exit 1 (case-sensitive)       | N | no |
//! | it063 | `it063_next_all_accepted`                           | `next::all` accepted with empty store → exit 0 (AC-01)     | P | no |
//! | it064 | `it064_next_session_accepted`                       | `next::session` accepted with empty store → exit 0 (AC-03) | P | no |
//! | it065 | `it065_next_endurance_accepted`                     | `next::endurance` accepted with empty store → exit 0       | P | no |
//! | it066 | `it066_next_drain_accepted`                         | `next::drain` accepted with empty store → exit 0           | P | no |
//! | it067 | `it067_next_reset_accepted`                         | `next::reset` accepted with empty store → exit 0           | P | no |
//! | it068 | `it068_next_invalid_value_exit_1`                   | `next::bogus` → exit 1, stderr names valid values (AC-07)  | N | no |
//! | it069 | `it069_next_all_suppresses_arrow_in_table`          | default next::all: 2 accounts → no `→` in any table row   | P | no |
//! | it070 | `it070_cols_sub_accepted`                           | `cols::+sub` accepted with empty store → exit 0            | P | no |
//! | it071 | `it071_cols_sub_shows_sub_column`                   | `cols::+sub` with account → output contains "Sub" header   | P | no |
//! | it072 | `it072_cols_unknown_id_exit_1`                      | `cols::+bogus_col` → exit 1, stderr names valid IDs        | N | no |
//! | it073 | `it073_usage_help_shows_next_cols_params`           | `.usage.help` lists `next` and `cols` params               | P | no |

use crate::helpers::{
  BIN,
  run_cs, run_cs_with_env, run_cs_without_home, run_cs_bytes_for_secs,
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

  // Use next::session to place → in the table body (next::all, the default, suppresses it).
  let out  = run_cs_with_env( &[ ".usage", "next::session" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let rec_marked = text.lines().any( |line| line.contains( '→' ) && line.contains( "acct-b" ) );
  assert!(
    rec_marked,
    "next::session: a line must contain both → and non-active account 'acct-b', got:\n{text}",
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
/// After registration, verifies `refresh::0` (explicit disable) has no effect on
/// empty-store output. Note: `refresh::1` is the default; this test explicitly
/// exercises the opt-out path.
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

  let child = std::process::Command::new( BIN )
    .args( [ ".usage", "live::1", "interval::30", "jitter::0" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clp binary" );

  // Wait for the cycle to complete: stagger (200–1500 ms) + instant fail + render + countdown start.
  std::thread::sleep( core::time::Duration::from_secs( 3 ) );

  // Send SIGINT via the system `kill` utility — no libc dep needed.
  let _ = std::process::Command::new( "kill" )
    .args( [ "-INT", &child.id().to_string() ] )
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

// ── it31 ──────────────────────────────────────────────────────────────────────

/// it31: `.usage.help` lists `live`, `interval`, and `jitter` params.
///
/// Verifies AC-32: all three live-monitor params must appear in the per-command
/// help output so users can discover them without reading source code.
/// The params are registered via `register_commands()` in `src/lib.rs`; this
/// test confirms the registration produces visible output in `.usage.help`.
#[ test ]
fn it31_usage_help_shows_live_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  for param in &[ "live", "interval", "jitter" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}` (AC-32), got:\n{text}",
    );
  }
}

// ── it33 ──────────────────────────────────────────────────────────────────────

/// it33: `.usage.help` refresh description mentions 401/403 but NOT 429.
///
/// # Root Cause
/// Task 150 removed 429 from the `apply_refresh` retry guard, but the parameter
/// description in `lib.rs register_commands()` was not updated — it still said
/// "401/403/429". Users reading `--help` would believe 429 triggers a refresh.
///
/// # Why Not Caught
/// Existing help test (it31) only checked for `live`, `interval`, `jitter` params.
/// No test verified the refresh description text excluded 429.
///
/// # Fix Applied
/// Changed description from "401/403/429" to "401/403" in `lib.rs:167`.
///
/// # Prevention
/// This test asserts `help` output contains "401/403" but NOT "401/403/429".
///
/// # Pitfall
/// The assertion relies on the exact substring "401/403/429" — a reformulated
/// description that mentions 429 in different phrasing would not be caught.
// test_kind: bug_reproducer(issue-refresh-help-429)
#[ test ]
fn it33_mre_refresh_help_excludes_429()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "401/403" ),
    "refresh description must mention 401/403, got:\n{text}",
  );
  assert!(
    !text.contains( "401/403/429" ),
    "refresh description must NOT mention 429 (task 150 removed it), got:\n{text}",
  );
}

// ── it32 ──────────────────────────────────────────────────────────────────────

/// it32 (`lim_it`): `refresh::1` with a real saved account — exercises the
/// per-account refresh loop (AC-19) and verifies no panic + exit 0.
///
/// The per-account loop reads `{credential_store}/{name}.credentials.json`
/// (not the live session file). When the account's quota fetch succeeds on the
/// first pass, `should_retry` is false and the loop is a no-op — the test
/// proves no regression in the happy path. When credentials are stale/expired,
/// the loop runs `run_isolated` and updates `aq.result`.
///
/// Requires one saved account with a live token reachable via `live_active_token()`.
#[ test ]
fn it32_lim_it_refresh_per_account()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it32: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "test-acct", &token, true );
  write_live_credentials_with_token( dir.path(), &token );

  let out = run_cs_with_env( &[ ".usage", "refresh::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "test-acct" ),
    "account must appear in output with refresh::1 (AC-19), got:\n{text}",
  );
}

// ── it34 ──────────────────────────────────────────────────────────────────────

/// it34: `trace::1` with a no-token account → stderr contains `[trace]` lines.
///
/// `trace::1` causes `fetch_all_quota` to emit `[trace]` lines per account to
/// stderr — one before reading credentials and one after. With a credential file
/// that has no `accessToken`, `read_token()` returns Err → trace emits
/// "cannot read token: missing accessToken". This test confirms the `trace`
/// parameter is accepted, wired through to `fetch_all_quota`, and produces
/// observable stderr output without affecting exit code or stdout.
#[ test ]
fn it34_trace_param_writes_to_stderr()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "trace-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must write [trace] lines to stderr, got:\n{err}",
  );
  assert!(
    err.contains( "trace-acct" ),
    "trace::1 must mention the account name, got:\n{err}",
  );
}

// ── it35 ──────────────────────────────────────────────────────────────────────

/// it35: empty credential store + `format::json` → output is `[]`.
///
/// `render_json(&[])` returns `"[]\n"` via the short-circuit branch. This verifies
/// that `format::json` and the empty-store path are compatible — no crash, no
/// "no accounts configured" text (that message is text-format-only).
#[ test ]
fn it35_empty_store_json_format()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "[]",
    "empty store with format::json must output '[]', got:\n{text}",
  );
}

// ── it37 ──────────────────────────────────────────────────────────────────────

/// it37: `.usage.help` shows `refresh::` default as `1` (enabled), not `0`.
///
/// ## Root Cause
/// `usage_routine()` in `src/usage.rs` matched `refresh` with fallback `_ => 0`,
/// making `refresh` default to disabled. Every `clp .usage` call without `refresh::`
/// skipped `apply_refresh()`, showing stale `(auth expired (401))` rows instead
/// of refreshing the token and retrying. Both the runtime default and the help-text
/// description were wrong — `lib.rs` said "(0 = disabled; 1 = enabled)" with no
/// indication which is default; `unilang.commands.yaml` carried `default: "0"`.
///
/// ## Why Not Caught
/// Existing tests (it19/it20) checked that both `refresh::0` and `refresh::1` are
/// accepted. Neither verified that the DEFAULT (no arg) was 1. The help text test
/// (it33) only checked the 429 exclusion, not the default value annotation.
///
/// ## Fix Applied
/// `usage_routine()` fallback changed from `_ => 0` to `_ => 1`. Description in
/// `lib.rs:167` updated to "(1 = enabled, default; 0 = disabled)". `unilang.commands.yaml`
/// default updated to `"1"`. All feature/CLI docs and IT specs updated to reflect
/// the new default.
///
/// ## Prevention
/// This test asserts `.usage.help` output contains `"1 = enabled, default"` — the
/// exact phrase added to the description — and does NOT contain `"0 = disabled, default"`.
///
/// ## Pitfall
/// Any future edit to the description string in `lib.rs` that removes `"1 = enabled, default"`
/// (e.g., reformulation keeping 429 but changing default wording) would break this test.
// test_kind: bug_reproducer(issue-155)
#[ test ]
fn it37_mre_bug155_refresh_defaults_to_1()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "1 = enabled, default" ),
    "refresh help must indicate 1 is the default (BUG-155), got:\n{text}",
  );
  // The `live` description legitimately contains "0 = off, default"; only check that
  // the refresh-specific "(0 = disabled, default)" phrasing is absent.
  assert!(
    !text.contains( "0 = disabled, default" ),
    "refresh help must NOT say '0 = disabled, default' (BUG-155), got:\n{text}",
  );
}

// ── it38 ──────────────────────────────────────────────────────────────────────

/// it38: `.usage.help` refresh description mentions 429 with locally-expired token.
///
/// ## Root Cause
/// `apply_refresh()` unconditionally excluded 429 from its retry guard. Accounts
/// returning 429 with a locally-expired `expiresAt` (stale per-account credentials
/// file) were never refreshed — the `Expires` column showed `EXPIRED` and the
/// 429 was displayed with no refresh attempt made. The guard now conditionally
/// includes 429 when `expires_at_ms / 1000 ≤ now_secs`.
///
/// ## Why Not Caught
/// Existing tests (it33, it19/it20) checked 401/403 refresh and the absence of
/// "401/403/429" as a combined string. None verified the 429+locally-expired case.
///
/// ## Fix Applied
/// `should_refresh()` extracted as a private helper; extended to return `true` for
/// 429 when `expires_at_ms / 1000 <= now_secs`. Description in `lib.rs:167` and
/// `unilang.commands.yaml` updated to document the conditional 429 case.
/// `apply_refresh()` propagates retry errors to `aq.result` (was: silent discard).
/// `aq.expires_at_ms` updated from credentials file after successful write (was: stale).
///
/// ## Prevention
/// This test asserts `.usage.help` contains "429", confirming the description was
/// updated — the code and docs are consistent with the new 429+expired behavior.
///
/// ## Pitfall
/// it33 still guards against the old "401/403/429" combined string. This test
/// adds the positive check: "429" appears separately for the conditional case.
// test_kind: bug_reproducer(issue-156)
#[ test ]
fn it38_mre_bug156_refresh_help_mentions_429_expired()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "429" ),
    "refresh help must mention 429 case (BUG-156), got:\n{text}",
  );
  // Ensure 429 appears in the conditional context, not as the old "401/403/429" pattern.
  assert!(
    !text.contains( "401/403/429" ),
    "refresh help must NOT say '401/403/429' (old incorrect format), got:\n{text}",
  );
}

// ── it36 ──────────────────────────────────────────────────────────────────────

/// it36: single no-token account → no "Valid:" footer (`valid_count` = 0 < 2).
///
/// The footer line "Valid: X / Y   →  Next: ..." is only emitted when
/// `valid_count >= 2` AND a recommendation exists. With one account whose quota
/// fetch fails (no `accessToken`), `valid_count = 0` → the footer is suppressed.
/// This guards against a regression where footer threshold checking is removed.
#[ test ]
fn it36_no_footer_when_no_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-quota@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Valid:" ),
    "single failed account must NOT show 'Valid:' footer line, got:\n{text}",
  );
}

// ── it39 ──────────────────────────────────────────────────────────────────────

/// it39 (EC-3): `refresh::2` is out of range for the boolean
/// parameter (only 0 and 1 are valid) → exit 1 with error on stderr.
///
/// Source: `tests/docs/cli/param/19_refresh.md § EC-3`.
#[ test ]
fn it39_refresh_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "refresh::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "refresh::2 must produce error on stderr",
  );
}

// ── it40 ──────────────────────────────────────────────────────────────────────

/// it40 (EC-4): `refresh::yes` is a type mismatch — the param
/// is a boolean integer, not a string → exit 1.
///
/// Source: `tests/docs/cli/param/19_refresh.md § EC-4`.
#[ test ]
fn it40_refresh_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "refresh::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "refresh::yes must produce error on stderr",
  );
}

// ── it41 ──────────────────────────────────────────────────────────────────────

/// it41 (EC-2): `live::0` explicit — single fetch exits 0; no
/// countdown footer emitted.
///
/// `live::0` disables live-monitor mode.  The command performs one fetch cycle
/// (here: empty store → "no accounts") and exits immediately without entering
/// the continuous loop.  The countdown footer ("Next update …") must not appear.
/// Source: `tests/docs/cli/param/20_live.md § EC-2`.
#[ test ]
fn it41_live_0_single_fetch_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "live::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Next update" ),
    "live::0 must not emit countdown footer, got:\n{text}",
  );
}

// ── it42 ──────────────────────────────────────────────────────────────────────

/// it42 (EC-4): `live::2` is out of range for the boolean parameter
/// (only 0 and 1 are valid) → exit 1.
///
/// Source: `tests/docs/cli/param/20_live.md § EC-4`.
#[ test ]
fn it42_live_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live::2 must produce error on stderr",
  );
}

// ── it43 ──────────────────────────────────────────────────────────────────────

/// it43 (EC-5): `live::yes` is a type mismatch → exit 1.
///
/// Source: `tests/docs/cli/param/20_live.md § EC-5`.
#[ test ]
fn it43_live_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live::yes must produce error on stderr",
  );
}

// ── it44 ──────────────────────────────────────────────────────────────────────

/// it44 (EC-6): `interval::abc` is a type error — the param is
/// `u64`, not a string → exit 1 before any credential or live-mode processing.
///
/// Type validation fires at argument parse time; the `live::` mode flag does not
/// affect it (contrast EC-5 where a valid-type but out-of-range value is accepted
/// in non-live mode).
/// Source: `tests/docs/cli/param/21_interval.md § EC-6`.
#[ test ]
fn it44_interval_abc_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "interval::abc" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "interval::abc must produce error on stderr",
  );
}

// ── it45 ──────────────────────────────────────────────────────────────────────

/// it45 (EC-3): `live::1 interval::60` — non-default value
/// accepted; the interval guard (≥ 30) passes for 60 → live mode is entered.
///
/// A chmod-000 credential store forces exit 2 after the guards pass, proving
/// live mode was entered.  Exit 1 would indicate a guard incorrectly fired.
/// Source: `tests/docs/cli/param/21_interval.md § EC-3`.
#[ cfg( unix ) ]
#[ test ]
fn it45_interval_60_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::60" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered (interval guard passed); exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "interval" ),
    "interval::60 must not trigger the interval guard, stderr:\n{err}",
  );
}

// ── it46 ──────────────────────────────────────────────────────────────────────

/// it46 (EC-1): `live::1 jitter::0` — explicit zero jitter accepted;
/// the jitter guard (jitter ≤ interval) passes for 0 ≤ 30 → live mode is entered.
///
/// Uses a chmod-000 store for offline verification.  Distinct from `it29` which
/// uses the implicit default (no `jitter::` param) — this test exercises the
/// explicit `jitter::0` path.
/// Source: `tests/docs/cli/param/22_jitter.md § EC-1`.
#[ cfg( unix ) ]
#[ test ]
fn it46_jitter_0_explicit_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "jitter::0" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered; exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "explicit jitter::0 must not trigger the jitter guard, stderr:\n{err}",
  );
}

// ── it47 ──────────────────────────────────────────────────────────────────────

/// it47 (EC-2): `live::1 interval::30 jitter::10` — jitter less
/// than interval is accepted; the guard (jitter ≤ interval) passes → live mode
/// is entered.
///
/// Uses a chmod-000 store for offline verification.
/// Source: `tests/docs/cli/param/22_jitter.md § EC-2`.
#[ cfg( unix ) ]
#[ test ]
fn it47_jitter_10_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::30", "jitter::10" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered (jitter::10 ≤ interval::30); exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "jitter::10 with interval::30 must not trigger the jitter guard, stderr:\n{err}",
  );
}

// ── it48 ──────────────────────────────────────────────────────────────────────

/// it48 (EC-7): `jitter::abc` is a type error — the param is `u64`,
/// not a string → exit 1.
///
/// Source: `tests/docs/cli/param/22_jitter.md § EC-7`.
#[ test ]
fn it48_jitter_abc_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "jitter::abc" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "jitter::abc must produce error on stderr",
  );
}

// ── it49 ──────────────────────────────────────────────────────────────────────

/// it49 (EC-2): `trace::0` explicit disable — no `[trace]` lines
/// appear on stderr; exit 0.
///
/// Uses a no-token account so the fetch path is exercised (increasing the chance
/// of accidental trace leakage if the disable is broken).
/// Source: `tests/docs/cli/param/23_trace.md § EC-2`.
#[ test ]
fn it49_trace_0_no_trace_on_stderr()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "trace-off-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "trace::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "[trace]" ),
    "trace::0 must not emit [trace] lines on stderr, got:\n{err}",
  );
}

// ── it50 ──────────────────────────────────────────────────────────────────────

/// it50 (EC-3): `trace::2` is out of range for the boolean parameter
/// (only 0 and 1 are valid) → exit 1.
///
/// Source: `tests/docs/cli/param/23_trace.md § EC-3`.
#[ test ]
fn it50_trace_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "trace::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "trace::2 must produce error on stderr",
  );
}

// ── it51 ──────────────────────────────────────────────────────────────────────

/// it51 (EC-4): `trace::yes` is a type mismatch → exit 1.
///
/// Source: `tests/docs/cli/param/23_trace.md § EC-4`.
#[ test ]
fn it51_trace_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "trace::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "trace::yes must produce error on stderr",
  );
}

// ── it52 ──────────────────────────────────────────────────────────────────────

/// it52 (EC-5): default behavior (no `trace::` param) — no `[trace]`
/// lines appear on stderr; trace is off by default (default = 0).
///
/// Uses a no-token account to exercise the fetch path; absence of `[trace]` lines
/// confirms the default is correctly set to disabled.
/// Source: `tests/docs/cli/param/23_trace.md § EC-5`.
#[ test ]
fn it52_trace_default_off()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-trace-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "[trace]" ),
    "default (no trace:: param) must not emit [trace] lines on stderr, got:\n{err}",
  );
}

// ── Sort parameter acceptance (IT-44 – IT-50) ─────────────────────────────────

/// it043 (IT-44/AC-01): `sort::name` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts the `sort::name` value without an unknown-parameter
/// error. The empty store produces `(no accounts configured)` — confirms the param
/// is parsed before any fetch occurs.
/// Source: `tests/docs/cli/command/009_usage.md § IT-44`.
#[ test ]
fn it043_sort_name_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::name must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it044 (IT-45/AC-02): `sort::endurance` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-45`.
#[ test ]
fn it044_sort_endurance_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::endurance must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it045 (IT-46/AC-03): `sort::drain` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-46`.
#[ test ]
fn it045_sort_drain_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::drain must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it046 (IT-47/AC-04): `sort::reset` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-47`.
#[ test ]
fn it046_sort_reset_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::reset must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it047 (IT-48/AC-09): unknown `sort::` value → exit 1; stderr names all four
/// valid values so the operator can correct the typo without consulting docs.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-48`.
#[ test ]
fn it047_sort_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "sort::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "name", "endurance", "drain", "reset" ]
  {
    assert!(
      err.contains( value ),
      "sort::bogus error must name valid value `{value}` (AC-09), got:\n{err}",
    );
  }
}

/// it048 (IT-49/AC-10): unknown `prefer::` value → exit 1; stderr names all three
/// valid values so the operator can correct the typo without consulting docs.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-49`.
#[ test ]
fn it048_prefer_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "prefer::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "any", "opus", "sonnet" ]
  {
    assert!(
      err.contains( value ),
      "prefer::bogus error must name valid value `{value}` (AC-10), got:\n{err}",
    );
  }
}

/// it049 (IT-50): `.usage.help` output includes `sort`, `desc`, and `prefer` params.
///
/// Verifies the parameter registration in `lib.rs` surfaced correctly to the
/// help system after TSK-177 added the three sort-control params.
/// Source: `tests/docs/cli/command/009_usage.md § IT-50`.
#[ test ]
fn it049_usage_help_shows_sort_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for param in &[ "sort", "desc", "prefer" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}` (IT-50), got:\n{text}",
    );
  }
}

// ── desc:: parameter acceptance and direction (026_desc EC-1–EC-3, CC-1–CC-2) ─

/// it050 (`026_desc` EC-1): `desc::0` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `desc::0` as a valid ascending-direction override
/// without an unknown-parameter or type-mismatch error.
/// Source: `tests/docs/cli/param/026_desc.md § EC-1`.
#[ test ]
fn it050_desc_0_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "desc::0 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it051 (`026_desc` EC-2): `desc::1` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `desc::1` as a valid descending-direction override.
/// Source: `tests/docs/cli/param/026_desc.md § EC-2`.
#[ test ]
fn it051_desc_1_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "desc::1 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// `it052_desc_2_rejected` (`026_desc` EC-3): `desc::2` out of range → exit 1.
///
/// `desc::` is a boolean integer param (0 or 1). The `_` arm in `parse_usage_params`
/// rejects any other integer with `ArgumentTypeMismatch`. Exit 1, stderr non-empty.
/// Source: `tests/docs/cli/param/026_desc.md § EC-3`.
#[ test ]
fn it052_desc_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "desc::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "desc::2 must produce error on stderr" );
}

/// it053 (`026_desc` CC-1): `sort::name desc::0` and `sort::name` produce identical row order.
///
/// Explicitly setting `desc::0` on `sort::name` (whose canonical direction is ascending)
/// must produce the same A→Z output as the implicit default — both display `a@x.com`
/// before `z@x.com` in the table. No divergence from omitting `desc::`.
/// Source: `tests/docs/cli/param/026_desc.md § CC-1`.
#[ test ]
fn it053_sort_name_desc_0_identical_to_sort_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "z@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out_default  = run_cs_with_env( &[ ".usage", "sort::name"           ], &[ ( "HOME", home ) ] );
  let out_explicit = run_cs_with_env( &[ ".usage", "sort::name", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default,  0 );
  assert_exit( &out_explicit, 0 );

  let text_d = stdout( &out_default );
  let text_e = stdout( &out_explicit );

  let a_d = text_d.find( "a@x.com" ).expect( "a@x.com must appear in sort::name output" );
  let z_d = text_d.find( "z@x.com" ).expect( "z@x.com must appear in sort::name output" );
  let a_e = text_e.find( "a@x.com" ).expect( "a@x.com must appear in sort::name desc::0 output" );
  let z_e = text_e.find( "z@x.com" ).expect( "z@x.com must appear in sort::name desc::0 output" );

  assert!(
    a_d < z_d,
    "sort::name must show a@x.com before z@x.com (ascending), got:\n{text_d}",
  );
  assert!(
    a_e < z_e,
    "sort::name desc::0 must show a@x.com before z@x.com (026_desc CC-1 — same as implicit default), got:\n{text_e}",
  );
}

/// it054 (`026_desc` CC-2): `sort::name desc::1` reverses alphabetical order — `z@x.com` before `a@x.com`.
///
/// `desc::1` on `sort::name` (canonical direction: ascending) produces descending (Z→A) row
/// order — the behavioral divergence from `sort::name desc::0`.
/// Source: `tests/docs/cli/param/026_desc.md § CC-2`.
#[ test ]
fn it054_sort_name_desc_1_reverses_order()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "z@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "sort::name", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let a_pos = text.find( "a@x.com" ).expect( "a@x.com must appear in output" );
  let z_pos = text.find( "z@x.com" ).expect( "z@x.com must appear in output" );
  assert!(
    z_pos < a_pos,
    "sort::name desc::1 must show z@x.com before a@x.com (026_desc CC-2 — reversed from ascending default), got:\n{text}",
  );
}

// ── prefer:: parameter acceptance (027_prefer EC-1–EC-3) ─────────────────────

/// it055 (`027_prefer` EC-1): `prefer::any` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `prefer::any` without unknown-parameter or type error.
/// Source: `tests/docs/cli/param/027_prefer.md § EC-1`.
#[ test ]
fn it055_prefer_any_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::any" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::any must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it056 (`027_prefer` EC-2): `prefer::opus` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/param/027_prefer.md § EC-2`.
#[ test ]
fn it056_prefer_opus_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::opus must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it057 (`027_prefer` EC-3): `prefer::sonnet` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/param/027_prefer.md § EC-3`.
#[ test ]
fn it057_prefer_sonnet_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::sonnet" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::sonnet must be accepted and show no-accounts message, got:\n{text}",
  );
}

// ── Sort × JSON interaction (025_sort CC-1, 004_sort_control CC-1) ────────────

/// it058 (`025_sort` CC-1 / `004_sort_control` CC-1): JSON array order is alphabetical
/// regardless of `sort::` strategy.
///
/// `render_json` always uses the original alphabetical account slice; `sort::` strategy
/// only reorders text rendering. Accounts written in non-alpha order (`b@x.com` before
/// `a@x.com`) are sorted by `account::list()` and stay alphabetical in JSON output
/// regardless of whether `sort::name` or `sort::endurance` is requested (AC-13).
/// Source: `tests/docs/cli/param/025_sort.md § CC-1`.
#[ test ]
fn it058_sort_json_unaffected_by_sort_strategy()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write in non-alphabetical order to verify account::list() sorts, not filesystem order.
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out_name      = run_cs_with_env( &[ ".usage", "sort::name",      "format::json" ], &[ ( "HOME", home ) ] );
  let out_endurance = run_cs_with_env( &[ ".usage", "sort::endurance", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_name,      0 );
  assert_exit( &out_endurance, 0 );

  let json_name      = stdout( &out_name );
  let json_endurance = stdout( &out_endurance );

  let a_n = json_name.find( "a@x.com" ).expect( "a@x.com in sort::name json" );
  let b_n = json_name.find( "b@x.com" ).expect( "b@x.com in sort::name json" );
  assert!(
    a_n < b_n,
    "sort::name format::json must place a@x.com before b@x.com (alphabetical), got:\n{json_name}",
  );

  let a_e = json_endurance.find( "a@x.com" ).expect( "a@x.com in sort::endurance json" );
  let b_e = json_endurance.find( "b@x.com" ).expect( "b@x.com in sort::endurance json" );
  assert!(
    a_e < b_e,
    "sort::endurance format::json must place a@x.com before b@x.com (sort:: does not affect JSON, AC-13), got:\n{json_endurance}",
  );
}

// ── Case-sensitivity corner cases ─────────────────────────────────────────────

/// it059: `sort::Name` (capital N) → exit 1 — `SortStrategy::parse` is case-sensitive.
///
/// `"Name"` does not match any branch in `SortStrategy::parse`; the underscore arm
/// returns `ArgumentTypeMismatch`. Exit 1, stderr contains the error message.
#[ test ]
fn it059_sort_uppercase_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "sort::Name" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "sort::Name must produce error on stderr (case-sensitive parse)" );
}

/// it060: `prefer::Opus` (capital O) → exit 1 — `PreferStrategy::parse` is case-sensitive.
///
/// `"Opus"` does not match any branch in `PreferStrategy::parse`; the underscore arm
/// returns `ArgumentTypeMismatch`. Exit 1, stderr contains the error message.
#[ test ]
fn it060_prefer_uppercase_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "prefer::Opus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "prefer::Opus must produce error on stderr (case-sensitive parse)" );
}

// ── sort::reset desc::1 combination acceptance ────────────────────────────────

/// it061: `sort::reset desc::1` accepted with empty credential store → exit 0.
///
/// Verifies that the `sort::reset desc::1` parameter combination does not cause
/// a parse error — both parameters are individually valid and the combination
/// must be accepted without `ArgumentTypeMismatch` or unknown-param errors.
#[ test ]
fn it061_sort_reset_desc1_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::reset", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::reset desc::1 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it062: `sort::endurance desc::0` accepted with empty credential store → exit 0.
///
/// `sort::endurance` has canonical direction `desc::1` (qualified first). `desc::0` explicitly
/// overrides to ascending — the parser must accept this as a valid direction override.
#[ test ]
fn it062_sort_endurance_desc0_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::endurance", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::endurance desc::0 must be accepted and show no-accounts message, got:\n{text}",
  );
}

// ── next:: parameter acceptance (023_next_account_strategies AC-01/AC-03–AC-07) ─

/// it063 (AC-01): `next::all` accepted with empty credential store → exit 0.
///
/// TDD guard: fails before `next` is registered (unknown-parameter error).
/// After registration, the parser accepts `all` and the empty store short-circuits
/// to `(no accounts configured)`.
#[ test ]
fn it063_next_all_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::all" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "next::all must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it064 (AC-03): `next::session` accepted with empty credential store → exit 0.
///
/// TDD guard for `session` value. The parser must accept the string without error;
/// empty store produces the no-accounts message.
#[ test ]
fn it064_next_session_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::session" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "next::session must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it065 (AC-04): `next::endurance` accepted with empty credential store → exit 0.
#[ test ]
fn it065_next_endurance_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "next::endurance must be accepted",
  );
}

/// it066 (AC-05): `next::drain` accepted with empty credential store → exit 0.
#[ test ]
fn it066_next_drain_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "next::drain must be accepted",
  );
}

/// it067 (AC-06): `next::reset` accepted with empty credential store → exit 0.
#[ test ]
fn it067_next_reset_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "next::reset must be accepted",
  );
}

/// it068 (AC-07): unknown `next::` value → exit 1; stderr names all five valid values.
///
/// `NextStrategy::parse` returns an error for unrecognised strings; `parse_usage_params`
/// converts it to `ArgumentTypeMismatch` → exit 1. The error message must name every
/// valid value so the operator can correct a typo.
#[ test ]
fn it068_next_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "next::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "all", "session", "endurance", "drain", "reset" ]
  {
    assert!(
      err.contains( value ),
      "next::bogus error must name valid value `{value}` (AC-07), got:\n{err}",
    );
  }
}

/// it069 (AC-01): default `next::all` — no `→` marker appears in any table row.
///
/// Two no-token accounts are written so the table is non-empty. With `next::all`
/// (the default, no param needed), the recommendation → marker is suppressed in
/// the table body; the multi-strategy footer handles recommendations instead.
/// No row in the text output should contain `→`.
#[ test ]
fn it069_next_all_suppresses_arrow_in_table()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );

  // Default (no next:: param) = next::all.
  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // No table row should contain the → Unicode arrow (U+2192).
  let arrow_in_row = text.lines().any( |l| l.contains( '\u{2192}' ) );
  assert!(
    !arrow_in_row,
    "next::all (default) must not place → in any table row, got:\n{text}",
  );
}

// ── cols:: parameter acceptance and column visibility (AC-22–AC-23) ──────────

/// it070 (AC-23): `cols::+sub` accepted with empty credential store → exit 0.
///
/// TDD guard: fails before `cols` is registered (unknown-parameter error).
/// After registration, the parser accepts `+sub` without error; empty store
/// produces the no-accounts message.
#[ test ]
fn it070_cols_sub_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "cols::+sub must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it071 (AC-22): `cols::+sub` with an account → output table contains the "Sub" header.
///
/// By default `sub` is OFF. `cols::+sub` adds it. This test writes a no-token
/// account (quota cells will be dashes) and verifies the "Sub" header appears
/// in the rendered table, confirming the column is actually emitted.
#[ test ]
fn it071_cols_sub_shows_sub_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must include the Sub column header in output, got:\n{text}",
  );
}

/// it072 (AC-23): `cols::+bogus_col` — unknown column ID → exit 1; stderr names valid IDs.
///
/// `ColsVisibility::apply_modifier` returns an error for unknown IDs; `parse_usage_params`
/// converts it to `ArgumentTypeMismatch` → exit 1. The error must name at least one
/// valid ID so the operator can identify the typo.
#[ test ]
fn it072_cols_unknown_id_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "cols::+bogus_col" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  // The error must mention at least one valid column ID.
  let mentions_valid = [ "status", "expires", "sub", "renews", "5h_left" ]
    .iter()
    .any( |id| err.contains( id ) );
  assert!(
    mentions_valid,
    "cols::+bogus_col error must name at least one valid column ID, got:\n{err}",
  );
}

/// it073: `.usage.help` output includes `next` and `cols` params.
///
/// Verifies the parameter registrations in `lib.rs` are surfaced correctly
/// to the help system after Phase 3 added both params.
#[ test ]
fn it073_usage_help_shows_next_cols_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for param in &[ "next", "cols" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}`, got:\n{text}",
    );
  }
}

// ── cols:: column visibility defaults and modifiers ───────────────────────────

/// it074 (AC-22): Sub absent by default — no `cols::` → "Sub" not in table header.
///
/// `sub` is off in `ColsVisibility::default_set()`. This test verifies that the
/// rendered table omits the "Sub" column header when `cols::` is not specified.
#[ test ]
fn it074_sub_hidden_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Sub" ),
    "without cols::+sub, the Sub column must not appear in output, got:\n{text}",
  );
}

/// it075 (AC-23): `cols::+7d_son_reset` → "7d Son Reset" appears in table header.
///
/// `7d_son_reset` is off by default. `cols::+7d_son_reset` adds it to the header.
#[ test ]
fn it075_cols_plus_7d_son_reset_shows_header()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+7d_son_reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "7d Son Reset" ),
    "cols::+7d_son_reset must include 7d Son Reset header, got:\n{text}",
  );
}

/// it076 (AC-22): "7d Son Reset" absent by default — no `cols::` → column not in header.
#[ test ]
fn it076_7d_son_reset_hidden_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "7d Son Reset" ),
    "without cols::+7d_son_reset, the column must not appear in output, got:\n{text}",
  );
}

/// it077 (AC-22): `cols::-renews` → "~Renews" absent from table header.
#[ test ]
fn it077_cols_minus_renews_hides_header()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::-renews" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "~Renews" ),
    "cols::-renews must hide the ~Renews column header, got:\n{text}",
  );
}

/// it078 (AC-22): `cols::+sub,-7d_son` composite modifier — Sub present, 7d(Son) absent.
#[ test ]
fn it078_cols_composite_add_and_remove()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+sub,-7d_son" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Sub" ),       "cols::+sub must add Sub header, got:\n{text}" );
  assert!( !text.contains( "7d(Son)" ),  "cols::-7d_son must remove 7d(Son) header, got:\n{text}" );
}

/// it079 (AC-22): flag and account (name) columns always present regardless of `cols::` removals.
///
/// Removing all optional columns still leaves the structural flag (blank) and
/// Account (name) columns. The account name must appear in the output.
#[ test ]
fn it079_cols_structural_cols_always_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "user@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "cols::-status,-expires,-renews,-5h_left,-5h_reset,-7d_left,-7d_son,-7d_reset" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "user@x.com" ),
    "account name must always appear in output regardless of cols:: removals, got:\n{text}",
  );
}

// ── next:: footer threshold (023_next_account_strategies AC-09) ───────────────

/// it080 (AC-09): footer absent when < 2 valid accounts.
///
/// Two no-token accounts result in zero valid (Ok) quota fetches.
/// The footer (Valid: X / Y …) must not appear when `valid_count < 2`.
#[ test ]
fn it080_next_footer_absent_when_no_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Valid:" ),
    "footer must not appear when no accounts have valid quota data, got:\n{text}",
  );
}

/// it081 (AC-06): `format::json` output is identical regardless of `next::` value.
///
/// `render_json` does not reference `NextStrategy`; JSON output is unaffected.
/// Tests with an empty store (JSON = `[]`) to avoid network calls.
#[ test ]
fn it081_next_json_output_unchanged_by_next_param()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env(
    &[ ".usage", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  let out_session = run_cs_with_env(
    &[ ".usage", "format::json", "next::session" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_default, 0 );
  assert_exit( &out_session, 0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_session ),
    "format::json output must be identical regardless of next:: value",
  );
}
