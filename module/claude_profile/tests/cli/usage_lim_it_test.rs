//! Integration tests: IT-205–IT-247 — `.usage` filter, `get::`, and format tests.
//!
//! Mixed population: tests whose names contain `lim_it` require a real Anthropic
//! OAuth access token; the rest are offline-deterministic. The threshold-filter
//! tests (it207–it210, it213) seed exact quota-cache values via `seed_quota_cache()`
//! and render through the G1 not-owned cache path (fetch.rs) — no token, no HTTP.
//!
//! Covers `min_5h/min_7d` threshold filters and edge cases, `get::` live/offline
//! extraction, `format::tsv`, `no_color::`, `abs::`, and Feature 037 owner
//! column display.
//!
//! No nextest filter excludes live tests — they run by default whenever
//! ~/.claude is mounted (see .config/nextest.toml) and panic loudly if a
//! live token is unavailable.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, assert_exit,
  write_account, write_account_with_token, write_account_profile_json,
  seed_quota_cache,
  live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;


/// it205 (028 FT-02): `offset::2 count::3` with 5 accounts shows rows 3-5 from
/// the full sorted list.
///
/// Validates that combining `offset::` and `count::` selects a window from the
/// sorted row set. Accounts have no tokens so quota shows errors, but the
/// names still appear in sorted order.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-02]
///       [`docs/feature/028_usage_row_filtering.md` AC-02]
#[ test ]
fn it205_ft028_02_offset2_count3_windows_result()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write 5 accounts with deterministic sort order (a < b < c < d < e).
  write_account( dir.path(), "acct-a@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-d@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-e@test.com", "max", "standard", FAR_FUTURE_MS, false );

  // When-A: all rows, no offset — gives baseline sorted order.
  let out_all = run_cs_with_env( &[ ".usage", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_all, 0 );
  let all_text = stdout( &out_all );
  let all_names : Vec< &str > = all_text.lines()
    .filter( | l | l.contains( "acct-" ) )
    .collect();

  // When-B: offset::2 count::3 — should show rows at positions 2, 3, 4 (0-indexed).
  let out_win = run_cs_with_env(
    &[ ".usage", "sort::name", "offset::2", "count::3" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_win, 0 );
  let win_text = stdout( &out_win );
  let win_names : Vec< &str > = win_text.lines()
    .filter( | l | l.contains( "acct-" ) )
    .collect();

  assert_eq!( win_names.len(), 3, "offset::2 count::3 with 5 accounts must show exactly 3 rows" );
  // Rows 3-5 from When-A (0-indexed: positions 2, 3, 4) must match When-B.
  assert_eq!(
    win_names, &all_names[ 2..5 ],
    "offset::2 count::3 rows must match rows 3-5 from full sorted list",
  );
}

// ── it211: min_5h::50 with absent session data — row passes (041 EC-6) ────────

/// it211 (041 EC-6): `min_5h::50` with an account whose session quota is absent
/// (no `five_hour` data from API) — the row is NOT hidden.
///
/// Absent session data is treated as 100% remaining so the filter passes.
/// In offline tests, accounts without tokens have no quota data (API fails),
/// so the row passes the `min_5h` filter.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-6]
#[ test ]
fn it211_min_5h_absent_data_passes_filter()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Account without accessToken — quota fetch will fail; five_hour data absent.
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "min_5h::50" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Row must appear (absent data does not trigger the threshold filter).
  assert!(
    text.contains( "acct@test.com" ),
    "min_5h::50 must not hide row when five_hour data is absent, got:\n{text}",
  );
}

// ── it212: min_7d::30 with absent weekly data — row passes (042 EC-6) ─────────

/// it212 (042 EC-6): `min_7d::30` with an account whose weekly quota is absent
/// (no `seven_day` data from API) — the row is NOT hidden.
///
/// Absent weekly data is treated as 100% remaining so the filter passes.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-6]
#[ test ]
fn it212_min_7d_absent_data_passes_filter()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "min_7d::30" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "acct@test.com" ),
    "min_7d::30 must not hide row when seven_day data is absent, got:\n{text}",
  );
}

// ── it241: min_5h + min_7d both applied — Err account passes both ─────────────

/// it241: `min_5h::50 min_7d::30` both applied simultaneously; Err account passes both.
///
/// Each threshold filter independently passes Err accounts (absent data ≠ exhausted).
/// When both are applied, the Err account survives both retain passes.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-6] and [`tests/docs/cli/param/042_min_7d.md` EC-6]
#[ test ]
fn it241_min_5h_and_min_7d_both_pass_err_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "min_5h::50", "min_7d::30" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "acct@test.com" ),
    "min_5h::50 min_7d::30 must not hide row when both quota fields are absent, got:\n{text}",
  );
}

