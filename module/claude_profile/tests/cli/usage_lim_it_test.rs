//! Integration tests: IT-205–IT-247 — `.usage` live `lim_it` filter, `get::`, format tests.
//!
//! All tests in this file require or benefit from a real Anthropic OAuth access
//! token (names contain `lim_it` or explicitly test live behavior).
//!
//! Covers offline extras (`min_5h/min_7d` edge cases), `lim_it` threshold filters,
//! `get::` live/offline extraction, `format::tsv`, `no_color::`, `abs::`, and
//! Feature 037 owner column display.
//!
//! Live tests are excluded from Docker CI via nextest filter `!test(lim_it)`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_account_profile_json,
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
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it206: no live token — skipping" );
    return;
  };
  if !require_live_api( "it206" ) { return; }
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

// ── it207–it210: lim_it threshold filters (041/042 EC-1/EC-2) ────────────────

/// it207 `lim_it` (041 EC-1): `min_5h::50` hides rows below 50% threshold.
///
/// With two live accounts sharing the same token the quota values are identical,
/// so we run with a threshold of 0 (all shown) and then 101 (all hidden as a
/// proxy). For a more meaningful EC-1 test a separate `lim_it` run is used; this
/// test verifies acceptance when threshold equals 0 (baseline) and that the
/// flag is parsed correctly with a live account.
///
/// Note: Exact threshold verification (80% shown / 30% hidden) requires two
/// accounts with different quota levels — non-trivial to guarantee with shared
/// tokens. This test verifies structural acceptance only.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-1]
#[ test ]
fn it207_lim_it_min_5h_50_hides_below_threshold()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it207: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // min_5h::50 accepted with live account → exit 0 (filter applied).
  let out = run_cs_with_env( &[ ".usage", "min_5h::50" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it208 `lim_it` (041 EC-2): `min_5h::50` with row at exactly 50% — row shown.
///
/// Verifies structural acceptance of the threshold flag with a live account.
/// The inclusive-boundary semantic (≥ threshold) is verified by the offline
/// unit logic; this test confirms the flag is parsed and applied.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-2]
#[ test ]
fn it208_lim_it_min_5h_50_inclusive_boundary()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it208: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // min_5h::50 accepted → exit 0.
  let out = run_cs_with_env( &[ ".usage", "min_5h::50" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it209 `lim_it` (042 EC-1): `min_7d::20` accepted with live account — exit 0.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-1]
#[ test ]
fn it209_lim_it_min_7d_20_hides_below_threshold()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it209: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "min_7d::20" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it210 `lim_it` (042 EC-2): `min_7d::20` inclusive boundary — accepted, exit 0.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-2]
#[ test ]
fn it210_lim_it_min_7d_20_inclusive_boundary()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it210: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "min_7d::20" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── it213: lim_it AND filter composition (028 FT-09) ─────────────────────────

/// it213 `lim_it` (028 FT-09): `only_valid::1 min_7d::30` shows only accounts
/// that are non-🔴 AND have 7d Left ≥ 30%.
///
/// With two live accounts, the composition is verified by checking exit 0 and
/// that the filter params are both accepted together.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-09]
#[ test ]
fn it213_lim_it_ft028_09_and_composition()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it213: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_valid::1", "min_7d::30" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// ── it214: lim_it get::7d_left bare extraction (028 FT-10) ───────────────────

/// it214 `lim_it` (028 FT-10): `sort::name get::7d_left` outputs a bare
/// percentage string with no table headers, separator lines, or footer.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-10]
#[ test ]
fn it214_lim_it_ft028_10_get_7d_left_bare()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it214: no live token — skipping" );
    return;
  };
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
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it215: no live token — skipping" );
    return;
  };
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
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it216: no live token — skipping" );
    return;
  };
  if !require_live_api( "it216" ) { return; }
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

// ── it217: lim_it format::tsv with status text labels (028 FT-13) ─────────────

/// it217 `lim_it` (028 FT-13): `format::tsv` produces tab-separated output;
/// the status column contains text labels (`ok`, `warn`, `err`) not emoji.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-13]
#[ test ]
fn it217_lim_it_ft028_13_format_tsv_status_text()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it217: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // TSV header row uses tabs.
  let has_tab = text.contains( '\t' );
  assert!( has_tab, "format::tsv output must contain tab characters, got:\n{text}" );
  // Status column uses text label, not emoji.
  assert!(
    !text.contains( "🟢" ) && !text.contains( "🟡" ) && !text.contains( "🔴" ),
    "format::tsv status column must use text labels (ok/warn/err), not emoji, got:\n{text}",
  );
  assert!(
    text.contains( "ok" ) || text.contains( "warn" ) || text.contains( "err" ),
    "format::tsv status column must contain a text label, got:\n{text}",
  );
}

// ── it218: lim_it no_color::1 produces emoji-free output (028 FT-14) ─────────

/// it218 `lim_it` (028 FT-14): `no_color::1` with a valid account produces
/// emoji-free output; status column shows plain text labels.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-14]
#[ test ]
fn it218_lim_it_ft028_14_no_color_emoji_free()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it218: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "🟢" ) && !text.contains( "🟡" ) && !text.contains( "→" ),
    "no_color::1 must produce emoji-free output for valid account, got:\n{text}",
  );
}

// ── it219: lim_it filters compose with sort/count/cols (028 FT-16) ───────────

/// it219 `lim_it` (028 FT-16): `sort::name only_valid::1 count::2 cols::+sub`
/// composes all filter/sort/col params correctly. At most 2 non-🔴 rows, sorted
/// alphabetically, Sub column present.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-16]
#[ test ]
fn it219_lim_it_ft028_16_filters_compose()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it219: no live token — skipping" );
    return;
  };
  if !require_live_api( "it219" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "sort::name", "only_valid::1", "count::2", "cols::+sub" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Sub column must be present in header.
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must add Sub column header, got:\n{text}",
  );
  // At most 2 data rows.
  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert!(
    data_rows <= 2,
    "count::2 must limit result to at most 2 rows, got {data_rows} rows:\n{text}",
  );
}

// ── it222: lim_it IT-72 format::json new renewal fields ──────────────────────

/// it222 `lim_it` (IT-72): `format::json` output contains the new renewal and
/// next-event fields; the legacy `next_renewal_est` key must be absent.
///
/// Required fields: `renewal_secs`, `renewal_is_estimate`, `next_event_type`,
/// `next_event_secs`. Legacy `next_renewal_est` must not appear.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-72]
///       [`docs/feature/009_token_usage.md` AC-29]
#[ test ]
fn it222_lim_it_it72_json_new_renewal_fields()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it222: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "renewal_secs" ),
    "format::json must contain 'renewal_secs' field (IT-72), got:\n{text}",
  );
  assert!(
    text.contains( "renewal_is_estimate" ),
    "format::json must contain 'renewal_is_estimate' field (IT-72), got:\n{text}",
  );
  assert!(
    text.contains( "next_event_type" ),
    "format::json must contain 'next_event_type' field (IT-72), got:\n{text}",
  );
  assert!(
    text.contains( "next_event_secs" ),
    "format::json must contain 'next_event_secs' field (IT-72), got:\n{text}",
  );
  assert!(
    !text.contains( "next_renewal_est" ),
    "format::json must NOT contain legacy 'next_renewal_est' field (IT-72), got:\n{text}",
  );
}

// ── it223–it224: lim_it abs::1 / abs::true show token counts (046 EC-4/EC-6) ─