// ── it242: min_5h + only_valid — only_valid removes Err even after min_5h passes ──

/// it242: `min_5h::1 only_valid::1` — Err account passes `min_5h` (absent data),
/// but is subsequently removed by `only_valid::1` (which filters on `result.is_err()`).
///
/// Tests that `min_5h` and `only_valid` are independent filters in AND-composition:
/// `only_valid` still applies to accounts that survived `min_5h`.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-6] + [`tests/docs/cli/param/043_only_valid.md` EC-4]
#[ test ]
fn it242_min_5h_only_valid_removes_err_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Err account: passes min_5h::1 (absent data), but NOT only_valid::1
  write_account( dir.path(), "acct-err@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "min_5h::1", "only_valid::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // only_valid::1 must remove the Err account even though min_5h::1 would have kept it.
  assert!(
    text.contains( "(no accounts configured)" ),
    "min_5h::1 only_valid::1 must produce empty table for all-Err accounts, got:\n{text}",
  );
}

// ── it243: min_5h::1 get::account on Err account — returns name ───────────────

/// it243: `min_5h::1 get::account` with an Err account — Err passes the `min_5h`
/// filter (absent data ≠ exhausted), and then `get::account` extracts its name.
///
/// This is the positive complement of it242: without `only_valid::1`, the Err account
/// survives `min_5h` and `get::` operates on it normally.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-6] + [`tests/docs/cli/param/045_get.md` EC-2]
#[ test ]
fn it243_min_5h_get_account_err_passes_returns_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "min_5h::1", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Err account passes min_5h (absent data), so get::account returns its name.
  assert_eq!(
    text.trim(),
    "acct@test.com",
    "min_5h::1 get::account must return account name when Err account passes filter, got:\n{text}",
  );
}

// ── it244: get::host when profile.json absent — empty stdout ─────────────────

/// it244: `get::host` on an account without `profile.json` — returns empty stdout.
///
/// `read_profile_metadata` returns `(String::new(), String::new())` when the file
/// is absent.  `extract_get_field(aq, GetField::Host, ...)` returns `aq.host.clone()`
/// = "".  Empty string → `content = String::new()` → empty stdout (exit 0).
///
/// Spec: [`tests/docs/cli/param_group/006_account_targeting.md` CC-2 implication]
#[ test ]
fn it244_get_host_absent_profile_json_empty_stdout()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No profile.json written — host is absent.
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "get::host" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim().is_empty(),
    "get::host on account without profile.json must output empty stdout, got:\n{text}",
  );
}

// ── it245: min_7d::1 get::account on Err account — returns name ──────────────

/// it245: `min_7d::1 get::account` with an Err account — Err passes the `min_7d`
/// filter (absent data ≠ exhausted), and then `get::account` extracts its name.
///
/// Symmetric counterpart of it243 (`min_5h`+`get::`): confirms the same Err-pass
/// semantics apply to the `min_7d` threshold filter.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md`] + [`tests/docs/cli/param/045_get.md` EC-2]
#[ test ]
fn it245_min_7d_get_account_err_passes_returns_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "min_7d::1", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(), "acct@test.com",
    "`min_7d::1 get::account` on Err account must return bare name, got:\n{text}",
  );
}

// ── it246: min_7d::1 + only_valid — only_valid removes Err even after min_7d passes ──

/// it246: `min_7d::1 only_valid::1` — Err account passes `min_7d` (absent data),
/// but is subsequently removed by `only_valid::1` (which filters on `result.is_ok()`).
///
/// Symmetric counterpart of it242 (`min_5h`+`only_valid`): confirms the AND-composition
/// ordering applies identically to the `min_7d` threshold filter.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md`] + [`tests/docs/cli/param/043_only_valid.md` EC-4]
#[ test ]
fn it246_min_7d_only_valid_removes_err_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-err@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "min_7d::1", "only_valid::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "`min_7d::1 only_valid::1` must produce empty table for all-Err accounts, got:\n{text}",
  );
}

// ── it220: cols::+host get::host extracts bare host string (029 FT-07) ────────