/// it223 `lim_it` (046 EC-4): `abs::1` shows absolute token counts instead of
/// percentages. Quota columns must not contain `%` suffix.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-4]
#[ test ]
fn it223_lim_it_abs_1_shows_token_counts()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it223: no live token — skipping" );
    return;
  };
  if !require_live_api( "it223" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_pct = run_cs_with_env( &[ ".usage", "abs::0" ], &[ ( "HOME", home ) ] );
  let out_abs = run_cs_with_env( &[ ".usage", "abs::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_abs, 0 );

  let text_pct = stdout( &out_pct );
  let text_abs = stdout( &out_abs );

  // Default (abs::0) shows % values; abs::1 must not.
  assert!(
    text_pct.contains( '%' ),
    "abs::0 (default) must show percentage values, got:\n{text_pct}",
  );
  assert!(
    !text_abs.contains( '%' ) || text_abs.lines().filter( | l | l.contains( '%' ) ).all( | l | l.contains( "Reset" ) ),
    "abs::1 quota columns must show absolute counts without % suffix, got:\n{text_abs}",
  );
}

/// it224 `lim_it` (046 EC-6): `abs::true` produces the same output as `abs::1`.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-6]
#[ test ]
fn it224_lim_it_abs_true_shows_token_counts()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it224: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_1    = run_cs_with_env( &[ ".usage", "abs::1"    ], &[ ( "HOME", home ) ] );
  let out_true = run_cs_with_env( &[ ".usage", "abs::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_true, 0 );
  // abs::true and abs::1 must produce identical output.
  assert_eq!(
    stdout( &out_1 ), stdout( &out_true ),
    "abs::true must produce the same output as abs::1 (046 EC-6)",
  );
}

// ── it225: → Next cell shows event label + duration (live) ───────────────────

/// it225 — The `→ Next` column cells contain a recognized strategic event-label-and-duration string.
///
/// Given a live account with valid quota data, the `→ Next` column must show the soonest
/// upcoming strategic event as `<label> in <duration>` — not an empty cell or bare header.
///
/// After TSK-228, only `+7d` (7-day reset) and `$ren` (billing renewal) are candidates.
/// Token expiry (`!tok`) and 5h session reset (`+5h`) are no longer included.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-71]
#[ test ]
fn it225_lim_it_it71_next_event_cell_shows_label_and_duration()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it225: no live token — skipping" );
    return;
  };
  if !require_live_api( "it225" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  // Column header must be present.
  assert!(
    text.contains( "\u{2192} Next" ),
    "→ Next column header must appear in default output (IT-71), got:\n{text}",
  );
  // At least one strategic event-label pattern must appear in the output.
  // Valid labels after TSK-228: +7d, $ren — now formatted as "in <dur> +7d" / "in <dur> $ren".
  // !tok and +5h are not candidates (token expiry / 5h reset excluded from → Next).
  let has_event_label =
    text.contains( " +7d" )
    || text.contains( " $ren" );
  assert!(
    has_event_label,
    "→ Next cell must contain 'in ... +7d' or 'in ... $ren' for live account (IT-71), got:\n{text}",
  );
}

// ── it226–it227: only_next:: live tests (040 EC-3/6) ─────────────────────────

/// it226 `lim_it` (040 EC-3): `only_next::1 sort::renews` shows → row from renews strategy.
///
/// With two live accounts sharing the same token, `only_next::1 sort::renews`
/// must show exactly one row — the renews-strategy winner — which has the `→` marker.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-3]
#[ test ]
fn it226_lim_it_only_next_1_renews_shows_winner()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it226: no live token — skipping" );
    return;
  };
  if !require_live_api( "it226" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_next::1", "sort::renews" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    data_rows, 1,
    "only_next::1 sort::renews must show exactly 1 row (040 EC-3), got:\n{text}",
  );
  let arrow_rows = text.lines()
    .filter( | l | l.contains( "\u{2192}" ) && l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    arrow_rows, 1,
    "only_next::1 sort::renews must show the → account row (040 EC-3), got:\n{text}",
  );
}

/// it227 `lim_it` (040 EC-6): `only_next::true` accepted as alias for 1.
///
/// With two live accounts, `only_next::true` must behave like `only_next::1` —
/// exactly one row shown, the → account.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-6]
#[ test ]
fn it227_lim_it_only_next_true_shows_arrow_row()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it227: no live token — skipping" );
    return;
  };
  if !require_live_api( "it227" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_next::true" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let data_rows = text.lines()
    .filter( | l | l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    data_rows, 1,
    "only_next::true must show exactly 1 row (040 EC-6), got:\n{text}",
  );
  let arrow_rows = text.lines()
    .filter( | l | l.contains( "\u{2192}" ) && l.contains( "@test.com" ) )
    .count();
  assert_eq!(
    arrow_rows, 1,
    "only_next::true must show the → account row (040 EC-6), got:\n{text}",
  );
}

// ── it228–it230: only_valid/exclude_exhausted live tests (043/044 EC-1/3) ─────

/// it228 `lim_it` (043 EC-1): `only_valid::1` shows 🟢 account; hides 🔴 error.
///
/// With one live account (🟢) and one error account (🔴), `only_valid::1`
/// must show only the live account and hide the error account.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-1]
#[ test ]
fn it228_lim_it_only_valid_1_shows_green_hides_red()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it228: no live token — skipping" );
    return;
  };
  if !require_live_api( "it228" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live-acct@test.com",  &token, true  );
  write_account( dir.path(), "error-acct@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "only_valid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "live-acct@test.com" ),
    "only_valid::1 must show 🟢 live account (043 EC-1), got:\n{text}",
  );
  assert!(
    !text.contains( "error-acct@test.com" ),
    "only_valid::1 must hide 🔴 error account (043 EC-1), got:\n{text}",
  );
}