/// it220 (029 FT-07): `cols::+host get::host` extracts the host value from
/// profile.json as a bare string (no table headers, no footer).
///
/// Host column data comes from `{name}.json`, not from the live quota
/// API. The bare extraction works offline even when quota fetch fails.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-07]
#[ test ]
fn it220_ft029_07_get_host_extracts_bare()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "acct@test.com", Some( "mybox" ), None );

  let out = run_cs_with_env(
    &[ ".usage", "cols::+host", "get::host" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert_eq!(
    text, "mybox",
    "cols::+host get::host must output bare 'mybox' with no table chrome, got:\n{text}",
  );
}

// ── it221: cols::+host with no profile.json — empty cell, exit 0 (029 FT-09) ─

/// it221 (029 FT-09 When-A): `cols::+host` with a saved account that has no
/// `profile.json` — the command must exit 0. The Host column is present in the
/// table header; the account row shows an empty cell, not an error.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-09]
#[ test ]
fn it221_ft029_09_usage_no_profile_shows_empty_host()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Account saved with no profile.json (no host:: was given).
  write_account( dir.path(), "acct@test.com", "max", "standard", FAR_FUTURE_MS, false );
  // Deliberately no write_account_profile_json call.

  let out = run_cs_with_env( &[ ".usage", "cols::+host" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Header row must contain "Host".
  assert!(
    text.contains( "Host" ),
    "cols::+host must show Host column header even when profile.json is absent, got:\n{text}",
  );
}

// ── it206: lim_it only_next::1 shows exactly the recommended account (028 FT-04) ───

/// it206 `lim_it` (028 FT-04): `only_next::1` shows exactly one row — the recommended account.
///
/// With two live accounts, the active sort strategy selects one winner.
/// `only_next::1` must show only that row; all others are hidden.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-04]
#[ test ]
fn it206_lim_it_ft028_04_only_next_1_shows_recommended()
{
  let token = live_active_token().expect( "it206: live API token required — no ~/.claude/.credentials.json" );
  require_live_api( "it206" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage", "only_next::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Count data rows (lines containing an account name).
  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    data_rows, 1,
    "only_next::1 must show exactly 1 row (the recommended account), got:\n{text}",
  );
}

// ── it207–it210: min_5h/min_7d threshold filters (041/042 EC-1/EC-2) ─────────

/// it207 (041 EC-1, 028 FT-05): `min_5h::50` hides rows below the threshold.
///
/// Offline-deterministic: each account's quota comes from a cache seeded with a
/// chosen utilization (`seed_quota_cache`), rendered via the G1 not-owned cache
/// path — no token, no HTTP. Fixture per FT-05: A `5h Left = 80%`, B `= 50%`,
/// C `= 30%`. A and B shown (B exactly at the threshold — inclusive `>=`);
/// C hidden.
///
/// Spec: [`tests/docs/cli/param/41_min_5h.md` EC-1]
///       [`tests/docs/feature/028_usage_row_filtering.md` FT-05]
#[ test ]
fn it207_min_5h_50_hides_below_threshold()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c@test.com", "max", "standard", FAR_FUTURE_MS, false );
  seed_quota_cache( dir.path(), "acct-a@test.com", Some( 20.0 ), Some( 50.0 ) ); // 5h Left 80
  seed_quota_cache( dir.path(), "acct-b@test.com", Some( 50.0 ), Some( 50.0 ) ); // 5h Left 50
  seed_quota_cache( dir.path(), "acct-c@test.com", Some( 70.0 ), Some( 50.0 ) ); // 5h Left 30

  let out = run_cs_with_env( &[ ".usage", "min_5h::50" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let rows : Vec< &str > = text.lines().filter( | l | l.contains( "@test.com" ) ).collect();
  assert!(
    rows.iter().any( | l | l.contains( "acct-a@test.com" ) ),
    "min_5h::50 must show acct-a (5h Left 80), got:\n{text}",
  );
  assert!(
    rows.iter().any( | l | l.contains( "acct-b@test.com" ) ),
    "min_5h::50 must show acct-b (5h Left 50 — inclusive boundary), got:\n{text}",
  );
  assert!(
    !rows.iter().any( | l | l.contains( "acct-c@test.com" ) ),
    "min_5h::50 must hide acct-c (5h Left 30), got:\n{text}",
  );
}

/// it208 (041 EC-2): `min_5h::50` boundary — row at exactly 50% Left shown
/// (inclusive `>=`), row just below hidden.
///
/// Offline-deterministic via seeded quota cache: A `5h Left = 50%` (shown),
/// B `5h Left = 49%` (hidden). Locks the comparison direction on both sides of
/// the boundary — a strict `>` would hide A; an inverted predicate would show B.
///
/// Spec: [`tests/docs/cli/param/41_min_5h.md` EC-2]
#[ test ]
fn it208_min_5h_50_inclusive_boundary()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b@test.com", "max", "standard", FAR_FUTURE_MS, false );
  seed_quota_cache( dir.path(), "acct-a@test.com", Some( 50.0 ), Some( 50.0 ) ); // 5h Left 50 — at threshold
  seed_quota_cache( dir.path(), "acct-b@test.com", Some( 51.0 ), Some( 50.0 ) ); // 5h Left 49 — just below

  let out = run_cs_with_env( &[ ".usage", "min_5h::50" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let rows : Vec< &str > = text.lines().filter( | l | l.contains( "@test.com" ) ).collect();
  assert!(
    rows.iter().any( | l | l.contains( "acct-a@test.com" ) ),
    "min_5h::50 must show acct-a at exactly 50% Left (inclusive >=), got:\n{text}",
  );
  assert!(
    !rows.iter().any( | l | l.contains( "acct-b@test.com" ) ),
    "min_5h::50 must hide acct-b at 49% Left, got:\n{text}",
  );
}

/// it209 (042 EC-1, 028 FT-06): `min_7d::20` hides rows below the threshold.
///
/// Offline-deterministic via seeded quota cache. Fixture per FT-06:
/// A `7d Left = 60%`, B `= 20%`, C `= 10%`. A and B shown (B exactly at the
/// threshold — inclusive `>=`); C hidden.
///
/// Spec: [`tests/docs/cli/param/42_min_7d.md` EC-1]
///       [`tests/docs/feature/028_usage_row_filtering.md` FT-06]
#[ test ]
fn it209_min_7d_20_hides_below_threshold()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c@test.com", "max", "standard", FAR_FUTURE_MS, false );
  seed_quota_cache( dir.path(), "acct-a@test.com", Some( 20.0 ), Some( 40.0 ) ); // 7d Left 60
  seed_quota_cache( dir.path(), "acct-b@test.com", Some( 20.0 ), Some( 80.0 ) ); // 7d Left 20
  seed_quota_cache( dir.path(), "acct-c@test.com", Some( 20.0 ), Some( 90.0 ) ); // 7d Left 10

  let out = run_cs_with_env( &[ ".usage", "min_7d::20" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let rows : Vec< &str > = text.lines().filter( | l | l.contains( "@test.com" ) ).collect();
  assert!(
    rows.iter().any( | l | l.contains( "acct-a@test.com" ) ),
    "min_7d::20 must show acct-a (7d Left 60), got:\n{text}",
  );
  assert!(
    rows.iter().any( | l | l.contains( "acct-b@test.com" ) ),
    "min_7d::20 must show acct-b (7d Left 20 — inclusive boundary), got:\n{text}",
  );
  assert!(
    !rows.iter().any( | l | l.contains( "acct-c@test.com" ) ),
    "min_7d::20 must hide acct-c (7d Left 10), got:\n{text}",
  );
}

/// it210 (042 EC-2): `min_7d::20` boundary — row at exactly 20% Left shown
/// (inclusive `>=`), row just below hidden.
///
/// Offline-deterministic via seeded quota cache: A `7d Left = 20%` (shown),
/// B `7d Left = 19%` (hidden).
///
/// Spec: [`tests/docs/cli/param/42_min_7d.md` EC-2]
#[ test ]
fn it210_min_7d_20_inclusive_boundary()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b@test.com", "max", "standard", FAR_FUTURE_MS, false );
  seed_quota_cache( dir.path(), "acct-a@test.com", Some( 20.0 ), Some( 80.0 ) ); // 7d Left 20 — at threshold
  seed_quota_cache( dir.path(), "acct-b@test.com", Some( 20.0 ), Some( 81.0 ) ); // 7d Left 19 — just below

  let out = run_cs_with_env( &[ ".usage", "min_7d::20" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let rows : Vec< &str > = text.lines().filter( | l | l.contains( "@test.com" ) ).collect();
  assert!(
    rows.iter().any( | l | l.contains( "acct-a@test.com" ) ),
    "min_7d::20 must show acct-a at exactly 20% Left (inclusive >=), got:\n{text}",
  );
  assert!(
    !rows.iter().any( | l | l.contains( "acct-b@test.com" ) ),
    "min_7d::20 must hide acct-b at 19% Left, got:\n{text}",
  );
}

// ── it213: AND filter composition (028 FT-09) ────────────────────────────────

/// it213 (028 FT-09): `only_valid::1 min_7d::30` AND-composition — shows only
/// non-🔴 rows with `7d Left >= 30%`.
///
/// Offline-deterministic via seeded quota cache. Fixture per FT-09/AC-09:
/// A 🟢 (`7d Left 40`) shown; B 🟢 (`7d Left 25`) hidden by `min_7d::30`;
/// C 🟡 (5h exhausted, `7d Left 40`) shown — `only_valid` keeps 🟡 rows (AC-07);
/// D 🔴 (no cache → quota Err) hidden by `only_valid::1` — even though a bare
/// `min_7d::30` alone would pass an Err row through (absent data ≠ exhausted).
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-09]
///       [`docs/feature/028_usage_row_filtering.md` AC-09]
#[ test ]
fn it213_ft028_09_and_composition()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  for name in [ "acct-a@test.com", "acct-b@test.com", "acct-c@test.com", "acct-d@test.com" ]
  {
    write_account( dir.path(), name, "max", "standard", FAR_FUTURE_MS, false );
  }
  seed_quota_cache( dir.path(), "acct-a@test.com", Some( 20.0 ), Some( 60.0 ) ); // 🟢, 7d Left 40
  seed_quota_cache( dir.path(), "acct-b@test.com", Some( 20.0 ), Some( 75.0 ) ); // 🟢, 7d Left 25
  seed_quota_cache( dir.path(), "acct-c@test.com", Some( 95.0 ), Some( 60.0 ) ); // 🟡 (5h Left 5), 7d Left 40
  // acct-d: no cache seeded — the G1 not-owned path yields quota Err → 🔴.

  let out = run_cs_with_env(
    &[ ".usage", "only_valid::1", "min_7d::30" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let rows : Vec< &str > = text.lines().filter( | l | l.contains( "@test.com" ) ).collect();
  assert!(
    rows.iter().any( | l | l.contains( "acct-a@test.com" ) ),
    "only_valid::1 min_7d::30 must show acct-a (🟢, 7d Left 40), got:\n{text}",
  );
  assert!(
    !rows.iter().any( | l | l.contains( "acct-b@test.com" ) ),
    "only_valid::1 min_7d::30 must hide acct-b (7d Left 25 < 30), got:\n{text}",
  );
  assert!(
    rows.iter().any( | l | l.contains( "acct-c@test.com" ) ),
    "only_valid::1 min_7d::30 must show acct-c (🟡 is valid — AC-07), got:\n{text}",
  );
  assert!(
    !rows.iter().any( | l | l.contains( "acct-d@test.com" ) ),
    "only_valid::1 min_7d::30 must hide acct-d (🔴 quota Err), got:\n{text}",
  );
}

// ── it214: lim_it get::7d_left bare extraction (028 FT-10) ───────────────────

/// it214 `lim_it` (028 FT-10): `sort::name get::7d_left` outputs a bare
/// percentage string with no table headers, separator lines, or footer.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-10]
#[ test ]
fn it214_lim_it_ft028_10_get_7d_left_bare()
{
  let token = live_active_token().expect( "it214: live API token required — no ~/.claude/.credentials.json" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env(
    &[ ".usage", "sort::name", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // No table chrome: no heading, no separator row, no footer.
  assert!(
    !text.contains( "Quota" ) && !text.contains( "7d Left" ) && !text.contains( "Valid:" ),
    "get::7d_left must produce bare value output with no table chrome, got:\n{text}",
  );
}

// ── it215: lim_it only_next::1 get::7d_left targeted extraction (028 FT-11) ──

/// it215 `lim_it` (028 FT-11): `only_next::1 get::7d_left` extracts 7d Left
/// for the → account as a bare string.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-11]
#[ test ]
fn it215_lim_it_ft028_11_only_next_get_7d_left()
{
  let token = live_active_token().expect( "it215: live API token required — no ~/.claude/.credentials.json" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_next::1", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Quota" ) && !text.contains( "Valid:" ),
    "only_next::1 get::7d_left must produce bare value, no table chrome, got:\n{text}",
  );
}

// ── it216: lim_it get::status on 🟢 account (028 FT-12) ─────────────────────

/// it216 `lim_it` (028 FT-12): `get::status` on a valid (🟢) account outputs
/// `🟢` (or `🟡`) as a bare string — single emoji, no table chrome.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-12]
#[ test ]
fn it216_lim_it_ft028_12_get_status_green()
{
  let token = live_active_token().expect( "it216: live API token required — no ~/.claude/.credentials.json" );
  require_live_api( "it216" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "get::status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out ).trim().to_string();
  assert!(
    text == "🟢" || text == "🟡",
    "get::status on valid account must output 🟢 or 🟡 as a bare value, got:\n{text}",
  );
}