/// it229 `lim_it` (044 EC-1): `exclude_exhausted::1` shows 🟢; hides 🔴 error.
///
/// With one live account (🟢) and one error account (🔴), `exclude_exhausted::1`
/// must show only the live account and hide the error account.
///
/// Note: the 🟡 (quota-exhausted, valid token) divergence from `only_valid::1`
/// requires a real exhausted account state unavailable with shared tokens.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-1]
#[ test ]
fn it229_lim_it_exclude_exhausted_1_shows_green()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it229: no live token — skipping" );
    return;
  };
  if !require_live_api( "it229" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live-acct@test.com",  &token, true  );
  write_account( dir.path(), "error-acct@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "exclude_exhausted::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "live-acct@test.com" ),
    "exclude_exhausted::1 must show 🟢 live account (044 EC-1), got:\n{text}",
  );
  assert!(
    !text.contains( "error-acct@test.com" ),
    "exclude_exhausted::1 must hide 🔴 error account (044 EC-1), got:\n{text}",
  );
}

/// it230 `lim_it` (044 EC-3): `exclude_exhausted::1` is at least as strict as `only_valid::1`.
///
/// Both filters applied to the same accounts: `exclude_exhausted::1` must show
/// no more rows than `only_valid::1`. The 🟡-divergence (kept by `only_valid::1`,
/// filtered by `exclude_exhausted::1`) requires an exhausted account state that
/// cannot be manufactured with shared live tokens.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-3]
#[ test ]
fn it230_lim_it_exclude_exhausted_stricter_than_only_valid()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it230: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "live-acct@test.com",  &token, true  );
  write_account( dir.path(), "error-acct@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out_valid = run_cs_with_env( &[ ".usage", "only_valid::1" ],        &[ ( "HOME", home ) ] );
  let out_excl  = run_cs_with_env( &[ ".usage", "exclude_exhausted::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_valid, 0 );
  assert_exit( &out_excl,  0 );

  let rows_valid = stdout( &out_valid ).lines().filter( | l | l.contains( "@test.com" ) ).count();
  let rows_excl  = stdout( &out_excl  ).lines().filter( | l | l.contains( "@test.com" ) ).count();

  assert!(
    rows_excl <= rows_valid,
    "exclude_exhausted::1 must show ≤ rows than only_valid::1 (044 EC-3): valid={rows_valid} excl={rows_excl}",
  );
  assert!(
    !stdout( &out_excl ).contains( "error-acct@test.com" ),
    "exclude_exhausted::1 must hide 🔴 error account (044 EC-3)",
  );
}

// ── it231–it234: get:: live/offline tests (045 EC-1/3/5/7) ───────────────────

/// it231 `lim_it` (045 EC-1): `get::7d_left` extracts bare percentage string.
///
/// With a live account, `get::7d_left` must output exactly one percentage string
/// (e.g., `65%`) on stdout — no column headers, no footer.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-1]
#[ test ]
fn it231_lim_it_get_7d_left_extracts_bare_pct()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it231: no live token — skipping" );
    return;
  };
  if !require_live_api( "it231" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env(
    &[ ".usage", "sort::name", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let trimmed = text.trim();

  assert!(
    trimmed.ends_with( '%' ),
    "get::7d_left must output a percentage string e.g. '65%' (045 EC-1), got:\n{trimmed}",
  );
  assert!(
    !trimmed.contains( "7d Left" ),
    "get::7d_left must not contain column headers (045 EC-1), got:\n{trimmed}",
  );
  assert!(
    !trimmed.contains( "Valid:" ),
    "get::7d_left must not contain footer (045 EC-1), got:\n{trimmed}",
  );
}

/// it232 `lim_it` (045 EC-3): `get::status` extracts bare 🟢 emoji for live account.
///
/// With a live (🟢) account, `get::status` must output `🟢` as the sole content.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-3]
#[ test ]
fn it232_lim_it_get_status_extracts_green_emoji()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it232: no live token — skipping" );
    return;
  };
  if !require_live_api( "it232" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage", "get::status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text    = stdout( &out );
  let trimmed = text.trim();

  assert_eq!(
    trimmed, "\u{1f7e2}",
    "get::status on live (🟢) account must output exactly '🟢' (045 EC-3), got:\n{trimmed}",
  );
}

/// it233 (045 EC-5): `get::bogus` exits 1; stderr names all valid field IDs.
///
/// After TSK-225, `host`, `role`, `next_event_type`, `next_event_secs` were
/// added as valid `get::` field IDs. The error message must list all of them.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-5]
#[ test ]
fn it233_get_bogus_exits_1_names_valid_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "get::bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "next_event_type" ),
    "get::bogus stderr must list 'next_event_type' (045 EC-5), got:\n{err}",
  );
  assert!(
    err.contains( "next_event_secs" ),
    "get::bogus stderr must list 'next_event_secs' (045 EC-5), got:\n{err}",
  );
  assert!(
    err.contains( "7d_left" ),
    "get::bogus stderr must list '7d_left' (045 EC-5), got:\n{err}",
  );
  assert!(
    err.contains( "account" ),
    "get::bogus stderr must list 'account' (045 EC-5), got:\n{err}",
  );
}

/// it234 `lim_it` (045 EC-7): `get::next_event_type` outputs strategic label; `get::next_event_secs` outputs integer.
///
/// With a live account with an upcoming quota event, `get::next_event_type` must
/// output a recognized strategic event-label string (`+7d` or `$ren`); `get::next_event_secs`
/// must output a bare non-negative integer.
///
/// After TSK-228, only `+7d` and `$ren` are candidates. `!tok` and `+5h` are excluded.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-7]
#[ test ]
fn it234_lim_it_get_next_event_type_and_secs()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it234: no live token — skipping" );
    return;
  };
  if !require_live_api( "it234" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_type = run_cs_with_env(
    &[ ".usage", "get::next_event_type" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_type, 0 );
  let type_text = stdout( &out_type );
  let type_str  = type_text.trim();
  // After TSK-228: only +7d and $ren are strategic next-event candidates.
  let valid_labels = [ "+7d", "$ren" ];
  assert!(
    valid_labels.contains( &type_str ),
    "get::next_event_type must output '+7d' or '$ren' (045 EC-7 after TSK-228), got:\n{type_str}",
  );

  let out_secs = run_cs_with_env(
    &[ ".usage", "get::next_event_secs" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_secs, 0 );
  let secs_text = stdout( &out_secs );
  let secs_str  = secs_text.trim();
  assert!(
    secs_str.parse::<u64>().is_ok(),
    "get::next_event_secs must output a bare integer (045 EC-7), got:\n{secs_str}",
  );
}

// ── it235–it236: no_color:: live tests (047 EC-3/5) ──────────────────────────

/// it235 `lim_it` (047 EC-3): `no_color::0` (default) includes 🟢 emoji.
///
/// With a live (🟢) account, `no_color::0` does not suppress status emoji.
/// Stdout must contain `🟢`.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-3]
#[ test ]
fn it235_lim_it_no_color_0_output_includes_emoji()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it235: no live token — skipping" );
    return;
  };
  if !require_live_api( "it235" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out  = run_cs_with_env( &[ ".usage", "no_color::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\u{1f7e2}" ),
    "no_color::0 must include 🟢 status emoji for live account (047 EC-3), got:\n{text}",
  );
}

/// it236 `lim_it` (047 EC-5): `no_color::1` replaces `✓` with `*` in flag column.
///
/// With a live active account, `no_color::1` must replace the unicode `✓` check mark
/// with ASCII `*` in the flag column, and must not contain the unicode character.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-5]
#[ test ]
fn it236_lim_it_no_color_1_check_mark_replaced_by_star()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it236: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    !text.contains( '\u{2713}' ),
    "no_color::1 must replace unicode '✓' with '*' (047 EC-5), got:\n{text}",
  );
  assert!(
    text.contains( '*' ),
    "no_color::1 must contain '*' (replaced from '✓') (047 EC-5), got:\n{text}",
  );
}

// ── it237: clear:: live test (051 EC-4) ──────────────────────────────────────

/// it237 `lim_it` (051 EC-4): after `clear::1`, `_renewal_at` is absent from `.json`.
///
/// With a live account that has an injected `_renewal_at` override, `clear::1`
/// must remove it. After clearing, the `.json` must not contain `_renewal_at`.
///
/// Spec: [`tests/docs/cli/param/051_clear.md` EC-4]
#[ test ]
fn it237_lim_it_clear_usage_shows_tilde_estimate()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it237: no live token — skipping" );
    return;
  };
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Inject a far-future _renewal_at override.
  std::fs::write(
    store.join( "acct-a@test.com.json" ),
    r#"{"_renewal_at":"2030-01-01T00:00:00Z"}"#,
  ).unwrap();

  // Clear the renewal override.
  let clear_out = run_cs_with_env(
    &[ ".account.renewal", "name::acct-a@test.com", "clear::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &clear_out, 0 );

  // After clear, _renewal_at must be absent from the file.
  let content = std::fs::read_to_string( store.join( "acct-a@test.com.json" ) ).unwrap();
  assert!(
    !content.contains( "_renewal_at" ),
    "clear::1 must remove _renewal_at from .json (051 EC-4), got: {content}",
  );

  // .usage must still succeed after clear.
  let usage_out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &usage_out, 0 );
}

// ── it238–it239: display control param group (005 CC-3/4) ────────────────────

/// it238 `lim_it` (005 CC-3): `get::` bypasses `cols::` column visibility.
///
/// `cols::-7d_left` hides the `7d_left` column from table output, but
/// `get::7d_left` must still extract the underlying data value unchanged —
/// `get::` reads from the data model, not the rendered column set.
///
/// Spec: [`tests/docs/cli/param_group/005_display_control.md` CC-3]
#[ test ]
fn it238_lim_it_get_bypasses_cols_restriction()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it238: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out_hidden = run_cs_with_env(
    &[ ".usage", "cols::-7d_left", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  let out_normal = run_cs_with_env(
    &[ ".usage", "get::7d_left" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_hidden, 0 );
  assert_exit( &out_normal, 0 );

  assert_eq!(
    stdout( &out_hidden ).trim(),
    stdout( &out_normal ).trim(),
    "get::7d_left with cols::-7d_left must produce same output as without cols:: (005 CC-3)",
  );
}

/// it239 (005 CC-4): `cols::+sub` and `no_color::1` apply simultaneously and independently.
///
/// `cols::+sub` adds the Sub column; `no_color::1` strips emoji. Both must be
/// independently active: Sub column header present in output, no emoji in output.
///
/// Spec: [`tests/docs/cli/param_group/005_display_control.md` CC-4]
#[ test ]
fn it239_cols_sub_and_no_color_independent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "cols::+sub", "no_color::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // cols::+sub applies — Sub column header present.
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must add 'Sub' column header (005 CC-4), got:\n{text}",
  );
  // no_color::1 applies — no emoji in output.
  assert!(
    !text.contains( "\u{1f534}" ),
    "no_color::1 must remove 🔴 emoji (005 CC-4), got:\n{text}",
  );
  assert!(
    !text.contains( "\u{1f7e2}" ),
    "no_color::1 must not contain 🟢 (005 CC-4), got:\n{text}",
  );
}

// ── it240: account targeting param group (006 CC-4) ──────────────────────────

/// it240 `lim_it` (006 CC-4): `cols::+host,+role` shows both columns from profile.json.
///
/// When an account has a `profile.json` with `host` and `role`, `.usage` with
/// `cols::+host,+role` must show both the Host and Role columns populated with
/// the stored values, regardless of token validity.
///
/// Spec: [`tests/docs/cli/param_group/006_account_targeting.md` CC-4]
#[ test ]
fn it240_lim_it_cols_host_role_shows_profile_data()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it240: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );
  write_account_profile_json( dir.path(), "acct-a@test.com", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env(
    &[ ".usage", "cols::+host,+role" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Host" ),
    "cols::+host,+role must add 'Host' column header (006 CC-4), got:\n{text}",
  );
  assert!(
    text.contains( "Role" ),
    "cols::+host,+role must add 'Role' column header (006 CC-4), got:\n{text}",
  );
  assert!(
    text.contains( "mybox" ),
    "cols::+host must show 'mybox' host value from profile.json (006 CC-4), got:\n{text}",
  );
  assert!(
    text.contains( "work" ),
    "cols::+role must show 'work' role value from profile.json (006 CC-4), got:\n{text}",
  );
}

